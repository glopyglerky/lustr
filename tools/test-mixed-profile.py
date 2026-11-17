#!/usr/bin/env python3
"""Corruption checks for diagnostic phase evidence, never compiler speed tests."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location("mixed_profile", Path(__file__).with_name("profile-mixed-compiler.py"))
REPORT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REPORT)


def records(success=True, reused=False):
    return [
        {"schema": "lustr.mixed-phase-profile", "version": 1, "pid": 123,
         "started_unix_ns": 10, "rustc": "a" * 40, "native": "b" * 16,
         "request_key": "c" * 16, "instrumented": True, "nested_spans": True},
        {"phase": "request-setup", "start_ns": 1, "end_ns": 2, "elapsed_ns": 1},
        {"phase": "native-compile", "start_ns": 3, "end_ns": 4, "elapsed_ns": 1},
        {"phase": "rustc-driver", "start_ns": 2, "end_ns": 5, "elapsed_ns": 3},
        {"phase": "driver", "start_ns": 0, "end_ns": 6, "elapsed_ns": 6},
        {"completed": True, "success": success, "reused": reused, "check_only": False},
    ]


def write(path, values):
    path.write_text("".join(json.dumps(value) + "\n" for value in values))


class ProfileEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.profiles = self.root / "profiles"
        self.work = self.root / "qualification"
        self.profiles.mkdir()
        self.work.mkdir()
        for n, (label, success) in enumerate(REPORT.EXPECTED.items()):
            name = f"123-{n}.jsonl"
            write(self.profiles / name, records(success, label in {"body", "arithmetic-edit", "link-failure-recovery"}))
            (self.work / f"{label}.profiles").write_text(name + "\n")
            (self.work / f"{label}.trace").write_text("request-key=" + "c" * 16 + " retained=true\n")
            (self.work / f"{label}.status").write_text(f"exit status: {0 if success else 101} elapsed_ns=12\n")

    def test_valid_nested_phase_records(self):
        report = REPORT.summarize(self.root)
        self.assertFalse(report["paired_speed_evidence"])
        self.assertEqual(len(report["invocations"]), len(REPORT.EXPECTED))
        self.assertIn("columns must not be added", REPORT.render(report))

    def test_missing_process_is_detected_by_independent_trace(self):
        (self.work / "body.profiles").write_text("")
        (self.profiles / "123-1.jsonl").unlink()
        with self.assertRaisesRegex(ValueError, "independent request trace"):
            REPORT.summarize(self.root)

    def test_unassigned_process_is_rejected(self):
        write(self.profiles / "999-1.jsonl", records())
        with self.assertRaisesRegex(ValueError, "unassigned"):
            REPORT.summarize(self.root)

    def test_incomplete_record_is_rejected(self):
        values = records()
        values[-1]["completed"] = False
        write(self.profiles / "123-0.jsonl", values)
        with self.assertRaisesRegex(ValueError, "incomplete"):
            REPORT.summarize(self.root)

    def test_wrong_duration_is_rejected(self):
        values = records()
        values[1]["elapsed_ns"] = 99
        write(self.profiles / "123-0.jsonl", values)
        with self.assertRaisesRegex(ValueError, "inconsistent phase duration"):
            REPORT.summarize(self.root)

    def test_span_outside_driver_is_rejected(self):
        values = records()
        values[2].update(start_ns=20, end_ns=21)
        write(self.profiles / "123-0.jsonl", values)
        with self.assertRaisesRegex(ValueError, "escaped driver"):
            REPORT.summarize(self.root)

    def test_missing_failure_is_rejected(self):
        write(self.profiles / "123-3.jsonl", records(True))
        with self.assertRaisesRegex(ValueError, "missing failed compiler"):
            REPORT.summarize(self.root)

    def test_lost_reuse_is_rejected(self):
        write(self.profiles / "123-1.jsonl", records(True, False))
        with self.assertRaisesRegex(ValueError, "lost retained execution"):
            REPORT.summarize(self.root)

    def test_identity_change_is_rejected(self):
        values = records()
        values[0]["native"] = "d" * 16
        write(self.profiles / "123-0.jsonl", values)
        with self.assertRaisesRegex(ValueError, "mixed compiler identities"):
            REPORT.summarize(self.root)

    def test_unknown_phase_is_rejected(self):
        values = records()
        values[2]["phase"] = "unaccounted-work"
        write(self.profiles / "123-0.jsonl", values)
        with self.assertRaisesRegex(ValueError, "unknown phase"):
            REPORT.summarize(self.root)

    def test_wrong_command_outcome_is_rejected(self):
        (self.work / "body.status").write_text("exit status: 101 elapsed_ns=12\n")
        with self.assertRaisesRegex(ValueError, "incorrect command outcome"):
            REPORT.summarize(self.root)


if __name__ == "__main__":
    unittest.main()
