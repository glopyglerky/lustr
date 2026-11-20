#!/usr/bin/env python3
"""Trace/diagnostic gate controls, separate from actual compiler qualification."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


spec = importlib.util.spec_from_file_location("native_metadata_campaign", Path(__file__).with_name("test-native-consumer-metadata.py"))
campaign = importlib.util.module_from_spec(spec)
spec.loader.exec_module(campaign)


class TraceControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="lustr-metadata-trace-")
        self.root = Path(self.temporary.name).resolve()
        self.candidate, self.clang, self.ar, self.forbidden = [self.root / name for name in ("qualifier", "clang", "ar", "rustc")]
        for path in (self.candidate, self.clang, self.ar, self.forbidden):
            path.write_bytes(b"parser fixture, not an executable")
        self.allowed = {self.candidate, self.clang, self.ar}
        self.trace = self.root / "trace"

    def tearDown(self):
        self.temporary.cleanup()

    def complete(self, path, pid=101, result="0"):
        return f'{pid} execve({json.dumps(str(path))}, ["fixture"], 0x1 /* 1 var */) = {result}\n'

    def split(self, path, pid=102):
        return (f'{pid} execve({json.dumps(str(path))}, ["fixture"], <unfinished ...>\n',
                f'{pid} <... execve resumed>0x1 /* 1 var */) = 0\n')

    def parse(self, contents):
        self.trace.write_text(contents)
        return campaign.traced_executables(self.trace, self.allowed, self.candidate)

    def test_allowed_complete_and_failed_attempts_are_accounted(self):
        result = self.parse(self.complete(self.candidate) + self.complete(self.clang, 102, "-1 EAGAIN (Resource temporarily unavailable)")
                            + self.complete(self.clang, 102) + "102 +++ exited with 0 +++\n101 +++ exited with 0 +++\n")
        self.assertTrue(result["complete"])
        self.assertEqual([item["status"] for item in result["attempts"]], [0, -1, 0])
        self.assertEqual(result["executed"], [str(self.candidate), str(self.clang)])

    def test_allowed_interleaved_unfinished_calls(self):
        clang_start, clang_end = self.split(self.clang, 102)
        ar_start, ar_end = self.split(self.ar, 103)
        result = self.parse(self.complete(self.candidate) + clang_start + ar_start + ar_end + clang_end)
        self.assertEqual(result["executed"], [str(self.candidate), str(self.ar), str(self.clang)])

    def test_hidden_forbidden_split_execution_fails(self):
        start, end = self.split(self.forbidden)
        with self.assertRaisesRegex(AssertionError, "undeclared"):
            self.parse(self.complete(self.candidate) + start + self.complete(self.ar, 103) + end)

    def test_failed_forbidden_attempt_fails(self):
        with self.assertRaisesRegex(AssertionError, "undeclared"):
            self.parse(self.complete(self.candidate) + self.complete(self.forbidden, 102, "-1 EACCES (Permission denied)"))

    def test_execveat_complete_and_resumed_fail(self):
        for record in [f'102 execveat(3, {json.dumps(str(self.clang))}, [], 0x1, 0) = 0\n',
                       '102 <... execveat resumed>) = 0\n']:
            with self.subTest(record=record), self.assertRaisesRegex(AssertionError, "execveat"):
                self.parse(self.complete(self.candidate) + record)

    def test_unmatched_wrong_pid_and_incomplete_resumes_fail(self):
        start, end = self.split(self.clang)
        for records in [end, start, start + end.replace("102", "103", 1), start + '102 <... execve resumed>\n',
                        start + '102 +++ exited with 0 +++\n']:
            with self.subTest(records=records), self.assertRaises(AssertionError):
                self.parse(self.complete(self.candidate) + records)

    def test_second_start_cannot_hide_pending_call(self):
        start, end = self.split(self.clang)
        for records in [start + start + end, start + self.complete(self.ar, 102) + end]:
            with self.subTest(records=records), self.assertRaises(AssertionError):
                self.parse(self.complete(self.candidate) + records)

    def test_shortened_unquoted_and_relative_paths_fail(self):
        for path in [json.dumps(str(self.clang)) + "...", "0x1234", '"clang"', '"/does-not-exist/compiler"']:
            with self.subTest(path=path), self.assertRaises(AssertionError):
                self.parse(self.complete(self.candidate) + f'102 execve({path}, [], 0x1) = 0\n')

    def test_missing_duplicate_and_late_candidate_fail(self):
        for records in ["", self.complete(self.clang), self.complete(self.clang) + self.complete(self.candidate, 102),
                        self.complete(self.candidate) + self.complete(self.candidate, 102)]:
            with self.subTest(records=records), self.assertRaisesRegex(AssertionError, "initial qualifier"):
                self.parse(records)

    def test_unparsed_and_unknown_exec_results_fail(self):
        for records in ['102 <... execve resumed garbage>) = 0\n',
                        self.complete(self.clang, 102, "?"), "strace: Process detached\n", "102 unrecognized record\n"]:
            with self.subTest(records=records), self.assertRaises(AssertionError):
                self.parse(self.complete(self.candidate) + records)


class NativeDiagnosticControls(unittest.TestCase):
    def test_only_original_type_failure_without_artifacts_qualifies(self):
        with tempfile.TemporaryDirectory(prefix="lustr-metadata-diagnostic-") as temporary:
            root = Path(temporary).resolve()
            source = root / "invalid-private.rs"
            source.write_bytes(campaign.INVALID_PRIVATE)
            output = root / "output"
            output.mkdir()
            error = ("error[L0110]: mismatched types: expected `u64`, found `bool`\n"
                     f"  --> {source}:3:29\n"
                     "   | fn hidden(value:u64)->u64 { true }\n"
                     "   |                             ^^^^\n"
                     "   = fix: Change the value to type `u64`, or change the expected type.\n\n")
            accepted = {"phase": "native-prepare", "error": error}
            campaign.native_private_rejection(accepted, source, output)
            for changed in ["cannot read source: permission denied", error.replace("L0110", "L0001"),
                            error.replace(":3:29", ":2:29"), error.replace("found `bool`", "found `u64`"),
                            error.replace("^^^^", "^")]:
                with self.subTest(changed=changed), self.assertRaises(AssertionError):
                    campaign.native_private_rejection({"phase": "native-prepare", "error": changed}, source, output)
            (output / "libconsumer.rlib").write_bytes(b"incorrectly published")
            with self.assertRaisesRegex(AssertionError, "published"):
                campaign.native_private_rejection(accepted, source, output)


if __name__ == "__main__":
    unittest.main(verbosity=2)
