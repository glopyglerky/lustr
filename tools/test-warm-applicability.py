#!/usr/bin/env python3
"""Exercise historical applicability with owned Git fixtures and failing controls.

The fixture failure records are explicitly synthetic. No compiler or benchmark
is run, and fixture identities never qualify a production package.
"""
from contextlib import ExitStack, redirect_stdout
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("warm_applicability", HERE / "check-warm-applicability.py")
A = importlib.util.module_from_spec(spec)
spec.loader.exec_module(A)


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def commit(root):
    git(root, "add", ".")
    git(root, "-c", "user.name=Applicability Fixture", "-c", "user.email=fixture@example.invalid",
        "commit", "--quiet", "--no-gpg-sign", "-m", "Synthetic applicability fixture")


class SourceControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="warm applicability ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.baseline = self.root / "baseline"
        self.baseline.mkdir()
        git(self.baseline, "init", "--quiet")
        for path in A.B.SHARED_SOURCE_PATHS:
            target = self.baseline / path
            if "." not in target.name:
                target /= "fixture.txt"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("Synthetic shared source: " + path + "\n")
        (self.baseline / "upstream/bluefe-core.pin").write_text('source = "' + A.BUILD_SOURCE + '"\n')
        proof_paths = ("compiler/native/src/ty/consteval.rs", "compiler/native/src/check/mod.rs",
                       "compiler/native/src/runtime/unicode_table.h")
        for path in proof_paths:
            target = self.baseline / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("Synthetic Unicode failure source " + path + "\n")
        commit(self.baseline)
        identity = A.B.source_identity(self.baseline)
        self.candidate = self.root / "candidate"
        shutil.copytree(self.baseline, self.candidate)
        original = json.loads((HERE.parent / A.FAILURE_RECORD).read_bytes())
        failure = original["native_complete"]
        failure["packages"]["baseline"]["source"] = identity
        proofs = []
        for path in proof_paths:
            data = (self.baseline / path).read_bytes()
            proofs.append({"path": path, "revision": identity["revision"],
                           "git_blob": git(self.baseline, "rev-parse", "HEAD:" + path),
                           "sha256": A.digest(data), "bytes": len(data)})
        failure["source_proof"], failure["runtime_unicode_table"] = proofs[:2], proofs[2]
        self.record = self.candidate / A.FAILURE_RECORD
        self.record.parent.mkdir(parents=True, exist_ok=True)
        self.record.write_text(json.dumps(original, indent=2) + "\n")
        commit(self.candidate)
        stack = self.enterContext(ExitStack())
        stack.enter_context(patch.multiple(A, COMPLETE_BASELINE=identity["revision"],
                                          COMPLETE_TREE=identity["tree"], FAILURE_SHA256=A.digest(self.record.read_bytes())))
        stack.enter_context(patch.multiple(A.N, BASELINE=identity["revision"], BASELINE_TREE=identity["tree"]))
        stack.enter_context(patch.object(A.B, "BASELINE", identity["revision"]))
        stack.enter_context(patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": A.BUILD_TOOLCHAIN}, clear=False))
        self.index = 0

    def run_check(self, campaign):
        self.index += 1
        output = self.root / ("evidence-" + str(self.index))
        args = SimpleNamespace(campaign=campaign, repository=self.candidate,
                               baseline_source=self.baseline, output=output)
        with redirect_stdout(io.StringIO()):
            code = A.check(args)
        report = json.loads((output / "applicability.json").read_text())
        self.assertEqual(report["qualification"], "unqualified")
        self.assertEqual(report["admitted_timing_samples"], 0)
        self.assertEqual(report["exit"], code)
        for row in (output / "SHA256SUMS").read_text().splitlines():
            digest, filename = row.split("  ")
            self.assertEqual(hashlib.sha256((output / filename).read_bytes()).hexdigest(), digest)
        return code, report, output

    def test_identical_entry_sources_only_admit_original_gates(self):
        code, report, _ = self.run_check("entry")
        self.assertEqual(code, 0)
        self.assertEqual(report["status"], "applicable")
        self.assertEqual(set(report["shared_source_objects"]), set(A.B.SHARED_SOURCE_PATHS))
        self.assertIn("Every original", report["reason"])

    def test_every_shared_source_difference_blocks_and_is_retained(self):
        for path in A.B.SHARED_SOURCE_PATHS:
            with self.subTest(path=path):
                target = self.candidate / path
                if target.is_dir():
                    target /= "fixture.txt"
                original = target.read_bytes()
                target.write_bytes(original + b"changed shared source\n")
                commit(self.candidate)
                code, report, _ = self.run_check("entry")
                self.assertEqual(code, 1)
                self.assertEqual(report["status"], "blocked")
                self.assertEqual(report["changed_shared_sources"], [path])
                self.assertEqual(report["reason_code"], "shared-runtime-source-differs")
                target.write_bytes(original)
                commit(self.candidate)

    def test_original_build_guard_still_rejects_before_using_package(self):
        path = self.candidate / "compiler/native/fixture.txt"
        path.write_text("different native runtime\n")
        commit(self.candidate)
        output = self.root / "original-build-guard"
        with self.assertRaisesRegex(ValueError, "shared runtime source differs: compiler/native"):
            A.B.build_baseline(SimpleNamespace(repository=self.candidate, baseline_source=self.baseline,
                                              built=self.root / "absent-package", output=output))
        self.assertEqual(json.loads((output / "receipt.json").read_text())["status"], "failed")

    def test_dirty_candidate_blocks_without_transferring_head_identity(self):
        (self.candidate / "library/fixture.txt").write_text("uncommitted\n")
        code, report, _ = self.run_check("entry")
        self.assertEqual(code, 1)
        self.assertEqual(report["reason_code"], "unverified-applicability")
        self.assertNotIn("candidate", report)

    def test_dirty_baseline_blocks(self):
        (self.baseline / "library/fixture.txt").write_text("uncommitted\n")
        code, report, _ = self.run_check("complete")
        self.assertEqual(code, 1)
        self.assertNotIn("historical_failure", report)

    def test_known_complete_failure_remains_blocked_and_retains_original_artifact(self):
        code, report, _ = self.run_check("complete")
        self.assertEqual(code, 1)
        self.assertEqual(report["status"], "blocked")
        self.assertEqual(report["reason_code"], "frozen-baseline-unicode-failure")
        self.assertEqual(report["historical_failure"]["artifact"]["zip_sha256"],
                         "1f9bc20f38c93469e8b791e0f7ce98a860c069815e47d6d9b41e275919aa5e2c")

    def test_changed_or_missing_history_is_unqualified(self):
        original = self.record.read_bytes()
        for data in (b"{}\n", original + b" "):
            with self.subTest(data=len(data)):
                self.record.write_bytes(data)
                commit(self.candidate)
                code, report, _ = self.run_check("complete")
                self.assertEqual(code, 1)
                self.assertEqual(report["reason_code"], "unverified-applicability")
                self.assertNotIn("historical_failure", report)
        self.record.unlink()
        commit(self.candidate)
        code, report, _ = self.run_check("complete")
        self.assertEqual(code, 1)
        self.assertEqual(report["error_type"], "FileNotFoundError")

    def test_recomputed_history_checksum_does_not_admit_wrong_semantics_or_source_proof(self):
        original = json.loads(self.record.read_text())
        mutations = (
            lambda f: f["differential"].update(exit=0),
            lambda f: f["differential"].update(rustc_unicode_version=[18, 0, 0]),
            lambda f: f["source_proof"][0].update(git_blob="f" * 40),
            lambda f: f["source_proof"][0].update(sha256="f" * 64),
            lambda f: f["build_sdk"].update(rustc_version="different SDK"),
            lambda f: f.update(admitted_timing_samples=1),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                data = copy.deepcopy(original)
                mutate(data["native_complete"])
                self.record.write_text(json.dumps(data, indent=2) + "\n")
                commit(self.candidate)
                with patch.object(A, "FAILURE_SHA256", A.digest(self.record.read_bytes())):
                    code, report, _ = self.run_check("complete")
                self.assertEqual(code, 1)
                self.assertEqual(report["reason_code"], "unverified-applicability")
                self.assertNotIn("historical_failure", report)

    def test_different_build_toolchain_cannot_reuse_known_failure(self):
        for value in ("1.99.0", "nightly", ""):
            with self.subTest(value=value), patch.dict(os.environ, RUSTUP_TOOLCHAIN=value):
                code, report, _ = self.run_check("complete")
                self.assertEqual(code, 1)
                self.assertNotIn("historical_failure", report)

    def test_changed_candidate_sdk_source_cannot_reuse_known_failure(self):
        (self.candidate / "upstream/bluefe-core.pin").write_text('source = "' + "f" * 40 + '"\n')
        commit(self.candidate)
        code, report, _ = self.run_check("complete")
        self.assertEqual(code, 1)
        self.assertNotIn("historical_failure", report)

    def test_changed_oracle_or_baseline_policy_is_never_implicitly_accepted(self):
        for name, value in (("NATIVE_ORACLE_RELEASE", "1.102.0-nightly"),
                            ("NATIVE_ORACLE_COMMIT", A.BUILD_SOURCE), ("BASELINE", "f" * 40),
                            ("BASELINE_TREE", "f" * 40)):
            with self.subTest(name=name), patch.object(A.N, name, value):
                code, report, _ = self.run_check("complete")
                self.assertEqual(code, 1)
                self.assertNotIn("historical_failure", report)

    def test_wrong_entry_baseline_is_rejected(self):
        with patch.object(A.B, "BASELINE", "f" * 40):
            code, report, _ = self.run_check("entry")
        self.assertEqual(code, 1)
        self.assertNotIn("shared_source_objects", report)

    def test_existing_evidence_is_never_overwritten(self):
        _, _, output = self.run_check("complete")
        before = {path.name: path.read_bytes() for path in output.iterdir()}
        with self.assertRaisesRegex(ValueError, "prior evidence is retained"):
            A.check(SimpleNamespace(campaign="complete", repository=self.candidate,
                                    baseline_source=self.baseline, output=output))
        self.assertEqual(before, {path.name: path.read_bytes() for path in output.iterdir()})

    def test_cli_has_no_override(self):
        output = self.root / "override-output"
        result = subprocess.run([sys.executable, str(HERE / "check-warm-applicability.py"), "complete",
                                 "--repository", str(self.candidate), "--baseline-source", str(self.baseline),
                                 "--output", str(output), "--override"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        self.assertIn("unrecognized arguments", result.stderr)
        self.assertFalse(output.exists())


def steps(text):
    starts = list(re.finditer(r"^      - (?:name|uses): .+$", text, re.M))
    return [text[match.start():starts[index + 1].start() if index + 1 < len(starts) else len(text)]
            for index, match in enumerate(starts)]


class WorkflowControls(unittest.TestCase):
    def test_active_historical_jobs_replay_without_constructing_compilers(self):
        for filename in ("warm-entry-performance.yml", "warm-native-performance.yml"):
            with self.subTest(workflow=filename):
                text = (HERE.parent / ".github/workflows" / filename).read_text()
                self.assertIn("Frozen evidence replay", text)
                self.assertIn("/replay.py", text)
                self.assertIn("Current performance remains unqualified", text)
                for forbidden in ("rustup toolchain install", "build-package --lane", "--current-sdk", " measure ", "continue-on-error"):
                    self.assertNotIn(forbidden, text)

    def test_native_selector_suites_keep_actual_package_and_raw_evidence(self):
        text = (HERE.parent / ".github/workflows/compiler-set.yml").read_text()
        section = text.split("  native-entry-and-services:\n", 1)[1].split("\n  compatibility:\n", 1)[0]
        blocks = steps(section)
        upload, = [block for block in blocks if "actions/upload-artifact@" in block]
        self.assertIn("if: always()", upload)
        for script, output in (("test-native-selection-cache.py", "native-selection-cache"),
                               ("test-native-selection-seed.py", "native-selection-seed"),
                               ("test-native-compact-selection.py", "native-compact-selection")):
            block, = [block for block in blocks if "python3 tools/" + script in block]
            self.assertIn('"$RUNNER_TEMP/lustr-native-built/toolchain/bin"', block)
            self.assertIn("steps.selector_build.outcome == 'success'", block)
            self.assertIn("!cancelled()", block)
            self.assertNotIn("continue-on-error", block)
            # Upload the whole evidence directory, including failure streams.
            self.assertIn("${{ runner.temp }}/" + output, upload)
        self.assertIn("--require-daemon", section)
        self.assertNotIn("continue-on-error", section)


if __name__ == "__main__":
    unittest.main(verbosity=2)
