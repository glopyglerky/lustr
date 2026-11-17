#!/usr/bin/env python3
"""Run the real mixed qualifier separately and attribute instrumented wall time.

This is diagnostic profiling, not a paired performance campaign. Nested spans
and concurrently compiled crates cannot be added into command elapsed time.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys


EXPECTED = {
    "cold": True,
    "body": True,
    "arithmetic-edit": True,
    "invalid": False,
    "recovery": True,
    "link-failure": False,
    "link-failure-recovery": True,
    "versioned-object": True,
    "versioned-object-noop": True,
    "signature": True,
    "dependency": True,
    "macro": True,
    "warning-cold": True,
    "warning-edit": True,
    "warning-denied": False,
}
PHASES = {
    "driver", "request-setup", "rustc-driver", "parse-candidates",
    "after-expansion", "select-native-after-analysis", "retained-attempt",
    "context-fingerprint", "changed-body-queries", "native-check",
    "native-compile", "native-symbol-rename", "edited-native-object-sync",
    "retain-link-recipe", "retained-object-sync", "relink",
    "temporary-link-copies", "original-linker", "publish-certificate",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def read_profile(path):
    require(path.is_file() and not path.is_symlink(), f"invalid profile file: {path.name}")
    records = [json.loads(line) for line in path.read_text().splitlines()]
    require(len(records) >= 5, f"incomplete profile: {path.name}")
    header, final = records[0], records[-1]
    require(header.get("schema") == "lustr.mixed-phase-profile" and header.get("version") == 1, "unknown profile schema")
    require(header.get("instrumented") is True and header.get("nested_spans") is True, "missing profile scope")
    require(type(header.get("pid")) is int and header["pid"] > 0, "invalid compiler PID")
    require(re.fullmatch(r"[0-9a-f]{40}", header.get("rustc", "")) is not None, "missing Rust identity")
    require(re.fullmatch(r"[0-9a-f]{16}", header.get("native", "")) is not None, "missing native identity")
    require(re.fullmatch(r"[0-9a-f]{16}", header.get("request_key", "")) is not None, "missing request identity")
    require(final.get("completed") is True, "incomplete compiler profile")
    for key in ("success", "reused", "check_only"):
        require(type(final.get(key)) is bool, f"missing {key} result")
    totals = {}
    spans = records[1:-1]
    for event in spans:
        phase = event.get("phase")
        require(phase in PHASES, f"unknown phase {phase}")
        start, end, elapsed = (event.get(key) for key in ("start_ns", "end_ns", "elapsed_ns"))
        require(all(type(value) is int for value in (start, end, elapsed)), "invalid phase duration")
        require(0 <= start <= end and elapsed == end - start, "inconsistent phase duration")
        totals[phase] = totals.get(phase, 0) + elapsed
    for phase in ("driver", "request-setup", "rustc-driver"):
        require(sum(event["phase"] == phase for event in spans) == 1, f"missing/duplicate {phase}")
    driver = next(event for event in spans if event["phase"] == "driver")
    require(all(driver["start_ns"] <= event["start_ns"] <= event["end_ns"] <= driver["end_ns"] for event in spans), "phase escaped driver span")
    return {"file": path.name, "sha256": digest(path), "identity": header, "result": final, "phase_nanoseconds": totals}


def summarize(root):
    profiles = root / "profiles"
    work = root / "qualification"
    indexes = {path.stem: path for path in work.glob("*.profiles")}
    require(set(indexes) == set(EXPECTED), "missing or unknown qualification profile labels")
    seen = set()
    invocations = {}
    modes = {}
    for label, expected_success in EXPECTED.items():
        names = indexes[label].read_text().split()
        require(len(names) == len(set(names)), f"duplicate profile in {label}")
        totals = {}
        mode = []
        for name in names:
            require(re.fullmatch(r"[0-9]+-[0-9]+\.jsonl", name) is not None, "unsafe profile path")
            require(name not in seen, "profile assigned to multiple commands")
            seen.add(name)
            result = read_profile(profiles / name)
            invocations[name] = result
            mode.append(result)
            for phase, ns in result["phase_nanoseconds"].items():
                totals[phase] = totals.get(phase, 0) + ns
        trace = (work / f"{label}.trace").read_text()
        observed_requests = re.findall(r"^request-key=([0-9a-f]{16}) retained=(?:true|false)$", trace, re.MULTILINE)
        require(Counter(observed_requests) == Counter(value["identity"]["request_key"] for value in mode),
                f"missing/extra compiler profile relative to independent request trace for {label}")
        if label in {"cold", "body", "arithmetic-edit", "signature", "link-failure", "link-failure-recovery", "invalid"}:
            require(mode, f"no compiler profile for {label}")
        if expected_success:
            require(all(value["result"]["success"] for value in mode), f"failed compiler hidden in successful {label}")
        else:
            require(any(not value["result"]["success"] for value in mode), f"missing failed compiler profile for {label}")
        if label in {"body", "arithmetic-edit", "link-failure-recovery"}:
            require(any(value["result"]["reused"] for value in mode), f"{label} lost retained execution")
        if label in {"body", "arithmetic-edit"}:
            require(totals.get("native-compile", 0) > 0, f"{label} did not profile its native edit")
        status = (work / f"{label}.status").read_text().strip()
        parsed_status = re.fullmatch(r"exit status: ([0-9]+) elapsed_ns=([0-9]+)", status)
        require(parsed_status is not None and int(parsed_status[2]) > 0, f"invalid command status for {label}")
        require((int(parsed_status[1]) == 0) == expected_success, f"incorrect command outcome for {label}")
        modes[label] = {"profiles": names, "expected_success": expected_success, "observed_command_status": status, "summed_phase_nanoseconds": totals}
    require(seen == {path.name for path in profiles.glob("*.jsonl")}, "unassigned or missing compiler profile")
    identities = {(v["identity"]["rustc"], v["identity"]["native"]) for v in invocations.values()}
    require(len(identities) == 1, "profiling mixed compiler identities")
    return {"schema": "lustr.mixed-phase-attribution", "version": 1, "instrumented": True,
            "paired_speed_evidence": False, "modes": modes, "invocations": invocations}


def render(report):
    columns = ["driver", "context-fingerprint", "native-compile", "native-symbol-rename",
               "temporary-link-copies", "original-linker", "edited-native-object-sync", "publish-certificate"]
    lines = ["# Mixed compiler phase attribution", "",
             "Instrumented execution of the existing mixed Cargo qualification. This is a separate diagnostic run, not paired speed evidence.", "",
             "Each cell sums one named phase across the compiler processes for that command, in milliseconds. Spans can nest and crates can run concurrently; columns must not be added into command wall time. Dynamic loading before driver entry and final process teardown are outside these spans. All original records and qualification stdout/stderr/status/trace files remain in this directory.", "",
             "| Command | Compiler invocations | " + " | ".join(columns) + " |",
             "|---|---:" + "|---:" * len(columns) + "|"]
    for label, mode in report["modes"].items():
        cells = [f"{mode['summed_phase_nanoseconds'].get(phase, 0) / 1e6:.3f}" for phase in columns]
        lines.append(f"| {label} | {len(mode['profiles'])} | " + " | ".join(cells) + " |")
    lines.extend(["", "The full JSON includes every named phase, per-process success/reuse/check state, exact compiler identities and record hashes. A zero in this table means no recorded duration for that phase. These observations do not isolate a before/after optimization effect or establish a compiler speedup.", ""])
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--qualifier", required=True, type=Path)
    parser.add_argument("--cargo", required=True, type=Path)
    parser.add_argument("--lustr", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    qualifier, cargo, lustr = (path.resolve(strict=True) for path in (args.qualifier, args.cargo, args.lustr))
    root = args.output.absolute()
    require(not root.exists(), "profiling output must be a new directory")
    root.mkdir(parents=True)
    profiles = root / "profiles"
    profiles.mkdir()
    command = [str(qualifier), str(root / "qualification"), str(cargo), str(lustr)]
    provenance = {"completed": False, "instrumented": True, "argv": command,
                  "binaries": {str(path): digest(path) for path in (qualifier, cargo, lustr, lustr.with_name("rustc-full"), lustr.with_name("lust-core"))}}
    (root / "run.json").write_text(json.dumps(provenance, indent=2) + "\n")
    env = dict(os.environ)
    env["LUSTR_MIXED_PROFILE_DIR"] = str(profiles)
    with (root / "qualifier.stdout").open("wb") as stdout, (root / "qualifier.stderr").open("wb") as stderr:
        process = subprocess.Popen(command, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
        try:
            code = process.wait(timeout=900)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise ValueError("profiling qualifier timed out; partial evidence retained")
    provenance["exit_code"] = code
    (root / "run.json").write_text(json.dumps(provenance, indent=2) + "\n")
    require(code == 0, "original mixed qualifier failed; partial evidence retained")
    report = summarize(root)
    (root / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    (root / "README.md").write_text(render(report))
    provenance["completed"] = True
    provenance["summary_sha256"] = digest(root / "summary.json")
    (root / "run.json").write_text(json.dumps(provenance, indent=2) + "\n")
    print(f"Validated {len(report['invocations'])} instrumented compiler invocations across {len(report['modes'])} qualification commands.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"mixed profiling failed: {error}", file=sys.stderr)
        sys.exit(1)
