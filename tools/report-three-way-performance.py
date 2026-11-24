#!/usr/bin/env python3
"""Render source-bound Lustr, pristine Lust, and rustc timings without rerunning compilers.

Input benchmark steps retain their original evidence and tests. This report is a
separate, sanitized projection; its output is never a performance-admission gate.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re
import statistics
import sys


SCHEMA = "lustr-three-way-performance-v1"
SHA40 = re.compile(r"[0-9a-f]{40}\Z")
SHA64 = re.compile(r"[0-9a-f]{64}\Z")
FIXED_PHASES = ("body-edit", "rejection", "compile-plus-test", "no-op", "check-no-op",
                "check-body-edit", "check", "cold", "signature-edit")
SLICE_PHASES = ("cold-check", "no-op-check", "cold-build", "no-op-build", "compile-plus-run", "runtime")
PROJECT_PHASES = ("clean-dependency-build", "primed-root-build", "primed-root-check", "no-op", "compile-plus-test")
FAMILIES = (
    ("fixed-cargo", "Fixed Cargo", FIXED_PHASES, False, "rustc via Cargo"),
    ("native-scalar", "Native scalar", FIXED_PHASES, True, "standalone rustc"),
    ("public-slice", "Public slice", SLICE_PHASES, True, "rustc via Cargo"),
    ("unchanged-project", "Unchanged project", PROJECT_PHASES, False, "rustc via Cargo"),
)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def cell(status: str, reason: str | None = None, median_ms: float | None = None,
         p95_ms: float | None = None) -> dict:
    result = {"status": status}
    if reason:
        result["reason"] = reason
    if median_ms is not None:
        result["median_ms"] = median_ms
    if p95_ms is not None:
        result["p95_ms"] = p95_ms
    return result


def empty_family(family: tuple, outcome: str) -> dict:
    name, title, phases, has_lust, rustc_mode = family
    if outcome in ("failure", "cancelled"):
        status, reason, measured_status = "FAIL", "benchmark step " + outcome, "FAIL"
    elif outcome == "skipped":
        status, reason, measured_status = "NT", "benchmark step skipped", "NT"
    else:
        status, reason, measured_status = "FAIL", "missing or invalid source-bound benchmark evidence after success", "FAIL"
    rows = []
    for phase in phases:
        rows.append({"operation": phase,
                     "lustr": cell(measured_status, reason),
                     "lust": cell(measured_status, reason) if has_lust else cell("NT", "no matched pristine Lust lane"),
                     "rustc": cell(measured_status, reason)})
    return {"name": name, "title": title, "status": status, "reason": reason,
            "rustc_mode": rustc_mode, "rows": rows}


def load_existing_replay(filename: str, alias: str):
    path = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(alias, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def measured(median_seconds: float, p95_seconds: float) -> dict:
    if not (type(median_seconds) in (int, float) and type(p95_seconds) in (int, float)
            and math.isfinite(median_seconds) and math.isfinite(p95_seconds)
            and 0 < median_seconds <= p95_seconds):
        raise ValueError("invalid source-bound duration statistics")
    return cell("MEASURED", median_ms=round(1000 * median_seconds, 9),
                p95_ms=round(1000 * p95_seconds, 9))


def paired(summary: dict) -> dict:
    analysis = summary["analysis"]
    ratio, interval = analysis["geometric_ratio"], analysis["bootstrap_95"]
    if not (type(ratio) in (float, int) and math.isfinite(ratio) and ratio > 0
            and isinstance(interval, list) and len(interval) == 2
            and all(type(value) in (float, int) and math.isfinite(value) and value > 0 for value in interval)):
        raise ValueError("invalid paired statistics")
    return {"rust_or_lust_over_lustr": ratio, "bootstrap_95": interval,
            "direction": "reference time / Lustr time; greater than one favors Lustr"}


def replay_equal(left, right) -> bool:
    """Allow only floating-point roundoff between stored and replayed stats."""
    if type(left) in (int, float) and type(right) in (int, float):
        return math.isfinite(left) and math.isfinite(right) and math.isclose(left, right, rel_tol=1e-12, abs_tol=1e-14)
    if isinstance(left, dict) and isinstance(right, dict):
        return left.keys() == right.keys() and all(replay_equal(left[key], right[key]) for key in left)
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(replay_equal(a, b) for a, b in zip(left, right))
    return type(left) is type(right) and left == right


def population(family: tuple, rows: list, source: dict, pairs: int) -> dict:
    name, title, phases, _, rustc_mode = family
    if [row["operation"] for row in rows] != list(phases) or not (type(pairs) is int and pairs >= 8):
        raise ValueError("incomplete benchmark phase coverage")
    return {"name": name, "title": title, "status": "MEASURED", "reason": "replayed qualified evidence",
            "rustc_mode": rustc_mode, "pairs": pairs, "source": source, "rows": rows}


def fixed_families(root: Path, revision: str) -> tuple[dict, dict]:
    if not (root / "lustr-paired-measurements/report.json").is_file():
        raise FileNotFoundError("missing fixed benchmark report")
    original = load_existing_replay("report-mixed-performance.py", "original_mixed_replay")
    evidence = original.Evidence(root)
    try:
        verified = original.verify(evidence)
    finally:
        evidence.close()
    if verified.get("checkout") != revision or verified.get("campaign_schema") != 2:
        raise ValueError("fixed benchmark source or required native rustc lane differs")
    items = {(s["family"], s["baseline"], s["candidate"], s["phase"]): s for s in verified["summaries"]}
    cargo_rows, native_rows = [], []
    for phase in FIXED_PHASES:
        cargo = items["cargo", "reference", "mixed", phase]
        cp = cargo["analysis"]
        cargo_rows.append({"operation": phase,
                           "lustr": measured(cp["candidate_median"], cp["candidate_p95"]),
                           "lust": cell("NT", "no matched pristine Lust Cargo lane"),
                           "rustc": measured(cp["baseline_median"], cp["baseline_p95"]),
                           "rustc_over_lustr": paired(cargo)})
        pristine = items["native", "pristine", "entry", phase]
        rustc = items["native", "rustc", "entry", phase]
        ap, rp = pristine["analysis"], rustc["analysis"]
        if not (math.isclose(ap["candidate_median"], rp["candidate_median"], rel_tol=1e-12)
                and math.isclose(ap["candidate_p95"], rp["candidate_p95"], rel_tol=1e-12)):
            raise ValueError("native packaged entry timing differs across the comparisons")
        native_rows.append({"operation": phase,
                            "lustr": measured(ap["candidate_median"], ap["candidate_p95"]),
                            "lust": measured(ap["baseline_median"], ap["baseline_p95"]),
                            "rustc": measured(rp["baseline_median"], rp["baseline_p95"]),
                            "lust_over_lustr": paired(pristine), "rustc_over_lustr": paired(rustc)})
    pristine_path = verified["native_lane_executables"]["pristine"]
    source = {"revision": verified["checkout"], "tree": verified["tree"],
              "original_raw_sha256": verified["raw_sha256"],
              "pristine_lust_sha256": verified["compiler_sha256"][pristine_path],
              "verified": verified["verified"],
              "input_report_sha256": source_digest(root / "lustr-paired-measurements/report.json")}
    return (population(FAMILIES[0], cargo_rows, source, verified["pairs"]),
            population(FAMILIES[1], native_rows, source, verified["pairs"]))


def slice_family(root: Path, revision: str) -> dict:
    if not (root / "report.json").is_file():
        raise FileNotFoundError("missing public slice report")
    original = load_existing_replay("run-upstream-slice-benchmark.py", "original_slice_replay")
    verified = original.replay(root)
    identity = verified.get("candidate_revision_and_tree")
    if not (isinstance(identity, list) and len(identity) == 2 and identity[0] == revision
            and isinstance(identity[1], str) and SHA40.fullmatch(identity[1])):
        raise ValueError("slice benchmark source identity differs")
    raw_records = (json.loads(line) for line in (root / "raw.jsonl").read_bytes().splitlines())
    raw_identities = [record for record in raw_records if record.get("kind") == "identities"]
    if len(raw_identities) != 1 or raw_identities[0].get("candidate_revision_and_tree") != identity:
        raise ValueError("slice source identity does not match retained original records")
    items = {(s["baseline"], s["candidate"], s["phase"]): s for s in verified["summaries"]}
    rows = []
    for phase in SLICE_PHASES:
        pristine = items["pristine", "entry", phase]
        cargo = items["cargo", "entry", phase]
        ap, cp = pristine["analysis"], cargo["analysis"]
        if not (math.isclose(ap["candidate_median"], cp["candidate_median"], rel_tol=1e-12)
                and math.isclose(ap["candidate_p95"], cp["candidate_p95"], rel_tol=1e-12)):
            raise ValueError("slice packaged entry timing differs across the comparisons")
        rows.append({"operation": phase, "lustr": measured(ap["candidate_median"], ap["candidate_p95"]),
                     "lust": measured(ap["baseline_median"], ap["baseline_p95"]),
                     "rustc": measured(cp["baseline_median"], cp["baseline_p95"]),
                     "lust_over_lustr": paired(pristine), "rustc_over_lustr": paired(cargo)})
    source = {"revision": identity[0], "tree": identity[1],
              "original_raw_sha256": verified["raw_sha256"],
              "pristine_lust_sha256": verified["compiler_and_runner_sha256"][verified["tool_paths"]["pristine"]],
              "input_report_sha256": source_digest(root / "report.json")}
    return population(FAMILIES[2], rows, source, verified["pairs"])


def project_family(root: Path, revision: str) -> dict:
    names = ("README.md", "campaign.json", "host.json", "replay.json", "samples.json")
    if not (root / "SHA256SUMS").is_file():
        raise FileNotFoundError("missing project export checksum manifest")
    hashes = (root / "SHA256SUMS").read_text(encoding="ascii")
    read = {}
    for name in names:
        path = root / name
        if not path.is_file() or path.is_symlink():
            raise ValueError("project public input missing or linked")
        read[name] = path.read_bytes()
    expected = "".join(f"{digest(read[name])}  {name}\n" for name in names)
    if hashes != expected:
        raise ValueError("project public export checksum differs")
    campaign, samples, replay = (json.loads(read[name]) for name in ("campaign.json", "samples.json", "replay.json"))
    if not (campaign.get("schema") == "lustr-coordinator-campaign-v1" and campaign.get("status") == "passed"
            and campaign.get("revision") == revision and campaign.get("scenarios") == list(PROJECT_PHASES)
            and samples.get("schema") == 1 and isinstance(samples.get("cases"), list)
            and campaign.get("pairs") == samples.get("pairs") and type(campaign.get("pairs")) is int
            and campaign["pairs"] >= 8 and campaign.get("warmup_pairs") == samples.get("warmup_pairs")
            and replay.get("verified_comparisons") == len(PROJECT_PHASES)
            and isinstance(replay.get("results"), list) and len(replay["results"]) == len(PROJECT_PHASES)):
        raise ValueError("project campaign/replay identities or coverage differ")
    pairs = campaign["pairs"]
    by_case = {case["name"]: case for case in samples["cases"]}
    replay_by_case = {case["case"]: case for case in replay["results"]}
    if (len(by_case) != len(PROJECT_PHASES) or set(by_case) != set(PROJECT_PHASES)
            or len(replay_by_case) != len(PROJECT_PHASES) or set(replay_by_case) != set(PROJECT_PHASES)):
        raise ValueError("project missing or duplicate scenario")
    rows = []
    for phase in PROJECT_PHASES:
        case, replay_case = by_case[phase], replay_by_case[phase]
        if (case.get("lanes") != ["matched-rust", "lustr-mixed"]
                or not isinstance(case.get("comparisons"), list) or len(case["comparisons"]) != 1):
            raise ValueError("project lane contract differs")
        comparison = case["comparisons"][0]
        if (comparison.get("baseline") != "matched-rust" or comparison.get("candidate") != "lustr-mixed"
                or replay_case.get("baseline") != "matched-rust" or replay_case.get("candidate") != "lustr-mixed"
                or not replay_equal(comparison.get("analysis"), replay_case.get("analysis"))):
            raise ValueError("project replay comparison differs from public samples")
        observed = {}
        for record in case["records"]:
            if record.get("phase") != "timed":
                continue
            key = record.get("lane"), record.get("pair")
            if (key[0] not in case["lanes"] or type(key[1]) is not int or not 0 <= key[1] < pairs
                    or key in observed or type(record.get("exit")) is not int or record["exit"] != 0
                    or record.get("signal") is not None
                    or record.get("timed_out") is not False or record.get("output_limit") is not False):
                raise ValueError("project timing execution contract differs")
            duration = record.get("wall_seconds")
            if type(duration) not in (int, float) or not math.isfinite(duration) or duration <= 0:
                raise ValueError("project sample duration invalid")
            observed[key] = duration
        if set(observed) != {(lane, i) for lane in case["lanes"] for i in range(pairs)}:
            raise ValueError("project missing or duplicated timed samples")
        analysis = comparison["analysis"]
        base = [observed["matched-rust", i] for i in range(pairs)]
        candidate = [observed["lustr-mixed", i] for i in range(pairs)]
        empirical = ((statistics.median(base), sorted(base)[math.ceil(.95 * pairs) - 1], "baseline"),
                     (statistics.median(candidate), sorted(candidate)[math.ceil(.95 * pairs) - 1], "candidate"))
        for median, p95, prefix in empirical:
            if not (math.isclose(analysis[prefix + "_median"], median, rel_tol=1e-9, abs_tol=1e-12)
                    and math.isclose(analysis[prefix + "_p95"], p95, rel_tol=1e-9, abs_tol=1e-12)):
                raise ValueError("project median/p95 differs from raw pairs")
        geometric = math.exp(sum(math.log(a / b) for a, b in zip(base, candidate)) / pairs)
        if not math.isclose(analysis["geometric_ratio"], geometric, rel_tol=1e-9):
            raise ValueError("project geometric ratio differs from raw pairs")
        rows.append({"operation": phase,
                     "lustr": measured(analysis["candidate_median"], analysis["candidate_p95"]),
                     "lust": cell("NT", "no matched pristine Lust project lane"),
                     "rustc": measured(analysis["baseline_median"], analysis["baseline_p95"]),
                     "rustc_over_lustr": paired(comparison)})
    source_trees = {case.get("source_tree_sha256") for case in samples["cases"]}
    if not (len(source_trees) == 1 and all(isinstance(sha, str) and SHA64.fullmatch(sha) for sha in source_trees)):
        raise ValueError("project source trees differ or lack valid hashes")
    source = {"revision": campaign["revision"], "input_report_sha256": digest(read["campaign.json"]),
              "source_tree_sha256": next(iter(source_trees)),
              "public_export_sha256": digest(hashes.encode())}
    return population(FAMILIES[3], rows, source, pairs)


def value(value: dict) -> str:
    return f"{value['median_ms']:.3f}" if value["status"] == "MEASURED" else value["status"]


def render(report: dict) -> str:
    lines = ["# Three-compiler performance dashboard", "",
             f"Measured source revision: `{report['source_revision']}`.", "",
             "All numerical cells are median wall-clock milliseconds, lower is faster. "
             "Measured families retain p95 and paired-ratio statistics in the JSON and original artifacts.",
             "NT means not measured or support not established. N/A is reserved for benchmark-bound executed unsupported-feature evidence and is not currently issued. "
             "FAIL denotes a failed benchmark step or rejected evidence, not necessarily a compiler correctness failure.",
             "Standalone rustc no-op invokes a compiler; Cargo no-op can be Fresh. These are separate workloads.", ""]
    for family in report["families"]:
        lines += [f"## {family['title']}", "", f"Campaign: **{family['status']}**. {family.get('reason', '')}", "",
                  "| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |", "| --- | ---: | ---: | ---: |"]
        for row in family["rows"]:
            lines.append(f"| {row['operation']} | {value(row['lustr'])} | {value(row['lust'])} | {value(row['rustc'])} |")
        lines += ["", f"Rust reference: {family['rustc_mode']}.", ""]
    lines += ["This is a derived presentation of the existing benchmark families, not an independent new timing campaign or release admission.", ""]
    return "\n".join(lines)


def serialized(report: dict) -> bytes:
    return (json.dumps(report, sort_keys=True, indent=2, allow_nan=False) + "\n").encode()


def source_digest(path: Path) -> str:
    return digest(path.read_bytes())


def publish(destination: Path, report: dict) -> None:
    if destination.exists():
        raise ValueError("output directory already exists; preserve prior evidence")
    destination.mkdir(parents=True)
    files = {"report.json": serialized(report), "README.md": render(report).encode()}
    for name, data in files.items():
        (destination / name).write_bytes(data)
    checksums = "".join(f"{digest(data)}  {name}\n" for name, data in sorted(files.items()))
    (destination / "SHA256SUMS").write_text(checksums, encoding="ascii")


def replay(destination: Path) -> None:
    expected = (destination / "SHA256SUMS").read_text(encoding="ascii")
    files = {"report.json": (destination / "report.json").read_bytes(),
             "README.md": (destination / "README.md").read_bytes()}
    actual = "".join(f"{digest(data)}  {name}\n" for name, data in sorted(files.items()))
    if actual != expected:
        raise ValueError("published output hashes differ")
    report = json.loads(files["report.json"])
    if serialized(report) != files["report.json"] or render(report).encode() != files["README.md"]:
        raise ValueError("published output does not reproduce from its report JSON")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--replay", type=Path, help="verify retained output bytes without running compilers")
    parser.add_argument("--source-revision")
    parser.add_argument("--fixed-root", type=Path)
    parser.add_argument("--slice-dir", type=Path)
    parser.add_argument("--project-public", type=Path)
    parser.add_argument("--fixed-outcome", choices=("success", "failure", "skipped", "cancelled"))
    parser.add_argument("--slice-outcome", choices=("success", "failure", "skipped", "cancelled"))
    parser.add_argument("--project-outcome", choices=("success", "failure", "skipped", "cancelled"))
    parser.add_argument("--lust-coverage", type=Path,
                        help="reserved: unsupported until a benchmark-owned executed source/argv gate exists")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)
    try:
        if args.replay is not None:
            if any((args.source_revision, args.fixed_root, args.slice_dir, args.project_public,
                    args.fixed_outcome, args.slice_outcome, args.project_outcome, args.output, args.lust_coverage)):
                raise ValueError("--replay cannot be combined with a new campaign")
            replay(args.replay)
            print("Three-way output replay passed; no compiler execution")
            return 0
        if not (args.source_revision and SHA40.fullmatch(args.source_revision)
                and args.fixed_root and args.slice_dir and args.project_public
                and args.fixed_outcome and args.slice_outcome and args.project_outcome and args.output):
            raise ValueError("source revision, inputs, outcomes and output are required")
        outcomes = (args.fixed_outcome, args.fixed_outcome, args.slice_outcome, args.project_outcome)
        families = [empty_family(family, outcome) for family, outcome in zip(FAMILIES, outcomes)]
        if args.fixed_outcome == "success":
            try:
                families[0], families[1] = fixed_families(args.fixed_root, args.source_revision)
            except (ValueError, KeyError, TypeError, OSError, OverflowError) as exc:
                families[0], families[1] = (empty_family(FAMILIES[0], "success"),
                                            empty_family(FAMILIES[1], "success"))
        if args.slice_outcome == "success":
            try:
                families[2] = slice_family(args.slice_dir, args.source_revision)
            except (ValueError, KeyError, TypeError, OSError, OverflowError) as exc:
                families[2] = empty_family(FAMILIES[2], "success")
        if args.project_outcome == "success":
            try:
                families[3] = project_family(args.project_public, args.source_revision)
            except (ValueError, KeyError, TypeError, OSError, OverflowError) as exc:
                families[3] = empty_family(FAMILIES[3], "success")
        # No current benchmark supplies a source/argv-bound pristine Lust unsupported
        # execution receipt. Even a hash-consistent diagnostic from unrelated code
        # cannot establish that this workload is unsupported. Never emit N/A here.
        coverage_status = "FAIL" if args.lust_coverage is not None else "NT"
        report = {"schema": SCHEMA, "source_revision": args.source_revision,
                  "status": "MEASURED" if all(f["status"] == "MEASURED" for f in families) else "INCOMPLETE",
                  "coverage_status": coverage_status, "families": families}
        if coverage_status == "FAIL":
            report["status"] = "INCOMPLETE"
        publish(args.output, report)
        replay(args.output)
        return 1 if coverage_status == "FAIL" or any(f["status"] == "FAIL" and outcome == "success"
                        for f, outcome in zip(families, outcomes)) else 0
    except (ValueError, OSError, KeyError, TypeError, json.JSONDecodeError) as exc:
        print(f"three-way report rejected: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
