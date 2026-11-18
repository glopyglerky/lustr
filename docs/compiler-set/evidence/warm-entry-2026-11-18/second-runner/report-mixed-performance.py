#!/usr/bin/env python3
"""Verify and render lustr-mixed-bench evidence; never execute its argv.

This is an evidence/statistics replay, not compiler qualification or release
admission. The statistics mirror crates/lustr-bench/src/stats.rs (paired-log-v1).
Schema 1 retains the original nine-mode/five-lane campaign. Schema 2 adds the
matched standalone rustc lane on exactly the same native corpus. Both have
fixed coverage; unfamiliar or missing samples are never silently dropped.
"""
from __future__ import annotations

import argparse
import collections
import csv
import io
import hashlib
import json
import math
from pathlib import Path, PurePosixPath
import re
import sys
import zipfile

PHASES = ("cold", "no-op", "body-edit", "signature-edit", "check", "check-no-op",
          "check-body-edit", "rejection", "compile-plus-test")
BUILD_PHASES = ("cold", "no-op", "body-edit", "signature-edit")
CARGO_SOURCES = ("Cargo.lock", "Cargo.toml", "app/Cargo.toml", "app/src/main.rs",
                 "fixture-macro/Cargo.toml", "fixture-macro/src/lib.rs", "portable/Cargo.toml", "portable/src/lib.rs")
LANES = {"native": ("pristine", "integrated", "entry"), "cargo": ("reference", "mixed")}
LANES_V2 = {"native": ("pristine", "integrated", "entry", "rustc"), "cargo": ("reference", "mixed")}
COMPARISONS = (("native", "pristine", "integrated"), ("native", "pristine", "entry"),
               ("cargo", "reference", "mixed"))
COMPARISONS_V2 = COMPARISONS + (("native", "rustc", "entry"),)
RUSTC_FLAGS = ["--edition=2024", "--target=x86_64-unknown-linux-gnu", "-C", "opt-level=0",
               "-C", "debuginfo=0", "-C", "panic=unwind", "-C", "overflow-checks=on",
               "-C", "debug-assertions=on"]
NATIVE_FLAGS = ["--target", "x86_64-unknown-linux-gnu", "-C", "panic=unwind", "-C", "overflow-checks=on",
                "-C", "debug-assertions=on"]
TEST_RECEIPT = "lustr benchmark rustc-test: "
REFERENCE_PROBES = {"reference-probe-check": 0, "reference-probe-build": 0,
                    "reference-probe-build-runtime": 0, "reference-probe-test": 0,
                    "reference-probe-invalid": 1, "reference-probe-failed-test": 101,
                    "reference-probe-recovery": 0, "reference-probe-recovery-runtime": 0}
METHOD = "paired-log-v1/splitmix64/rejection-index/percentile"
DIRECTION = "baseline time / candidate time; greater than 1 is faster"
UNTIMED = {"excluded-prime", "cold-runtime", "no-op-runtime", "body-edit-runtime",
           "signature-edit-runtime", "generate-lockfile", "prime", "warm", "reuse",
           "body-probe", "probe-cold", "probe-body", "body-runtime", "probe-check",
           "probe-check-body", "probe-rejection"}
MASK = (1 << 64) - 1
SHA = re.compile(r"[0-9a-f]{64}\Z")


def campaign_layout(schema):
    require(type(schema) is int and schema in (1, 2), "unsupported schema")
    return (LANES, COMPARISONS) if schema == 1 else (LANES_V2, COMPARISONS_V2)


def native_reference_contract(compiler, test_driver):
    return dict(compiler=compiler, test_driver=test_driver, flags=RUSTC_FLAGS,
                incremental="disabled; standalone rustc default", build="--emit=link", check="--emit=metadata",
                test="--test then execute --test-threads=1 --color never; one outer wall-time measurement",
                source="identical native_source bytes across all four native lanes",
                native_profile="edition 2024, TinyCC dev/no debug; DEV_UNWIND with overflow checks and debug assertions; test uses the native test harness defaults")


def argument(value):
    require(isinstance(value, dict) and set(value) == {"Unix"} and isinstance(value["Unix"], list)
            and all(integer(x, 0, 255) for x in value["Unix"]), "invalid command argument")
    text = bytes(value["Unix"]).decode("utf-8")
    require("\x00" not in text, "NUL in command argument")
    return text


