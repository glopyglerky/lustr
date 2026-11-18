#!/usr/bin/env python3
"""Paired actual-selector component timing; never a full compiler benchmark."""
import argparse
import hashlib
import json
import math
import random
import statistics
import subprocess
import time
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--before", required=True, type=Path)
    parser.add_argument("--after", required=True, type=Path)
    parser.add_argument("--package", required=True, type=Path)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--pairs", type=int, default=24)
    parser.add_argument("--excluded-pairs", type=int, default=8)
    args = parser.parse_args()
    assert args.pairs > 0 and args.pairs % 2 == 0
    assert args.excluded_pairs >= 0 and args.excluded_pairs % 2 == 0
    root = args.output.resolve()
    root.mkdir(mode=0o700)
    streams = root / "streams"
    streams.mkdir()
    for name in ("home", "cargo-home", "target", "intermediates", "tmp", "before", "after"):
        (root / name).mkdir(mode=0o700)
    binaries = {"before":args.before.resolve(), "after":args.after.resolve()}
    hashes = {label:digest(binary) for label, binary in binaries.items()}
    build_receipts = {label:json.loads((binary.parents[2] / "receipt.json").read_text()) for label, binary in binaries.items()}
    for label, receipt in build_receipts.items():
        assert receipt["status"] == "completed" and receipt["binary_sha256"] == hashes[label]
        (root / f"{label}-build-receipt.json").write_bytes((binaries[label].parents[2] / "receipt.json").read_bytes())
    receipt = args.package / "lib/lustr-selection-tools.txt"
    (root / "selection-tools.txt").write_bytes(receipt.read_bytes())
    environment = {
        "PATH":str(args.package / "bin") + ":/usr/local/bin:/usr/bin:/bin",
        "HOME":str(root / "home"), "LANG":"C", "LC_ALL":"C", "TZ":"UTC",
        "CARGO_HOME":str(root / "cargo-home"), "CARGO_TARGET_DIR":str(root / "target"),
        "CARGO_BUILD_BUILD_DIR":str(root / "intermediates"), "CARGO_NET_OFFLINE":"true",
        "CARGO_TERM_COLOR":"never", "TMPDIR":str(root / "tmp"),
        "LUSTR_SELECTION_DIAGNOSTICS":str(root / "diagnostics.jsonl"),
    }
    arguments = ["selection", str(args.package / "bin"), "check", str(args.fixture / "src/slice_main.rs"),
                 "--manifest-path", str(args.fixture / "Cargo.toml"), "--offline", "--locked"]
    fixture_hashes = {str(path.relative_to(args.fixture)):digest(path)
                      for path in sorted(args.fixture.rglob("*")) if path.is_file()}
    records = []
    reference = None
    report = {"schema":"selection-path-component-v1",
              "scope":"production selector component with hit diagnostics; no native compilation, daemon or full public command timing claim",
              "revisions":{label:receipt["source"]["revision"] for label, receipt in build_receipts.items()},
              "source_trees":{label:receipt["source"]["tree"] for label, receipt in build_receipts.items()},
              "helper_sha256":hashes, "arguments":arguments, "cwd":str(args.fixture),
              "runner_sha256":digest(Path(__file__)), "package_receipt_sha256":digest(receipt),
              "environment":environment, "fixture_sha256":fixture_hashes,
              "schedule":{"initial_primes_per_lane":3,"excluded_pairs":args.excluded_pairs,"measured_pairs":args.pairs,"seed":73491},
              "records":records, "status":"running", "error":None}

    def save():
        (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    def invoke(label, phase, pair):
        nonlocal reference
        env = dict(environment, LUST_CACHE_DIR=str(root / label))
        argv = [str(binaries[label]), *arguments]
        index = len(records)
        started = time.perf_counter_ns()
        timeout = None
        try:
            process = subprocess.run(argv, cwd=args.fixture, env=env, capture_output=True, timeout=30)
        except subprocess.TimeoutExpired as error:
            timeout = str(error)
            process = subprocess.CompletedProcess(argv, None, error.stdout or b"", error.stderr or b"")
        elapsed = time.perf_counter_ns() - started
        stdout, stderr = streams / f"{index:03d}.stdout", streams / f"{index:03d}.stderr"
        stdout.write_bytes(process.stdout)
        stderr.write_bytes(process.stderr)
        record = {"index":index,"label":label,"phase":phase,"pair":pair,"excluded":phase != "measured",
                  "exit":process.returncode,"timeout":timeout,"wall_ns":elapsed,"argv":argv,"environment":env,
                  "stdout":str(stdout.relative_to(root)),"stdout_sha256":digest(stdout),
                  "stderr":str(stderr.relative_to(root)),"stderr_sha256":digest(stderr)}
        records.append(record)
        save()
        assert timeout is None, record
        assert process.returncode == 0, record
        data = json.loads(process.stdout)
        record["selection_ns"] = data.pop("elapsed_ns")
        assert data["success"], data
        if reference is None:
            reference = data
        assert data == reference, (data, reference)
        events = (root / "diagnostics.jsonl").read_text().splitlines()
        assert len(events) == len(records), events
        record["cache"] = json.loads(events[-1])["selection_cache"]
        if phase != "prime":
            assert record["cache"] == "hit", record
            assert not process.stderr, process.stderr
        save()
        return record

    try:
        for label in ("before", "after"):
            for ordinal in range(3):
                invoke(label, "prime", ordinal)
        rng = random.Random(73491)
        for phase, pairs in (("warmup", args.excluded_pairs), ("measured", args.pairs)):
            orders = [("before", "after"), ("after", "before")] * (pairs // 2)
            rng.shuffle(orders)
            for pair, order in enumerate(orders):
                for label in order:
                    invoke(label, phase, pair)
        measured = [row for row in records if row["phase"] == "measured"]
        paired = [{row["label"]:row["selection_ns"] for row in measured if row["pair"] == pair} for pair in range(args.pairs)]
        logs = [math.log(row["before"] / row["after"]) for row in paired]
        resampling = random.Random(91573)
        bootstrap = sorted(math.exp(statistics.mean(resampling.choices(logs, k=len(logs)))) for _ in range(10000))
        report["summary"] = {label:{"median_ns":statistics.median(row["selection_ns"] for row in measured if row["label"] == label)}
                             for label in ("before", "after")}
        report["summary"].update(paired_geometric_ratio=math.exp(statistics.mean(logs)), bootstrap_95=[bootstrap[250],bootstrap[9749]])
        assert hashes == {label:digest(binary) for label, binary in binaries.items()}
        assert report["package_receipt_sha256"] == digest(receipt)
        assert fixture_hashes == {str(path.relative_to(args.fixture)):digest(path)
                                  for path in sorted(args.fixture.rglob("*")) if path.is_file()}
        report["status"] = "completed"
        save()
        print(json.dumps(report["summary"], indent=2))
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save()
        raise


if __name__ == "__main__":
    main()
