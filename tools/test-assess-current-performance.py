#!/usr/bin/env python3
"""Adversarial assessor controls using explicitly synthetic evidence.

No compiler executes. These tests qualify arithmetic and evidence rejection,
not compiler behavior, actual performance, independent capture or release.
"""
import contextlib
import importlib.util
import io
import json
import math
from pathlib import Path
import tempfile
import unittest


HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("current_performance", HERE / "assess-current-performance.py")
A = importlib.util.module_from_spec(spec)
spec.loader.exec_module(A)


class AssessmentControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="lustr-synthetic-assessor-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.evidence = self.root / "evidence"
        self.evidence.mkdir()
        self.plan_file, self.samples_file = self.root / "plan.json", self.root / "samples.jsonl"

    def file(self, name, data):
        path = self.evidence / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return {"path": name, "sha256": A.digest(data)}

    def json_file(self, name, value):
        return self.file(name, A.serialized(value))

    def campaign(self, cases=None, pairs=24):
        cases = cases or [{"id": "warm-build", "class": "supported", "frequency": 1,
                           "times": {"rustc": .005, "lustr": .0007, "lust": .0002}}]
        compilers = {}
        for index, engine in enumerate(("lustr", "rustc", "lust"), 1):
            compilers[engine] = {"revision": str(index) * 40, "tree": str(index + 3) * 40, "entry_kind": "direct-compiler",
                                "artifacts": {role: self.file(f"compilers/{engine}-{role}", f"SYNTHETIC {engine} {role}".encode())
                                              for role in ("entry", "library", "helper")}}
        source = self.file("input.rs", b'// SYNTHETIC assessor data; never compiled\nfn main() {}\n')
        self.plan = {"schema": A.PLAN_SCHEMA, "scope": "Synthetic assessor controls, not compiler evidence",
                     "pairs": pairs, "seed": 0x4C55535452435552, "sampling_order": A.ORDER,
                     "timing_boundary": A.BOUNDARY, "compilers": compilers, "cases": [], "qualification_records": []}
        times = {}
        for item in cases:
            request = {"operation": "build", "target": "x86_64-unknown-linux-gnu", "profile": "dev",
                       "lifecycle": item.get("lifecycle", "warm-no-op"), "flags": {"panic": "unwind", "opt-level": 0},
                       "source_inputs": [source], "artifacts": ["executable"] if item.get("expected_exit", 0) == 0 else []}
            case = {key: item[key] for key in ("id", "class", "frequency")}
            case["expected_exit"] = item.get("expected_exit", 0)
            case["request"] = self.json_file(f"requests/{item['id']}.json", request)
            case["commands"] = {engine: ["./" + compilers[engine]["artifacts"]["entry"]["path"], "input.rs", "-o", "application"]
                                for engine in A.engine_order(case, 0)}
            self.plan["cases"].append(case)
            times[item["id"]] = item["times"]
        qualification = {"candidate_revision": compilers["lustr"]["revision"], "candidate_tree": compilers["lustr"]["tree"],
                         "compiler_identity_sha256": A.digest(A.serialized(compilers["lustr"])),
                         "status": "synthetic-only", "full_coverage": True}
        self.plan["qualification_records"].append(self.json_file("qualification.json", qualification))
        self.plan_file.write_bytes(A.serialized(self.plan))
        empty = self.file("empty.txt", b"")
        output = self.file("output.bin", b"SYNTHETIC artifact, never executed")
        self.rows = []
        for index, (pair, case, engine) in enumerate(A.sequence(self.plan)):
            seconds = times[case["id"]][engine]
            if isinstance(seconds, list):
                seconds = seconds[pair]
            record = {"schema": A.RECEIPT_SCHEMA, "plan_sha256": A.digest(self.plan_file.read_bytes()),
                      "pair": pair, "case": case["id"], "engine": engine,
                      "compiler_identity_sha256": A.digest(A.serialized(compilers[engine])),
                      "working_directory": ".", "execution_entry": compilers[engine]["artifacts"]["entry"],
                      "request_sha256": case["request"]["sha256"], "argv": case["commands"][engine],
                      "seconds": seconds, "exit_status": case["expected_exit"], "signal": None, "timing_boundary": A.BOUNDARY,
                      "stdout": empty, "stderr": empty, "artifacts": {"executable": output} if case["expected_exit"] == 0 else {}}
            self.rows.append({"pair": pair, "case": case["id"], "engine": engine,
                              "receipt": self.json_file(f"receipts/{index}.json", record)})
        self.write_rows()

    def write_rows(self):
        self.samples_file.write_text("".join(json.dumps(row) + "\n" for row in self.rows))

    def change_receipt(self, mutate):
        row = next(row for row in self.rows if row["engine"] == "lustr")
        path = row["receipt"]["path"]
        record = json.loads((self.evidence / path).read_bytes())
        mutate(record)
        row["receipt"] = self.json_file(path, record)
        self.write_rows()

    def assess(self):
        return A.assess(self.plan_file, self.samples_file, self.evidence)

    def test_old_absolute_allowance_cannot_establish_near_lust(self):
        self.campaign()
        report = self.assess()
        self.assertTrue(report["overall"]["performance_target_met"])
        self.assertEqual(report["cases"][0]["near_stock_target"]["status"], "missed")
        self.assertLess(.0007 - .0002, .0005 + 1e-15)
        self.assertFalse(report["release_admission"], "even a supplied full_coverage Boolean has no release authority")
        self.assertEqual(report["full_replacement_qualification"], "not-established-by-this-assessment")

    def test_favorable_geometric_mean_cannot_hide_worse_total(self):
        self.campaign([
            {"id": "short", "class": "mixed", "frequency": 1, "times": {"rustc": 1, "lustr": .1}},
            {"id": "long", "class": "mixed", "frequency": 1, "times": {"rustc": 100, "lustr": 500}},
        ])
        self.assertGreater(math.sqrt((1 / .1) * (100 / 500)), 1)
        report = self.assess()
        self.assertEqual(report["overall"]["status"], "slower")
        self.assertAlmostEqual(report["overall"]["reference_over_lustr"], 101 / 500.1)

    def test_both_near_stock_targets_can_pass_while_total_is_slower(self):
        self.campaign([
            {"id": "supported", "class": "supported", "frequency": 1,
             "times": {"rustc": .12, "lustr": .109, "lust": .1}},
            {"id": "complement", "class": "complementary", "frequency": 1,
             "times": {"rustc": 100, "lustr": 104}},
        ])
        report = self.assess()
        self.assertEqual([case["near_stock_target"]["status"] for case in report["cases"]], ["met", "met"])
        self.assertFalse(report["overall"]["performance_target_met"])
        self.assertEqual(report["overall"]["status"], "slower")

    def test_declared_frequencies_decide_total_elapsed_cost(self):
        self.campaign([
            {"id": "frequent", "class": "mixed", "frequency": 100, "times": {"rustc": 1, "lustr": 2}},
            {"id": "rare", "class": "mixed", "frequency": 1, "times": {"rustc": 100, "lustr": 99}},
            {"id": "diagnostic", "class": "mixed", "frequency": 0, "times": {"rustc": 10000, "lustr": 1}},
        ])
        report = self.assess()
        self.assertAlmostEqual(report["overall"]["reference_over_lustr"], 200 / 299)
        self.assertEqual(len(report["cases"]), 3)
        self.assertEqual(report["overall"]["paired_workload_totals_seconds"]["lustr"], [299] * 24)

    def test_uncertain_point_speedup_is_not_a_pass(self):
        self.campaign([{"id": "mixed", "class": "mixed", "frequency": 1,
                        "times": {"rustc": 2, "lustr": [1] * 13 + [3] * 11}}])
        report = self.assess()
        self.assertGreater(report["overall"]["reference_over_lustr"], 1)
        self.assertLess(report["overall"]["bootstrap_95"][0], 1)
        self.assertEqual(report["overall"]["status"], "inconclusive")
        self.assertFalse(report["overall"]["performance_target_met"])
        self.assertEqual(report["cases"][0]["distributions"]["lustr"]["p95_seconds"], 3)

    def test_equal_cost_is_not_a_speedup(self):
        self.campaign([{"id": "mixed", "class": "mixed", "frequency": 1, "times": {"rustc": 2, "lustr": 2}}])
        report = self.assess()
        self.assertEqual(report["overall"]["bootstrap_95"], [1, 1])
        self.assertEqual(report["overall"]["status"], "inconclusive")

    def test_missing_case_and_duplicate_observation_are_refused(self):
        self.campaign()
        self.rows.pop()
        self.write_rows()
        with self.assertRaisesRegex(ValueError, "missing or extra"):
            self.assess()
        self.rows.append(self.rows[0])
        self.write_rows()
        with self.assertRaisesRegex(ValueError, "measurement order"):
            self.assess()

    def test_bad_durations_fail_even_after_receipt_digest_is_updated(self):
        for value in (0, -1, True, "0.1"):
            with self.subTest(value=value):
                self.campaign()
                self.change_receipt(lambda record: record.update(seconds=value))
                with self.assertRaisesRegex(ValueError, "duration must be"):
                    self.assess()
        with self.assertRaisesRegex(ValueError, "nonfinite"):
            A.exact_json(b'{"seconds": NaN}')
        with self.assertRaisesRegex(ValueError, "nonfinite"):
            A.exact_json(b'{"seconds": Infinity}')
        with self.assertRaisesRegex(ValueError, "nonfinite"):
            A.exact_json(b'{"seconds": 1e999}')

    def test_cold_supported_work_does_not_inherit_the_warm_numeric_budget(self):
        self.campaign([{"id": "cold", "class": "supported", "frequency": 1, "lifecycle": "cold",
                        "times": {"rustc": 2, "lustr": 1, "lust": .5}}])
        report = self.assess()
        self.assertTrue(report["overall"]["performance_target_met"])
        self.assertNotIn("near_stock_target", report["cases"][0])

    def test_expected_rejection_does_not_inherit_the_successful_warm_target(self):
        self.campaign([{"id": "rejection", "class": "supported", "frequency": 1, "expected_exit": 1,
                        "times": {"rustc": 2, "lustr": 1, "lust": 1}}])
        report = self.assess()
        self.assertTrue(report["overall"]["performance_target_met"])
        self.assertNotIn("near_stock_target", report["cases"][0])
        self.assertIn("rustc_comparison", report["cases"][0])

    def test_warm_near_stock_does_not_imply_a_rustc_speedup(self):
        self.campaign([{"id": "warm", "class": "supported", "frequency": 1,
                        "times": {"rustc": .5, "lustr": 1, "lust": 1}}])
        target = self.assess()["cases"][0]["near_stock_target"]
        self.assertEqual(target["relative_budget_status"], "met")
        self.assertFalse(target["rustc_speedup_established"])
        self.assertEqual(target["status"], "missed")

    def test_successful_build_cannot_predeclare_away_all_output_artifacts(self):
        self.campaign()
        case = self.plan["cases"][0]
        request = json.loads((self.evidence / case["request"]["path"]).read_bytes())
        request["artifacts"] = []
        case["request"] = self.json_file(case["request"]["path"], request)
        self.plan_file.write_bytes(A.serialized(self.plan))
        with self.assertRaisesRegex(ValueError, "successful build must retain"):
            self.assess()

    def test_changed_command_request_identity_and_timing_boundary_are_refused(self):
        for change, message in [
            ({"argv": ["lustr", "input.rs", "--emit=metadata"]}, "requested operation"),
            ({"request_sha256": "0" * 64}, "requested operation"),
            ({"compiler_identity_sha256": "0" * 64}, "compiler/source/artifact"),
            ({"working_directory": "elsewhere"}, "execution entry or working directory"),
            ({"execution_entry": {"path": "output.bin", "sha256": A.digest(b"SYNTHETIC artifact, never executed")}}, "execution entry or working directory"),
            ({"timing_boundary": "native-emission-only"}, "subphase"),
            ({"plan_sha256": "0" * 64}, "different plan"),
        ]:
            with self.subTest(change=change):
                self.campaign()
                self.change_receipt(lambda record: record.update(change))
                with self.assertRaisesRegex(ValueError, message):
                    self.assess()

    def test_matching_plan_and_receipt_cannot_time_a_different_executable(self):
        self.campaign()
        wrong = self.file("other-entry", b"SYNTHETIC unrelated fast executable")
        for case in self.plan["cases"]:
            case["commands"]["lustr"][0] = "./other-entry"
        self.plan_file.write_bytes(A.serialized(self.plan))
        for row in self.rows:
            name = row["receipt"]["path"]
            record = json.loads((self.evidence / name).read_bytes())
            record["plan_sha256"] = A.digest(self.plan_file.read_bytes())
            if row["engine"] == "lustr":
                record["argv"][0] = "./other-entry"
                record["execution_entry"] = wrong
            row["receipt"] = self.json_file(name, record)
        self.write_rows()
        with self.assertRaisesRegex(ValueError, "does not execute the declared compiler entry"):
            self.assess()

    def test_failed_interrupted_and_missing_artifact_requests_are_refused(self):
        for change, message in [
            ({"exit_status": 2}, "failed or interrupted"),
            ({"exit_status": False}, "failed or interrupted"),
            ({"signal": 9}, "failed or interrupted"),
            ({"artifacts": {}}, "artifact roles"),
        ]:
            with self.subTest(change=change):
                self.campaign()
                self.change_receipt(lambda record: record.update(change))
                with self.assertRaisesRegex(ValueError, message):
                    self.assess()

    def test_declared_helper_and_output_tampering_are_refused(self):
        for path in ("compilers/lustr-helper", "output.bin"):
            with self.subTest(path=path):
                self.campaign()
                (self.evidence / path).write_bytes(b"changed")
                with self.assertRaisesRegex(ValueError, "digest mismatch"):
                    self.assess()

    def test_mismatched_qualification_source_is_refused(self):
        self.campaign()
        record = json.loads((self.evidence / "qualification.json").read_bytes())
        record["candidate_revision"] = "0" * 40
        self.plan["qualification_records"] = [self.json_file("qualification.json", record)]
        self.plan_file.write_bytes(A.serialized(self.plan))
        with self.assertRaisesRegex(ValueError, "qualification reference belongs"):
            self.assess()

    def test_incomplete_and_unbalanced_sampling_designs_are_refused(self):
        for pairs in (12, 25):
            with self.subTest(pairs=pairs):
                self.campaign(pairs=pairs)
                with self.assertRaisesRegex(ValueError, "matched pair blocks|engine-order cycle"):
                    self.assess()

    def test_plan_hash_binding_rejects_changed_frequencies(self):
        self.campaign()
        self.plan["cases"][0]["frequency"] = 100
        self.plan_file.write_bytes(A.serialized(self.plan))
        with self.assertRaisesRegex(ValueError, "different plan"):
            self.assess()

    def test_duplicate_json_keys_and_path_escapes_are_refused(self):
        with self.assertRaisesRegex(ValueError, "duplicate JSON"):
            A.exact_json(b'{"seconds": 2, "seconds": 1}')
        outside = self.root / "outside"
        outside.write_bytes(b"not evidence")
        (self.evidence / "escape").symlink_to(outside)
        evidence = A.Evidence(self.evidence)
        for path in ("../outside", "escape"):
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, "inside evidence|escapes root"):
                evidence.read({"path": path, "sha256": A.digest(outside.read_bytes())})

    def test_cli_writes_a_separate_reproducible_assessment_without_release_permission(self):
        self.campaign()
        before = (self.plan_file.read_bytes(), self.samples_file.read_bytes())
        output = self.root / "assessment"
        with contextlib.redirect_stdout(io.StringIO()):
            code = A.main(["--plan", str(self.plan_file), "--samples", str(self.samples_file),
                           "--evidence-root", str(self.evidence), "--output", str(output)])
        self.assertEqual(code, 0)
        report = json.loads((output / "assessment.json").read_bytes())
        self.assertFalse(report["release_admission"])
        self.assertEqual((output / "README.md").read_text(), A.render(report))
        self.assertEqual(before, (self.plan_file.read_bytes(), self.samples_file.read_bytes()))
        self.assertEqual(report["supplied_qualification_records"], self.plan["qualification_records"])

    def test_cli_refusal_writes_no_report(self):
        self.campaign()
        self.change_receipt(lambda record: record.update(exit_status=1))
        output = self.root / "refused"
        with contextlib.redirect_stderr(io.StringIO()):
            code = A.main(["--plan", str(self.plan_file), "--samples", str(self.samples_file),
                           "--evidence-root", str(self.evidence), "--output", str(output)])
        self.assertEqual(code, 1)
        self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
