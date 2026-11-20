#!/usr/bin/env python3
"""Assess measured full-request costs against a fixed workload plan.

This checks evidence integrity and the performance dimension only. It does not
execute a compiler, establish that supplied timings are truthful, prove semantic
equivalence, or grant release admission. Historical assessors remain unchanged.
"""
from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import math
from pathlib import Path
import re
import sys


PLAN_SCHEMA = "lustr-current-performance-plan-v1"
RECEIPT_SCHEMA = "lustr-current-performance-execution-v1"
REPORT_SCHEMA = "lustr-current-performance-assessment-v2"
BOUNDARY = "public-command-start-to-completion"
ORDER = "rotated-cases-balanced-engine-permutations-v1"
RESAMPLES = 10000
MASK = (1 << 64) - 1
SHA40 = re.compile(r"[0-9a-f]{40}\Z")
SHA64 = re.compile(r"[0-9a-f]{64}\Z")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def serialized(value):
    return (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + "\n").encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def positive(value):
    return type(value) in (int, float) and math.isfinite(value) and value > 0


def exact_json(data):
    def object_pairs(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, f"duplicate JSON key: {key}")
            result[key] = value
        return result

    def finite_float(value):
        result = float(value)
        require(math.isfinite(result), f"nonfinite JSON value: {value}")
        return result

    return json.loads(data, object_pairs_hook=object_pairs, parse_float=finite_float,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(f"nonfinite JSON value: {value}")))


class Evidence:
    def __init__(self, root):
        self.root = root.resolve(strict=True)
        self.files = {}

    def read(self, ref):
        require(isinstance(ref, dict) and set(ref) == {"path", "sha256"}, "file reference needs path and sha256")
        name, expected = ref["path"], ref["sha256"]
        require(isinstance(name, str) and name and isinstance(expected, str) and SHA64.fullmatch(expected),
                "invalid evidence file reference")
        relative = Path(name)
        require(not relative.is_absolute() and ".." not in relative.parts, "evidence path must stay inside evidence root")
        path = (self.root / relative).resolve(strict=True)
        require(path.is_relative_to(self.root) and path.is_file(), "evidence path escapes root or is not a file")
        data = path.read_bytes()
        require(digest(data) == expected, f"evidence digest mismatch: {name}")
        require(name not in self.files or self.files[name] == expected, f"evidence identity changed: {name}")
        self.files[name] = expected
        return data

    def unchanged(self):
        for name, expected in self.files.copy().items():
            self.read({"path": name, "sha256": expected})


def engine_order(case, pair):
    engines = ["rustc", "lustr"] + (["lust"] if case["class"] == "supported" else [])
    orders = list(itertools.permutations(engines))
    return orders[pair % len(orders)]


def sequence(plan):
    cases = plan["cases"]
    for pair in range(plan["pairs"]):
        for offset in range(len(cases)):
            case = cases[(pair + offset) % len(cases)]
            for engine in engine_order(case, pair):
                yield pair, case, engine


