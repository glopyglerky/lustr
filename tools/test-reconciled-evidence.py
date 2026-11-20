#!/usr/bin/env python3
"""Relocation and corruption checks using existing synthetic verifier fixtures.

No compiler campaign, private recipient key, or new statistics implementation.
"""
import argparse
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
from types import SimpleNamespace
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "tools" / filename)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


fixture_module = load("existing_coordinator_fixtures", "test-coordinator-campaign.py")
campaign = fixture_module.campaign
retention = load("coordinator_retention", "encrypt-coordinator-evidence.py")
reconciliation = load("coordinator_reconciliation", "reconcile-coordinator-evidence.py")


class ReconciliationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="coordinator-relocation-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        fixture = fixture_module.CampaignTests()
        fixture.setUp()
        self.addCleanup(fixture.tearDown)
        self.repo = self.root / "recorded-source"
        (self.repo / "tools").mkdir(parents=True)
        self.exporter = (ROOT / "tools/measure-coordinator-project.py").read_bytes()
        (self.repo / "tools/measure-coordinator-project.py").write_bytes(self.exporter)
        shutil.copytree(fixture.source, self.repo / "crates/lustr")
        # The recorded runner tree contains the actual existing runner sources;
        # synthetic reports still carry an explicitly invented binary identity.
        archived = subprocess.run(["git", "-C", str(ROOT), "archive", "--format=tar", "HEAD:crates/lustr-bench"],
                                  check=True, capture_output=True).stdout
        with tarfile.open(fileobj=io.BytesIO(archived)) as source:
            for entry in source:
                path = self.repo / "crates/lustr-bench" / entry.name
                if entry.isdir():
                    path.mkdir(parents=True, exist_ok=True)
                else:
                    self.assertTrue(entry.isfile())
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(source.extractfile(entry).read())
                    path.chmod(entry.mode)
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "user.name=Verifier fixture", "-c", "user.email=fixture@example.invalid",
                 "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-qm", "synthetic evidence source")
        revision = self.git("rev-parse", "HEAD")
        original_spec = fixture.spec
        fixture.spec = lambda scenario: {**original_spec(scenario), "revision": revision}
        profile = fixture.package.parent / "BUILD-PROFILE.txt"
        profile.write_text("Synthetic verifier receipt; not compiler qualification.\n")
        fixture.inputs.append(profile)
        (fixture.private / "specs").mkdir()
        (fixture.private / "reject.rs").write_text("synthetic verifier source, never compiled\n")
        identity = {"repository": "https://github.com/glopyglerky/lustr", "revision": revision,
                    "coordinator_git_tree": self.git("rev-parse", "HEAD:crates/lustr"),
                    "rust_source": "c" * 40, "host": "x86_64-unknown-linux-gnu",
                    "compiler_components_sha256": {name: campaign.digest(fixture.package / "bin" / name)
                        for name in ("lustr", "rustc-full", "lust", "lust-core", "cargo-lustr")},
                    "constructor_receipts_sha256": {name: campaign.digest(fixture.package.parent / name)
                        for name in ("COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt", "BUILD-PROFILE.txt")},
                    "runner_sha256": "d" * 64, "gate_sha256": campaign.digest(fixture.gate),
                    "replay_sha256": campaign.digest(REPLAY), "cargo_sha256": campaign.digest(fixture.sdk / "bin/cargo"),
                    "reference_rustc_sha256": campaign.digest(fixture.sdk / "bin/rustc")}
        cases = []
        for name in campaign.SCENARIOS:
            path, report = fixture.report(name)
            rows = {os.fsencode(p.relative_to(fixture.source)).hex(): f"{p.stat().st_mode:o}:{campaign.digest(p)}"
                    for p in fixture.source.rglob("*") if p.is_file()}
            tree = hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            for key in ("source_before", "source_after", "final_source_observation"):
                report[key].update(files_hex_paths=rows, tree_sha256=tree)
            report["runner_sha256"] = identity["runner_sha256"]
            path.write_bytes(reconciliation.encoded(report))
            shutil.copy2(path.parent / "spec.original.json", fixture.private / "specs" / (name + ".json"))
            cases.append(campaign.project_report(path))
        data = {"schema": 1, "pairs": campaign.PAIRS, "warmup_pairs": campaign.WARMUPS, "seed": campaign.SEED,
                "bootstrap_resamples": campaign.RESAMPLES, "runner_sha256": identity["runner_sha256"], "cases": cases}
        host = {"recorded_utc": "synthetic verifier fixture only", "cpu_model": {"value": None}}
        self.public = self.root / "public"
        self.public.mkdir()
        campaign.write_json(self.public / "samples.json", data)
        campaign.write_json(self.public / "host.json", host)
        campaign.write_json(self.public / "campaign.json", {
            **identity, "schema": "lustr-coordinator-campaign-v1", "status": "passed", "scenarios": list(campaign.SCENARIOS),
            "project_profiles": "unchanged dev and test defaults", "jobs": 2, "pairs": campaign.PAIRS, "warmup_pairs": campaign.WARMUPS,
            "measurement": "wall time only; detached daemon CPU, memory and energy are not measured",
            "privacy": "Allowlist projection. Original report/spec/output SHA256 values identify private originals, not these rewritten bytes."})
        replayed = json.loads(subprocess.run([str(REPLAY), str(self.public / "samples.json")], check=True, capture_output=True).stdout)
        campaign.write_json(self.public / "replay.json", replayed)
        (self.public / "README.md").write_text(campaign.public_readme(data, identity, host))
        self.public_manifest()
        campaign.write_json(fixture.private / "identity.json", identity)
        campaign.write_json(fixture.private / "host.json", host)
        campaign.write_json(fixture.private / "campaign-result.json", {
            "schema": "lustr-coordinator-campaign-v1", "status": "passed", "scenarios": list(campaign.SCENARIOS),
            "public_samples_sha256": campaign.digest(self.public / "samples.json"),
            "original_report_sha256": {case["name"]: case["full_private_report_sha256"] for case in cases}})
        self.restored = self.root / "restored"
        self.restored.mkdir()
        self.receipt = self.root / "receipt.json"
        self.archive_serial = 0
        archive = self.private_manifest(fixture.private)
        with tarfile.open(archive, "r:gz") as source:
            for entry in source:
                path = self.restored / entry.name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(source.extractfile(entry).read())
                path.chmod(entry.mode)
        self.old_root = fixture.root
        fixture.tearDown()
        self.assertFalse(self.old_root.exists())
        self.args = SimpleNamespace(restored=self.restored, public=self.public, receipt=self.receipt,
                                    repository=self.repo, replay=REPLAY)

    def git(self, *arguments):
        return subprocess.run(["git", "-C", str(self.repo), *arguments], check=True, capture_output=True).stdout.decode().strip()

    def public_manifest(self):
        (self.public / "SHA256SUMS").write_text("".join(f"{campaign.digest(self.public / name)}  {name}\n"
                                                      for name in sorted(reconciliation.PUBLIC)))

    def private_manifest(self, private=None):
        private = private or self.restored / "private"
        entries = retention.inventory(private)
        archive = self.root / f"originals-{self.archive_serial}.tar.gz"
        self.archive_serial += 1
        manifest_hash = retention.archive(entries, archive)
        receipt = {"schema": "lustr-encrypted-evidence-receipt-v1", "encrypted_manifest_sha256": manifest_hash,
                   "original_file_count": len(entries), "original_file_bytes": sum(entry.size for entry in entries),
                   "plaintext_archive_sha256": retention.digest(archive), "plaintext_archive_bytes": archive.stat().st_size}
        retention.verify_archive(archive, receipt)
        self.receipt.write_bytes(reconciliation.encoded(receipt))
        with tarfile.open(archive, "r:gz") as source:
            (self.restored / "MANIFEST.json").write_bytes(source.extractfile("MANIFEST.json").read())
        return archive

    def test_relocated_success_loads_recorded_source_and_discloses_binary_limits(self):
        with self.assertRaises(FileNotFoundError):
            campaign.project_report(self.restored / "private/no-op/report.json")
        # Working-tree substitutions are never imported; the commit blob wins.
        (self.repo / "tools/measure-coordinator-project.py").write_text("raise RuntimeError('unrecorded exporter')\n")
        result = reconciliation.reconcile(self.args)
        self.assertEqual(result["status"], "passed")
        self.assertEqual(result["scenario_count"], 5)
        self.assertTrue(result["samples_and_readme_byte_identical"])
        self.assertEqual(result["verified_statistical_comparisons"], 5)
        self.assertFalse(result["compiler_binaries_freshly_rehashed"])
        source = result["source_identity"]
        self.assertEqual(source["exporter_sha256"], hashlib.sha256(self.exporter).hexdigest())
        self.assertEqual(source["runner_source_git_tree"], self.git("rev-parse", "HEAD:crates/lustr-bench"))
        self.assertIsNone(source["campaign_exporter_runtime_sha256"])
        self.assertFalse(source["runner_binary_freshly_rehashed"])
        self.assertTrue(source["replay_matches_recorded_binary"])

    def test_private_stream_tampering_is_rejected(self):
        (self.restored / "private/no-op/00000.stderr").write_text("changed private original\n")
        with self.assertRaisesRegex(reconciliation.CheckFailure, "private-file-digest-or-size"):
            reconciliation.reconcile(self.args)

    def test_git_replace_cannot_substitute_the_recorded_exporter(self):
        original = self.git("rev-parse", "HEAD")
        original_tree = self.git("rev-parse", "HEAD^{tree}")
        path = self.repo / "tools/measure-coordinator-project.py"
        path.write_bytes(self.exporter + b"\nREVIEW_REPLACEMENT_SENTINEL = True\n")
        self.git("add", "tools/measure-coordinator-project.py")
        self.git("-c", "user.name=Verifier fixture", "-c", "user.email=fixture@example.invalid",
                 "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "commit", "-qm", "replacement fixture")
        replacement = self.git("rev-parse", "HEAD")
        self.git("replace", original, replacement)
        self.assertIn("REVIEW_REPLACEMENT_SENTINEL", self.git("show", original + ":tools/measure-coordinator-project.py"))
        result = reconciliation.reconcile(self.args)
        source = result["source_identity"]
        self.assertEqual(source["revision"], original)
        self.assertEqual(source["git_tree"], original_tree)
        self.assertEqual(source["exporter_sha256"], hashlib.sha256(self.exporter).hexdigest())

    def test_public_tampering_is_rejected(self):
        with (self.public / "samples.json").open("ab") as stream:
            stream.write(b" ")
        with self.assertRaisesRegex(reconciliation.CheckFailure, "public-file-digest"):
            reconciliation.reconcile(self.args)

    def test_updated_hashes_cannot_hide_projection_disagreement(self):
        path = self.public / "samples.json"
        data = json.loads(path.read_bytes())
        data["cases"][0]["records"][0]["wall_seconds"] = 2.0
        path.write_bytes(reconciliation.encoded(data))
        result_path = self.restored / "private/campaign-result.json"
        result = json.loads(result_path.read_bytes())
        result["public_samples_sha256"] = campaign.digest(path)
        result_path.write_bytes(reconciliation.encoded(result))
        self.public_manifest()
        self.private_manifest()
        with self.assertRaisesRegex(reconciliation.CheckFailure, "public-case-does-not-reproduce"):
            reconciliation.reconcile(self.args)

    def test_replay_receipt_tampering_is_rejected_after_manifest_update(self):
        path = self.public / "replay.json"
        replayed = json.loads(path.read_bytes())
        replayed["verified_comparisons"] = 0
        path.write_bytes(reconciliation.encoded(replayed))
        self.public_manifest()
        with self.assertRaisesRegex(reconciliation.CheckFailure, "public-replay-receipt-does-not-reproduce"):
            reconciliation.reconcile(self.args)

    def test_source_tree_claim_is_checked_against_recorded_git_revision(self):
        identity_path = self.restored / "private/identity.json"
        identity = json.loads(identity_path.read_bytes())
        identity["coordinator_git_tree"] = "f" * 40
        identity_path.write_bytes(reconciliation.encoded(identity))
        public_path = self.public / "campaign.json"
        public = json.loads(public_path.read_bytes())
        public["coordinator_git_tree"] = identity["coordinator_git_tree"]
        public_path.write_bytes(reconciliation.encoded(public))
        self.public_manifest()
        self.private_manifest()
        with self.assertRaisesRegex(reconciliation.CheckFailure, "recorded-coordinator-tree-differs"):
            reconciliation.reconcile(self.args)

    def test_original_runner_self_hash_must_match_campaign_identity(self):
        path = self.restored / "private/no-op/report.json"
        report = json.loads(path.read_bytes())
        report["runner_sha256"] = "e" * 64
        path.write_bytes(reconciliation.encoded(report))
        samples_path = self.public / "samples.json"
        samples = json.loads(samples_path.read_bytes())
        case = next(case for case in samples["cases"] if case["name"] == "no-op")
        case["full_private_report_sha256"] = campaign.digest(path)
        samples_path.write_bytes(reconciliation.encoded(samples))
        result_path = self.restored / "private/campaign-result.json"
        result = json.loads(result_path.read_bytes())
        result["original_report_sha256"]["no-op"] = campaign.digest(path)
        result["public_samples_sha256"] = campaign.digest(samples_path)
        result_path.write_bytes(reconciliation.encoded(result))
        self.public_manifest()
        self.private_manifest()
        with self.assertRaisesRegex(reconciliation.CheckFailure, "recorded-runner-identity-disagrees"):
            reconciliation.reconcile(self.args)

    def test_different_local_replay_hash_is_disclosed_without_claiming_source_equivalence(self):
        for path in (self.restored / "private/identity.json", self.public / "campaign.json"):
            value = json.loads(path.read_bytes())
            value["replay_sha256"] = "e" * 64
            path.write_bytes(reconciliation.encoded(value))
        self.public_manifest()
        self.private_manifest()
        result = reconciliation.reconcile(self.args)
        source = result["source_identity"]
        self.assertFalse(source["replay_matches_recorded_binary"])
        self.assertEqual(source["replay_local_binary_sha256"], campaign.digest(REPLAY))
        self.assertEqual(source["replay_recorded_binary_sha256"], "e" * 64)
        self.assertEqual(source["replay_binary_source_equivalence"], "not re-established by this check")

    def test_parent_symlink_is_rejected_before_manifest_reads(self):
        scenario = self.restored / "private/no-op"
        outside = self.root / "outside-original-tree"
        scenario.rename(outside)
        scenario.symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(reconciliation.CheckFailure, "private-symlink"):
            reconciliation.reconcile(self.args)

    def test_relative_escape_is_not_a_relocated_read(self):
        original = Path("/original/private")
        with self.assertRaisesRegex(reconciliation.CheckFailure, "relative-path-escape"):
            reconciliation.relocated(original / ".." / "outside", original, self.restored / "private")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--replay", required=True, type=Path, help="the built existing lustr-bench replay example")
    args, tests = parser.parse_known_args()
    REPLAY = args.replay.resolve(strict=True)
    unittest.main(argv=[__file__, *tests])
