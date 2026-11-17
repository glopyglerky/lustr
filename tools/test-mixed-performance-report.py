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


def native_record(row, meta):
    """Synthetic command/output contract; these bytes never claim execution."""
    lane, phase = row["lane"], row["phase"]
    source = next(iter(row["source_sha256"]))
    encode = lambda values: [{"Unix": list(value.encode())} for value in values]
    executable = meta["native_lane_executables"][lane]
    output = f"/synthetic/state/{lane}/{phase.removesuffix('-runtime')}-{row['sample']}"
    stdout, stderr = b"synthetic output\n", b"synthetic output\n"
    if phase.endswith("-runtime"):
        argv = [output]
        stdout, stderr = b"59\n", b""
    else:
        check = phase in {"check", "check-no-op", "check-body-edit", "rejection", "reference-probe-check", "reference-probe-invalid"}
        test = phase in {"compile-plus-test", "reference-probe-test", "reference-probe-failed-test"}
        operation = "test" if test else "check" if check else "build"
        if lane == "rustc":
            argv = ([meta["native_rustc_reference"]["test_driver"], "--native-rustc-test", executable, source, output] if test else
                    [executable, source, *reporter.RUSTC_FLAGS, "--emit=metadata" if check else "--emit=link", "-o", output])
        else:
            argv = [executable, operation, source]
            argv += ["--json"] if test else reporter.NATIVE_FLAGS if check else ["-o", output, *reporter.NATIVE_FLAGS]
        if test:
            passed = phase != "reference-probe-failed-test"
            if lane == "rustc":
                stdout = (f"test scalar_contract ... {'ok' if passed else 'FAILED'}\n"
                          f"test result: {'ok' if passed else 'FAILED'}. {int(passed)} passed; {int(not passed)} failed; "
                          "0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n").encode()
                child_commands = [[executable, source, *reporter.RUSTC_FLAGS, "--test", "-o", output],
                                  [output, "--test-threads=1", "--color", "never"]]
                stderr = b"".join((reporter.TEST_RECEIPT + json.dumps(dict(phase=p, argv=encode(a), exit=c, signal=None)) + "\n").encode()
                                  for p, a, c in zip(("compile", "execute"), child_commands, (0, 0 if passed else 101)))
            else:
                stdout = (json.dumps(dict(kind="summary", build="ok", total=1, passed=int(passed), failed=int(not passed),
                                          timed_out=0, ignored=0, filtered_out=0)) + "\n").encode()
    row["argv"] = encode(argv)
    return {"stdout": stdout, "stderr": stderr}


