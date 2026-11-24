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
    "native-compile", "native-request-setup", "native-daemon-roundtrip",
    "native-symbol-rename", "edited-native-object-sync",
    "retain-link-recipe", "retained-object-sync", "relink",
    "temporary-link-copies", "original-linker", "publish-certificate",
}
EMITTER_PHASES = (
    "c-prepare", "output-prepare", "c-write", "libtcc", "nm",
    "rename", "file-sync", "directory-sync",
)
EMITTER_PREFIX = "lustr mixed native profile: "


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def read_native_responses(records, spans, header):
    parents = [event for event in spans if event["phase"] == "native-compile"]
    require(len({event["start_ns"] for event in parents}) == len(parents), "ambiguous native request intervals")
    require(len(records) == len(parents), "missing or extra native response detail")
    seen = set()
    results = []
    for record in records:
        require(set(record) == {"kind", "parent_start_ns", "roundtrip", "unit_identity", "response_code", "emitter_status", "emitter_records"}, "unknown native response fields")
        start = record["parent_start_ns"]
        require(type(start) is int and start >= 0 and start not in seen, "invalid or duplicate native response binding")
        seen.add(start)
        parent = next((event for event in parents if event["start_ns"] == start), None)
        require(parent is not None, "native response has no enclosing request")
        unit = record["unit_identity"]
        require(type(unit) is str and re.fullmatch(r"[0-9a-f]{16}", unit) is not None, "invalid native unit identity")
        bounds = record["roundtrip"]
        require(type(bounds) is list and len(bounds) == 2 and all(type(n) is int for n in bounds), "missing native roundtrip bounds")
        roundtrips = [event for event in spans if event["phase"] == "native-daemon-roundtrip"
                      and [event["start_ns"], event["end_ns"]] == bounds]
        require(len(roundtrips) == 1, "native response has no unique roundtrip")
        roundtrip = roundtrips[0]
        require(parent["start_ns"] <= bounds[0] <= bounds[1] <= parent["end_ns"], "native roundtrip escaped its request")
        raw = record["emitter_records"]
        code = record["response_code"]
        require(type(code) is int and 0 <= code <= 255, "invalid native response code")
        require(type(raw) is list and all(type(line) is str for line in raw), "invalid native emitter records")
        result = {"parent_start_ns": start, "unit_identity": unit, "response_code": code,
                  "client_request_elapsed_ns": parent["elapsed_ns"],
                  "roundtrip_start_ns": bounds[0], "roundtrip_elapsed_ns": roundtrip["elapsed_ns"],
                  "emitter_status": record["emitter_status"]}
        if code != 0:
            require(record["emitter_status"] == "unavailable-failed-response" and not raw,
                    "failed native response must keep error diagnostics undecorated and mark emitter detail unavailable")
            results.append({**result, "emitter": None, "emitter_record": None, "protocol_unattributed_ns": None})
            continue
        require(record["emitter_status"] == "reported", "successful native response requires reported emitter detail")
        require(type(raw) is list and len(raw) == 1 and type(raw[0]) is str, "missing or duplicate native emitter record")
        emitter = json.loads(raw[0])
        require(type(emitter) is dict and set(emitter) == {"schema", "version", "native", "unit_identity", "success", "elapsed_ns", "phases"}, "unknown native emitter fields")
        require(emitter["schema"] == "lustr.mixed-native-emitter" and type(emitter["version"]) is int and emitter["version"] == 1, "unknown native emitter schema")
        require(emitter["native"] == header["native"] and emitter["unit_identity"] == unit, "native emitter identity mismatch")
        success, elapsed = emitter["success"], emitter["elapsed_ns"]
        require(success is True, "native emitter outcome mismatch")
        require(type(elapsed) is int and 0 <= elapsed <= roundtrip["elapsed_ns"], "native emitter duration exceeds its roundtrip")
        phases = emitter["phases"]
        require(type(phases) is list and 1 <= len(phases) <= len(EMITTER_PHASES), "incomplete native emitter phases")
        require(all(type(phase) is dict and set(phase) == {"phase", "elapsed_ns", "completed"} for phase in phases), "unknown native emitter phase fields")
        require([phase["phase"] for phase in phases] == list(EMITTER_PHASES[:len(phases)]), "invalid native emitter phase order")
        require(len(phases) == len(EMITTER_PHASES), "successful native emitter omitted phases")
        for phase in phases:
            require(type(phase["elapsed_ns"]) is int and phase["elapsed_ns"] >= 0, "invalid native emitter phase duration")
            require(phase["completed"] is True, "native emitter phase completion mismatch")
        require(sum(phase["elapsed_ns"] for phase in phases) == elapsed, "native emitter phases do not cover its duration")
        results.append({**result, "emitter": emitter, "emitter_record": raw[0],
                        "protocol_unattributed_ns": roundtrip["elapsed_ns"] - elapsed})
    return results