def test_output(stdout, stderr, lane, passed, argv, contract):
    if lane == "rustc":
        expected = ("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;" if passed else
                    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;")
        summaries = [line for line in stdout.decode().splitlines() if line.startswith("test result:")]
        require(len(summaries) == 1 and summaries[0].startswith(expected), "rustc test was not executed exactly once")
        require(f"test scalar_contract ... {'ok' if passed else 'FAILED'}" in stdout.decode().splitlines(),
                "wrong executed rustc test")
        receipts = [json_load(line[len(TEST_RECEIPT):]) for line in stderr.decode().splitlines() if line.startswith(TEST_RECEIPT)]
        require(len(argv) == 5 and argv[:3] == [contract["test_driver"], "--native-rustc-test", contract["compiler"]],
                "wrong rustc test driver")
        expected_commands = [[contract["compiler"], argv[3], *RUSTC_FLAGS, "--test", "-o", argv[4]],
                             [argv[4], "--test-threads=1", "--color", "never"]]
        require(len(receipts) == 2, "missing/extra rustc test child receipt")
        for record, phase, command, code in zip(receipts, ("compile", "execute"), expected_commands, (0, 0 if passed else 101)):
            require(set(record) == {"phase", "argv", "exit", "signal"} and record["phase"] == phase
                    and type(record["exit"]) is int and record["exit"] == code and record["signal"] is None
                    and [argument(arg) for arg in record["argv"]] == command, "rustc test child contract differs")
    else:
        records = []
        for line in stdout.splitlines():
            try:
                value = json_load(line)
            except ValueError:
                continue
            if isinstance(value, dict) and value.get("kind") == "summary":
                records.append(value)
        require(len(records) == 1 and records[0].get("build") == "ok", "native test build/summary differs")
        expected = dict(total=1, passed=int(passed), failed=int(not passed), timed_out=0, ignored=0, filtered_out=0)
        require(all(type(records[0].get(k)) is int and records[0][k] == v for k, v in expected.items()),
                "native test was not executed exactly once")


def native_command(row, streams, contract, lane_executables):
    """Schema-2 command semantics, including the real two-child test operation."""
    lane, phase = row["lane"], row["phase"]
    argv = [argument(arg) for arg in row["argv"]]
    source = source_map(row["source_sha256"])
    require(source == source_map(row["source_after_sha256"]) and len(source) == 1, "native source changed")
    source = next(iter(source))
    if phase.endswith("-runtime"):
        require(len(argv) == 1 and streams["stdout"] == (b"57\n" if phase == "body-edit-runtime" else b"59\n")
                and streams["stderr"] == b"" and row["exit"] == row["expected_exit"] == 0,
                "native runtime result differs")
        return
    checks = {"check", "check-no-op", "check-body-edit", "rejection", "reference-probe-check", "reference-probe-invalid"}
    tests = {"compile-plus-test", "reference-probe-test", "reference-probe-failed-test"}
    builds = {"cold", "no-op", "body-edit", "signature-edit", "excluded-prime", "reference-probe-build", "reference-probe-recovery"}
    executable = contract["test_driver"] if lane == "rustc" and phase in tests else lane_executables[lane]
    require(argv and argv[0] == executable, "native command executable differs from declared lane")
    if phase not in checks | tests | builds:
        return  # Existing untimed retained-route probes have their own stats flags.
    operation = "check" if phase in checks else "test" if phase in tests else "build"
    if lane == "rustc":
        if operation == "test":
            require(len(argv) == 5 and argv[3] == source, "rustc test uses different source")
        else:
            require(len(argv) == len(RUSTC_FLAGS) + 5 and argv[:2] == [contract["compiler"], source]
                    and argv[2:-3] == RUSTC_FLAGS and argv[-3:-1] == ["--emit=metadata" if operation == "check" else "--emit=link", "-o"],
                    "rustc native reference flags/source differ")
    else:
        require(argv[1:3] == [operation, source], "native command uses different source/operation")
        if operation == "test":
            require(argv[3:] == ["--json"], "native test flags differ")
        elif operation == "check":
            require(argv[3:] == NATIVE_FLAGS, "native check flags differ")
        else:
            require(len(argv) == len(NATIVE_FLAGS) + 5 and argv[3] == "-o" and argv[5:] == NATIVE_FLAGS,
                    "native build flags differ")
    if operation == "test":
        test_output(streams["stdout"], streams["stderr"], lane, phase != "reference-probe-failed-test", argv, contract)
    if operation == "build":
        require(row["exit"] == 0, "native build did not succeed")
        return argv[-1] if lane == "rustc" else argv[4]


def cargo_executables(report):
    """Recover the fixed recorder's Cargo entries from its original identities."""
    paths = report["compiler_sha256"]
    references = [path for path in paths if PurePosixPath(path).name == "cargo"]
    require(len(references) == 1 and PurePosixPath(references[0]).is_absolute(),
            "missing/ambiguous Cargo executable identity")
    reference = references[0]
    mixed = str(PurePosixPath(reference).with_name("lustr"))
    require(mixed in paths, "missing mixed Cargo executable identity")
    if report["schema"] == 2:
        require(report["native_lane_executables"]["entry"] == mixed,
                "mixed Cargo entry differs from the identified product entry")
    return {"reference": reference, "mixed": mixed}


def cargo_command(row, streams, executables):
    """Verify complete measured Cargo work in the original schema-1/2 corpus."""
    lane, phase, sample = row["lane"], row["phase"], row["sample"]
    source = source_map(row["source_sha256"])
    applications = [PurePosixPath(path) for path in source if path.endswith("/app/src/main.rs")]
    require(len(applications) == 1 and applications[0].is_absolute(), "measured Cargo source is ambiguous")
    root = applications[0].parents[2]
    require(root.name == "source" and root.parent.name == f"cargo-{sample:03}"
            and row.get("cwd") == str(root)
            and set(source) == {str(root / path) for path in CARGO_SOURCES},
            "measured Cargo working directory/source contract differs")
    state = root.parent / f"lane{LANES['cargo'].index(lane)}"
    operation = "check" if phase.startswith("check") else "test" if phase == "compile-plus-test" else "build"
    expected = [executables[lane], operation, "--locked", "--offline", "--jobs", "2", "-p", "mixed-app",
                "--bin", "mixed-app", "--target-dir", str(state / "target"), "--config",
                "build.build-dir=" + json.dumps(str(state / "intermediates"), ensure_ascii=False)]
    if operation == "test":
        expected += ["--", "--test-threads=1"]
    require([argument(value) for value in row["argv"]] == expected,
            "measured Cargo executable/operation/flags differ from its lane contract")
    if operation == "test":
        lines = streams["stdout"].decode("utf-8").splitlines()
        summaries = [line for line in lines if line.startswith("test result:")]
        tests = [line for line in lines if line.startswith("test ") and not line.startswith("test result:")]
        summary = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;"
        require(lines.count("running 1 test") == 1 and tests == ["test scalar_contract ... ok"]
                and len(summaries) == 1 and summaries[0].startswith(summary),
                "Cargo scalar_contract test was not executed exactly once")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def json_load(data):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, f"duplicate JSON key: {key}")
            result[key] = value
        return result
    def constant(value):
        raise ValueError(f"non-finite JSON value: {value}")
    return json.loads(data, object_pairs_hook=pairs, parse_constant=constant)


