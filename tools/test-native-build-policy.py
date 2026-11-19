#!/usr/bin/env python3
"""Exercise the actual native library build script's product/research boundary.

The script is compiled from source, then denied configurations must fail before
querying an SDK, compiling dependencies or emitting a linking directive. A child
sentinel identifies attempted compiler work. The admitted product control stops
at that sentinel; it does not claim to construct or qualify a native library.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


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
