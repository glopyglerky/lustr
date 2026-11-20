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
            target = GUARD.PRODUCT_TARGETS[manifest]
            sources = {"build.rs", *target["bin"].values(), *(path for path, _ in target["example"].values())}
            if target["lib"]:
                sources.add(target["lib"][0])
            for relative in sources:
                path = root / Path(manifest).parent / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / Path(manifest).parent / relative, path)

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
        rewrite("Cargo.toml", lambda text: text.replace('source-native-runtime = ["exclusive-native"]',
                'source-native-runtime = ["selected-policy"]\nselected-policy = ["exclusive-native"]'))
        assert not checked(), checked()

        cases = [
            ("default-feature-sdk", "crates/lustr-native-core/Cargo.toml",
             lambda text: text.replace("[features]", '[features]\ndefault = ["upstream-semantic"]'), "product features enable research"),
            ("transitive-feature-sdk", "crates/lustr-native-core/Cargo.toml",
             lambda text: text.replace("exclusive-native = []", 'exclusive-native = ["ordinary"]\nordinary = ["upstream-semantic"]'), "product features enable research"),
            ("source-runtime-research", "Cargo.toml",
             lambda text: text.replace('source-native-runtime = ["exclusive-native"]', 'source-native-runtime = ["exclusive-native", "hidden"]\nhidden = ["native-qualification"]\nnative-qualification = []'), "product features enable research"),
            ("source-runtime-loses-product-policy", "Cargo.toml",
             lambda text: text.replace('source-native-runtime = ["exclusive-native"]', 'source-native-runtime = []'), "constructor feature closure omits exclusive-native"),
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
            ("entry-source-redirection", "Cargo.toml",
             lambda text: text.replace('path = "src/main.rs"', 'path = "crates/lustr-semantic/src/main.rs"'), "bin target sources"),
            ("implicit-library-redirection", "Cargo.toml",
             lambda text: text + '\n[lib]\npath = "unreviewed.rs"\n', "product library selection"),
            ("native-library-redirection", "crates/lustr-native-core/Cargo.toml",
             lambda text: text.replace('[lib]', '[lib]\npath = "../lustr-semantic/src/main.rs"'), "product library selection"),
            ("build-script-redirection", "Cargo.toml",
             lambda text: text.replace('[package]', '[package]\nbuild = "crates/lustr-semantic/build.rs"'), "build-script source"),
            ("build-script-disabled", "Cargo.toml",
             lambda text: text.replace('[package]', '[package]\nbuild = false'), "build-script source"),
            ("foreign-workspace", "crates/lustr-native-core/Cargo.toml",
             lambda text: text.replace('[package]', '[package]\nworkspace = "../lustr-semantic"'), "product workspace selection"),
            ("removed-workspace-exclusions", "Cargo.toml",
             lambda text: text.replace('exclude = ["compiler/native", "crates/lustr", "crates/lustr-semantic", "crates/lustr-native-core"]', 'exclude = []'), "product workspace selection"),
            ("implicit-library-disabled", "Cargo.toml",
             lambda text: text.replace('[package]', '[package]\nautolib = false'), "product library selection"),
            ("extra-native-entry", "crates/lustr-native-core/Cargo.toml",
             lambda text: text + '\n[[bin]]\nname = "compiler"\npath = "../lustr-semantic/src/main.rs"\n', "bin target sources"),
            ("library-procedural-macro", "Cargo.toml",
             lambda text: text + '\n[lib]\nproc-macro = true\n', "product library selection"),
            ("qualification-gate-removed", "crates/lustr-native-core/Cargo.toml",
             lambda text: text.replace('required-features = ["upstream-semantic"]', 'required-features = []'), "example target sources or admission features"),
            ("default-package-target", "Cargo.toml",
             lambda text: 'cargo-features = ["per-package-target"]\n' + text.replace('[package]', '[package]\ndefault-target = "wasm32-unknown-unknown"'), "manifest target or build-generation override"),
            ("forced-package-target", "Cargo.toml",
             lambda text: 'cargo-features = ["per-package-target"]\n' + text.replace('[package]', '[package]\nforced-target = "wasm32-unknown-unknown"'), "manifest target or build-generation override"),
            ("manifest-feature-activation", "Cargo.toml",
             lambda text: 'cargo-features = ["profile-rustflags"]\n' + text, "unreviewed Cargo feature gates"),
            ("profile-compiler-cfg", "crates/lustr-native-core/Cargo.toml",
             lambda text: 'cargo-features = ["profile-rustflags"]\n' + text + '\n[profile.dev]\nrustflags = [\'--cfg=feature="upstream-semantic"\']\n', "unreviewed compiler flags or backend"),
            ("profile-build-script-cfg", "Cargo.toml",
             lambda text: text + '\n[profile.dev.build-override]\nrustflags = [\'--cfg=feature="source-native-runtime"\']\n', "unreviewed compiler flags or backend"),
            ("profile-package-backend", "Cargo.toml",
             lambda text: text + '\n[profile.dev.package.lustr-compiler]\ncodegen-backend = "unreviewed"\n', "unreviewed compiler flags or backend"),
        ]
        for label, name, transform, diagnostic in cases:
            for original, text in originals.items():
                (root / original).write_text(text)
            rewrite(name, transform)
            failures = checked()
            assert any(diagnostic in failure for failure in failures), (label, failures)
        for original, text in originals.items():
            (root / original).write_text(text)
        extra_sources = [
            ("compiler/native/src/lib.rs", "unreviewed product library target"),
            ("crates/lustr-native-core/src/bin/compiler.rs", "bin target sources"),
            ("src/bin/hidden/main.rs", "bin target sources"),
            (".cargo/config.toml", "unreviewed package Cargo configuration"),
        ]
        for relative, diagnostic in extra_sources:
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("// unreviewed Cargo input\n")
            assert any(diagnostic in failure for failure in checked()), (relative, checked())
            path.unlink()
        build = root / "build.rs"
        original_build = build.read_bytes()
        build.unlink()
        build.symlink_to(root / "crates/lustr-native-core/build.rs")
        assert any("source is missing or redirected" in failure for failure in checked()), checked()
        build.unlink()
        build.write_bytes(original_build)
        assert not checked(), checked()
        return len(cases) + len(extra_sources) + 1


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
        for relative in ("crates/lustr-semantic/src/mixed_entry.rs", "crates/lustr/scripts/build-cranelift.py",
                         "src/native_expansion.rs", "library/lustr/std/src/platform.rs"):
            restored = root / relative
            restored.parent.mkdir(parents=True, exist_ok=True)
            restored.write_text("// restored retired route\n")
            assert checked() == ["retired whole-compiler route reappeared: " + relative]
            restored.unlink()
            assert not checked()
    return 4


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
            ("crates/lustr-metadata-encoder/Cargo.toml", lambda text: 'cargo-features = ["per-package-target"]\n' + text.replace('[package]', '[package]\nforced-target = "wasm32-unknown-unknown"'), "manifest target or build-generation override"),
            ("crates/lustr-metadata-sdk/Cargo.toml", lambda text: text + '\n[profile.dev]\nrustflags = [\'--cfg=feature="unreviewed"\']\n', "unreviewed compiler flags or backend"),
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
        shutil.copyfile(ROOT / "crates/lustr-semantic/build.rs", source.parent.parent / "build.rs")
        assert not checked()
        for restored in ("\nfn delegate() {}\n", "\nfn launch() { command.exec(); }\n"):
            source.write_text(original + restored)
            assert checked() == ["independent semantic inspection regained an unchecked compiler passthrough"]
        source.write_text(original)
        build = source.parent.parent / "build.rs"
        build.write_text(build.read_text() + '\n#[cfg(feature = "full-core")] fn other_compiler() {}\n')
        assert checked() == ["independent semantic inspection regained the retired full-core build mode"]
        return 3


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
    pin_path = ROOT / "upstream/bluefe-core.pin"

    def moved_pin(path, *args, **kwargs):
        text = original_read(path, *args, **kwargs)
        return text.replace(original["rust_source"], "4b5350e87e487aed08ab2f9f0160a4b2616438bc") if path == pin_path else text

    with patch.object(Path, "read_text", moved_pin):
        _, failures = GUARD.check(ROOT)
        assert any("selective imports and declared SDK source disagree" in failure for failure in failures), failures
    packages = product_package_controls()
    retired = retired_policy_control()
    metadata = metadata_research_controls()
    inspection = inspection_passthrough_controls()
    print(json.dumps({"original_passed": True, "tampered_inventories_rejected": len(mutations),
                      "product_package_mutations_rejected": packages, "unrelated_manifest_changes_admitted": True,
                      "restored_retired_policy_rejected": retired,
                      "metadata_research_mutations_rejected": metadata,
                      "inspection_passthrough_mutations_rejected": inspection,
                      "moving_donor_pin_rejected": True}))


if __name__ == "__main__":
    main()