def fixture(root, schema=1):
    n, seed = (6 if schema == 1 else 12), 19
    lanes_by_family, comparisons = reporter.campaign_layout(schema)
    meta = dict(schema=schema, pairs=n, seed=seed, pristine_revision="a" * 40,
                pristine_tree="b" * 40, compiler_sha256={"synthetic/compiler": "c" * 64},
                profiles="synthetic", rust_reference_version="synthetic", cold_scope="synthetic",
                warm_scope="synthetic", timing_scope="synthetic", scope="synthetic",
                inference_limits="synthetic", order_method="randomized-cyclic-blocks-v1",
                performance_admission="not granted by this runner")
    if schema == 2:
        meta["native_rustc_reference"] = reporter.native_reference_contract("/synthetic/package/rustc-reference", "/synthetic/runner")
        meta["native_lane_executables"] = {"pristine": "/synthetic/pristine/lust", "integrated": "/synthetic/integrated/lust",
                                           "entry": "/synthetic/package/lustr", "rustc": "/synthetic/package/rustc-reference"}
        meta["compiler_sha256"] = {path: "c" * 64 for path in ("/synthetic/pristine/lust", "/synthetic/integrated/lust",
                                      "/synthetic/package/lustr", "/synthetic/package/rustc-reference", "/synthetic/runner")}
    meta["compiler_sha256"].update({"/synthetic/package/cargo": "c" * 64, "/synthetic/package/lustr": "c" * 64})
    command_meta = dict(meta, native_lane_executables={
        "pristine": "/synthetic/pristine/lust", "integrated": "/synthetic/integrated/lust",
        "entry": "/synthetic/package/lustr", "rustc": "/synthetic/package/rustc-reference"})
    rows = [dict(kind="identities", report=copy.deepcopy(dict(meta, status="failed", summaries=None)))]
    durations = {}
    folder = root / "lustr-paired-measurements"
    folder.mkdir()
    def store(row, streams):
        for stream in ("stdout", "stderr"):
            name = f"{len(rows):05}-{row['family']}-{row['lane']}-{row['phase']}-{row['sample']}.{stream}"
            data = streams[stream]
            (folder / name).write_bytes(data)
            row[stream] = {"Unix": list(name.encode())}
            row[stream + "_bytes"] = len(data)
            row[stream + "_sha256"] = reporter.sha256(data)
        rows.append(row)
    for family, lanes in lanes_by_family.items():
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
                               source_sha256={f"/synthetic/source/{'app/src/' if family == 'cargo' else ''}main.rs": "d" * 64},
                               source_after_sha256={f"/synthetic/source/{'app/src/' if family == 'cargo' else ''}main.rs": "d" * 64})
                    if family == "native":
                        streams = native_record(row, command_meta)
                    else:
                        operation = "check" if phase.startswith("check") else "test" if phase == "compile-plus-test" else "build"
                        source = f"/synthetic/cargo-{i:03}/source"
                        state = f"/synthetic/cargo-{i:03}/lane{lane_index}"
                        target = state + "/target"
                        row["source_sha256"] = {source + "/" + path: "d" * 64 for path in
                                                ("Cargo.lock", "Cargo.toml", "app/Cargo.toml", "app/src/main.rs",
                                                 "fixture-macro/Cargo.toml", "fixture-macro/src/lib.rs",
                                                 "portable/Cargo.toml", "portable/src/lib.rs")}
                        row["source_after_sha256"] = dict(row["source_sha256"])
                        row["cwd"] = source
                        executable = "/synthetic/package/" + ("cargo" if lane == "reference" else "lustr")
                        args = [executable, operation, "--locked", "--offline", "--jobs", "2", "-p", "mixed-app",
                                "--bin", "mixed-app", "--target-dir", target, "--config", f'build.build-dir="{state}/intermediates"']
                        if operation == "test":
                            args += ["--", "--test-threads=1"]
                        row["argv"] = [{"Unix": list(value.encode())} for value in
                                       args]
                        streams = {s: b"synthetic output\n" for s in ("stdout", "stderr")}
                        if operation == "test":
                            streams["stdout"] = (b"\nrunning 1 test\ntest scalar_contract ... ok\n\n"
                                                 b"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n")
                    store(row, streams)
                    if phase in ("cold", "no-op", "body-edit", "signature-edit"):
                        args = [reporter.argument(value) for value in row["argv"]]
                        output = args[args.index("-o") + 1] if family == "native" else target + "/debug/mixed-app"
                        runtime = {key: copy.deepcopy(value) for key, value in row.items()
                                   if not key.startswith(("stdout", "stderr"))}
                        runtime.update(phase=phase + "-runtime", sample=None, seconds=.001,
                                       argv=[{"Unix": list(output.encode())}])
                        if family == "cargo":
                            runtime["source_sha256"] = runtime["source_after_sha256"] = {source + "/app/src/main.rs": "d" * 64}
                        store(runtime, {"stdout": b"57\n" if phase == "body-edit" else b"59\n", "stderr": b""})
            rows.append(dict(kind="daemon-exits", scope=f"{family}-{i}", successful_exits=3 if family == "native" else 1))
    if schema == 2:
        for phase, code in reporter.REFERENCE_PROBES.items():
            for lane in ("entry", "rustc"):
                row = dict(kind="command", family="native", lane=lane, phase=phase, sample=None,
                           seconds=.01, expected_exit=code, exit=code, timed_out=False, signal=None,
                           source_sha256={"/synthetic/source/main.rs": "d" * 64},
                           source_after_sha256={"/synthetic/source/main.rs": "d" * 64})
                store(row, native_record(row, meta))
        rows.append(dict(kind="daemon-exits", scope="native-reference-probes", successful_exits=1))
    rows += [dict(kind="daemon-exits", scope="native-probes", successful_exits=3),
             dict(kind="daemon-exits", scope="mixed-probes", successful_exits=1)]
    rows += [dict(kind="native-retained-check", lane=lane, route="bodies-without-body-work")
             for lane in reporter.LANES["native"]]
    summaries = []
    for family, baseline, candidate in comparisons:
        for phase in reporter.PHASES:
            pairs = [(durations[family, baseline, phase, i], durations[family, candidate, phase, i]) for i in range(n)]
            analysis = reporter.summarize(pairs, 100, seed)
            low, high = analysis["bootstrap_95"]
            summaries.append(dict(family=family, baseline=baseline, candidate=candidate,
                                  phase=phase, analysis=analysis, ratio_direction=reporter.DIRECTION,
                                  observation="faster-on-this-corpus" if low > 1 else "slower-on-this-corpus" if high < 1 else "inconclusive"))
    (root / "lustr-source-revision.txt").write_text("e" * 40 + "\n" + "f" * 40 + "\n")
    return dict(meta, status="completed", records=rows, summaries=summaries)


