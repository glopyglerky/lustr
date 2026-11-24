#!/usr/bin/env python3
"""Failure controls for the independent retained-history qualifier.

Synthetic verdicts exercise admission, not compiler correctness. Process tests
exercise the observer's actual public-command, artifact and single-run boundary.
"""
import argparse
import copy
import importlib.util
import json
import os
from pathlib import Path
import signal
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("retained_results", ROOT / "tools/qualify-retained-results.py")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
ORACLE = None


def result(code=0, stdout=b"right\n", stderr=b""):
    return {"exit": code, "stdout": stdout, "stderr": stderr, "timed_out": False}


def observation():
    return {"checks": {mode: result() for mode in ("warm", "cold")},
            "builds": {mode: {"compile": result(), "artifact_exists": True, "artifact_sha256": "1" * 64, "runtime": result()} for mode in ("warm", "cold")},
            "oracle_build": dict(result(), tool="rustc"), "oracle_runtime": result()}


class VerdictControls(unittest.TestCase):
    def setUp(self):
        self.observed = observation()

    def evaluate(self):
        return runner.compare_version(self.observed, ["check", "run"])

    def test_exact_complete_result(self):
        self.assertEqual(self.evaluate(), "executed")

    def test_both_valid_rejections_are_a_regression(self):
        for mode in ("warm", "cold"):
            self.observed["checks"][mode] = result(1, stderr=b"error[L0001]: rejected")
            self.observed["builds"][mode] = {"compile": result(1, stderr=b"error[L0001]: rejected"), "artifact_exists": False}
        with self.assertRaisesRegex(ValueError, "rejected Rust accepted"):
            self.evaluate()

    def test_both_checks_cannot_crash_as_a_reject(self):
        for mode in ("warm", "cold"):
            self.observed["checks"][mode] = result(-signal.SIGSEGV)
        with self.assertRaisesRegex(ValueError, "signalled"):
            self.evaluate()

    def test_check_failure_cannot_hide_behind_successful_build(self):
        self.observed["checks"]["warm"] = result(1, stderr=b"error[L0101]: absent")
        with self.assertRaisesRegex(ValueError, "candidate check rejected"):
            self.evaluate()

    def test_stdout_is_compared(self):
        self.observed["builds"]["warm"]["runtime"]["stdout"] = b"wrong\n"
        with self.assertRaisesRegex(ValueError, "stdout"):
            self.evaluate()

    def test_stderr_is_compared(self):
        self.observed["builds"]["cold"]["runtime"]["stderr"] = b"unexpected\n"
        with self.assertRaisesRegex(ValueError, "stderr"):
            self.evaluate()

    def test_exit_is_compared(self):
        self.observed["builds"]["warm"]["runtime"]["exit"] = 7
        with self.assertRaisesRegex(ValueError, "exit differs"):
            self.evaluate()

    def test_program_nonzero_exit_is_compared_as_execution(self):
        for mode in ("warm", "cold"):
            self.observed["builds"][mode]["runtime"]["exit"] = 7
        self.observed["oracle_runtime"]["exit"] = 7
        self.assertEqual(self.evaluate(), "executed")

    def test_unexpected_signal_is_not_success(self):
        for mode in ("warm", "cold"):
            self.observed["builds"][mode]["runtime"]["exit"] = -signal.SIGABRT
        self.observed["oracle_runtime"]["exit"] = -signal.SIGABRT
        with self.assertRaisesRegex(ValueError, "signalled"):
            self.evaluate()
        self.assertEqual(runner.compare_version(self.observed, ["check", "run"], [signal.SIGABRT]), "executed")

    def test_expected_program_signal_does_not_allow_compiler_signal(self):
        self.observed["oracle_build"]["exit"] = -signal.SIGABRT
        with self.assertRaisesRegex(ValueError, "signalled"):
            runner.compare_version(self.observed, ["check", "run"], [signal.SIGABRT])

    def test_timeout_cannot_be_a_reject(self):
        self.observed["oracle_build"]["timed_out"] = True
        with self.assertRaisesRegex(ValueError, "interrupted"):
            self.evaluate()

    def test_missing_artifact(self):
        self.observed["builds"]["warm"]["artifact_exists"] = False
        with self.assertRaisesRegex(ValueError, "executable result"):
            self.evaluate()

    def test_missing_runtime(self):
        del self.observed["builds"]["warm"]["runtime"]
        with self.assertRaisesRegex(ValueError, "executable result"):
            self.evaluate()

    def test_missing_check(self):
        del self.observed["checks"]["cold"]
        with self.assertRaisesRegex(ValueError, "check result"):
            self.evaluate()

    def test_missing_build(self):
        del self.observed["builds"]["warm"]
        with self.assertRaisesRegex(ValueError, "build result"):
            self.evaluate()

    def test_missing_oracle_runtime(self):
        del self.observed["oracle_runtime"]
        with self.assertRaisesRegex(ValueError, "oracle has no runtime"):
            self.evaluate()

    def test_check_only_version_still_requires_cold_artifact(self):
        del self.observed["builds"]["warm"]
        self.assertEqual(runner.compare_version(self.observed, ["check"]), "executed")
        del self.observed["builds"]["cold"]
        with self.assertRaisesRegex(ValueError, "build result"):
            runner.compare_version(self.observed, ["check"])

    def test_invalid_program_requires_actual_native_rejections(self):
        rejected = result(1, stdout=b"", stderr=b"error[L0101]: missing")
        self.observed = {"checks": {m: copy.deepcopy(rejected) for m in ("warm", "cold")},
                         "builds": {m: {"compile": copy.deepcopy(rejected), "artifact_exists": False} for m in ("warm", "cold")},
                         "oracle_build": dict(result(1, stderr=b"error[E0425]: missing"), tool="rustc"),
                         "oracle_semantic_check": dict(result(1), source_rejection=True)}
        self.assertEqual(self.evaluate(), "rejected")
        self.observed["builds"]["warm"]["compile"] = result(2, stderr=b"file not found")
        with self.assertRaisesRegex(ValueError, "native diagnostic"):
            self.evaluate()

    def test_invalid_oracle_cannot_be_an_internal_error(self):
        self.observed["oracle_build"] = dict(result(101, stderr=b"error: internal compiler error"), tool="rustc")
        self.observed["oracle_semantic_check"] = dict(result(1), source_rejection=True)
        with self.assertRaisesRegex(ValueError, "unexpected status"):
            self.evaluate()

    def test_linker_failure_cannot_classify_valid_source_as_invalid(self):
        self.observed["oracle_build"] = dict(result(1, stderr=b"error: linking with cc failed"), tool="rustc")
        self.observed["oracle_semantic_check"] = dict(result(0), source_rejection=False)
        with self.assertRaisesRegex(ValueError, "semantic source rejection"):
            self.evaluate()

    def test_reviewed_removed_target_has_a_distinct_verdict(self):
        rejected = dict(result(2, stdout=b""), selection_rejection=True)
        self.observed = {"checks": {mode: rejected.copy() for mode in ("warm", "cold")},
                         "builds": {mode: {"compile": rejected.copy(), "artifact_exists": False} for mode in ("warm", "cold")},
                         "oracle_build": dict(result(101), tool="cargo", selection_rejection=True),
                         "oracle_selection_check": dict(result(101), selection_rejection=True)}
        self.assertEqual(runner.compare_version(self.observed, ["check", "run"], selection_rejection=True), "selection-rejected")
        with self.assertRaisesRegex(ValueError, "no separate no-link"):
            self.evaluate()
        self.observed["builds"]["warm"]["runtime"] = result()
        with self.assertRaisesRegex(ValueError, "produced or executed"):
            runner.compare_version(self.observed, ["check", "run"], selection_rejection=True)

    def test_removed_target_requires_both_pinned_cargo_rejections(self):
        self.observed["oracle_build"] = dict(result(101), tool="cargo", selection_rejection=True)
        self.observed["oracle_selection_check"] = dict(result(0), selection_rejection=False)
        with self.assertRaisesRegex(ValueError, "Cargo check oracle"):
            runner.compare_version(self.observed, ["check", "run"], selection_rejection=True)

    def test_only_run_specific_thread_id_is_normalized(self):
        runner.compare_runtime(result(101, stderr=b"thread 'main' (10) panicked at main.rs:3:4:\nvalue\n"), result(101, stderr=b"thread 'main' (20) panicked at main.rs:3:4:\nvalue\n"), panic_thread_id=True)
        with self.assertRaisesRegex(ValueError, "stderr"):
            runner.compare_runtime(result(101, stderr=b"thread 'main' (10) panicked at main.rs:2:4:\nvalue\n"), result(101, stderr=b"thread 'main' (20) panicked at main.rs:3:4:\nvalue\n"), panic_thread_id=True)

    def test_ordinary_stderr_numbers_are_never_normalized(self):
        with self.assertRaisesRegex(ValueError, "stderr"):
            runner.compare_runtime(result(stderr=b"thread 'main' (10) value\n"), result(stderr=b"thread 'main' (20) value\n"))
        with self.assertRaisesRegex(ValueError, "stderr"):
            runner.compare_runtime(result(101, stderr=b"thread 'main' (10) panicked at main.rs:3:4:\nvalue\n"), result(101, stderr=b"thread 'main' (20) panicked at main.rs:3:4:\nvalue\n"))