def read_profile(path):
    require(path.is_file() and not path.is_symlink(), f"invalid profile file: {path.name}")
    records = [json.loads(line) for line in path.read_text().splitlines()]
    require(len(records) >= 5, f"incomplete profile: {path.name}")
    require(all(type(record) is dict for record in records), "profile records must be objects")
    header, final = records[0], records[-1]
    require(header.get("schema") == "lustr.mixed-phase-profile" and type(header.get("version")) is int and header["version"] in (1, 2), "unknown profile schema")
    require(header.get("instrumented") is True and header.get("nested_spans") is True, "missing profile scope")
    require(type(header.get("pid")) is int and header["pid"] > 0, "invalid compiler PID")
    require(re.fullmatch(r"[0-9a-f]{40}", header.get("rustc", "")) is not None, "missing Rust identity")
    require(re.fullmatch(r"[0-9a-f]{16}", header.get("native", "")) is not None, "missing native identity")
    require(re.fullmatch(r"[0-9a-f]{16}", header.get("request_key", "")) is not None, "missing request identity")
    require(final.get("completed") is True, "incomplete compiler profile")
    for key in ("success", "reused", "check_only"):
        require(type(final.get(key)) is bool, f"missing {key} result")
    totals = {}
    responses = [event for event in records[1:-1] if event.get("kind") == "native-response"]
    spans = [event for event in records[1:-1] if event.get("kind") != "native-response"]
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
    if header["version"] == 1:
        require(not responses, "legacy profile contains unversioned native detail")
        native = None
    else:
        native = read_native_responses(responses, spans, header)
        parents = [event for event in spans if event["phase"] in {"native-check", "native-compile"}]
        for child in (event for event in spans if event["phase"] in {"native-request-setup", "native-daemon-roundtrip"}):
            require(sum(parent["start_ns"] <= child["start_ns"] <= child["end_ns"] <= parent["end_ns"] for parent in parents) == 1,
                    "native client phase has no unique enclosing request")
        for parent in parents:
            require(sum(event["phase"] == "native-request-setup" and parent["start_ns"] <= event["start_ns"] <= event["end_ns"] <= parent["end_ns"] for event in spans) == 1,
                    "missing or duplicate native request setup")
    return {"file": path.name, "sha256": digest(path), "identity": header, "result": final,
            "phase_nanoseconds": totals, "native_requests": native}


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
        versions = {value["identity"]["version"] for value in mode}
        require(len(versions) <= 1, "mixed profile schema versions in one command")
        emitter = None
        if versions == {2}:
            native = [request for value in mode for request in value["native_requests"]]
            measured = [request for request in native if request["emitter"] is not None]
            traced = [line[len(EMITTER_PREFIX):] for line in trace.splitlines() if line.startswith(EMITTER_PREFIX)]
            require(Counter(traced) == Counter(request["emitter_record"] for request in measured),
                    f"missing/extra native response relative to independent diagnostic trace for {label}")
            emitter_totals, emitter_counts = Counter(), Counter()
            for request in measured:
                for phase in request["emitter"]["phases"]:
                    emitter_totals[phase["phase"]] += phase["elapsed_ns"]
                    emitter_counts[phase["phase"]] += 1
            emitter = {"response_count": len(native), "measured_count": len(measured),
                       "unavailable_failed_count": len(native) - len(measured),
                       "elapsed_ns": sum(request["emitter"]["elapsed_ns"] for request in measured) if measured else None,
                       "phase_nanoseconds": dict(emitter_totals), "phase_observations": dict(emitter_counts),
                       "protocol_unattributed_ns": sum(request["protocol_unattributed_ns"] for request in measured) if measured else None}
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
        modes[label] = {"profiles": names, "expected_success": expected_success, "observed_command_status": status,
                        "summed_phase_nanoseconds": totals, "native_emitter": emitter}
    require(seen == {path.name for path in profiles.glob("*.jsonl")}, "unassigned or missing compiler profile")
    identities = {(v["identity"]["rustc"], v["identity"]["native"]) for v in invocations.values()}
    require(len(identities) == 1, "profiling mixed compiler identities")
    require(len({v["identity"]["version"] for v in invocations.values()}) == 1, "profiling mixed schema versions")
    return {"schema": "lustr.mixed-phase-attribution", "version": 2, "instrumented": True,
            "paired_speed_evidence": False, "modes": modes, "invocations": invocations}


