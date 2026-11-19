#!/usr/bin/env python3
"""Run the actual native build-script transition test with both Cargo layouts."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
TEST = "driver::buildscript::tests::script_reruns_when_what_it_read_changes"
SETTING = "__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT"
CARGO_SOURCE = "c07ac2c4d12237effc4b94f4543769b421790045"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    report = {"schema": 1, "test": TEST, "runs": [], "passed": False}

    def run(name, command, env=None):
        result = subprocess.run(command, cwd=ROOT / "compiler/native", env=env, capture_output=True)
        for suffix, data in (("stdout", result.stdout), ("stderr", result.stderr)):
            (args.output / (name + "." + suffix)).write_bytes(data)
        record = {"name": name, "command": command, "status": result.returncode,
                  "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(),
                  "stderr_sha256": hashlib.sha256(result.stderr).hexdigest()}
        report["runs"].append(record)
        if result.returncode:
            raise RuntimeError(name + " failed; original output is retained")
        return result.stdout

    try:
        version = run("cargo", ["cargo", "-Vv"]).decode()
        report["cargo"] = version
        if "commit-hash: " + CARGO_SOURCE + "\n" not in version:
            raise RuntimeError("the private layout switch requires the exact declared Cargo source")
        # Reuse Cargo's actual test artifact. Discovering it through JSON avoids
        # selecting a stale test binary or assuming either target layout.
        built = run("test-artifact", ["cargo", "test", "--locked", "--offline", "--bin", "lust-core",
                                      "--no-run", "--message-format=json"])
        executables = set()
        for line in built.splitlines():
            try:
                message = json.loads(line)
            except (ValueError, UnicodeError):
                continue
            if (message.get("reason") == "compiler-artifact"
                    and message.get("target", {}).get("name") == "lust-core"
                    and message.get("profile", {}).get("test") is True
                    and message.get("executable")):
                executables.add(message["executable"])
        if len(executables) != 1:
            raise RuntimeError("Cargo must identify exactly one native core test executable")
        executable = executables.pop()
        report["test_binary_sha256"] = hashlib.sha256(Path(executable).read_bytes()).hexdigest()
        for name, value in (("default-layout", None), ("compatibility-layout", "1")):
            env = os.environ.copy()
            if value is None:
                env.pop(SETTING, None)
            else:
                env[SETTING] = value
            output = run(name, [executable, TEST, "--exact", "--nocapture", "--test-threads=1"], env)
            if b"1 passed; 0 failed" not in output:
                raise RuntimeError(name + " did not execute the required transition test")
            report["runs"][-1]["layout_setting"] = value
        report["passed"] = True
    except Exception as error:
        report["error"] = str(error)
    finally:
        (args.output / "verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