class InventoryControls(unittest.TestCase):
    def test_current_fixtures_match_pinned_donor_inventory(self):
        inventory = runner.load_inventory(ROOT, ROOT / "upstream/retained-scenario-results.json")
        self.assertEqual(sum(row["status"] == "required" for row in inventory["scenarios"]), 349)
        self.assertEqual(sum(row["status"] == "unresolved" for row in inventory["scenarios"]), 13)

    def test_required_history_cannot_be_reclassified_as_unresolved(self):
        inventory = json.loads((ROOT / "upstream/retained-scenario-results.json").read_text())
        required = next(row for row in inventory["scenarios"] if row["status"] == "required")
        required.update(status="unresolved", historical_unsupported_versions=1)
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "inventory.json"
            manifest.write_text(json.dumps(inventory))
            with self.assertRaisesRegex(ValueError, "classification differs"):
                runner.load_inventory(ROOT, manifest)

    def test_upstream_history_cannot_be_excluded_as_integration(self):
        inventory = json.loads((ROOT / "upstream/retained-scenario-results.json").read_text())
        removed = inventory["scenarios"].pop(0)
        inventory["integration_histories"].append({"name": removed["name"]})
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "inventory.json"
            manifest.write_text(json.dumps(inventory))
            with self.assertRaisesRegex(ValueError, "reclassified"):
                runner.load_inventory(ROOT, manifest)

    def test_declared_upstream_tree_cannot_change_with_candidate(self):
        inventory = json.loads((ROOT / "upstream/retained-scenario-results.json").read_text())
        inventory["upstream_scenario_tree"] = "0" * 40
        with tempfile.TemporaryDirectory() as temporary:
            manifest = Path(temporary) / "inventory.json"
            manifest.write_text(json.dumps(inventory))
            with self.assertRaisesRegex(ValueError, "upstream revision/tree"):
                runner.load_inventory(ROOT, manifest)

    def test_git_tree_matches_git_on_real_fixture(self):
        path = "compiler/native/tests/daemon/mainret01_exitcode_edits"
        expected = subprocess.check_output(["git", "-C", str(ROOT), "rev-parse", "HEAD:" + path], text=True).strip()
        self.assertEqual(runner.tree_id(ROOT / path), expected)

    def test_changed_or_missing_fixtures_change_identity(self):
        with tempfile.TemporaryDirectory() as name:
            path = Path(name)
            (path / "a").write_text("right")
            before = runner.tree_id(path)
            (path / "a").write_text("wrong")
            self.assertNotEqual(runner.tree_id(path), before)
            (path / "a").unlink()
            self.assertNotEqual(runner.tree_id(path), before)


