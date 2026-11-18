#!/usr/bin/env python3
"""Bounded startup-diagnostic controls with synthetic child results only."""
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
def module(name, file):
    spec = importlib.util.spec_from_file_location(name, HERE / file)
    result = importlib.util.module_from_spec(spec); spec.loader.exec_module(result); return result
D = module("startup_diagnostic", "run-warm-native-startup.py")
E = module("complete_controls", "test-warm-native-comparison.py")


class StartupControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        with patch("builtins.print"):
            cls.campaign = E.synthetic_campaign(cls.root)
        primary = json.loads((cls.campaign / "report.json").read_text())
        rows = [json.loads(line) for line in (cls.campaign / "raw.jsonl").read_text().splitlines()]
        # Turn this synthetic complete campaign into a complete target miss.
        # Its proper raw bindings/statistics remain consistent and replayable.
        command_ids = {row["commands"][0] for row in primary["samples"] if row["lane"] == "entry"}
        for row in rows:
            if row["kind"] == "command" and row["id"] in command_ids or row["kind"] == "timing" and row["lane"] == "entry": row["seconds"] *= 2
        primary["samples"] = [{key: value for key, value in row.items() if key != "kind"} for row in rows if row["kind"] == "timing"]
        primary["summaries"] = D.N.summaries(primary["samples"], 24)
        primary["warm_target"] = D.N.parity(primary["summaries"])
        (cls.campaign / "raw.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
        primary["raw_sha256"] = D.S.digest(cls.campaign / "raw.jsonl")
        D.S.write_json(cls.campaign / "report.json", primary)
        (cls.campaign / "README.md").write_text(D.N.render(primary))
        baseline = Path(primary["package_builds"]["baseline"]["source_path"])
        pristine = Path(primary["pristine_source"])
        manifest = '[package]\nname="lust"\nversion="0.2.0"\n'
        (baseline / "compiler/native/Cargo.toml").write_text(manifest)
        (pristine / "Cargo.toml").write_text(manifest)
        cls.intermediates = cls.root / "pristine-intermediates"
        emitted = cls.intermediates / "release/build/lust/synthetic/output"
        emitted.parent.mkdir(parents=True); emitted.write_text("cargo:rustc-env=LUST_BUILD_HASH=0123456789abcdef\n")
        cls.output = cls.root / "diagnostic"
        baseline_receipt = primary["package_builds"]["baseline"]["files"]["NATIVE-BUILD-RECEIPT.txt"]
        expected = {"pristine": b"lust 0.2.0 (build 01234567)\n", "baseline": D.expected_version(manifest.encode(), D.N.evidence(cls.root / "baseline-build", baseline_receipt))}
        def run(recorder, argv, env, cwd, lane, phase, expected_exit=0):
            row = {"kind": "command", "id": len(recorder.records), "argv": list(map(str, argv)), "environment": dict(env), "cwd": str(cwd),
                "lane": lane, "phase": phase, "expected_exit": expected_exit, "exit": expected_exit, "signal": None,
                "timed_out": False, "output_limit": False, "seconds": .001}
            for name, data in (("stdout", expected[lane]), ("stderr", b"")):
                path = recorder.output / "commands" / f"{row['id']:05}.{name}"; path.write_bytes(data); row[name] = D.N.binding(path, recorder.output)
            recorder.event(row); return row, expected[lane], b""
        original_verify = D.N.verify_package
        with patch.object(D.N, "verify_package", side_effect=lambda root, lane, live=False: original_verify(root, lane)), \
             patch.object(D.N, "source_identity", return_value={"revision": D.S.PIN, "tree": D.S.TREE}), patch.object(D.S.Recorder, "run", run), patch("builtins.print"):
            D.run(SimpleNamespace(campaign=cls.campaign, pristine_intermediates=cls.intermediates, output=cls.output))
    @classmethod
    def tearDownClass(cls): cls.temporary.cleanup()

    def corrupt(self, mutate):
        output = self.root / self.id().split(".")[-1]; shutil.copytree(self.output, output)
        rows = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
        mutate(rows, output)
        (output / "raw.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
        report = json.loads((output / "report.json").read_text()); report["raw_sha256"] = D.S.digest(output / "raw.jsonl"); D.S.write_json(output / "report.json", report)
        return output

    def test_completed_target_miss_allows_independent_192_observations(self):
        primary = D.N.verify(self.campaign); self.assertFalse(primary["warm_target"]["meets_target"])
        report = D.verify(self.output); self.assertEqual(len(report["samples"]), 192)
        self.assertFalse(any("SOCKET" in key or "DAEMON" in key for key in report["environment"]))

    def test_wrong_version_stdout_is_rejected_even_with_new_stream_digest(self):
        def mutate(rows, output):
            row = next(row for row in rows if row["kind"] == "command")
            path = output / row["stdout"]["file"]; path.write_bytes(b"lust 0.2.0 (build wrong-id)\n"); row["stdout"] = D.N.binding(path, output)
        with self.assertRaises(ValueError): D.verify(self.corrupt(mutate))

    def test_request_work_cannot_replace_version_command(self):
        def mutate(rows, _): next(row for row in rows if row["kind"] == "command")["argv"][1] = "daemon"
        with self.assertRaises(ValueError): D.verify(self.corrupt(mutate))

    def test_missing_prime_is_rejected(self):
        def mutate(rows, _): next(row for row in rows if row["kind"] == "startup-prime")["kind"] = "omitted"
        with self.assertRaises(ValueError): D.verify(self.corrupt(mutate))

    def test_changed_sample_order_is_rejected(self):
        def mutate(rows, _): next(row for row in rows if row["kind"] == "startup-timing")["sample"] = 1
        with self.assertRaises(ValueError): D.verify(self.corrupt(mutate))

    def test_ambiguous_emitted_build_receipt_is_rejected(self):
        root = self.root / "ambiguous-intermediates"; shutil.copytree(self.intermediates, root)
        duplicate = root / "release/build/other/stdout"; duplicate.parent.mkdir(); duplicate.write_text("cargo:rustc-env=LUST_BUILD_HASH=fedcba9876543210\n")
        with self.assertRaises(ValueError): D.emitted_receipt(root)

    def test_incomplete_primary_records_failure_without_timing(self):
        primary = self.root / "incomplete-primary"; primary.mkdir(); D.S.write_json(primary / "report.json", {"schema": D.N.SCHEMA, "status": "failed"})
        output = self.root / "incomplete-diagnostic"
        with self.assertRaises(ValueError): D.run(SimpleNamespace(campaign=primary, pristine_intermediates=self.intermediates, output=output))
        report = json.loads((output / "report.json").read_text()); self.assertEqual(report["status"], "failed"); self.assertEqual(report["samples"], [])

    def test_workflow_uses_completion_only_and_preserves_diagnostic(self):
        text = (HERE.parent / ".github/workflows/warm-native-performance.yml").read_text()
        block = text.split("      - name: Diagnose frozen native-client startup", 1)[1].split("      - name:", 1)[0]
        self.assertIn("if: always() && steps.comparison.outputs.completed == 'true'", block)
        self.assertNotIn("meets_target", block)
        for path in ("report.json", "raw.jsonl", "samples.csv", "commands", "provenance"): self.assertIn("native-startup-diagnostic/" + path, text)


if __name__ == "__main__": unittest.main()
