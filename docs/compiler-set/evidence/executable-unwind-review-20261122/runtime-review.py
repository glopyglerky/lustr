#!/usr/bin/env python3
"""Independent, bounded runtime probes for the 930b384a Cargo unwind increment."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import tempfile
import time
import traceback

PROFILE = '''struct Guard(&'static str);
impl Drop for Guard { fn drop(&mut self) { println!("drop:{}:{}", self.0, std::thread::panicking()); } }
fn hit() { let _guard = Guard("guard"); panic!("profile panic"); }
fn main() {
    std::panic::set_hook(Box::new(|_| println!("hook")));
    let r = std::panic::catch_unwind(hit);
    println!("caught:{}:{}", r.is_err(), std::thread::panicking());
}
'''
EDIT = PROFILE.replace('Guard("guard")', 'Guard("edited")')
INVALID = EDIT.replace('let _guard =', 'let _guard: bool =')
FIXTURES = [
    ("resume_payload", '''struct Payload;
impl Drop for Payload { fn drop(&mut self) { println!("payload"); } }
struct Guard;
impl Drop for Guard { fn drop(&mut self) { println!("guard:{}", std::thread::panicking()); } }
fn main() {
    std::panic::set_hook(Box::new(|_| println!("hook")));
    let outer = std::panic::catch_unwind(|| {
        let _guard = Guard;
        let inner = std::panic::catch_unwind(|| std::panic::panic_any(Payload));
        assert!(!std::thread::panicking());
        println!("resuming");
        std::panic::resume_unwind(inner.unwrap_err());
    });
    let payload = outer.unwrap_err();
    assert!(payload.downcast_ref::<Payload>().is_some());
    println!("caught:{}", std::thread::panicking());
    drop(payload);
}
''', 0, "hook\nresuming\nguard:true\ncaught:false\npayload\n", ""),
    ("catch_in_drop", '''struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        println!("before:{}", std::thread::panicking());
        let inner = std::panic::catch_unwind(|| panic!("inner"));
        println!("inner:{} after:{}", inner.is_err(), std::thread::panicking());
    }
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let outer = std::panic::catch_unwind(|| { let _guard = Guard; panic!("outer"); });
    println!("after:{}:{}", outer.is_err(), std::thread::panicking());
}
''', 0, "before:true\ninner:true after:true\nafter:true:false\n", ""),
    ("partial_init", '''struct D(u64);
impl Drop for D { fn drop(&mut self) { println!("drop:{}", self.0); } }
struct S { a: D, b: D, c: D }
fn fail() -> D { panic!("partial init"); }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(|| {
        let _outer = D(0);
        let _s = S { a: D(1), b: D(2), c: fail() };
    });
    println!("caught:{}", r.is_err());
}
''', 0, "drop:2\ndrop:1\ndrop:0\ncaught:true\n", ""),
    ("partial_move", '''struct D(u64);
impl Drop for D { fn drop(&mut self) { println!("drop:{}", self.0); } }
struct S { a: D, b: D, c: D }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(|| {
        let _outer = D(0);
        let s = S { a: D(1), b: D(2), c: D(3) };
        let _moved = s.b;
        panic!("partial move");
    });
    println!("caught:{}", r.is_err());
}
''', 0, "drop:2\ndrop:1\ndrop:3\ndrop:0\ncaught:true\n", ""),
    ("double_panic", '''struct Bomb;
impl Drop for Bomb { fn drop(&mut self) { println!("drop"); panic!("drop panic"); } }
fn main() {
    std::panic::set_hook(Box::new(|_| println!("hook")));
    let _ = std::panic::catch_unwind(|| { let _bomb = Bomb; panic!("outer"); });
    println!("unexpected recovery");
}
''', -signal.SIGABRT, "hook\ndrop\nhook\nhook\n", "thread caused non-unwinding panic. aborting.\n"),
    ("thread_isolation", '''use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
struct Guard(Arc<AtomicUsize>);
impl Drop for Guard { fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); } }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let drops = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for id in 0..4 {
        let d = drops.clone();
        handles.push(std::thread::spawn(move || {
            let mut count = 0;
            for _ in 0..10 {
                let d = d.clone();
                let result = std::panic::catch_unwind(move || { let _g = Guard(d); panic!("caught"); });
                assert!(!std::thread::panicking());
                if result.is_err() { count += 1; }
            }
            if id == 3 { panic!("uncaught thread"); }
            count
        }));
    }
    let mut caught = 0;
    let mut died = 0;
    for h in handles { match h.join() { Ok(n) => caught += n, Err(_) => died += 1 } }
    println!("caught:{} died:{} drops:{} main:{}", caught, died, drops.load(Ordering::SeqCst), std::thread::panicking());
}
''', 0, "caught:30 died:1 drops:40 main:false\n", ""),
]

def sha(path):
    with Path(path).open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()

def main():
    p = argparse.ArgumentParser()
    for flag in ["package", "sdk", "source", "output", "strace"]:
        p.add_argument("--" + flag, required=True, type=Path)
    args = p.parse_args()
    package, sdk, source = (v.resolve(strict=True) for v in [args.package, args.sdk, args.source])
    root = args.output.absolute()
    root.mkdir(parents=True, exist_ok=False)
    for d in ["commands", "projects", "tmp", "cargo-home", "native-cache", "observations"]:
        (root / d).mkdir()
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    helper = source / "tools/test-rustc-profile-defaults.py"
    spec = importlib.util.spec_from_file_location("profile_reader", helper)
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    report = {"schema": "lustr-unwind-increment-semantic-probes-v1", "passed": False,
              "scope": "actual Cargo binary builds, retained panic profile history, independent cleanup cases; no speed claim",
              "commands": [], "cases": [], "errors": [], "unexpected_candidate_executions": [],
              "inputs": [{"path": str(x), "sha256": sha(x)} for x in
                         [Path(__file__), helper, package / "bin/rustc", package / "bin/lust",
                          package / "bin/lust-core", package / "bin/cargo", sdk / "bin/rustc", sdk / "bin/cargo"]]}
    def save():
        (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    trace = lambda dest, argv: [str(args.strace), "-f", "-qq", "-s", "1048576", "-e", "trace=execve,execveat", "-o", str(dest), *map(str, argv)]
    sockdir = tempfile.TemporaryDirectory(prefix="lustr-930-")
    sock = Path(sockdir.name) / "owner.sock"
    env = {"PATH": str(package / "bin") + ":/usr/bin:/bin", "LANG": "C", "LC_ALL": "C",
           "CARGO_HOME": str(root / "cargo-home"), "CARGO_NET_OFFLINE": "true", "CARGO_BUILD_JOBS": "1",
           "CARGO_TERM_COLOR": "never", "RUSTC": str(package / "bin/rustc"), "RUST_BACKTRACE": "0",
           "RUST_MIN_STACK": "268435456", "TMPDIR": str(root / "tmp"), "LUST_CACHE_DIR": str(root / "native-cache"),
           "LUST_DAEMON_SOCKET": str(sock), "LUST_DAEMON_IDLE": "1800", "LUST_THREADS": "2",
           "LUSTR_LIBRARY_REPORT_DIR": str(root / "observations")}
    if "HOME" in os.environ:
        env["HOME"] = os.environ["HOME"]
    reference = {k: v for k, v in env.items() if not k.startswith(("LUST_", "LUSTR_"))}
    reference.update(RUSTC=str(sdk / "bin/rustc"), LD_LIBRARY_PATH=str(sdk / "lib"), PATH=str(sdk / "bin") + ":/usr/bin:/bin")
    allowed = {(package / p).resolve() for p in ["bin/rustc", "bin/lust", "bin/lust-core", "bin/cargo", "lib/lust/tcc/bin/tcc"]}
    def inspect(events):
        for event in events:
            if event["result"] == "0" and Path(event["path"]).resolve() not in allowed:
                report["unexpected_candidate_executions"].append(event)
    def run(label, argv, cwd, environment, traced=False, timeout=90):
        d = root / "commands" / label
        d.mkdir()
        row = {"label": label, "argv": list(map(str, argv)), "cwd": str(cwd)}
        report["commands"].append(row)
        save()
        process = subprocess.Popen(trace(d / "exec.trace", argv) if traced else list(map(str, argv)),
                                   cwd=cwd, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        try:
            out, err = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            out, err = process.communicate()
            row["timed_out"] = True
        row["status"] = process.returncode
        for name, data in [("stdout", out), ("stderr", err)]:
            (d / name).write_bytes(data)
            row[name] = {"path": str((d / name).relative_to(root)), "sha256": sha(d / name), "bytes": len(data)}
        if traced:
            row["executions"] = reader.exec_events((d / "exec.trace").read_bytes())
            inspect(row["executions"])
        save()
        return row, out, err
    owner = None
    handles = [(root / "daemon.stdout").open("wb"), (root / "daemon.stderr").open("wb")]
    daemon = subprocess.Popen(trace(root / "daemon.trace", [package / "bin/lust-core", "daemon"]), cwd=root,
                              env=env, stdout=handles[0], stderr=handles[1], start_new_session=True)
    try:
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline and daemon.poll() is None:
            children = Path(f"/proc/{daemon.pid}/task/{daemon.pid}/children").read_text().split()
            if children and sock.exists():
                assert len(children) == 1
                owner = int(children[0])
                assert Path(f"/proc/{owner}/exe").resolve() == (package / "bin/lust-core").resolve()
                break
            time.sleep(0.02)
        assert owner is not None, "retained owner did not start"
        report["owner_pid"] = owner
        history = [("profile_cold", PROFILE, None, 0, "hook\ndrop:guard:true\ncaught:true:false\n", ""),
                   ("profile_noop", PROFILE, None, 0, "hook\ndrop:guard:true\ncaught:true:false\n", ""),
                   ("profile_abort", PROFILE, "abort", -signal.SIGABRT, "hook\n", ""),
                   ("profile_abort_noop", PROFILE, "abort", -signal.SIGABRT, "hook\n", ""),
                   ("profile_restore", PROFILE, None, 0, "hook\ndrop:guard:true\ncaught:true:false\n", ""),
                   ("profile_edit", EDIT, None, 0, "hook\ndrop:edited:true\ncaught:true:false\n", ""),
                   ("profile_invalid", INVALID, None, None, "", ""),
                   ("profile_repair", EDIT, None, 0, "hook\ndrop:edited:true\ncaught:true:false\n", "")]
        plans = [(name, text, panic, status, out, err, "profile") for name, text, panic, status, out, err in history]
        plans += [(name, text, None, status, out, err, name) for name, text, status, out, err in FIXTURES]
        report["plan"] = [{"name": n, "panic_override": pa, "runtime_status": s, "stdout": o, "stderr": e,
                           "source_sha256": hashlib.sha256(t.encode()).hexdigest()} for n,t,pa,s,o,e,_ in plans]
        save()
        for name, text, panic, expected, expected_out, expected_err, project in plans:
            path = root / "projects" / project
            (path / "src").mkdir(parents=True, exist_ok=True)
            source_path = path / "src/main.rs"
            source_path.write_text(text)
            manifest = '[package]\nname="semantic_probe"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[profile.dev]\ndebug=0\nincremental=false\n'
            if panic:
                manifest += 'panic="' + panic + '"\n'
            (path / "Cargo.toml").write_text(manifest)
            case = {"name": name, "engines": {}, "source_sha256": sha(source_path), "manifest_sha256": sha(path / "Cargo.toml")}
            report["cases"].append(case)
            for engine, environment, cargo in [("reference", reference, sdk / "bin/cargo"), ("candidate", env, package / "bin/cargo")]:
                observed_before = set((root / "observations").glob("*/report.json"))
                options = dict(environment, CARGO_TARGET_DIR=str(path / (engine + "-target")), CARGO_BUILD_BUILD_DIR=str(path / (engine + "-intermediates")))
                result = {"passed": False}
                case["engines"][engine] = result
                try:
                    built, out, err = run(name + "-" + engine + "-build", [cargo, "build", "--bin", "semantic_probe", "--offline", "--message-format=json"], path, options, engine == "candidate")
                    result["compile"] = built["label"]
                    if engine == "candidate":
                        compiles = [e for e in built["executions"] if e["result"] == "0" and Path(e["path"]).resolve() == (package / "bin/rustc").resolve() and "src/main.rs" in e["argv"]]
                        assert len(compiles) == 1, "actual Cargo compilation not observed exactly once"
                        result["compiler_argv"] = compiles[0]["argv"]
                        result["native_records"] = [json.loads(p.read_text()) for p in set((root / "observations").glob("*/report.json")) - observed_before]
                    if expected is None:
                        assert built["status"] == 101 and (b"E0308" if engine == "reference" else b"L0110") in out + err
                        result["runtime_executed"] = False
                    else:
                        assert built["status"] == 0, "compilation failed before runtime"
                        notifications = [json.loads(line) for line in out.splitlines() if line.startswith(b"{")]
                        executables = [r["executable"] for r in notifications if r.get("reason") == "compiler-artifact" and r.get("executable")]
                        assert len(executables) == 1
                        executable = Path(executables[0])
                        result["binary_sha256"] = sha(executable)
                        runtime, stdout, stderr = run(name + "-" + engine + "-runtime", [executable], path, options, timeout=15)
                        result["runtime"] = runtime["label"]
                        result["runtime_executed"] = True
                        assert runtime["status"] == expected and stdout == expected_out.encode() and stderr == expected_err.encode(), "runtime differs from predeclared behavior"
                    result["passed"] = True
                except Exception as exc:
                    result["failure"] = str(exc)
            case["passed"] = all(r["passed"] for r in case["engines"].values())
            print(json.dumps({"name": name, "passed": case["passed"], "failures": {k:v.get("failure") for k,v in case["engines"].items() if not v["passed"]}}), flush=True)
            save()
    except Exception:
        report["errors"].append(traceback.format_exc())
    finally:
        if owner:
            try:
                os.kill(owner, signal.SIGTERM)
            except ProcessLookupError:
                pass
        try:
            daemon.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(daemon.pid, signal.SIGKILL)
            daemon.wait()
        for h in handles:
            h.close()
        sockdir.cleanup()
        try:
            report["daemon_executions"] = reader.exec_events((root / "daemon.trace").read_bytes())
            inspect(report["daemon_executions"])
        except Exception:
            report["errors"].append(traceback.format_exc())
        report["passed"] = len(report["cases"]) == 14 and all(c["passed"] for c in report["cases"]) and not report["errors"] and not report["unexpected_candidate_executions"]
        save()
    print(json.dumps({"passed": report["passed"], "cases": len(report["cases"]), "errors": report["errors"], "unexpected_executions": report["unexpected_candidate_executions"]}), flush=True)
    return 0 if report["passed"] else 1

if __name__ == "__main__":
    raise SystemExit(main())