class OracleDiagnosticControls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.cwd = Path(self.temp.name)
        (self.cwd / "main.rs").write_text("fn main() { missing(); }")
        self.span = {"is_primary": True, "file_name": "main.rs"}
        self.diagnostic = {"level": "error", "message": "source error", "spans": [self.span]}

    def rejection(self, *diagnostics):
        return runner.source_rejection(result(1, stderr=b"\n".join(json.dumps(row).encode() for row in diagnostics)), self.cwd)

    def test_rust_and_cargo_require_source_bound_structured_errors(self):
        with tempfile.TemporaryDirectory() as name:
            cwd = Path(name)
            (cwd / "main.rs").write_text("fn main() { missing(); }")
            diagnostic = {"level": "error", "message": "cannot find function missing", "spans": [{"is_primary": True, "file_name": "main.rs"}]}
            direct = result(1, stdout=b"", stderr=json.dumps(diagnostic).encode())
            cargo = result(101, stdout=json.dumps({"reason": "compiler-message", "message": diagnostic}).encode(), stderr=b"error: could not compile")
            self.assertTrue(runner.source_rejection(direct, cwd))
            self.assertTrue(runner.source_rejection(cargo, cwd, cargo=True))
            summary = {"level": "error", "message": "aborting due to 1 previous error; 1 warning emitted", "spans": []}
            with_warning = result(1, stderr=json.dumps(diagnostic).encode() + b"\n" + json.dumps(summary).encode())
            self.assertTrue(runner.source_rejection(with_warning, cwd))
            self.assertFalse(runner.source_rejection(result(1, stderr=b"error: linking with cc failed"), cwd))
            diagnostic["spans"] = []
            self.assertFalse(runner.source_rejection(result(1, stderr=json.dumps(diagnostic).encode()), cwd))
            diagnostic["spans"] = [{"is_primary": True, "file_name": "/unrelated/source.rs"}]
            self.assertFalse(runner.source_rejection(result(1, stderr=json.dumps(diagnostic).encode()), cwd))
            self.assertFalse(runner.source_rejection(result(101, stderr=b"error: malformed Cargo.toml"), cwd, cargo=True))

    def test_primary_macro_call_sites_can_anchor_nested_expansions(self):
        self.span.update(file_name="/rustc/core/macros.rs", expansion={"span": {"file_name": "/external/macro.rs", "expansion": {"span": {"file_name": "main.rs"}}}})
        self.assertTrue(self.rejection(self.diagnostic))

    def test_external_missing_and_definition_only_call_sites_fail(self):
        for call_site in ({"file_name": "/external/macro.rs"}, {"file_name": "missing.rs"}, {"file_name": "/external/macro.rs", "def_site_span": {"file_name": "main.rs"}}):
            with self.subTest(call_site=call_site):
                self.span.update(file_name="/rustc/core/macros.rs", expansion={"span": call_site, "def_site_span": {"file_name": "main.rs"}})
                self.assertFalse(self.rejection(self.diagnostic))

    def test_notes_and_secondary_source_spans_cannot_anchor_errors(self):
        self.span["file_name"] = "/external/error.rs"
        self.diagnostic["children"] = [{"level": "note", "spans": [{"is_primary": True, "file_name": "main.rs"}]}]
        self.diagnostic["spans"].append({"is_primary": False, "file_name": "main.rs"})
        self.assertFalse(self.rejection(self.diagnostic))

    def test_good_source_error_cannot_hide_unanchored_fatal_error(self):
        fatal = {"level": "error", "message": "could not find the required SDK", "spans": []}
        self.assertFalse(self.rejection(self.diagnostic, fatal))

    def test_missing_sdk_diagnostic_is_not_a_source_error_despite_its_span(self):
        self.diagnostic.update(code={"code": "E0463"}, message="can't find crate for `std`")
        self.assertFalse(self.rejection(self.diagnostic))

    def test_dependency_artifact_errors_cannot_use_the_imports_source_span(self):
        self.diagnostic.update(code=None, message="extern location for dep does not exist: missing.rlib")
        self.assertFalse(self.rejection(self.diagnostic))
        self.diagnostic.update(code={"code": "E0786"}, message="found invalid metadata files for crate `dep`")
        self.assertFalse(self.rejection(self.diagnostic))

    def test_mismatched_and_ambiguous_dependency_artifacts_are_operational_errors(self):
        for code, message in (("E0460", "found possibly newer version of crate `dep`"), ("E0464", "multiple candidates for rlib dependency `dep` found"), ("E0514", "found crate `dep` compiled by an incompatible version of rustc")):
            with self.subTest(code=code):
                self.diagnostic.update(code={"code": code}, message=message)
                self.assertFalse(self.rejection(self.diagnostic))

    def test_cargo_source_errors_cannot_hide_compiler_or_build_crashes(self):
        stdout = json.dumps({"reason": "compiler-message", "message": self.diagnostic}).encode()
        for stderr in (b"thread 'rustc' (123) panicked at rustc.rs:1:1", b"error: internal compiler error: broken", b"error: the compiler unexpectedly panicked", b"Caused by:\n  process didn't exit successfully: rustc (signal: 11, SIGSEGV)", b"error: could not execute process rustc (never executed)", b"signal: 6, SIGABRT", b"error: failed to run custom build command for dep"):
            with self.subTest(stderr=stderr):
                self.assertFalse(runner.source_rejection(result(101, stdout=stdout, stderr=stderr), self.cwd, cargo=True))
        failure = {"level": "failure-note", "message": "the compiler unexpectedly panicked"}
        stdout += b"\n" + json.dumps({"reason": "compiler-message", "message": failure}).encode()
        self.assertFalse(runner.source_rejection(result(101, stdout=stdout, stderr=b"error: could not compile"), self.cwd, cargo=True))

    def test_malformed_diagnostics_and_internal_status_fail(self):
        for encoded in (b"{malformed", b"[]", b'{"level":"error","spans":[null]}'):
            with self.subTest(encoded=encoded):
                self.assertFalse(runner.source_rejection(result(1, stderr=encoded), self.cwd))
        self.assertFalse(runner.source_rejection(result(101, stderr=json.dumps(self.diagnostic).encode()), self.cwd))

    def test_missing_target_is_not_a_generic_cargo_failure_allowance(self):
        rule = json.loads((ROOT / "upstream/retained-scenario-results.json").read_text())["selection_rejections"]["cargo01_bin_discovery"]["v7"]
        self.assertTrue(runner.removed_target_rejection(result(101, stdout=b"", stderr=rule["cargo_stderr"].encode()), rule, cargo=True))
        for stderr in (b"error: malformed Cargo.toml\n", b"error: linking with cc failed\n", rule["cargo_stderr"].replace("`e`", "`other`").encode()):
            with self.subTest(stderr=stderr):
                self.assertFalse(runner.removed_target_rejection(result(101, stdout=b"", stderr=stderr), rule, cargo=True))


