#!/usr/bin/env python3
"""Compare exact before/after public entries with pristine Lust and Cargo.

The two public entries share one source-built native runtime and Rust SDK. The
baseline entry is rebuilt from its pinned source with the same build recipe.
This measures entry changes only; it is not a second complete toolchain build.
"""
import argparse
from contextlib import ExitStack
import csv
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import random
import shutil
import statistics
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("public_slice", HERE / "run-upstream-slice-benchmark.py")
S = importlib.util.module_from_spec(spec)
spec.loader.exec_module(S)
require = S.require
BASELINE = "d57f485eeb2b638d1f95a0594294d11e8001fe8e"
SCHEMA = "lustr-warm-entry-comparison-v1"
LANES = ("baseline", "entry", "pristine", "cargo")
PHASES = ("cold-check", "no-op-check", "cold-build", "no-op-build")
SEED = 0x5741524D
SHARED_SOURCE_PATHS = ("compiler/native", "crates/lustr-native-core", "library", "upstream/bluefe-core.pin",
                       "src/host_runtime.rs", "tools/native-tcc-input-observer.rs", "upstream/runtime")


def git(source, *args):
    return subprocess.check_output(["git", "-C", str(source), *args], env={
        "PATH": "/usr/local/bin:/usr/bin:/bin", "LANG": "C", "LC_ALL": "C",
        "GIT_NO_REPLACE_OBJECTS": "1"}).decode().strip()


def source_identity(source):
    require(not git(source, "status", "--porcelain", "--untracked-files=no"), "tracked source changed")
    return {"revision": git(source, "rev-parse", "HEAD"), "tree": git(source, "rev-parse", "HEAD^{tree}")}


def selection_seed(data, executable, receipt):
    """Bind an archived constructor record to its actual installed launcher."""
    record = json.loads(data)
    payload, tools = record["payload"], record["payload"]["tools"]
    encoded = lambda value: json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
    require(record["sha256"] == hashlib.sha256(encoded(payload)).hexdigest()
            and payload["schema"] == "lustr-cargo-selection-v2"
            and tools["identities"][0][:2] == [str(executable), str(executable)]
            and tools["receipt"].encode() == receipt, "selection seed belongs to another launcher or receipt")
    return "tools-" + hashlib.sha256(encoded(tools)).hexdigest() + ".json"








def orders(pairs):
    require(12 <= pairs <= 96 and pairs % 4 == 0, "pairs must be a multiple of four in 12..96")
    rng, schedule = random.Random(SEED), []
    while len(schedule) < pairs:
        lanes, rotations = list(LANES), list(range(4))
        rng.shuffle(lanes)
        rng.shuffle(rotations)
        schedule.extend(lanes[n:] + lanes[:n] for n in rotations)
    return schedule


def summaries(samples, pairs):
    require(len(samples) == pairs * len(LANES) * len(PHASES), "missing or extra samples")
    values = {}
    for row in samples:
        key = (row["sample"], row["lane"], row["phase"])
        require(key not in values and type(key[0]) is int and 0 <= key[0] < pairs and key[1] in LANES and key[2] in PHASES
                and S.STATS.positive(row["seconds"]), "invalid or duplicate sample")
        values[key] = row["seconds"]
    result = []
    for phase in PHASES:
        for baseline, candidate in (("baseline", "entry"), ("pristine", "baseline"), ("pristine", "entry"), ("cargo", "entry")):
            paired = [(values[i, baseline, phase], values[i, candidate, phase]) for i in range(pairs)]
            result.append({"phase":phase, "baseline":baseline, "candidate":candidate,
                           "analysis":S.STATS.summarize(paired, 10000, SEED)})
    return result


def parity(comparisons):
    result = {}
    for phase in ("no-op-check", "no-op-build"):
        row, = [x for x in comparisons if x["phase"] == phase and x["baseline"] == "pristine" and x["candidate"] == "entry"]
        a = row["analysis"]
        # Exact non-regression target. An interval overlapping one is inconclusive,
        # not evidence of equivalence. Keep the measured ratio even when it passes.
        result[phase] = {"meets_target":a["bootstrap_95"][0] >= 1.0,
                         "slowdown_median":a["candidate_median"] / a["baseline_median"],
                         "speed_ratio":a["geometric_ratio"], "speed_ratio_bootstrap_95":a["bootstrap_95"]}
    return {"criterion":"Both warm modes: lower conditional paired-bootstrap 95% speed ratio >= 1.0; no equivalence claim from overlapping intervals.",
            "meets_target":all(row["meets_target"] for row in result.values()), "modes":result}


def priming_summary(records, pairs):
    rows = [row for row in records if row["kind"] == "excluded-prime"]
    require(len(rows) == pairs * len(LANES) * 2 * 3, "missing or extra priming commands")
    values = {}
    for row in rows:
        key = (row["sample"], row["lane"], row["operation"], row["ordinal"])
        require(key not in values and 0 <= key[0] < pairs and key[1] in LANES and key[2] in ("check","build")
                and key[3] in (1,2,3) and S.STATS.positive(row["seconds"]), "invalid priming command")
        values[key] = row["seconds"]
    return [{"lane":lane, "operation":operation, "ordinal":ordinal,
             "median_seconds":statistics.median(values[sample,lane,operation,ordinal] for sample in range(pairs))}
            for operation in ("check","build") for lane in LANES for ordinal in (1,2,3)]


