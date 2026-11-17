#!/usr/bin/env python3
"""Synthetic reporting tests only; these do not execute/qualify a compiler."""
import copy
import csv
import io
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

TOOL = Path(__file__).with_name("report-mixed-performance.py")
spec = importlib.util.spec_from_file_location("report_mixed", TOOL)
reporter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reporter)


def fixture(root):
    n, seed = 6, 19
    meta = dict(schema=1, pairs=n, seed=seed, pristine_revision="a" * 40,
                pristine_tree="b" * 40, compiler_sha256={"synthetic/compiler": "c" * 64},
                profiles="synthetic", rust_reference_version="synthetic", cold_scope="synthetic",
                warm_scope="synthetic", timing_scope="synthetic", scope="synthetic",
                inference_limits="synthetic", order_method="randomized-cyclic-blocks-v1",
                performance_admission="not granted by this runner")
    rows = [dict(kind="identities", report=dict(meta, status="failed", summaries=None))]
    durations = {}
    folder = root / "lustr-paired-measurements"
    folder.mkdir()
    for family, lanes in reporter.LANES.items():
        for i in range(n):
            order = [(j + i) % len(lanes) for j in range(len(lanes))]
            rows.append(dict(kind="order", family=family, lanes=order, sample=i))
            for phase in reporter.PHASES:
                for lane_index in order:
                    lane = lanes[lane_index]
                    value = .01 * (i + 1) * (lane_index + 1)
                    durations[family, lane, phase, i] = value
                    outcome = (1 if family == "native" else 101) if phase == "rejection" else 0
                    row = dict(kind="command", family=family, lane=lane, phase=phase, sample=i,
                               seconds=value, expected_exit=outcome, exit=outcome,
                               timed_out=False, signal=None,
                               source_sha256={"/synthetic/source/main.rs": "d" * 64},
                               source_after_sha256={"/synthetic/source/main.rs": "d" * 64})
                    for stream in ("stdout", "stderr"):
                        name = f"{family}-{lane}-{phase}-{i}.{stream}"
                        data = b"synthetic output\n"
                        (folder / name).write_bytes(data)
                        row[stream] = {"Unix": list(name.encode())}
                        row[stream + "_bytes"] = len(data)
                        row[stream + "_sha256"] = reporter.sha256(data)
                    rows.append(row)
    rows += [dict(kind="native-retained-check", lane=lane, route="bodies-without-body-work")
             for lane in reporter.LANES["native"]]
    summaries = []
    for family, lanes in reporter.LANES.items():
        for candidate in lanes[1:]:
            for phase in reporter.PHASES:
                pairs = [(durations[family, lanes[0], phase, i], durations[family, candidate, phase, i])
                         for i in range(n)]
                analysis = reporter.summarize(pairs, 100, seed)
                summaries.append(dict(family=family, baseline=lanes[0], candidate=candidate,
                                      phase=phase, analysis=analysis, ratio_direction=reporter.DIRECTION,
                                      observation="slower-on-this-corpus"))
    (root / "lustr-source-revision.txt").write_text("e" * 40 + "\n" + "f" * 40 + "\n")
    return dict(meta, status="completed", records=rows, summaries=summaries)


class ReportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.data = fixture(self.root)
        self.save()

    def tearDown(self):
        self.temp.cleanup()

    def save(self):
        raw = b"".join((json.dumps(row, separators=(",", ":")) + "\n").encode()
                       for row in self.data["records"])
        self.data["raw_sha256"] = reporter.sha256(raw)
        p = self.root / "lustr-paired-measurements"
        (p / "raw.jsonl").write_bytes(raw)
        (p / "report.json").write_text(json.dumps(self.data))

    def first_command(self):
        return next(x for x in self.data["records"] if x["kind"] == "command")

    def verify(self):
        evidence = reporter.Evidence(self.root)
        try:
            return reporter.verify(evidence)
        finally:
            evidence.close()

    def rejected(self):
        self.save()
        with self.assertRaises((ValueError, KeyError, OSError)):
            self.verify()

    def test_complete_report_and_markdown(self):
        result = self.verify()
        self.assertEqual(result["verified"]["comparisons"], 27)
        self.assertEqual(result["verified"]["timed_commands"], 270)
        self.assertEqual(result["verified"]["command_outputs"], 540)
        self.assertEqual(len(result["timing_samples"]), 45)
        text = reporter.markdown(result)
        self.assertEqual(text.count("| rejection |"), 3)
        self.assertIn("not equivalence tests", text)

    def test_csv_roundtrip(self):
        result = self.verify()
        rows = list(csv.reader(io.StringIO(reporter.csv_samples(result))))
        self.assertEqual(rows[0][1:], [f"pair_{i}_seconds" for i in range(6)])
        self.assertEqual({x[0]: list(map(float, x[1:])) for x in rows[1:]}, result["timing_samples"])

    def test_unknown_phase_and_replay_capacity(self):
        self.first_command()["phase"] = "new-unrecognized-work"
        self.rejected()
        with self.assertRaises(ValueError):
            reporter.summarize([(1., 1.)] * 100, 1000000, 1)

    def test_failed_campaign(self):
        self.data["status"] = "failed"
        self.rejected()

    def test_changed_raw_digest(self):
        p = self.root / "lustr-paired-measurements/raw.jsonl"
        p.write_bytes(p.read_bytes() + b"\n")
        with self.assertRaises(ValueError):
            self.verify()

    def test_record_copy_disagrees(self):
        self.data["records"] = self.data["records"][:-1]
        (self.root / "lustr-paired-measurements/report.json").write_text(json.dumps(self.data))
        with self.assertRaises(ValueError):
            self.verify()

    def test_strict_json(self):
        for text in ('{"a":1,"a":2}', '{"a":NaN}', '{"a":Infinity}'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                reporter.json_load(text)

    def test_duplicate_and_missing_timed_samples(self):
        original = copy.deepcopy(self.data)
        self.data["records"].append(copy.deepcopy(self.first_command()))
        self.rejected()
        self.data = original
        self.data["records"].remove(self.first_command())
        self.rejected()

    def test_boolean_and_out_of_range_sample(self):
        for value in (True, -1, 6):
            with self.subTest(sample=value):
                self.first_command()["sample"] = value
                self.rejected()

    def test_invalid_time(self):
        for value in (0, -1, True):
            with self.subTest(seconds=value):
                self.first_command()["seconds"] = value
                self.rejected()

    def test_wrong_exit_timeout_and_signal(self):
        original = copy.deepcopy(self.first_command())
        for key, value in (("exit", 1), ("expected_exit", True), ("timed_out", True), ("signal", 9)):
            with self.subTest(key=key):
                row = self.first_command()
                row.clear(); row.update(original); row[key] = value
                self.rejected()

    def test_wrong_rejection_contract(self):
        row = next(x for x in self.data["records"] if x.get("phase") == "rejection")
        row["expected_exit"] = row["exit"] = 0
        self.rejected()

    def test_changed_command_output(self):
        name = reporter.output_name(self.first_command()["stdout"])
        (self.root / "lustr-paired-measurements" / name).write_bytes(b"altered")
        with self.assertRaises(ValueError):
            self.verify()

    def test_output_path_escape(self):
        self.first_command()["stdout"] = {"Unix": list(b"../outside")}
        self.rejected()

    def test_mutation_during_timing(self):
        self.first_command()["source_after_sha256"] = {"/synthetic/source/main.rs": "0" * 64}
        self.rejected()

    def test_mismatched_paired_input(self):
        row = self.first_command()
        row["source_sha256"] = row["source_after_sha256"] = {"/synthetic/source/main.rs": "0" * 64}
        self.rejected()

    def test_missing_duplicate_and_unbalanced_order(self):
        original = copy.deepcopy(self.data)
        order = next(x for x in self.data["records"] if x["kind"] == "order")
        self.data["records"].remove(order)
        self.rejected()
        self.data = copy.deepcopy(original)
        self.data["records"].append(order)
        self.rejected()
        self.data = original
        for row in self.data["records"]:
            if row["kind"] == "order":
                row["lanes"] = list(range(len(reporter.LANES[row["family"]])))
        self.rejected()

    def test_summary_tampering(self):
        original = copy.deepcopy(self.data)
        for key, value in (("geometric_ratio", 100), ("bootstrap_95", [100, 101]), ("seed", 20)):
            with self.subTest(key=key):
                self.data = copy.deepcopy(original)
                self.data["summaries"][0]["analysis"][key] = value
                self.rejected()

    def test_summary_coverage(self):
        original = copy.deepcopy(self.data)
        self.data["summaries"].pop()
        self.rejected()
        self.data = original
        self.data["summaries"].append(copy.deepcopy(self.data["summaries"][0]))
        self.rejected()

    def test_observation_and_direction(self):
        original = copy.deepcopy(self.data)
        self.data["summaries"][0]["observation"] = "faster-on-this-corpus"
        self.rejected()
        self.data = original
        self.data["summaries"][0]["ratio_direction"] = "candidate/baseline"
        self.rejected()

    def test_missing_retained_check_and_changed_identity(self):
        original = copy.deepcopy(self.data)
        self.data["records"].pop()
        self.rejected()
        self.data = original
        self.data["pristine_revision"] = "0" * 40
        self.rejected()

    def test_statistics_constant_null_symmetry_and_ties(self):
        summary = reporter.summarize([(2., 1.)] * 8, 1000, 7)
        self.assertAlmostEqual(summary["geometric_ratio"], 2)
        self.assertEqual(summary["exact_sign_flip_p"], 2 / 256)
        null = reporter.summarize([(3., 3.)] * 8, 1000, 7)
        self.assertEqual(null["bootstrap_95"], [1., 1.])
        self.assertEqual(null["exact_sign_flip_p"], 1)
        forward = reporter.summarize([(1., 2.), (4., 2.), (3., 2.), (8., 2.)], 2000, 19)
        reverse = reporter.summarize([(2., 1.), (2., 4.), (2., 3.), (2., 8.)], 2000, 19)
        self.assertAlmostEqual(forward["geometric_ratio"] * reverse["geometric_ratio"], 1)
        self.assertAlmostEqual(forward["bootstrap_95"][0] * reverse["bootstrap_95"][1], 1)
        self.assertEqual(reporter.summarize([(2., 1.), (2., 1.), (1., 1.)], 1000, 1)["exact_sign_flip_p"], .5)

    def test_zip_reading_and_unsafe_members(self):
        archive = self.root / "artifact.zip"
        with zipfile.ZipFile(archive, "w") as z:
            for path in self.root.rglob("*"):
                if path.is_file() and path != archive:
                    z.write(path, str(path.relative_to(self.root)))
        evidence = reporter.Evidence(archive)
        try:
            self.assertEqual(reporter.verify(evidence)["verified"]["comparisons"], 27)
        finally:
            evidence.close()
        with zipfile.ZipFile(archive, "w") as z:
            z.writestr("../escape", "bad")
        with self.assertRaises(ValueError):
            reporter.Evidence(archive)

    def test_cli_preserves_existing_and_does_not_publish_failure(self):
        output = self.root / "output"
        output.mkdir()
        (output / "keep").write_text("original")
        run = subprocess.run([sys.executable, str(TOOL), str(self.root), "--output", str(output)],
                             capture_output=True, text=True)
        self.assertNotEqual(run.returncode, 0)
        self.assertEqual((output / "keep").read_text(), "original")
        self.data["status"] = "failed"
        self.save()
        missing = self.root / "not-created"
        run = subprocess.run([sys.executable, str(TOOL), str(self.root), "--output", str(missing)],
                             capture_output=True, text=True)
        self.assertNotEqual(run.returncode, 0)
        self.assertFalse(missing.exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
