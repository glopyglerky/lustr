#!/usr/bin/env python3
"""Assess a fully replayed complete-package campaign against practical warm budgets.

This never executes a compiler or changes the original report, raw evidence or
strict warm_target. It does not grant release admission or assess general Rust.
"""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("warm_package_evidence", HERE / "run-warm-native-comparison.py")
N = importlib.util.module_from_spec(spec)
spec.loader.exec_module(N)
require = N.require
SCHEMA = "lustr-practical-warm-performance-v1"
RELATIVE_BUDGET = 1.10
ABSOLUTE_BUDGET_SECONDS = 0.0005
RESAMPLES = 10000
SEEDS = {"no-op-check": 0x4c55535452574152, "no-op-build": 0x4c55535452574153}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def serialized(value):
    return (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + "\n").encode()


def median(values):
    ordered = sorted(values)
    n = len(ordered)
    return ordered[n // 2] if n % 2 else ordered[n // 2 - 1] / 2 + ordered[n // 2] / 2


def added_latency(pairs, seed):
    """Bootstrap paired candidate-minus-pristine differences, keeping pair identity."""
    require(len(pairs) == 24 and all(N.S.STATS.positive(x) for pair in pairs for x in pair),
            "24 positive finite matched timing pairs required")
    differences = [candidate - pristine for pristine, candidate in pairs]
    n, state, boots = len(differences), seed, []
    threshold = ((1 << 64) - n) % n
    for _ in range(RESAMPLES):
        sample = []
        for _ in range(n):
            while True:
                state, value = N.S.STATS.random64(state)
                if value >= threshold:
                    break
            sample.append(differences[value % n])
        boots.append(median(sample))
    boots.sort()
    return {"method": "paired-difference-median/splitmix64/rejection-index/percentile-v1",
            "direction": "candidate seconds minus pristine Lust seconds for each matched pair",
            "pairs": n, "paired_differences_seconds": differences,
            "median_added_seconds": median(differences),
            "bootstrap_95_seconds": [boots[math.floor(.025 * (RESAMPLES - 1))],
                                     boots[math.ceil(.975 * (RESAMPLES - 1))]],
            "bootstrap_resamples": RESAMPLES, "seed": seed}


def assess(campaign, status_file):
    status_bytes = status_file.read_bytes()
    require(status_bytes in (b"0", b"0\n", b"2", b"2\n"),
            "only original measurement exit 0 or 2 may be assessed")
    measurement_exit = int(status_bytes)
    report_bytes = (campaign / "report.json").read_bytes()
    raw_bytes = (campaign / "raw.jsonl").read_bytes()
    # Full existing verification is mandatory, including the original strict
    # target, package builds/gates, raw commands, samples and execution evidence.
    report = N.verify(campaign)
    require(report_bytes == (campaign / "report.json").read_bytes()
            and raw_bytes == (campaign / "raw.jsonl").read_bytes()
            and status_bytes == status_file.read_bytes(), "assessment inputs changed during replay")
    strict = report["warm_target"]
    require(measurement_exit == (0 if strict["meets_target"] else 2),
            "original measurement exit disagrees with verified strict target")
    # N.verify proves report samples equal raw timing records and binds every
    # timing to its actual command. Pair by sample identity, never sorted time.
    values = {(x["sample"], x["lane"], x["phase"]): x["seconds"] for x in report["samples"]}
    modes = {}
    for phase, seed in SEEDS.items():
        lust, = [x["analysis"] for x in report["summaries"] if
                 (x["phase"], x["baseline"], x["candidate"]) == (phase, "pristine", "entry")]
        rust, = [x["analysis"] for x in report["summaries"] if
                 (x["phase"], x["baseline"], x["candidate"]) == (phase, "cargo", "entry")]
        latency = added_latency([(values[i, "pristine", phase], values[i, "entry", phase])
                                 for i in range(report["pairs"])], seed)
        relative_met = lust["bootstrap_95"][0] >= 1 / RELATIVE_BUDGET
        absolute_met = latency["bootstrap_95_seconds"][1] <= ABSOLUTE_BUDGET_SECONDS
        rust_win = rust["bootstrap_95"][0] > 1
        modes[phase] = {
            "meets_practical_target": (relative_met or absolute_met) and rust_win,
            "relative_budget_met": relative_met, "absolute_budget_met": absolute_met,
            "faster_than_rustc": rust_win,
            "lust_over_candidate_ratio": lust["geometric_ratio"],
            "lust_over_candidate_bootstrap_95": lust["bootstrap_95"],
            "rustc_over_candidate_ratio": rust["geometric_ratio"],
            "rustc_over_candidate_bootstrap_95": rust["bootstrap_95"],
            "added_latency": latency}
    passed = all(x["meets_practical_target"] for x in modes.values())
    return {"schema": SCHEMA, "status": "met" if passed else "not-met",
            "meets_practical_target": passed, "release_admission": False,
            "scope": "warm check and build on this verified complete-package campaign only",
            "criterion": "Each warm mode: lower Lust/candidate 95% ratio >= 1/1.10 OR upper paired-median added-latency 95% bound <= 0.0005 seconds; AND lower Rust/candidate 95% ratio > 1.",
            "relative_overhead_budget": 0.10,
            "absolute_overhead_budget_seconds": ABSOLUTE_BUDGET_SECONDS,
            "measurement_exit_status": measurement_exit,
            "original_strict_warm_target": strict,
            "candidate_identity": report["candidate_identity"],
            "baseline_identity": report["baseline_identity"],
            "input_sha256": {"report.json": digest(report_bytes), "raw.jsonl": digest(raw_bytes),
                             "measurement.status": digest(status_bytes)},
            "assessor_sha256": digest(Path(__file__).read_bytes()),
            "verifier_sha256": digest((HERE / "run-warm-native-comparison.py").read_bytes()),
            "modes": modes,
            "limits": [
                "Budgets were chosen after inspecting earlier measurements; historical strict results remain unchanged.",
                "The added-latency statistic is the median of matched differences, not the difference of marginal medians.",
                "The 10000 paired bootstrap resamples use fixed SplitMix64 seeds and rejection-sampled pair indices; percentile bounds use floor(0.025*(B-1)) and ceil(0.975*(B-1)).",
                "Intervals are conditional on the observed host, source and repeated workload. They do not establish portability, causal optimization benefit or universal equivalence.",
                "Cold work, other edits, rejection, tails and program runtime remain in the original report. This assessment does not replace those results or grant correctness/platform release admission."]}


def render(report):
    lines = ["# Practical warm-performance assessment", "",
             f"Practical target met: **{str(report['meets_practical_target']).lower()}**.",
             f"Original strict target met: **{str(report['original_strict_warm_target']['meets_target']).lower()}**. Original measurement exit: `{report['measurement_exit_status']}`.",
             "", report["criterion"], "",
             "Full original package/command/runtime evidence replay passed before this assessment. Original report and raw evidence remain unchanged.", "",
             "| Operation | Lust / candidate 95% interval | Added-latency median / 95% interval, ms | Rust / candidate 95% interval | Practical target |",
             "| --- | --- | --- | --- | --- |"]
    for phase in SEEDS:
        mode = report["modes"][phase]
        lo, hi = mode["lust_over_candidate_bootstrap_95"]
        rlo, rhi = mode["rustc_over_candidate_bootstrap_95"]
        added = mode["added_latency"]
        alo, ahi = [x * 1000 for x in added["bootstrap_95_seconds"]]
        lines.append(f"| {phase} | [{lo:.6f}, {hi:.6f}] | {added['median_added_seconds'] * 1000:.6f} / [{alo:.6f}, {ahi:.6f}] | [{rlo:.6f}, {rhi:.6f}] | {'met' if mode['meets_practical_target'] else 'not met'} |")
    lines += ["", "Each operation must pass independently. This is a warm performance decision only; release admission remains separate.", ""]
    lines += ["- " + line for line in report["limits"]]
    return "\n".join(lines) + "\n"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--campaign", type=Path, required=True)
    parser.add_argument("--measurement-status", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        campaign, output = args.campaign.resolve(), args.output.resolve()
        require(not output.exists() and not output.is_relative_to(campaign)
                and not campaign.is_relative_to(output), "new separate assessment output directory required")
        report = assess(campaign, args.measurement_status)
        output.mkdir(parents=True)
        files = {"assessment.json": serialized(report), "README.md": render(report).encode()}
        for name, data in files.items():
            (output / name).write_bytes(data)
        (output / "SHA256SUMS").write_text("".join(f"{digest(data)}  {name}\n" for name, data in sorted(files.items())))
        print(render(report))
        return 0 if report["meets_practical_target"] else 2
    except (OSError, ValueError, KeyError, TypeError, AssertionError) as error:
        print(f"Warm assessment rejected: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
