#!/usr/bin/env python3
"""Exercise public-slice recording failures and evidence contracts.

Synthetic records below test the recorder, never qualify a compiler or claim
speed. Process tests launch real short-lived children and preserve failures.
"""
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("slice_bench", Path(__file__).with_name("run-upstream-slice-benchmark.py"))
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)


def cargo_output(target, operation="build", fresh=True):
    artifact = {"reason": "compiler-artifact", "target": {"name": "slice", "kind": ["bin"]},
                "profile": {"opt_level": "0", "debuginfo": 0, "debug_assertions": True, "overflow_checks": True},
                "fresh": fresh, "executable": str(target / bench.TARGET / "debug/slice") if operation == "build" else None}
    return [artifact, {"reason": "build-finished", "success": True}]


def encoded(values):
    return b"\n".join(json.dumps(value).encode() for value in values) + b"\n"


def synthetic_campaign(output):
    """Drive real campaign orchestration with explicitly synthetic child results."""
    args = SimpleNamespace(output=output, timeout=10, pairs=12, built=output / "synthetic-tools",
                           pristine_lust=output / "synthetic-pristine/lust",
                           pristine_source=output / "synthetic-pristine-source",
                           repository=output / "synthetic-candidate-source")
    campaign = bench.Campaign(args)
    campaign.fixture.mkdir()
    (campaign.fixture / "src").mkdir()
    (campaign.fixture / "src/slice_main.rs").write_text("fn main() {}\n")
    (campaign.fixture / "Cargo.toml").write_text("# Synthetic replay fixture only\n")
    (campaign.fixture / "Cargo.lock").write_text("# Synthetic lockfile\n")
    reject = output / "rejected.rs"
    reject.write_text(bench.REJECTION)
    campaign.report["rejection_input"] = {"file": "rejected.rs", "sha256": bench.digest(reject)}
    recorder = campaign.recorder
    serial_pid = 500

    def run(argv, env, cwd, lane, phase, expected=0):
        argv = [str(value) for value in argv]
        data, error = b"", b""
        if phase == "gate-reject":
            data = b'{"code":"L0110","severity":"error"}\n'
        elif phase == "gate-retained":
            error = (b"lust-stats: mode=reuse fns=4 rechecked=0 invalidated=0 c_bytes=0 dirty_units=0 units_compiled=0 reemitted=0 discovered=0\n"
                     b"lust-visits: path=reuse why=noop checked=0\n")
        elif len(argv) == 1:
            data = b"synthetic runtime output\n"
        elif lane in bench.LANES and argv[1] in ("build", "check"):
            if lane == "cargo":
                target = Path(env["CARGO_TARGET_DIR"])
                fresh = phase not in ("gate-build", "cold-build", "cold-check")
                data = encoded(cargo_output(target, argv[1], fresh))
                binary = target / bench.TARGET / "debug/slice" if argv[1] == "build" else None
            else:
                binary = Path(argv[argv.index("-o") + 1]) if "-o" in argv else None
            if binary is not None:
                binary.parent.mkdir(parents=True, exist_ok=True)
                binary.write_bytes(b"synthetic nonexecutable bytes, never run\n")
            if "LUSTR_TRACE_PATH" in env:
                Path(env["LUSTR_TRACE_PATH"]).write_bytes(encoded([{"engine": "lust", "entry": "upstream-process" if "LUST_NO_DAEMON" in env else "daemon"}]))
        row = {"kind": "command", "id": len(recorder.records), "argv": argv, "environment": env,
               "cwd": str(cwd), "lane": lane, "phase": phase, "expected_exit": expected, "exit": expected,
               "signal": None, "timed_out": False, "output_limit": False, "seconds": .01}
        for stream, value in (("stdout", data), ("stderr", error)):
            path = output / "commands" / f"{row['id']:05}.{stream}"
            path.write_bytes(value)
            row[stream] = {"file": str(path.relative_to(output)), "bytes": len(value), "sha256": bench.digest(path)}
        recorder.event(row)
        return row, data, error

    class SyntheticDaemon:
        def __init__(self, recorder, executable, env, cwd, lane, scope=None):
            nonlocal serial_pid
            serial_pid += 1
            self.pid, self.lane, self.executable, self.scope, self.env, self.cwd = serial_pid, lane, executable, scope, env, cwd

        def __enter__(self):
            recorder.event({"kind": "daemon-start", "lane": self.lane, "pid": self.pid,
                            "executable": str(self.executable), "scope": self.scope, "socket": self.env["LUST_DAEMON_SOCKET"],
                            "environment": self.env, "cwd": str(self.cwd)})
            return self

        def alive(self):
            pass

        def __exit__(self, *arguments):
            streams = []
            for name in ("stdout", "stderr"):
                path = output / "commands" / f"{len(recorder.records):05}.daemon.{name}"
                path.write_bytes(b"synthetic daemon record\n")
                streams.append({"file": str(path.relative_to(output)), "sha256": bench.digest(path)})
            recorder.event({"kind": "daemon-stop", "lane": self.lane, "scope": self.scope, "pid": self.pid,
                            "exit": 0, "error": None, "streams": streams})

    with patch.object(recorder, "run", side_effect=run), patch.object(bench, "Daemon", SyntheticDaemon):
        for lane, phase in (("source", "pristine-identity"), ("source", "pristine-identity"),
                            ("source", "pristine-clean"), ("source", "candidate-identity"),
                            ("source", "candidate-clean"), ("cargo", "rust-identity")):
            run(["/synthetic/identity-tool", phase], {}, output, lane, phase)
        run(["/synthetic/identity-tool", "lockfile"], {}, campaign.fixture, "cargo", "lockfile")
        identity = {"compiler_and_runner_sha256": {"/synthetic/compiler": "c" * 64},
                    "tool_paths": {key: str(value) for key, value in campaign.tools.items()},
                    "fixture_files": bench.inventory(campaign.fixture),
                    "rejection_input": campaign.report["rejection_input"],
                    "reference_rustc": str(campaign.reference), "build_receipts": {}}
        receipts = output / "build-receipts"
        receipts.mkdir()
        for name in ("BUILD-PROFILE.txt", "COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt", "pristine-build.log"):
            path = receipts / name
            path.write_text("Synthetic receipt for replay tests only\n")
            source = f"/synthetic/build-receipts/{name}"
            identity["compiler_and_runner_sha256"][source] = bench.digest(path)
            identity["build_receipts"][name] = {"source_path": source, "sha256": bench.digest(path)}
        campaign.report.update(identity)
        recorder.event({"kind": "identities", **identity})
        sockets = output / "synthetic-sockets"
        sockets.mkdir()
        campaign.qualification(sockets, "initial")
        campaign.report["orders"] = bench.orders(12)
        for sample, order in enumerate(campaign.report["orders"]):
            recorder.event({"kind": "sample-start", "sample": sample, "order": order})
            for operation in ("check", "build"):
                campaign.measure_group(sample, order, operation, sockets)
            recorder.event({"kind": "sample-end", "sample": sample})
        campaign.qualification(sockets, "final")
    campaign.report.update(status="completed", samples=campaign.samples, summaries=bench.summaries(campaign.samples, 12))
    recorder.raw.close()
    campaign.report["raw_sha256"] = bench.digest(output / "raw.jsonl")
    bench.write_json(output / "report.json", campaign.report)
    (output / "README.md").write_text(bench.render(campaign.report))
    return campaign.report


class FullReplayTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.original = Path(cls.temporary.name) / "synthetic-original"
        cls.original.mkdir()
        synthetic_campaign(cls.original)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def corrupted(self, mutate):
        target = Path(self.temporary.name) / self.id().split(".")[-1]
        shutil.copytree(self.original, target)
        records = [json.loads(line) for line in (target / "raw.jsonl").read_text().splitlines()]
        mutate(records)
        (target / "raw.jsonl").write_bytes(encoded(records))
        report = json.loads((target / "report.json").read_text())
        report["raw_sha256"] = bench.digest(target / "raw.jsonl")
        bench.write_json(target / "report.json", report)
        return target

    def test_full_synthetic_campaign_replays_all_288_samples(self):
        result = bench.replay(self.original)
        self.assertEqual(len(result["samples"]), 288)
        self.assertEqual(len(result["summaries"]), 18)

    def test_removed_initial_and_final_rejection_gates_are_not_accepted(self):
        target = self.corrupted(lambda rows: rows.__setitem__(slice(None), [row for row in rows if row.get("phase") != "gate-reject"]))
        with self.assertRaises(ValueError):
            bench.replay(target)

    def test_removed_cold_and_noop_runtime_gates_are_not_accepted(self):
        def mutate(rows):
            removed = {row["id"] for row in rows if row.get("kind") == "command" and row.get("phase") in ("cold-build-runtime", "no-op-build-runtime")}
            rows[:] = [row for row in rows if not (row.get("kind") == "command" and row.get("id") in removed)
                       and not (row.get("kind") == "runtime-identity" and row.get("command") in removed)]
        target = self.corrupted(mutate)
        with self.assertRaises(ValueError):
            bench.replay(target)

    def test_measured_lane_reordering_is_not_hidden_by_declared_order(self):
        def mutate(rows):
            positions = [i for i, row in enumerate(rows) if row.get("kind") == "command" and row.get("phase") == "cold-check"][:2]
            rows[positions[0]], rows[positions[1]] = rows[positions[1]], rows[positions[0]]
        target = self.corrupted(mutate)
        with self.assertRaises(ValueError):
            bench.replay(target)

    def test_daemon_stop_must_match_started_process_and_lane(self):
        def mutate(rows):
            row = next(row for row in rows if row.get("kind") == "daemon-stop")
            row.update(pid=99999, lane="another")
        target = self.corrupted(mutate)
        with self.assertRaises(ValueError):
            bench.replay(target)

    def test_missing_packaged_route_and_native_retained_evidence_rejects(self):
        target = self.corrupted(lambda rows: rows.__setitem__(slice(None), [row for row in rows if row.get("kind") not in ("entry-route", "retained-route")]))
        with self.assertRaises(ValueError):
            bench.replay(target)


