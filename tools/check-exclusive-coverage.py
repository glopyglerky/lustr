#!/usr/bin/env python3
"""Structural guard for Lustr ownership and the reviewed import inventory.

This guard does not prove compiler semantics, minimal source closure or speed.
Actual process controls and source-bound compiler qualification remain required.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib


def check(root):
    failures = []

    def require(condition, message):
        if not condition:
            failures.append(message)

    inventory = json.loads((root / "upstream/complement-imports.json").read_text())
    require(inventory["schema"] == "lustr-complement-imports-v1", "unknown complement inventory schema")
    require(inventory["full_rust_coverage"] is False, "full Rust coverage needs an independent admission change")
    require(inventory["release_status"] == "unavailable", "full-replacement release remains unavailable")
    unicode = json.loads((root / "compiler/native/unicode-semantics.json").read_text())
    require(inventory["rust_source"] == unicode["rust_source"], "import inventory and native Unicode contract disagree")
    imports = {item["id"]: item for item in inventory["imports"]}
    require(set(imports) == {"identifier-normalization", "declared-sdk-unicode-data"}, "new import needs a reviewed necessity/ownership gate")
    normalization = imports.get("identifier-normalization", {})
    expected_dependencies = {name: unicode["compiler_dependencies"][name]
                             for name in unicode["native_identifier_normalization"]["dependencies"]}
    require(normalization.get("source_dependencies") == expected_dependencies,
            "normalization import identities disagree with the declared SDK provenance")
    require(normalization.get("upstream_paths") == unicode["native_identifier_normalization"]["upstream_sources"],
            "normalization donor source hashes disagree with the declared SDK provenance")
    require(normalization.get("dependency_edges") == [["unicode-normalization", "tinyvec"], ["tinyvec", "tinyvec_macros"]],
            "normalization dependency edges need review")
    generated = imports.get("declared-sdk-unicode-data", {})
    require(generated.get("generated_inputs") == unicode["generated_and_retained_data"],
            "Unicode import inventory differs from the checked generated-data identities")
    for lockfile in unicode["native_identifier_normalization"]["lockfiles"]:
        locked = {package["name"]: package for package in tomllib.loads((root / lockfile).read_text())["package"]}
        for name, identity in expected_dependencies.items():
            package = locked.get(name, {})
            require(package.get("version") == identity["version"] and package.get("checksum") == identity["crate_sha256"],
                    f"{lockfile}: {name} differs from its reviewed imported identity")
    for item in imports.values():
        for field in ["missing_capability", "owner", "semantic_boundary", "upstream_rust_source", "upstream_paths", "dependency_edges", "targets", "features", "adapter", "licenses", "excluded_duplicate_phases", "qualification"]:
            require(field in item, f"{item['id']}: missing {field}")
        require((root / item["adapter"]).is_file(), f"{item['id']}: missing adapter")
        require(item["upstream_rust_source"] == inventory["rust_source"], f"{item['id']}: imported Rust pin disagrees")
    for path, expected in unicode["generated_and_retained_data"].items():
        require(hashlib.sha256((root / path).read_bytes()).hexdigest() == expected, f"Unicode source changed without declared identity: {path}")
    require(bool(inventory["research_components"]), "full rustc SDK research dependencies must remain disclosed")
    for component in inventory["research_components"]:
        require((root / component["path"]).exists(), "missing disclosed research path: " + component["path"])
        require(component["production_admitted"] is False, "whole rustc SDK research is not a selective production import")
        require(bool(component["required_for"]) and bool(component["blocking_reason"]), "research closure needs its actual purpose and block")

    entry = (root / "src/main.rs").read_text()
    for token in ["rustc-full", "rustc-reference", "full-rust-core", "run_cli_with_default_root", "expanded_answer", "native-inference-worker"]:
        require(token not in entry, "public entry regained a retired route: " + token)
    require("integration-unimplemented" in entry, "unintegrated work must be explicit")
    require('var_os("LUST_PROBE_FAKE_DIR").is_some()' in entry, "external native probe modes must be refused, including empty values")
    require("Invocation::ReleaseCc" in entry, "native release must use the actual parsed request")
    require("run_program" in entry, "native run must remain reachable")
    for path in ["src/full.rs", "tools/assemble-mixed-toolchain.rs", "tools/build-mixed-toolchain.rs", "tools/native-relink.rs", "crates/lustr-semantic/src/mixed_main.rs"]:
        require(not (root / path).exists(), "retired whole-compiler route reappeared: " + path)
    cargo = tomllib.loads((root / "Cargo.toml").read_text())
    require("cargo_lust" not in cargo["dependencies"], "public compiler must not depend on the retired coordinator")
    semantic = tomllib.loads((root / "crates/lustr-semantic/Cargo.toml").read_text())
    require(not any(item.get("name") == "lustr-mixed" for item in semantic.get("bin", [])), "whole-rustc driver is a product target again")
    coordinator = (root / "crates/lustr/src/lib.rs").read_text()
    require("fn execute(" not in coordinator and ".exec()" not in coordinator, "retired coordinator must not execute compiler plans")
    builder = (root / "tools/build-native-toolchain.rs").read_text()
    for token in ["rustc-full", "rustc-reference", "lustr-mixed", "cargo-lustr", 'read_dir(sdk.join("lib"))']:
        require(token not in builder, "native builder packages whole-compiler dependencies: " + token)
    require("PARTIAL-NATIVE-COMPILER" in builder, "native build must state incomplete complementary coverage")
    require(builder.count('"--features", "exclusive-native"') >= 2, "both native executables and the library require the no-delegation feature")
    root_build = (root / "build.rs").read_text()
    require("cargo:rustc-env=LUST_EXCLUSIVE_NATIVE=1" in root_build, "entry must require guarded-runtime receipt")
    require('pin["lust_build"]' not in root_build, "product entry must not bind the unguarded historical runtime")
    for path in ["compiler/native/Cargo.toml", "crates/lustr-native-core/Cargo.toml"]:
        require("exclusive-native" in tomllib.loads((root / path).read_text())["features"], path + ": no product native policy feature")
    for path in (root / "src").rglob("*.rs"):
        source = path.read_text()
        require("rustc_private" not in source and not re.search(r"extern\s+crate\s+rustc_", source), "product source imports full rustc SDK: " + str(path.relative_to(root)))
    for path in ["compiler/rustc_driver", "compiler/rustc_driver_impl", "compiler/rustc_interface", "src/bootstrap"]:
        require(not (root / path).exists(), "whole upstream compiler source belongs in blueFe: " + path)
    return inventory, failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    inventory, failures = check(args.repository)
    if args.release:
        failures.append("complete Rust replacement is unavailable: selective complementary coverage and its qualification remain incomplete")
    print(json.dumps({"schema": "lustr-exclusive-coverage-guard-v1", "structural_checks_passed": not failures,
                      "full_rust_coverage": inventory["full_rust_coverage"], "release_status": inventory["release_status"],
                      "scope": inventory["guard_scope"], "failures": failures}, indent=2))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
