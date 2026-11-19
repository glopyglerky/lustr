#!/usr/bin/env python3
"""Compare supported upstream capability results through the public compiler.

The unchanged upstream declaration probe checks acceptance and diagnostic codes.
This companion checks the actual generated programs, including fixtures that
only print values. Rust is an independent qualification oracle, never a product
fallback. This finite cold campaign does not qualify retained speed or all Rust.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def supported_probes(directory):
    probes = []
    for path in sorted(directory.glob("*.rs")):
        header = []
        for line in path.read_text().splitlines():
            if not line.startswith("//"):
                break
            header.append(line)
        expectations = [line.removeprefix("// expect: ") for line in header if line.startswith("// expect: ")]
        require(len(expectations) == 1, f"{path.name}: one declared expectation required")
        if expectations[0] != "ok":
            require(re.fullmatch(r"L\d{4}", expectations[0]), f"{path.name}: unknown expectation")
            continue
        commands = [line.removeprefix("// command: ") for line in header if line.startswith("// command: ")]
        require(len(commands) <= 1, f"{path.name}: duplicate command")
        command = commands[0] if commands else "run"
        require(command in ("run", "test"), f"{path.name}: unqualified supported command {command}")
        probes.append((path, command))
    require(bool(probes), "no supported capability probes found")
    return probes


def compare_program(candidate, oracle):
    require(candidate.returncode == 0, f"candidate program exited {candidate.returncode}")
    require(oracle.returncode == 0, f"oracle program exited {oracle.returncode}")
    require(candidate.stdout == oracle.stdout, "program stdout differs from Rust")
    require(candidate.stderr == oracle.stderr, "program stderr differs from Rust")


def rust_test_outcomes(stdout):
    text = stdout.decode("utf-8")
    matches = re.findall(r"^test (\S+?)(?: - should panic)? \.\.\. (ok|FAILED|ignored)$", text, re.M)
    outcomes = {name: {"ok": "pass", "FAILED": "fail", "ignored": "ignored"}[status] for name, status in matches}
    summary = re.findall(r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;", text, re.M)
    require(len(summary) == 1 and matches and len(outcomes) == len(matches), "Rust test outcomes are missing or duplicated")
    passed, failed, ignored, measured, filtered = map(int, summary[0])
    require(failed == measured == filtered == 0 and passed > 0, "Rust test oracle did not execute all selected tests")
    require(passed == sum(value == "pass" for value in outcomes.values()) and ignored == sum(value == "ignored" for value in outcomes.values()), "Rust test summary differs from its individual outcomes")
    require(passed + ignored == len(outcomes), "Rust test outcome count differs")
    return outcomes


def native_test_outcomes(stdout):
    records = [json.loads(line) for line in stdout.splitlines() if line.strip()]
    require(all(isinstance(item, dict) for item in records), "native test records must be objects")
    require(all(type(item.get("schema")) is int and item["schema"] == 1
                and item.get("kind") in ("test", "summary") for item in records),
            "native test records have an unknown schema or kind")
    tests = [item for item in records if item.get("kind") == "test"]
    summaries = [item for item in records if item.get("kind") == "summary"]
    require(all(isinstance(item.get("name"), str) and item["name"]
                and item.get("status") in ("pass", "ignored") for item in tests),
            "native test name or status is invalid")
    require(all(item.get("exit_code") is None if item["status"] == "ignored"
                else type(item.get("exit_code")) is int and item["exit_code"] in (0, 101)
                for item in tests), "native test has an invalid or signalled exit")
    outcomes = {item["name"]: item["status"] for item in tests}
    require(len(summaries) == 1 and tests and len(outcomes) == len(tests), "native test outcomes are missing or duplicated")
    require(all(value in ("pass", "ignored") for value in outcomes.values()), "native test failed or timed out")
    summary = summaries[0]
    require(all(type(summary.get(field)) is int and summary[field] >= 0
                for field in ("total", "passed", "failed", "timed_out", "ignored", "filtered_out")),
            "native test counts must be nonnegative integers")
    require(summary["build"] == "ok" and summary["failed"] == summary["timed_out"] == summary["filtered_out"] == 0, "native test summary reports an incomplete or failed run")
    require(summary["total"] == len(outcomes) and summary["passed"] > 0
            and summary["passed"] == sum(value == "pass" for value in outcomes.values())
            and summary["ignored"] == sum(value == "ignored" for value in outcomes.values()),
            "native test summary differs from its individual outcomes")
    return outcomes


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT)
    parser.add_argument("--entry", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    args.source, args.output = args.source.resolve(), args.output.resolve()
    # Preserve the requested entry name and sibling runtime layout.
    args.entry, args.rustc = args.entry.absolute(), args.rustc.absolute()
    args.output.mkdir(parents=True, exist_ok=False)
    report = {"schema": "lustr-capability-results-v1", "status": "preparing", "passed": False,
              "commands": [], "cases": [], "full_rust_coverage": False,
              "scope": "Unchanged supported upstream probes through the public entry, cold compiled artifacts and test outcomes; no retained or performance qualification."}

    def save():
        (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    save()
    env = dict(os.environ)
    for name in tuple(env):
        if name.startswith(("LUST_", "LUSTR_")) or name in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "LD_LIBRARY_PATH"):
            env.pop(name)
    env.update(LUST_NO_DAEMON="1", LUST_NO_CACHE="1", RUST_BACKTRACE="0", LC_ALL="C")
    work = args.output / "work"
    work.mkdir()
    (work / "tmp").mkdir()
    env.update(TMPDIR=str(work / "tmp"), LUST_CACHE_DIR=str(work / "cache"))

    def run(label, command, cwd=work, timeout=120):
        command = list(map(str, command))
        row = {"label": label, "argv": command, "exit": None, "timed_out": False}
        report["commands"].append(row)
        save()
        outpath, errpath = args.output / (label + ".stdout"), args.output / (label + ".stderr")
        with outpath.open("wb") as out, errpath.open("wb") as err:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdout=out, stderr=err, start_new_session=True)
            try:
                process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                row["timed_out"] = True
                # Only this invocation's live process group is addressed.
                if process.poll() is None:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                process.wait(timeout=5)
        row.update(exit=process.returncode, stdout_sha256=digest(outpath), stderr_sha256=digest(errpath))
        save()
        require(not row["timed_out"], label + " timed out")
        return subprocess.CompletedProcess(command, process.returncode, outpath.read_bytes(), errpath.read_bytes())

    def accepted(label, command, cwd=work):
        result = run(label, command, cwd)
        require(result.returncode == 0, f"{label} failed with exit {result.returncode}")
        return result

    try:
        source_probes = args.source / "compiler/native/tests/capabilities"
        probes = supported_probes(source_probes)
        report["expected_cases"] = [path.name for path, _ in probes]
        report["fixture_sha256"] = {str(path.relative_to(source_probes)): digest(path) for path in sorted(source_probes.rglob("*")) if path.is_file()}
        runtime = args.entry.parent.parent
        components = [args.entry, args.entry.with_name("lust"), args.entry.with_name("lust-core"), args.rustc]
        components += sorted((runtime / "lib").glob("liblustr_native_core.*"))
        report["binary_sha256"] = {str(path): digest(path) for path in components}
        report["runner_sha256"] = digest(Path(__file__))
        report["upstream_probe_sha256"] = digest(args.source / "compiler/native/scripts/test-capabilities.py")
        report["source_head"] = subprocess.check_output(["git", "-C", str(args.source), "rev-parse", "HEAD"], text=True).strip()
        version = accepted("rust-version", [args.rustc, "-vV"])
        declared = re.search(r'^source = "([0-9a-f]{40})"$', (args.source / "upstream/bluefe-core.pin").read_text(), re.M)
        require(declared is not None and f"commit-hash: {declared[1]}\n".encode() in version.stdout, "Rust oracle differs from the declared semantic SDK")
        report["rust_source"] = declared[1]
        shutil.copytree(source_probes, work / "sources")
        report["status"] = "running"
        for original, command in probes:
            source = work / "sources" / original.name
            label = original.stem
            case = {"fixture": original.name, "command": command, "passed": False}
            report["cases"].append(case)
            save()
            try:
                oracle = work / (label + "-rust")
                flags = ["--edition=2024", "--crate-name", "capability_probe", "-A", "warnings", "-C", "overflow-checks=on"]
                if command == "run":
                    native = work / (label + "-native")
                    accepted(label + "-rust-build", [args.rustc, *flags, source, "-o", oracle])
                    accepted(label + "-native-build", [args.entry, "build", source, "-o", native])
                    case["artifact_sha256"] = {"native": digest(native), "rust": digest(oracle)}
                    expected = run(label + "-rust-run", [oracle], timeout=10)
                    actual = run(label + "-native-run", [native], timeout=10)
                    compare_program(actual, expected)
                    case["comparison"] = "exact stdout, stderr and zero exit status"
                else:
                    accepted(label + "-rust-build", [args.rustc, *flags, "--test", source, "-o", oracle])
                    case["artifact_sha256"] = {"rust": digest(oracle)}
                    expected = accepted(label + "-rust-test", [oracle, "--test-threads=1", "--format=pretty"])
                    actual = accepted(label + "-native-test", [args.entry, "test", source, "--json"])
                    expected_tests, actual_tests = rust_test_outcomes(expected.stdout), native_test_outcomes(actual.stdout)
                    require(actual_tests == expected_tests, "native test names/statuses differ from Rust")
                    case.update(comparison="exact test names/statuses, complete nonempty summaries and zero harness exits", outcomes=actual_tests)
                case["passed"] = True
            except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
                case["failure"] = str(error)
            save()
            print(json.dumps(case), flush=True)
        require(len(report["cases"]) == len(probes), "capability case inventory is incomplete")
        require(report["binary_sha256"] == {str(path): digest(path) for path in components}, "compiler identity changed during qualification")
        require(report["fixture_sha256"] == {str(path.relative_to(source_probes)): digest(path) for path in sorted(source_probes.rglob("*")) if path.is_file()}, "upstream fixture inputs changed during qualification")
        require(report["fixture_sha256"] == {str(path.relative_to(work / "sources")): digest(path) for path in sorted((work / "sources").rglob("*")) if path.is_file()}, "copied fixture inputs changed during qualification")
        report["inputs_unchanged"] = True
        report["passed"] = all(case["passed"] for case in report["cases"])
        report["status"] = "passed" if report["passed"] else "failed"
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        report.update(status="setup-failed", failure=str(error))
    save()
    print(json.dumps({"status": report["status"], "cases": len(report["cases"]), "failed": sum(not case["passed"] for case in report["cases"])}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
