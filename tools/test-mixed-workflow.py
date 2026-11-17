#!/usr/bin/env python3
"""Exercise the mixed CI failure guard and its explicit suite prerequisites.

Uses only the standard library. This checks the workflow's deliberately simple
condition subset; GitHub remains the interpreter of the complete workflow.
The shell probes are harness tests, not compiler qualification or benchmarks.
"""
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest

WORKFLOW = Path(__file__).resolve().parents[1] / ".github/workflows/compiler-set.yml"
MARKER = "link-failure-recovery: wrong application result"


def mixed_steps():
    text = WORKFLOW.read_text(encoding="utf-8")
    start = text.index("  mixed-source-build:\n")
    stop = text.index("\n  compatibility:\n", start)
    section = text[start:stop]
    starts = list(re.finditer(r"^      - (?:name|uses): .+$", section, re.MULTILINE))
    steps = {}
    for index, match in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(section)
        block = section[match.start():end]
        identity = re.search(r"^        id: ([a-z_]+)$", block, re.MULTILINE)
        if identity:
            name = identity.group(1)
            if name in steps:
                raise AssertionError(f"duplicate step id: {name}")
            steps[name] = block
    return section, steps


def dependencies(block):
    match = re.search(r"^        if: \$\{\{ (.+) \}\}$", block, re.MULTILINE)
    if not match:
        raise AssertionError("expected an explicit cancellation-aware condition")
    terms = match.group(1).split(" && ")
    if terms.pop(0) != "!cancelled()":
        raise AssertionError("condition must override implicit success without ignoring cancellation")
    result = []
    for term in terms:
        dependency = re.fullmatch(r"steps\.([a-z_]+)\.outcome == 'success'", term)
        if not dependency:
            raise AssertionError(f"unexpected condition term: {term}")
        result.append(dependency.group(1))
    if not result or len(result) != len(set(result)):
        raise AssertionError("prerequisites must be nonempty and unique")
    return result


class MixedWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.section, self.steps = mixed_steps()
        self.success = dict.fromkeys(self.steps, "success")

    def eligible(self, step, outcomes=None, cancelled=False):
        outcomes = self.success if outcomes is None else outcomes
        return not cancelled and all(outcomes.get(dep) == "success" for dep in dependencies(self.steps[step]))

    def shell_probe(self, status, diagnostic, missing_command=False):
        block = self.steps["baseline_regression"].split("        run: |\n", 1)[1]
        lines = []
        for line in block.splitlines():
            if line and not line.startswith("          "):
                break
            lines.append(line[10:] if line else "")
        script = "\n".join(lines) + "\n"
        bash = shutil.which("bash")
        self.assertIsNotNone(bash)
        with tempfile.TemporaryDirectory(prefix="lustr workflow ") as directory:
            root = Path(directory)
            commands = root / "commands"
            commands.mkdir()
            # Intentionally no ripgrep or ambient user tools on this PATH.
            for name in ("cat", "grep"):
                executable = shutil.which(name)
                self.assertIsNotNone(executable, name)
                (commands / name).symlink_to(executable)
            probe = root / "mixed-compiler-qualification"
            if not missing_command:
                probe.write_text(
                    f'#!{bash}\nprintf "%s\\n" "$PROBE_DIAGNOSTIC" >&2\nexit "$PROBE_STATUS"\n',
                    encoding="utf-8",
                )
                probe.chmod(0o755)
            environment = {
                "PATH": str(commands), "RUNNER_TEMP": str(root),
                "PROBE_DIAGNOSTIC": diagnostic, "PROBE_STATUS": str(status),
                "LC_ALL": "C",
            }
            result = subprocess.run(
                [bash, "--noprofile", "--norc", "-e", "-o", "pipefail", "-c", script],
                env=environment, capture_output=True, text=True, check=False, timeout=10,
            )
            recorded = (root / "baseline-regression.status").read_text().strip()
            return result, recorded

    def test_expected_failure_works_without_ripgrep(self):
        result, status = self.shell_probe(101, f"assertion failed: {MARKER}\nleft: 1080\nright: 1081")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(status, "101")
        self.assertIn(MARKER, result.stdout)

    def test_unrelated_failure_is_not_regression_evidence(self):
        result, status = self.shell_probe(101, "compiler setup failed")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(status, "101")

    def test_success_with_marker_is_not_regression_evidence(self):
        result, status = self.shell_probe(0, MARKER)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(status, "0")

    def test_missing_qualifier_is_not_regression_evidence(self):
        result, status = self.shell_probe(0, "", missing_command=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(status, "127")

    def test_dependencies_are_prior_steps_and_failures_are_not_masked(self):
        self.assertNotIn("continue-on-error:", self.section)
        known = set()
        for name, block in self.steps.items():
            if name != "source_toolchain":
                self.assertTrue(set(dependencies(block)) <= known, name)
            known.add(name)
        self.assertIn("        if: always()\n        uses: actions/upload-artifact@v4", self.section)

    def test_independent_suites_run_after_baseline_failure(self):
        outcomes = dict(self.success, baseline_regression="failure", baseline_toolchain="failure")
        for name in ("mixed_execution", "check_qualifier", "retained_checks", "library_qualifier", "mixed_libraries", "entry_contract"):
            with self.subTest(step=name):
                self.assertTrue(self.eligible(name, outcomes))
        self.assertFalse(self.eligible("measurements", outcomes))

    def test_independent_suites_run_after_execution_failure(self):
        outcomes = dict(self.success, mixed_execution="failure")
        for name in ("retained_checks", "mixed_libraries", "entry_contract"):
            self.assertTrue(self.eligible(name, outcomes), name)
        self.assertFalse(self.eligible("measurements", outcomes))

    def test_no_suite_runs_after_failed_toolchain(self):
        outcomes = dict.fromkeys(self.steps, "skipped")
        outcomes["source_toolchain"] = "failure"
        for name in self.steps:
            if name != "source_toolchain":
                self.assertFalse(self.eligible(name, outcomes), name)

    def test_cancellation_stops_every_guarded_step(self):
        for name in self.steps:
            if name != "source_toolchain":
                self.assertFalse(self.eligible(name, cancelled=True), name)

    def test_missing_qualifier_blocks_only_its_own_suite(self):
        for qualifier, suite in (("mixed_qualifier", "mixed_execution"), ("check_qualifier", "retained_checks"), ("library_qualifier", "mixed_libraries")):
            for outcome in ("failure", "skipped", "cancelled"):
                with self.subTest(qualifier=qualifier, outcome=outcome):
                    outcomes = dict(self.success, **{qualifier: outcome})
                    self.assertFalse(self.eligible(suite, outcomes))
                    self.assertTrue(self.eligible("entry_contract", outcomes))

    def test_measurements_require_every_correctness_gate(self):
        required = {"source_toolchain", "target_layout", "baseline_regression", "mixed_execution", "retained_checks", "mixed_libraries", "entry_contract", "pristine_build", "bench_build", "host_record"}
        self.assertEqual(set(dependencies(self.steps["measurements"])), required)
        self.assertTrue(self.eligible("measurements"))
        for gate in required:
            for outcome in ("failure", "skipped", "cancelled"):
                with self.subTest(gate=gate, outcome=outcome):
                    self.assertFalse(self.eligible("measurements", dict(self.success, **{gate: outcome})))

    def test_profiling_is_separate_from_timing_and_keeps_its_failure(self):
        self.assertTrue(self.eligible("mixed_profile"))
        self.assertFalse(self.eligible("mixed_profile", dict(self.success, mixed_execution="failure")))
        self.assertTrue(self.eligible("measurements", dict(self.success, mixed_profile="failure")))
        self.assertNotIn("LUSTR_MIXED_PROFILE_DIR", self.steps["measurements"])
        self.assertIn("lustr-mixed-profile", self.steps["mixed_profile"])
        self.assertIn("lustr-paired-measurements", self.steps["measurements"])

    def test_project_measurements_require_correctness_and_own_runner(self):
        required = {"source_toolchain", "target_layout", "baseline_regression", "mixed_execution", "retained_checks", "mixed_libraries", "entry_contract", "project_bench_build", "host_record"}
        self.assertEqual(set(dependencies(self.steps["project_measurements"])), required)
        for gate in required:
            for outcome in ("failure", "skipped", "cancelled"):
                with self.subTest(gate=gate, outcome=outcome):
                    self.assertFalse(self.eligible("project_measurements", dict(self.success, **{gate: outcome})))
        # Independent complete campaigns keep their evidence after a different
        # measurement/profiling failure, without making the overall job green.
        self.assertTrue(self.eligible("project_measurements", dict(self.success, measurements="failure", mixed_profile="failure")))
        self.assertNotIn("LUSTR_MIXED_PROFILE_DIR", self.steps["project_measurements"])

    def test_project_artifact_cannot_publish_private_raw_reports(self):
        marker = "      - name: Retain only the sanitized unchanged-project performance export\n"
        block = self.section.split(marker, 1)[1].split("      - name:", 1)[0]
        self.assertIn("        if: always()\n", block)
        paths = [line.strip() for line in block.splitlines() if "runner.temp" in line]
        expected = {f"${{{{ runner.temp }}}}/lustr-coordinator-campaign/public/{name}" for name in
                    ("README.md", "campaign.json", "samples.json", "replay.json", "host.json", "SHA256SUMS")}
        self.assertEqual(set(paths), expected)
        self.assertNotIn("/private", block)

    def test_originals_are_encrypted_after_failure_without_masking_it(self):
        marker = "      - name: Encrypt original project evidence after success or failure\n"
        block = self.section.split(marker, 1)[1].split("      - name:", 1)[0]
        self.assertIn("        if: always()\n", block)
        self.assertIn("--expected-certificate-sha256 69e1167422ba3edfc45139eda5bdcec97d5bebcdf3e764054f3457488aaa5c4f", block)
        self.assertIn('tools/benchmark-evidence-recipient.pem"', block)
        self.assertNotIn("continue-on-error", block)
        self.assertNotIn("|| true", block)
        self.assertNotIn("PRIVATE", block)

    def test_encrypted_artifact_upload_has_only_fixed_outputs(self):
        marker = "      - name: Retain encrypted project originals and explicit retention status\n"
        block = self.section.split(marker, 1)[1].split("      - name:", 1)[0]
        self.assertIn("        if: always()\n", block)
        self.assertIn("          if-no-files-found: error\n", block)
        paths = [line.strip() for line in block.splitlines() if "runner.temp" in line]
        expected = {f"${{{{ runner.temp }}}}/lustr-coordinator-campaign/encrypted/{name}" for name in
                    ("private-evidence.tar.gz.cms", "receipt.json", "status.json")}
        self.assertEqual(set(paths), expected)
        self.assertNotIn("/private/", block)


if __name__ == "__main__":
    unittest.main(verbosity=2)