def retained_proof(data, operation):
    """Keep answer-reuse checks strict and distinguish cached-object builds.

    compile.rs always assembles main/launcher unit 0 for builds. Its C bytes
    are counted before object-cache lookup, so a retained build can report
    nonzero c_bytes while every generated unit is served from that cache.
    """
    require(operation in ("check", "build"), "unknown retained operation")
    def fields(prefix):
        rows = [line[len(prefix):] for line in data.decode().splitlines() if line.startswith(prefix)]
        require(len(rows) == 1, "missing or repeated native diagnostic record")
        result = {}
        for item in rows[0].split():
            key, separator, value = item.partition("=")
            require(separator and key and value and key not in result, "malformed native diagnostic field")
            result[key] = value
        return result
    stats, visits = fields("lust-stats: "), fields("lust-visits: ")
    def number(row, key):
        value = row.get(key, "")
        require(value.isascii() and value.isdecimal() and int(value) < 2**64, "invalid native work counter")
        return int(value)
    counters = {key:number(stats, key) for key in ("fns", "rechecked", "invalidated", "c_bytes", "dirty_units",
                "units_compiled", "unit_cache_hits", "table_rebuilt", "reemitted", "templates", "discovered")}
    counters["checked"] = number(visits, "checked")
    route = [stats.get("mode"), visits.get("path"), visits.get("why")]
    if operation == "check":
        require(route == list(S.retained_check(data)), "retained check route differs")
        require(counters["templates"] == counters["table_rebuilt"] == 0,
                "retained check repeated template or table work")
    else:
        require(counters["fns"] > 0, "empty retained fixture")
        require(all(counters[key] == 0 for key in ("rechecked", "invalidated", "dirty_units", "units_compiled",
                    "table_rebuilt", "reemitted", "templates", "discovered", "checked")),
                "retained build repeated body work or compiled new objects")
        require(route in (["bodies", "patch", "-"], ["bodies", "chunks", "bodies"]),
                "build did not use a known retained route")
        require(counters["c_bytes"] == 0 or counters["unit_cache_hits"] > 0,
                "retained build C has no cached-object evidence")
    return {"route":route, "counters":counters}


def edit_variants(original):
    anchor = b"fn main() { bench_driver::run(); }"
    require(original.count(anchor) == 1, "missing unique public-slice edit anchor")
    return {"initial":original,
            "edited":original.replace(anchor, b'fn main() { println!("selection-cache-edited"); bench_driver::run(); }'),
            "invalid":original.replace(anchor, b'fn main() { let _value: u32 = "not an integer"; bench_driver::run(); }'),
            "restored":original}


