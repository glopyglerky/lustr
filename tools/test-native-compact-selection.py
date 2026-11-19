#!/usr/bin/env python3
"""Validate compact selection records and fresh Cargo metadata selection.

This is a component gate. It does not run the retained native compiler or
establish public-entry performance. Deliberately malformed private records
have their checksums recomputed to exercise shape validation independently;
the checksum is not authentication against the cache's own trusted owner.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profiler", type=Path, required=True)
    parser.add_argument("--package-bin", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    profiler = args.profiler.resolve(strict=True)
    package_bin = args.package_bin.resolve(strict=True)
    root = args.output.absolute()
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    for directory in ("project/src", "home", "cargo-home", "tmp", "cache", "streams"):
        (root / directory).mkdir(parents=True)
    project = root / "project"
    manifest = project / "Cargo.toml"
    source = project / "src/main.rs"
    second_source = project / "src/second.rs"
    manifest.write_text('[package]\nname="compact-selection-regression"\nversion="0.1.0"\n'
                        'edition="2024"\n[workspace]\n[[bin]]\nname="first"\npath="src/main.rs"\n'
                        '[[bin]]\nname="second"\npath="src/second.rs"\n')
    original = 'fn main() { println!("selected-first:{}", if cfg!(debug_assertions) { "debug" } else { "nodebug" }); }\n'
    source.write_text(original)
    second_source.write_text('fn main() { println!("selected-second"); }\n')
    environment = {
        "PATH": str(package_bin) + ":/usr/local/bin:/usr/bin:/bin",
        "HOME": str(root / "home"), "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
        "CARGO_HOME": str(root / "cargo-home"), "CARGO_TARGET_DIR": str(root / "target"),
        "CARGO_BUILD_BUILD_DIR": str(root / "intermediates"), "CARGO_NET_OFFLINE": "true",
        "CARGO_TERM_COLOR": "never", "TMPDIR": str(root / "tmp"),
        "LUST_CACHE_DIR": str(root / "cache"),
        "LUSTR_SELECTION_DIAGNOSTICS": str(root / "diagnostics.jsonl"),
        "LUSTR_PROFILE_SELECTION_DETAIL": "1",
    }
    records, assertions = [], []
    report = {"schema": "lustr-compact-selection-regression-v1",
              "scope": "complete selection identity, malformed private records, and actual Cargo selection; no native or performance qualification",
              "profiler_sha256": sha256(profiler.read_bytes()), "runner_sha256": sha256(Path(__file__).read_bytes()),
              "environment": environment, "records": records, "assertions": assertions,
              "status": "running", "error": None}

    def save():
        (root / "results.json").write_text(json.dumps(report, indent=2) + "\n")

    def require(condition, description):
        assertions.append({"description": description, "passed": bool(condition)})
        save()
        if not condition:
            raise AssertionError(description)

    def execute(label, argv, *, env=None, kind="helper"):
        actual_env = dict(environment if env is None else env)
        prefix = f"{len(records):03d}-{label}"
        timeout = None
        try:
            result = subprocess.run(argv, cwd=project, env=actual_env, capture_output=True, timeout=60)
        except subprocess.TimeoutExpired as error:
            timeout = 60
            result = subprocess.CompletedProcess(argv, None, error.stdout or b"", error.stderr or b"")
        stdout = root / "streams" / (prefix + ".stdout")
        stderr = root / "streams" / (prefix + ".stderr")
        stdout.write_bytes(result.stdout)
        stderr.write_bytes(result.stderr)
        record = {"label": label, "kind": kind, "argv": [str(value) for value in argv],
                  "cwd": str(project), "environment": actual_env, "exit": result.returncode,
                  "timeout_seconds": timeout, "stdout": str(stdout.relative_to(root)),
                  "stderr": str(stderr.relative_to(root)), "stdout_sha256": sha256(result.stdout),
                  "stderr_sha256": sha256(result.stderr)}
        records.append(record)
        save()
        require(timeout is None and result.returncode is not None and result.returncode >= 0,
                label + " completes without a timeout or signal")
        return result, record

    def arguments(command="check", *, second=False, out=None, flags=()):
        result = [command, str(second_source if second else source), "--manifest-path", str(manifest),
                  "--offline", "--locked", "--bin", "second" if second else "first"]
        if out is not None:
            result.extend(["-o", str(out)])
        return result + list(flags)

    default_args = arguments()

    def invoke(label, native_args=default_args, *, phase="selection", uncached=False, expected=0, event=None):
        diagnostic = Path(environment["LUSTR_SELECTION_DIAGNOSTICS"])
        diagnostic.write_text("")
        env = dict(environment)
        if uncached:
            env["LUSTR_NO_SELECTION_CACHE"] = "1"
        result, record = execute(label, [str(profiler), phase, str(package_bin), *native_args], env=env)
        events = [json.loads(line) for line in diagnostic.read_text().splitlines()]
        record["diagnostics"] = events
        require(result.returncode == expected, label + " preserves the expected exit status")
        data = json.loads(result.stdout)
        record["result"] = data
        if expected == 0:
            require(isinstance(data.get("selection"), dict), label + " exposes the complete actual Selection")
        if event is not None:
            require(events == [{"selection_cache": event}], label + " observes " + event)
        save()
        return data, result

    def selection_pair(label, native_args=default_args, *, expected=0, event=None):
        candidate, candidate_run = invoke(label, native_args, expected=expected, event=event)
        reference, reference_run = invoke(label + "-uncached", native_args, expected=expected, uncached=True)
        candidate.pop("elapsed_ns", None)
        reference.pop("elapsed_ns", None)
        require(candidate == reference, label + " preserves every Selection field or the exact rejection")
        require(candidate_run.stderr == reference_run.stderr, label + " preserves Cargo selection stderr")
        return candidate

    def prime(label, native_args=default_args):
        selection_pair(label + "-seed", native_args)
        return selection_pair(label + "-hit", native_args, event="hit")

    def current_record():
        entries = []
        for path in (root / "cache/cargo-selection-v1").glob("*.json"):
            if path.name.startswith("tools-"):
                continue
            record = json.loads(path.read_bytes())
            if record["payload"].get("schema") == "lustr-cargo-selection-exclusive-v2":
                entries.append((path, record))
        require(len(entries) == 1, "malformed-record controls address the only current selection context")
        path, record = entries[0]
        require(record["sha256"] == sha256(encoded(record["payload"])),
                "test checksum serialization agrees with the actual private record")
        return path, record

    def malformed(label, mutate):
        path, original_record = current_record()
        record = copy.deepcopy(original_record)
        mutate(record["payload"])
        record["sha256"] = sha256(encoded(record["payload"]))
        path.write_bytes(encoded(record))
        (root / (label + ".json")).write_bytes(encoded(record))
        selection_pair(label, event="miss")
        selection_pair(label + "-recovered", event="hit")

    try:
        setup, _ = execute("setup", [str(package_bin / "cargo"), "generate-lockfile", "--offline", "--manifest-path", str(manifest)], kind="setup")
        require(setup.returncode == 0, "real Cargo lockfile setup succeeds")
        prime("initial")
        controls = [
            ("old-selection-schema", lambda p: p.update(schema="lustr-cargo-selection-v2")),
            ("missing-state-digest", lambda p: p.pop("state_sha256")),
            ("changed-state-digest", lambda p: p.update(state_sha256="0" * 64)),
            ("missing-selection", lambda p: p.pop("selection")),
            ("null-package", lambda p: p["selection"].update(package=None)),
            ("extra-selection-field", lambda p: p["selection"].update(unexpected=True)),
            ("inconsistent-manifest", lambda p: p["selection"].update(manifest=str(project / "Other.toml"))),
            ("inconsistent-name", lambda p: p["selection"].update(name="second")),
            ("nonarray-kind", lambda p: p["selection"]["target"].update(kind="bin")),
            ("empty-kind", lambda p: p["selection"]["target"].update(kind=[])),
            ("nonstring-kind", lambda p: p["selection"]["target"].update(kind=[1])),
            ("inconsistent-target-source", lambda p: p["selection"]["target"].update(src_path=str(second_source))),
            ("target-absent-from-package", lambda p: p["selection"]["package"].update(targets=p["selection"]["package"]["targets"][1:])),
            ("missing-target-bindings", lambda p: p.pop("targets")),
            ("malformed-metadata-provenance", lambda p: p.update(metadata_sha256="not-a-digest")),
        ]
        for label, mutate in controls:
            malformed(label, mutate)

        # Profile/output/target context changes still compare complete cached
        # and fresh Cargo selection. Native emission is independently exercised
        # by the public-entry controls; the old stock retry no longer exists.
        prime("explicit-package-check", arguments(flags=("--package", "compact-selection-regression")))
        prime("default-build", arguments("build", out=root / "first-program"))
        prime("changed-profile-and-output", arguments("build", out=root / "changed-output-program", flags=("-C", "debug-assertions=off")))
        prime("different-selected-target", arguments("build", second=True, out=root / "second-program", flags=("--package", "compact-selection-regression")))
        wrong_package_args = arguments(flags=("--package", "absent-package"))
        selection_pair("wrong-package-rejected", wrong_package_args, expected=2, event="miss")
        report["status"] = "completed"
        report["success"] = True
        save()
        print(json.dumps({"success": True, "commands": len(records), "assertions": len(assertions), "report": str(root / "results.json")}))
    except BaseException as error:
        report.update(status="failed", success=False, error=f"{type(error).__name__}: {error}")
        save()
        raise
    finally:
        source.write_text(original)


if __name__ == "__main__":
    main()