class ProcessControls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)
        self.entry = self.directory / "lustr"
        self.counter = self.directory / "executions"
        program = "#!" + sys.executable + "\nfrom pathlib import Path\np=Path(" + repr(str(self.counter)) + ")\np.open('a').write('x')\nprint('right')\n"
        self.entry.write_text("#!" + sys.executable + "\nimport json,os,sys\nfrom pathlib import Path\nargs=sys.argv[1:]\ncode=int(os.environ.get('CONTROL_COMPILE_EXIT','0'))\nif code==0 and args[0]=='build' and not os.environ.get('CONTROL_OMIT_ARTIFACT'):\n p=Path(args[args.index('-o')+1]);p.write_text(" + repr(program) + ");p.chmod(0o755)\nif not os.environ.get('CONTROL_OMIT_TRACE'): Path(os.environ['LUSTR_TRACE_PATH']).write_text(json.dumps({'engine':'lust' if code==0 else 'lust-failed','entry':'native-library' if os.environ.get('LUST_NO_DAEMON') else 'daemon'})+'\\n')\nif code: print(os.environ.get('CONTROL_ERROR','error[L0101]: control reject'),file=sys.stderr)\nsys.exit(code)\n")
        self.entry.chmod(0o755)
        self.observer = runner.Observer({"source": str(ROOT), "output": str(self.directory), "entry": str(self.entry), "rustc": sys.executable, "cargo": sys.executable}, "mainret01_exitcode_edits")
        self.env = dict(os.environ, LUST_NO_DAEMON="1")
        self.target = self.directory / "program"

    def tearDown(self):
        self.temp.cleanup()

    def request(self):
        return self.observer.run([str(self.entry), "run", "main.rs", "-o", str(self.target)], self.directory, self.env)

    def test_build_then_one_execution_and_bound_artifact(self):
        code, stdout, _ = self.request()
        self.assertEqual((code, stdout), (0, "right\n"))
        self.assertEqual(self.counter.read_text(), "x")
        self.assertEqual(len(self.observer.report["commands"]), 2)
        self.assertTrue(self.observer.pending["builds"]["cold"]["artifact_sha256"])

    def test_compile_failure_never_executes_or_retries(self):
        self.env["CONTROL_COMPILE_EXIT"] = "1"
        self.target.write_text("stale output")
        self.assertEqual(self.request()[0], 1)
        self.assertFalse(self.target.exists())
        self.assertFalse(self.counter.exists())
        self.assertEqual(len(self.observer.report["commands"]), 1)

    def test_zero_exit_without_artifact_fails(self):
        self.env["CONTROL_OMIT_ARTIFACT"] = "1"
        with self.assertRaisesRegex(ValueError, "did not produce"):
            self.request()
        self.assertFalse(self.counter.exists())

    def test_stopped_original_daemon_cannot_be_replaced(self):
        self.env.pop("LUST_NO_DAEMON")
        self.observer.daemons = [Mock(poll=Mock(return_value=1))]
        with self.assertRaisesRegex(ValueError, "not alive before"):
            self.request()
        self.assertFalse(self.counter.exists())
        self.assertEqual(len(self.observer.report["commands"]), 0)

    def test_daemon_exit_during_request_invalidates_trace(self):
        self.env.pop("LUST_NO_DAEMON")
        self.observer.daemons = [Mock(poll=Mock(side_effect=[None, 1]))]
        with self.assertRaisesRegex(ValueError, "exited during"):
            self.request()
        self.assertFalse(self.counter.exists())

    def test_unreviewed_precompilation_refusal_reports_actual_diagnostic(self):
        self.env.update(CONTROL_COMPILE_EXIT="2", CONTROL_OMIT_TRACE="1", CONTROL_ERROR="error: blocked Cargo graph")
        with self.assertRaisesRegex(ValueError, "public-refusal-before-owner: exit 2.*blocked Cargo graph"):
            self.request()
        self.assertFalse(self.counter.exists())
        self.assertEqual(self.observer.report["commands"][-1]["classification"], "public-refusal-before-owner")

    def test_success_without_owner_trace_fails(self):
        self.env["CONTROL_OMIT_TRACE"] = "1"
        with self.assertRaisesRegex(ValueError, "missing-owner-trace: exit 0"):
            self.request()
        self.assertFalse(self.counter.exists())

    def test_removed_target_expectation_cannot_cover_a_present_source(self):
        rule = json.loads((ROOT / "upstream/retained-scenario-results.json").read_text())["selection_rejections"]["cargo01_bin_discovery"]["v7"]
        path = self.directory / rule["source"]
        path.parent.mkdir(parents=True)
        path.write_text("fn main() {}")
        (self.directory / "Cargo.toml").write_text("[package]\nname='probe'\nversion='0.0.0'\n")
        argv = [str(self.entry), "check", rule["source"], "--manifest-path", "Cargo.toml", "--bin", "e"]
        with self.assertRaisesRegex(ValueError, "target is present"):
            self.observer.require_removed_target(argv, self.directory, rule)