class Campaign(S.Campaign):
    def __init__(self, args):
        super().__init__(args)
        self.tools["baseline"] = args.built / "toolchain/bin/lustr-warm-baseline"
        self.report.update(schema=SCHEMA, phases=list(PHASES), lanes=list(LANES), seed=SEED,
                           scope="Identical public slice with pinned baseline and candidate public entries, pristine Lust and Rust/Cargo.",
                           limits="Entry-only before/after comparison with identical native runtime and SDK. Broader compiler qualification remains separate.")

    def compile(self, lane, operation, state, env, phase, fresh=False, trace=False):
        if lane != "baseline" or not trace:
            return super().compile(lane, operation, state, env, phase, fresh, trace)
        local = dict(env)
        trace_path = state / f"entry-{len(self.recorder.records)}.trace"
        local["LUSTR_TRACE_PATH"] = str(trace_path)
        result, _, _ = self.recorder.run(self.command(lane, operation, state), local, self.fixture, lane, phase)
        rows = [json.loads(line) for line in trace_path.read_bytes().splitlines()]
        expected = "upstream-process" if "LUST_NO_DAEMON" in env else "daemon"
        require(rows == [{"engine":"lust", "entry":expected}], "baseline public entry did not use the required native route")
        archived = self.output / "commands" / f"{result['id']:05}.entry.trace"
        shutil.copyfile(trace_path, archived)
        self.recorder.event({"kind":"entry-route", "command":result["id"], "phase":phase, "records":rows,
                             "trace":{"file":str(archived.relative_to(self.output)), "sha256":S.digest(archived), "bytes":archived.stat().st_size}})
        binary = state / "program" if operation == "build" else None
        if binary is not None:
            require(binary.is_file() and not binary.is_symlink(), "baseline build did not produce an ordinary executable")
            self.built_artifacts[binary] = {"build_command":result["id"], "binary_sha256":S.digest(binary)}
            self.recorder.event({"kind":"build-identity", "binary":str(binary), **self.built_artifacts[binary]})
        return result, binary

    def daemon_tool(self, lane):
        return self.tools["pristine"] if lane == "pristine" else self.tools["integrated"]

    def qualification(self, sockets, stage):
        self.recorder.event({"kind":"qualification-start", "stage":stage})
        for lane in ("cargo", "pristine", "baseline", "entry"):
            state = self.output / "work" / stage / "runtime" / lane
            env = self.environment(state, sockets / (lane + ".sock"), True)
            _, binary = self.compile(lane, "build", state, env, "gate-build", trace=True)
            self.runtime(lane, binary, env, "gate-runtime")
        for lane in ("pristine", "baseline", "entry"):
            for operation in ("check", "build"):
                state = self.output / "work" / stage / ("retained-" + operation) / lane
                env = dict(self.environment(state, sockets / (lane + ".sock")), LUST_STATS="1")
                selection_trace = state / "selection-events.jsonl"
                if lane in ("baseline", "entry"):
                    env["LUSTR_SELECTION_DIAGNOSTICS"] = str(selection_trace)
                with S.Daemon(self.recorder, self.daemon_tool(lane), env, self.fixture, lane, f"{stage}-{operation}-{lane}") as daemon:
                    self.compile(lane, operation, state, env, "gate-warm-prime", trace=True)
                    if lane in ("baseline", "entry"):
                        for _ in range(3):
                            self.compile(lane, operation, state, env, "gate-selection-prime")
                        before = selection_trace.read_bytes()
                        command, _ = self.compile(lane, operation, state, env, "gate-selection-hit")
                        after = selection_trace.read_bytes()
                        require(after.startswith(before), "selection diagnostics were replaced")
                        rows = [json.loads(line) for line in after[len(before):].splitlines()]
                        require(rows == [{"selection_cache":"hit"}], "warm public entry did not reuse exact target selection")
                        archive = self.output / "commands" / f"{command['id']:05}.selection.trace"
                        archive.write_bytes(after[len(before):])
                        self.recorder.event({"kind":"selection-cache", "lane":lane, "command":command["id"], "operation":operation,
                                             "trace":{"file":str(archive.relative_to(self.output)), "sha256":S.digest(archive), "bytes":archive.stat().st_size}})
                    command, _, stderr = self.recorder.run(self.command(lane, operation, state) + ["--timings"],
                                                          env, self.fixture, lane, "gate-retained-" + operation)
                    proof = retained_proof(stderr, operation)
                    self.recorder.event({"kind":"retained-route", "command":command["id"], "lane":lane, "operation":operation, **proof})
                    if operation == "build":
                        binary = state / "program"
                        self.built_artifacts[binary] = {"build_command":command["id"], "binary_sha256":S.digest(binary)}
                        self.recorder.event({"kind":"build-identity", "binary":str(binary), **self.built_artifacts[binary]})
                        self.runtime(lane, state / "program", env, "gate-retained-runtime")
                    daemon.alive()
        reject = self.output / "rejected.rs"
        for lane in LANES:
            state = self.output / "work" / stage / "rejection" / lane
            env = self.environment(state, sockets / (lane + ".sock"), True)
            command = ([self.reference, reject, "--edition=2024", "--emit=metadata", "-o", state / "reject.rmeta", *S.FLAGS]
                       if lane == "cargo" else [self.tools[lane], "check", reject, *S.FLAGS])
            _, out, err = self.recorder.run(command, env, self.fixture, lane, "gate-reject", 1)
            require(b"E0308" in out + err or b"L0110" in out + err, "wrong rejection diagnostic")
        self.recorder.event({"kind":"qualification", "stage":stage, "completed":True})

    def cache_edit_qualification(self, sockets):
        """Actual cached public commands must still see edits and rejection."""
        main = self.fixture / "src/slice_main.rs"
        original = main.read_bytes()
        source = S.inventory(self.fixture)
        states = {lane:self.output / "work/cache-edits" / lane for lane in ("cargo", "entry")}
        envs = {lane:self.environment(states[lane], sockets / ("cache-edits-" + lane + ".sock")) for lane in states}
        selection_path, entry_path = states["entry"] / "selection.jsonl", states["entry"] / "entry.jsonl"
        envs["entry"].update(LUST_STATS="1", LUSTR_SELECTION_DIAGNOSTICS=str(selection_path), LUSTR_TRACE_PATH=str(entry_path))
        def archive(command, suffix, data):
            path = self.output / "commands" / f"{command['id']:05}.{suffix}"
            path.write_bytes(data)
            return {"file":str(path.relative_to(self.output)), "sha256":S.digest(path), "bytes":len(data)}
        self.recorder.event({"kind":"cache-edit-start"})
        try:
            with S.Daemon(self.recorder, self.daemon_tool("entry"), envs["entry"], self.fixture, "entry", "cache-edits") as daemon:
                for operation in ("check", "build"):
                    for _ in range(3):
                        self.compile("entry", operation, states["entry"], envs["entry"], "cache-edit-prime")
                for stage, contents in edit_variants(original).items():
                    main.write_bytes(contents)
                    snapshot = self.output / "commands" / f"cache-edit-{stage}.rs"
                    snapshot.write_bytes(contents)
                    observed = S.inventory(self.fixture)
                    require(observed == dict(source, **{"src/slice_main.rs":S.digest(main)}), "edit changed another source input")
                    self.recorder.event({"kind":"cache-edit-source", "stage":stage,
                                         "source":{"file":str(snapshot.relative_to(self.output)), "sha256":S.digest(snapshot), "bytes":len(contents)},
                                         "fixture_files":observed})
                    commands, runs = {}, {}
                    for operation in ("check", "build"):
                        for lane in ("cargo", "entry"):
                            state, env = states[lane], envs[lane]
                            before_selection = selection_path.read_bytes() if lane == "entry" else b""
                            before_entry = entry_path.read_bytes() if lane == "entry" else b""
                            binary = state / (f"target/{S.TARGET}/debug/slice" if lane == "cargo" else "program")
                            if operation == "build" and binary.exists():
                                binary.unlink()
                            phase = f"cache-edit-{stage}-{operation}"
                            expected = 101 if stage == "invalid" else 0
                            command, out, err = self.recorder.run(self.command(lane, operation, state), env, self.fixture, lane, phase, expected)
                            commands[lane + "-" + operation] = command["id"]
                            if lane == "entry":
                                selection, entry = selection_path.read_bytes(), entry_path.read_bytes()
                                require(selection.startswith(before_selection) and entry.startswith(before_entry), "edit diagnostics were replaced")
                                selection, entry = selection[len(before_selection):], entry[len(before_entry):]
                                require([json.loads(line) for line in selection.splitlines()] == [{"selection_cache":"hit"}], "body edit did not retain target selection")
                                expected_route = ([{"engine":"lust-rejected", "entry":"daemon"}, {"engine":"full-rust-core", "entry":"cargo"}]
                                                  if stage == "invalid" else [{"engine":"lust", "entry":"daemon"}])
                                require([json.loads(line) for line in entry.splitlines()] == expected_route, "edit escaped retained native compilation")
                                self.recorder.event({"kind":"cache-edit-proof", "command":command["id"], "stage":stage, "operation":operation,
                                                     "selection":archive(command, "edit-selection.trace", selection), "entry":archive(command, "edit-entry.trace", entry),
                                                     "retained":retained_proof(err, operation) if stage == "initial" else None})
                            if stage == "invalid":
                                require(b"E0308" in out + err, "cached invalid body has the wrong rejection")
                                require(operation != "build" or not binary.exists(), "rejected edit left an executable")
                            elif lane == "cargo":
                                S.cargo_artifact(out, operation, state / "target", False)
                            if operation == "build" and stage != "invalid":
                                require(binary.is_file() and not binary.is_symlink(), "edited build did not produce an ordinary executable")
                                built = {"build_command":command["id"], "binary_sha256":S.digest(binary)}
                                self.recorder.event({"kind":"build-identity", "binary":str(binary), **built})
                                run, stdout, stderr = self.recorder.run([binary], env, self.fixture, lane, "cache-edit-execute")
                                self.recorder.event({"kind":"runtime-identity", "command":run["id"], "binary":str(binary), **built})
                                runs[lane] = run["id"]
                                expected_output = (b"selection-cache-edited\n" if stage == "edited" else b"") + self.expected_output
                                require(stdout == expected_output and stderr == b"", "edited runtime differs from the required Rust output")
                            daemon.alive()
                    self.recorder.event({"kind":"cache-edit-step", "stage":stage, "commands":commands, "runs":runs})
        finally:
            main.write_bytes(original)
        require(S.inventory(self.fixture) == source, "edit gate did not restore the original source")
        self.recorder.event({"kind":"cache-edit-completed"})

    def measure_group(self, sample, order, operation, sockets):
        self.recorder.event({"kind":"group-start", "sample":sample, "operation":operation, "order":order})
        with ExitStack() as stack:
            daemons, states, environments = {}, {}, {}
            for lane in order:
                state = self.output / "work" / f"{sample:03}-{operation}" / lane
                require(not state.exists(), "cold state was reused")
                env = self.environment(state, sockets / (lane + ".sock"), True)
                states[lane] = state
                result, binary = self.compile(lane, operation, state, env, "cold-" + operation)
                self.timing(sample, lane, "cold-" + operation, [result])
                if binary is not None:
                    self.runtime(lane, binary, env, "cold-build-runtime")
            for lane in order:
                env = self.environment(states[lane], sockets / (lane + ".sock"))
                environments[lane] = env
                if lane != "cargo":
                    daemons[lane] = stack.enter_context(S.Daemon(self.recorder, self.daemon_tool(lane), env, self.fixture, lane, f"{sample}-{operation}-{lane}"))
                for ordinal in range(1,4):
                    result, _ = self.compile(lane, operation, states[lane], env, "excluded-prime", fresh=lane == "cargo")
                    self.recorder.event({"kind":"excluded-prime", "sample":sample, "lane":lane, "operation":operation,
                                         "ordinal":ordinal, "seconds":result["seconds"], "command":result["id"]})
            for lane in order:
                if lane in daemons:
                    daemons[lane].alive()
                result, binary = self.compile(lane, operation, states[lane], environments[lane], "no-op-" + operation, fresh=lane == "cargo")
                self.timing(sample, lane, "no-op-" + operation, [result])
                if binary is not None:
                    self.runtime(lane, binary, environments[lane], "no-op-build-runtime")
                if lane in daemons:
                    daemons[lane].alive()
        self.recorder.event({"kind":"group-end", "sample":sample, "operation":operation})



