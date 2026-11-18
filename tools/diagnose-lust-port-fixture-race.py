#!/usr/bin/env python3
"""Force one reachable fixture collision and one corrected interleaving.

The scanner pause and same-process test driver exist only in exported copies.
This is a correctness diagnostic, not a claim about historical scheduling.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tarfile
import time

BASE = "8191c173d1f5036c1cbaf84a50cb2dddbc88076a"
PREFIX = "compiler/native/tools/lust-port/"
TEST = "tests/scan_fixture.rs"
BEFORE = 'fn unknown_package_is_an_error() {\n    let root = fixture("human");'
AFTER = 'fn unknown_package_is_an_error() {\n    let root = fixture("unknown-package");'
PROBE = "LUST_PORT_FIXTURE_RACE_PROBE"

SCANNER_PAUSE = r'''
    // Exported diagnostic copy only: pause after real Cargo root discovery.
    if let Some(dir) = std::env::var_os("LUST_PORT_FIXTURE_RACE_PROBE") {
        let dir = PathBuf::from(dir);
        let ready = serde_json::json!({
            "pid": std::process::id(), "manifest": manifest,
            "roots": roots.iter().map(|r| &r.path).collect::<Vec<_>>(),
            "all_roots_exist": roots.iter().all(|r| r.path.is_file()),
            "argv": std::env::args().collect::<Vec<_>>(),
        });
        std::fs::write(dir.join("roots-ready.tmp"), serde_json::to_vec_pretty(&ready).unwrap()).unwrap();
        std::fs::rename(dir.join("roots-ready.tmp"), dir.join("roots-ready.json")).unwrap();
        let deadline = Instant::now() + std::time::Duration::from_secs(20);
        while !dir.join("sibling-removed.json").is_file() {
            assert!(Instant::now() < deadline, "diagnostic sibling did not delete its fixture");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
'''

FIXTURE_BEFORE = r'''
    let probe_sibling = std::thread::current().name() == Some("fixture-race-sibling");
    if probe_sibling { race_wait("roots-ready.json"); }
'''
FIXTURE_AFTER = r'''
    if probe_sibling {
        race_event("sibling-removed", serde_json::json!({
            "pid": std::process::id(), "name": name, "path": root,
            "exists_after_remove": root.exists(),
        }));
        race_wait("human-returned.json");
    }
'''
SCAN_RECORD = r'''
    if std::env::var_os("LUST_PORT_FIXTURE_RACE_PROBE").is_some() {
        race_event("SCAN_ROLE", serde_json::json!({
            "pid": std::process::id(), "fixture": root,
            "status": out.status.code(), "success": out.status.success(),
            "stdout": String::from_utf8_lossy(&out.stdout),
            "stderr": String::from_utf8_lossy(&out.stderr),
        }));
    }
'''
TEST_DRIVER = r'''

// Exported diagnostic copy only. Both original tests run with the same PID.
fn race_event(name: &str, value: serde_json::Value) {
    let dir = PathBuf::from(std::env::var_os("LUST_PORT_FIXTURE_RACE_PROBE").unwrap());
    let tmp = dir.join(format!("{name}.tmp"));
    std::fs::write(&tmp, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    std::fs::rename(tmp, dir.join(format!("{name}.json"))).unwrap();
}

fn race_wait(name: &str) {
    let dir = PathBuf::from(std::env::var_os("LUST_PORT_FIXTURE_RACE_PROBE").unwrap());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !dir.join(name).is_file() {
        assert!(std::time::Instant::now() < deadline, "diagnostic barrier timed out: {name}");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

#[test]
fn diagnostic_forced_fixture_interleaving() {
    let human = std::thread::Builder::new().name("fixture-race-human".into()).spawn(|| {
        let result = std::panic::catch_unwind(human_output_groups_findings);
        race_event("human-returned", serde_json::json!({
            "pid": std::process::id(), "passed": result.is_ok(),
        }));
        result.is_ok()
    }).unwrap();
    let sibling = std::thread::Builder::new().name("fixture-race-sibling".into()).spawn(|| {
        let result = std::panic::catch_unwind(unknown_package_is_an_error);
        race_event("sibling-returned", serde_json::json!({
            "pid": std::process::id(), "passed": result.is_ok(),
        }));
        result.is_ok()
    }).unwrap();
    let human_passed = human.join().unwrap();
    let sibling_passed = sibling.join().unwrap();
    race_event("outcome", serde_json::json!({
        "pid": std::process::id(), "human_passed": human_passed,
        "sibling_passed": sibling_passed,
    }));
    assert!(sibling_passed, "original unknown-package assertion failed");
    assert!(human_passed, "original human-report assertion failed during forced interleaving");
}
'''


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def overlay(files):
    result = dict(files)
    main = files["src/main.rs"].decode()
    needle = "    let base = meta.workspace_root.clone();\n    let mut files = walk::load(roots, &base, &target);"
    assert main.count(needle) == 1
    result["src/main.rs"] = main.replace(needle, SCANNER_PAUSE + "\n" + needle).encode()
    test = files[TEST].decode()
    needle = "    let _ = std::fs::remove_dir_all(&root);"
    # Only the first occurrence is the shared fixture helper.
    assert test.index(needle) < test.index("fn scan_json")
    test = test.replace(needle, FIXTURE_BEFORE + needle + FIXTURE_AFTER, 1)
    for function, role in [("human_output_groups_findings", "human-scan"), ("unknown_package_is_an_error", "sibling-scan")]:
        start = test.index("fn " + function + "() {")
        end = test.index("        .unwrap();", start) + len("        .unwrap();")
        test = test[:end] + SCAN_RECORD.replace("SCAN_ROLE", role) + test[end:]
    result[TEST] = (test + TEST_DRIVER).encode()
    return result


class Recorder:
    def __init__(self, root, env):
        self.root, self.env = root, env

    def run(self, name, argv, cwd, timeout=120, extra=None):
        dest = self.root / "commands" / name
        dest.mkdir(parents=True, exist_ok=False)
        env = self.env | (extra or {})
        write_json(dest / "command.json", {"argv": [str(v) for v in argv], "cwd": str(cwd),
            "environment": env, "timeout_seconds": timeout, "started_ns": time.time_ns()})
        start = time.monotonic_ns()
        status = {"returncode": None, "timed_out": False, "start_error": None}
        try:
            p = subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            try:
                out, err = p.communicate(timeout=timeout)
            except subprocess.TimeoutExpired:
                os.killpg(p.pid, signal.SIGKILL)
                out, err = p.communicate()
                status["timed_out"] = True
            status["returncode"] = p.returncode
        except OSError as exc:
            out, err = b"", str(exc).encode()
            status["start_error"] = str(exc)
        (dest / "stdout").write_bytes(out)
        (dest / "stderr").write_bytes(err)
        status.update(elapsed_ns=time.monotonic_ns()-start, stdout_sha256=sha(out), stderr_sha256=sha(err))
        write_json(dest / "status.json", status)
        return status, out


def must_succeed(record):
    status, data = record
    assert status["returncode"] == 0 and not status["timed_out"] and status["start_error"] is None, status
    return data


def snapshot(root, files):
    for name, data in files.items():
        p = root / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(data)


def archive_files(data, revision):
    files = {}
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        assert archive.pax_headers.get("comment") == revision
        for entry in archive:
            if entry.isdir():
                continue
            assert entry.isfile() and entry.name.startswith(PREFIX) and ".." not in Path(entry.name).parts
            files[entry.name[len(PREFIX):]] = archive.extractfile(entry).read()
    return files


def exported(rec, repo, label, revision):
    data = must_succeed(rec.run(label + "-export", ["git", "archive", "--format=tar", revision, PREFIX], repo))
    return archive_files(data, revision)


def execute(args):
    root, repo, sdk = args.output.resolve(), args.repository.resolve(), args.sdk.resolve()
    root.mkdir(parents=True, exist_ok=False)
    for d in ["home", "tmp", "execution-source"]:
        (root / d).mkdir()
    env = {"PATH": str(sdk / "bin") + ":/usr/bin:/bin", "HOME": str(root / "home"),
        "CARGO_HOME": str(args.cargo_home.resolve()), "RUSTC": str(sdk / "bin/rustc"),
        "CARGO_NET_OFFLINE": "true", "TMPDIR": str(root / "tmp"), "LC_ALL": "C", "TZ": "UTC"}
    rec = Recorder(root, env)
    report = {"schema": "lust-port-fixture-race-v1", "baseline": BASE, "candidate": args.candidate,
              "status": "incomplete", "historical_scheduling_proven": False}
    write_json(root / "report.json", report)
    try:
        source_ids = {}
        for label, rev in [("baseline", BASE), ("candidate", args.candidate)]:
            source_ids[label] = must_succeed(rec.run(label + "-identity",
                ["git", "rev-parse", rev, rev + "^{tree}", rev + ":compiler/native"], repo)).decode().splitlines()
        baseline = exported(rec, repo, "baseline", BASE)
        candidate = exported(rec, repo, "candidate", args.candidate)
        assert baseline[TEST].decode().count(BEFORE) == 1
        expected = dict(baseline)
        expected[TEST] = baseline[TEST].decode().replace(BEFORE, AFTER).encode()
        assert candidate == expected, "candidate tool delta must be only the distinct fixture label"
        for label, files in [("baseline", baseline), ("candidate", candidate)]:
            snapshot(root / "source" / label, files)
            snapshot(root / "source" / (label + "-diagnostic"), overlay(files))
        rustc = must_succeed(rec.run("rustc-version", [sdk / "bin/rustc", "-vV"], repo)).decode()
        cargo = must_succeed(rec.run("cargo-version", [sdk / "bin/cargo", "-vV"], repo)).decode()
        assert "b940084d7eb6a299eb4bfeb8e34901bc051e7ac4" in rustc and "cargo 1.99.0 (5f94df478" in cargo
        write_json(root / "identities.json", {"source": source_ids, "sdk": {
            "rustc_sha256": sha((sdk / "bin/rustc").read_bytes()), "cargo_sha256": sha((sdk / "bin/cargo").read_bytes()),
            "rustc_verbose": rustc, "cargo_verbose": cargo}, "runner_sha256": sha(Path(__file__).read_bytes())})
        (root / "runner.py").write_bytes(Path(__file__).read_bytes())
        src = root / "execution-source"
        binaries = {}
        for label, files, wanted in [("baseline", baseline, 101), ("candidate", candidate, 0)]:
            snapshot(src, overlay(files))
            build = must_succeed(rec.run(label + "-build", [sdk / "bin/cargo", "test", "--manifest-path", src / "Cargo.toml",
                "--locked", "--offline", "--jobs", "1", "--target-dir", root / "target", "--test", "scan_fixture",
                "--no-run", "--message-format=json"], src, timeout=180))
            artifacts = [json.loads(line) for line in build.splitlines() if line.startswith(b"{")]
            tests = [r["executable"] for r in artifacts if r.get("reason") == "compiler-artifact"
                     and r.get("target", {}).get("name") == "scan_fixture" and r.get("executable")]
            scanners = [r["executable"] for r in artifacts if r.get("reason") == "compiler-artifact"
                        and r.get("target", {}).get("name") == "lust-port" and r.get("executable")]
            assert len(tests) == len(scanners) == 1
            binaries[label] = {"test": tests[0], "test_sha256": sha(Path(tests[0]).read_bytes()),
                "scanner": scanners[0], "scanner_sha256": sha(Path(scanners[0]).read_bytes())}
            # Preserve both actual executables before the next incremental build.
            dest = root / "binaries" / label
            dest.mkdir(parents=True)
            shutil.copy2(tests[0], dest / "scan_fixture")
            shutil.copy2(scanners[0], dest / "lust-port")
            events = root / "events" / label
            events.mkdir(parents=True)
            status, _ = rec.run(label + "-interleaving", [tests[0], "diagnostic_forced_fixture_interleaving",
                "--exact", "--nocapture", "--test-threads=1"], src, timeout=60, extra={PROBE: str(events)})
            assert status["returncode"] == wanted and not status["timed_out"] and status["start_error"] is None, status
            assert sha(Path(tests[0]).read_bytes()) == binaries[label]["test_sha256"]
            assert sha(Path(scanners[0]).read_bytes()) == binaries[label]["scanner_sha256"]
        assert binaries["baseline"]["scanner_sha256"] == binaries["candidate"]["scanner_sha256"]
        write_json(root / "binaries.json", binaries)
        # One full ordinary candidate test run, with all diagnostic overlays removed.
        snapshot(src, candidate)
        ordinary = must_succeed(rec.run("candidate-ordinary-tests", [sdk / "bin/cargo", "test", "--manifest-path", src / "Cargo.toml",
            "--locked", "--offline", "--jobs", "1", "--target-dir", root / "target", "--message-format=json", "--", "--nocapture"], src, timeout=180))
        artifacts = [json.loads(line) for line in ordinary.splitlines() if line.startswith(b"{")]
        ordinary_bins = {}
        for row in artifacts:
            if row.get("reason") != "compiler-artifact" or not row.get("executable"):
                continue
            name = row["target"]["name"] + ("-unit" if row["profile"]["test"] and row["target"]["name"] == "lust-port" else "")
            assert name not in ordinary_bins
            executable = Path(row["executable"])
            ordinary_bins[name] = {"path": str(executable), "sha256": sha(executable.read_bytes())}
            dest = root / "binaries" / "ordinary" / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(executable, dest)
        assert set(ordinary_bins) == {"lust-port", "lust-port-unit", "scan_fixture"}
        after = {str(p.relative_to(src)): p.read_bytes() for p in src.rglob("*") if p.is_file()}
        assert after == candidate, "ordinary execution source differs from the uninstrumented candidate"
        write_json(root / "ordinary-execution.json", {"binaries": ordinary_bins,
            "source_after_sha256": {name: sha(data) for name, data in after.items()}})
        report["status"] = "completed"
        write_json(root / "report.json", report)
        verify(root)
    except BaseException as exc:
        report.update(status="incomplete", error=f"{type(exc).__name__}: {exc}")
        write_json(root / "report.json", report)
        raise


def verify(root):
    load = lambda name: json.loads((root / name).read_text())
    report = load("report.json")
    assert report["schema"] == "lust-port-fixture-race-v1" and report["status"] == "completed"
    assert report["baseline"] == BASE and not report["historical_scheduling_proven"]
    original, changed = {}, {}
    for label, store in [("baseline", original), ("candidate", changed)]:
        for p in (root / "source" / label).rglob("*"):
            if p.is_file(): store[str(p.relative_to(root / "source" / label))] = p.read_bytes()
        diagnostic = {str(p.relative_to(root / "source" / (label + "-diagnostic"))): p.read_bytes()
                      for p in (root / "source" / (label + "-diagnostic")).rglob("*") if p.is_file()}
        assert diagnostic == overlay(store)
    expected = dict(original)
    assert original[TEST].decode().count(BEFORE) == 1
    expected[TEST] = original[TEST].decode().replace(BEFORE, AFTER).encode()
    assert expected == changed
    identities, bins = load("identities.json"), load("binaries.json")
    assert identities["source"]["baseline"][0] == BASE
    assert identities["source"]["candidate"][0] == report["candidate"]
    assert sha((root / "runner.py").read_bytes()) == identities["runner_sha256"]
    assert "b940084d7eb6a299eb4bfeb8e34901bc051e7ac4" in identities["sdk"]["rustc_verbose"]
    assert "cargo 1.99.0 (5f94df478" in identities["sdk"]["cargo_verbose"]
    expected_commands = {"baseline-identity", "candidate-identity", "baseline-export", "candidate-export",
        "rustc-version", "cargo-version", "baseline-build", "candidate-build", "baseline-interleaving",
        "candidate-interleaving", "candidate-ordinary-tests"}
    assert {p.name for p in (root / "commands").iterdir()} == expected_commands
    for label, revision, files in [("baseline", BASE, original), ("candidate", report["candidate"], changed)]:
        assert archive_files((root / "commands" / (label + "-export") / "stdout").read_bytes(), revision) == files
        command = load("commands/" + label + "-export/command.json")
        assert command["argv"] == ["git", "archive", "--format=tar", revision, PREFIX]
        assert (root / "commands" / (label + "-identity") / "stdout").read_text().splitlines() == identities["source"][label]
        build = (root / "commands" / (label + "-build") / "stdout").read_bytes()
        artifacts = [json.loads(line) for line in build.splitlines() if line.startswith(b"{")]
        for key, name in [("test", "scan_fixture"), ("scanner", "lust-port")]:
            executables = [r["executable"] for r in artifacts if r.get("reason") == "compiler-artifact"
                           and r.get("target", {}).get("name") == name and r.get("executable")]
            assert executables == [bins[label][key]]
    for p in (root / "commands").iterdir():
        command = json.loads((p / "command.json").read_text())
        s = json.loads((p / "status.json").read_text())
        assert not s["timed_out"] and s["start_error"] is None
        assert s["returncode"] == (101 if p.name == "baseline-interleaving" else 0), p.name
        for stream in ["stdout", "stderr"]: assert sha((p / stream).read_bytes()) == s[stream + "_sha256"]
        if p.name.endswith("-interleaving"):
            label = p.name.split("-")[0]
            assert command["argv"] == [bins[label]["test"], "diagnostic_forced_fixture_interleaving",
                "--exact", "--nocapture", "--test-threads=1"]
            assert PROBE in command["environment"]
    assert bins["baseline"]["scanner_sha256"] == bins["candidate"]["scanner_sha256"]
    for label, fixed in [("baseline", False), ("candidate", True)]:
        for key, name in [("scanner", "lust-port"), ("test", "scan_fixture")]:
            assert sha((root / "binaries" / label / name).read_bytes()) == bins[label][key + "_sha256"]
        events = {p.stem: json.loads(p.read_text()) for p in (root / "events" / label).glob("*.json")}
        assert set(events) == {"roots-ready", "sibling-removed", "human-scan", "human-returned", "sibling-scan", "sibling-returned", "outcome"}
        ready, removed, human = (events[k] for k in ["roots-ready", "sibling-removed", "human-scan"])
        assert ready["all_roots_exist"] and len(ready["roots"]) == 1
        assert Path(ready["roots"][0]) == Path(human["fixture"]) / "src/lib.rs"
        assert Path(ready["manifest"]) == Path(human["fixture"]) / "Cargo.toml"
        assert ready["argv"] == [bins[label]["scanner"], "scan", "--manifest-path", ready["manifest"],
                                  "--target", "x86_64-unknown-linux-gnu"]
        assert not removed["exists_after_remove"]
        assert removed["pid"] == human["pid"] == events["outcome"]["pid"]
        assert (removed["path"] != human["fixture"]) == fixed
        assert removed["name"] == ("unknown-package" if fixed else "human")
        assert human["status"] == 0 and human["success"]
        assert "Gaps by root cause" in human["stdout"]
        assert human["stdout"].count("attrs.unsupported") == (2 if fixed else 0)
        assert ("files: 7 (18 lines, 6 functions), findings: 3 in 3 files, parse errors: 1" if fixed
                else "files: 0 (0 lines, 0 functions), findings: 0 in 0 files, parse errors: 0") in human["stdout"]
        assert events["human-returned"]["passed"] == fixed
        assert events["outcome"]["human_passed"] == fixed and events["outcome"]["sibling_passed"]
        assert events["sibling-returned"]["passed"]
        assert not events["sibling-scan"]["success"] and "not found" in events["sibling-scan"]["stderr"]
    raw = (root / "commands/baseline-interleaving/stderr").read_text()
    assert 'assertion failed: text.contains("attrs.unsupported")' in raw
    ordinary = (root / "commands/candidate-ordinary-tests/stdout").read_text()
    assert "test result: ok. 8 passed; 0 failed" in ordinary and "test result: ok. 5 passed; 0 failed" in ordinary
    executed = load("ordinary-execution.json")
    assert executed["source_after_sha256"] == {name: sha(data) for name, data in changed.items()}
    artifacts = [json.loads(line) for line in ordinary.splitlines() if line.startswith("{")]
    ordinary_bins = {}
    for row in artifacts:
        if row.get("reason") == "compiler-artifact" and row.get("executable"):
            name = row["target"]["name"] + ("-unit" if row["profile"]["test"] and row["target"]["name"] == "lust-port" else "")
            assert name not in ordinary_bins
            ordinary_bins[name] = {"path": row["executable"], "sha256": sha((root / "binaries/ordinary" / name).read_bytes())}
    assert set(ordinary_bins) == {"lust-port", "lust-port-unit", "scan_fixture"}
    assert ordinary_bins == executed["binaries"]
    print(json.dumps({"verified": True, "baseline_forced_human_failure": True,
        "candidate_forced_tests_passed": 2, "candidate_ordinary_tests_passed": 13,
        "diagnostic_scanner_identical": True, "historical_scheduling_proven": False}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    run = sub.add_parser("run")
    for flag in ["repository", "output", "sdk", "cargo-home"]:
        run.add_argument("--" + flag, type=Path, required=True)
    run.add_argument("--candidate", required=True)
    check = sub.add_parser("verify")
    check.add_argument("root", type=Path)
    args = parser.parse_args()
    execute(args) if args.mode == "run" else verify(args.root)