class FixtureLifecycleControls(unittest.TestCase):
    """Run the unchanged donor loader with synthetic processes, never a compiler claim."""
    def campaign(self, stale):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            source, output, package = root / "source", root / "output", root / "package"
            scripts = source / "compiler/native/scripts"
            fixture = source / "compiler/native/tests/daemon/probe"
            scripts.mkdir(parents=True)
            fixture.mkdir(parents=True)
            output.mkdir()
            package.mkdir()
            for filename in ("test-daemon-scenarios.py", "oracle_log.py"):
                shutil.copy2(ROOT / "compiler/native/scripts" / filename, scripts / filename)
            (fixture / "v1.rs").write_text('fn main() { println!("one"); }\n')
            (fixture / "v2.rs").write_text('fn main() { missing(); }\n')
            (fixture / "v3.rs").write_text('fn main() { println!("three"); }\n')
            program = r'''
import json,os,re,signal,sys
from pathlib import Path
args=sys.argv[1:]
if args==['daemon']:
    Path(os.environ['LUST_DAEMON_SOCKET']).touch()
    signal.pause()
native=Path(sys.argv[0]).name=='lustr'
source=Path(next(a for a in args if a.endswith('.rs'))).read_text()
code=1 if 'missing' in source else 0
if native:
    Path(os.environ['LUSTR_TRACE_PATH']).write_text(json.dumps({'engine':'lust' if code==0 else 'lust-failed','entry':'native-library' if os.environ.get('LUST_NO_DAEMON') else 'daemon'})+'\n')
if code:
    if '--error-format=json' in args:
        print(json.dumps({'level':'error','message':'missing name','spans':[{'is_primary':True,'file_name':next(a for a in args if a.endswith('.rs'))}]}),file=sys.stderr)
    else:
        print('error[L0101]: missing' if native else 'error[E0425]: missing',file=sys.stderr)
elif not (native and args[0]=='check'):
    word=re.search(r'println!\("([^\"]+)"\)',source).group(1)
    if native and STALE and word=='three': word='one'
    target=Path(args[args.index('-o')+1])
    target.write_text('#!'+sys.executable+'\nprint('+repr(word)+')\n')
    target.chmod(0o755)
sys.exit(code)
'''.replace("STALE", repr(stale))
            for filename in ("lustr", "lust", "rustc"):
                path = package / filename
                path.write_text("#!" + sys.executable + "\n" + program)
                path.chmod(0o755)
            config = {"source": str(source), "output": str(output), "entry": str(package / "lustr"), "rustc": str(package / "rustc"), "cargo": str(package / "rustc")}
            report = runner.scenario_worker((config, "probe"))
            self.assertTrue((output / "probe/daemon-0.stderr").is_file())
            return report

    def test_accept_reject_recovery_runs_through_original_fixture_loader(self):
        report = self.campaign(False)
        self.assertTrue(report["passed"], report.get("failure"))
        self.assertEqual([row["verdict"] for row in report["versions"]], ["executed", "rejected", "executed"])

    def test_stale_recovered_output_fails_original_fixture_lifecycle(self):
        report = self.campaign(True)
        self.assertFalse(report["passed"])
        self.assertIn("stdout differs", report["failure"])
        self.assertEqual(len(report["versions"]), 2)

    def cargo_campaign(self, stale):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            source, output, package = root / "source", root / "output", root / "package"
            scripts = source / "compiler/native/scripts"
            scenario = "cargo01_bin_discovery"
            scripts.mkdir(parents=True)
            output.mkdir()
            package.mkdir()
            for filename in ("test-daemon-scenarios.py", "oracle_log.py"):
                shutil.copy2(ROOT / "compiler/native/scripts" / filename, scripts / filename)
            shutil.copytree(ROOT / "compiler/native/tests/daemon" / scenario, source / "compiler/native/tests/daemon" / scenario)
            program = r'''
import json,os,re,signal,sys
from pathlib import Path
args=sys.argv[1:]
if args==['daemon']:
    Path(os.environ['LUST_DAEMON_SOCKET']).touch()
    signal.pause()
native=Path(sys.argv[0]).name=='lustr'
selected=args[args.index('--bin')+1]
source=Path(args[1]) if native else Path('src/bin/'+selected+'.rs')
if not native and not source.exists() and Path('src/bin/'+selected+'/main.rs').exists():
    source=Path('src/bin/'+selected+'/main.rs')
if not source.exists():
    print('error: invalid Cargo target selection: No such file or directory (os error 2)' if native else 'error: no bin target named `e` in default-run packages\n\nhelp: a target with a similar name exists: `c`',file=sys.stderr)
    sys.exit(2 if native else 101)
if native:
    Path(os.environ['LUSTR_TRACE_PATH']).write_text(json.dumps({'engine':'lust','entry':'native-library' if os.environ.get('LUST_NO_DAEMON') else 'daemon'})+'\n')
if args[0]=='build':
    word=re.search(r'println!\("([^\"]+)"\)',source.read_text()).group(1)
    if native and STALE and word=='e2': word='e'
    target=Path(args[args.index('-o')+1]) if native else Path.cwd()/'oracle.bin'
    target.write_text('#!'+sys.executable+'\nprint('+repr(word)+')\n')
    target.chmod(0o755)
    if not native: print(json.dumps({'reason':'compiler-artifact','target':{'kind':['bin']},'executable':str(target)}))
'''.replace("STALE", repr(stale))
            for filename in ("lustr", "lust", "cargo"):
                path = package / filename
                path.write_text("#!" + sys.executable + "\n" + program)
                path.chmod(0o755)
            inventory = json.loads((ROOT / "upstream/retained-scenario-results.json").read_text())
            config = {"source": str(source), "output": str(output), "entry": str(package / "lustr"), "rustc": sys.executable, "cargo": str(package / "cargo"), "selection_rejections": inventory["selection_rejections"]}
            return runner.scenario_worker((config, scenario))

    def test_removed_target_and_recovery_use_unchanged_donor_overlay(self):
        report = self.cargo_campaign(False)
        self.assertTrue(report["passed"], report.get("failure"))
        self.assertEqual([row["verdict"] for row in report["versions"]], ["executed"] * 6 + ["selection-rejected", "executed"])
        rejected = [row for row in report["commands"] if row.get("selection_rejection")]
        self.assertEqual(len(rejected), 4)
        self.assertTrue(all(row["exit"] == 2 and "trace" not in row for row in rejected))

    def test_removed_target_cannot_hide_stale_recovery(self):
        report = self.cargo_campaign(True)
        self.assertFalse(report["passed"])
        self.assertIn("stdout differs", report["failure"])
        self.assertEqual(len(report["versions"]), 7)