def render(report):
    lines = ["# Warm public-entry comparison", "", f"Status: {report['status']}.", "", report["scope"], "", report["limits"], ""]
    if report["status"] != "completed":
        return "\n".join(lines + ["No completed performance result. All original streams and failure records remain retained.", ""])
    lines += ["All times are measured end-to-end command medians in milliseconds.", "",
              f"Baseline public source: `{report['baseline_composition']['baseline']['revision']}`. Both installed public launchers run their own source-built constructor tool initialization before the campaign; their separate seed records are retained.", "",
              "| Operation | Baseline Lustr | Candidate Lustr | Pristine Lust | rustc via Cargo |", "| --- | ---: | ---: | ---: | ---: |"]
    for phase in PHASES:
        values = [statistics.median(row["seconds"] for row in report["samples"] if row["lane"] == lane and row["phase"] == phase) * 1000 for lane in LANES]
        lines.append(f"| {phase} | " + " | ".join(f"{value:.3f}" for value in values) + " |")
    lines += ["", "Each lane then primes its new warm state with three complete commands before the measured no-op. These costs remain visible below, in milliseconds. The first native prime can include daemon initialization and selection-cache population; this table does not isolate either component.", "",
              "| Operation | Lane | First prime | Second prime | Third prime |", "| --- | --- | ---: | ---: | ---: |"]
    for operation in ("check","build"):
        for lane in LANES:
            values = [row["median_seconds"] * 1000 for row in report["priming_summary"] if row["operation"] == operation and row["lane"] == lane]
            lines.append(f"| {operation} | {lane} | " + " | ".join(f"{value:.3f}" for value in values) + " |")
    lines += ["", "Ratios are paired reference/candidate geometric means; values above one favor the candidate.", "",
              "| Operation | Reference | Candidate | Ratio | Conditional bootstrap 95% interval |", "| --- | --- | --- | ---: | --- |"]
    for row in report["summaries"]:
        a = row["analysis"]
        lines.append(f"| {row['phase']} | {row['baseline']} | {row['candidate']} | {a['geometric_ratio']:.4f} | [{a['bootstrap_95'][0]:.4f}, {a['bootstrap_95'][1]:.4f}] |")
    lines += ["", f"Warm target met: {report['warm_target']['meets_target']}.", "", report["warm_target"]["criterion"], "",
              "Retained checks require zero generated C. Retained builds require zero body rechecks, dirty function units and newly compiled objects; they may reassemble launcher/support C whose object is served from cache. Full counters and exact runtime-output checks are retained in the raw evidence.", "",
              "Before timing, actual public-entry selection-cache hits are exercised through a valid body edit, a type-error edit, and restoration. Check/build exit statuses match Rust/Cargo; successful edited builds match its exact runtime output. These are correctness gates, not edit-speed measurements.", "",
              "Intervals are conditional exploratory estimates on this runner. Overlap with one does not prove equality. Full compiler correctness and release admission are separate gates.", ""]
    return "\n".join(lines)


