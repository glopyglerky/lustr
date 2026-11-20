#!/usr/bin/env python3
"""Negative controls for the capability-result evaluator, not compiler coverage.

Actual child processes return matching/mismatching results; parser controls
plant missing or inconsistent test outcomes. The separate qualifier must still
compile and run the actual public compiler and independent Rust oracle.
"""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("capability_results", Path(__file__).with_name("qualify-capability-results.py"))
Q = importlib.util.module_from_spec(spec)
spec.loader.exec_module(Q)


def child(stdout="42\n", stderr="", status=0):
    code = f"import sys; sys.stdout.write({stdout!r}); sys.stderr.write({stderr!r}); sys.exit({status})"
    return subprocess.run([sys.executable, "-c", code], capture_output=True, timeout=10)


RUST_TESTS = b"""running 2 tests
test adds ... ok
test slow ... ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
"""
PANIC_OUTPUT = "\nthread 'adds' (123) panicked at /fixture/tests.rs:5:7:\nboom\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n"


def rust_panics():
    oracle = RUST_TESTS.replace(b"test adds ... ok", b"test adds - should panic ... ok")
    shown = "successes:\n\n---- adds stdout ----\n" + PANIC_OUTPUT + "\nsuccesses:\n    adds\n\n"
    return oracle.replace(b"test result:", shown.encode() + b"test result:")


def native_records():
    return [{"schema": 1, "kind": "test", "name": "adds", "status": "pass", "exit_code": 0, "stdout": "", "stderr": ""},
            {"schema": 1, "kind": "test", "name": "slow", "status": "ignored", "exit_code": None, "stdout": "", "stderr": ""},
            {"schema": 1, "kind": "summary", "build": "ok", "total": 2, "passed": 1,
             "failed": 0, "timed_out": 0, "ignored": 1, "filtered_out": 0}]


def encoded(records):
    return b"\n".join(json.dumps(record).encode() for record in records)


