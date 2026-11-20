#!/usr/bin/env python3
"""Negative provenance controls for the deliberately structural import guard."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("guard", ROOT / "tools/check-exclusive-coverage.py")
GUARD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GUARD)


def product_package_controls():
    # Plant real Cargo manifest/lockfile changes. These exercise aliases,
    # feature reachability and transitive source admission without invoking a
    # compiler or making these checks part of compilation's hot path.
    with tempfile.TemporaryDirectory(prefix="lustr-product-admission-") as directory:
        root = Path(directory)
        originals = {}
        for manifest in GUARD.PRODUCT_PACKAGES:
            for name in [manifest, str(Path(manifest).with_name("Cargo.lock"))]:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, path)
                originals[name] = path.read_text()

        def checked():
            failures = []
            GUARD.product_package_checks(root, lambda condition, message: None if condition else failures.append(message))
            return failures

        def rewrite(name, transform):
            path = root / name
            path.write_text(transform(path.read_text()))

        assert not checked()
        # An unrelated profile change and a local empty feature alias do not
        # change the package source graph and must remain admissible.
        rewrite("compiler/native/Cargo.toml", lambda text: text.replace("exclusive-native = []", 'exclusive-native = ["selected"]\nselected = ["cycle"]\ncycle = ["selected"]'))
        rewrite("Cargo.toml", lambda text: text.replace('version = "0.1.0"', 'version = "0.1.1"').replace('opt-level = 3', 'opt-level = 2'))
        assert not checked(), checked()

        cases = [
            ("default-feature-sdk", "crates/lustr-native-core/Cargo.toml",
             lambda text: text.replace("[features]", '[features]\ndefault = ["upstream-semantic"]'), "product features enable research"),
            ("transitive-feature-sdk", "crates/lustr-native-core/Cargo.toml",
             lambda text: text.replace("exclusive-native = []", 'exclusive-native = ["ordinary"]\nordinary = ["upstream-semantic"]'), "product features enable research"),
            ("source-runtime-research", "Cargo.toml",
             lambda text: text.replace('source-native-runtime = ["exclusive-native"]', 'source-native-runtime = ["exclusive-native", "hidden"]\nhidden = ["native-qualification"]\nnative-qualification = []'), "product features enable research"),
            ("renamed-compiler-dependency", "Cargo.toml",
             lambda text: text.replace("[dependencies]", '[dependencies]\nhelper = { package = "rustc_driver", version = "0.1" }'), "unreviewed product dependency helper -> rustc_driver"),
            ("renamed-path-override", "Cargo.toml",
             lambda text: text.replace('serde_json = "=1.0.154"', 'json = { package = "serde_json", path = "crates/lustr-semantic" }'), "unreviewed source for dependency json"),
            ("dependency-default-activation", "compiler/native/Cargo.toml",
             lambda text: text.replace('default-features = false', 'default-features = true', 1), "unreviewed feature configuration for dependency mimalloc"),
            ("dependency-named-feature", "Cargo.toml",
             lambda text: text.replace('serde_json = "=1.0.154"', 'serde_json = { version = "=1.0.154", features = ["preserve_order"] }'), "unreviewed feature configuration for dependency serde_json"),
            ("dependency-made-optional", "Cargo.toml",
             lambda text: text.replace('serde_json = "=1.0.154"', 'serde_json = { version = "=1.0.154", optional = true }'), "unreviewed feature configuration for dependency serde_json"),
            ("build-compiler-dependency", "Cargo.toml",
             lambda text: text + '\n[build-dependencies]\ncompiler = { package = "rustc_driver", version = "0.1" }\n', "unreviewed build-dependencies edges"),
            ("target-compiler-dependency", "Cargo.toml",
             lambda text: text + '\n[target.\'cfg(unix)\'.dependencies]\ncompiler = { package = "rustc_driver", version = "0.1" }\n', "unreviewed target.cfg(unix).dependencies edges"),
            ("workspace-inherited-source", "Cargo.toml",
             lambda text: text.replace('serde_json = "=1.0.154"', 'serde_json = { workspace = true }') + '\n[workspace.dependencies]\nserde_json = { path = "crates/lustr-semantic" }\n', "unreviewed workspace dependency"),
            ("patch-source-override", "Cargo.toml",
             lambda text: text + '\n[patch.crates-io]\nserde_json = { path = "crates/lustr-semantic" }\n', "unreviewed dependency source override"),
            ("optional-activation", "Cargo.toml",
             lambda text: text.replace("exclusive-native = []", 'exclusive-native = ["dep:serde_json"]'), "unreviewed dependency activation"),
            ("dependency-feature-activation", "Cargo.toml",
             lambda text: text.replace("exclusive-native = []", 'exclusive-native = ["serde_json/hidden-compiler"]'), "unreviewed dependency activation"),
            ("unknown-implicit-dependency", "Cargo.toml",
             lambda text: text.replace("exclusive-native = []", 'exclusive-native = ["hidden-compiler"]'), "unreviewed implicit dependency"),
            ("transitive-package-addition", "Cargo.lock",
             lambda text: text + '\n[[package]]\nname = "ordinary-helper"\nversion = "0.1.0"\n', "lockfile package closure differs"),
            ("transitive-edge-addition", "Cargo.lock",
             lambda text: text.replace(' "itoa",', ' "itoa",\n "rustc_driver",'), "unreviewed lockfile dependency edges from serde_json"),
            ("transitive-source-override", "Cargo.lock",
             lambda text: text.replace('source = "registry+https://github.com/rust-lang/crates.io-index"', 'source = "git+https://example.invalid/compiler"', 1), "unreviewed locked package source"),
        ]
        for label, name, transform, diagnostic in cases:
            for original, text in originals.items():
                (root / original).write_text(text)
            rewrite(name, transform)
            failures = checked()
            assert any(diagnostic in failure for failure in failures), (label, failures)
        return len(cases)


def retired_policy_control():
    # Restore a real unused policy file, without importing or executing it.
    # Its presence must fail the same retired-path check used by the full gate.
    with tempfile.TemporaryDirectory(prefix="lustr-retired-policy-") as directory:
        root = Path(directory)

        def checked():
            failures = []
            GUARD.retired_path_checks(root, lambda condition, message: None if condition else failures.append(message))
            return failures

        assert not checked()
        relative = "crates/lustr-semantic/src/mixed_entry.rs"
        restored = root / relative
        restored.parent.mkdir(parents=True)
        restored.write_text("pub fn full_rust_only(_: &[String], _: &str) -> bool { true }\n")
        assert checked() == ["retired whole-compiler route reappeared: " + relative]
        restored.unlink()
        assert not checked()
    return 1


def metadata_research_controls():
    with tempfile.TemporaryDirectory(prefix="lustr-metadata-admission-") as directory:
        root = Path(directory)
        inventory = json.loads((ROOT / "upstream/complement-imports.json").read_text())
        for crate in ("lustr-metadata-encoder", "lustr-metadata-sdk"):
            shutil.copytree(ROOT / "crates" / crate, root / "crates" / crate,
                            ignore=shutil.ignore_patterns("target"))

        def checked():
            failures = []
            GUARD.metadata_research_checks(root, inventory, lambda condition, message: None if condition else failures.append(message))
            return failures

        assert not checked(), checked()
        cases = [
            ("crates/lustr-metadata-encoder/src/upstream/odht_writer.rs", lambda text: text + "\n// changed adapted source\n", "adapted sources differ"),
            ("crates/lustr-metadata-encoder/Cargo.toml", lambda text: text.replace("default-features = false", "default-features = true", 1), "dependency or oracle boundary changed"),
            ("crates/lustr-metadata-encoder/Cargo.toml", lambda text: text.replace("[dev-dependencies]", 'odht = "=0.3.1"\n\n[dev-dependencies]'), "dependency or oracle boundary changed"),
            ("crates/lustr-metadata-sdk/Cargo.toml", lambda text: text + '\n[features]\nhidden = []\n', "unreviewed metadata feature"),
            ("crates/lustr-metadata-encoder/source-slices.json", lambda text: text.replace('"native_caller_committed": false', '"native_caller_committed": true'), "overstates integration"),
            ("crates/lustr-metadata-encoder/Cargo.toml", lambda text: text + '\n[lib]\npath = "unreviewed.rs"\n', "library source or workspace selection changed"),
            ("crates/lustr-metadata-encoder/Cargo.toml", lambda text: text.replace('[workspace]', '[workspace]\ndefault-members = ["other"]'), "library source or workspace selection changed"),
            ("crates/lustr-metadata-sdk/Cargo.toml", lambda text: text.replace('name = "lustr-metadata-sdk"', 'name = "other"'), "library source or workspace selection changed"),
        ]
        for name, change, diagnostic in cases:
            path = root / name
            original = path.read_text()
            path.write_text(change(original))
            failures = checked()
            assert any(diagnostic in failure for failure in failures), (name, failures)
            path.write_text(original)
        extra = root / "crates/lustr-metadata-encoder/src/unreviewed.rs"
        extra.write_text("pub fn alternate_encoder() {}\n")
        assert any("adapted sources differ" in failure for failure in checked())
        extra.unlink()
        assert not checked(), checked()
        return len(cases) + 1


def inspection_passthrough_controls():
    relative = "crates/lustr-semantic/src/main.rs"
    with tempfile.TemporaryDirectory(prefix="lustr-inspection-admission-") as directory:
        root = Path(directory)
        source = root / relative
        source.parent.mkdir(parents=True)
        original = (ROOT / relative).read_text()

        def checked():
            failures = []
            GUARD.inspection_passthrough_checks(root, lambda condition, message: None if condition else failures.append(message))
            return failures

        source.write_text(original)
        assert not checked()
        for restored in ("\nfn delegate() {}\n", "\nfn launch() { command.exec(); }\n"):
            source.write_text(original + restored)
            assert checked() == ["independent semantic inspection regained an unchecked compiler passthrough"]
        return 2


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
        lambda data: data["research_components"].pop(),
        lambda data: data["research_components"][2].update(native_caller_committed=True),
    ]
    for mutate in mutations:
        candidate = copy.deepcopy(original)
        mutate(candidate)

        def read(path, *args, **kwargs):
            return json.dumps(candidate) if path == inventory_path else original_read(path, *args, **kwargs)

        with patch.object(Path, "read_text", read):
            _, failures = GUARD.check(ROOT)
            assert failures, "forged import provenance was admitted"
    packages = product_package_controls()
    retired = retired_policy_control()
    metadata = metadata_research_controls()
    inspection = inspection_passthrough_controls()
    print(json.dumps({"original_passed": True, "tampered_inventories_rejected": len(mutations),
                      "product_package_mutations_rejected": packages, "unrelated_manifest_changes_admitted": True,
                      "restored_retired_policy_rejected": retired,
                      "metadata_research_mutations_rejected": metadata,
                      "inspection_passthrough_mutations_rejected": inspection}))


if __name__ == "__main__":
    main()
