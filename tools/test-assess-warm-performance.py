#!/usr/bin/env python3
"""Practical assessment controls using explicitly synthetic complete evidence.

No compiler executes and no test timing is performance evidence. The existing
synthetic package fixture supplies real evidence structure; the unmodified full
package verifier must accept it before the practical assessor runs.
"""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent


def module(name, file):
    spec = importlib.util.spec_from_file_location(name, HERE / file)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


A = module("practical_warm_assessment", "assess-warm-performance.py")
E = module("synthetic_complete_campaign", "test-warm-native-comparison.py")


class AssessmentControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        with patch("builtins.print"):
            cls.original = E.synthetic_campaign(cls.root)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def campaign(self, times=None):
        output = self.root / self.id().split(".")[-1]
        shutil.copytree(self.original, output)
        rows = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
        values = {("pristine", "no-op-check"): .001, ("entry", "no-op-check"): .0013,
                  ("cargo", "no-op-check"): .009,
                  ("pristine", "no-op-build"): .003, ("entry", "no-op-build"): .0032,
                  ("cargo", "no-op-build"): .009}
        values.update(times or {})
        commands = {row["id"]: row for row in rows if row["kind"] == "command"}
        for row in rows:
            if row["kind"] != "timing" or (row["lane"], row["phase"]) not in values:
                continue
            seconds = values[row["lane"], row["phase"]]
            if isinstance(seconds, list):
                seconds = seconds[row["sample"]]
            row["seconds"] = commands[row["commands"][0]]["seconds"] = seconds
        primary = json.loads((output / "report.json").read_text())
        primary["samples"] = [{key: value for key, value in row.items() if key != "kind"}
                              for row in rows if row["kind"] == "timing"]
        primary["summaries"] = A.N.summaries(primary["samples"], 24)
        primary["warm_target"] = A.N.parity(primary["summaries"])
        (output / "raw.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
        primary["raw_sha256"] = A.N.S.digest(output / "raw.jsonl")
        A.N.S.write_json(output / "report.json", primary)
        (output / "README.md").write_text(A.N.render(primary))
        status = output.with_suffix(".status")
        status.write_text("0\n" if primary["warm_target"]["meets_target"] else "2\n")
        return output, status

    def test_absolute_budget_accepts_small_latency_despite_relative_miss(self):
        campaign, status = self.campaign()
        before = {p: p.read_bytes() for p in (campaign / "report.json", campaign / "raw.jsonl", campaign / "README.md", status)}
        output = self.root / "accepted-assessment"
        with patch.object(A.N, "verify", wraps=A.N.verify) as verifier:
            with contextlib.redirect_stdout(io.StringIO()):
                code = A.main(["--campaign", str(campaign), "--measurement-status", str(status), "--output", str(output)])
        verifier.assert_called_once_with(campaign)
        self.assertEqual(code, 0)
        report = json.loads((output / "assessment.json").read_text())
        self.assertEqual((output / "README.md").read_text(), A.render(report))
        self.assertTrue(report["meets_practical_target"])
        self.assertFalse(report["original_strict_warm_target"]["meets_target"])
        self.assertEqual(report["measurement_exit_status"], 2)
        check = report["modes"]["no-op-check"]
        self.assertFalse(check["relative_budget_met"])
        self.assertTrue(check["absolute_budget_met"])
        self.assertAlmostEqual(check["added_latency"]["bootstrap_95_seconds"][1], .0003)
        self.assertFalse(report["release_admission"])
        self.assertEqual(before, {p: p.read_bytes() for p in before})

    def test_relative_budget_accepts_larger_absolute_gap(self):
        campaign, status = self.campaign({("pristine", "no-op-check"): .037,
                                          ("entry", "no-op-check"): .040,
                                          ("cargo", "no-op-check"): .060})
        mode = A.assess(campaign, status)["modes"]["no-op-check"]
        self.assertTrue(mode["meets_practical_target"])
        self.assertTrue(mode["relative_budget_met"])
        self.assertFalse(mode["absolute_budget_met"])

    def test_original_strict_pass_retains_zero_exit(self):
        campaign, status = self.campaign({("entry", "no-op-check"): .0009,
                                          ("entry", "no-op-build"): .0029})
        report = A.assess(campaign, status)
        self.assertEqual(report["measurement_exit_status"], 0)
        self.assertTrue(report["original_strict_warm_target"]["meets_target"])
        self.assertTrue(report["meets_practical_target"])

    def test_one_slow_operation_cannot_be_averaged_away(self):
        campaign, status = self.campaign({("entry", "no-op-build"): .005})
        report = A.assess(campaign, status)
        self.assertFalse(report["meets_practical_target"])
        self.assertTrue(report["modes"]["no-op-check"]["meets_practical_target"])
        self.assertFalse(report["modes"]["no-op-build"]["meets_practical_target"])

    def test_equal_rust_time_does_not_establish_a_rust_win(self):
        campaign, status = self.campaign({("cargo", "no-op-check"): .0013})
        report = A.assess(campaign, status)
        self.assertFalse(report["meets_practical_target"])
        self.assertTrue(report["modes"]["no-op-check"]["absolute_budget_met"])
        self.assertFalse(report["modes"]["no-op-check"]["faster_than_rustc"])

    def test_uncertain_median_does_not_pass_using_only_its_point_estimate(self):
        # 13 smaller + 11 larger gaps have a small sample median but an upper
        # bootstrap median bound above the budget. Pair indices are retained.
        campaign, status = self.campaign({("entry", "no-op-check"): [.0013] * 13 + [.0018] * 11})
        mode = A.assess(campaign, status)["modes"]["no-op-check"]
        self.assertLess(mode["added_latency"]["median_added_seconds"], .0005)
        self.assertGreater(mode["added_latency"]["bootstrap_95_seconds"][1], .0005)
        self.assertFalse(mode["meets_practical_target"])

    def test_incomplete_report_is_rejected_by_original_verifier(self):
        campaign, status = self.campaign()
        primary = json.loads((campaign / "report.json").read_text())
        primary["status"] = "failed"
        A.N.S.write_json(campaign / "report.json", primary)
        with self.assertRaisesRegex(ValueError, "only complete"):
            A.assess(campaign, status)

    def test_raw_tamper_is_rejected_even_with_a_new_raw_digest(self):
        campaign, status = self.campaign()
        rows = [json.loads(line) for line in (campaign / "raw.jsonl").read_text().splitlines()]
        next(x for x in rows if x["kind"] == "timing")["seconds"] += .001
        (campaign / "raw.jsonl").write_text("".join(json.dumps(x) + "\n" for x in rows))
        primary = json.loads((campaign / "report.json").read_text())
        primary["raw_sha256"] = A.N.S.digest(campaign / "raw.jsonl")
        A.N.S.write_json(campaign / "report.json", primary)
        with self.assertRaises(ValueError):
            A.assess(campaign, status)

    def test_unexpected_exit_statuses_fail_before_assessment(self):
        status = self.root / "unexpected-status"
        for code in (b"1\n", b"3\n", b"124\n", b"-9\n", b"0.0\n", b"true\n", b""):
            status.write_bytes(code)
            with self.subTest(code=code), self.assertRaisesRegex(ValueError, "only original measurement exit"):
                A.assess(self.original, status)

    def test_exit_status_cannot_relabel_a_strict_target_miss(self):
        campaign, status = self.campaign()
        status.write_text("0\n")
        with self.assertRaisesRegex(ValueError, "disagrees with verified strict target"):
            A.assess(campaign, status)

    def test_missing_evidence_is_fatal_and_publishes_no_assessment(self):
        output = self.root / "missing-assessment"
        with contextlib.redirect_stderr(io.StringIO()):
            code = A.main(["--campaign", str(self.root / "missing"), "--measurement-status", str(self.root / "missing.status"), "--output", str(output)])
        self.assertEqual(code, 1)
        self.assertFalse(output.exists())

    def test_cli_retains_failed_practical_decision_separately(self):
        campaign, status = self.campaign({("entry", "no-op-build"): .005})
        output = self.root / "failed-assessment"
        with contextlib.redirect_stdout(io.StringIO()):
            code = A.main(["--campaign", str(campaign), "--measurement-status", str(status), "--output", str(output)])
        self.assertEqual(code, 2)
        report = json.loads((output / "assessment.json").read_text())
        self.assertEqual(report["status"], "not-met")
        self.assertEqual((output / "README.md").read_text(), A.render(report))
        for line in (output / "SHA256SUMS").read_text().splitlines():
            checksum, name = line.split("  ", 1)
            self.assertEqual(A.digest((output / name).read_bytes()), checksum)

    def test_output_cannot_overwrite_or_enter_original_campaign(self):
        for output in (self.original, self.original / "assessment"):
            with self.subTest(output=output), contextlib.redirect_stderr(io.StringIO()):
                code = A.main(["--campaign", str(self.original), "--measurement-status", str(self.root / "unused.status"), "--output", str(output)])
            self.assertEqual(code, 1)

    def test_added_latency_replay_is_deterministic_and_keeps_pairs(self):
        pairs = [(.001 + i * .0001, .0013 + i * .0001) for i in range(24)]
        first = A.added_latency(pairs, A.SEEDS["no-op-check"])
        self.assertEqual(first, A.added_latency(pairs, A.SEEDS["no-op-check"]))
        self.assertEqual(first["bootstrap_resamples"], 10000)
        for bound in first["bootstrap_95_seconds"]:
            self.assertAlmostEqual(bound, .0003)


if __name__ == "__main__":
    unittest.main()