def verify(output):
    report = json.loads((output / "report.json").read_text())
    require(report.get("schema") == SCHEMA and report.get("status") == "completed", "only complete campaigns have results")
    require(S.digest(output / "raw.jsonl") == report["raw_sha256"], "raw log changed")
    records = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
    def stream(record):
        p = Path(record["file"])
        require(not p.is_absolute() and ".." not in p.parts and not (output / p).is_symlink(), "invalid evidence path")
        data = (output / p).read_bytes()
        require(S.digest(output / p) == record["sha256"] and ("bytes" not in record or len(data) == record["bytes"]), "stream changed")
        return data
    identity, = [row for row in records if row["kind"] == "identities"]
    require(identity == {"kind":"identities", **{key:report[key] for key in
                         ("tool_paths", "compiler_and_runner_sha256", "fixture_files", "candidate_identity", "baseline_composition", "build_receipts", "rustc_version")}}, "source/build identity record differs")
    require(f"commit-hash: {S.RUST}" in report["rustc_version"].splitlines(), "Rust pin differs")
    for receipt in report["build_receipts"].values(): stream(receipt)
    before = json.loads(stream(report["build_receipts"]["receipt.json"]))
    require(before == report["baseline_composition"] and before["status"] == "completed" and before["baseline"]["revision"] == BASELINE
            and before["candidate"] == report["candidate_identity"], "baseline composition differs")
    require(before["baseline_binary"]["path"] == report["tool_paths"]["baseline"] and before["baseline_binary"]["sha256"] ==
            report["compiler_and_runner_sha256"][report["tool_paths"]["baseline"]], "baseline binary binding differs")
    selection_receipt = str(Path(report["tool_paths"]["entry"]).parent.parent / "lib/lustr-selection-tools.txt")
    require(report["build_receipts"]["lustr-selection-tools.txt"]["sha256"] == before["shared_package_sha256"][selection_receipt],
            "selection-toolchain receipt bytes differ from the executed package")
    seed = before["baseline_seed"]
    require(seed["argv"] == [report["tool_paths"]["baseline"], "--lustr-prime-selection-tools"] and seed["exit"] == 0
            and seed["cwd"] == str(Path(report["tool_paths"]["entry"]).parent.parent), "baseline constructor command differs")
    require(set(seed["streams"]) == {"stdout", "stderr"}, "baseline constructor streams are incomplete")
    for binding in seed["streams"].values():
        archived = report["build_receipts"][binding["file"]]
        require(archived["sha256"] == binding["sha256"] and stream(archived) == b"", "baseline constructor stream differs")
    for label, lane in (("baseline", "baseline"), ("candidate", "entry")):
        binding = seed[label + "_certificate"]
        archived = report["build_receipts"][binding["file"]]
        require(archived["sha256"] == binding["sha256"] == before["shared_package_sha256"][binding["path"]], "constructor certificate binding differs")
        name = selection_seed(stream(archived), report["tool_paths"][lane], stream(report["build_receipts"]["lustr-selection-tools.txt"]))
        require(binding["path"] == str(Path(selection_receipt).parent / "lustr-selection-verified" / name), "constructor seed path differs")
    commands, retained, active, built, runtime_commands, runtime_builds, selection = {}, [], {}, {}, set(), {}, []
    edit_active, edit_done, edit_sources, edit_proofs, edit_steps = False, False, [], [], []
    variants = edit_variants((output / "fixture/src/slice_main.rs").read_bytes())
    group = None
    oracle = (output / "runtime-oracle.stdout").read_bytes()
    require(oracle and S.digest(output / "runtime-oracle.stdout") == report["runtime_oracle_sha256"], "runtime oracle changed")
    for index, row in enumerate(records):
        if row["kind"] == "command":
            require(row["id"] == index and row["id"] not in commands and row["exit"] == row["expected_exit"]
                    and row["signal"] is None and not row["timed_out"] and not row["output_limit"] and not row.get("launch_error"), "failed or unbound command")
            stream(row["stdout"]); stream(row["stderr"])
            commands[row["id"]] = row
            if row["phase"].endswith("runtime"):
                require(stream(row["stdout"]) == oracle and stream(row["stderr"]) == b"", "runtime differs from Rust")
            if row["phase"] == "gate-reject":
                require(row["exit"] == 1 and (b"E0308" in stream(row["stdout"]) + stream(row["stderr"])
                        or b"L0110" in stream(row["stdout"]) + stream(row["stderr"])), "rejection differs")
        if row["kind"] == "build-identity":
            require(row["build_command"] in commands and commands[row["build_command"]]["exit"] == 0, "build identity lacks successful command")
            built[row["binary"]] = {key:row[key] for key in ("build_command", "binary_sha256")}
        if row["kind"] == "runtime-identity":
            require(row["binary"] in built and built[row["binary"]] == {key:row[key] for key in ("build_command", "binary_sha256")}, "runtime used another artifact")
            require(commands[row["command"]]["argv"] == [row["binary"]], "runtime command differs")
            require(row["command"] not in runtime_commands, "duplicate runtime identity")
            runtime_commands.add(row["command"])
            runtime_builds[row["command"]] = row["build_command"]
        if row["kind"] == "group-start":
            require(group is None, "measurement groups overlap")
            group = row
        if row["kind"] == "group-end":
            require(group is not None and row["sample"] == group["sample"] and row["operation"] == group["operation"], "measurement group end differs")
            group = None
        if row["kind"] == "daemon-start":
            require(row["lane"] not in active, "daemon lane already active")
            active[row["lane"]] = row
        if row["kind"] == "daemon-stop":
            require(row["lane"] in active and active[row["lane"]]["pid"] == row["pid"]
                    and active[row["lane"]]["scope"] == row["scope"], "daemon owner differs")
            del active[row["lane"]]
            require(row["exit"] == 0 and row["error"] is None, "daemon failed")
            for item in row["streams"]: stream(item)
        if row["kind"] == "retained-route":
            command = commands[row["command"]]
            require(command["phase"] == "gate-retained-" + row["operation"], "wrong retained proof")
            require(retained_proof(stream(command["stderr"]), row["operation"]) == {key:row[key] for key in ("route", "counters")}, "retained proof differs")
            require(row["lane"] in active and active[row["lane"]]["environment"].get("LUST_STATS") == "1", "retained daemon not instrumented")
            retained.append((row["lane"], row["operation"]))
        if row["kind"] == "cache-edit-start":
            require(not edit_active and not edit_done and group is None, "edit gate overlaps another campaign phase")
            edit_active = True
        if row["kind"] == "cache-edit-source":
            require(edit_active and "entry" in active and active["entry"]["scope"] == "cache-edits", "edit has no retained owned daemon")
            require(row["stage"] in variants and stream(row["source"]) == variants[row["stage"]], "edit source snapshot differs")
            require(row["fixture_files"] == dict(report["fixture_files"], **{"src/slice_main.rs":row["source"]["sha256"]}), "edit changed another input")
            edit_sources.append(row["stage"])
        if row["kind"] == "cache-edit-proof":
            command = commands[row["command"]]
            require(edit_active and edit_sources and row["stage"] == edit_sources[-1]
                    and command["lane"] == "entry" and command["phase"] == f"cache-edit-{row['stage']}-{row['operation']}", "edit proof is misbound")
            require("entry" in active and command["environment"] == active["entry"]["environment"], "edit command lost its retained daemon environment")
            require([json.loads(line) for line in stream(row["selection"]).splitlines()] == [{"selection_cache":"hit"}], "edit used fresh target selection")
            route = ([{"engine":"lust-rejected", "entry":"daemon"}, {"engine":"full-rust-core", "entry":"cargo"}]
                     if row["stage"] == "invalid" else [{"engine":"lust", "entry":"daemon"}])
            require([json.loads(line) for line in stream(row["entry"]).splitlines()] == route, "edit bypassed its native daemon")
            require(row["retained"] == (retained_proof(stream(command["stderr"]), row["operation"]) if row["stage"] == "initial" else None), "initial edit state was not retained")
            edit_proofs.append((row["stage"], row["operation"], row["command"]))
        if row["kind"] == "cache-edit-step":
            require(edit_active and edit_sources and row["stage"] == edit_sources[-1], "edit step escaped its source state")
            require(set(row["commands"]) == {lane + "-" + op for lane in ("cargo", "entry") for op in ("check", "build")}, "edit omitted a compiler operation")
            for operation in ("check", "build"):
                for lane in ("cargo", "entry"):
                    command = commands[row["commands"][lane + "-" + operation]]
                    env, fixture = command["environment"], Path(command["cwd"])
                    state = Path(env["CARGO_TARGET_DIR"]).parent
                    require(command["lane"] == lane and command["phase"] == f"cache-edit-{row['stage']}-{operation}"
                            and command["exit"] == (101 if row["stage"] == "invalid" else 0), "edited compilation status differs")
                    require(state.parts[-2:] == ("cache-edits", lane) and "LUST_NO_DAEMON" not in env and "LUST_NO_CACHE" not in env, "edit state is not retained")
                    require(env["LUST_CACHE_DIR"] == str(state / "native-cache") and env["CARGO_HOME"] == str(state / "cargo-home"), "edit cache directories differ")
                    if lane == "cargo":
                        expected = [report["tool_paths"][lane], operation, "--offline", "--locked", "--manifest-path", str(fixture / "Cargo.toml"),
                                    "--bin", "slice", "--target", S.TARGET, "--message-format=json-render-diagnostics"]
                        if row["stage"] != "invalid": S.cargo_artifact(stream(command["stdout"]), operation, state / "target", False)
                    else:
                        expected = [report["tool_paths"][lane], operation, str(fixture / "src/slice_main.rs"), "--manifest-path", str(fixture / "Cargo.toml"),
                                    "--bin", "slice", "--offline", "--locked", *S.FLAGS]
                        if operation == "build": expected += ["-o", str(state / "program")]
                        require((row["stage"], operation, command["id"]) in edit_proofs, "edited public command has no cache/daemon proof")
                    require(command["argv"] == expected, "edited request changed")
                    if row["stage"] == "invalid":
                        require(b"E0308" in stream(command["stdout"]) + stream(command["stderr"]), "edited type error was not rejected")
            require(set(row["runs"]) == (set() if row["stage"] == "invalid" else {"cargo", "entry"}), "edit runtime evidence count differs")
            for lane, command_id in row["runs"].items():
                command = commands[command_id]
                expected = (b"selection-cache-edited\n" if row["stage"] == "edited" else b"") + oracle
                require(command_id in runtime_commands and command["lane"] == lane and command["phase"] == "cache-edit-execute"
                        and runtime_builds[command_id] == row["commands"][lane + "-build"]
                        and command["exit"] == 0 and stream(command["stdout"]) == expected and stream(command["stderr"]) == b"", "edited runtime differs from Rust")
            edit_steps.append(row["stage"])
        if row["kind"] == "cache-edit-completed":
            require(edit_active and not edit_done and "entry" not in active, "edit daemon did not shut down cleanly")
            require(edit_sources == edit_steps == list(variants) and [(stage,op) for stage,op,_ in edit_proofs] ==
                    [(stage,op) for stage in variants for op in ("check", "build")], "cache-hit edit transition sequence is incomplete")
            edit_active, edit_done = False, True
        if row["kind"] == "selection-cache":
            command = commands[row["command"]]
            require(row["lane"] in ("baseline", "entry") and command["lane"] == row["lane"] and command["phase"] == "gate-selection-hit"
                    and command["argv"][0] == report["tool_paths"][row["lane"]], "selection-cache proof uses another command")
            require([json.loads(line) for line in stream(row["trace"]).splitlines()] == [{"selection_cache":"hit"}], "selection-cache hit missing")
            selection.append((row["lane"], row["operation"]))
        if row["kind"] == "entry-route":
            trace = [json.loads(x) for x in stream(row["trace"]).splitlines()]
            command = commands[row["command"]]
            route = "upstream-process" if "LUST_NO_DAEMON" in command["environment"] else "daemon"
            require(trace == row["records"] == [{"engine":"lust", "entry":route}], "native route differs")
        if row["kind"] == "timing" and row["phase"].startswith("no-op-") and row["lane"] != "cargo":
            require(row["lane"] in active, "warm timing had no owned daemon")
        if row["kind"] == "timing":
            require(group is not None and row["sample"] == group["sample"] and row["phase"].endswith("-" + group["operation"]), "timing escaped its group")
        if row["kind"] == "excluded-prime":
            require(group is not None and row["sample"] == group["sample"] and row["operation"] == group["operation"], "prime escaped its group")
            command = commands[row["command"]]
            require(command["phase"] == "excluded-prime" and command["lane"] == row["lane"] and command["seconds"] == row["seconds"], "prime timing differs")
    require(not active, "daemon still active")
    require(group is None, "measurement group did not finish")
    require(runtime_commands == {row["id"] for row in commands.values() if row["phase"].endswith("runtime") or row["phase"] == "cache-edit-execute"}, "runtime identity missing")
    require(edit_done and not edit_active, "cache-hit edit transition qualification is missing")
    require(retained == [(lane,op) for _ in range(2) for lane in ("pristine", "baseline", "entry") for op in ("check", "build")], "missing retained check/build proof")
    require(selection == [(lane,op) for _ in range(2) for lane in ("baseline", "entry") for op in ("check", "build")], "missing initial/final selection reuse proof")
    samples = [dict((k,v) for k,v in row.items() if k != "kind") for row in records if row["kind"] == "timing"]
    require(samples == report["samples"], "sample log differs")
    for sample in samples:
        require(len(sample["commands"]) == 1, "timing must cover one complete public command")
        command = commands[sample["commands"][0]]
        require(command["lane"] == sample["lane"] and command["phase"] == sample["phase"] and command["seconds"] == sample["seconds"], "timing differs")
        require(command["argv"][0] == report["tool_paths"][sample["lane"]], "wrong timed entry")
        require(all(key not in command["environment"] for key in ("LUST_STATS","LUSTR_TRACE_PATH","LUSTR_SELECTION_DIAGNOSTICS")), "timed command is instrumented")
        env = command["environment"]
        cold = sample["phase"].startswith("cold-")
        require((env.get("LUST_NO_DAEMON") == "1" and env.get("LUST_NO_CACHE") == "1") if cold else
                "LUST_NO_DAEMON" not in env and "LUST_NO_CACHE" not in env, "cold/warm controls differ")
        operation = sample["phase"].split("-", 2)[-1]
        state = Path(env["CARGO_TARGET_DIR"]).parent
        fixture = Path(command["cwd"])
        require(state.parts[-2:] == (f"{sample['sample']:03}-{operation}", sample["lane"]), "lane state is shared")
        require(env["LUST_CACHE_DIR"] == str(state / "native-cache") and env["CARGO_HOME"] == str(state / "cargo-home"), "cache directory differs")
        if sample["lane"] == "cargo":
            expected = [report["tool_paths"]["cargo"], operation, "--offline", "--locked", "--manifest-path", str(fixture / "Cargo.toml"),
                        "--bin", "slice", "--target", S.TARGET, "--message-format=json-render-diagnostics"]
            S.cargo_artifact(stream(command["stdout"]), operation, state / "target", not cold)
        else:
            expected = [report["tool_paths"][sample["lane"]], operation, str(fixture / "src/slice_main.rs"), "--manifest-path", str(fixture / "Cargo.toml"),
                        "--bin", "slice", "--offline", "--locked", *S.FLAGS]
            if operation == "build": expected += ["-o", str(state / "program")]
        require(command["argv"] == expected, "timed request changed")
    require([row for row in records if row["kind"] == "qualification"] == [
        {"kind":"qualification", "stage":stage, "completed":True} for stage in ("initial", "final")], "qualification is incomplete")
    require(report["orders"] == orders(report["pairs"]), "order changed")
    groups = [row for row in records if row["kind"] == "group-start"]
    require(groups == [{"kind":"group-start", "sample":i, "operation":op, "order":order}
                       for i,order in enumerate(report["orders"]) for op in ("check", "build")], "measured schedule changed")
    require([(row["sample"], row["phase"], row["lane"]) for row in samples] == [
        (i,phase + "-" + op,lane) for i,order in enumerate(report["orders"]) for op in ("check","build")
        for phase in ("cold","no-op") for lane in order], "actual timed order differs")
    actual = summaries(samples, report["pairs"])
    S.STATS.compare(actual, report["summaries"], "warm comparison statistics")
    S.STATS.compare(parity(actual), report["warm_target"], "warm target")
    S.STATS.compare(priming_summary(records,report["pairs"]), report["priming_summary"], "excluded priming costs")
    require((output / "README.md").read_text() == render(report), "table changed")
    require(S.inventory(output / "fixture") == report["fixture_files"], "fixture changed")
    return report




def main():
    if sys.argv[1:2] != ["verify"]:
        print("Retired whole-rustc acquisition is disabled. Use verify DIRECTORY for immutable historical evidence.", file=sys.stderr)
        return 2
    parser = argparse.ArgumentParser(description="Verify frozen historical evidence without compiler execution or current qualification.")
    parser.add_argument("command", choices=("verify",))
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    verify(args.output.resolve())
    print("Verified historical evidence; no compiler execution or current qualification.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