class PinnedOracleControls(unittest.TestCase):
    """Actual pinned Rust processes; optional locally, required by constructor CI."""
    def setUp(self):
        if ORACLE is None:
            self.skipTest("supply --oracle-rustc, --oracle-cargo and --oracle-output for pinned process controls")
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.cwd = Path(self.temp.name)
        self.output = ORACLE["output"] / self._testMethodName
        self.output.mkdir()
        self.commands = []

    def execute(self, tool, *args):
        completed = subprocess.run([str(ORACLE[tool]), *map(str, args)], cwd=self.cwd, env=dict(os.environ, RUSTC=str(ORACLE["rustc"]), CARGO_TARGET_DIR=str(self.cwd / "target")), capture_output=True, timeout=60)
        value = result(completed.returncode, completed.stdout, completed.stderr)
        label = f"{len(self.commands):02d}"
        for stream in ("stdout", "stderr"):
            (self.output / (label + "." + stream)).write_bytes(value[stream])
        self.commands.append({"tool": tool, "argv": list(map(str, args)), "exit": value["exit"], "stdout": label + ".stdout", "stderr": label + ".stderr", "source_rejection": runner.source_rejection(value, self.cwd, cargo=tool == "cargo")})
        (self.output / "commands.json").write_text(json.dumps(self.commands, indent=2) + "\n")
        return value

    def rustc(self, *args):
        return self.execute("rustc", "--edition=2024", "-C", "overflow-checks=on", "main.rs", "--error-format=json", *args)

    def test_codegen_source_errors_are_not_lost_by_metadata_only_checks(self):
        for name, version in (("cyc09_allow_nested_chunk", "v4"), ("sc02_assoc_const_request_edits", "v2"), ("sc03_assoc_const_plain_header_template", "v2"), ("sc04_assoc_const_template_body", "v2")):
            with self.subTest(scenario=name):
                shutil.copy2(ROOT / "compiler/native/tests/daemon" / name / (version + ".rs"), self.cwd / "main.rs")
                self.assertEqual(self.rustc("--emit=metadata", "-o", "metadata.rmeta")["exit"], 0)
                checked = self.rustc("--emit=llvm-ir", "-o", "check.ll")
                self.assertTrue(runner.source_rejection(checked, self.cwd))
                self.assertFalse((self.cwd / "check.ll").exists())

    def test_real_std_macro_errors_resolve_through_primary_call_sites(self):
        shutil.copy2(ROOT / "compiler/native/tests/daemon/sd23_std_impl_partialeq/v2.rs", self.cwd / "main.rs")
        checked = self.rustc("--emit=llvm-ir", "-o", "check.ll")
        diagnostics = [json.loads(line) for line in checked["stderr"].splitlines()]
        self.assertTrue(any(span.get("is_primary") and span.get("expansion") for error in diagnostics for span in error.get("spans", [])))
        self.assertTrue(runner.source_rejection(checked, self.cwd))

    def test_linker_and_sdk_failures_remain_failures(self):
        (self.cwd / "main.rs").write_text("fn main() {}\n")
        failed = self.rustc("-C", "linker=./absent-linker", "-o", "program")
        self.assertEqual(failed["exit"], 1)
        self.assertFalse(runner.source_rejection(failed, self.cwd))
        checked = self.rustc("-C", "linker=./absent-linker", "--emit=llvm-ir", "-o", "check.ll")
        self.assertEqual(checked["exit"], 0)
        self.assertFalse(runner.source_rejection(checked, self.cwd))
        failed = self.rustc("--sysroot=./absent-sdk", "--emit=llvm-ir", "-o", "bad-sdk.ll")
        self.assertEqual(failed["exit"], 1)
        self.assertFalse(runner.source_rejection(failed, self.cwd))

    def test_real_cargo_rejects_removed_target_then_restores_it(self):
        scripts = ROOT / "compiler/native/scripts"
        sys.path.insert(0, str(scripts))
        spec = importlib.util.spec_from_file_location("oracle_donor_scenarios", scripts / "test-daemon-scenarios.py")
        donor = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(donor)
        fixture = ROOT / "compiler/native/tests/daemon/cargo01_bin_discovery"
        for index in range(1, 8):
            donor.apply_overlay(str(fixture / f"v{index}"), str(self.cwd), 2_000_000_000 + index * 2)
        rule = json.loads((ROOT / "upstream/retained-scenario-results.json").read_text())["selection_rejections"]["cargo01_bin_discovery"]["v7"]
        self.assertFalse((self.cwd / rule["source"]).exists())
        for mode in ("build", "check"):
            checked = self.execute("cargo", mode, "-q", "--offline", "--bin", "e", "--message-format=json")
            self.assertTrue(runner.removed_target_rejection(checked, rule, cargo=True))
        donor.apply_overlay(str(fixture / "v8"), str(self.cwd), 2_000_000_016)
        checked = self.execute("cargo", "build", "-q", "--offline", "--bin", "e", "--message-format=json")
        self.assertEqual(checked["exit"], 0)
        program = subprocess.run([str(self.cwd / "target/debug/e")], cwd=self.cwd, capture_output=True, timeout=10)
        self.assertEqual((program.returncode, program.stdout, program.stderr), (0, b"e2\n", b""))

    def test_missing_and_invalid_dependency_artifacts_are_not_source_errors(self):
        (self.cwd / "main.rs").write_text("extern crate dep; fn main() {}\n")
        (self.cwd / "libdep.rlib").write_bytes(b"not a Rust library")
        for filename in ("missing.rlib", "libdep.rlib"):
            with self.subTest(filename=filename):
                failed = self.rustc("--extern", "dep=" + filename, "--emit=llvm-ir", "-o", "missing-dep.ll")
                self.assertEqual(failed["exit"], 1)
                self.assertFalse(runner.source_rejection(failed, self.cwd))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--oracle-rustc", type=Path)
    parser.add_argument("--oracle-cargo", type=Path)
    parser.add_argument("--oracle-output", type=Path)
    args, remaining = parser.parse_known_args()
    if any((args.oracle_rustc, args.oracle_cargo, args.oracle_output)):
        runner.require(all((args.oracle_rustc, args.oracle_cargo, args.oracle_output)), "all three pinned oracle control arguments are required")
        ORACLE = {"rustc": args.oracle_rustc.absolute(), "cargo": args.oracle_cargo.absolute(), "output": args.oracle_output.absolute()}
        ORACLE["output"].mkdir(parents=True, exist_ok=False)
        version = subprocess.run([str(ORACLE["rustc"]), "-vV"], capture_output=True, timeout=30, check=True)
        pin = next(line.split('"')[1] for line in (ROOT / "upstream/bluefe-core.pin").read_text().splitlines() if line.startswith("source = "))
        runner.require(f"commit-hash: {pin}\n".encode() in version.stdout, "process controls require the pinned Rust oracle")
        (ORACLE["output"] / "rustc-version.stdout").write_bytes(version.stdout)
        (ORACLE["output"] / "identities.json").write_text(json.dumps({"rust_source": pin, "rustc_sha256": runner.digest(ORACLE["rustc"]), "cargo_sha256": runner.digest(ORACLE["cargo"]), "runner_sha256": runner.digest(ROOT / "tools/qualify-retained-results.py"), "test_sha256": runner.digest(Path(__file__)), "inventory_sha256": runner.digest(ROOT / "upstream/retained-scenario-results.json")}, indent=2) + "\n")
    unittest.main(argv=[sys.argv[0], *remaining])
