#!/usr/bin/env python3
"""Measure the pinned public goport-util-small slice with four compiler entries.

This is a separate campaign. It neither changes nor extends schema-1/2 scalar
results. Only complete, qualified campaigns receive a speed table.
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
import signal
import stat
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
PIN = "240e08d91d4aed0299639f6f11fd522912b248f9"
TREE = "0ab1b09815802894e6078de0af59273cbdf46cb0"
RUST = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494"
TARGET = "x86_64-unknown-linux-gnu"
LANES = ("pristine", "integrated", "entry", "cargo")
PHASES = ("cold-check", "no-op-check", "cold-build", "no-op-build", "compile-plus-run", "runtime")
SEED = 0x534C494345
FLAGS = ("--target", TARGET, "-C", "panic=unwind", "-C", "overflow-checks=on", "-C", "debug-assertions=on")
LIMIT = 8 * 1024 * 1024
SCHEMA = "lustr-public-slice-v1"
PROFILE = ("[profile.dev]\nopt-level = 0\ndebug = 0\npanic = \"unwind\"\n"
           "overflow-checks = true\ndebug-assertions = true\nincremental = true\n")
REJECTION = 'fn main() { let _value: u32 = "not an integer"; }\n'


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, HERE / filename)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


STATS = module("slice_statistics", "report-mixed-performance.py")
HOST = module("slice_host", "capture-benchmark-host.py")
require = STATS.require


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    Path(path).write_text(json.dumps(value, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def inventory(root):
    result = {}
    for path in sorted(Path(root).rglob("*")):
        require(not path.is_symlink(), "source contains a symlink")
        if path.is_dir():
            continue
        require(path.is_file(), "source contains a special file")
        result[str(path.relative_to(root))] = digest(path)
    return result


def prepare_fixture(repository, pristine, output):
    relative = Path("bench/slices/goport-util-small")
    source = repository / "compiler/native" / relative
    require(inventory(source) == inventory(pristine / relative), "vendored slice differs from the pristine reference")
    driver = repository / "compiler/native/bench/slices/driver.rs"
    require(driver.read_bytes() == (pristine / "bench/slices/driver.rs").read_bytes(), "upstream bench driver differs")
    prune = STATS.json_load((source / "prune.json").read_bytes())
    require(prune["spec"]["root"] == "slice_main.rs" and not prune.get("dev_features"), "pinned slice contract changed")
    shutil.copytree(source / "src", output / "src")
    shutil.copyfile(driver, output / "src/bench_driver.rs")
    main = output / "src/slice_main.rs"
    text = main.read_text()
    require(text.count("fn main() {}") == 1, "missing unique upstream driver anchor")
    main.write_text(text.replace("fn main() {}", "mod bench_driver;\nfn main() { bench_driver::run(); }"))
    (output / "Cargo.toml").write_text(
        '[package]\nname = "slice"\nversion = "0.0.0"\nedition = "2024"\n'
        '[[bin]]\nname = "slice"\npath = "src/slice_main.rs"\n' + PROFILE + '\n[workspace]\n')
    return {"upstream_slice_files": inventory(source), "upstream_driver_sha256": digest(driver),
            "tsrust_revision": prune.get("tsrust_commit"), "features": [],
            "preparation": "Upstream slice bytes retained; bench driver replaces the one empty main; explicit standalone manifest added."}


def orders(count, seed=SEED):
    require(count >= 12 and count <= 96 and count % 4 == 0, "pairs must be a multiple of four in 12..96")
    rng = random.Random(seed)
    result = []
    while len(result) < count:
        labels, rotations = list(LANES), list(range(4))
        rng.shuffle(labels)
        rng.shuffle(rotations)
        result.extend([labels[n:] + labels[:n] for n in rotations])
    return result


def cargo_artifact(data, operation, target, fresh):
    messages = [STATS.json_load(line) for line in data.splitlines() if line]
    require(sum(m.get("reason") == "build-finished" and m.get("success") is True for m in messages) == 1,
            "Cargo did not report one successful completed build")
    artifacts = [m for m in messages if m.get("reason") == "compiler-artifact"
                 and m.get("target", {}).get("name") == "slice" and m.get("target", {}).get("kind") == ["bin"]]
    require(len(artifacts) == 1, "Cargo did not produce exactly the selected slice artifact")
    artifact = artifacts[0]
    require(artifact.get("fresh") is fresh, "Cargo Fresh state differs from the declared scenario")
    profile = artifact.get("profile", {})
    require(profile.get("opt_level") == "0" and profile.get("debuginfo") in (0, None)
            and profile.get("debug_assertions") is True and profile.get("overflow_checks") is True,
            "Cargo artifact profile differs")
    if operation == "build":
        expected = target / TARGET / "debug/slice"
        require(artifact.get("executable") == str(expected), "Cargo executable escaped the owned lane")
        return expected
    require(artifact.get("executable") is None, "Cargo check unexpectedly produced an executable")
    return None


def retained_check(data):
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
    require(number(stats, "fns") > 0, "empty retained fixture")
    require(all(number(stats, key) == 0 for key in
                ("rechecked", "invalidated", "c_bytes", "dirty_units", "units_compiled", "reemitted", "discovered"))
            and number(visits, "checked") == 0, "native no-op repeated body or generated-code work")
    route = (stats.get("mode"), visits.get("path"), visits.get("why"))
    # Visits::render uses '-' for the empty reason on answer reuse and
    # resolver body patches. Only the chunk path assigns its mode to why.
    require(route in (("reuse", "reuse", "-"), ("bodies", "patch", "-"), ("bodies", "chunks", "bodies")),
            "native no-op did not use a known retained route")
    return route


def controlled_environment(toolchain, state, socket, cold=False):
    result = {"PATH": str(toolchain / "bin") + ":/usr/local/bin:/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
              "CARGO": str(toolchain / "bin/cargo"), "RUSTC": str(toolchain / "bin/rustc-reference"), "RUSTDOC": str(toolchain / "bin/rustdoc"),
              "CARGO_HOME": str(state / "cargo-home"), "CARGO_TARGET_DIR": str(state / "target"),
              "CARGO_BUILD_BUILD_DIR": str(state / "intermediates"), "CARGO_NET_OFFLINE": "true", "CARGO_TERM_COLOR": "never",
              "CARGO_BUILD_JOBS": "2", "CARGO_INCREMENTAL": "1", "TMPDIR": str(state / "tmp"),
              "LUST_CACHE_DIR": str(state / "native-cache"), "LUST_DAEMON_SOCKET": str(socket), "LUST_DAEMON_IDLE": "120s"}
    if cold:
        result.update(LUST_NO_DAEMON="1", LUST_NO_CACHE="1")
    return result


class Recorder:
    def __init__(self, output, timeout):
        self.output, self.timeout, self.records = output, timeout, []
        (output / "commands").mkdir()
        self.raw = (output / "raw.jsonl").open("x")

    def event(self, value):
        self.raw.write(json.dumps(value, allow_nan=False) + "\n")
        self.raw.flush()
        self.records.append(value)
        return value

    def run(self, argv, env, cwd, lane, phase, expected=0):
        raise ValueError("Retired whole-rustc acquisition is disabled; historical record models accept synthetic test outcomes only")


class Daemon:
    def __init__(self, *args, **kwargs):
        raise ValueError("Retired whole-rustc acquisition is disabled; historical record models cannot start daemons")


class Campaign:
    """Historical record model for replay controls; compiler/process sinks are disabled."""
    def __init__(self, args):
        self.args = args
        self.output = args.output
        self.recorder = Recorder(self.output, args.timeout)
        self.fixture = self.output / "fixture"
        self.tools = {"pristine": args.pristine_lust, "integrated": args.built / "toolchain/bin/lust",
                      "entry": args.built / "toolchain/bin/lustr", "cargo": args.built / "toolchain/bin/cargo"}
        self.reference = args.built / "toolchain/bin/rustc-reference"
        self.expected_output = None
        self.samples = []
        self.built_artifacts = {}
        self.report = {"schema": SCHEMA, "status": "failed", "pairs": args.pairs, "summaries": None,
                       "reference_rustc": str(self.reference),
                       "pristine_source": str(args.pristine_source), "candidate_source": str(args.repository),
                       "scope": "Pinned public goport-util-small slice, upstream driver, no source edits between samples.",
                       "order_method": "seeded-python-cyclic-rotations-v1", "seed": SEED,
                       "profiles": "Application dev opt0/debug0/unwind/overflow+debug assertions; Cargo incremental enabled; native TinyCC dev.",
                       "cold_scope": "New owned lane directories; native daemon/disk cache disabled; OS caches not flushed.",
                       "warm_scope": "Retained native daemons versus Cargo Fresh; unchanged source; no body/signature-edit claim.",
                       "timing_scope": "Command wall time with 100us wait polling; compile-plus-run sums its two child command spans, excluding bookkeeping between them.",
                       "limits": "One public pruned/adapted slice; not unchanged full ts-rust, full Rust, fastest Rust setup, resources or release admission."}

    def environment(self, state, socket, cold=False):
        for folder in (state, state / "tmp", state / "cargo-home"):
            folder.mkdir(parents=True, exist_ok=True)
        return controlled_environment(self.args.built / "toolchain", state, socket, cold)

    def command(self, lane, operation, state):
        if lane == "cargo":
            return [self.tools[lane], operation, "--offline", "--locked", "--manifest-path", self.fixture / "Cargo.toml",
                    "--bin", "slice", "--target", TARGET, "--message-format=json-render-diagnostics"]
        argv = [self.tools[lane], operation, self.fixture / "src/slice_main.rs", "--manifest-path", self.fixture / "Cargo.toml",
                "--bin", "slice", "--offline", "--locked", *FLAGS]
        return argv + (["-o", state / "program"] if operation == "build" else [])

    def compile(self, lane, operation, state, env, phase, fresh=False, trace=False):
        local = dict(env)
        trace_path = state / f"entry-{len(self.recorder.records)}.trace"
        if trace and lane == "entry":
            local["LUSTR_TRACE_PATH"] = str(trace_path)
        result, stdout, stderr = self.recorder.run(self.command(lane, operation, state), local, self.fixture, lane, phase)
        binary = state / "program" if operation == "build" else None
        if lane == "cargo":
            binary = cargo_artifact(stdout, operation, state / "target", fresh)
        if trace and lane == "entry":
            rows = [STATS.json_load(line) for line in trace_path.read_bytes().splitlines()]
            expected = "upstream-process" if "LUST_NO_DAEMON" in env else "daemon"
            require(rows == [{"engine": "lust", "entry": expected}], "packaged entry did not use the required native route")
            archived = self.output / "commands" / f"{result['id']:05}.entry.trace"
            shutil.copyfile(trace_path, archived)
            self.recorder.event({"kind": "entry-route", "command": result["id"], "phase": phase, "records": rows,
                                 "trace": {"file": str(archived.relative_to(self.output)), "sha256": digest(archived), "bytes": archived.stat().st_size}})
        if binary is not None:
            require(binary.is_file() and not binary.is_symlink(), "successful build did not produce an ordinary executable")
            self.built_artifacts[binary] = {"build_command": result["id"], "binary_sha256": digest(binary)}
            self.recorder.event({"kind": "build-identity", "binary": str(binary), **self.built_artifacts[binary]})
        return result, binary

    def runtime(self, lane, binary, env, phase):
        identity = digest(binary)
        built = self.built_artifacts.get(binary)
        require(built is not None and built["binary_sha256"] == identity, "runtime executable differs from its successful build")
        result, stdout, stderr = self.recorder.run([binary], env, self.fixture, lane, phase)
        require(not stderr, "application emitted unexpected stderr")
        if self.expected_output is None:
            require(lane == "cargo" and stdout, "Rust runtime oracle must run first and print output")
            self.expected_output = stdout
            (self.output / "runtime-oracle.stdout").write_bytes(stdout)
            self.report["runtime_oracle_sha256"] = digest(self.output / "runtime-oracle.stdout")
            new_oracle = True
        else:
            new_oracle = False
        require(stdout == self.expected_output, "application output differs from matched Rust")
        self.recorder.event({"kind": "runtime-identity", "command": result["id"], "binary": str(binary), **built})
        if new_oracle:
            self.recorder.event({"kind": "runtime-oracle", "command": result["id"], "sha256": self.report["runtime_oracle_sha256"]})
        return result

    def qualification(self, sockets, stage):
        self.recorder.event({"kind": "qualification-start", "stage": stage})
        for lane in ("cargo", "pristine", "integrated", "entry"):
            state = self.output / "work" / stage / "gates" / lane
            env = self.environment(state, sockets / (lane + ".sock"), True)
            _, binary = self.compile(lane, "build", state, env, "gate-build", trace=True)
            self.runtime(lane, binary, env, "gate-runtime")
        reject = self.output / "rejected.rs"
        require(digest(reject) == self.report["rejection_input"]["sha256"], "rejection input changed")
        for lane in LANES:
            state = self.output / "work" / stage / "rejection" / lane
            env = self.environment(state, sockets / (lane + ".sock"), True)
            if lane == "cargo":
                argv = [self.reference, reject, "--edition=2024", "--error-format=json", "--emit=metadata", "-o", state / "reject.rmeta", *FLAGS]
            else:
                argv = [self.tools[lane], "check", reject, "--json", *FLAGS]
            _, stdout, stderr = self.recorder.run(argv, env, self.output, lane, "gate-reject", 1)
            codes = []
            for line in (stdout + b"\n" + stderr).splitlines():
                try:
                    value = STATS.json_load(line)
                except (ValueError, UnicodeError):
                    continue
                if isinstance(value, dict):
                    code = value.get("code")
                    codes.append(code.get("code") if isinstance(code, dict) else code)
            require("L0110" in codes or "E0308" in codes or b"error[E0308]" in stderr,
                    "rejection was not the required type mismatch")
        for lane in LANES[:3]:
            state = self.output / "work" / stage / "warm-gates" / lane
            # stats_on() reads the daemon process environment. Enabling it on
            # a client request alone cannot instrument the retained compiler.
            # Keep the prime/probe setting equal because check_fingerprint()
            # includes the client environment. Measured daemons stay uninstrumented.
            env = dict(self.environment(state, sockets / (lane + ".sock")), LUST_STATS="1")
            daemon_tool = self.tools["integrated"] if lane == "entry" else self.tools[lane]
            with Daemon(self.recorder, daemon_tool, env, self.fixture, lane, f"qualification-{stage}-{lane}") as daemon:
                self.compile(lane, "check", state, env, "gate-warm-prime", trace=True)
                argv = self.command(lane, "check", state) + ["--timings"]
                command, _, stderr = self.recorder.run(argv, env, self.fixture, lane, "gate-retained")
                route = retained_check(stderr)
                daemon.alive()
                self.recorder.event({"kind": "retained-route", "command": command["id"], "lane": lane, "route": list(route)})
        self.recorder.event({"kind": "qualification", "stage": stage, "completed": True})

    def measure_group(self, sample, order, operation, sockets):
        self.recorder.event({"kind": "group-start", "sample": sample, "operation": operation, "order": order})
        with ExitStack() as stack:
            daemons, states, environments = {}, {}, {}
            for lane in order:
                state = self.output / "work" / f"{sample:03}-{operation}" / lane
                env = self.environment(state, sockets / (lane + ".sock"), True)
                states[lane] = state
                result, binary = self.compile(lane, operation, state, env, "cold-" + operation)
                if binary is not None:
                    self.runtime(lane, binary, env, "cold-build-runtime")
                self.timing(sample, lane, "cold-" + operation, [result])
            for lane in order:
                state = states[lane]
                env = self.environment(state, sockets / (lane + ".sock"))
                environments[lane] = env
                if lane != "cargo":
                    daemon_tool = self.tools["integrated"] if lane == "entry" else self.tools[lane]
                    daemons[lane] = stack.enter_context(Daemon(self.recorder, daemon_tool, env, self.fixture, lane, f"sample-{sample}-{operation}-{lane}"))
                for _ in range(2):
                    self.compile(lane, operation, state, env, "excluded-prime", fresh=lane == "cargo")
            for lane in order:
                if lane in daemons:
                    daemons[lane].alive()
                result, binary = self.compile(lane, operation, states[lane], environments[lane], "no-op-" + operation, fresh=lane == "cargo")
                if binary is not None:
                    self.runtime(lane, binary, environments[lane], "no-op-build-runtime")
                self.timing(sample, lane, "no-op-" + operation, [result])
                if lane in daemons:
                    daemons[lane].alive()
            if operation == "build":
                for lane in order:
                    if lane in daemons:
                        daemons[lane].alive()
                    build, binary = self.compile(lane, "build", states[lane], environments[lane], "compile-plus-run-build", fresh=lane == "cargo")
                    run = self.runtime(lane, binary, environments[lane], "compile-plus-run-runtime")
                    self.timing(sample, lane, "compile-plus-run", [build, run])
                    if lane in daemons:
                        daemons[lane].alive()
                for lane in order:
                    if lane in daemons:
                        daemons[lane].alive()
                    binary = states[lane] / (f"target/{TARGET}/debug/slice" if lane == "cargo" else "program")
                    run = self.runtime(lane, binary, environments[lane], "runtime")
                    self.timing(sample, lane, "runtime", [run])
                    if lane in daemons:
                        daemons[lane].alive()
        self.recorder.event({"kind": "group-end", "sample": sample, "operation": operation})

    def timing(self, sample, lane, phase, commands):
        value = {"sample": sample, "lane": lane, "phase": phase, "seconds": sum(c["seconds"] for c in commands), "commands": [c["id"] for c in commands]}
        self.samples.append(value)
        self.recorder.event({"kind": "timing", **value})



def summaries(samples, pairs):
    require(len(samples) == pairs * len(LANES) * len(PHASES), "missing or extra timing samples")
    by_key = {}
    for sample in samples:
        key = (sample["sample"], sample["lane"], sample["phase"])
        require(key not in by_key and 0 <= key[0] < pairs and key[1] in LANES and key[2] in PHASES
                and STATS.positive(sample["seconds"]), "invalid or duplicate sample")
        by_key[key] = sample["seconds"]
    result = []
    for phase in PHASES:
        for baseline, candidate in (("pristine", "integrated"), ("pristine", "entry"), ("cargo", "entry")):
            values = [(by_key[i, baseline, phase], by_key[i, candidate, phase]) for i in range(pairs)]
            result.append({"phase": phase, "baseline": baseline, "candidate": candidate, "analysis": STATS.summarize(values, 10000, SEED)})
    return result


def render(report):
    lines = ["# Public upstream slice performance", "", f"Status: **{report['status']}**.", "", report["scope"], "", report["limits"], ""]
    if report["status"] != "completed":
        return "\n".join(lines + ["No completed performance result. Original command streams and the failure are retained.", ""])
    lines += ["Times are medians and nearest-rank p95 in milliseconds. Ratio is paired baseline/candidate geometric mean with a conditional bootstrap 95% interval.",
              "Below one favors the reference. These exploratory intervals do not establish equivalence. Twelve-pair p95 is the maximum.", "",
              "| Scenario | Comparison | Reference median | Candidate median | Reference p95 | Candidate p95 | Ratio [95% interval] |",
              "| --- | --- | ---: | ---: | ---: | ---: | --- |"]
    for item in report["summaries"]:
        a = item["analysis"]
        values = " | ".join(f"{1000 * a[key]:.3f}" for key in ("baseline_median", "candidate_median", "baseline_p95", "candidate_p95"))
        low, high = a["bootstrap_95"]
        lines.append(f"| {item['phase']} | {item['baseline']} → {item['candidate']} | {values} | {a['geometric_ratio']:.3f} [{low:.3f}, {high:.3f}] |")
    return "\n".join(lines + ["", report["profiles"], "", report["cold_scope"], "", report["warm_scope"], "", report["timing_scope"], ""])


def verify_schedule(records, report, stream):
    """Consume the exact executed sequence, including untimed correctness work."""
    position, derived, active = 0, [], {}

    def take(kind, **fields):
        nonlocal position
        require(position < len(records), f"missing {kind} record")
        row = records[position]
        position += 1
        require(row.get("kind") == kind and all(row.get(key) == value for key, value in fields.items()),
                f"unexpected record {position - 1}; expected {kind} {fields}")
        return row

    for lane, phase in (("source", "pristine-identity"), ("source", "pristine-identity"),
                        ("source", "pristine-clean"), ("source", "candidate-identity"),
                        ("source", "candidate-clean"), ("cargo", "rust-identity")):
        take("command", lane=lane, phase=phase)
    lock = take("command", lane="cargo", phase="lockfile")
    fixture = Path(lock["cwd"])
    root = fixture.parent
    take("identities")
    oracle = stream({"file": "runtime-oracle.stdout", "sha256": report["runtime_oracle_sha256"]})
    tools = report["tool_paths"]

    def compile(lane, phase, operation, state, cold=False, traced=False):
        row = take("command", lane=lane, phase=phase, expected_exit=0, exit=0, cwd=str(fixture))
        env = row["environment"]
        require(env.get("CARGO_TARGET_DIR") == str(state / "target") and env.get("TMPDIR") == str(state / "tmp")
                and env.get("CARGO_HOME") == str(state / "cargo-home") and env.get("LUST_CACHE_DIR") == str(state / "native-cache")
                and env.get("CARGO_INCREMENTAL") == "1" and env.get("RUSTC") == report["reference_rustc"],
                "compiler state or Rust reference differs")
        require((env.get("LUST_NO_DAEMON") == "1" and env.get("LUST_NO_CACHE") == "1") if cold else
                "LUST_NO_DAEMON" not in env and "LUST_NO_CACHE" not in env, "cold/warm native state differs")
        instrumented = phase in ("gate-warm-prime", "gate-retained")
        require(env.get("LUST_STATS") == "1" if instrumented else "LUST_STATS" not in env,
                "qualification diagnostics are missing or leaked into other compiler commands")
        if lane == "cargo":
            expected = [tools[lane], operation, "--offline", "--locked", "--manifest-path", str(fixture / "Cargo.toml"),
                        "--bin", "slice", "--target", TARGET, "--message-format=json-render-diagnostics"]
            binary = str(state / "target" / TARGET / "debug/slice")
        else:
            expected = [tools[lane], operation, str(fixture / "src/slice_main.rs"), "--manifest-path", str(fixture / "Cargo.toml"),
                        "--bin", "slice", "--offline", "--locked", *FLAGS]
            binary = str(state / "program")
            if operation == "build":
                expected += ["-o", binary]
            if phase == "gate-retained":
                expected += ["--timings"]
            if not cold:
                require(lane in active and env.get("LUST_DAEMON_SOCKET") == active[lane]["socket"],
                        "native command is not bound to its active daemon")
        require(row["argv"] == expected, "compiler executable, source, operation or flags differ")
        require("LUSTR_TRACE_PATH" not in env or traced and lane == "entry", "tracing leaked into measured commands")
        if traced and lane == "entry":
            expected_route = [{"engine": "lust", "entry": "upstream-process" if cold else "daemon"}]
            route = take("entry-route", command=row["id"], phase=phase, records=expected_route)
            require([STATS.json_load(line) for line in stream(route["trace"]).splitlines()] == expected_route,
                    "original entry trace differs from declared native route")
        built = None
        if operation == "build":
            built = take("build-identity", build_command=row["id"], binary=binary)
            require(len(built["binary_sha256"]) == 64 and all(c in "0123456789abcdef" for c in built["binary_sha256"]),
                    "invalid generated executable hash")
        return row, built

    def runtime(lane, phase, built, first=False):
        row = take("command", lane=lane, phase=phase, expected_exit=0, exit=0, argv=[built["binary"]])
        take("runtime-identity", command=row["id"], binary=built["binary"], build_command=built["build_command"],
             binary_sha256=built["binary_sha256"])
        require(stream(row["stdout"]) == oracle and not stream(row["stderr"]), "runtime differs from oracle")
        if first:
            take("runtime-oracle", command=row["id"], sha256=report["runtime_oracle_sha256"])
        return row

    def start(lane, scope, state, instrumented=False):
        require(lane not in active, "daemon lane was started twice")
        row = take("daemon-start", lane=lane, scope=scope, cwd=str(fixture),
                   executable=tools["integrated"] if lane == "entry" else tools[lane])
        require(type(row.get("pid")) is int and row["pid"] > 0 and row.get("socket") not in [d["socket"] for d in active.values()]
                and row["environment"].get("LUST_DAEMON_SOCKET") == row["socket"]
                and row["environment"].get("LUST_CACHE_DIR") == str(state / "native-cache"), "daemon ownership differs")
        require(row["environment"].get("LUST_STATS") == "1" if instrumented else "LUST_STATS" not in row["environment"],
                "qualification daemon diagnostics are missing or leaked into measured daemons")
        active[lane] = row

    def stop(lane):
        started = active.pop(lane)
        row = take("daemon-stop", lane=lane, scope=started["scope"], pid=started["pid"], exit=0, error=None)
        require(len(row["streams"]) == 2, "daemon streams missing")
        for output in row["streams"]:
            stream(output)

    def qualification(stage):
        take("qualification-start", stage=stage)
        for lane in ("cargo", "pristine", "integrated", "entry"):
            state = root / "work" / stage / "gates" / lane
            _, built = compile(lane, "gate-build", "build", state, cold=True, traced=True)
            runtime(lane, "gate-runtime", built, first=stage == "initial" and lane == "cargo")
        for lane in LANES:
            row = take("command", lane=lane, phase="gate-reject", expected_exit=1, exit=1, cwd=str(root))
            state = root / "work" / stage / "rejection" / lane
            expected = ([report["reference_rustc"], str(root / "rejected.rs"), "--edition=2024", "--error-format=json", "--emit=metadata",
                         "-o", str(state / "reject.rmeta"), *FLAGS] if lane == "cargo" else
                        [tools[lane], "check", str(root / "rejected.rs"), "--json", *FLAGS])
            require(row["argv"] == expected, "rejection executable, source or flags differ")
            out, err = stream(row["stdout"]), stream(row["stderr"])
            codes = []
            for line in (out + b"\n" + err).splitlines():
                try:
                    value = STATS.json_load(line)
                except (ValueError, UnicodeError):
                    continue
                if isinstance(value, dict):
                    code = value.get("code")
                    codes.append(code.get("code") if isinstance(code, dict) else code)
            require("L0110" in codes or "E0308" in codes or b"error[E0308]" in err, "missing expected rejection diagnostic")
        for lane in LANES[:3]:
            state = root / "work" / stage / "warm-gates" / lane
            start(lane, f"qualification-{stage}-{lane}", state, instrumented=True)
            compile(lane, "gate-warm-prime", "check", state, traced=True)
            row, _ = compile(lane, "gate-retained", "check", state)
            route = list(retained_check(stream(row["stderr"])))
            take("retained-route", command=row["id"], lane=lane, route=route)
            stop(lane)
        take("qualification", stage=stage, completed=True)

    def timing(sample, lane, phase, commands):
        value = {"sample": sample, "lane": lane, "phase": phase, "seconds": sum(c["seconds"] for c in commands), "commands": [c["id"] for c in commands]}
        row = take("timing", sample=sample, lane=lane, phase=phase, commands=value["commands"])
        STATS.compare(row["seconds"], value["seconds"], "raw measured duration")
        derived.append(value)

    qualification("initial")
    for sample, order in enumerate(orders(report["pairs"])):
        take("sample-start", sample=sample, order=order)
        for operation in ("check", "build"):
            take("group-start", sample=sample, operation=operation, order=order)
            states = {lane: root / "work" / f"{sample:03}-{operation}" / lane for lane in order}
            latest = {}
            for lane in order:
                row, built = compile(lane, "cold-" + operation, operation, states[lane], cold=True)
                if operation == "build":
                    runtime(lane, "cold-build-runtime", built)
                timing(sample, lane, "cold-" + operation, [row])
            for lane in order:
                if lane != "cargo":
                    start(lane, f"sample-{sample}-{operation}-{lane}", states[lane])
                for _ in range(2):
                    compile(lane, "excluded-prime", operation, states[lane])
            for lane in order:
                row, built = compile(lane, "no-op-" + operation, operation, states[lane])
                if operation == "build":
                    runtime(lane, "no-op-build-runtime", built)
                timing(sample, lane, "no-op-" + operation, [row])
            if operation == "build":
                for lane in order:
                    row, built = compile(lane, "compile-plus-run-build", "build", states[lane])
                    executed = runtime(lane, "compile-plus-run-runtime", built)
                    timing(sample, lane, "compile-plus-run", [row, executed])
                    latest[lane] = built
                for lane in order:
                    row = runtime(lane, "runtime", latest[lane])
                    timing(sample, lane, "runtime", [row])
            for lane in reversed(order):
                if lane != "cargo":
                    stop(lane)
            take("group-end", sample=sample, operation=operation)
        take("sample-end", sample=sample)
    qualification("final")
    require(position == len(records) and not active, "unexpected trailing work or live daemon")
    STATS.compare(report["samples"], derived, "samples derived from executed order")


def replay(output):
    """Verify retained records and recompute tables without executing log commands."""
    output = output.resolve(strict=True)
    report = STATS.json_load((output / "report.json").read_bytes())
    require(report.get("schema") == SCHEMA and report.get("status") == "completed", "only completed slice campaigns have speed results")
    require(digest(output / "raw.jsonl") == report.get("raw_sha256"), "original command-record digest differs")
    records = [STATS.json_load(line) for line in (output / "raw.jsonl").read_bytes().splitlines()]
    identities = [r for r in records if r.get("kind") == "identities"]
    require(len(identities) == 1, "missing campaign identities")
    for key in ("compiler_and_runner_sha256", "fixture_files", "tool_paths", "rejection_input", "reference_rustc", "build_receipts"):
        require(identities[0].get(key) == report.get(key), "report identity differs from original records")
    require(inventory(output / "fixture") == report["fixture_files"], "retained fixture changed")
    qualifications = [r for r in records if r.get("kind") == "qualification"]
    require(qualifications == [{"kind": "qualification", "stage": stage, "completed": True} for stage in ("initial", "final")],
            "initial/final qualification is missing")
    require(report.get("orders") == orders(report["pairs"]), "declared lane order differs")
    def stream(record):
        relative = Path(record["file"])
        require(not relative.is_absolute() and ".." not in relative.parts, "recorded stream escapes archive")
        path = output / relative
        require(not path.is_symlink() and path.resolve().is_relative_to(output) and path.stat().st_size <= LIMIT,
                "recorded stream is linked, escaped or oversized")
        require(digest(path) == record["sha256"], "original output digest differs")
        if "bytes" in record:
            require(path.stat().st_size == record["bytes"], "original output length differs")
        return path.read_bytes()
    require(report["rejection_input"]["file"] == "rejected.rs" and stream(report["rejection_input"]) == REJECTION.encode(),
            "retained type-mismatch source differs")
    require(set(report["build_receipts"]) == {"BUILD-PROFILE.txt", "COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt", "pristine-build.log"},
            "build receipts are missing")
    for name, binding in report["build_receipts"].items():
        require(report["compiler_and_runner_sha256"].get(binding["source_path"]) == binding["sha256"], "receipt lost its original source binding")
        stream({"file": "build-receipts/" + name, "sha256": binding["sha256"]})
    commands = {}
    for row in records:
        if row.get("kind") == "command":
            require(row["id"] not in commands and row.get("exit") == row.get("expected_exit") and row["exit"] in (0, 1)
                    and not row.get("signal") and not row.get("timed_out") and not row.get("output_limit") and not row.get("launch_error"),
                    "failed, duplicated or incomplete command in completed campaign")
            require(type(row["id"]) is int and row["id"] >= 0 and STATS.positive(row.get("seconds")), "invalid command identity or duration")
            require(row["exit"] == 0 or row["phase"] == "gate-reject", "unexpected rejected command")
            commands[row["id"]] = row
            stream(row["stdout"])
            stream(row["stderr"])
            if row["lane"] in LANES[:3] and len(row["argv"]) > 1:
                require(row["argv"][0] == report["tool_paths"][row["lane"]], "native lane executable differs")
            if row["lane"] == "cargo" and len(row["argv"]) > 1 and row["argv"][1] in ("check", "build"):
                require(row["argv"][0] == report["tool_paths"]["cargo"], "Cargo lane executable differs")
                cargo_artifact(stream(row["stdout"]), row["argv"][1], Path(row["environment"]["CARGO_TARGET_DIR"]),
                               row["phase"] not in ("gate-build", "cold-check", "cold-build"))
        elif row.get("kind") == "daemon-stop":
            require(row.get("exit") == 0 and row.get("error") is None, "failed daemon shutdown")
            for value in row["streams"]:
                stream(value)
    verify_schedule(records, report, stream)
    STATS.compare(report["summaries"], summaries(report["samples"], report["pairs"]), "slice statistics")
    require((output / "README.md").read_text() == render(report), "published table differs from raw samples")
    return report


def main():
    if sys.argv[1:2] != ["--replay"]:
        print("Retired whole-rustc acquisition is disabled. Use --replay DIRECTORY for immutable historical evidence.", file=sys.stderr)
        return 2
    require(len(sys.argv) == 3, "replay requires one retained campaign directory")
    result = replay(Path(sys.argv[2]))
    print(f"Verified {len(result['samples'])} historical timings and {len(result['summaries'])} comparisons; no compiler execution or current qualification.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