def integer(value, low, high):
    return type(value) is int and low <= value <= high


def positive(value):
    return type(value) in (int, float) and math.isfinite(value) and value > 0


class Evidence:
    """Read a downloaded ZIP or extracted artifact, with no extraction/execution."""
    def __init__(self, source):
        self.source = Path(source)
        self.zip = None
        self.archive_sha256 = None
        if self.source.is_dir():
            self.root = self.source.resolve()
        else:
            self.archive_sha256 = sha256(self.source.read_bytes())
            self.zip = zipfile.ZipFile(self.source)
            infos = self.zip.infolist()
            names = [x.filename for x in infos]
            require(len(names) == len(set(names)), "duplicate ZIP members")
            require(sum(x.file_size for x in infos) <= 512 * 1024 * 1024, "ZIP too large")
            for info in infos:
                self.relative(info.filename.rstrip("/"))
                require((info.external_attr >> 16) & 0o170000 != 0o120000, "ZIP symlink")

    @staticmethod
    def relative(name):
        require(isinstance(name, str) and name and "\\" not in name and "\x00" not in name,
                "invalid evidence path")
        path = PurePosixPath(name)
        require(not path.is_absolute() and ".." not in path.parts and ":" not in name,
                "evidence path escapes root")
        return path

    def read(self, name):
        path = self.relative(name)
        if self.zip is not None:
            info = self.zip.getinfo(str(path))
            require(info.file_size <= 64 * 1024 * 1024, "evidence member too large")
            return self.zip.read(info)
        target = (self.root / str(path)).resolve()
        require(target.is_relative_to(self.root), "evidence symlink escapes root")
        require(target.stat().st_size <= 64 * 1024 * 1024, "evidence file too large")
        return target.read_bytes()

    def close(self):
        if self.zip is not None:
            self.zip.close()


def output_name(value):
    # The measured Linux runner serializes OsString as serde's Unix byte array.
    require(isinstance(value, dict) and set(value) == {"Unix"}, "unsupported output path encoding")
    require(isinstance(value["Unix"], list) and
            all(integer(x, 0, 255) for x in value["Unix"]), "invalid Unix path bytes")
    name = bytes(value["Unix"]).decode("utf-8")
    require(len(PurePosixPath(name).parts) == 1, "output must be a leaf filename")
    Evidence.relative(name)
    return name


