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


def detailed_records(success=True, reused=False):
    values = records(success, reused)
    values[0]["version"] = 2
    emitter = {"schema": "lustr.mixed-native-emitter", "version": 1,
               "native": "b" * 16, "unit_identity": "e" * 16, "success": True,
               "elapsed_ns": len(REPORT.EMITTER_PHASES),
               "phases": [{"phase": phase, "elapsed_ns": 1, "completed": True} for phase in REPORT.EMITTER_PHASES]}
    return [values[0], values[1],
            {"phase": "native-request-setup", "start_ns": 3, "end_ns": 5, "elapsed_ns": 2},
            {"phase": "native-daemon-roundtrip", "start_ns": 5, "end_ns": 24, "elapsed_ns": 19},
            {"kind": "native-response", "parent_start_ns": 3, "roundtrip": [5, 24],
             "unit_identity": "e" * 16, "response_code": 0, "emitter_status": "reported", "emitter_records": [json.dumps(emitter)]},
            {"phase": "native-compile", "start_ns": 3, "end_ns": 25, "elapsed_ns": 22},
            {"phase": "rustc-driver", "start_ns": 2, "end_ns": 29, "elapsed_ns": 27},
            {"phase": "driver", "start_ns": 0, "end_ns": 30, "elapsed_ns": 30}, values[-1]]


def typed_records(outcome="emitted", success=None, reused=False):
    """Synthetic protocol records, not observations of compiler execution."""
    values = detailed_records(outcome != "failed" if success is None else success, reused)
    values[0]["version"] = 3
    values[4]["native_outcome"] = outcome
    if outcome == "complementary":
        values[4].update(emitter_status="unavailable-complementary", emitter_records=[])
    elif outcome == "failed":
        values[4].update(response_code=3, emitter_status="unavailable-failed-response", emitter_records=[])
    return values


class ProfileEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.profiles = self.root / "profiles"
        self.work = self.root / "qualification"
        self.profiles.mkdir()
        self.work.mkdir()
        for n, (label, success) in enumerate(REPORT.LEGACY_EXPECTED.items()):
            name = f"123-{n}.jsonl"
            write(self.profiles / name, records(success, label in {"body", "arithmetic-edit", "link-failure-recovery"}))
            (self.work / f"{label}.profiles").write_text(name + "\n")
            (self.work / f"{label}.trace").write_text("request-key=" + "c" * 16 + " retained=true\n")
            (self.work / f"{label}.status").write_text(f"exit status: {0 if success else 101} elapsed_ns=12\n")

    def test_valid_nested_phase_records(self):
        report = REPORT.summarize(self.root)
        self.assertFalse(report["paired_speed_evidence"])
        self.assertEqual(len(report["invocations"]), len(REPORT.LEGACY_EXPECTED))
        self.assertIn("columns must not be added", REPORT.render(report))
        self.assertIsNone(report["modes"]["body"]["native_emitter"])
        self.assertIn("not recorded", REPORT.render(report))
        row = next(line for line in REPORT.render(report).splitlines() if line.startswith("| cold |"))
        self.assertEqual([cell.strip() for cell in row.split("|")[1:-1]][5:7], ["not recorded", "not recorded"])

    def test_concurrent_request_concatenation_is_rejected(self):
        # The original 40d2ac3 dependency trace joined two concurrent request
        # records before either final newline. All profiles were present, but
        # the strict independent coverage check correctly rejected its framing.
        keys = ["a8635c96722e5567", "27b2ad9f5eece79d", "002200ea77fa9450"]
        names = ["123-10.jsonl", "124-10.jsonl", "125-10.jsonl"]
        for key, name in zip(keys, names):
            values = records()
            values[0]["request_key"] = key
            write(self.profiles / name, values)
        (self.work / "dependency.profiles").write_text("\n".join(names) + "\n")
        requests = [f"request-key={key} retained=false" for key in keys]
        trace = self.work / "dependency.trace"
        trace.write_text(requests[0] + requests[1] + "\n\n" + requests[2] + "\n")
        with self.assertRaisesRegex(ValueError, "compiler profile relative to independent request trace for dependency"):
            REPORT.summarize(self.root)
        trace.write_text("\n".join(requests) + "\n")
        self.assertEqual(len(REPORT.summarize(self.root)["modes"]["dependency"]["profiles"]), 3)

    def detailed(self):
        for n, (label, success) in enumerate(REPORT.LEGACY_EXPECTED.items()):
            values = detailed_records(success, label in {"body", "arithmetic-edit", "link-failure-recovery"})
            write(self.profiles / f"123-{n}.jsonl", values)
            with (self.work / f"{label}.trace").open("a") as trace:
                trace.write(REPORT.EMITTER_PREFIX + values[4]["emitter_records"][0] + "\n")

    def test_remote_durations_are_bound_without_aligned_timestamps(self):
        self.detailed()
        report = REPORT.summarize(self.root)
        result = report["invocations"]["123-1.jsonl"]
        request = result["native_requests"][0]
        self.assertEqual(request["client_request_elapsed_ns"], 22)
        self.assertEqual(request["roundtrip_elapsed_ns"], 19)
        self.assertEqual(request["protocol_unattributed_ns"], 11)
        self.assertEqual(request["emitter"]["elapsed_ns"], 8)
        self.assertNotIn("libtcc", result["phase_nanoseconds"])
        self.assertNotIn("start_ns", request["emitter"])
        self.assertEqual(report["modes"]["body"]["native_emitter"]["phase_nanoseconds"]["libtcc"], 1)
        row = next(line for line in REPORT.render(report).splitlines() if line.startswith("| cold |"))
        self.assertEqual([cell.strip() for cell in row.split("|")[1:-1]][5:7], ["0.000", "0.000"])

    def test_native_response_needs_actual_parent_and_roundtrip(self):
        for field, value, message in (("parent_start_ns", 9, "no enclosing request"),
                                      ("roundtrip", [6, 24], "no unique roundtrip")):
            with self.subTest(field=field):
                values = detailed_records()
                values[4][field] = value
                path = self.profiles / "123-0.jsonl"
                write(path, values)
                with self.assertRaisesRegex(ValueError, message):
                    REPORT.read_profile(path)

    def test_missing_or_duplicate_remote_detail_is_not_zero(self):
        for raw in ([], ["{}", "{}"]):
            with self.subTest(raw=raw):
                values = detailed_records()
                values[4]["emitter_records"] = raw
                path = self.profiles / "123-0.jsonl"
                write(path, values)
                with self.assertRaisesRegex(ValueError, "missing or duplicate native emitter"):
                    REPORT.read_profile(path)
        values = detailed_records()
        del values[4]
        write(path, values)
        with self.assertRaisesRegex(ValueError, "missing or extra native response"):
            REPORT.read_profile(path)

    def test_native_record_is_also_required_in_original_trace(self):
        self.detailed()
        (self.work / "body.trace").write_text("request-key=" + "c" * 16 + " retained=true\n")
        with self.assertRaisesRegex(ValueError, "independent diagnostic trace"):
            REPORT.summarize(self.root)

    def emitter_mutation(self, update, message):
        values = detailed_records()
        emitter = json.loads(values[4]["emitter_records"][0])
        update(emitter)
        values[4]["emitter_records"] = [json.dumps(emitter)]
        path = self.profiles / "123-0.jsonl"
        write(path, values)
        with self.assertRaisesRegex(ValueError, message):
            REPORT.read_profile(path)

    def test_remote_duration_cannot_exceed_actual_exchange(self):
        self.emitter_mutation(lambda value: value.update(elapsed_ns=20), "exceeds its roundtrip")

    def test_remote_stages_must_cover_the_reported_duration(self):
        self.emitter_mutation(lambda value: value.update(elapsed_ns=9), "do not cover its duration")

    def test_remote_build_and_unit_identity_must_match(self):
        for field in ("native", "unit_identity"):
            with self.subTest(field=field):
                self.emitter_mutation(lambda value: value.update({field: "a" * 16}), "identity mismatch")

    def test_remote_phase_order_completion_and_schema_are_checked(self):
        self.emitter_mutation(lambda value: value["phases"].reverse(), "phase order")
        self.emitter_mutation(lambda value: value["phases"][0].update(completed=False), "completion mismatch")
        self.emitter_mutation(lambda value: value.update(extra="unreviewed"), "unknown native emitter fields")
        self.emitter_mutation(lambda value: value["phases"][0].update(elapsed_ns=True), "invalid native emitter phase duration")

    def test_in_process_symbol_check_has_a_distinct_version_and_phase(self):
        self.detailed()
        for n, label in enumerate(REPORT.LEGACY_EXPECTED):
            path = self.profiles / f"123-{n}.jsonl"
            values = [json.loads(line) for line in path.read_text().splitlines()]
            old = values[4]["emitter_records"][0]
            emitter = json.loads(old)
            emitter["version"] = 2
            emitter["phases"][4]["phase"] = "symbol-check"
            new = json.dumps(emitter)
            values[4]["emitter_records"] = [new]
            write(path, values)
            trace = self.work / f"{label}.trace"
            trace.write_text(trace.read_text().replace(old, new))
        report = REPORT.summarize(self.root)
        rendered = REPORT.render(report)
        self.assertIn("| symbol-check |", rendered)
        self.assertNotIn("| nm |", rendered)
        self.assertIn("with no symbol-tool child", rendered)
        self.assertEqual(report["modes"]["body"]["native_emitter"]["phase_nanoseconds"]["symbol-check"], 1)

    def test_historical_nm_and_current_symbol_phases_cannot_be_relabeled(self):
        self.emitter_mutation(lambda value: value.update(version=2), "phase order")
        self.emitter_mutation(lambda value: value["phases"][4].update(phase="symbol-check"), "phase order")
        self.emitter_mutation(lambda value: value.update(version=3), "unknown native emitter schema")

    def test_one_campaign_cannot_mix_symbol_check_implementations(self):
        self.detailed()
        path = self.profiles / "123-1.jsonl"
        values = [json.loads(line) for line in path.read_text().splitlines()]
        old = values[4]["emitter_records"][0]
        emitter = json.loads(old)
        emitter["version"] = 2
        emitter["phases"][4]["phase"] = "symbol-check"
        new = json.dumps(emitter)
        values[4]["emitter_records"] = [new]
        write(path, values)
        trace = self.work / "body.trace"
        trace.write_text(trace.read_text().replace(old, new))
        with self.assertRaisesRegex(ValueError, "mixed native emitter versions"):
            REPORT.summarize(self.root)

    def test_failed_native_response_has_unavailable_detail_not_zero(self):
        values = detailed_records()
        values[4].update(response_code=1, emitter_status="unavailable-failed-response", emitter_records=[])
        path = self.profiles / "123-0.jsonl"
        write(path, values)
        response = REPORT.read_profile(path)["native_requests"][0]
        self.assertEqual(response["response_code"], 1)
        self.assertEqual(response["client_request_elapsed_ns"], 22)
        self.assertIsNone(response["emitter"])
        self.assertIsNone(response["protocol_unattributed_ns"])
        # Even a supplied partial timing record cannot qualify a failed native
        # response: timing text must never decorate the guard's error channel.
        values[4]["emitter_records"] = detailed_records()[4]["emitter_records"]
        write(path, values)
        with self.assertRaisesRegex(ValueError, "error diagnostics undecorated"):
            REPORT.read_profile(path)
        values[4].update(response_code=0, emitter_records=[])
        write(path, values)
        with self.assertRaisesRegex(ValueError, "successful native response requires"):
            REPORT.read_profile(path)

    def test_failed_response_denominators_remain_visible(self):
        self.detailed()
        values = detailed_records()
        values[4].update(response_code=1, emitter_status="unavailable-failed-response", emitter_records=[])
        path = self.profiles / "123-0.jsonl"
        write(path, values)
        (self.work / "cold.trace").write_text("request-key=" + "c" * 16 + " retained=true\n")
        report = REPORT.summarize(self.root)
        native = report["modes"]["cold"]["native_emitter"]
        self.assertEqual((native["response_count"], native["measured_count"], native["unavailable_failed_count"]), (1, 0, 1))
        self.assertIsNone(native["elapsed_ns"])
        self.assertIsNone(native["protocol_unattributed_ns"])
        self.assertEqual(native["phase_nanoseconds"], {})
        self.assertIn("| cold | 1 / 0 / 1 | unavailable |", REPORT.render(report))

        values = detailed_records()
        values[-3].update(end_ns=59, elapsed_ns=57)
        values[-2].update(end_ns=60, elapsed_ns=60)
        values[6:6] = [
            {"phase": "native-request-setup", "start_ns": 26, "end_ns": 28, "elapsed_ns": 2},
            {"phase": "native-daemon-roundtrip", "start_ns": 28, "end_ns": 47, "elapsed_ns": 19},
            {"kind": "native-response", "parent_start_ns": 26, "roundtrip": [28, 47], "unit_identity": "f" * 16,
             "response_code": 1, "emitter_status": "unavailable-failed-response", "emitter_records": []},
            {"phase": "native-compile", "start_ns": 26, "end_ns": 48, "elapsed_ns": 22},
        ]
        write(path, values)
        with (self.work / "cold.trace").open("a") as trace:
            trace.write(REPORT.EMITTER_PREFIX + values[4]["emitter_records"][0] + "\n")
        report = REPORT.summarize(self.root)
        native = report["modes"]["cold"]["native_emitter"]
        self.assertEqual((native["response_count"], native["measured_count"], native["unavailable_failed_count"]), (2, 1, 1))
        self.assertEqual(native["elapsed_ns"], 8)
        self.assertEqual(native["protocol_unattributed_ns"], 11)
        self.assertIn("| cold | 2 / 1 / 1 |", REPORT.render(report))

    def test_malformed_remote_json_is_rejected(self):
        values = detailed_records()
        values[4]["emitter_records"] = ["{"]
        path = self.profiles / "123-0.jsonl"
        write(path, values)
        with self.assertRaises(json.JSONDecodeError):
            REPORT.read_profile(path)

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

    def typed_campaign(self):
        for directory in [self.work, self.profiles]:
            for path in directory.iterdir():
                path.unlink()
        for n, (label, success) in enumerate(REPORT.EXPECTED.items()):
            if label in {"native-output-recovery", "retained-recipe-recovery"}:
                values = records(success, True)
                values[0]["version"] = 3
                values[2]["phase"] = "relink"
            else:
                outcome = "failed" if label == "native-output-failure" else "emitted"
                values = typed_records(outcome, success, label in REPORT.RETAINED_SUCCESS)
                if label == "retained-recipe-failure":
                    values.insert(-3, {"phase": "relink", "start_ns": 26, "end_ns": 27, "elapsed_ns": 1})
            name = f"123-{n}.jsonl"
            write(self.profiles / name, values)
            (self.work / f"{label}.profiles").write_text(name + "\n")
            trace = "request-key=" + "c" * 16 + " retained=true\n"
            for record in values:
                for emitter in record.get("emitter_records", []):
                    trace += REPORT.EMITTER_PREFIX + emitter + "\n"
            (self.work / f"{label}.trace").write_text(trace)
            (self.work / f"{label}.status").write_text(f"exit status: {0 if success else 101} elapsed_ns=30\n")

    def profile_for(self, label):
        name = (self.work / f"{label}.profiles").read_text().strip()
        path = self.profiles / name
        return path, [json.loads(line) for line in path.read_text().splitlines()]

    def test_typed_emitted_response_keeps_complete_detail_requirement(self):
        path = self.profiles / "typed.jsonl"
        values = typed_records()
        write(path, values)
        response = REPORT.read_profile(path)["native_requests"][0]
        self.assertEqual(response["native_outcome"], "emitted")
        self.assertEqual(response["protocol_unattributed_ns"], 11)
        values[4]["emitter_records"] = []
        write(path, values)
        with self.assertRaisesRegex(ValueError, "missing or duplicate native emitter"):
            REPORT.read_profile(path)

    def test_typed_complement_is_separate_from_failure_and_emission(self):
        self.typed_campaign()
        path, _ = self.profile_for("cold")
        write(path, typed_records("complementary"))
        (self.work / "cold.trace").write_text("request-key=" + "c" * 16 + " retained=true\n")
        report = REPORT.summarize(self.root)
        native = report["modes"]["cold"]["native_emitter"]
        self.assertEqual((native["response_count"], native["measured_count"],
                          native["unavailable_complementary_count"], native["unavailable_failed_count"]), (1, 0, 1, 0))
        self.assertIsNone(native["elapsed_ns"])
        self.assertIsNone(native["protocol_unattributed_ns"])
        self.assertEqual(native["phase_nanoseconds"], {})
        self.assertIn("| cold | 1 / 0 / 1 / 0 | not emitted |", REPORT.render(report))
        self.assertEqual(len(report["modes"]), 19)

    def test_typed_failure_cannot_become_a_successful_compiler(self):
        values = typed_records("failed")
        path = self.profiles / "typed.jsonl"
        write(path, values)
        response = REPORT.read_profile(path)["native_requests"][0]
        self.assertEqual(response["native_outcome"], "failed")
        self.assertEqual(response["response_code"], 3)
        self.assertIsNone(response["emitter"])
        values[-1]["success"] = True
        write(path, values)
        with self.assertRaisesRegex(ValueError, "failed native work became a successful compiler"):
            REPORT.read_profile(path)

    def test_typed_outcome_code_and_detail_must_agree(self):
        for outcome, changes, error in [
            ("emitted", {"response_code": 1}, "outcome and response code disagree"),
            ("complementary", {"response_code": 1}, "outcome and response code disagree"),
            ("failed", {"response_code": 0}, "outcome and response code disagree"),
            ("emitted", {"emitter_status": "unavailable-complementary"}, "requires reported emitter detail"),
            ("complementary", {"emitter_status": "reported"}, "must have no emitted record"),
            ("complementary", {"emitter_records": detailed_records()[4]["emitter_records"]}, "must have no emitted record"),
            ("failed", {"emitter_records": detailed_records()[4]["emitter_records"]}, "error diagnostics undecorated"),
        ]:
            with self.subTest(outcome=outcome, changes=changes):
                values = typed_records(outcome)
                values[4].update(changes)
                path = self.profiles / "typed.jsonl"
                write(path, values)
                with self.assertRaisesRegex(ValueError, error):
                    REPORT.read_profile(path)

    def test_compile_outcome_must_be_present_known_and_not_checked(self):
        path = self.profiles / "typed.jsonl"
        for outcome in [None, "checked", "unknown", 0]:
            with self.subTest(outcome=outcome):
                values = typed_records()
                values[4]["native_outcome"] = outcome
                write(path, values)
                with self.assertRaisesRegex(ValueError, "requires an explicit"):
                    REPORT.read_profile(path)
        values = typed_records()
        del values[4]["native_outcome"]
        write(path, values)
        with self.assertRaisesRegex(ValueError, "unknown native response fields"):
            REPORT.read_profile(path)

    def test_legacy_response_cannot_claim_a_new_typed_outcome(self):
        path = self.profiles / "typed.jsonl"
        values = typed_records()
        values[0]["version"] = 2
        write(path, values)
        with self.assertRaisesRegex(ValueError, "unknown native response fields"):
            REPORT.read_profile(path)

    def test_current_campaign_requires_both_failures_and_retained_recovery(self):
        self.typed_campaign()
        report = REPORT.summarize(self.root)
        self.assertEqual(len(report["modes"]), 19)
        self.assertIn("| native-output-failure | 1 / 0 / 0 / 1 | unavailable |", REPORT.render(report))
        for label in ["native-output-recovery", "retained-recipe-recovery"]:
            with self.subTest(label=label):
                self.typed_campaign()
                path, values = self.profile_for(label)
                values[-1]["reused"] = False
                write(path, values)
                with self.assertRaisesRegex(ValueError, "lost retained execution"):
                    REPORT.summarize(self.root)
        self.typed_campaign()
        (self.work / "native-output-failure.profiles").unlink()
        with self.assertRaisesRegex(ValueError, "missing or unknown qualification profile labels"):
            REPORT.summarize(self.root)

    def test_new_failure_cases_need_their_actual_failed_phase(self):
        self.typed_campaign()
        path, values = self.profile_for("native-output-failure")
        replacement = typed_records("emitted", success=False)
        write(path, replacement)
        with (self.work / "native-output-failure.trace").open("a") as trace:
            trace.write(REPORT.EMITTER_PREFIX + replacement[4]["emitter_records"][0] + "\n")
        with self.assertRaisesRegex(ValueError, "lacks an actual failed native response"):
            REPORT.summarize(self.root)
        self.typed_campaign()
        path, values = self.profile_for("retained-recipe-failure")
        values = [record for record in values if record.get("phase") != "relink"]
        write(path, values)
        with self.assertRaisesRegex(ValueError, "followed by relink"):
            REPORT.summarize(self.root)

    def test_current_schema_cannot_claim_the_shorter_legacy_scope(self):
        self.typed_campaign()
        for label in set(REPORT.EXPECTED) - set(REPORT.LEGACY_EXPECTED):
            path, _ = self.profile_for(label)
            path.unlink()
            (self.work / f"{label}.profiles").unlink()
        with self.assertRaisesRegex(ValueError, "labels do not match the profile schema"):
            REPORT.summarize(self.root)

    def test_legacy_schema_cannot_claim_new_failure_qualification(self):
        self.typed_campaign()
        for path in self.profiles.iterdir():
            values = [json.loads(line) for line in path.read_text().splitlines()]
            values[0]["version"] = 2
            for value in values:
                value.pop("native_outcome", None)
            write(path, values)
        with self.assertRaisesRegex(ValueError, "labels do not match the profile schema"):
            REPORT.summarize(self.root)


if __name__ == "__main__":
    unittest.main()