def validate_plan(plan, evidence):
    require(plan["schema"] == PLAN_SCHEMA, "unsupported plan schema")
    require(type(plan["pairs"]) is int and 24 <= plan["pairs"] <= 4096, "plan requires 24 to 4096 matched pair blocks")
    require(type(plan["seed"]) is int and 0 <= plan["seed"] <= MASK, "plan seed must be an unsigned 64-bit integer")
    require(plan["sampling_order"] == ORDER and plan["timing_boundary"] == BOUNDARY,
            "only complete public-request timings and the declared counterbalanced order are assessed")
    require(isinstance(plan["scope"], str) and plan["scope"].strip(), "a concrete qualified workload scope is required")
    require(isinstance(plan["cases"], list) and plan["cases"], "predeclared workload cases required")
    identities = plan["compilers"]
    require(isinstance(identities, dict) and {"lustr", "rustc"} <= identities.keys()
            and identities.keys() <= {"lustr", "rustc", "lust"}, "unexpected compiler identities")
    for name, identity in identities.items():
        require(SHA40.fullmatch(identity["revision"]) and SHA40.fullmatch(identity["tree"]), f"{name}: exact source identity required")
        require(identity["entry_kind"] == "direct-compiler", f"{name}: this assessment version requires a direct compiler entry")
        artifacts = identity["artifacts"]
        require(isinstance(artifacts, dict) and artifacts and "entry" in artifacts,
                f"{name}: entry and every required compiler/library/helper artifact must be declared")
        for role, ref in artifacts.items():
            require(isinstance(role, str) and role, "compiler artifact roles must be named")
            evidence.read(ref)
    ids = set()
    request_digests = {}
    for case in plan["cases"]:
        name = case["id"]
        require(isinstance(name, str) and name and name not in ids, "case IDs must be nonempty and unique")
        ids.add(name)
        require(case["class"] in {"supported", "complementary", "mixed"}, f"{name}: unknown declared ownership class")
        require(type(case["frequency"]) is int and 0 <= case["frequency"] <= 1_000_000_000,
                f"{name}: frequency must be a nonnegative integer")
        require(type(case["expected_exit"]) is int and 0 <= case["expected_exit"] <= 255,
                f"{name}: expected exit must be an ordinary exit status")
        engines = set(engine_order(case, 0))
        require(set(case["commands"]) == engines and engines <= identities.keys(), f"{name}: missing or extra engine command")
        for engine, argv in case["commands"].items():
            require(isinstance(argv, list) and argv and all(isinstance(arg, str) and "\0" not in arg for arg in argv),
                    f"{name}: each command needs complete literal argv")
            require(argv[0] == "./" + identities[engine]["artifacts"]["entry"]["path"],
                    f"{name}: command does not execute the declared compiler entry from the evidence root")
        request = exact_json(evidence.read(case["request"]))
        for field in ("operation", "target", "profile", "lifecycle"):
            require(isinstance(request[field], str) and request[field], f"{name}: request must define {field}")
        require(isinstance(request["flags"], dict), f"{name}: semantic/profile flags must be declared")
        require(isinstance(request["source_inputs"], list) and request["source_inputs"], f"{name}: source input manifest is empty")
        for source in request["source_inputs"]:
            evidence.read(source)
        require(isinstance(request["artifacts"], list) and all(isinstance(role, str) and role for role in request["artifacts"])
                and len(set(request["artifacts"])) == len(request["artifacts"]), f"{name}: artifact roles must be unique")
        require(case["expected_exit"] != 0 or request["operation"] not in {"build", "link", "compile-plus-run", "compile-plus-test"}
                or request["artifacts"], f"{name}: a successful build must retain its requested artifact evidence")
        request_digests[name] = (case["request"]["sha256"], set(request["artifacts"]), request)
    require(any(case["frequency"] for case in plan["cases"]), "the measured workload needs a positive frequency")
    cycle = 6 if any(case["class"] == "supported" for case in plan["cases"]) else 2
    require(plan["pairs"] % cycle == 0, "pair count must complete every engine-order cycle")
    references = plan["qualification_records"]
    require(isinstance(references, list), "qualification record references must be a list")
    qualified = []
    candidate = identities["lustr"]
    for ref in references:
        record = exact_json(evidence.read(ref))
        require(record["candidate_revision"] == candidate["revision"] and record["candidate_tree"] == candidate["tree"]
                and record["compiler_identity_sha256"] == digest(serialized(candidate)),
                "qualification reference belongs to a different source or artifact inventory")
        qualified.append(ref)
    return request_digests, qualified


