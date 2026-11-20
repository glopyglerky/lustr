#!/usr/bin/env python3
"""Black-box tests for the three-compiler evidence dashboard CLI."""

import json
import hashlib
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("report-three-way-performance.py")
REVISION = "e" * 40


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, SCRIPT.with_name(path))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ThreeWayReportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def run_report(self, *more, statuses=("skipped", "skipped", "skipped"), expect=0):
        destination = self.root / "dashboard"
        argv = [sys.executable, str(SCRIPT), "--source-revision", REVISION,
                "--fixed-root", str(self.root / "fixed"),
                "--slice-dir", str(self.root / "slice"),
                "--project-public", str(self.root / "project"),
                "--fixed-outcome", statuses[0], "--slice-outcome", statuses[1],
                "--project-outcome", statuses[2], "--output", str(destination), *more]
        result = subprocess.run(argv, text=True, capture_output=True, check=False)
        self.assertEqual(result.returncode, expect, result.stderr)
        return destination, result

    def prepare_fixed(self):
        synthetic = load_module("synthetic_fixed", "test-mixed-performance-report.py")
        source = self.root / "fixed"
        source.mkdir()
        report = synthetic.fixture(source, schema=2)
        raw = b"".join((json.dumps(row, separators=(",", ":")) + "\n").encode()
                       for row in report["records"])
        report["raw_sha256"] = hashlib.sha256(raw).hexdigest()
        (source / "lustr-paired-measurements/raw.jsonl").write_bytes(raw)
        (source / "lustr-paired-measurements/report.json").write_text(json.dumps(report))

    def prepare_slice(self):
        synthetic = load_module("synthetic_slice", "test-upstream-slice-benchmark.py")
        source = self.root / "slice"
        source.mkdir()
        report = synthetic.synthetic_campaign(source)
        report["candidate_revision_and_tree"] = [REVISION, "b" * 40]
        pristine = report["tool_paths"]["pristine"]
        report["compiler_and_runner_sha256"][pristine] = "c" * 64
        rows = [json.loads(line) for line in (source / "raw.jsonl").read_text().splitlines()]
        identity = next(row for row in rows if row.get("kind") == "identities")
        identity["compiler_and_runner_sha256"][pristine] = "c" * 64
        identity["candidate_revision_and_tree"] = [REVISION, "b" * 40]
        raw = b"".join((json.dumps(row) + "\n").encode() for row in rows)
        (source / "raw.jsonl").write_bytes(raw)
        report["raw_sha256"] = hashlib.sha256(raw).hexdigest()
        synthetic.bench.write_json(source / "report.json", report)
        (source / "README.md").write_text(synthetic.bench.render(report))

    def prepare_project(self):
        source = self.root / "project"
        source.mkdir()
        names = ("clean-dependency-build", "primed-root-build", "primed-root-check", "no-op", "compile-plus-test")
        analysis = {"method": "paired-log-v1/splitmix64/rejection-index/percentile", "pairs": 12,
                    "baseline_median": .1, "candidate_median": .2,
                    "baseline_p95": .1, "candidate_p95": .2,
                    "geometric_ratio": .5, "bootstrap_95": [.5, .5]}
        cases, results = [], []
        for name in names:
            comparison = {"baseline": "matched-rust", "candidate": "lustr-mixed", "analysis": analysis}
            records = [{"phase": "timed", "lane": lane, "pair": i, "wall_seconds": t,
                        "exit": 0, "timed_out": False, "signal": None, "output_limit": False}
                       for i in range(12) for lane, t in (("matched-rust", .1), ("lustr-mixed", .2))]
            cases.append({"name": name, "lanes": ["matched-rust", "lustr-mixed"],
                          "source_tree_sha256": "c" * 64, "records": records,
                          "comparisons": [comparison]})
            results.append({"case": name, **comparison})
        files = {"campaign.json": {"schema": "lustr-coordinator-campaign-v1", "status": "passed",
                                    "revision": REVISION, "pairs": 12, "warmup_pairs": 2,
                                    "scenarios": list(names)},
                 "samples.json": {"schema": 1, "pairs": 12, "warmup_pairs": 2, "cases": cases},
                 "replay.json": {"verified_comparisons": 5, "results": results},
                 "host.json": {}, "README.md": "Synthetic project source, not a measured compiler.\n"}
        for name, content in files.items():
            value = content if isinstance(content, str) else json.dumps(content, sort_keys=True)
            (source / name).write_text(value)
        (source / "SHA256SUMS").write_text("".join(hashlib.sha256((source / name).read_bytes()).hexdigest()
                                               + "  " + name + "\n" for name in sorted(files)))

    def test_skipped_campaigns_have_all_rows_as_nt_and_no_na(self):
        destination, _ = self.run_report()
        data = json.loads((destination / "report.json").read_text())
        self.assertEqual(data["schema"], "lustr-three-way-performance-v1")
        self.assertEqual(data["source_revision"], REVISION)
        self.assertEqual([len(f["rows"]) for f in data["families"]], [9, 9, 6, 5])
        self.assertTrue(all(f["status"] == "NT" for f in data["families"]))
        self.assertTrue(all(cell["status"] == "NT" for family in data["families"]
                            for row in family["rows"] for cell in (row["lustr"], row["lust"], row["rustc"])))
        markdown = (destination / "README.md").read_text()
        self.assertIn("| Operation | Lustr (ms) | Lust (ms) | rustc (ms) |", markdown)
        self.assertNotIn("N/A", markdown.split("## Fixed Cargo")[1].split("## ")[0])
        self.assertTrue((destination / "SHA256SUMS").is_file())

    def test_failed_campaign_is_fail_not_na_or_zero(self):
        destination, _ = self.run_report(statuses=("failure", "skipped", "skipped"))
        families = json.loads((destination / "report.json").read_text())["families"]
        self.assertEqual([f["status"] for f in families], ["FAIL", "FAIL", "NT", "NT"])
        self.assertEqual(families[0]["rows"][0]["lustr"]["status"], "FAIL")
        self.assertEqual(families[0]["rows"][0]["lust"]["status"], "NT")

    def test_success_without_source_bound_artifacts_is_fail(self):
        destination, _ = self.run_report(statuses=("success", "skipped", "skipped"), expect=1)
        families = json.loads((destination / "report.json").read_text())["families"]
        self.assertEqual([f["status"] for f in families], ["FAIL", "FAIL", "NT", "NT"])
        self.assertIn("missing", families[0]["reason"].lower())

    def test_replay_detects_missing_or_changed_published_file(self):
        destination, _ = self.run_report()
        command = [sys.executable, str(SCRIPT), "--replay", str(destination)]
        ok = subprocess.run(command, capture_output=True, text=True, check=False)
        self.assertEqual(ok.returncode, 0, ok.stderr)
        (destination / "README.md").write_text("corrupted\n")
        rejected = subprocess.run(command, capture_output=True, text=True, check=False)
        self.assertNotEqual(rejected.returncode, 0)

    def test_fixed_campaign_populates_all_native_lanes_and_cargo_rust(self):
        self.prepare_fixed()
        destination, _ = self.run_report(statuses=("success", "skipped", "skipped"))
        families = json.loads((destination / "report.json").read_text())["families"]
        self.assertEqual([f["status"] for f in families], ["MEASURED", "MEASURED", "NT", "NT"])
        cargo_body = next(r for r in families[0]["rows"] if r["operation"] == "body-edit")
        native_body = next(r for r in families[1]["rows"] if r["operation"] == "body-edit")
        self.assertEqual(cargo_body["lust"]["status"], "NT")
        self.assertEqual(cargo_body["lustr"]["median_ms"], 130.0)
        self.assertEqual(cargo_body["rustc"]["median_ms"], 65.0)
        self.assertEqual(native_body["lustr"]["median_ms"], 195.0)
        self.assertEqual(native_body["lust"]["median_ms"], 65.0)
        self.assertEqual(native_body["rustc"]["median_ms"], 260.0)
        self.assertEqual(cargo_body["rustc_over_lustr"]["direction"],
                         "reference time / Lustr time; greater than one favors Lustr")

    def test_fixed_campaign_with_changed_statistics_is_fail(self):
        self.prepare_fixed()
        report_path = self.root / "fixed/lustr-paired-measurements/report.json"
        report = json.loads(report_path.read_text())
        report["summaries"][0]["analysis"]["candidate_median"] *= 100
        report_path.write_text(json.dumps(report))
        destination, _ = self.run_report(statuses=("success", "skipped", "skipped"), expect=1)
        families = json.loads((destination / "report.json").read_text())["families"]
        self.assertEqual([f["status"] for f in families[:2]], ["FAIL", "FAIL"])

    def test_wrong_source_checkout_is_fail(self):
        self.prepare_fixed()
        destination, _ = self.run_report("--source-revision", "f" * 40,
                                         statuses=("success", "skipped", "skipped"), expect=1)
        self.assertEqual(json.loads((destination / "report.json").read_text())["families"][1]["status"], "FAIL")

    def test_public_slice_populates_three_lanes(self):
        self.prepare_slice()
        destination, _ = self.run_report(statuses=("skipped", "success", "skipped"))
        family = json.loads((destination / "report.json").read_text())["families"][2]
        self.assertEqual(family["status"], "MEASURED")
        self.assertEqual(len(family["rows"]), 6)
        self.assertTrue(all(row[lane]["status"] == "MEASURED" for row in family["rows"]
                            for lane in ("lustr", "lust", "rustc")))

    def test_tampered_public_slice_is_fail(self):
        self.prepare_slice()
        raw = self.root / "slice/raw.jsonl"
        raw.write_bytes(raw.read_bytes() + b"malicious extra record\n")
        destination, _ = self.run_report(statuses=("skipped", "success", "skipped"), expect=1)
        self.assertEqual(json.loads((destination / "report.json").read_text())["families"][2]["status"], "FAIL")

    def test_unchanged_project_populates_two_measured_lanes_and_lust_nt(self):
        self.prepare_project()
        destination, _ = self.run_report(statuses=("skipped", "skipped", "success"))
        family = json.loads((destination / "report.json").read_text())["families"][3]
        self.assertEqual(family["status"], "MEASURED")
        self.assertEqual(family["rows"][0]["lustr"]["median_ms"], 200.0)
        self.assertEqual(family["rows"][0]["rustc"]["median_ms"], 100.0)
        self.assertEqual(family["rows"][0]["lust"]["status"], "NT")

    def test_modified_project_export_is_fail_not_a_speed_result(self):
        self.prepare_project()
        samples = self.root / "project/samples.json"
        samples.write_text(samples.read_text().replace('"pair": 0', '"pair": 99', 1))
        destination, _ = self.run_report(statuses=("skipped", "skipped", "success"), expect=1)
        family = json.loads((destination / "report.json").read_text())["families"][3]
        self.assertEqual(family["status"], "FAIL")

    def test_project_timing_change_with_recomputed_file_hash_is_fail(self):
        self.prepare_project()
        source = self.root / "project"
        samples = source / "samples.json"
        record = json.loads(samples.read_text())
        record["cases"][0]["records"][0]["wall_seconds"] = 8.0
        samples.write_text(json.dumps(record, sort_keys=True))
        files = ("README.md", "campaign.json", "host.json", "replay.json", "samples.json")
        (source / "SHA256SUMS").write_text("".join(hashlib.sha256((source / name).read_bytes()).hexdigest()
                                               + "  " + name + "\n" for name in files))
        destination, _ = self.run_report(statuses=("skipped", "skipped", "success"), expect=1)
        self.assertEqual(json.loads((destination / "report.json").read_text())["families"][3]["status"], "FAIL")

    def test_unbound_unsupported_claim_is_rejected_not_marked_na(self):
        self.prepare_fixed()
        receipt = self.root / "unbound-claim.json"
        receipt.write_text(json.dumps({"family": "fixed-cargo", "operation": "body-edit",
                                       "feature": "union item", "exit": 1}))
        destination, _ = self.run_report("--lust-coverage", str(receipt),
                                         statuses=("success", "skipped", "skipped"), expect=1)
        report = json.loads((destination / "report.json").read_text())
        self.assertEqual(report["coverage_status"], "FAIL")
        self.assertEqual(report["status"], "INCOMPLETE")
        self.assertTrue(all(row["lust"]["status"] == "NT" for row in report["families"][0]["rows"]))

    def test_nonexistent_unsupported_receipt_is_also_rejected(self):
        destination, _ = self.run_report("--lust-coverage", str(self.root / "missing.json"), expect=1)
        report = json.loads((destination / "report.json").read_text())
        self.assertEqual(report["coverage_status"], "FAIL")
        self.assertEqual(report["families"][0]["rows"][0]["lust"]["status"], "NT")

    def test_na_display_only_for_future_externally_proven_unsupported_lane(self):
        reporter = load_module("three_way_report", "report-three-way-performance.py")
        family = reporter.empty_family(reporter.FAMILIES[0], "skipped")
        family["rows"][0]["lust"] = reporter.cell("N/A", "future source/argv-bound unsupported execution")
        markdown = reporter.render({"source_revision": REVISION, "families": [family]})
        self.assertIn("| body-edit | NT | N/A | NT |", markdown)

    def test_boolean_project_exit_is_invalid_not_success(self):
        self.prepare_project()
        source = self.root / "project"
        samples = source / "samples.json"
        record = json.loads(samples.read_text())
        record["cases"][0]["records"][0]["exit"] = False
        samples.write_text(json.dumps(record, sort_keys=True))
        files = ("README.md", "campaign.json", "host.json", "replay.json", "samples.json")
        (source / "SHA256SUMS").write_text("".join(hashlib.sha256((source / name).read_bytes()).hexdigest()
                                               + "  " + name + "\n" for name in files))
        destination, _ = self.run_report(statuses=("skipped", "skipped", "success"), expect=1)
        self.assertEqual(json.loads((destination / "report.json").read_text())["families"][3]["status"], "FAIL")


if __name__ == "__main__":
    unittest.main()
