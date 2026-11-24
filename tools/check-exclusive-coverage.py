#!/usr/bin/env python3
"""Structural guard for Lustr ownership and the reviewed import inventory.

This guard does not prove compiler semantics, minimal source closure or speed.
Actual process controls and source-bound compiler qualification remain required.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tomllib


# These are the dependency edges of the three packages the native constructor
# actually builds. They are deliberately not hashes of entire manifests: source,
# descriptions, profiles and ordinary version updates do not need a new gate.
# An added package, source override or dependency edge does need source/import
# review. Cargo.lock is an over-approximation across targets and features, not a
# proof of the feature-specific minimum or of semantic ownership.
PRODUCT_PACKAGES = {
    "Cargo.toml": ("lustr-compiler", {"serde_json", "sha2"}, {"source-native-runtime"}),
    "compiler/native/Cargo.toml": ("lust", {"mimalloc", "libmimalloc-sys", "unicode-normalization", "lustr-metadata-encoder", "lustr-metadata-sdk", "object", "serde_json", "sha2"}, {"exclusive-native"}),
    "crates/lustr-native-core/Cargo.toml": ("lustr-native-core", {"mimalloc", "libmimalloc-sys", "unicode-normalization", "lustr-metadata-encoder", "lustr-metadata-sdk", "object", "serde_json", "sha2"}, {"exclusive-native"}),
}
# Bind the files Cargo actually compiles, including its implicit library and
# binary discovery. Dependency checks alone cannot establish this: a manifest
# can keep the same dependency graph while replacing its executable or build
# script with a different compiler entry.
PRODUCT_TARGETS = {
    "Cargo.toml": {
        "workspace": {"resolver": "3", "exclude": ["compiler/native", "crates/lustr", "crates/lustr-semantic", "crates/lustr-native-core"]},
        "lib": ("src/lib.rs", ["lib"]),
        "bin": {"lustr": "src/main.rs"},
        "example": {"profile-native-preflight": ("tools/profile-native-preflight.rs", ["source-native-runtime"]),
                    "native_package": ("examples/native_package.rs", []),
                    "native_notices": ("examples/native_notices.rs", [])},
    },
    "compiler/native/Cargo.toml": {
        "workspace": {}, "lib": None,
        "bin": {"lust": "src/client.rs", "lust-core": "src/main.rs"}, "example": {},
    },
    "crates/lustr-native-core/Cargo.toml": {
        "workspace": {"resolver": "3"}, "lib": ("src/lib.rs", ["cdylib"]), "bin": {},
        "example": {name: (f"../../tools/{name}.rs", [feature]) for name, feature in [
            ("native-upstream-qualification", "upstream-semantic"),
            ("native-output-layout-qualification", "native-qualification"),
            ("native-phase-qualification", "native-qualification"),
            ("native-crate-qualification", "native-qualification"),
            ("native-consumer-qualification", "native-qualification"),
            ("host-proc-macro", "upstream-semantic"),
        ]},
    },
}
REVIEWED_DEPENDENCY_EDGES = {
    "serde_json": {"itoa", "memchr", "serde", "serde_core", "zmij"},
    "serde": {"serde_core"}, "serde_core": {"serde_derive"},
    "serde_derive": {"proc-macro2", "quote", "syn"},
    "proc-macro2": {"unicode-ident"}, "quote": {"proc-macro2"},
    "syn": {"proc-macro2", "quote", "unicode-ident"},
    "sha2": {"cfg-if", "cpufeatures", "digest"}, "cpufeatures": {"libc"},
    "digest": {"block-buffer", "const-oid", "crypto-common"},
    "block-buffer": {"hybrid-array"}, "crypto-common": {"hybrid-array"},
    "hybrid-array": {"typenum"},
    "mimalloc": {"libmimalloc-sys"}, "libmimalloc-sys": {"cc"},
    "cc": {"find-msvc-tools", "shlex"},
    "unicode-normalization": {"tinyvec"}, "tinyvec": {"tinyvec_macros"},
}
RESEARCH_FEATURES = {"native-inference", "native-qualification", "upstream-semantic"}
REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
# Every current direct package is required and selects no extra named features.
# Both native allocator packages explicitly disable their defaults. Enabling a
# different configuration requires review even if Cargo.lock's package names
# happen not to change.
NO_DEFAULT_FEATURES = {"mimalloc", "libmimalloc-sys"}


METADATA_PRODUCT_ROOTS = {"compiler/native/Cargo.toml", "crates/lustr-native-core/Cargo.toml"}

# Component paths are schema identities, not list positions. Keep research and
# selective product disclosures separate and reject undeclared provenance fields
# instead of silently treating contradictory claims as harmless annotations.
COMPONENT_BASE_FIELDS = {"path", "scope", "dependency", "production_admitted", "required_for", "blocking_reason"}
SELECTIVE_METADATA_FIELDS = COMPONENT_BASE_FIELDS | {
    "source_inventory", "native_caller_committed", "native_caller_path", "public_retained_integration",
    "qualification_record", "retained_native_caller_authored", "retained_native_qualification",
    "public_native_caller_path", "production_source_authored",
}
COMPONENT_SCHEMAS = {
    "research_components": {
        "crates/lustr-native-core/src/upstream_definitions.rs": COMPONENT_BASE_FIELDS,
        "crates/lustr-semantic": COMPONENT_BASE_FIELDS,
        "crates/lustr-native-metadata-qualification": COMPONENT_BASE_FIELDS | {
            "source_inventory", "public_retained_integration", "qualification_record",
            "retained_native_caller_authored", "retained_native_qualification",
        },
    },
    "selective_components": {
        "crates/lustr-metadata-encoder": SELECTIVE_METADATA_FIELDS,
        "crates/lustr-metadata-sdk": SELECTIVE_METADATA_FIELDS,
    },
}


def component_schema_checks(inventory, require):
    for collection, schema in COMPONENT_SCHEMAS.items():
        records = inventory.get(collection, [])
        paths = [record.get("path") for record in records if isinstance(record, dict)]
        valid_paths = len(paths) == len(records) and all(isinstance(path, str) for path in paths)
        require(valid_paths and len(paths) == len(set(paths)) and set(paths) == set(schema),
                collection + ": component identities changed or repeat a path")
        for record in records:
            path = record.get("path") if isinstance(record, dict) else None
            require(isinstance(path, str) and path in schema and set(record) == schema[path],
                    str(path) + ": undeclared or missing component provenance fields")


def metadata_product_dependencies(manifest):
    prefix = "../../crates" if manifest.startswith("compiler/") else ".."
    return {
        "lustr-metadata-encoder": {"path": prefix + "/lustr-metadata-encoder", "optional": True},
        "lustr-metadata-sdk": {"path": prefix + "/lustr-metadata-sdk", "optional": True},
        "object": {"version": "=0.39.1", "default-features": False, "features": ["write_std", "read_core", "elf", "archive"], "optional": True},
        "serde_json": {"version": "=1.0.154", "optional": True},
        "sha2": {"version": "=0.11.1", "optional": True},
    }


def retired_path_checks(root, require):
    for path in ["src/full.rs", "tools/assemble-mixed-toolchain.rs", "tools/build-mixed-toolchain.rs",
                 "tools/native-relink.rs", "crates/lustr-semantic/src/mixed_main.rs",
                 "crates/lustr-semantic/src/mixed_entry.rs",
                 "crates/lustr/scripts/build-cranelift.py", "crates/lustr/scripts/build-cranelift.sh",
                 "src/native_expansion.rs", "library/lustr/std/src/env.rs", "library/lustr/std/src/os_string.rs",
                 "library/lustr/std/src/path_capacity.rs", "library/lustr/std/src/path_from.rs", "library/lustr/std/src/platform.rs"]:
        require(not (root / path).exists(), "retired whole-compiler route reappeared: " + path)


def inspection_passthrough_checks(root, require):
    inspection = (root / "crates/lustr-semantic/src/main.rs").read_text()
    require("fn delegate(" not in inspection and ".exec()" not in inspection,
            "independent semantic inspection regained an unchecked compiler passthrough")
    build = (root / "crates/lustr-semantic/build.rs").read_text()
    require('feature = "full-core"' not in build,
            "independent semantic inspection regained the retired full-core build mode")


def product_target_checks(root, manifest, data, require):
    directory = (root / manifest).parent
    package = data["package"]
    expected = PRODUCT_TARGETS[manifest]
    require(data.get("workspace", {}) == expected["workspace"]
            and "workspace" not in package,
            f"{manifest}: product workspace selection changed without review")
    require(package.get("build", "build.rs") in (True, "build.rs"),
            f"{manifest}: product build-script source changed without review")
    require("links" not in package,
            f"{manifest}: unreviewed native linkage override")

    def source(relative):
        path = directory / relative
        require(path.is_file() and not path.is_symlink()
                and path.resolve() == Path(os.path.abspath(path))
                and path.resolve().is_relative_to(root.resolve()),
                f"{manifest}: reviewed Cargo source is missing or redirected: {relative}")

    source("build.rs")
    for name in ("config", "config.toml"):
        path = directory / ".cargo" / name
        require(not path.exists() and not path.is_symlink(),
                f"{manifest}: unreviewed package Cargo configuration")
    library = data.get("lib")
    actual_library = library is not None or (package.get("autolib", True) and (directory / "src/lib.rs").is_file())
    if expected["lib"] is None:
        require(not actual_library, f"{manifest}: unreviewed product library target")
    else:
        path, types = expected["lib"]
        library = library or {}
        require(actual_library and library.get("path", "src/lib.rs") == path
                and library.get("name", package["name"].replace("-", "_")) == package["name"].replace("-", "_")
                and library.get("crate-type", ["lib"]) == types
                and not library.get("proc-macro", False)
                and not library.get("required-features"),
                f"{manifest}: product library selection changed without review")
        source(path)

    def discovered(folder):
        base = directory / folder
        return {path.stem: str(path.relative_to(directory)) for path in base.glob("*.rs")} | {
            path.parent.name: str(path.relative_to(directory)) for path in base.glob("*/main.rs")}

    for kind, folder, automatic in (("bin", "src/bin", "autobins"), ("example", "examples", "autoexamples")):
        records = data.get(kind, [])
        actual = {}
        paths = set()
        for record in records:
            name = record.get("name")
            path = record.get("path")
            require(name not in actual and isinstance(name, str) and isinstance(path, str),
                    f"{manifest}: ambiguous or implicit {kind} target declaration")
            actual[name] = (path, record.get("required-features", []))
            paths.add(path)
        if package.get(automatic, True):
            implicit = discovered(folder)
            if kind == "bin" and (directory / "src/main.rs").is_file():
                implicit.setdefault(package["name"], "src/main.rs")
            for name, path in implicit.items():
                if name not in actual and path not in paths:
                    actual[name] = (path, [])
        targets = {name: (path, []) for name, path in expected[kind].items()} if kind == "bin" else expected[kind]
        require(actual == targets, f"{manifest}: {kind} target sources or admission features changed without review")
        for path, _ in targets.values():
            source(path)
    require(package.get("default-run") in (None, *expected["bin"]),
            f"{manifest}: unreviewed default executable selection")


def construction_mode_checks(manifest, data, require):
    require(not data.get("cargo-features"), f"{manifest}: unreviewed Cargo feature gates")
    require(not any(key in data["package"] for key in ("default-target", "forced-target", "metabuild")),
            f"{manifest}: unreviewed manifest target or build-generation override")

    def profiles(table, name="profile"):
        for key, value in table.items():
            require(key not in ("rustflags", "codegen-backend"),
                    f"{manifest}: unreviewed compiler flags or backend in {name}.{key}")
            if isinstance(value, dict):
                profiles(value, name + "." + key)

    # Profile cfg injection can activate a different source composition without
    # setting Cargo's CARGO_FEATURE_* build-script admission variables.
    profiles(data.get("profile", {}))


def product_package_checks(root, require):
    for manifest, (package_name, direct, selected) in PRODUCT_PACKAGES.items():
        data = tomllib.loads((root / manifest).read_text())
        require(data["package"]["name"] == package_name, f"{manifest}: unreviewed product package identity")
        product_target_checks(root, manifest, data, require)
        construction_mode_checks(manifest, data, require)
        require(not data.get("patch") and not data.get("replace"), f"{manifest}: unreviewed dependency source override")
        workspace = data.get("workspace", {})
        require(not workspace.get("dependencies") and not workspace.get("members")
                and not workspace.get("default-members"), f"{manifest}: unreviewed workspace dependency or build member")
        scopes = [("", data)] + [(f"target.{target}.", table) for target, table in data.get("target", {}).items()]
        actual = set()
        for prefix, table in scopes:
            for kind in ("dependencies", "build-dependencies", "dev-dependencies"):
                dependencies = table.get(kind, {})
                require(not dependencies or (not prefix and kind == "dependencies"),
                        f"{manifest}: unreviewed {prefix}{kind} edges")
                for alias, dependency in dependencies.items():
                    declaration = {"version": dependency} if isinstance(dependency, str) else dependency
                    name = declaration.get("package", alias)
                    if not prefix and kind == "dependencies":
                        actual.add(name)
                    require(name in direct, f"{manifest}: unreviewed product dependency {alias} -> {name}")
                    metadata_dependencies = metadata_product_dependencies(manifest) if manifest in METADATA_PRODUCT_ROOTS else {}
                    if name in metadata_dependencies:
                        require(alias == name and declaration == metadata_dependencies[name],
                                f"{manifest}: unreviewed selective metadata dependency declaration {alias}")
                    else:
                        require(not any(key in declaration for key in ("path", "git", "registry", "registry-index", "workspace")),
                                f"{manifest}: unreviewed source for dependency {alias}")
                        require(declaration.get("default-features", True) == (name not in NO_DEFAULT_FEATURES)
                                and declaration.get("optional", False) is False
                                and declaration.get("features", []) == [],
                                f"{manifest}: unreviewed feature configuration for dependency {alias}")
        require(actual == direct, f"{manifest}: product dependency roots differ from the reviewed graph")

        # Cargo enables defaults unless explicitly disabled by the constructor.
        # Follow every local feature edge, including aliases, before deciding
        # whether a product build can acquire the separate research compiler.
        features = data.get("features", {})
        require(selected <= set(features), f"{manifest}: constructor product features are not declared")
        pending = list(selected | {"default"})
        enabled = set()
        activated_dependencies = set()
        while pending:
            feature = pending.pop()
            if feature in enabled:
                continue
            enabled.add(feature)
            require(feature not in RESEARCH_FEATURES, f"{manifest}: product features enable research compiler {feature}")
            for edge in features.get(feature, []):
                admitted_metadata_edge = (manifest in METADATA_PRODUCT_ROOTS and feature == "exclusive-native"
                                          and edge.startswith("dep:") and edge[4:] in metadata_product_dependencies(manifest))
                if admitted_metadata_edge:
                    activated_dependencies.add(edge)
                require(admitted_metadata_edge or (not edge.startswith("dep:") and "/" not in edge),
                        f"{manifest}: product feature has an unreviewed dependency activation {edge}")
                if not edge.startswith("dep:") and "/" not in edge:
                    require(edge in features, f"{manifest}: product feature references unreviewed implicit dependency {edge}")
                    pending.append(edge)
        require("exclusive-native" in enabled,
                f"{manifest}: constructor feature closure omits exclusive-native")

        locked = tomllib.loads((root / manifest).with_name("Cargo.lock").read_text())["package"]
        if manifest in METADATA_PRODUCT_ROOTS:
            closure = json.loads((root / "upstream/metadata-dependency-closure.json").read_text())
            expected_packages = closure["packages"]
            actual_packages = [item for item in locked if item["name"] != package_name]
            identity = lambda item: (item["name"], item["version"])
            require(sorted(actual_packages, key=identity) == sorted(expected_packages, key=identity),
                    f"{manifest}: exact selective dependency identities or edges changed")
            roots = [item for item in locked if item["name"] == package_name]
            require(len(roots) == 1 and set(roots[0].get("dependencies", [])) == direct and "source" not in roots[0],
                    f"{manifest}: selected product root dependency cut changed")
            # Local feature aliases, including empty cycles, do not change the
            # selected dependency cut. Required activations still belong only
            # to exclusive-native; the traversal rejects every other edge.
            require(activated_dependencies == {"dep:" + name for name in metadata_product_dependencies(manifest)},
                    f"{manifest}: selective product feature closure changed")
            continue
        packages = {item["name"]: item for item in locked}
        require(len(packages) == len(locked), f"{manifest}: multiple package versions need dependency-identity review")
        expected = {package_name}
        pending = list(direct)
        while pending:
            name = pending.pop()
            if name not in expected:
                expected.add(name)
                pending.extend(REVIEWED_DEPENDENCY_EDGES.get(name, set()))
        require(set(packages) == expected, f"{manifest}: lockfile package closure differs from the reviewed graph")
        for name, package in packages.items():
            dependencies = {edge.split()[0] for edge in package.get("dependencies", [])}
            require(dependencies == (direct if name == package_name else REVIEWED_DEPENDENCY_EDGES.get(name, set())),
                    f"{manifest}: unreviewed lockfile dependency edges from {name}")
            if name == package_name:
                require("source" not in package, f"{manifest}: product package unexpectedly comes from another source")
            else:
                require(package.get("source") == REGISTRY_SOURCE and re.fullmatch(r"[0-9a-f]{64}", package.get("checksum", "")),
                        f"{manifest}: unreviewed locked package source {name}")


def metadata_research_checks(root, inventory, require):
    """Bind the selective research cut without admitting it to production.

    The donor hashes are provenance records, not re-fetched donor verification.
    Adapted file hashes bind those records to the actual local source. The
    separate bare-metal CI compile verifies the core's effective dependency cut.
    The separate caller remains a research executable; these source checks do
    not establish its actual metadata compatibility or product integration.
    """
    component_schema_checks(inventory, require)
    encoder = "crates/lustr-metadata-encoder"
    sdk = "crates/lustr-metadata-sdk"
    qualifier = "crates/lustr-native-metadata-qualification"
    components = {item["path"]: item for item in inventory["selective_components"] + inventory["research_components"]}
    for path in (encoder, sdk):
        component = components.get(path, {})
        require(component.get("source_inventory") == encoder + "/source-slices.json",
                path + ": selective research source inventory is missing")
        require(component.get("production_admitted") is False
                and component.get("native_caller_committed") is True
                and component.get("production_source_authored") is True
                and component.get("native_caller_path") == qualifier
                and component.get("retained_native_caller_authored") is True
                and component.get("public_retained_integration") is True
                and component.get("public_native_caller_path") == "crates/lustr-native-core/src/public_library.rs",
                path + ": selective metadata public caller boundary changed")

    source_root = root / encoder
    slices = json.loads((source_root / "source-slices.json").read_text())
    require(slices.get("schema") == "lustr-original-source-slices-v1"
            and slices.get("revision") == inventory["rust_source"],
            "metadata source inventory differs from the declared Rust identity")
    status = slices.get("integration_status", {})
    require(status.get("production_admitted") is False
            and status.get("native_caller_committed") is True
            and status.get("native_caller_path") == qualifier
            and status.get("retained_native_caller_authored") is True
            and status.get("public_retained_integration") is True
            and status.get("public_native_caller_path") == "crates/lustr-native-core/src/public_library.rs",
            "metadata source inventory misstates integration")
    actual = {str(path.relative_to(source_root)): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in (source_root / "src").rglob("*.rs")}
    require(slices.get("adapted_sources_sha256") == actual,
            "metadata adapted sources differ from their reviewed inventory")
    require(bool(slices.get("sources")) and bool(slices.get("registry_sources")),
            "metadata source inventory omits its donor slices")
    for item in slices.get("sources", []) + slices.get("registry_sources", []):
        require(all(item.get(field) for field in
                    ("source", "source_sha256", "destination", "adaptation", "necessity", "semantic_owner")),
                "metadata donor slice lacks source, adaptation or ownership")
        require(re.fullmatch(r"[0-9a-f]{64}", item.get("source_sha256", "")) is not None
                and item.get("destination") in actual,
                "metadata donor slice has an invalid identity or unbound destination")
        for license_file in item.get("license_files", []):
            path = source_root / license_file
            require(path.is_file() and path.resolve().is_relative_to(source_root.resolve()),
                    "metadata donor license is missing or outside the source cut")
    require(slices.get("no_std_scope", {}).get("qualification_target") == "x86_64-unknown-none"
            and "#![no_std]" in (source_root / "src/lib.rs").read_text(),
            "metadata serializer no_std boundary changed without review")

    # The core's original std-dependent hashing/map crates remain independent
    # dev-only oracles. The SDK loader is the sole host service adapter here.
    expected = {
        encoder: ({"md-5": {"version": "=0.10.6", "default-features": False},
                   "twox-hash": {"version": "=2.1.2", "default-features": False,
                                 "features": ["alloc", "xxhash3_128"]}},
                  {"odht": "=0.3.1", "rustc-stable-hash": "=0.1.2"}),
        sdk: ({"lustr-metadata-encoder": {"path": "../lustr-metadata-encoder"},
               "serde_json": "=1.0.154", "sha2": "=0.11.1"}, {}),
    }
    for relative, (dependencies, dev_dependencies) in expected.items():
        manifest = tomllib.loads((root / relative / "Cargo.toml").read_text())
        construction_mode_checks(relative + "/Cargo.toml", manifest, require)
        require(manifest["package"].get("name") == Path(relative).name
                and manifest.get("lib", {}) in ({}, {"path": "src/lib.rs"})
                and manifest.get("workspace", {}) == {"resolver": "3"},
                relative + ": metadata package, library source or workspace selection changed without review")
        require(manifest.get("dependencies", {}) == dependencies
                and manifest.get("dev-dependencies", {}) == dev_dependencies,
                relative + ": metadata dependency or oracle boundary changed without review")
        require(not any(manifest.get(key) for key in
                        ("features", "target", "build-dependencies", "patch", "replace"))
                and "build" not in manifest["package"]
                and not (root / relative / "build.rs").exists(),
                relative + ": unreviewed metadata feature, target or construction dependency")

    component = components.get(qualifier, {})
    require(component.get("production_admitted") is False
            and component.get("retained_native_caller_authored") is True
            and component.get("public_retained_integration") is False
            and component.get("source_inventory") == qualifier + "/source-slices.json",
            "native metadata qualifier must be an indexed research component")
    manifest_path = root / qualifier / "Cargo.toml"
    manifest = tomllib.loads(manifest_path.read_text())
    construction_mode_checks(str(manifest_path), manifest, require)
    require(manifest.get("package", {}).get("name") == "lustr-native-metadata-qualification"
            and manifest["package"].get("autobins") is False
            and manifest.get("workspace") == {"resolver": "3"}
            and manifest.get("bin") == [{"name": "lustr-native-metadata-qualification", "path": "src/main.rs", "required-features": ["native-qualification"]}]
            and manifest.get("features") == {"default": ["native-qualification"], "native-qualification": []},
            "native metadata qualifier source or research mode changed without review")
    dependencies = {
        "lustr-metadata-encoder": {"path": "../lustr-metadata-encoder"},
        "lustr-metadata-sdk": {"path": "../lustr-metadata-sdk"},
        "mimalloc": {"version": "=0.1.52", "default-features": False},
        "libmimalloc-sys": {"version": "=0.1.49", "default-features": False},
        "unicode-normalization": "=0.1.25",
        "object": {"version": "=0.39.1", "default-features": False, "features": ["write_std", "read_core", "elf", "archive"]},
        "serde_json": "=1.0.154", "sha2": "=0.11.1",
    }
    require(manifest.get("dependencies") == dependencies
            and not any(manifest.get(key) for key in
                        ("lib", "example", "test", "bench", "target", "build-dependencies", "dev-dependencies", "patch", "replace"))
            and manifest["package"].get("build", "build.rs") == "build.rs",
            "native metadata qualifier dependency or target closure changed without review")
    record = json.loads((root / qualifier / "source-slices.json").read_text())
    require(record.get("revision") == inventory["rust_source"]
            and record.get("production_admitted") is False
            and record.get("retained_native_caller_authored") is True,
            "native metadata qualifier source identity or product boundary changed")
    files = ["Cargo.toml", "Cargo.lock", "build.rs"] + sorted(
        str(path.relative_to(root / qualifier)) for path in (root / qualifier / "src").rglob("*.rs"))
    shared = ["../lustr-native-core/src/library_history.rs", "../lustr-native-core/src/library_artifact.rs", "../lustr-native-core/src/metadata_archive.rs",
              "../lustr-native-core/src/unit_object.rs", "../lustr-native-core/src/unit_publication.rs"]
    files += shared
    actual = {path: hashlib.sha256((root / qualifier / path).read_bytes()).hexdigest() for path in files}
    require(record.get("adapted_sources_sha256") == actual,
            "native metadata qualifier adapted sources differ from reviewed identities")
    for source in record.get("sources", []):
        require(source.get("destination") in actual
                and all(source.get(field) for field in ("source", "source_sha256", "adaptation", "necessity", "semantic_owner")),
                "native metadata qualifier lacks original source provenance")
    require(bool(record.get("sources")) and all((root / qualifier / name).is_file() for name in ["LICENSE-MIT", "LICENSE-APACHE"]),
            "native metadata qualifier original source/license records are absent")


def check(root):
    failures = []

    def require(condition, message):
        if not condition:
            failures.append(message)

    product_package_checks(root, require)
    inventory = json.loads((root / "upstream/complement-imports.json").read_text())
    require(inventory["schema"] == "lustr-complement-imports-v1", "unknown complement inventory schema")
    require(inventory["full_rust_coverage"] is False, "full Rust coverage needs an independent admission change")
    require(inventory["release_status"] == "unavailable", "full-replacement release remains unavailable")
    sdk = tomllib.loads((root / "upstream/bluefe-core.pin").read_text())
    require(inventory["rust_source"] == sdk["source"],
            "selective imports and declared SDK source disagree; moving donor branches cannot change Lustr semantics")
    unicode = json.loads((root / "compiler/native/unicode-semantics.json").read_text())
    require(inventory["rust_source"] == unicode["rust_source"], "import inventory and native Unicode contract disagree")
    imports = {item["id"]: item for item in inventory["imports"]}
    require(set(imports) == {"identifier-normalization", "declared-sdk-unicode-data", "native-scalar-metadata",
                            "native-scalar-artifact-host", "native-libtest-runtime", "host-proc-macro-compiler",
                            "proc-macro-expansion", "external-scalar-interface", "freestanding-executable-composition"},
            "new imports require source and ownership declarations")
    macro_expansion = imports.get("proc-macro-expansion", {}).get("closure_sha256", {})
    require(set(macro_expansion) == {
        "upstream/proc-macro-expansion.patch",
        "tools/proc-macro-expansion/client/Cargo.toml", "tools/proc-macro-expansion/client/Cargo.lock",
        "tools/proc-macro-expansion/server/Cargo.toml", "tools/proc-macro-expansion/server/Cargo.lock",
    }, "proc-macro expansion requires a pinned source and dependency closure")
    for path, digest in macro_expansion.items():
        require(hashlib.sha256((root / path).read_bytes()).hexdigest() == digest,
                "proc-macro expansion closure identity changed: " + path)
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
    libtest = imports.get("native-libtest-runtime", {})
    require(libtest.get("adapter") == "tools/native-test-runtime.rs"
            and libtest.get("dependency_edges") == [["native-test-runtime", "libtest"], ["libtest", "std"]]
            and libtest.get("excluded_duplicate_phases") == imports.get("native-scalar-artifact-host", {}).get("excluded_duplicate_phases"),
            "the native test adapter may import the test runtime but must keep compilation in Lust")
    for path, expected in unicode["generated_and_retained_data"].items():
        require(hashlib.sha256((root / path).read_bytes()).hexdigest() == expected, f"Unicode source changed without declared identity: {path}")
    require(bool(inventory["research_components"]), "full rustc SDK research dependencies must remain disclosed")
    for component in inventory["research_components"]:
        require((root / component["path"]).exists(), "missing disclosed research path: " + component["path"])
        require(component["production_admitted"] is False, "whole rustc SDK research is not a selective production import")
        require(bool(component["required_for"]) and bool(component["blocking_reason"]), "research closure needs its actual purpose and block")
    metadata_research_checks(root, inventory, require)
    public = json.loads((root / "upstream/public-library-integration.json").read_text())
    require(public.get("schema") == "lustr-public-library-integration-v1"
            and public.get("full_rust_coverage") is False
            and public.get("performance_qualified") is False,
            "public library declaration must preserve its bounded unqualified scope")
    for path, digest in public["source_sha256"].items():
        require(hashlib.sha256((root / path).read_bytes()).hexdigest() == digest,
                "public library source identity changed: " + path)
    require(public["metadata_dependency_closure"] == "upstream/metadata-dependency-closure.json"
            and public["artifact_tool_closure"] == "upstream/metadata-artifact-tools.json",
            "public library dependency/tool boundary changed")


    entry = (root / "src/main.rs").read_text()
    for token in ["rustc-full", "rustc-reference", "full-rust-core", "run_cli_with_default_root", "expanded_answer", "native-inference-worker"]:
        require(token not in entry, "public entry regained a retired route: " + token)
    require("integration-unimplemented" in entry, "unintegrated work must be explicit")
    require('var_os("LUST_PROBE_FAKE_DIR").is_some()' in entry, "external native probe modes must be refused, including empty values")
    require("Invocation::ReleaseCc" in entry, "native release must use the actual parsed request")
    require("run_program" in entry, "native run must remain reachable")
    retired_path_checks(root, require)
    cargo = tomllib.loads((root / "Cargo.toml").read_text())
    require("cargo_lust" not in cargo["dependencies"], "public compiler must not depend on the retired coordinator")
    semantic = tomllib.loads((root / "crates/lustr-semantic/Cargo.toml").read_text())
    require(not any(item.get("name") == "lustr-mixed" for item in semantic.get("bin", [])), "whole-rustc driver is a product target again")
    inspection_passthrough_checks(root, require)
    coordinator = (root / "crates/lustr/src/lib.rs").read_text()
    require("fn execute(" not in coordinator and ".exec()" not in coordinator, "retired coordinator must not execute compiler plans")
    builder = (root / "tools/build-native-toolchain.rs").read_text()
    for token in ["rustc-full", "rustc-reference", "lustr-mixed", "cargo-lustr", 'read_dir(sdk.join("lib"))',
                  "library/lustr/std", "lib/lustr-expanded", "source-library-identity.sock"]:
        require(token not in builder, "native builder packages whole-compiler dependencies: " + token)
    require("PARTIAL-NATIVE-COMPILER" in builder, "native build must state incomplete complementary coverage")
    require('"exclusive-coverage-admission"' in builder and 'repo.join("tools/check-exclusive-coverage.py")' in builder,
            "native constructor must admit the effective product graph before building compiler components")
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
