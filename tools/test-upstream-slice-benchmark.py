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
import subprocess
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


def synthetic_campaign(output, diagnostics=None):
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
    active_daemons = {}

    def run(argv, env, cwd, lane, phase, expected=0):
        argv = [str(value) for value in argv]
        data, error = b"", b""
        if phase == "gate-reject":
            data = b'{"code":"L0110","severity":"error"}\n'
        elif phase == "gate-retained":
            error = (diagnostics(active_daemons[lane], "patch" if lane == "entry" else "reuse") if diagnostics else
                     b"lust-stats: mode=reuse fns=4 rechecked=0 invalidated=0 c_bytes=0 dirty_units=0 units_compiled=0 reemitted=0 discovered=0\n"
                     b"lust-visits: path=reuse why=- checked=0\n")
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
            active_daemons[self.lane] = self.env
            recorder.event({"kind": "daemon-start", "lane": self.lane, "pid": self.pid,
                            "executable": str(self.executable), "scope": self.scope, "socket": self.env["LUST_DAEMON_SOCKET"],
                            "environment": self.env, "cwd": str(self.cwd)})
            return self

        def alive(self):
            pass

        def __exit__(self, *arguments):
            del active_daemons[self.lane]
            streams = []
            for name in ("stdout", "stderr"):
                path = output / "commands" / f"{len(recorder.records):05}.daemon.{name}"
                path.write_bytes(b"synthetic daemon record\n")
                streams.append({"file": str(path.relative_to(output)), "sha256": bench.digest(path)})
            recorder.event({"kind": "daemon-stop", "lane": self.lane, "scope": self.scope, "pid": self.pid,
                            "exit": 0, "error": None, "streams": streams})

    with recorder.raw, patch.object(recorder, "run", side_effect=run), patch.object(bench, "Daemon", SyntheticDaemon):
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

    def test_qualification_daemon_requires_its_own_stats_environment(self):
        def mutate(rows):
            row = next(row for row in rows if row.get("kind") == "daemon-start"
                       and row["scope"].startswith("qualification-"))
            row["environment"].pop("LUST_STATS", None)
        with self.assertRaises(ValueError):
            bench.replay(self.corrupted(mutate))

    def test_stats_cannot_leak_into_measured_daemons(self):
        def mutate(rows):
            row = next(row for row in rows if row.get("kind") == "daemon-start"
                       and row["scope"].startswith("sample-"))
            row["environment"]["LUST_STATS"] = "1"
        with self.assertRaises(ValueError):
            bench.replay(self.corrupted(mutate))

    def test_prime_and_probe_keep_the_same_stats_setting(self):
        def mutate(rows):
            row = next(row for row in rows if row.get("phase") == "gate-warm-prime")
            row["environment"].pop("LUST_STATS", None)
        with self.assertRaises(ValueError):
            bench.replay(self.corrupted(mutate))