class EvidenceTests(unittest.TestCase):
    def test_each_lane_visits_each_position_equally(self):
        rows = bench.orders(12)
        for lane in bench.LANES:
            self.assertEqual([sum(row[position] == lane for row in rows) for position in range(4)], [3, 3, 3, 3])
        self.assertEqual(rows, bench.orders(12))
        self.assertNotEqual(rows, bench.orders(12, seed=1))

    def test_unbalanced_or_unbounded_campaign_size_rejects(self):
        for pairs in (0, 8, 13, 100):
            with self.assertRaises(ValueError):
                bench.orders(pairs)

    def test_cargo_fresh_requires_selected_artifact_and_profile(self):
        target = Path("/synthetic/lane/target")
        rows = cargo_output(target)
        self.assertEqual(bench.cargo_artifact(encoded(rows), "build", target, True), target / bench.TARGET / "debug/slice")
        for mutation in (lambda a: a.update(fresh=False), lambda a: a.update(executable="/elsewhere/program"),
                         lambda a: a["profile"].update(debug_assertions=False), lambda a: a["target"].update(name="different")):
            changed = copy.deepcopy(rows)
            mutation(changed[0])
            with self.assertRaises(ValueError):
                bench.cargo_artifact(encoded(changed), "build", target, True)
        with self.assertRaises(ValueError):
            bench.cargo_artifact(encoded(rows + rows), "build", target, True)
        with self.assertRaises(ValueError):
            bench.cargo_artifact(encoded(rows[:1]), "build", target, True)

    def test_cargo_check_cannot_be_an_executable_build(self):
        target = Path("/synthetic/target")
        self.assertIsNone(bench.cargo_artifact(encoded(cargo_output(target, "check", False)), "check", target, False))
        with self.assertRaises(ValueError):
            bench.cargo_artifact(encoded(cargo_output(target)), "check", target, True)

    def test_retained_check_rejects_full_or_repeated_work(self):
        observed = ("lust-stats: mode=bodies fns=4 rechecked=0 invalidated=0 c_bytes=0 dirty_units=0 units_compiled=0 reemitted=0 discovered=0\n"
                    "lust-visits: path=chunks why=bodies checked=0\n").encode()
        self.assertEqual(bench.retained_check(observed), ("bodies", "chunks", "bodies"))
        for old, new in ((b"mode=bodies", b"mode=full"), (b"checked=0", b"checked=1"),
                         (b"fns=4", b"fns=0"), (b"c_bytes=0", b"c_bytes=0 c_bytes=0"),
                         (b"rechecked=0", b"rechecked=-0")):
            with self.assertRaises(ValueError):
                bench.retained_check(observed.replace(old, new))
        with self.assertRaises(ValueError):
            bench.retained_check(observed + observed)

    def test_all_modes_and_pairs_are_required_before_statistics(self):
        samples = [{"sample": pair, "lane": lane, "phase": phase, "seconds": (index + 1) * .01, "commands": [1]}
                   for pair in range(12) for phase in bench.PHASES for index, lane in enumerate(bench.LANES)]
        rows = bench.summaries(samples, 12)
        self.assertEqual(len(rows), 18)
        for item in rows:
            self.assertEqual(item["analysis"]["method"], bench.STATS.METHOD)
        with self.assertRaises(ValueError):
            bench.summaries(samples[:-1], 12)
        changed = copy.deepcopy(samples)
        changed[-1] = changed[0]
        with self.assertRaises(ValueError):
            bench.summaries(changed, 12)

    def test_failed_campaign_never_renders_a_speed_table(self):
        text = bench.render({"status": "failed", "scope": "synthetic", "limits": "synthetic", "summaries": [{"untrusted": True}]})
        self.assertIn("No completed performance result", text)
        self.assertNotIn("Ratio [95%", text)


class ActualProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.recorder = bench.Recorder(self.root, .15)
        self.env = {"PATH": "/usr/bin:/bin", "LANG": "C"}

    def tearDown(self):
        self.recorder.raw.close()
        self.temp.cleanup()

    def run_child(self, program, expected=0):
        return self.recorder.run([sys.executable, "-c", program], self.env, self.root, "test", "actual-child", expected)

    def test_success_records_the_actual_original_stream(self):
        row, out, err = self.run_child('print("actual output")')
        self.assertEqual(out, b"actual output\n")
        self.assertEqual(err, b"")
        self.assertEqual(row["stdout"]["sha256"], bench.digest(self.root / row["stdout"]["file"]))
        self.assertGreater(row["seconds"], 0)

    def test_nonzero_exit_is_not_a_fast_success(self):
        with self.assertRaises(ValueError):
            self.run_child('import sys; print("original failure", file=sys.stderr); sys.exit(7)')
        row = self.recorder.records[-1]
        self.assertEqual(row["exit"], 7)
        self.assertEqual((self.root / row["stderr"]["file"]).read_bytes(), b"original failure\n")

    def test_timeout_is_preserved_and_never_reclassified_as_rejection(self):
        with self.assertRaises(ValueError):
            self.run_child('import time; print("started", flush=True); time.sleep(5)', expected=1)
        row = self.recorder.records[-1]
        self.assertTrue(row["timed_out"])
        self.assertEqual(row["signal"], 9)
        self.assertIsNone(row["exit"])
        self.assertEqual((self.root / row["stdout"]["file"]).read_bytes(), b"started\n")

    def test_output_limit_applies_even_if_the_child_already_exited(self):
        self.recorder.timeout = 5
        with self.assertRaises(ValueError):
            self.run_child('import sys; sys.stdout.buffer.write(b"x" * (9 * 1024 * 1024))')
        self.assertTrue(self.recorder.records[-1]["output_limit"])

    def test_missing_tool_retains_launch_failure(self):
        with self.assertRaises(ValueError):
            self.recorder.run([self.root / "absent"], self.env, self.root, "test", "missing")
        self.assertIn("launch_error", self.recorder.records[-1])

    def test_daemon_start_failure_keeps_its_original_diagnostic(self):
        executable = self.root / "failed-daemon"
        executable.write_text(f'#!{sys.executable}\nimport sys\nprint("socket denied", file=sys.stderr)\nsys.exit(2)\n')
        executable.chmod(0o700)
        env = dict(self.env, LUST_DAEMON_SOCKET=str(self.root / "never.sock"))
        with self.assertRaises(ValueError):
            with bench.Daemon(self.recorder, executable, env, self.root, "test", "failure-test"):
                self.fail("failed daemon was admitted")
        row = self.recorder.records[-1]
        self.assertEqual(row["exit"], 2)
        self.assertEqual((self.root / row["streams"][1]["file"]).read_bytes(), b"socket denied\n")


if __name__ == "__main__":
    unittest.main()
