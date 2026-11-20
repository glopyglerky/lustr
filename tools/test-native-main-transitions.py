#!/usr/bin/env python3
"""Strict retained-main return-type gate, independent of the legacy scenario gate.

Build every version successfully, then compare the actual warm and cold
executables with rustc's exact numeric exit status and stdout/stderr bytes.
Signals, timeouts, rejected compilation, and lost daemon reuse fail the gate.
Keep all command streams and the report in a new caller-selected directory.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tempfile
import time


ROOT = Path(__file__).resolve().parents[1]
SCENARIO = Path("compiler/native/tests/daemon/mainret04_typed_entry_transition")
EXPECTED_EXITS = {"v1.rs": 1, "v2.rs": 0, "v3.rs": 7, "v4.rs": 0}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stop_group(process):
    # Only this runner's start_new_session process group is addressed.
    # poll() reaps an exited leader; never signal that numeric group afterward.
    if process.poll() is not None:
        return
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT)
    parser.add_argument("--lust", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.source, args.output = args.source.resolve(), args.output.resolve()
    # Preserve argv[0] for rustup proxies and the requested client/core layout.
    # Reading the files for hashes still follows symlinks.
    args.lust, args.rustc = args.lust.absolute(), args.rustc.absolute()
    args.output.mkdir(parents=True, exist_ok=False)
    work = args.output / "work"
    work.mkdir()
    commands, results = [], []
    report = {
        "schema": "lustr-native-main-transitions-v1",
        "status": "preparing",
        "commands": commands,
        "results": results,
        "passed": False,
    }

    def save():
        (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    # An always-run CI gate must explain a missing compiler after an earlier
    # build failure. Record failure before reading optional source/binary files.
    save()
    try:
        report.update(
            source_head=subprocess.check_output(
                ["git", "-C", str(args.source), "rev-parse", "HEAD"],
                text=True, stderr=subprocess.PIPE,
            ).strip(),
            execution_workspace_cgen_sha256=digest(args.source / "compiler/native/src/cgen.rs"),
            lust_sha256=digest(args.lust),
            lust_core_sha256=digest(args.lust.with_name("lust-core")),
            rustc_sha256=digest(args.rustc),
            runner_sha256=digest(Path(__file__)),
        )
    except (OSError, subprocess.SubprocessError) as error:
        report.update(status="setup-failed", failure=type(error).__name__)
        (args.output / "setup.stderr").write_text(str(error) + "\n")
        save()
        return 2
    save()

    def run(label, argv, env, timeout=60):
        result = {
            "label": label, "argv": [str(item) for item in argv],
            "exit": None, "timed_out": False, "status": "starting",
        }
        commands.append(result)
        save()
        stdout_path = args.output / f"{label}.stdout"
        stderr_path = args.output / f"{label}.stderr"
        # Files retain partial streams even if a deadline or cleanup fails, and
        # avoid waiting on a pipe inherited by an exited command's descendant.
        with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
            process = subprocess.Popen(
                [str(item) for item in argv], cwd=work, env=env,
                stdout=out, stderr=err, start_new_session=True,
            )
            result["status"] = "running"
            save()
            try:
                process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                result.update(timed_out=True, status="timed-out",
                              stdout_sha256=digest(stdout_path), stderr_sha256=digest(stderr_path))
                save()
                stop_group(process)
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    result["cleanup_status"] = "failed"
                    save()
                    raise
        stdout, stderr = stdout_path.read_bytes(), stderr_path.read_bytes()
        result.update(exit=process.returncode, status="timed-out" if result["timed_out"] else "completed",
                      stdout_sha256=hashlib.sha256(stdout).hexdigest(),
                      stderr_sha256=hashlib.sha256(stderr).hexdigest())
        save()
        return result, stdout, stderr

    def accepted(result):
        return result[0]["exit"] == 0 and not result[0]["timed_out"]

    def same_runtime(actual, expected):
        return (
            not actual[0]["timed_out"] and actual[0]["exit"] >= 0
            and not expected[0]["timed_out"] and expected[0]["exit"] >= 0
            and actual[0]["exit"] == expected[0]["exit"]
            and actual[1:] == expected[1:]
        )

    env = dict(os.environ)
    for name in (
        "LUST_NO_DAEMON", "LUST_FRONT_STATE", "LUST_FRONT_STATE_VERBOSE",
        "LUST_STD_SRC", "LUST_LIBRARY", "LUST_RELEASE_C", "LUST_UNWIND_TABLES",
        "LUST_SOURCES_READY", "LUST_DAEMON_PREWARM", "LUST_DAEMON_LOCK_FD",
        "LUST_DAEMON_DIRECTORY_FD",
    ):
        env.pop(name, None)
    # The socket is deliberately short even when evidence lives in a long path.
    socket_dir = Path(tempfile.mkdtemp(prefix="lustr-main-", dir="/tmp"))
    env.update(LUST_DAEMON_SOCKET=str(socket_dir / "d.sock"), LUST_NO_CACHE="1",
               LUST_CACHE_DIR=str(work / "cache"), LUST_STATS="1")
    cold = dict(env, LUST_NO_DAEMON="1")
    daemon = None
    try:
        version = run("rustc-version", [args.rustc, "--version", "--verbose"], cold)
        if not accepted(version):
            report.update(status="setup-failed", failure="rustc-version-failed")
            save()
            return 2
        report["status"] = "running"
        save()
        with (args.output / "daemon.stdout").open("wb") as out, (args.output / "daemon.stderr").open("wb") as err:
            daemon = subprocess.Popen(
                [str(args.lust), "daemon"], cwd=work, env=env,
                stdout=out, stderr=err, start_new_session=True,
            )
            deadline = time.monotonic() + 10
            while not (socket_dir / "d.sock").exists():
                if daemon.poll() is not None or time.monotonic() >= deadline:
                    report.update(status="failed", failure="daemon-start-failed")
                    save()
                    return 1
                time.sleep(0.01)
            mtime = time.time()
            for version, expected_exit in EXPECTED_EXITS.items():
                fixture = args.source / SCENARIO / version
                target = work / "main.rs"
                shutil.copyfile(fixture, target)
                mtime += 2
                os.utime(target, (mtime, mtime))
                label = version.removesuffix(".rs")
                row = {"version": version, "fixture_sha256": digest(fixture),
                       "expected_exit": expected_exit, "passed": False}
                results.append(row)
                oracle_build = run(label + "-rustc-build", [args.rustc, "--edition=2024", "-C", "overflow-checks=on", "main.rs", "-o", label + "-oracle"], cold)
                check = run(label + "-warm-check", [args.lust, "check", "main.rs"], env)
                recheck = run(label + "-warm-recheck", [args.lust, "check", "main.rs"], env)
                # A repeated std-free check uses the retained body/chunk path;
                # a repeated std-backed check uses the reuse path. Both must
                # recheck zero bodies. A fresh fallback reports full analysis.
                row["retained_recheck"] = (
                    accepted(recheck) and daemon.poll() is None
                    and re.search(rb"^lust-stats: mode=(?:reuse|bodies) fns=\d+ rechecked=0(?: |$)", recheck[2], re.M) is not None
                )
                warm_build = run(label + "-warm-build", [args.lust, "build", "main.rs", "-o", label + "-warm"], env)
                # Check reuse alone cannot prove that the following build used
                # the daemon; the client can fall back to a fresh core process.
                row["retained_build"] = (
                    accepted(warm_build) and daemon.poll() is None
                    and re.search(rb"^lust-stats: mode=(?:reuse|bodies)(?: |$)", warm_build[2], re.M) is not None
                )
                cold_build = run(label + "-cold-build", [args.lust, "build", "main.rs", "-o", label + "-cold"], cold)
                row["compile_accepted"] = all(accepted(item) for item in (oracle_build, check, recheck, warm_build, cold_build))
                if row["compile_accepted"]:
                    oracle = run(label + "-rustc-run", [work / (label + "-oracle")], cold, timeout=10)
                    warm = run(label + "-warm-run", [work / (label + "-warm")], cold, timeout=10)
                    fresh = run(label + "-cold-run", [work / (label + "-cold")], cold, timeout=10)
                    row.update(oracle_exit=oracle[0]["exit"], warm_exit=warm[0]["exit"], cold_exit=fresh[0]["exit"],
                               warm_timed_out=warm[0]["timed_out"], cold_timed_out=fresh[0]["timed_out"],
                               warm_matches=same_runtime(warm, oracle), cold_matches=same_runtime(fresh, oracle))
                    row["passed"] = (
                        row["retained_recheck"] and row["retained_build"] and oracle[0]["exit"] == expected_exit
                        and row["warm_matches"] and row["cold_matches"]
                    )
                save()
                print(json.dumps(row), flush=True)
            report["passed"] = len(results) == len(EXPECTED_EXITS) and all(row["passed"] for row in results)
            report["status"] = "passed" if report["passed"] else "failed"
            save()
            return 0 if report["passed"] else 1
    finally:
        try:
            if daemon is not None:
                stop_group(daemon)
                daemon.wait(timeout=5)
            shutil.rmtree(socket_dir)
        except Exception as error:
            report.update(passed=False, pre_cleanup_status=report["status"],
                          status="cleanup-failed", cleanup_failure=type(error).__name__)
            save()
            raise


if __name__ == "__main__":
    raise SystemExit(main())
