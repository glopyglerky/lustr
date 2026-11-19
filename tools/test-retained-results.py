#!/usr/bin/env python3
"""Failure controls for the independent retained-history qualifier.

Synthetic verdicts exercise admission, not compiler correctness. Process tests
exercise the observer's actual public-command, artifact and single-run boundary.
"""
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


class ProcessControls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)
        self.entry = self.directory / "lustr"
        self.counter = self.directory / "executions"
        program = "#!" + sys.executable + "\nfrom pathlib import Path\np=Path(" + repr(str(self.counter)) + ")\np.open('a').write('x')\nprint('right')\n"
        self.entry.write_text("#!" + sys.executable + "\nimport json,os,sys\nfrom pathlib import Path\nargs=sys.argv[1:]\ncode=int(os.environ.get('CONTROL_COMPILE_EXIT','0'))\nif code==0 and args[0]=='build' and not os.environ.get('CONTROL_OMIT_ARTIFACT'):\n p=Path(args[args.index('-o')+1]);p.write_text(" + repr(program) + ");p.chmod(0o755)\nPath(os.environ['LUSTR_TRACE_PATH']).write_text(json.dumps({'engine':'lust' if code==0 else 'lust-failed','entry':'native-library' if os.environ.get('LUST_NO_DAEMON') else 'daemon'})+'\\n')\nif code: print('error[L0101]: control reject',file=sys.stderr)\nsys.exit(code)\n")
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


if __name__ == "__main__":
    unittest.main()
