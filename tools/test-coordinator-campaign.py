#!/usr/bin/env python3
"""Artifact-verifier tests; invented durations are never compiler measurements."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

module = importlib.util.spec_from_file_location("campaign", Path(__file__).with_name("measure-coordinator-project.py"))
campaign = importlib.util.module_from_spec(module)
module.loader.exec_module(campaign)


class CampaignTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="private-coordinator-verifier-")
        self.root = Path(self.temporary.name)
        self.private = self.root / "private"
        self.private.mkdir()
        self.source = self.private / "snapshot"
        self.source.mkdir()
        (self.source / "Cargo.lock").write_text("version = 4\n")
        self.sdk, self.package = self.root / "sdk", self.root / "package"
        for root in (self.sdk, self.package):
            (root / "bin").mkdir(parents=True)
            for name in ("cargo", "rustc", "rustdoc", "lustr", "rustc-full", "lust", "lust-core", "cargo-lustr"):
                (root / "bin" / name).write_text(f"fixture identity: {name}\n")
        direct = self.package / "bin/lustr-cargo/rustc"
        direct.parent.mkdir()
        direct.symlink_to("../rustc-full")
        self.gate = self.root / "gate"
        self.gate.write_text("fixture gate identity\n")
        self.inputs = [direct] + [self.package / "bin" / name for name in ("lustr", "rustc-full", "lust", "lust-core", "cargo-lustr")]
        for name in ("COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt"):
            path = self.package.parent / name
            path.write_text("synthetic verifier receipt; not a built compiler\n")
            self.inputs.append(path)

    def tearDown(self):
        self.temporary.cleanup()

    def spec(self, scenario):
        return campaign.make_spec(scenario, source=self.source, private=self.private, package=self.package,
                                  sdk=self.sdk, gate=self.gate, external_inputs=self.inputs, revision="a" * 40,
                                  host="x86_64-unknown-linux-gnu", jobs=2)

    def report(self, scenario="no-op"):
        """Create a coherent synthetic verifier fixture, not a runnable campaign."""
        spec = self.spec(scenario)
        output = self.private / scenario
        output.mkdir()
        campaign.write_json(output / "spec.original.json", spec)
        records, receipts, order = [], [], []
        epoch = 0

        def shutdown(reason):
            receipts.append({"event": "shutdown", "epoch": epoch, "reason": reason, "status": "passed",
                             "remaining_children": 0, "observed_successful_exits": 0})

        def clear(lane, reason):
            nonlocal epoch
            shutdown(reason)
            epoch += 1
            receipts.append({"event": "cleared", "epoch": epoch, "reason": reason,
                             "target": str(output / "targets" / lane), "entries_after_clear": 0,
                             "cache_scope": "cargo+mixed+native+tmp+xdg"})

        def append(lane, phase, pair=None):
            serial = len(records)
            target = output / "targets" / lane
            socket = f"/private-fixture-socket-{epoch}/{lane}.sock"
            env = {key: str(target / name) for key, name in (("TMPDIR", "tmp"), ("XDG_CACHE_HOME", "xdg"),
                   ("LUSTR_MIXED_CACHE_DIR", "mixed-cache"), ("LUST_CACHE_DIR", "native-cache"))}
            env.update(LUST_DAEMON_SOCKET=socket, LUST_DAEMON_IDLE="120s")
            receipts.append({"event": "prepared", "epoch": epoch, "command": serial, "lane": lane,
                             "target": str(target), "socket": socket, "environment": env, "cache_scope": "owned-lane-target"})
            record = {"lane": lane, "phase": phase, "pair": pair, "wall_seconds": 1.0,
                      "exit": 1 if phase.endswith(":invalid-borrow") else 0, "signal": None, "timed_out": False,
                      "output_limit": False, "resources": {"availability": "disabled", "usage": None},
                      "argv": ["/private/secret/command"], "host_before": {"private": "sensitive-fixture-value"}}
            for stream in ("stdout", "stderr"):
                content = b"raw-private-output: /private/secret/source.rs\n"
                if stream == "stdout" and phase.endswith(":executed-tests"):
                    content = json.dumps({"kind": "tests", "groups": ["3 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out"],
                                          "passed_test_names": ["private names are omitted"]}).encode()
                path = output / f"{serial:05}.{stream}"
                path.write_bytes(content)
                record.update({stream: path.name, f"{stream}_bytes": len(content), f"{stream}_sha256": campaign.digest(path)})
            records.append(record)

        for lane in spec["lanes"]:
            for phase in ["identity"] * len(lane["tools"]) + ["dependency-selection", "compiler-evidence-before", "gate:executed-tests", "gate:invalid-borrow"]:
                append(lane["name"], phase)
        for warm, count in ((True, campaign.WARMUPS), (False, campaign.PAIRS)):
            for pair in range(count):
                indices = [0, 1] if pair % 2 == 0 else [1, 0]
                order.append({"warmup": warm, "pair": pair, "lane_indices": indices})
                for index in indices:
                    lane = campaign.LANES[index]
                    if scenario == "clean-dependency-build":
                        clear(lane, "cold-warmup" if warm else "cold-timed")
                    append(lane, "excluded-warmup" if warm else "timed", pair)
        for lane in campaign.LANES:
            if spec["force_root"] is None:
                clear(lane, "final-evidence")
            for phase in ("compiler-evidence-after", "final-gate:executed-tests", "final-gate:invalid-borrow"):
                append(lane, phase)
        shutdown("final")
        summary = {"method": "paired-log-v1/splitmix64/rejection-index/percentile", "pairs": campaign.PAIRS,
                   "raw_ratios": [1.0] * campaign.PAIRS, "log_ratios": [0.0] * campaign.PAIRS,
                   "geometric_ratio": 1.0, "bootstrap_95": [1.0, 1.0], "baseline_median": 1.0, "candidate_median": 1.0,
                   "baseline_p95": 1.0, "candidate_p95": 1.0, "exact_sign_flip_p": 1.0,
                   "bootstrap_resamples": campaign.RESAMPLES, "seed": campaign.SEED}
        source = {"tree_sha256": "b" * 64, "cargo_lock_sha256": campaign.digest(self.source / "Cargo.lock"),
                  "external_inputs": {str(path).encode().hex(): campaign.digest(path) for path in self.inputs}}
        report = {"spec": spec, "status": "passed", "source_before": source, "source_after": source,
                  "final_source_observation": source, "order_method": campaign.ORDER, "order": order,
                  "spec_sha256": campaign.digest(output / "spec.original.json"), "records": records,
                  "native_state": {"kind": campaign.POLICY, "status": "passed", "error": None, "receipts": receipts},
                  "actual_cargo_fingerprints": {lane: {"66696e6765727072696e74": {"sha256": "c" * 64, "data": {}}} for lane in campaign.LANES},
                  "summaries": [{"baseline": campaign.LANES[0], "candidate": campaign.LANES[1], "analysis": summary}]}
        path = output / "report.json"
        campaign.write_json(path, report)
        return path, report

    def corrupt(self, report_path, report):
        report_path.write_text(json.dumps(report))
        with self.assertRaises((ValueError, KeyError, IndexError)):
            campaign.project_report(report_path)

    def test_all_five_scenarios_keep_default_profile_and_real_test_routes(self):
        for scenario in campaign.SCENARIOS:
            path, report = self.report(scenario)
            projected = campaign.project_report(path)
            self.assertEqual(projected["name"], scenario)
            spec = report["spec"]
            self.assertEqual(spec["profile"], "test" if scenario == "compile-plus-test" else "dev")
            self.assertEqual(spec["semantic_settings"]["debug"], "2")
            self.assertEqual(spec["lanes"][0]["gates"][0]["command"]["argv"][1], "tests")
            self.assertEqual(spec["lanes"][1]["gates"][0]["command"]["argv"][1], "tests-via")
            self.assertNotIn("--no-run", str(spec))

    def test_cargo_compiler_evidence_tracks_actual_direct_rustc(self):
        spec = self.spec("clean-dependency-build")
        rust, mixed = spec["lanes"]
        direct = self.package / "bin/lustr-cargo/rustc"
        self.assertEqual(rust["env"]["RUSTC"], str(self.sdk / "bin/rustc"))
        self.assertEqual(mixed["env"]["RUSTC"], str(direct))
        self.assertIn({"path": str(direct), "sha256": campaign.digest(direct), "version_args": ["-vV"]},
                      mixed["tools"])
        self.assertIn(str(direct), spec["external_inputs"])
        self.assertEqual(direct.resolve(strict=True), (self.package / "bin/rustc-full").resolve(strict=True))

    def test_sanitized_projection_omits_paths_commands_environment_and_raw_output(self):
        path, report = self.report()
        projected = campaign.project_report(path)
        text = json.dumps(projected)
        for private in (str(self.root), "sensitive-fixture-value", "/private/secret", "private names", "private-fixture-socket"):
            self.assertNotIn(private, text)
        self.assertEqual(projected["full_private_report_sha256"], campaign.digest(path))
        self.assertNotEqual(hashlib.sha256(text.encode()).hexdigest(), projected["full_private_report_sha256"])
        self.assertEqual(len(projected["records"]), len(report["records"]))
        self.assertEqual(projected["test_counts"][0]["harnesses"][0]["ignored"], 2)

    def test_final_shutdown_required(self):
        path, report = self.report()
        report["native_state"]["receipts"].pop()
        self.corrupt(path, report)

    def test_abnormal_shutdown_cannot_qualify(self):
        path, report = self.report()
        report["native_state"]["receipts"][-1]["status"] = "failed"
        self.corrupt(path, report)

    def test_cold_clear_requires_prior_shutdown(self):
        path, report = self.report("clean-dependency-build")
        receipts = report["native_state"]["receipts"]
        index = next(i for i, row in enumerate(receipts) if row["event"] == "cleared")
        del receipts[index - 1]
        self.corrupt(path, report)

    def test_warm_cache_receipt_cannot_disappear(self):
        path, report = self.report()
        row = next(row for row in report["native_state"]["receipts"] if row["event"] == "prepared")
        del row["environment"]["LUSTR_MIXED_CACHE_DIR"]
        self.corrupt(path, report)

    def test_raw_stream_tampering_is_rejected(self):
        path, _ = self.report()
        (path.parent / "00000.stdout").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "digest/size"):
            campaign.project_report(path)

    def test_profile_drift_is_rejected(self):
        path, report = self.report()
        report["spec"]["lanes"][1]["command"]["argv"].append("--release")
        self.corrupt(path, report)

    def test_old_cargo_compiler_path_cannot_pass_evidence_contract(self):
        path, report = self.report()
        report["spec"]["lanes"][1]["env"]["RUSTC"] = str(self.package / "bin/rustc")
        self.corrupt(path, report)

    def test_constructor_receipt_cannot_disappear(self):
        path, report = self.report()
        report["source_before"]["external_inputs"].pop(str(self.inputs[-1]).encode().hex())
        self.corrupt(path, report)

    def test_cargo_fingerprints_must_be_observed(self):
        path, report = self.report()
        report["actual_cargo_fingerprints"][campaign.LANES[1]] = {}
        self.corrupt(path, report)

    def test_private_summary_fields_cannot_pass_into_public_export(self):
        path, report = self.report()
        report["summaries"][0]["private"] = "/private/secret/sentinel"
        self.corrupt(path, report)

    def test_private_analysis_fields_cannot_pass_into_public_export(self):
        path, report = self.report()
        report["summaries"][0]["analysis"]["private"] = "/private/secret/sentinel"
        self.corrupt(path, report)

    def test_private_order_fields_cannot_pass_into_public_export(self):
        path, report = self.report()
        report["order"][0]["private"] = "/private/secret/sentinel"
        self.corrupt(path, report)

    def test_readable_table_uses_the_replayed_statistics_and_scope(self):
        cases = [campaign.project_report(self.report(scenario)[0]) for scenario in campaign.SCENARIOS]
        identity = {"revision": "a" * 40, "coordinator_git_tree": "b" * 40, "rust_source": "c" * 40,
                    "host": "x86_64-unknown-linux-gnu"}
        text = campaign.public_readme({"cases": cases}, identity,
                                     {"recorded_utc": "synthetic test fixture", "cpu_model": {"value": None}})
        for scenario in campaign.SCENARIOS:
            self.assertIn(f"| `{scenario}` | 1000.000 | 1000.000 |", text)
        self.assertIn("[1.0000, 1.0000]", text)
        self.assertIn("debug=2", text)
        self.assertIn("byte-preserving timestamp", text)
        self.assertIn("3 passed and 2 existing", text)
        self.assertNotIn(str(self.root), text)

    def test_existing_rust_replay_checks_the_public_projection(self):
        path, _ = self.report()
        case = campaign.project_report(path)
        data = {"schema": 1, "pairs": campaign.PAIRS, "seed": campaign.SEED,
                "bootstrap_resamples": campaign.RESAMPLES, "cases": [case]}
        output = self.root / "public-synthetic-fixture.json"
        campaign.write_json(output, data)
        result = subprocess.run([str(REPLAY), str(output)], capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual(json.loads(result.stdout)["verified_comparisons"], 1)
        data["cases"][0]["records"] = [row for row in case["records"] if not (row["phase"] == "timed" and row["pair"] == 0)]
        output.write_text(json.dumps(data))
        self.assertNotEqual(subprocess.run([str(REPLAY), str(output)], capture_output=True).returncode, 0)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--replay", required=True, type=Path, help="the built existing lustr-bench replay example")
    args, tests = parser.parse_known_args()
    REPLAY = args.replay.resolve(strict=True)
    unittest.main(argv=[__file__, *tests])
