#!/usr/bin/env python3
"""Check the retained component's replay and reject reordered timing evidence."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("component", ROOT / "tools/replay-mixed-symbol-measurement.py")
component = importlib.util.module_from_spec(spec)
spec.loader.exec_module(component)
EVIDENCE = ROOT / "performance/components/2026-11-17-symbol-check"


class ComponentReplayTests(unittest.TestCase):
    def test_original_component_replays_all_pairs(self):
        result = component.replay(EVIDENCE)
        self.assertEqual((result["verified_pairs"], result["timed_values"]), (64, 128))

    def test_relabelled_digest_does_not_admit_reordered_execution(self):
        with tempfile.TemporaryDirectory(prefix="lustr-component-replay-") as directory:
            root = Path(directory) / "component"
            shutil.copytree(EVIDENCE, root)
            path = root / "samples.csv"
            rows = path.read_bytes().splitlines(keepends=True)
            rows[1], rows[2] = rows[2], rows[1]
            path.write_bytes(b"".join(rows))
            identity_path = root / "identity.json"
            identity = json.loads(identity_path.read_text())
            identity["samples_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
            identity_path.write_text(json.dumps(identity))
            with self.assertRaisesRegex(ValueError, "recorded execution order"):
                component.replay(root)


if __name__ == "__main__":
    unittest.main()