def render(report):
    columns = ["driver", "context-fingerprint", "native-compile", "native-request-setup", "native-daemon-roundtrip", "native-symbol-rename",
               "temporary-link-copies", "original-linker", "edited-native-object-sync", "publish-certificate"]
    legacy = any(value["identity"]["version"] == 1 for value in report["invocations"].values())
    unrecorded = {"native-request-setup", "native-daemon-roundtrip"} if legacy else set()
    lines = ["# Mixed compiler phase attribution", "",
             "Instrumented execution of the existing mixed Cargo qualification. This is a separate diagnostic run, not paired speed evidence.", "",
             "Each cell sums one named phase across the compiler processes for that command, in milliseconds. Spans can nest and crates can run concurrently; columns must not be added into command wall time. Dynamic loading before driver entry and final process teardown are outside these spans. All original records and qualification stdout/stderr/status/trace files remain in this directory.", "",
             "| Command | Compiler invocations | " + " | ".join(columns) + " |",
             "|---|---:" + "|---:" * len(columns) + "|"]
    for label, mode in report["modes"].items():
        cells = ["not recorded" if phase in unrecorded else f"{mode['summed_phase_nanoseconds'].get(phase, 0) / 1e6:.3f}" for phase in columns]
        lines.append(f"| {label} | {len(mode['profiles'])} | " + " | ".join(cells) + " |")
    lines.extend(["", "The full JSON includes every named phase, per-process success/reuse/check state, exact compiler identities and record hashes. A zero in this table means no recorded duration for that phase. These observations do not isolate a before/after optimization effect or establish a compiler speedup.", ""])
    remote_columns = ["emitter total", *EMITTER_PHASES, "roundtrip outside emitter"]
    lines.extend(["## Native emitter durations", "",
                  "These durations come from the daemon response, separately from the driver clock. Each record is bound to its actual client request and successful protocol exchange, checked against that exchange's elapsed time, and matched to the original diagnostic trace. The daemon's sequential phase durations cover its emitter total; they do not partition the enclosing driver span.", "",
                  "Roundtrip outside emitter remains unattributed. It includes protocol transport and queueing, native frontend and generated-code work before the emitter, retained-state work after it, guard handling, and response production. It is not a frontend or codegen measurement. Client request setup is recorded separately; retries and daemon startup can also occupy the enclosing request.", "",
                  "| Command | Responses / measured / unavailable failures | " + " | ".join(remote_columns) + " |",
                  "|---|---:" + "|---:" * len(remote_columns) + "|"])
    for label, mode in report["modes"].items():
        native = mode["native_emitter"]
        if native is None:
            cells, count = ["not recorded"] * len(remote_columns), "not recorded"
        elif native["response_count"] == 0:
            cells, count = ["n/a"] * len(remote_columns), "0 / 0 / 0"
        else:
            count = f"{native['response_count']} / {native['measured_count']} / {native['unavailable_failed_count']}"
            if native["measured_count"] == 0:
                cells = ["unavailable"] * len(remote_columns)
            else:
                cells = [f"{native['elapsed_ns'] / 1e6:.3f}"]
                cells.extend(f"{native['phase_nanoseconds'][phase] / 1e6:.3f}" for phase in EMITTER_PHASES)
                cells.append(f"{native['protocol_unattributed_ns'] / 1e6:.3f}")
        lines.append(f"| {label} | {count} | " + " | ".join(cells) + " |")
    lines.extend(["", "Values are milliseconds summed only across the measured successful responses. Failed native responses retain their original error diagnostics, actual response code and complete client intervals, with emitter detail and outside-emitter residual explicitly unavailable. Their remote durations are null, never zero. Timing text must not alter the daemon's byte-for-byte comparison of failed warm/fresh responses. Missing success detail or unexpected failure decoration fails profiling qualification. Version 1 archives did not collect these records and remain labeled not recorded. The single-unit libtcc span includes its in-process compiler setup, compilation, object write and teardown; nm includes its child process. Rename and both synchronization operations remain unchanged.", ""])
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