def random64(state):
    state = (state + 0x9e3779b97f4a7c15) & MASK
    z = ((state ^ (state >> 30)) * 0xbf58476d1ce4e5b9) & MASK
    z = ((z ^ (z >> 27)) * 0x94d049bb133111eb) & MASK
    return state, z ^ (z >> 31)


def summarize(pairs, resamples, seed):
    require(pairs and len(pairs) <= 10000 and all(positive(v) for p in pairs for v in p),
            "positive finite paired times required")
    require(integer(resamples, 100, 1000000) and integer(seed, 0, MASK), "invalid statistics controls")
    require(len(pairs) * resamples <= 2_000_000, "replay work exceeds bounded reporter capacity")
    logs = [math.log(a) - math.log(b) for a, b in pairs]
    n = len(logs)
    mean = lambda xs: sum(x / n for x in xs)
    ratios = [math.exp(x) for x in logs]
    estimate = math.exp(mean(logs))
    require(all(positive(x) for x in ratios + [estimate]), "unrepresentable ratio")
    state, threshold, boots = seed, ((1 << 64) - n) % n, []
    for _ in range(resamples):
        sample = []
        for _ in range(n):
            while True:
                state, r = random64(state)
                if r >= threshold:
                    break
            sample.append(logs[r % n])
        boots.append(math.exp(mean(sample)))
    boots.sort()
    tolerance = 64 * sys.float_info.epsilon * max(sum(abs(x) for x in logs), 1)
    observed = abs(mean(logs))
    p = None
    if n <= 16:
        extreme = sum(abs(mean([x if mask & (1 << i) else -x for i, x in enumerate(logs)]))
                      + tolerance >= observed for mask in range(1 << n))
        p = extreme / (1 << n)
    a, b = sorted(x[0] for x in pairs), sorted(x[1] for x in pairs)
    median = lambda xs: xs[n // 2] if n % 2 else xs[n // 2 - 1] / 2 + xs[n // 2] / 2
    return dict(method=METHOD, pairs=n, raw_ratios=ratios, log_ratios=logs,
                geometric_ratio=estimate,
                bootstrap_95=[boots[math.floor(.025 * (resamples - 1))],
                              boots[math.ceil(.975 * (resamples - 1))]],
                baseline_median=median(a), candidate_median=median(b),
                baseline_p95=a[math.ceil(.95 * n) - 1], candidate_p95=b[math.ceil(.95 * n) - 1],
                exact_sign_flip_p=p, bootstrap_resamples=resamples, seed=seed)


def compare(got, expected, where):
    if isinstance(expected, dict):
        require(isinstance(got, dict) and set(got) == set(expected), f"{where}: fields differ")
        for key, value in expected.items():
            compare(got[key], value, f"{where}.{key}")
    elif isinstance(expected, list):
        require(isinstance(got, list) and len(got) == len(expected), f"{where}: length differs")
        for i, value in enumerate(expected):
            compare(got[i], value, f"{where}[{i}]")
    elif type(expected) is float:
        require(type(got) in (int, float) and math.isfinite(got) and
                math.isclose(got, expected, rel_tol=1e-12, abs_tol=1e-14), f"{where}: replay differs")
    else:
        require(type(got) is type(expected) and got == expected, f"{where}: value differs")


def source_map(value):
    require(isinstance(value, dict) and value, "missing source identity")
    require(all(isinstance(k, str) and isinstance(v, str) and SHA.fullmatch(v)
                for k, v in value.items()), "invalid source identity")
    return value


def measured_runtime(evidence, build, runtime):
    """Bind an immediate post-build execution to that lane's recorded artifact."""
    require(runtime.get("kind") == "command" and runtime.get("family") == build["family"]
            and runtime.get("lane") == build["lane"] and runtime.get("phase") == build["phase"] + "-runtime"
            and runtime.get("sample") is None, "measured build is missing its immediate runtime receipt")
    argv = [argument(value) for value in build["argv"]]
    flag = "-o" if build["family"] == "native" else "--target-dir"
    require(argv.count(flag) == 1 and argv.index(flag) + 1 < len(argv), "measured build output is ambiguous")
    output = argv[argv.index(flag) + 1]
    source = source_map(build["source_sha256"])
    if build["family"] == "cargo":
        # This fixed corpus uses Cargo's default dev output. This binding does
        # not extend the native schema-2 flag validator to every Cargo argument.
        output = str(PurePosixPath(output) / "debug/mixed-app")
        app = [path for path in source if path.endswith("/app/src/main.rs")]
        require(len(app) == 1, "measured Cargo application source is ambiguous")
        source = {app[0]: source[app[0]]}
    require([argument(value) for value in runtime["argv"]] == [output]
            and source_map(runtime["source_sha256"]) == source_map(runtime["source_after_sha256"]) == source,
            "measured runtime does not match its build artifact/source")
    prefix = "lustr-paired-measurements/"
    stdout = evidence.read(prefix + output_name(runtime["stdout"]))
    stderr = evidence.read(prefix + output_name(runtime["stderr"]))
    require(runtime["exit"] == runtime["expected_exit"] == 0 and stderr == b""
            and stdout == (b"57\n" if build["phase"] == "body-edit" else b"59\n"),
            "measured runtime result differs")


def verify_execution(evidence, rows, lanes_by_family, pairs, schema):
    """Require the recorder's lane order, post-build runs and owned shutdowns.

    Both historical schemas record these operations. Schema 2 adds only the
    reference-probe shutdown; schema-1 flags keep their original contract.
    """
    sample_scopes = {f"{family}-{sample}": 3 if family == "native" else 1
                     for family in lanes_by_family for sample in range(pairs)}
    expected_daemons = dict(sample_scopes, **{"native-probes": 3, "mixed-probes": 1})
    if schema == 2:
        expected_daemons["native-reference-probes"] = 1
    seen_daemons = set()
    active, schedule, position, pending = None, [], 0, None
    for record_index, row in enumerate(rows):
        if pending is not None:
            measured_runtime(evidence, pending, row)
            pending = None
            continue
        kind = row["kind"]
        if kind == "order":
            require(active is None, "sample was not completed and shut down before the next order")
            active = row["family"], row["sample"]
            lanes = lanes_by_family[row["family"]]
            schedule = [(phase, lanes[lane_index]) for phase in PHASES for lane_index in row["lanes"]]
            position = 0
        elif kind == "command" and row["phase"] in PHASES:
            require(active == (row["family"], row["sample"]), "measured command has no preceding sample order")
            require(position < len(schedule) and (row["phase"], row["lane"]) == schedule[position],
                    "recorded lane/phase order differs from executed commands")
            position += 1
            if row["phase"] in BUILD_PHASES:
                pending = row
        elif kind == "command" and row["phase"].endswith("-runtime"):
            require(active is None, "runtime receipt has no immediately preceding measured build")
            if row["phase"].removesuffix("-runtime") in BUILD_PHASES:
                previous = rows[record_index - 1] if record_index else {}
                # The historical mixed probe also names its untimed execution
                # cold-runtime. No other runtime with these names is untimed.
                require(row["family"] == "cargo" and row["lane"] == "mixed" and row["phase"] == "cold-runtime"
                        and previous.get("kind") == "command" and previous.get("family") == "cargo"
                        and previous.get("lane") == "mixed" and previous.get("phase") == "probe-cold",
                        "unexpected or duplicated measured runtime receipt")
        elif kind == "daemon-exits":
            scope, count = row.get("scope"), row.get("successful_exits")
            require(isinstance(scope, str) and scope in expected_daemons and scope not in seen_daemons,
                    "missing/duplicate/unrecognized daemon shutdown scope")
            require(type(count) is int and count >= expected_daemons[scope], "insufficient successful daemon exits")
            if scope in sample_scopes:
                require(active is not None and scope == f"{active[0]}-{active[1]}" and position == len(schedule),
                        "sample daemon shutdown precedes or disagrees with its measured work")
                active = None
            else:
                require(active is None, "probe shutdown interrupts a measured sample")
            seen_daemons.add(scope)
    require(pending is None and active is None, "last measured sample lacks execution or shutdown evidence")
    require(seen_daemons == set(expected_daemons), "missing daemon shutdown receipts")


def verify(evidence):
    prefix = "lustr-paired-measurements/"
    raw = evidence.read(prefix + "raw.jsonl")
    report = json_load(evidence.read(prefix + "report.json"))
    schema = report.get("schema")
    lanes_by_family, comparisons = campaign_layout(schema)
    require(report.get("status") == "completed", "campaign is incomplete/failed")
    require(report.get("raw_sha256") == sha256(raw), "raw SHA-256 mismatch")
    rows = [json_load(line) for line in raw.splitlines()]
    require(report.get("records") == rows, "report/raw record mismatch")
    n = report.get("pairs")
    require(integer(n, 2, 10000), "invalid pair count")
    if schema == 2:
        require(12 <= n <= 96 and n % 4 == 0, "schema-2 pair count must complete four-lane blocks")
    require(report.get("order_method") == "randomized-cyclic-blocks-v1", "unrecognized order method")
    identities = [x for x in rows if x.get("kind") == "identities"]
    require(len(identities) == 1, "missing/duplicate identities record")
    # Initial fail-closed status/summaries are intentionally not final outcomes.
    initial = identities[0]["report"]
    for key in ("schema", "pairs", "seed", "pristine_revision", "pristine_tree", "compiler_sha256",
                "profiles", "rust_reference_version", "cold_scope", "warm_scope", "timing_scope",
                "scope", "inference_limits", "order_method", "performance_admission"):
        require(report.get(key) == initial.get(key) and key in report, f"identity mismatch: {key}")
    require(report["performance_admission"] == "not granted by this runner", "unexpected admission claim")
    host = None
    if "host" in report or "host" in initial:
        require("host" in report and report["host"] == initial.get("host"), "host identity mismatch")
        host = report["host"]
        require(isinstance(host, dict) and set(host) == {"sha256", "record"}, "invalid host identity")
        raw_host = evidence.read(prefix + "host.json")
        require(sha256(raw_host) == host["sha256"], "host record digest mismatch")
        require(isinstance(host["record"], dict) and json_load(raw_host) == host["record"] and
                host["record"].get("schema") == "lustr-benchmark-host-v1", "host record mismatch")
    for key in ("pristine_revision", "pristine_tree"):
        require(re.fullmatch(r"[0-9a-f]{40}", report[key]) is not None, f"invalid {key}")
    require(report["compiler_sha256"] and all(SHA.fullmatch(v) for v in report["compiler_sha256"].values()),
            "missing/malformed compiler identities")
    reference, lane_executables = None, None
    if schema == 2:
        reference = report.get("native_rustc_reference")
        require(isinstance(reference, dict) and reference == initial.get("native_rustc_reference"), "native reference identity mismatch")
        require(reference == native_reference_contract(reference.get("compiler"), reference.get("test_driver")),
                "native reference command contract differs")
        require(all(isinstance(reference[key], str) and reference[key] in report["compiler_sha256"]
                    for key in ("compiler", "test_driver")), "native reference executable identity missing")
        lane_executables = report.get("native_lane_executables")
        require(isinstance(lane_executables, dict) and lane_executables == initial.get("native_lane_executables")
                and set(lane_executables) == set(LANES_V2["native"]), "native lane executable identity mismatch")
        require(all(isinstance(path, str) and path in report["compiler_sha256"] for path in lane_executables.values())
                and len(set(lane_executables.values())) == len(lane_executables), "missing or aliased native lane executable")
        require(lane_executables["rustc"] == reference["compiler"] and reference["test_driver"] not in lane_executables.values(),
                "native rustc compiler/adapter identity differs")
    cargo_lanes = cargo_executables(report)
    groups, outputs, orders, reference_probes, native_builds = {}, set(), {}, {}, {}
    counts = collections.Counter()
    for row in rows:
        kind = row.get("kind")
        require(kind in ("identities", "command", "order", "native-retained-check", "daemon-exits"),
                "unrecognized record kind")
        counts[kind] += 1
        if kind == "order":
            family, sample = row.get("family"), row.get("sample")
            require(family in lanes_by_family and integer(sample, 0, n - 1), "invalid order identity")
            key = family, sample
            require(key not in orders, "duplicate order")
            order = row.get("lanes")
            require(isinstance(order, list) and all(type(x) is int for x in order) and
                    sorted(order) == list(range(len(lanes_by_family[family]))), "invalid lane permutation")
            orders[key] = order
        if kind != "command":
            continue
        require(row.get("timed_out") is False and row.get("signal") is None, "timeout/signal")
        require(type(row.get("exit")) is int and type(row.get("expected_exit")) is int and
                row["exit"] == row["expected_exit"], "unexpected command exit")
        require(positive(row.get("seconds")), "invalid command duration")
        streams = {}
        for stream in ("stdout", "stderr"):
            name = output_name(row[stream])
            require(name not in outputs, "duplicate command output")
            outputs.add(name)
            data = evidence.read(prefix + name)
            streams[stream] = data
            require(type(row.get(stream + "_bytes")) is int and len(data) == row[stream + "_bytes"] and
                    sha256(data) == row.get(stream + "_sha256"), f"changed/missing {stream} evidence")
        if schema == 2 and row.get("family") == "native":
            require(row.get("lane") in lanes_by_family["native"], "unknown native lane")
            output = native_command(row, streams, reference, lane_executables)
            if row["phase"].endswith("-runtime"):
                build = native_builds.pop(row["lane"], None)
                require(build is not None and row["phase"] == build["phase"] + "-runtime"
                        and row["source_sha256"] == build["source"] and argument(row["argv"][0]) == build["output"],
                        "native runtime is not the matching lane/source build output")
            elif output is not None:
                native_builds[row["lane"]] = dict(phase=row["phase"], source=row["source_sha256"], output=output)
        if row.get("phase") not in PHASES:
            allowed = UNTIMED | (set(REFERENCE_PROBES) if schema == 2 else set())
            require(row.get("phase") in allowed and row.get("sample") is None,
                    "unrecognized command phase/sample")
            if row["phase"] in REFERENCE_PROBES:
                key = row.get("lane"), row["phase"]
                require(row.get("family") == "native" and key[0] in ("entry", "rustc")
                        and key not in reference_probes and row["expected_exit"] == REFERENCE_PROBES[key[1]],
                        "native reference correctness gate differs")
                reference_probes[key] = row
            continue
        family, lane, phase, sample = (row.get(k) for k in ("family", "lane", "phase", "sample"))
        require(family in lanes_by_family and lane in lanes_by_family[family] and integer(sample, 0, n - 1),
                "invalid timed sample identity")
        expected_exit = (1 if family == "native" else 101) if phase == "rejection" else 0
        require(row["expected_exit"] == expected_exit, "wrong measured exit contract")
        before = source_map(row.get("source_sha256"))
        require(before == source_map(row.get("source_after_sha256")), "source changed during timing")
        if family == "cargo":
            cargo_command(row, streams, cargo_lanes)
        key = family, lane, phase, sample
        require(key not in groups, "duplicate timed sample")
        groups[key] = row
    verify_execution(evidence, rows, lanes_by_family, n, schema)
    expected_keys = {(f, lane, phase, i) for f, lanes in lanes_by_family.items() for lane in lanes
                     for phase in PHASES for i in range(n)}
    require(set(groups) == expected_keys, "missing/unexpected timed samples")
    require(set(orders) == {(f, i) for f in lanes_by_family for i in range(n)}, "missing orders")
    if schema == 2:
        require(set(reference_probes) == {(lane, phase) for lane in ("entry", "rustc") for phase in REFERENCE_PROBES},
                "missing native reference correctness gates")
        for phase in REFERENCE_PROBES:
            a, b = (reference_probes[lane, phase] for lane in ("entry", "rustc"))
            require(source_map(a["source_sha256"]) == source_map(a["source_after_sha256"])
                    == source_map(b["source_sha256"]) == source_map(b["source_after_sha256"]),
                    "native reference correctness gates use different source")
    for family, lanes in lanes_by_family.items():
        for position in range(len(lanes)):
            positions = collections.Counter(orders[family, i][position] for i in range(n))
            amounts = [positions[x] for x in range(len(lanes))]
            require(max(amounts) - min(amounts) <= 1, "unbalanced lane positions")
        for phase in PHASES:
            for i in range(n):
                source = groups[family, lanes[0], phase, i]["source_sha256"]
                require(all(groups[family, lane, phase, i]["source_sha256"] == source for lane in lanes),
                        "paired lanes use different source")
    retained = [x for x in rows if x["kind"] == "native-retained-check"]
    require(len(retained) == 3 and {x.get("lane") for x in retained} == set(LANES["native"]),
            "missing/duplicate retained-check evidence")
    require(all(x.get("route") in ("reuse", "bodies-without-body-work") for x in retained),
            "unrecognized retained-check route")
    expected_summaries = {(family, baseline, candidate, phase) for family, baseline, candidate in comparisons for phase in PHASES}
    require(isinstance(report.get("summaries"), list), "missing summaries")
    seen = set()
    for item in report["summaries"]:
        key = tuple(item.get(k) for k in ("family", "baseline", "candidate", "phase"))
        require(key in expected_summaries and key not in seen, "duplicate/unexpected summary")
        seen.add(key)
        family, baseline, candidate, phase = key
        a = item["analysis"]
        require(a.get("seed") == report["seed"] and a.get("method") == METHOD and
                item.get("ratio_direction") == DIRECTION, "statistics identity/direction mismatch")
        paired = [(groups[family, baseline, phase, i]["seconds"],
                   groups[family, candidate, phase, i]["seconds"]) for i in range(n)]
        actual = summarize(paired, a["bootstrap_resamples"], a["seed"])
        compare(a, actual, "/".join(key))
        lo, hi = actual["bootstrap_95"]
        observation = "faster-on-this-corpus" if lo > 1 else "slower-on-this-corpus" if hi < 1 else "inconclusive"
        require(item.get("observation") == observation, "observation disagrees with interval")
    require(seen == expected_summaries, "missing summary")
    source = evidence.read("lustr-source-revision.txt").decode("ascii").splitlines()
    require(len(source) == 2 and all(re.fullmatch(r"[0-9a-f]{40}", x) for x in source),
            "invalid checkout/tree receipt")
    samples = {f + "/" + lane + "/" + phase:
               [groups[f, lane, phase, i]["seconds"] for i in range(n)]
               for f, lanes in lanes_by_family.items() for lane in lanes for phase in PHASES}
    result = dict(schema="lustr-paired-replay-v1", kind="derived evidence; not original raw.jsonl",
                  checkout=source[0], tree=source[1], archive_sha256=evidence.archive_sha256,
                  raw_sha256=report["raw_sha256"], pairs=n, seed=report["seed"],
                  pristine_revision=report["pristine_revision"], pristine_tree=report["pristine_tree"],
                  rust_reference_version=report["rust_reference_version"],
                  profiles=report["profiles"], scope=report["scope"],
                  compiler_sha256=report["compiler_sha256"],
                  cold_scope=report["cold_scope"], warm_scope=report["warm_scope"],
                  timing_scope=report["timing_scope"], inference_limits=report["inference_limits"],
                  units="seconds", sample_index="array index is original pair/sample id; no exclusions",
                  timing_samples=samples, summaries=report["summaries"],
                  verified=dict(records=len(rows), command_outputs=len(outputs), timed_commands=len(groups),
                                comparisons=len(seen)),
                  performance_admission="not granted; replay is not compiler qualification")
    if host is not None:
        result["host"] = host
    if schema == 2:
        result.update(schema="lustr-paired-replay-v2", campaign_schema=2, native_rustc_reference=reference,
                      native_lane_executables=lane_executables)
    return result


def markdown(result):
    lines = ["# Paired Lustr performance evidence", "",
             f"Measured checkout: `{result['checkout']}`; tree: `{result['tree']}`.",
             f"Original raw JSONL SHA-256: `{result['raw_sha256']}`.",
             f"{result['pairs']} pairs per mode; all samples retained. Ratios are baseline/candidate time; >1 is faster.",
             "Medians/p95 are milliseconds; ratio is geometric paired mean, not ratio of medians.",
             "Exploratory shared-host intervals are not equivalence tests or full-workload speed proof.",
             "No compiler/release admission is granted by this replay.", ""]
    if result.get("campaign_schema", 1) == 2:
        lines += ["The native rustc lane uses the matching SDK on the identical standalone source, nonincrementally. "
                  "Every no-op starts rustc again; it is not Cargo Fresh. Check emits metadata. "
                  "Compile-plus-test includes rustc --test and actual execution of the one test, with both child receipts retained.", ""]
    for family, baseline, candidate in campaign_layout(result.get("campaign_schema", 1))[1]:
        lines += [f"## {family}: {baseline} → {candidate}", "",
                  "| Mode | Baseline median | Candidate median | Baseline p95 | Candidate p95 | Ratio [95% interval] | Observation |",
                  "|---|---:|---:|---:|---:|---:|---|"]
        for phase in PHASES:
            item = next(s for s in result["summaries"] if (s["family"], s["baseline"], s["candidate"], s["phase"])
                        == (family, baseline, candidate, phase))
            a = item["analysis"]
            lo, hi = a["bootstrap_95"]
            values = " | ".join(f"{1000 * a[k]:.3f}" for k in
                                ("baseline_median", "candidate_median", "baseline_p95", "candidate_p95"))
            lines.append(f"| {phase} | {values} | {a['geometric_ratio']:.3f} [{lo:.3f}, {hi:.3f}] | {item['observation']} |")
        lines.append("")
    return "\n".join(lines) + "\n"


def csv_samples(result):
    """Compact lossless timing projection; pair ids remain array/column indices."""
    output = io.StringIO(newline="")
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["series"] + [f"pair_{i}_seconds" for i in range(result["pairs"])])
    for series, values in result["timing_samples"].items():
        writer.writerow([series] + values)
    return output.getvalue()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact", type=Path, help="downloaded ZIP or extracted artifact root")
    parser.add_argument("--output", type=Path, required=True, help="new output directory (must not exist)")
    args = parser.parse_args()
    evidence = None
    try:
        require(not args.output.exists(), "output already exists; preserve earlier evidence")
        evidence = Evidence(args.artifact)
        result = verify(evidence)
        # Write only after the entire input has passed verification.
        args.output.mkdir(parents=True, exist_ok=False)
        (args.output / "samples.json").write_text(json.dumps(result, indent=2, allow_nan=False) + "\n", encoding="utf-8")
        (args.output / "README.md").write_text(markdown(result), encoding="utf-8")
        (args.output / "timings.csv").write_text(csv_samples(result), encoding="utf-8")
        print(json.dumps(result["verified"], sort_keys=True))
    except (ValueError, KeyError, TypeError, OSError, OverflowError, zipfile.BadZipFile) as exc:
        print(f"performance evidence rejected: {exc}", file=sys.stderr)
        return 1
    finally:
        if evidence is not None:
            evidence.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
