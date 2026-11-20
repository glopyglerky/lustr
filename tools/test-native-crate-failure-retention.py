#!/usr/bin/env python3
"""Exercise artifact handling and the real crate runner's runtime comparisons.

Compiler stubs here induce known runner outcomes. They qualify evidence
retention and exit preservation only, not native compilation or daemon behavior.
"""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
RETAINER = ROOT / "tools/retain-native-crate-failure.py"


def executable(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    path.chmod(0o755)


class RetentionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="lustr-crate-retention-")
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.case = self.base / "case"
        self.case.mkdir()
        self.output = self.base / "evidence"

    def retain(self, output=None, extra=()):
        return subprocess.run(["python3", str(RETAINER), "--case", str(self.case),
                               "--output", str(output or self.output), "--name", "odd-path",
                               "--reason", "original failure status 3", *extra],
                              capture_output=True, text=True)

    def test_raw_log_and_nonfollowed_special_entries(self):
        log = self.case / "src/.lust-cache/daemon.log"
        log.parent.mkdir(parents=True)
        raw = b"panic:\x00\xff\noriginal daemon bytes\n"
        log.write_bytes(raw)
        os.mkfifo(self.case / "stream")
        secret = self.base / "outside"
        secret.write_bytes(b"must not be read")
        (self.case / "external-link").symlink_to(secret)
        result = self.retain()
        self.assertEqual(result.returncode, 0, result.stderr)
        capture, = self.output.iterdir()
        record = json.loads((capture / "failure.json").read_text())
        self.assertTrue(record["capture_complete"])
        self.assertEqual(record["failure"], "original failure status 3")
        self.assertEqual(record["skipped_special_files"], ["case/stream"])
        with tarfile.open(capture / "case.tar.gz") as archive:
            self.assertEqual(archive.extractfile("case/src/.lust-cache/daemon.log").read(), raw)
            self.assertTrue(archive.getmember("case/external-link").issym())
            self.assertNotIn("case/stream", archive.getnames())

    def test_repeat_capture_never_overwrites(self):
        (self.case / "state").write_text("first")
        self.assertEqual(self.retain().returncode, 0)
        first, = self.output.iterdir()
        original = (first / "case.tar.gz").read_bytes()
        (self.case / "state").write_text("second")
        self.assertEqual(self.retain().returncode, 0)
        self.assertEqual(len(list(self.output.iterdir())), 2)
        self.assertEqual((first / "case.tar.gz").read_bytes(), original)

    def test_symlink_and_recursive_destinations_are_rejected(self):
        target = self.base / "target"
        target.mkdir()
        self.output.symlink_to(target, target_is_directory=True)
        self.assertNotEqual(self.retain().returncode, 0)
        self.assertEqual(list(target.iterdir()), [])
        self.assertNotEqual(self.retain(self.case / "nested").returncode, 0)
        self.assertFalse((self.case / "nested").exists())

    def test_explicit_external_daemon_log(self):
        log = self.base / "daemon.log"
        log.write_bytes(b"external daemon failure\n")
        self.assertEqual(self.retain(extra=("--daemon-log", str(log))).returncode, 0)
        capture, = self.output.iterdir()
        with tarfile.open(capture / "case.tar.gz") as archive:
            self.assertEqual(archive.extractfile("external-daemon.log").read(), log.read_bytes())

    def runner(self, mode="fail", retain=True, output=None, filter="odd-path",
               oracle_exit=0, daemon_exits=None):
        checkout = self.base / "checkout"
        native = checkout / "compiler/native"
        scripts = native / "scripts"
        scripts.mkdir(parents=True, exist_ok=True)
        for name in ["test-crates.sh", "timeout-lib.sh"]:
            shutil.copyfile(ROOT / "compiler/native/scripts" / name, scripts / name)
        (checkout / "tools").mkdir(exist_ok=True)
        shutil.copyfile(RETAINER, checkout / "tools" / RETAINER.name)
        fixture = native / "tests/crates/odd-path"
        fixture.mkdir(parents=True, exist_ok=True)
        (fixture / "Cargo.toml").write_text('[package]\nname = "odd-path"\n')
        (fixture / "fixture.txt").write_text("projdir=sp ace#é\ndaemon=1\n")
        (fixture / "src").mkdir(exist_ok=True)
        (fixture / "src/main.rs").write_text("fn main() {}\n")
        bin_dir = self.base / "bin"
        executable(bin_dir / "rustc", '#!/bin/sh\nprintf "host: x86_64-unknown-linux-gnu\\n"\n')
        executable(bin_dir / "cargo", '''#!/usr/bin/env python3
import os,pathlib,sys
out=pathlib.Path(sys.argv[sys.argv.index('--target-dir')+1])/'debug/odd-path'
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text('#!/bin/sh\\nprintf "fixture output\\n"\\nprintf "fixture stderr\\n" >&2\\nexit '+os.environ['STUB_ORACLE_EXIT']+'\\n');out.chmod(0o755)
''')
        executable(native / "target/release/lust", '''#!/usr/bin/env python3
import os,pathlib,sys
args=sys.argv[1:];out=pathlib.Path(args[args.index('-o')+1])
project=pathlib.Path(args[args.index('--manifest-path')+1]).parent
status=os.environ['STUB_ORACLE_EXIT']
if 'LUST_NO_DAEMON' not in os.environ:
 log=project/'src/.lust-cache/daemon.log';log.parent.mkdir(parents=True,exist_ok=True)
 marker=project/'daemon-called'
 phase='WARM' if marker.exists() else 'COLD'
 if marker.exists() and os.environ['STUB_MODE']=='fail':
  log.write_bytes(b'original daemon failure\\x00\\xff\\n')
  sys.stderr.write('error: daemon returned a truncated or oversized response header\\nraw detail\\n');sys.exit(3)
 marker.touch()
 status=os.environ['STUB_'+phase+'_EXIT']
out.write_text('#!/bin/sh\\nprintf "fixture output\\n"\\nprintf "fixture stderr\\n" >&2\\nexit '+status+'\\n');out.chmod(0o755)
''')
        daemon_exits = daemon_exits or {}
        env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ["PATH"],
                   STUB_MODE=mode, STUB_ORACLE_EXIT=str(oracle_exit),
                   STUB_COLD_EXIT=str(daemon_exits.get("cold", oracle_exit)),
                   STUB_WARM_EXIT=str(daemon_exits.get("warm", oracle_exit)))
        env.pop("LUST_CACHE_DIR", None)
        env.pop("LUST_CRATE_FAILURE_DIR", None)
        if retain:
            env["LUST_CRATE_FAILURE_DIR"] = str(output or self.output)
        return subprocess.run(["bash", str(scripts / "test-crates.sh"), filter], env=env,
                              capture_output=True, text=True, timeout=20)

    def test_runner_retains_failed_warm_case_and_original_result(self):
        run = self.runner()
        self.assertEqual(run.returncode, 1, run.stderr)
        self.assertIn("test-crates: 0 passed, 1 failed", run.stdout)
        capture, = self.output.iterdir()
        with tarfile.open(capture / "case.tar.gz") as archive:
            self.assertEqual(archive.extractfile("case/daemon.warm.status").read(), b"3\n")
            self.assertIn(b"raw detail", archive.extractfile("case/daemon.warm.err").read())
            self.assertEqual(archive.extractfile("case/sp ace#é/src/.lust-cache/daemon.log").read(),
                             b"original daemon failure\x00\xff\n")

    def test_runner_capture_error_does_not_replace_fixture_failure(self):
        self.output.write_text("keep this existing file")
        run = self.runner()
        self.assertEqual(run.returncode, 1)
        self.assertIn("original failure is unchanged", run.stderr)
        self.assertEqual(self.output.read_text(), "keep this existing file")

    def test_default_cleanup_and_success_do_not_create_artifacts(self):
        self.assertEqual(self.runner(retain=False).returncode, 1)
        self.assertFalse(self.output.exists())
        self.assertEqual(self.runner(mode="pass").returncode, 0)
        self.assertFalse(self.output.exists())

    def test_empty_filter_match_still_fails_setup(self):
        run = self.runner(filter="not-a-fixture")
        self.assertEqual(run.returncode, 2)
        self.assertIn("no fixtures matched", run.stdout)
        self.assertFalse(self.output.exists())

    def test_runner_rejects_daemon_exit_mismatch_with_matching_stdout(self):
        for phase, oracle_status, daemon_status in (("cold", 0, 7), ("warm", 0, 7),
                                                   ("cold", 101, 0), ("warm", 101, 0)):
            with self.subTest(phase=phase, oracle=oracle_status, daemon=daemon_status):
                output = self.base / f"mismatch-{phase}-{oracle_status}-{daemon_status}"
                run = self.runner(mode="pass", output=output, oracle_exit=oracle_status,
                                  daemon_exits={phase: daemon_status})
                self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
                self.assertIn(f"daemon {phase} build: exit status cargo={oracle_status} lust={daemon_status}",
                              run.stdout)
                self.assertIn("test-crates: 0 passed, 1 failed", run.stdout)
                capture, = output.iterdir()
                with tarfile.open(capture / "case.tar.gz") as archive:
                    self.assertEqual(archive.extractfile("case/cargo.run.status").read(),
                                     f"{oracle_status}\n".encode())
                    self.assertEqual(archive.extractfile("case/lust.run.status").read(),
                                     f"{oracle_status}\n".encode())
                    self.assertEqual(archive.extractfile(f"case/daemon.{phase}.run.status").read(),
                                     f"{daemon_status}\n".encode())
                    self.assertEqual(archive.extractfile("case/cargo.run.out").read(),
                                     archive.extractfile(f"case/daemon.{phase}.run.out").read())
                    self.assertEqual(archive.extractfile(f"case/daemon.{phase}.run.err").read(),
                                     b"fixture stderr\n")
                    if phase == "cold":
                        self.assertNotIn("case/daemon.warm.status", archive.getnames())

    def test_runner_accepts_matching_nonzero_exits(self):
        for status in (7, 101, 134):
            with self.subTest(status=status):
                run = self.runner(mode="pass", oracle_exit=status)
                self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
                self.assertIn("test-crates: 1 passed, 0 failed", run.stdout)
                self.assertFalse(self.output.exists())

    def test_runner_does_not_normalize_timeout_or_signal_status_codes(self):
        # These processes return the raw codes, without inducing a real timeout
        # or signal. The runner must compare them as captured, not coerce them.
        for status in (124, 137, 143):
            with self.subTest(status=status):
                output = self.base / f"raw-status-{status}"
                run = self.runner(mode="pass", output=output, daemon_exits={"warm": status})
                self.assertEqual(run.returncode, 1, run.stdout + run.stderr)
                self.assertIn(f"exit status cargo=0 lust={status}", run.stdout)
                capture, = output.iterdir()
                with tarfile.open(capture / "case.tar.gz") as archive:
                    self.assertEqual(archive.extractfile("case/daemon.warm.run.status").read(),
                                     f"{status}\n".encode())


if __name__ == "__main__":
    unittest.main()
