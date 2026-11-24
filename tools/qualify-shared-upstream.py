#!/usr/bin/env python3
"""Build and exercise the first shared upstream service on its declared SDK."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
RUST_SOURCE = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494"


def digest(path):
    value = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            value.update(block)
    return value.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--tcc", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--emission-only", action="store_true",
                        help="qualify the imported code ownership lifecycle without repeating the full semantic campaign")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    cargo, rustc, tcc = args.cargo.absolute(), args.rustc.absolute(), args.tcc.resolve()
    env = os.environ.copy()
    env.update(RUSTC=str(rustc), LUST_TCC_HOME=str(tcc), LUST_NO_CACHE="1", LUST_NO_DAEMON="1",
               LUST_CACHE_DIR=str(output / "native-cache"), TMPDIR=str(output / "tmp"),
               LUST_LIBRARY=str(ROOT / "compiler/native/library"))
    env["PATH"] = str(rustc.parent) + os.pathsep + env.get("PATH", "")
    (output / "tmp").mkdir()
    report = {"schema": 1, "scope": ("internal imported emission lifecycle" if args.emission_only else "internal shared definition service") + "; no public routing or performance claim",
              "passed": False, "commands": [], "rust_source": RUST_SOURCE}

    def save():
        (output / "verification.json").write_text(json.dumps(report, indent=2) + "\n")

    def run(name, command, timeout=120):
        command = [str(value) for value in command]
        stdout, stderr = output / (name + ".stdout"), output / (name + ".stderr")
        record = {"name": name, "command": command, "status": None}
        report["commands"].append(record)
        try:
            with stdout.open("wb") as out, stderr.open("wb") as err:
                result = subprocess.run(command, cwd=ROOT, env=env, stdout=out, stderr=err, timeout=timeout)
            record["status"] = result.returncode
        finally:
            for suffix, path in (("stdout", stdout), ("stderr", stderr)):
                if path.exists():
                    record[suffix + "_sha256"] = digest(path)
                    record[suffix + "_bytes"] = path.stat().st_size
            save()
        if result.returncode:
            raise RuntimeError(f"{name} failed with {result.returncode}; see {stderr}")
        return stdout

    try:
        identity = run("rustc-identity", [rustc, "-vV"]).read_text()
        report["rustc_identity"] = identity
        if f"commit-hash: {RUST_SOURCE}\n" not in identity:
            raise RuntimeError("the imported service requires the exact declared Rust source")
        report["cargo_identity"] = run("cargo-identity", [cargo, "-Vv"]).read_text()
        report["source"] = run("source", ["git", "rev-parse", "HEAD", "HEAD^{tree}", "HEAD:compiler/native"]).read_text().splitlines()
        dirty = run("source-state", ["git", "status", "--porcelain", "--", "compiler/native",
                    "crates/lustr-native-core", "tests/shared-imports", "tools/native-upstream-qualification.rs",
                    "tools/test-native-upstream.py", "tools/test-native-import-emission.py",
                    "tools/qualify-shared-upstream.py", "upstream/bluefe-core.pin"]).read_text()
        if dirty:
            raise RuntimeError("shared-service source must be committed before qualification")
        report["inputs_sha256"] = {
            "rustc": digest(rustc), "cargo": digest(cargo), "tcc": digest(tcc / "bin/tcc"),
            "libtcc": digest(tcc / "lib/libtcc.a"), "launcher": digest(__file__),
            "campaign": digest(ROOT / "tools/test-native-upstream.py"),
            "emission_campaign": digest(ROOT / "tools/test-native-import-emission.py"),
            "sdk_declaration": digest(ROOT / "upstream/bluefe-core.pin"),
        }
        built = run("build", [cargo, "build", "--manifest-path", ROOT / "crates/lustr-native-core/Cargo.toml",
                    "--locked", "--offline", "--jobs", "2", "--target-dir", output / "target",
                    "--config", "build.build-dir=" + json.dumps(str(output / "intermediates")),
                    "--config", "profile.dev.debug=0", "--config", "profile.dev.incremental=false",
                    "--features", "upstream-semantic", "--example", "native-upstream-qualification",
                    "--message-format=json"], timeout=1200)
        executables = set()
        for line in built.read_bytes().splitlines():
            try:
                message = json.loads(line)
            except (ValueError, UnicodeError):
                continue
            if (message.get("reason") == "compiler-artifact"
                    and message.get("target", {}).get("name") == "native-upstream-qualification"
                    and "example" in message.get("target", {}).get("kind", [])
                    and message.get("executable")):
                executables.add(message["executable"])
        if len(executables) != 1:
            raise RuntimeError("Cargo must identify exactly one shared-service qualification executable")
        qualifier = Path(executables.pop())
        report["qualifier_sha256"] = digest(qualifier)
        report["qualifier"] = str(qualifier)
        campaign_script = "tools/test-native-import-emission.py" if args.emission_only else "tools/test-native-upstream.py"
        run("campaign", [sys.executable, ROOT / campaign_script, "--rustc", rustc,
                         "--qualifier", qualifier, "--out", output / "campaign"], timeout=900)
        campaign = json.loads((output / "campaign/summary.json").read_text())
        if (not campaign.get("passed") or campaign.get("qualifier_sha256") != report["qualifier_sha256"]
                or digest(qualifier) != report["qualifier_sha256"]):
            raise RuntimeError("campaign did not qualify the unchanged built executable")
        report["controls"] = len(campaign["cases"])
        if not args.emission_only:
            adapter_state = output / "campaign/integer-signatures-state"
            report["integer_adapters"] = {
                "count": 30, "checked_results": 262720,
                "llvm_input_sha256": digest(adapter_state / "integer-adapters.ll"),
                "tool_identity": (adapter_state / "integer-adapter-tool.txt").read_text(),
            }
            run("emission-campaign", [sys.executable, ROOT / "tools/test-native-import-emission.py",
                "--rustc", rustc, "--qualifier", qualifier, "--out", output / "emission-campaign"], timeout=900)
            emission = json.loads((output / "emission-campaign/summary.json").read_text())
            if (not emission.get("passed") or emission.get("qualifier_sha256") != report["qualifier_sha256"]
                    or digest(qualifier) != report["qualifier_sha256"]):
                raise RuntimeError("emission campaign did not qualify the unchanged built executable")
            report["emission_controls"] = len(emission["cases"])
        report["passed"] = True
    except Exception as error:
        report["error"] = str(error)
    finally:
        save()
    print(json.dumps(report, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