class FixtureTestCase(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.data = fixture(self.root, getattr(self, "schema", 1))
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


class ExecutionContractTests:
    def test_recorded_order_must_match_executed_lane_order(self):
        # Permutations stay balanced, but no longer describe the commands.
        for row in self.data["records"]:
            if row["kind"] == "order":
                row["lanes"].reverse()
        self.rejected()

    def test_order_record_must_precede_its_commands(self):
        rows = self.data["records"]
        order = next(row for row in rows if row["kind"] == "order")
        rows.remove(order)
        rows.append(order)
        self.rejected()

    def test_measured_phase_order_cannot_change(self):
        rows = self.data["records"]
        first = next(i for i, row in enumerate(rows) if row.get("phase") == "check")
        later = next(i for i, row in enumerate(rows) if row.get("phase") == "check-no-op")
        rows[first], rows[later] = rows[later], rows[first]
        self.rejected()

    def test_every_measured_build_requires_its_runtime(self):
        original = copy.deepcopy(self.data)
        for family in ("native", "cargo"):
            with self.subTest(family=family):
                self.data = copy.deepcopy(original)
                rows = self.data["records"]
                rows.remove(next(row for row in rows if row.get("family") == family and row.get("phase") == "cold-runtime"))
                self.rejected()

    def test_runtime_cannot_be_moved_after_another_command(self):
        rows = self.data["records"]
        runtime = next(row for row in rows if row.get("phase") == "cold-runtime")
        rows.remove(runtime)
        rows.insert(rows.index(next(row for row in rows if row.get("phase") == "check")), runtime)
        self.rejected()

    def test_measured_runtime_must_use_its_own_artifact_and_source(self):
        original = copy.deepcopy(self.data)
        for family in ("native", "cargo"):
            for mutation in ("artifact", "source"):
                with self.subTest(family=family, mutation=mutation):
                    self.data = copy.deepcopy(original)
                    row = next(row for row in self.data["records"]
                               if row.get("family") == family and row.get("phase") == "cold-runtime")
                    if mutation == "artifact":
                        row["argv"] = [{"Unix": list(b"/synthetic/unrelated-program")}]
                    else:
                        row["source_sha256"] = row["source_after_sha256"] = {"/other/source.rs": "d" * 64}
                    self.rejected()

    def test_measured_runtime_output_is_checked_after_rehash(self):
        original = copy.deepcopy(self.data)
        for family in ("native", "cargo"):
            with self.subTest(family=family):
                self.data = copy.deepcopy(original)
                row = next(row for row in self.data["records"]
                           if row.get("family") == family and row.get("phase") == "cold-runtime")
                data = b"wrong\n"
                path = self.root / "lustr-paired-measurements" / reporter.output_name(row["stdout"])
                original_bytes = path.read_bytes()
                try:
                    path.write_bytes(data)
                    row["stdout_bytes"], row["stdout_sha256"] = len(data), reporter.sha256(data)
                    self.rejected()
                finally:
                    path.write_bytes(original_bytes)

    def test_duplicate_runtime_with_distinct_outputs_is_rejected(self):
        original = copy.deepcopy(self.data)
        for family in ("native", "cargo"):
            for placement in ("adjacent", "after-samples"):
                with self.subTest(family=family, placement=placement):
                    self.data = copy.deepcopy(original)
                    rows = self.data["records"]
                    index = next(i for i, row in enumerate(rows)
                                 if row.get("family") == family and row.get("phase") == "cold-runtime")
                    duplicate = copy.deepcopy(rows[index])
                    for stream in ("stdout", "stderr"):
                        name = reporter.output_name(duplicate[stream])
                        folder = self.root / "lustr-paired-measurements"
                        (folder / ("duplicate-" + name)).write_bytes((folder / name).read_bytes())
                        duplicate[stream] = {"Unix": list(("duplicate-" + name).encode())}
                    rows.insert(index + 1 if placement == "adjacent" else len(rows), duplicate)
                    self.rejected()

    def test_daemon_receipts_cannot_be_missing_or_duplicated(self):
        original = copy.deepcopy(self.data)
        for mutation in ("missing-one", "missing-all", "duplicate"):
            with self.subTest(mutation=mutation):
                self.data = copy.deepcopy(original)
                rows = self.data["records"]
                receipt = next(row for row in rows if row["kind"] == "daemon-exits")
                if mutation == "missing-one":
                    rows.remove(receipt)
                elif mutation == "missing-all":
                    self.data["records"] = [row for row in rows if row["kind"] != "daemon-exits"]
                else:
                    rows.append(copy.deepcopy(receipt))
                self.rejected()

    def test_daemon_counts_must_meet_the_executed_contract(self):
        original = copy.deepcopy(self.data)
        for scope, count in (("native-probes", 2), ("mixed-probes", 0), ("native-0", 2),
                             ("cargo-0", 0), ("native-0", True), ("cargo-0", -1)):
            with self.subTest(scope=scope, count=count):
                self.data = copy.deepcopy(original)
                next(row for row in self.data["records"] if row.get("scope") == scope)["successful_exits"] = count
                self.rejected()

    def test_sample_daemon_receipt_must_follow_all_sample_work(self):
        rows = self.data["records"]
        receipt = next(row for row in rows if row.get("scope") == "native-0")
        rows.remove(receipt)
        rows.insert(rows.index(next(row for row in rows if row.get("phase") == "check")), receipt)
        self.rejected()

    def test_unrecognized_daemon_scope_is_rejected(self):
        receipt = next(row for row in self.data["records"] if row["kind"] == "daemon-exits")
        receipt["scope"] = "unrecorded-probe"
        self.rejected()


class CargoCommandTests:
    def cargo_command(self, lane="mixed", phase="compile-plus-test"):
        return next(row for row in self.data["records"] if row.get("kind") == "command"
                    and row.get("family") == "cargo" and row.get("lane") == lane and row.get("phase") == phase)

    def test_cargo_lane_cannot_run_reference_under_mixed_label(self):
        replacement = copy.deepcopy(self.cargo_command("reference", "cold")["argv"][0])
        for row in self.data["records"]:
            if row.get("family") == "cargo" and row.get("lane") == "mixed" and row.get("phase") in reporter.PHASES:
                row["argv"][0] = replacement
        self.rejected()

    def test_cargo_compile_without_test_execution_is_rejected(self):
        row = self.cargo_command()
        row["argv"][1] = {"Unix": list(b"build")}
        for stream in ("stdout", "stderr"):
            path = self.root / "lustr-paired-measurements" / reporter.output_name(row[stream])
            path.write_bytes(b"")
            row[stream + "_bytes"], row[stream + "_sha256"] = 0, reporter.sha256(b"")
        self.rejected()

    def test_cargo_compiler_identities_must_be_unambiguous(self):
        original = copy.deepcopy(self.data)
        for mutation in ("missing-reference", "missing-entry", "ambiguous-reference", "different-entry"):
            with self.subTest(mutation=mutation):
                self.data = copy.deepcopy(original)
                hashes = self.data["compiler_sha256"]
                if mutation.startswith("missing"):
                    del hashes["/synthetic/package/" + ("cargo" if mutation == "missing-reference" else "lustr")]
                elif mutation == "ambiguous-reference":
                    hashes["/unrelated/cargo"] = "c" * 64
                else:
                    del hashes["/synthetic/package/lustr"]
                    hashes["/unrelated/lustr"] = "c" * 64
                self.data["records"][0]["report"]["compiler_sha256"] = copy.deepcopy(hashes)
                self.rejected()

    def test_cargo_operation_and_flags_match_the_measured_phase(self):
        original = copy.deepcopy(self.data)
        mutations = (("check", "check", "build"), ("check-no-op", "check", "build"),
                     ("rejection", "build", "check"), ("compile-plus-test", "test", "build"),
                     ("cold", "--locked", "--release"), ("cold", "--offline", "--frozen"),
                     ("cold", "2", "1"), ("cold", "mixed-app", "different-app"),
                     ("compile-plus-test", "--test-threads=1", "--ignored"))
        for phase, old, new in mutations:
            with self.subTest(phase=phase, flag=old):
                self.data = copy.deepcopy(original)
                row = self.cargo_command(phase=phase)
                row["argv"] = [{"Unix": list((new if reporter.argument(value) == old else reporter.argument(value)).encode())}
                               for value in row["argv"]]
                self.rejected()
        for extra in (("--no-run",), ("--config", "profile.dev.debug=2"), ("--", "--skip", "scalar_contract")):
            with self.subTest(extra=extra):
                self.data = copy.deepcopy(original)
                self.cargo_command()["argv"] += [{"Unix": list(value.encode())} for value in extra]
                self.rejected()

    def test_cargo_caches_cannot_be_shared_or_replaced(self):
        original = copy.deepcopy(self.data)
        for flag in ("--target-dir", "--config"):
            with self.subTest(flag=flag):
                self.data = copy.deepcopy(original)
                row = self.cargo_command(phase="check-no-op")
                reference = self.cargo_command("reference", "check-no-op")
                position = [reporter.argument(value) for value in row["argv"]].index(flag) + 1
                row["argv"][position] = copy.deepcopy(reference["argv"][position])
                self.rejected()

    def test_cargo_working_directory_must_select_the_recorded_workspace(self):
        self.cargo_command(phase="check-no-op")["cwd"] = "/synthetic/unrelated-workspace"
        self.rejected()

    def test_cargo_test_receipt_requires_exactly_one_executed_scalar_test(self):
        row = self.cargo_command()
        path = self.root / "lustr-paired-measurements" / reporter.output_name(row["stdout"])
        original = path.read_bytes()
        replacements = (b"", b"Finished test profile\n",
                        original.replace(b"1 passed", b"0 passed"),
                        original.replace(b"0 ignored", b"1 ignored"),
                        original.replace(b"0 filtered out", b"1 filtered out"),
                        original.replace(b"scalar_contract", b"unrelated_test"),
                        original.replace(b"running 1 test\n", b""),
                        original + b"test scalar_contract ... ok\n", original + original)
        for output in replacements:
            with self.subTest(output=output):
                path.write_bytes(output)
                row["stdout_bytes"], row["stdout_sha256"] = len(output), reporter.sha256(output)
                self.rejected()


class ReportTests(CargoCommandTests, ExecutionContractTests, FixtureTestCase):
    def test_complete_report_and_markdown(self):
        result = self.verify()
        self.assertEqual(result["verified"]["comparisons"], 27)
        self.assertEqual(result["verified"]["timed_commands"], 270)
        self.assertEqual(result["verified"]["command_outputs"], 780)
        self.assertEqual(len(result["timing_samples"]), 45)
        text = reporter.markdown(result)
        self.assertEqual(text.count("| rejection |"), 3)
        self.assertIn("not equivalence tests", text)

    def test_csv_roundtrip(self):
        result = self.verify()
        rows = list(csv.reader(io.StringIO(reporter.csv_samples(result))))
        self.assertEqual(rows[0][1:], [f"pair_{i}_seconds" for i in range(6)])
        self.assertEqual({x[0]: list(map(float, x[1:])) for x in rows[1:]}, result["timing_samples"])

    def host_record(self):
        value = {"schema": "lustr-benchmark-host-v1", "cpu_model": {"value": None, "unavailable_reason": "not available"}}
        raw = (json.dumps(value) + "\n").encode()
        host = {"record": value, "sha256": reporter.sha256(raw)}
        (self.root / "lustr-paired-measurements/host.json").write_bytes(raw)
        self.data["host"] = host
        self.data["records"][0]["report"]["host"] = copy.deepcopy(host)
        self.save()
        return host

    def test_host_record_is_preserved_without_inventing_historical_facts(self):
        self.assertNotIn("host", self.verify())
        host = self.host_record()
        self.assertEqual(self.verify()["host"], host)

    def test_changed_host_file_is_rejected(self):
        self.host_record()
        (self.root / "lustr-paired-measurements/host.json").write_text('{}\n')
        self.rejected()

    def test_missing_host_file_is_rejected(self):
        self.host_record()
        (self.root / "lustr-paired-measurements/host.json").unlink()
        self.rejected()

    def test_changed_host_identity_is_rejected(self):
        self.host_record()
        self.data["host"]["record"]["cpu_model"]["value"] = "invented"
        self.rejected()

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


class Schema2Tests(CargoCommandTests, ExecutionContractTests, FixtureTestCase):
    schema = 2

    def command(self, lane="rustc", phase="compile-plus-test"):
        return next(row for row in self.data["records"] if row.get("kind") == "command"
                    and row.get("family") == "native" and row.get("lane") == lane and row.get("phase") == phase)

    def stream(self, row, name, data):
        path = self.root / "lustr-paired-measurements" / reporter.output_name(row[name])
        path.write_bytes(data)
        row[name + "_bytes"] = len(data)
        row[name + "_sha256"] = reporter.sha256(data)

    def test_four_native_lanes_and_exact_four_contrasts(self):
        result = self.verify()
        self.assertEqual(result["campaign_schema"], 2)
        self.assertEqual(result["verified"]["comparisons"], 36)
        self.assertEqual(result["verified"]["timed_commands"], 648)
        self.assertEqual(len(result["timing_samples"]), 54)
        self.assertEqual({(r["family"], r["baseline"], r["candidate"]) for r in result["summaries"]}, set(reporter.COMPARISONS_V2))
        text = reporter.markdown(result)
        self.assertIn("native: rustc → entry", text)
        self.assertIn("not Cargo Fresh", text)
        self.assertEqual(text.count("| rejection |"), 4)

    def test_missing_rustc_sample_rejected(self):
        self.data["records"].remove(self.command(phase="cold"))
        self.rejected()

    def test_cannot_relabel_new_samples_as_schema1(self):
        self.data["schema"] = self.data["records"][0]["report"]["schema"] = 1
        self.rejected()

    def test_new_comparison_cannot_be_omitted_or_reversed(self):
        original = copy.deepcopy(self.data)
        for reverse in (False, True):
            with self.subTest(reverse=reverse):
                self.data = copy.deepcopy(original)
                row = next(row for row in self.data["summaries"] if row["baseline"] == "rustc")
                if reverse:
                    row["baseline"], row["candidate"] = row["candidate"], row["baseline"]
                else:
                    self.data["summaries"].remove(row)
                self.rejected()

    def test_reference_flags_cannot_disable_checks_or_enable_incremental(self):
        original = copy.deepcopy(self.data)
        for old, new in (("panic=unwind", "panic=abort"), ("overflow-checks=on", "overflow-checks=off"),
                         ("debug-assertions=on", "incremental=/tmp/cache")):
            with self.subTest(flag=old):
                self.data = copy.deepcopy(original)
                row = self.command(phase="body-edit")
                row["argv"] = [{"Unix": list((new if reporter.argument(value) == old else reporter.argument(value)).encode())}
                               for value in row["argv"]]
                self.rejected()

    def test_rustc_cannot_compile_another_source(self):
        row = self.command(phase="body-edit")
        row["source_sha256"] = row["source_after_sha256"] = {"/other/source.rs": "d" * 64}
        self.rejected()

    def test_lane_cannot_use_another_identified_compiler(self):
        # Review regression: all packaged-entry samples formerly accepted the
        # hash-known integrated Lust executable under unchanged entry labels.
        replacement = copy.deepcopy(self.command(lane="integrated", phase="cold")["argv"][0])
        changed = 0
        for row in self.data["records"]:
            if row.get("kind") == "command" and row.get("family") == "native" and row.get("lane") == "entry" and not row["phase"].endswith("-runtime"):
                row["argv"][0] = replacement
                changed += 1
        self.assertEqual(changed, 114)
        self.rejected()

    def test_native_lane_identity_must_be_complete_and_initially_bound(self):
        original = copy.deepcopy(self.data)
        for change in ("missing", "changed", "aliased", "unknown", "inconsistent-rustc"):
            with self.subTest(change=change):
                self.data = copy.deepcopy(original)
                paths = self.data["native_lane_executables"]
                if change == "missing":
                    del paths["entry"]
                elif change == "changed":
                    paths["entry"], paths["integrated"] = paths["integrated"], paths["entry"]
                else:
                    if change == "aliased":
                        paths["entry"] = paths["integrated"]
                    elif change == "unknown":
                        paths["entry"] = "/synthetic/unidentified/compiler"
                    else:
                        paths["entry"], paths["rustc"] = paths["rustc"], paths["entry"]
                    self.data["records"][0]["report"]["native_lane_executables"] = copy.deepcopy(paths)
                self.rejected()

    def test_compiled_but_unexecuted_test_rejected_after_rehash(self):
        row = self.command()
        self.stream(row, "stdout", b"test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n")
        self.rejected()

    def test_missing_execute_receipt_rejected_after_rehash(self):
        row = self.command()
        path = self.root / "lustr-paired-measurements" / reporter.output_name(row["stderr"])
        self.stream(row, "stderr", path.read_bytes().splitlines(keepends=True)[0])
        self.rejected()

    def test_negative_test_gate_is_required(self):
        self.data["records"].remove(self.command(phase="reference-probe-failed-test"))
        self.rejected()

    def test_recovery_runtime_output_checked_after_rehash(self):
        row = self.command(phase="reference-probe-recovery-runtime")
        self.stream(row, "stdout", b"wrong\n")
        self.rejected()

    def test_runtime_must_execute_its_matching_build_output(self):
        original = copy.deepcopy(self.data)
        for change in ("executable", "source", "phase"):
            with self.subTest(change=change):
                self.data = copy.deepcopy(original)
                row = self.command(phase="reference-probe-recovery-runtime")
                if change == "executable":
                    row["argv"][0] = {"Unix": list(b"/synthetic/unrelated-program")}
                elif change == "source":
                    row["source_sha256"] = row["source_after_sha256"] = {"/synthetic/other.rs": "d" * 64}
                else:
                    row["phase"] = "reference-probe-build-runtime"
                self.rejected()

    def test_rustc_cannot_claim_a_native_daemon_probe(self):
        self.data["records"].append(dict(kind="native-retained-check", lane="rustc", route="reuse"))
        self.rejected()


if __name__ == "__main__":
    unittest.main(verbosity=2)
