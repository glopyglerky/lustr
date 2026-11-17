#!/usr/bin/env python3
"""Verify and render lustr-mixed-bench schema-1 evidence; never execute its argv.

This is an evidence/statistics replay, not compiler qualification or release
admission. The statistics mirror crates/lustr-bench/src/stats.rs (paired-log-v1).
Only the existing nine-mode/five-lane campaign is accepted. A changed campaign
requires a reviewed schema update, not silently dropping unfamiliar samples.
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
LANES = {"native": ("pristine", "integrated", "entry"), "cargo": ("reference", "mixed")}
METHOD = "paired-log-v1/splitmix64/rejection-index/percentile"
DIRECTION = "baseline time / candidate time; greater than 1 is faster"
UNTIMED = {"excluded-prime", "cold-runtime", "no-op-runtime", "body-edit-runtime",
           "signature-edit-runtime", "generate-lockfile", "prime", "warm", "reuse",
           "body-probe", "probe-cold", "probe-body", "body-runtime", "probe-check",
           "probe-check-body", "probe-rejection"}
MASK = (1 << 64) - 1
SHA = re.compile(r"[0-9a-f]{64}\Z")


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


def verify(evidence):
    prefix = "lustr-paired-measurements/"
    raw = evidence.read(prefix + "raw.jsonl")
    report = json_load(evidence.read(prefix + "report.json"))
    require(report.get("schema") == 1 and type(report.get("schema")) is int, "unsupported schema")
    require(report.get("status") == "completed", "campaign is incomplete/failed")
    require(report.get("raw_sha256") == sha256(raw), "raw SHA-256 mismatch")
    rows = [json_load(line) for line in raw.splitlines()]
    require(report.get("records") == rows, "report/raw record mismatch")
    n = report.get("pairs")
    require(integer(n, 2, 10000), "invalid pair count")
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
    for key in ("pristine_revision", "pristine_tree"):
        require(re.fullmatch(r"[0-9a-f]{40}", report[key]) is not None, f"invalid {key}")
    require(report["compiler_sha256"] and all(SHA.fullmatch(v) for v in report["compiler_sha256"].values()),
            "missing/malformed compiler identities")
    groups, outputs, orders = {}, set(), {}
    counts = collections.Counter()
    for row in rows:
        kind = row.get("kind")
        require(kind in ("identities", "command", "order", "native-retained-check", "daemon-exits"),
                "unrecognized record kind")
        counts[kind] += 1
        if kind == "order":
            family, sample = row.get("family"), row.get("sample")
            require(family in LANES and integer(sample, 0, n - 1), "invalid order identity")
            key = family, sample
            require(key not in orders, "duplicate order")
            order = row.get("lanes")
            require(isinstance(order, list) and all(type(x) is int for x in order) and
                    sorted(order) == list(range(len(LANES[family]))), "invalid lane permutation")
            orders[key] = order
        if kind != "command":
            continue
        require(row.get("timed_out") is False and row.get("signal") is None, "timeout/signal")
        require(type(row.get("exit")) is int and type(row.get("expected_exit")) is int and
                row["exit"] == row["expected_exit"], "unexpected command exit")
        require(positive(row.get("seconds")), "invalid command duration")
        for stream in ("stdout", "stderr"):
            name = output_name(row[stream])
            require(name not in outputs, "duplicate command output")
            outputs.add(name)
            data = evidence.read(prefix + name)
            require(type(row.get(stream + "_bytes")) is int and len(data) == row[stream + "_bytes"] and
                    sha256(data) == row.get(stream + "_sha256"), f"changed/missing {stream} evidence")
        if row.get("phase") not in PHASES:
            require(row.get("phase") in UNTIMED and row.get("sample") is None,
                    "unrecognized command phase/sample")
            continue
        family, lane, phase, sample = (row.get(k) for k in ("family", "lane", "phase", "sample"))
        require(family in LANES and lane in LANES[family] and integer(sample, 0, n - 1),
                "invalid timed sample identity")
        expected_exit = (1 if family == "native" else 101) if phase == "rejection" else 0
        require(row["expected_exit"] == expected_exit, "wrong measured exit contract")
        before = source_map(row.get("source_sha256"))
        require(before == source_map(row.get("source_after_sha256")), "source changed during timing")
        key = family, lane, phase, sample
        require(key not in groups, "duplicate timed sample")
        groups[key] = row
    expected_keys = {(f, lane, phase, i) for f, lanes in LANES.items() for lane in lanes
                     for phase in PHASES for i in range(n)}
    require(set(groups) == expected_keys, "missing/unexpected timed samples")
    require(set(orders) == {(f, i) for f in LANES for i in range(n)}, "missing orders")
    for family, lanes in LANES.items():
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
    expected_summaries = {(f, lanes[0], lane, phase) for f, lanes in LANES.items()
                          for lane in lanes[1:] for phase in PHASES}
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
               for f, lanes in LANES.items() for lane in lanes for phase in PHASES}
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
    return result


def markdown(result):
    lines = ["# Paired Lustr performance evidence", "",
             f"Measured checkout: `{result['checkout']}`; tree: `{result['tree']}`.",
             f"Original raw JSONL SHA-256: `{result['raw_sha256']}`.",
             f"{result['pairs']} pairs per mode; all samples retained. Ratios are baseline/candidate time; >1 is faster.",
             "Medians/p95 are milliseconds; ratio is geometric paired mean, not ratio of medians.",
             "Exploratory shared-host intervals are not equivalence tests or full-workload speed proof.",
             "No compiler/release admission is granted by this replay.", ""]
    for family, baseline, candidate in (("native", "pristine", "integrated"),
                                         ("native", "pristine", "entry"), ("cargo", "reference", "mixed")):
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
