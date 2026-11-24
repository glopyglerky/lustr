#!/usr/bin/env python3
"""Verify and replay the retained symbol-check component measurements.

Reads evidence only. It never executes an archived command or binary.
"""
import argparse
from collections import Counter
import csv
import hashlib
import json
import math
from pathlib import Path
import random
import statistics


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replay(root):
    identity = json.loads((root / "identity.json").read_text())
    summary = json.loads((root / "summary.json").read_text())
    for filename, field in (("fixture.c", "fixture_sha256"), ("fixture.o", "object_sha256"),
                            ("samples.csv", "samples_sha256"), ("summary.json", "summary_sha256")):
        require(digest(root / filename) == identity[field], f"changed {filename}")
    require((root / "fixture.o").stat().st_size == identity["object_bytes"], "object size mismatch")
    require((root / "stderr").read_bytes() == b"", "component probe had diagnostics")
    source = identity["source_sha256"]
    require(set(source) == {"crates/lustr-native-core/src/mixed_symbols.rs", "tools/check-mixed-symbols.rs"}, "unknown component source files")
    for name, expected in source.items():
        require(digest(root / "source" / name) == expected, f"changed source {name}")
    with (root / "samples.csv").open(newline="") as stream:
        reader = csv.DictReader(stream)
        require(reader.fieldnames == ["pair", "position", "lane", "elapsed_ns"], "unexpected sample schema")
        rows = list(reader)
    require(summary["pairs"] == 64 and summary["timed_values"] == len(rows) == 128
            and summary["excluded_values"] == 0, "incomplete component pairs")
    groups = {}
    lanes = {"nm": [], "in-process": []}
    for ordinal, row in enumerate(rows):
        require(row["lane"] in lanes and all(row[key].isdigit() for key in ("pair", "position", "elapsed_ns")), "invalid component sample")
        pair, position, elapsed = (int(row[key]) for key in ("pair", "position", "elapsed_ns"))
        require(0 <= pair < 64 and position in (0, 1) and elapsed > 0, "invalid sample value")
        require((pair, position) == (ordinal // 2, ordinal % 2), "component rows do not follow recorded execution order")
        group = groups.setdefault(pair, {})
        require(row["lane"] not in group, "duplicate component sample")
        group[row["lane"]] = (position, elapsed)
        lanes[row["lane"]].append(elapsed)
    logs = []
    for pair in range(64):
        group = groups[pair]
        require(set(group) == set(lanes) and group["nm"][0] == pair % 2
                and group["in-process"][0] == 1 - pair % 2, "incorrect paired ordering")
        logs.append(math.log(group["nm"][1] / group["in-process"][1]))
    require(all(Counter(group[lane][0] for group in groups.values()) == {0: 32, 1: 32} for lane in lanes), "unbalanced positions")
    medians = {lane: statistics.median(values) for lane, values in lanes.items()}
    require(medians == summary["medians_ns"], "median replay mismatch")
    require(summary["bootstrap_replicates"] == 10000 and summary["bootstrap_seed"] == 20261117, "unknown bootstrap contract")
    # Python Random uses MT19937. choices draws each index as floor(random()*64).
    # Whole paired log-ratios are sampled, preserving the within-pair contrast.
    rng = random.Random(20261117)
    boot = sorted(math.exp(statistics.mean(rng.choices(logs, k=64))) for _ in range(10000))
    def quantile(probability):
        at = (len(boot) - 1) * probability
        lower = int(at)
        return boot[lower] + (boot[min(lower + 1, len(boot) - 1)] - boot[lower]) * (at - lower)
    ratio = math.exp(statistics.mean(logs))
    interval = [quantile(0.025), quantile(0.975)]
    require(math.isclose(ratio, summary["geometric_mean_nm_over_in_process"], rel_tol=1e-12), "ratio replay mismatch")
    require(all(math.isclose(value, original, rel_tol=1e-12) for value, original in zip(interval, summary["conditional_paired_bootstrap_95pct"], strict=True)), "interval replay mismatch")
    return {"verified_pairs": 64, "timed_values": 128, "medians_ns": medians,
            "geometric_mean_nm_over_in_process": ratio, "conditional_paired_bootstrap_95pct": interval,
            "scope": "one local symbol-check operation; not whole-compiler speed"}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    print(json.dumps(replay(args.directory), indent=2))