def read_samples(data, plan, plan_sha256, evidence, requests):
    rows = [exact_json(line) for line in data.splitlines() if line.strip()]
    expected = list(sequence(plan))
    require(len(rows) == len(expected), "missing or extra measurements for the fixed workload")
    values = {}
    receipts = set()
    for row, (pair, case, engine) in zip(rows, expected):
        key = (pair, case["id"], engine)
        require(set(row) == {"pair", "case", "engine", "receipt"} and type(row["pair"]) is int
                and (row["pair"], row["case"], row["engine"]) == key,
                "measurement order, pair identity, or workload case differs from the fixed plan")
        require(row["receipt"]["path"] not in receipts, "an execution receipt was reused for multiple observations")
        receipts.add(row["receipt"]["path"])
        record = exact_json(evidence.read(row["receipt"]))
        require(record["schema"] == RECEIPT_SCHEMA and record["plan_sha256"] == plan_sha256, "execution receipt uses a different plan")
        require(type(record["pair"]) is int and (record["pair"], record["case"], record["engine"]) == key,
                "execution receipt identity differs from sample")
        request_digest, artifacts, _ = requests[case["id"]]
        require(record["request_sha256"] == request_digest and record["argv"] == case["commands"][engine],
                "execution changed the requested operation or literal command")
        require(record["compiler_identity_sha256"] == digest(serialized(plan["compilers"][engine])),
                "execution compiler/source/artifact identity differs")
        require(record["working_directory"] == "." and record["execution_entry"] == plan["compilers"][engine]["artifacts"]["entry"],
                "captured execution entry or working directory differs from the direct compiler declaration")
        evidence.read(record["execution_entry"])
        require(record["timing_boundary"] == BOUNDARY, "subphase timings cannot replace complete public-request costs")
        require(positive(record["seconds"]), "duration must be finite and positive")
        require(type(record["exit_status"]) is int and record["exit_status"] == case["expected_exit"] and record["signal"] is None,
                "failed or interrupted requests cannot supply successful timing evidence")
        for stream in ("stdout", "stderr"):
            evidence.read(record[stream])
        require(isinstance(record["artifacts"], dict) and set(record["artifacts"]) == artifacts,
                "measured request did not retain exactly its required artifact roles")
        for ref in record["artifacts"].values():
            evidence.read(ref)
        values[key] = record["seconds"]
    return values


def random64(state):
    # Same fixed SplitMix64 primitive used by the frozen paired-statistics tool.
    state = (state + 0x9E3779B97F4A7C15) & MASK
    value = state
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & MASK
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & MASK
    return state, value ^ (value >> 31)


def bootstrap_counts(n, seed):
    state, threshold, result = seed, ((1 << 64) - n) % n, []
    for _ in range(RESAMPLES):
        counts = [0] * n
        for _ in range(n):
            while True:
                state, value = random64(state)
                if value >= threshold:
                    break
            counts[value % n] += 1
        result.append(counts)
    return result


def ratio(reference, candidate, counts):
    require(all(positive(value) for value in reference + candidate), "weighted durations must be finite and positive")
    point = math.fsum(reference) / math.fsum(candidate)
    boots = sorted(math.fsum(n * value for n, value in zip(draw, reference))
                   / math.fsum(n * value for n, value in zip(draw, candidate)) for draw in counts)
    require(positive(point) and all(positive(value) for value in boots), "invalid ratio after summing elapsed costs")
    return {"statistic": "ratio-of-paired-total-sums", "reference_over_lustr": point,
            "bootstrap_95": [boots[math.floor(.025 * (RESAMPLES - 1))], boots[math.ceil(.975 * (RESAMPLES - 1))]]}


def paired_log_ratio(reference, candidate, counts):
    """Retain the earlier statistic for diagnosis, never target admission."""
    logs = [math.log(left) - math.log(right) for left, right in zip(reference, candidate)]
    point = math.exp(math.fsum(logs) / len(logs))
    boots = sorted(math.exp(math.fsum(n * value for n, value in zip(draw, logs)) / len(logs)) for draw in counts)
    require(positive(point) and all(positive(value) for value in boots), "invalid paired-log ratio")
    return {"statistic": "geometric-mean-of-within-pair-ratios", "reference_over_lustr": point,
            "bootstrap_95": [boots[math.floor(.025 * (RESAMPLES - 1))], boots[math.ceil(.975 * (RESAMPLES - 1))]]}


