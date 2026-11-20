#!/usr/bin/env python3
"""Exercise the retained reconstruction's integrity checks in both Python modes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
ARCHIVE = ROOT.parents[1] / "archives/2026-11-17-40d2ac3-qualification.zip"


class ReconstructionIntegrityTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(ARCHIVE.is_file(), "supply the retained original with --archive")
        temporary = tempfile.TemporaryDirectory(prefix="lustr-replay-integrity-")
        self.addCleanup(temporary.cleanup)
        self.copy = Path(temporary.name) / "bundle"
        shutil.copytree(ROOT, self.copy)

    def refresh_checksum(self, name):
        path = self.copy / "SHA256SUMS"
        lines = path.read_text().splitlines()
        matches = [index for index, line in enumerate(lines) if line.split("  ", 1)[1] == name]
        self.assertEqual(len(matches), 1)
        lines[matches[0]] = hashlib.sha256((self.copy / name).read_bytes()).hexdigest() + "  " + name
        path.write_text("\n".join(lines) + "\n")

    def run_modes(self, rejection=None):
        for optimized in (False, True):
            with self.subTest(optimized=optimized):
                environment = dict(os.environ)
                environment.pop("PYTHONOPTIMIZE", None)
                if optimized:
                    environment["PYTHONOPTIMIZE"] = "1"
                result = subprocess.run(
                    [sys.executable, str(self.copy / "replay.py"), "--archive", str(ARCHIVE)],
                    env=environment, capture_output=True, timeout=30)
                if rejection is None:
                    self.assertEqual(result.returncode, 0, result.stderr.decode())
                    report = json.loads(result.stdout)
                    self.assertEqual(report["original_ci_profile"], "failed as recorded")
                    self.assertEqual(report["original_inputs_verified"], 130)
                    self.assertEqual((report["profiles"], report["commands"], report["native_successful_records"]), (26, 15, 35))
                    self.assertFalse(report["new_compiler_qualification"])
                else:
                    self.assertEqual(result.returncode, 1)
                    self.assertEqual(result.stdout, b"", "corrupt evidence must not produce a verification claim")
                    self.assertIn(rejection, result.stderr.decode())

    def corrupt_receipt(self):
        path = self.copy / "reconstruction.json"
        value = json.loads(path.read_bytes())
        value["artifact_sha256"] = "0" * 64
        path.write_text(json.dumps(value, indent=2) + "\n")

    def test_original_reconstruction_in_both_modes(self):
        self.run_modes()

    def test_changed_receipt_fails_bundle_checksum_in_both_modes(self):
        self.corrupt_receipt()
        self.run_modes("bundle checksum mismatch: reconstruction.json")

    def test_changed_receipt_cannot_bypass_original_archive_binding(self):
        self.corrupt_receipt()
        self.refresh_checksum("reconstruction.json")
        self.run_modes("original archive digest mismatch")

    def test_changed_timing_cannot_bypass_exact_derived_replay(self):
        path = self.copy / "derived-summary.json"
        value = json.loads(path.read_bytes())
        value["modes"]["body"]["native_emitter"]["phase_nanoseconds"]["nm"] += 1
        path.write_text(json.dumps(value, indent=2) + "\n")
        self.refresh_checksum("derived-summary.json")
        self.run_modes("derived summary mismatch")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, default=ARCHIVE)
    args, remaining = parser.parse_known_args()
    ARCHIVE = args.archive.resolve()
    unittest.main(argv=[__file__, *remaining])