class ResultControls(unittest.TestCase):
    def test_actual_matching_child_streams_and_exit_are_accepted(self):
        Q.compare_program(child(), child())

    def test_successful_child_with_wrong_output_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "stdout"):
            Q.compare_program(child("41\n"), child())

    def test_successful_child_with_wrong_stderr_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "stderr"):
            Q.compare_program(child(stderr="unexpected"), child())

    def test_matching_output_does_not_hide_failure_or_matching_failures(self):
        for candidate, oracle in [(child(status=7), child()), (child(), child(status=7)), (child(status=7), child(status=7))]:
            with self.subTest(candidate=candidate.returncode, oracle=oracle.returncode), self.assertRaisesRegex(ValueError, "exited"):
                Q.compare_program(candidate, oracle)

    def test_real_signal_is_a_failure(self):
        candidate = subprocess.run([sys.executable, "-c", "import os,signal; os.kill(os.getpid(), signal.SIGTERM)"], capture_output=True, timeout=10)
        self.assertLess(candidate.returncode, 0)
        with self.assertRaisesRegex(ValueError, "exited"):
            Q.compare_program(candidate, child())

    def test_test_names_and_statuses_match_independent_formats(self):
        self.assertEqual(Q.native_test_outcomes(encoded(native_records())), Q.rust_test_outcomes(RUST_TESTS))

    def test_should_panic_name_is_not_changed(self):
        records = native_records()
        records[0]["exit_code"] = 101
        records[0]["stderr"] = PANIC_OUTPUT.replace("'adds' (123)", "'main' (987)")
        self.assertEqual(Q.rust_test_outcomes(rust_panics()), Q.native_test_outcomes(encoded(records)))

    def test_ordinary_test_panic_cannot_pass_by_label(self):
        records = native_records()
        records[0]["exit_code"] = 101
        records[0]["stderr"] = PANIC_OUTPUT
        self.assertNotEqual(Q.native_test_outcomes(encoded(records)), Q.rust_test_outcomes(RUST_TESTS))

    def test_should_panic_test_must_actually_panic(self):
        self.assertNotEqual(Q.native_test_outcomes(encoded(native_records())), Q.rust_test_outcomes(rust_panics()))

    def test_should_panic_wrong_payload_or_location_cannot_pass_by_label(self):
        for output in (PANIC_OUTPUT.replace("boom", "different panic"), PANIC_OUTPUT.replace(":5:7:", ":8:9:")):
            records = native_records()
            records[0].update(exit_code=101, stderr=output)
            self.assertNotEqual(Q.native_test_outcomes(encoded(records)), Q.rust_test_outcomes(rust_panics()))

    def test_stdout_substring_and_ignored_output_cannot_hide_a_wrong_panic(self):
        records = native_records()
        records[0].update(exit_code=101, stdout="boom", stderr=PANIC_OUTPUT.replace("boom", "different panic"))
        with self.assertRaisesRegex(ValueError, "unexpected captured output"):
            Q.native_test_outcomes(encoded(records))
        for index in (0, 1):
            for stream in ("stdout", "stderr"):
                records = native_records()
                records[index][stream] = "unmatched output"
                with self.subTest(index=index, stream=stream), self.assertRaisesRegex(ValueError, "unexpected captured output"):
                    Q.native_test_outcomes(encoded(records))

    def test_expected_panic_requires_observed_output_in_both_formats(self):
        oracle = RUST_TESTS.replace(b"test adds ... ok", b"test adds - should panic ... ok")
        with self.assertRaisesRegex(ValueError, "panic output"):
            Q.rust_test_outcomes(oracle)
        records = native_records()
        records[0]["exit_code"] = 101
        with self.assertRaisesRegex(ValueError, "panic output"):
            Q.native_test_outcomes(encoded(records))

    def test_only_thread_and_first_panic_note_are_normalized(self):
        second_panic = PANIC_OUTPUT.split("note:")[0].replace("'adds' (123)", "'main' (987)")
        self.assertEqual(Q.panic_signature(PANIC_OUTPUT), Q.panic_signature(second_panic))
        for invalid in ("", "boom", PANIC_OUTPUT + "another message\n", PANIC_OUTPUT + PANIC_OUTPUT):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                Q.panic_signature(invalid)

    def test_ignored_test_panic_contract_does_not_claim_execution(self):
        oracle = RUST_TESTS.replace(b"test slow ... ignored", b"test slow - should panic ... ignored")
        self.assertEqual(Q.native_test_outcomes(encoded(native_records())), Q.rust_test_outcomes(oracle))

    def test_empty_duplicate_and_incomplete_rust_results_are_rejected(self):
        for text in (b"", RUST_TESTS.replace(b"test adds ... ok\n", b""),
                     RUST_TESTS.replace(b"test adds ... ok\n", b"test adds ... ok\ntest adds ... ok\n"),
                     RUST_TESTS.replace(b"0 filtered out", b"1 filtered out"),
                     RUST_TESTS.replace(b"1 passed", b"0 passed")):
            with self.subTest(text=text), self.assertRaises(ValueError):
                Q.rust_test_outcomes(text)

    def test_empty_duplicate_and_inconsistent_native_results_are_rejected(self):
        rows = native_records()
        variants = [[], rows[:2], [rows[-1]], [rows[0], *rows], [*rows, rows[-1]]]
        for field, value in [("total", 0), ("passed", 0), ("ignored", 0), ("failed", 1), ("timed_out", 1), ("filtered_out", 1), ("build", "failed")]:
            variant = native_records()
            variant[-1][field] = value
            variants.append(variant)
        for records in variants:
            with self.subTest(records=records), self.assertRaises(ValueError):
                Q.native_test_outcomes(encoded(records))

    def test_missing_or_added_test_changes_the_compared_result(self):
        variant = native_records()
        variant[0]["name"] = "different_test"
        self.assertNotEqual(Q.native_test_outcomes(encoded(variant)), Q.rust_test_outcomes(RUST_TESTS))

    def test_malformed_success_counts_are_not_numbers(self):
        for field in ("total", "passed", "failed", "timed_out", "ignored", "filtered_out"):
            for value in (False, True, 0.0, "0", -1, None):
                records = native_records()
                records[-1][field] = value
                with self.subTest(field=field, value=value), self.assertRaisesRegex(ValueError, "nonnegative integers"):
                    Q.native_test_outcomes(encoded(records))

    def test_unknown_and_nonobject_result_records_are_rejected(self):
        for record in ({"schema": 1, "kind": "unreviewed"}, {"schema": True, "kind": "test"}, [], None):
            with self.subTest(record=record), self.assertRaises(ValueError):
                Q.native_test_outcomes(encoded([record, *native_records()]))

    def test_duplicate_json_fields_cannot_overwrite_a_failure(self):
        original = encoded(native_records())
        for before, after in ((b'"status": "pass"', b'"status": "fail", "status": "pass"'),
                              (b'"exit_code": 0', b'"exit_code": 101, "exit_code": 0'),
                              (b'"failed": 0', b'"failed": 1, "failed": 0')):
            with self.subTest(field=before), self.assertRaisesRegex(ValueError, "duplicate"):
                Q.native_test_outcomes(original.replace(before, after))

    def test_passing_label_cannot_hide_a_signal_or_invalid_exit(self):
        for value in (None, False, True, -15, 1, 101.0, "0"):
            records = native_records()
            records[0]["exit_code"] = value
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "signalled exit"):
                Q.native_test_outcomes(encoded(records))

    def test_every_supported_probe_is_selected_including_print_only(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "print.rs").write_text('// expect: ok\nfn main(){println!("42");}\n')
            (directory / "tests.rs").write_text('// expect: ok\n// command: test\nfn main(){}\n')
            (directory / "unsupported.rs").write_text('// expect: L1234\nfn main(){}\n')
            self.assertEqual([(path.name, command) for path, command in Q.supported_probes(directory)], [("print.rs", "run"), ("tests.rs", "test")])
            (directory / "new.rs").write_text('// expect: ok\n// command: unqualified\nfn main(){}\n')
            with self.assertRaisesRegex(ValueError, "unqualified"):
                Q.supported_probes(directory)

    def test_empty_inventory_cannot_pass(self):
        with tempfile.TemporaryDirectory() as temporary, self.assertRaisesRegex(ValueError, "no supported"):
            Q.supported_probes(Path(temporary))

    def test_pinned_inventory_cannot_shrink_or_reclassify_a_supported_case(self):
        source = Path(__file__).resolve().parents[1]
        self.assertEqual(len(Q.pinned_probes(source)), 18)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "upstream").mkdir()
            shutil.copy2(source / "upstream/lust.pin", root / "upstream/lust.pin")
            native = root / "compiler/native"
            shutil.copytree(source / "compiler/native/tests/capabilities", native / "tests/capabilities")
            (native / "scripts").mkdir()
            shutil.copy2(source / "compiler/native/scripts/test-capabilities.py", native / "scripts/test-capabilities.py")
            fixture = native / "tests/capabilities/attributes-test.rs"
            original = fixture.read_bytes()
            for mutation in (None, original.replace(b"// expect: ok", b"// expect: L1234"), original + b"\n"):
                if mutation is None:
                    fixture.unlink()
                else:
                    fixture.write_bytes(mutation)
                with self.subTest(mutation=mutation is None), self.assertRaisesRegex(ValueError, "fixtures differ"):
                    Q.pinned_probes(root)
                fixture.write_bytes(original)
            loader = native / "scripts/test-capabilities.py"
            loader.write_text("# empty declaration probe\n")
            with self.assertRaisesRegex(ValueError, "declaration probe differs"):
                Q.pinned_probes(root)


if __name__ == "__main__":
    unittest.main()
