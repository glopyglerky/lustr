#!/usr/bin/env python3
"""Compare real native no-daemon library selection with the matching Cargo/Rust.

Each output directory must be new. A failing report is retained and exits 1.
The checkout's native tree must be committed before building and running it.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("lust", "cargo", "rustc", "repository", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    args.lust, args.cargo, args.rustc, args.repository = (
        path.resolve() for path in (args.lust, args.cargo, args.rustc, args.repository))
    output = args.output.absolute()
    output.mkdir()
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--", "compiler/native"],
                   cwd=args.repository, check=True, stdout=subprocess.PIPE)
    revision, tree = subprocess.check_output(
        ["git", "rev-parse", "HEAD", "HEAD:compiler/native"], cwd=args.repository,
        text=True).splitlines()
    env = dict(os.environ, RUSTC=str(args.rustc), CARGO=str(args.cargo),
               PATH=str(args.cargo.parent) + os.pathsep + os.environ.get("PATH", ""),
               LUST_NO_DAEMON="1", LUST_NO_CACHE="1", LUST_CACHE_DIR=str(output / "cache"),
               CARGO_TARGET_DIR=str(output / "cargo-target"),
               __CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT="1")
    for name in list(env):
        if name.startswith("LUST_FRONT_") or name in ("LUST_LIBRARY", "LUST_SOURCES_READY", "RUSTC_WRAPPER",
                                                        "RUSTC_WORKSPACE_WRAPPER", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            del env[name]
    records = []

    def run(label, command):
        result = subprocess.run([str(arg) for arg in command], cwd=output, env=env,
                                capture_output=True, timeout=90)
        (output / (label + ".stdout")).write_bytes(result.stdout)
        (output / (label + ".stderr")).write_bytes(result.stderr)
        record = {"label": label, "command": [str(arg) for arg in command], "exit_code": result.returncode,
                  "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(),
                  "stderr_sha256": hashlib.sha256(result.stderr).hexdigest()}
        records.append(record)
        return result

    identities = {}
    for label, binary, flags in [("lust-version", args.lust, ["--version"]),
                                  ("rustc-version", args.rustc, ["-vV"]),
                                  ("cargo-version", args.cargo, ["--version"])]:
        result = run(label, [binary, *flags])
        if result.returncode:
            raise RuntimeError(label + " failed")
        identities[label] = result.stdout.decode()
    identities.update({"lust_sha256": digest(args.lust), "lust_core_sha256": digest(args.lust.with_name("lust-core")),
                       "rustc_sha256": digest(args.rustc), "cargo_sha256": digest(args.cargo),
                       "source_revision": revision, "native_source_tree": tree,
                       "runner_sha256": digest(Path(__file__))})
    failures = []
    cases = []
    for case, package, lib_name, kind in [
        ("hyphen", "hyphen-lib", None, None), ("renamed", "app", "app_library", None),
        *[(kind, "library_kind_" + kind.replace("-", "_"), None, kind)
          for kind in ("rlib", "staticlib", "cdylib", "dylib", "proc-macro")],
    ]:
        root = output / case
        (root / "src").mkdir(parents=True)
        manifest = root / "Cargo.toml"
        library = '\n[lib]\n' if lib_name or kind else ''
        if lib_name:
            library += 'name=' + json.dumps(lib_name) + '\n'
        if kind:
            library += 'proc-macro=true\n' if kind == 'proc-macro' else 'crate-type=[' + json.dumps(kind) + ']\n'
        manifest.write_text('[package]\nname=' + json.dumps(package) + '\nversion="0.1.0"\nedition="2024"\n[workspace]\n' + library)
        source = root / "src/lib.rs"
        visibility = '' if kind == 'proc-macro' else 'pub '
        source.write_text(visibility + 'fn answer()->u32{42}\n')
        metadata = run(case + "-metadata", [args.cargo, "metadata", "--offline", "--no-deps", "--format-version", "1",
                                            "--manifest-path", manifest])
        if metadata.returncode:
            raise RuntimeError(case + ": metadata discovery failed")
        targets = [target for package in json.loads(metadata.stdout)["packages"] for target in package["targets"]
                   if Path(target["src_path"]).resolve() == source.resolve()]
        if len(targets) != 1 or targets[0]["kind"] != [kind or "lib"]:
            raise RuntimeError(case + ": Cargo metadata did not identify the declared library kind")
        for state, source_text, bin_text, success in [
            ("valid", visibility + 'fn answer()->u32{42}\n', 'fn main(){let _:u32="wrong bin";}\n', True),
            ("invalid", visibility + 'fn answer()->u32{"wrong library"}\n', 'fn main(){}\n', False),
            ("recovered", visibility + 'fn answer()->u32{42}\n', 'fn main(){let _:u32="wrong bin";}\n', True),
        ]:
            source.write_text(source_text)
            if lib_name:
                (root / "src/main.rs").write_text(bin_text)
            inputs = {str(path.relative_to(root)): digest(path) for path in sorted(root.rglob("*")) if path.is_file()}
            label = case + "-" + state
            stock = run(label + "-cargo", [args.cargo, "check", "--offline", "--lib", "--manifest-path", manifest])
            native = run(label + "-native", [args.lust, "check", source, "--manifest-path", manifest, "--lib", "--offline"])
            if (stock.returncode == 0) != success:
                failures.append(label + ": Cargo oracle did not match fixture expectation")
            if (native.returncode == 0) != success or (not success and native.returncode != 1):
                failures.append(label + ": native selected the wrong target or failed outside semantic checking")
            for path, value in inputs.items():
                if digest(root / path) != value:
                    failures.append(label + ": compiler changed " + path)
            cases.append({"case": label, "expect_success": success, "inputs_sha256": inputs,
                          "source": source_text, "bin_source": bin_text if lib_name else None,
                          "cargo_exit_code": stock.returncode, "native_exit_code": native.returncode})
    root = output / "bin-only"
    (root / "src").mkdir(parents=True)
    manifest, source = root / "Cargo.toml", root / "src/main.rs"
    manifest.write_text('[package]\nname="binary_only"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
    source.write_text('fn main(){}\n')
    inputs = {str(path.relative_to(root)): digest(path) for path in sorted(root.rglob("*")) if path.is_file()}
    stock = run("bin-only-cargo", [args.cargo, "check", "--offline", "--lib", "--manifest-path", manifest])
    native = run("bin-only-native", [args.lust, "check", source, "--manifest-path", manifest, "--lib", "--offline"])
    if stock.returncode == 0 or native.returncode != 2:
        failures.append("bin-only: --lib must fail target selection instead of substituting a binary")
    for path, value in inputs.items():
        if digest(root / path) != value:
            failures.append("bin-only: compiler changed " + path)
    cases.append({"case": "bin-only", "expect_success": False, "inputs_sha256": inputs,
                  "source": source.read_text(), "cargo_exit_code": stock.returncode,
                  "native_exit_code": native.returncode})
    report = {"schema": "lustr.native-library-selection.v1", "identities": identities,
              "environment": {name: env[name] for name in ("LUST_NO_DAEMON", "LUST_NO_CACHE", "RUSTC", "CARGO", "PATH")},
              "cases": cases, "commands": records, "failures": failures, "passed": not failures}
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    for case in cases:
        print(f"{case['case']}: native={case['native_exit_code']} Cargo={case['cargo_exit_code']}")
    for failure in failures:
        print(failure, file=sys.stderr)
    return int(bool(failures))


if __name__ == "__main__":
    sys.exit(main())