class UpstreamDiagnosticTests(unittest.TestCase):
    """Execute the pinned renderer and stats switch, not a simulated compiler.

    This small Rust extraction needs no dependencies or daemon/socket access.
    Campaign orchestration remains synthetic and makes no performance claim.
    """
    @classmethod
    def setUpClass(cls):
        compiler = os.environ.get("RUSTC") or shutil.which("rustc")
        if compiler is None:
            raise unittest.SkipTest("Rust compiler unavailable for actual upstream diagnostic renderer")
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name)
        source = (Path(__file__).resolve().parent.parent / "compiler/native/src/driver/compile.rs").read_text()
        start = source.index("fn stats_on() -> bool {")
        enabled = source[start:source.index("\n}\n", start) + 3]
        start = source.index("#[derive(Default)]\nstruct Stats {")
        stats = source[start:source.index("\n/// What each front-end", start)]
        start = source.index("#[derive(Default)]\nstruct Visits {")
        visits = source[start:source.index("\n/// Appends one", start)]
        probe = cls.root / "diagnostics.rs"
        probe.write_text('use std::fmt::Write;\n' + enabled + stats + visits + r'''
fn main() {
    let mut out = String::new();
    if stats_on() {
        let (mode, path, why) = match std::env::args().nth(1).as_deref() {
            Some("reuse") => ("reuse", "reuse", ""),
            Some("patch") => ("bodies", "patch", ""),
            Some("chunks") => ("bodies", "chunks", "bodies"),
            _ => panic!("unknown diagnostic case"),
        };
        Stats { mode, fns: 1797, ..Default::default() }.render(&mut out);
        Visits { path, why, ..Default::default() }.render(&mut out);
    }
    eprint!("{out}");
}
''')
        cls.executable = cls.root / "diagnostics"
        # The workflow runs Python preflight before installing its matching
        # nightly. This standalone renderer works with the host default Rust.
        env = dict(os.environ)
        env.pop("RUSTUP_TOOLCHAIN", None)
        subprocess.run([compiler, "--edition=2024", str(probe), "-o", str(cls.executable)],
                       cwd=cls.root, env=env, check=True, capture_output=True)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    @classmethod
    def diagnostics(cls, daemon_environment, route):
        result = subprocess.run([str(cls.executable), route], env=daemon_environment,
                                check=True, capture_output=True)
        return result.stderr

    def test_client_stats_cannot_enable_daemon_process_diagnostics(self):
        output = self.diagnostics({}, "reuse")
        self.assertEqual(output, b"")
        with self.assertRaisesRegex(ValueError, "missing or repeated native diagnostic record"):
            bench.retained_check(output)

    def test_exact_upstream_retained_routes_are_accepted(self):
        for case, route in (("reuse", ("reuse", "reuse", "-")),
                            ("patch", ("bodies", "patch", "-")),
                            ("chunks", ("bodies", "chunks", "bodies"))):
            with self.subTest(case=case):
                self.assertEqual(bench.retained_check(self.diagnostics({"LUST_STATS": "1"}, case)), route)

    def test_retained_routes_reject_reasons_the_upstream_path_does_not_emit(self):
        for case, old, new in (("reuse", b"why=-", b"why=noop"),
                               ("patch", b"why=-", b"why=bodies"),
                               ("chunks", b"why=bodies", b"why=-")):
            with self.subTest(case=case), self.assertRaises(ValueError):
                bench.retained_check(self.diagnostics({"LUST_STATS": "1"}, case).replace(old, new))

    def test_upstream_retained_routes_still_require_zero_repeated_work(self):
        for case in ("reuse", "patch", "chunks"):
            observed = self.diagnostics({"LUST_STATS": "1"}, case)
            for key in ("rechecked", "invalidated", "c_bytes", "dirty_units", "units_compiled", "reemitted", "discovered", "checked"):
                with self.subTest(case=case, counter=key), self.assertRaises(ValueError):
                    bench.retained_check(observed.replace(f" {key}=0".encode(), f" {key}=1".encode()))

    def test_qualification_enables_stats_where_upstream_reads_them(self):
        output = self.root / "actual-renderer-synthetic-campaign"
        output.mkdir()
        report = synthetic_campaign(output, self.diagnostics)
        self.assertEqual(report["status"], "completed")
        bench.replay(output)


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


class RetiredProcessControls(unittest.TestCase):
    def test_record_models_cannot_execute_a_child_or_start_a_daemon(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            recorder = bench.Recorder(root, .15)
            with recorder.raw:
                with self.assertRaisesRegex(ValueError, "acquisition is disabled"):
                    recorder.run([sys.executable, "-c", "raise SystemExit(0)"], {}, root, "test", "retired")
                with self.assertRaisesRegex(ValueError, "acquisition is disabled"):
                    bench.Daemon(recorder, root / "compiler", {}, root, "test")
                self.assertEqual(recorder.records, [])
                self.assertEqual(list((root / "commands").iterdir()), [])


if __name__ == "__main__":
    unittest.main()