def distribution(values):
    ordered = sorted(values)
    n = len(ordered)
    return {"mean_seconds": math.fsum(values) / n,
            "median_seconds": (ordered[(n - 1) // 2] + ordered[n // 2]) / 2,
            "p95_seconds": ordered[math.ceil(.95 * n) - 1], "maximum_seconds": ordered[-1]}


def assess(plan_file, samples_file, evidence_root):
    plan_bytes, sample_bytes = plan_file.read_bytes(), samples_file.read_bytes()
    plan, evidence = exact_json(plan_bytes), Evidence(evidence_root)
    requests, qualification = validate_plan(plan, evidence)
    values = read_samples(sample_bytes, plan, digest(plan_bytes), evidence, requests)
    n, counts = plan["pairs"], bootstrap_counts(plan["pairs"], plan["seed"])
    totals = {engine: [math.fsum(case["frequency"] * values[pair, case["id"], engine] for case in plan["cases"])
                       for pair in range(n)] for engine in ("rustc", "lustr")}
    overall = ratio(totals["rustc"], totals["lustr"], counts)
    overall["status"] = "faster" if overall["bootstrap_95"][0] > 1 else "slower" if overall["bootstrap_95"][1] < 1 else "inconclusive"
    overall["performance_target_met"] = overall["status"] == "faster"
    overall["paired_workload_totals_seconds"] = totals
    overall["distributions"] = {engine: distribution(series) for engine, series in totals.items()}
    cases = []
    for case in plan["cases"]:
        request = requests[case["id"]][2]
        series = {engine: [values[pair, case["id"], engine] for pair in range(n)] for engine in engine_order(case, 0)}
        row = {"id": case["id"], "declared_class": case["class"], "frequency": case["frequency"],
               "rustc_comparison": ratio(series["rustc"], series["lustr"], counts),
               "distributions": {engine: distribution(times) for engine, times in series.items()}}
        warm_supported = (case["class"] == "supported" and request["operation"] in {"check", "build"}
                          and request["lifecycle"] == "warm-no-op" and case["expected_exit"] == 0)
        ordinary_complement = case["class"] == "complementary" and request["operation"] in {"check", "build"} and case["expected_exit"] == 0
        if warm_supported or ordinary_complement:
            reference, allowance = ("lust", .10) if case["class"] == "supported" else ("rustc", .05)
            comparison = ratio(series[reference], series["lustr"], counts)
            low, high = comparison["bootstrap_95"]
            threshold = 1 / (1 + allowance)
            relative_status = "met" if low >= threshold else "missed" if high < threshold else "inconclusive"
            row["near_stock_target"] = {"reference": reference, "relative_allowance": allowance, **comparison,
                "status": relative_status, "relative_budget_status": relative_status,
                "geometric_ratio_diagnostic": paired_log_ratio(series[reference], series["lustr"], counts),
                "controls_overall_performance_decision": False}
            if warm_supported:
                rust = row["rustc_comparison"]
                rust_win = rust["bootstrap_95"][0] > 1
                row["near_stock_target"].update(rustc_speedup_established=rust_win, rustc_comparison=rust,
                    rustc_geometric_ratio_diagnostic=paired_log_ratio(series["rustc"], series["lustr"], counts))
                if not rust_win and relative_status != "missed":
                    row["near_stock_target"]["status"] = "missed" if rust["bootstrap_95"][1] < 1 else "inconclusive"
        cases.append(row)
    evidence.unchanged()
    require(plan_bytes == plan_file.read_bytes() and sample_bytes == samples_file.read_bytes(), "plan or samples changed during assessment")
    return {"schema": REPORT_SCHEMA, "scope": plan["scope"], "release_admission": False,
        "full_replacement_qualification": "not-established-by-this-assessment", "overall": overall, "cases": cases,
        "declared_classes": sorted({case["class"] for case in plan["cases"]}),
        "unmeasured_classes": sorted({"supported", "complementary", "mixed"} - {case["class"] for case in plan["cases"]}),
        "compiler_identities": plan["compilers"], "supplied_qualification_records": qualification,
        "input_sha256": {"plan": digest(plan_bytes), "samples": digest(sample_bytes)},
        "evidence_sha256": evidence.files, "assessor_sha256": digest(Path(__file__).read_bytes()),
        "statistics": {"method": "ratio-of-paired-workload-total-sums/splitmix64/rejection-index/percentile-v1",
                       "pairs": n, "bootstrap_resamples": RESAMPLES, "seed": plan["seed"],
                       "p95": "nearest-rank empirical p95, not a tail confidence bound"},
        "limits": [
            "Hash binding checks supplied evidence integrity. It cannot prove that the plan predates measurement or that execution records and durations are truthful. Inspect independent capture and qualification records.",
            "Every required entry, compiler library, helper and runtime must be in the declared identity inventory. This assessor hashes declared files; it does not discover an omitted execution dependency.",
            "This version binds direct compiler execution from the evidence root. It does not verify arbitrary Cargo or wrapper invocation chains; such operations need a separately defined and qualified capture contract.",
            "Shared request contracts and argv bind the declared work. This assessor does not prove semantic equivalence, generated-program behavior, full Rust coverage, exclusive Lust ownership or selective-import minimality.",
            "The point estimate and every bootstrap replicate divide summed reference elapsed costs by summed candidate elapsed costs using fixed integer frequencies. No geometric mean of per-case speedups decides the result.",
            "Bootstrap resampling keeps complete pair blocks and all workload cases together. Intervals remain conditional on the measured host, fixed workload and an appropriate independent block sampling design.",
            "Per-case near-stock claims and their rustc speedup condition use paired total-time ratios. Geometric ratios are diagnostic only because they can conceal increased elapsed cost within one case. The 10% supported target applies only to warm-no-op check/build; the 5% complementary target applies to successful check/build. Other modes have no invented numeric target.",
            "Per-case near-stock targets and empirical tails remain visible. A target miss cannot be relabeled near-stock, and is not by itself a failure of the overall speed criterion.",
            "Empirical p95 can omit fewer than 5% of observations. Means and maxima are reported too; none of these observations is a tail confidence guarantee.",
            "Zero-frequency cases are diagnostic only. Their costs do not enter the workload total; their results remain reported.",
            "A performance pass is not release authorization. Full correctness, ownership, source-import and actual package qualification remain separate obligations."]}


def render(report):
    overall = report["overall"]
    low, high = overall["bootstrap_95"]
    lines = ["# Current performance assessment", "", report["scope"], "",
             f"Overall measured speed target: **{overall['status']}**. Rustc / Lustr total-time ratio: {overall['reference_over_lustr']:.6f}, paired 95% interval [{low:.6f}, {high:.6f}].",
             "", "Release admission: **false**. This result assesses performance only.", "",
             "| Case | Declared class | Frequency | Rustc / Lustr | Lustr mean, ms | Lustr p95, ms | Lustr maximum, ms | Near-stock target |",
             "| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |"]
    for case in report["cases"]:
        near = case.get("near_stock_target", {}).get("status", "not a separate near-stock comparison")
        name = case["id"].replace("|", "\\|").replace("\n", " ")
        times = case["distributions"]["lustr"]
        lines.append(f"| {name} | {case['declared_class']} | {case['frequency']} | {case['rustc_comparison']['reference_over_lustr']:.6f} | {times['mean_seconds'] * 1000:.6f} | {times['p95_seconds'] * 1000:.6f} | {times['maximum_seconds'] * 1000:.6f} | {near} |")
    comparisons = [case for case in report["cases"] if "near_stock_target" in case]
    if comparisons:
        lines += ["", "## Near-stock total-time comparisons", "",
                  "Geometric ratios remain in the JSON for diagnosis and cannot establish these targets.", "",
                  "| Case | Reference | Reference / Lustr total time | Paired 95% interval | Near-stock target |",
                  "| --- | --- | ---: | --- | --- |"]
        for case in comparisons:
            near = case["near_stock_target"]
            low, high = near["bootstrap_95"]
            name = case["id"].replace("|", "\\|").replace("\n", " ")
            lines.append(f"| {name} | {near['reference']} | {near['reference_over_lustr']:.6f} | [{low:.6f}, {high:.6f}] | {near['status']} |")
    lines += ["", "## Limits", ""] + ["- " + value for value in report["limits"]]
    return "\n".join(lines) + "\n"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--samples", type=Path, required=True)
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        output, evidence_root = args.output.resolve(), args.evidence_root.resolve()
        require(not output.exists() and not output.is_relative_to(evidence_root) and not evidence_root.is_relative_to(output),
                "use a new assessment directory separate from evidence")
        report = assess(args.plan, args.samples, evidence_root)
        output.mkdir(parents=True)
        files = {"assessment.json": serialized(report), "README.md": render(report).encode()}
        for name, data in files.items():
            (output / name).write_bytes(data)
        (output / "SHA256SUMS").write_text("".join(f"{digest(data)}  {name}\n" for name, data in sorted(files.items())))
        print(render(report))
        return 0 if report["overall"]["performance_target_met"] else 2
    except (OSError, ValueError, KeyError, TypeError, AssertionError, OverflowError) as error:
        print(f"Current performance assessment rejected: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
