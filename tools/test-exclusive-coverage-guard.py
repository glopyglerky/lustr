#!/usr/bin/env python3
"""Negative provenance controls for the deliberately structural import guard."""
import copy
import importlib.util
import json
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("guard", ROOT / "tools/check-exclusive-coverage.py")
GUARD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GUARD)


def main():
    inventory_path = ROOT / "upstream/complement-imports.json"
    original_read = Path.read_text
    original = json.loads(inventory_path.read_text())
    _, failures = GUARD.check(ROOT)
    assert not failures, failures
    mutations = [
        lambda data: data["imports"][0]["source_dependencies"]["tinyvec"].update(crate_sha256="0" * 64),
        lambda data: data["imports"][0]["upstream_paths"].update({"compiler/rustc_parse/src/lexer/mod.rs": "0" * 64}),
        lambda data: data["imports"][1]["generated_inputs"].update({"compiler/native/src/unicode_data.rs": "0" * 64}),
        lambda data: data["imports"][0].update(upstream_rust_source="0" * 40),
        lambda data: data["research_components"][0].update(production_admitted=True),
        lambda data: data.update(full_rust_coverage=True),
    ]
    for mutate in mutations:
        candidate = copy.deepcopy(original)
        mutate(candidate)

        def read(path, *args, **kwargs):
            return json.dumps(candidate) if path == inventory_path else original_read(path, *args, **kwargs)

        with patch.object(Path, "read_text", read):
            _, failures = GUARD.check(ROOT)
            assert failures, "forged import provenance was admitted"
    print(json.dumps({"original_passed": True, "tampered_inventories_rejected": len(mutations)}))


if __name__ == "__main__":
    main()
