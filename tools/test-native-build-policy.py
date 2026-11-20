#!/usr/bin/env python3
"""Exercise the actual native library build script's product/research boundary.

The script is compiled from source, then denied configurations must fail before
querying an SDK, compiling dependencies or emitting a linking directive. A child
sentinel identifies attempted compiler work. The admitted product control stops
at that sentinel; it does not claim to construct or qualify a native library.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import shutil
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def construction_controls(root, rustc, work):
    """Execute the real constructor; no candidate compiler is built here."""
    source = root / "tools/build-native-toolchain.rs"
    helper = work / "native-constructor"
    built = subprocess.run([str(rustc), "--edition=2024", str(source), "-o", str(helper)],
                           cwd=root, capture_output=True, timeout=60)
    report = {"source_sha256": digest(source), "build_status": built.returncode,
              "build_stdout": built.stdout.decode(errors="replace"),
              "build_stderr": built.stderr.decode(errors="replace"), "controls": [],
              "scope": "actual constructor admission only; the admitted control stops at a failing SDK sentinel"}
    assert built.returncode == 0, report
    report["executable_sha256"] = digest(helper)
    spec = importlib.util.spec_from_file_location("construction_guard", root / "tools/check-exclusive-coverage.py")
    guard = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(guard)
    fixture = work / "construction-source"
    fixture.mkdir()

    def copy(relative):
        destination = fixture / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / relative, destination)

    for manifest, target in guard.PRODUCT_TARGETS.items():
        base = Path(manifest).parent
        paths = {"build.rs", *target["bin"].values(), *(path for path, _ in target["example"].values())}
        if target["lib"]:
            paths.add(target["lib"][0])
        for relative in (Path(manifest), base / "Cargo.lock", *(base / path for path in paths)):
            copy(relative)
    for crate in ("lustr-metadata-encoder", "lustr-metadata-sdk", "lustr-native-metadata-qualification"):
        shutil.copytree(root / "crates" / crate, fixture / "crates" / crate,
                        ignore=shutil.ignore_patterns("target"))
    for relative in ("upstream/complement-imports.json", "upstream/bluefe-core.pin",
                     "compiler/native/unicode-semantics.json", "compiler/native/src/lexer.rs",
                     "compiler/native/tools/gen_unicode_semantics.rs", "tools/check-exclusive-coverage.py",
                     "tools/build-native-toolchain.rs", "crates/lustr-native-core/src/upstream_definitions.rs",
                     "crates/lustr-semantic/src/main.rs", "crates/lustr-semantic/Cargo.toml",
                     "crates/lustr-semantic/build.rs", "crates/lustr/src/lib.rs"):
        copy(relative)
    # Admission now verifies the public library's complete declared source cut,
    # including shared artifact code, the SDK receipt and dependency/tool records.
    # Copy those original bytes; do not regenerate identities for this fixture.
    public_inventory = Path("upstream/public-library-integration.json")
    copy(public_inventory)
    public = json.loads((root / public_inventory).read_text())
    for relative in public["source_sha256"]:
        copy(relative)
    unicode = json.loads((root / "compiler/native/unicode-semantics.json").read_text())
    for relative in unicode["generated_and_retained_data"]:
        copy(relative)
    _, failures = guard.check(fixture)
    assert not failures, failures

    def git(*args):
        result = subprocess.run(["git", "-C", str(fixture), "-c", "user.name=Constructor Control",
                                 "-c", "user.email=constructor-control@example.invalid",
                                 "-c", "commit.gpgsign=false", *args], capture_output=True, timeout=10)
        assert result.returncode == 0, result.stderr.decode(errors="replace")

    git("init", "--quiet")
    git("add", ".")
    git("commit", "--quiet", "-m", "Source-bound constructor control fixture")
    sdk = work / "constructor-sdk"
    (sdk / "bin").mkdir(parents=True)
    tcc = work / "constructor-tcc"
    tcc.mkdir()
    marker = work / "constructor-compiler-invocations"
    for name in ("cargo", "rustc"):
        sentinel = sdk / "bin" / name
        sentinel.write_text('#!/bin/sh\nprintf "%s\\n" "$0" "$@" >> "$LUSTR_BUILD_POLICY_MARKER"\nexit 72\n')
        sentinel.chmod(0o755)
    cargo_home = work / "constructor-cargo-home"
    cargo_home.mkdir()
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith("CARGO_") and key not in ("RUSTFLAGS", "RUSTC_BOOTSTRAP", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")}
    environment.update(CARGO_HOME=str(cargo_home), LUSTR_BUILD_POLICY_MARKER=str(marker), RUST_BACKTRACE="0")

    def run(label, additions=None, expected=None, phase=None):
        output = work / ("constructor-output-" + label)
        result = subprocess.run([str(helper), str(fixture), str(sdk), str(tcc), str(output)],
                                cwd=root, env=dict(environment, **(additions or {})), capture_output=True, timeout=15)
        record = {"case": label, "status": result.returncode,
                  "stdout": result.stdout.decode(errors="replace"), "stderr": result.stderr.decode(errors="replace"),
                  "compiler_invoked": marker.exists(), "output_created": output.exists()}
        report["controls"].append(record)
        assert result.returncode == 101, record
        if expected:
            assert expected.encode() in result.stderr, record
        if phase:
            record["admission_stdout"] = (output / "logs" / (phase + ".stdout")).read_text()
            record["admission_stderr"] = (output / "logs" / (phase + ".stderr")).read_text()
            assert not (output / "toolchain").exists(), record
        else:
            assert not output.exists(), record
        return record

    for label, additions in (
        ("compiler-wrapper", {"RUSTC_WRAPPER": str(sdk / "bin/rustc")}),
        ("compiler-cfg-flags", {"CARGO_ENCODED_RUSTFLAGS": '--cfg\x1ffeature="upstream-semantic"'}),
        ("construction-profile", {"CARGO_PROFILE_DEV_OPT_LEVEL": "3"}),
        ("construction-target", {"CARGO_BUILD_TARGET": "x86_64-unknown-none"}),
    ):
        run(label, additions, "unreviewed compiler-construction override")
        assert not marker.exists()
    for label, path in (
        ("ancestor-config", work / ".cargo/config.toml"),
        ("source-config", fixture / ".cargo/config"),
        ("cargo-home-config", cargo_home / "config.toml"),
    ):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('[build]\nrustc-wrapper = "unreviewed-compiler"\n')
        run(label, expected="unreviewed compiler-construction Cargo configuration")
        assert not marker.exists()
        path.unlink()
    relative_home = fixture / "construction-cargo-home"
    relative_home.mkdir()
    (relative_home / "config.toml").write_text('[build]\nrustc-wrapper = "unreviewed-compiler"\n')
    run("relative-cargo-home-config", {"CARGO_HOME": relative_home.name},
        expected="unreviewed compiler-construction Cargo configuration")
    assert not marker.exists()
    shutil.rmtree(relative_home)
    config_directory = fixture / ".cargo"
    config_directory.rmdir()
    config_directory.write_text("not a Cargo configuration directory\n")
    run("uninspectable-cargo-config", expected="cannot inspect compiler-construction Cargo configuration")
    assert not marker.exists()
    config_directory.unlink()

    manifest = fixture / "Cargo.toml"
    original = manifest.read_text()
    manifest.write_text(original.replace('path = "src/main.rs"', 'path = "crates/lustr-semantic/src/main.rs"'))
    git("add", "Cargo.toml")
    git("commit", "--quiet", "-m", "Negative control: changed compiler source")
    record = run("entry-source-redirection", phase="exclusive-coverage-admission")
    assert not marker.exists() and "bin target sources" in record["admission_stdout"], record
    manifest.write_text(original)
    git("add", "Cargo.toml")
    git("commit", "--quiet", "-m", "Restore source admission control")
    # Empty wrapper settings and an unrelated terminal setting do not override
    # construction. Admission must reach the selected SDK, which fails here.
    record = run("admitted-selected-sdk", {"RUSTC_WRAPPER": "", "CARGO_TERM_COLOR": "never"}, phase="sdk-version")
    calls = marker.read_text().splitlines() if marker.exists() else []
    assert calls == [str(sdk / "bin/rustc"), "-vV"], (record, calls)
    record["expected_child_failure"] = 72
    marker.unlink()
    (sdk / "bin/rustc").write_text(
        '#!/bin/sh\nprintf "%s\\n" "$0" "$@" >> "$LUSTR_BUILD_POLICY_MARKER"\n'
        'printf "rustc moved-donor-control\\ncommit-hash: 4b5350e87e487aed08ab2f9f0160a4b2616438bc\\n"\n')
    record = run("moving-sdk-source", expected="select the matching pinned Rust SDK", phase="sdk-version")
    assert marker.read_text().splitlines() == [str(sdk / "bin/rustc"), "-vV"], record
    assert "4b5350e87e487aed08ab2f9f0160a4b2616438bc" in record["admission_stdout"], record
    report["passed"] = True
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--repository", type=Path, default=ROOT)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    root = args.repository.resolve(strict=True)
    source = root / "crates/lustr-native-core/build.rs"
    report = {"schema": 1, "passed": False, "build_script_sha256": digest(source),
              "runner_sha256": digest(Path(__file__)), "controls": [],
              "scope": "actual build-script feature admission; child sentinel is not compiler qualification"}
    try:
        with tempfile.TemporaryDirectory(prefix="lustr-build-policy-") as directory:
            work = Path(directory)
            helper = work / "native-build-policy"
            built = subprocess.run([str(args.rustc.absolute()), "--edition=2024", str(source), "-o", str(helper)],
                                   cwd=root, capture_output=True, timeout=60)
            report["build"] = {"status": built.returncode, "stdout": built.stdout.decode(errors="replace"),
                               "stderr": built.stderr.decode(errors="replace")}
            assert built.returncode == 0, report["build"]
            report["build_script_executable_sha256"] = digest(helper)
            sentinel = work / "compiler-sentinel"
            sentinel.write_text('#!/bin/sh\nprintf "%s\\n" "$@" >> "$LUSTR_BUILD_POLICY_MARKER"\nexit 72\n')
            sentinel.chmod(0o755)
            marker = work / "compiler-invocations"
            environment = {key: value for key, value in os.environ.items() if not key.startswith("CARGO_FEATURE_")}
            environment.update(RUSTC=str(sentinel), CARGO_MANIFEST_DIR=str(source.parent),
                               OUT_DIR=str(work), LUSTR_BUILD_POLICY_MARKER=str(marker), RUST_BACKTRACE="0")
            cases = [
                ("no-explicit-mode", {}, "select exclusive-native"),
                ("product-plus-native-qualification", {"EXCLUSIVE_NATIVE": "1", "NATIVE_QUALIFICATION": "1"}, "cannot be combined"),
                ("product-plus-upstream-sdk", {"EXCLUSIVE_NATIVE": "1", "NATIVE_QUALIFICATION": "1", "UPSTREAM_SEMANTIC": "1"}, "cannot be combined"),
            ]
            for label, features, expected in cases:
                result = subprocess.run([str(helper)], env=dict(environment, **{"CARGO_FEATURE_" + key: value for key, value in features.items()}),
                                        capture_output=True, timeout=10)
                record = {"case": label, "status": result.returncode, "stdout": result.stdout.decode(errors="replace"),
                          "stderr": result.stderr.decode(errors="replace"), "compiler_invoked": marker.exists()}
                report["controls"].append(record)
                assert result.returncode == 101 and expected.encode() in result.stderr, record
                assert result.stdout == b"" and not marker.exists(), record
            result = subprocess.run([str(helper)], env=dict(environment, CARGO_FEATURE_EXCLUSIVE_NATIVE="1"),
                                    capture_output=True, timeout=10)
            calls = marker.read_text().splitlines() if marker.exists() else []
            record = {"case": "product-reaches-native-build", "status": result.returncode,
                      "stdout": result.stdout.decode(errors="replace"), "stderr": result.stderr.decode(errors="replace"),
                      "compiler_invoked": bool(calls), "expected_child_failure": 72}
            report["controls"].append(record)
            assert calls == ["--edition=2024", str(root / "compiler/native/build.rs"), "-o", str(work / "upstream-build-helper")], calls
            assert result.returncode == 101 and result.stdout == b"", record
            assert b"cannot be combined" not in result.stderr and b"select exclusive-native" not in result.stderr, record
            report["constructor"] = construction_controls(root, args.rustc.absolute(), work)
            report["passed"] = True
    except Exception as error:
        report["error"] = str(error)
    finally:
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
