#!/usr/bin/env python3
"""Compare Rust profile semantics through installed Lustr and pinned Rust.

Every case uses a fresh source and output identity. The candidate requests use
one isolated native owner, but this is cold correctness evidence, not retained
edit qualification or a speed measurement. Cargo chooses its actual compiler
invocation; `cargo rustc` supplies the final request's explicit profile options.

Reference: Rust 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494,
compiler/rustc_session/src/session.rs, Session::overflow_checks.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import traceback
import unittest

PINNED_SOURCE = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494"
RUSTC_SHA = "ca9d833d09469a534ca1449e71ca9961608409cdb257df96cf8035338fac28de"
CARGO_SHA = "da29c2a9ff7f85bda5587e9c974e05c9e2a7196377e54466f6e9d6f9e00b2530"
TARGET = "x86_64-unknown-linux-gnu"
CASES = [
    ("defaults", [], True, True),
    ("debug-off-implicit-overflow", ["debug-assertions=off"], False, False),
    ("debug-off-explicit-overflow-on", ["debug-assertions=off", "overflow-checks=on"], False, True),
    ("debug-on-explicit-overflow-off", ["debug-assertions=on", "overflow-checks=off"], True, False),
    ("debug-last-off", ["debug-assertions=on", "debug-assertions=off"], False, False),
    ("debug-last-on", ["debug-assertions=off", "debug-assertions=on"], True, True),
    ("overflow-last-off", ["debug-assertions=off", "overflow-checks=on", "overflow-checks=off"], False, False),
    ("overflow-last-on", ["debug-assertions=off", "overflow-checks=off", "overflow-checks=on"], False, True),
]
FIXTURES = {
    "overflow": ('fn add(x: u64) -> u64 { x + 1 }\n'
                 'fn main() { println!("wrapped={}", add(18446744073709551615u64)); }\n',
                 b"wrapped=0\n", b"attempt to add with overflow"),
    "debug-assert": ('fn main() { debug_assert!(false, "profile-default-debug-assertion"); '
                     'println!("debug-ok"); }\n', b"debug-ok\n", b"profile-default-debug-assertion"),
}


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def digest(path):
    with Path(path).open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def identity(path):
    path = Path(path)
    return {"path": str(path), "resolved": str(path.resolve(strict=True)),
            "bytes": path.stat().st_size, "sha256": digest(path)}


def write_json(path, value):
    Path(path).write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")


def exec_events(data):
    """Read the selected exec syscalls; truncation or unknown records fail."""
    require(not data or data.endswith(b"\n"), "incomplete exec trace")
    pending, events = {}, []
    decoder = json.JSONDecoder()
    for line in data.decode().splitlines():
        match = re.fullmatch(r"\s*(\d+)\s+(.*)", line)
        require(match is not None, "exec trace lacks a PID: " + line)
        pid, record = match.groups()
        if record.startswith("--- SIG") or record.startswith("+++"):
            continue
        if record.endswith(" <unfinished ...>"):
            require(pid not in pending and record.startswith("execve("), "unrepresented unfinished exec")
            pending[pid] = record.removesuffix(" <unfinished ...>")
            continue
        if record.startswith("<... execve resumed>"):
            require(pid in pending, "unmatched resumed exec")
            record = pending.pop(pid) + record.removeprefix("<... execve resumed>")
        require(record.startswith("execve("), "unrepresented exec syscall: " + record)
        filename, end = decoder.raw_decode(record[7:])
        tail = record[7 + end:]
        require(tail.startswith(", "), "missing exec arguments")
        argv, end = decoder.raw_decode(tail[2:])
        tail = tail[2 + end:]
        result = re.fullmatch(r", (?:0x[0-9a-f]+|NULL)(?: /\* \d+ vars \*/)?\)\s+=\s+(.*)", tail)
        require(result is not None and isinstance(filename, str) and isinstance(argv, list)
                and argv and all(isinstance(arg, str) for arg in argv), "incomplete exec payload")
        result = result.group(1)
        require(result == "0" or re.fullmatch(r"-1 [A-Z0-9_]+ \([^\n]*\)", result), "unknown exec result")
        events.append({"pid": int(pid), "path": filename, "argv": argv, "result": result})
    require(not pending and events, "missing or unfinished process execution")
    return events


def profile_arguments(argv):
    """Retain all explicit values, including the direct driver's last-wins cases."""
    result = {}
    args = iter(argv[1:])
    for arg in args:
        if arg == "-C":
            value = next(args, None)
            require(value is not None, "missing codegen value in actual invocation")
        elif arg.startswith("-C"):
            value = arg[2:]
        else:
            continue
        key, sep, value = value.partition("=")
        require(sep, "valueless codegen option in actual invocation")
        result.setdefault(key, []).append(value)
    return result


def check_runtime(status, stdout, stderr, fixture, panics):
    _, success, marker = FIXTURES[fixture]
    if panics:
        return status == -signal.SIGABRT and stdout == b"" and marker in stderr and b"panicked at" in stderr
    return status == 0 and stdout == success and stderr == b""


class Controls(unittest.TestCase):
    def test_crash_and_compile_error_are_not_runtime_panics(self):
        marker = b"panicked at fixture:1:1: attempt to add with overflow"
        self.assertTrue(check_runtime(-signal.SIGABRT, b"", marker, "overflow", True))
        for code in (0, 1, 101, -signal.SIGSEGV):
            self.assertFalse(check_runtime(code, b"", marker, "overflow", True))
        self.assertFalse(check_runtime(-signal.SIGABRT, b"", b"", "overflow", True))

    def test_exact_success_and_explicit_last_wins_inventory(self):
        self.assertTrue(check_runtime(0, b"wrapped=0\n", b"", "overflow", False))
        self.assertFalse(check_runtime(0, b"wrapped=1\n", b"", "overflow", False))
        self.assertEqual(profile_arguments(["rustc", "-Cdebug-assertions=on", "-C", "debug-assertions=off"]),
                         {"debug-assertions": ["on", "off"]})

    def test_exec_trace_controls(self):
        valid = b'12 execve("/compiler", ["/compiler", "a.rs"], 0x123 /* 3 vars */) = 0\n'
        self.assertEqual(exec_events(valid)[0]["argv"], ["/compiler", "a.rs"])
        for bad in (valid[:-1], valid.replace(b'["/compiler", "a.rs"]', b'["/compiler", ...]'),
                    valid.replace(b'execve(', b'execveat('), b'12 <... execve resumed>) = 0\n'):
            with self.assertRaises((RuntimeError, ValueError)):
                exec_events(bad)
        split = (b'12 execve("/compiler", ["/compiler"], 0x123 /* 3 vars */ <unfinished ...>\n'
                 b'12 <... execve resumed>) = 0\n')
        self.assertEqual(exec_events(split)[0]["result"], "0")


def run(args):
    package, sdk = args.package.resolve(strict=True), args.sdk.resolve(strict=True)
    root = args.output.absolute()
    root.mkdir(parents=True, exist_ok=False)
    report = {"schema": "lustr-rustc-profile-defaults-v1", "passed": False,
              "scope": "cold semantic comparisons through direct Rust requests and real Cargo binaries; fresh source identities",
              "retained_edit_qualified": False, "performance_qualified": False, "full_rust_qualified": False,
              "pinned_rust_source": PINNED_SOURCE, "case_plan": CASES, "commands": [], "cases": [], "errors": []}
    save = lambda: write_json(root / "results.json", report)
    daemon, owner, handles, socket_dir = None, None, [], None
    strace = shutil.which("strace", path=os.pathsep.join([*map(str, args.tools_dir), os.environ.get("PATH", "")]))
    traces = []
    try:
        require(sys.platform == "linux" and strace, "Linux and strace are required for no-fallback evidence")
        require(digest(sdk / "bin/rustc") == RUSTC_SHA and digest(sdk / "bin/cargo") == CARGO_SHA,
                "reference compiler/Cargo differs from the declared pinned SDK")
        require(digest(package / "bin/cargo") == CARGO_SHA, "candidate Cargo differs from the pinned original")
        require((package / "bin/rustc").resolve() == (package / "bin/lustr").resolve(),
                "candidate rustc entry is not Lustr")
        paths = [package / "bin" / name for name in ("rustc", "lust", "lust-core", "cargo")]
        paths += [sdk / "bin/rustc", sdk / "bin/cargo", Path(__file__), Path(strace)]
        report["tools"] = [identity(path) for path in paths]
        for path in (package / "PARTIAL-NATIVE-COMPILER", package / "lib/lustr-metadata/sdk-receipt.json",
                     package.parent / "COMPONENTS.txt", package.parent / "logs/source-revision.stdout"):
            if path.is_file():
                report.setdefault("package_records", []).append(identity(path))
        if args.source_evidence:
            evidence = args.source_evidence.resolve(strict=True)
            value = json.loads(evidence.read_bytes())
            shutil.copyfile(evidence, root / "source-evidence.json")
            report["source_evidence"] = {**identity(evidence), "record": "source-evidence.json",
                                         "head": value.get("head"), "tree": value.get("tree"),
                                         "native_tree": value.get("native_tree")}
        for name in ("commands", "cases", "tmp", "cargo-home", "native-cache"):
            (root / name).mkdir()
        socket_dir = tempfile.TemporaryDirectory(prefix="lustr-profile-")
        endpoint = Path(socket_dir.name) / "owner.sock"
        environment = {"PATH": os.pathsep.join([str(package / "bin"), *map(str, args.tools_dir), "/usr/bin", "/bin"]),
                       "LANG": "C", "LC_ALL": "C", "CARGO_HOME": str(root / "cargo-home"),
                       "CARGO_NET_OFFLINE": "true", "CARGO_BUILD_JOBS": "1", "CARGO_TERM_COLOR": "never",
                       "TMPDIR": str(root / "tmp"), "RUST_BACKTRACE": "0", "RUST_MIN_STACK": "268435456",
                       "RUSTC": str(package / "bin/rustc"), "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
                       "LUST_CACHE_DIR": str(root / "native-cache"), "LUST_DAEMON_SOCKET": str(endpoint),
                       "LUST_DAEMON_IDLE": "1800", "LUST_THREADS": "2"}
        if "HOME" in os.environ:
            environment["HOME"] = os.environ["HOME"]
        original = dict(environment, PATH=os.pathsep.join([str(sdk / "bin"), *map(str, args.tools_dir), "/usr/bin", "/bin"]),
                        RUSTC=str(sdk / "bin/rustc"), LD_LIBRARY_PATH=str(sdk / "lib"))
        for key in list(original):
            if key.startswith("LUST_"):
                del original[key]
        report["candidate_environment"] = {k: v for k, v in environment.items() if k != "HOME"}
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

        def trace_command(path, argv):
            return [strace, "-f", "-qq", "-s", "1048576", "-e", "trace=execve,execveat", "-o", str(path), *map(str, argv)]

        def command(label, argv, cwd, env, *, traced=False, timeout=60):
            directory = root / "commands" / label
            directory.mkdir()
            row = {"label": label, "argv": list(map(str, argv)), "cwd": str(cwd)}
            report["commands"].append(row)
            save()
            trace = directory / "exec.trace"
            invocation = trace_command(trace, argv) if traced else list(map(str, argv))
            process = subprocess.Popen(invocation, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       start_new_session=True)
            try:
                stdout, stderr = process.communicate(timeout=timeout)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                stdout, stderr = process.communicate(timeout=5)
                row["timeout"] = True
            row["status"] = process.returncode
            for name, data in (("stdout", stdout), ("stderr", stderr)):
                path = directory / name
                path.write_bytes(data)
                row[name] = {"path": str(path.relative_to(root)), "sha256": digest(path), "bytes": len(data)}
            if traced:
                row["trace"] = str(trace.relative_to(root))
                traces.append(trace)
                row["executions"] = exec_events(trace.read_bytes())
            save()
            return row, stdout, stderr

        daemon_trace = root / "daemon.trace"
        handles = [(root / "daemon.stdout").open("wb"), (root / "daemon.stderr").open("wb")]
        daemon = subprocess.Popen(trace_command(daemon_trace, [package / "bin/lust-core", "daemon"]),
                                  cwd=root, env=environment, stdout=handles[0], stderr=handles[1], start_new_session=True)
        deadline = time.monotonic() + 20
        while daemon.poll() is None and time.monotonic() < deadline:
            children = Path(f"/proc/{daemon.pid}/task/{daemon.pid}/children").read_text().split()
            for pid in children:
                try:
                    if Path(f"/proc/{pid}/exe").resolve(strict=True) == (package / "bin/lust-core").resolve() and endpoint.exists():
                        owner = int(pid)
                except FileNotFoundError:
                    pass
            if owner is not None:
                break
            time.sleep(0.02)
        require(owner is not None and daemon.poll() is None, "isolated native owner did not start")
        report["owner_pid"] = owner
        for mode in ("direct", "cargo"):
            for label, flags, debug, overflow in CASES if mode == "direct" else CASES[:4]:
                for fixture, (source_text, _, _) in FIXTURES.items():
                    case_name = f"{mode}-{label}-{fixture}"
                    case = root / "cases" / case_name
                    case.mkdir()
                    source = case / ("main.rs" if mode == "direct" else "src/main.rs")
                    source.parent.mkdir(exist_ok=True)
                    source.write_text(source_text)
                    row = {"name": case_name, "mode": mode, "fixture": fixture, "explicit_codegen": flags,
                           "expected_debug_assertions": debug, "expected_overflow_checks": overflow,
                           "expected_panic": overflow if fixture == "overflow" else debug,
                           "source": identity(source), "engines": {}, "passed": False}
                    report["cases"].append(row)
                    if mode == "cargo":
                        manifest = case / "Cargo.toml"
                        manifest.write_text('[package]\nname="profile_defaults"\nversion="0.1.0"\nedition="2024"\n'
                                            '[workspace]\n[profile.dev]\npanic="abort"\ndebug=0\nopt-level=0\nincremental=false\n')
                        row["manifest"] = identity(manifest)
                    for engine, toolchain, env in (("reference", sdk, original), ("candidate", package, environment)):
                        record = row["engines"][engine] = {"passed": False, "runtime_executed": False}
                        try:
                            options = [arg for flag in flags for arg in ("-C", flag)]
                            if mode == "direct":
                                binary = case / engine
                                argv = [toolchain / "bin/rustc", "--edition=2024", source, "-o", binary,
                                        "-Cpanic=abort", "-Copt-level=0", "-Cdebuginfo=0", *options]
                                compile_env = env
                            else:
                                target = case / (engine + "-target")
                                compile_env = dict(env, CARGO_TARGET_DIR=str(target), CARGO_BUILD_BUILD_DIR=str(case / (engine + "-build")))
                                argv = [toolchain / "bin/cargo", "rustc" if flags else "build", "--bin", "profile_defaults",
                                        "--offline", "--message-format=json"]
                                if flags:
                                    argv += ["--", *options]
                            built, stdout, _ = command(case_name + "-" + engine + "-build", argv, case, compile_env, traced=True)
                            record["compile"] = built["label"]
                            require(built["status"] == 0 and not built.get("timeout"), "compilation failed; no runtime outcome exists")
                            if mode == "cargo":
                                artifacts = [json.loads(line) for line in stdout.splitlines() if line.startswith(b"{")]
                                paths = [Path(item["executable"]) for item in artifacts
                                         if item.get("reason") == "compiler-artifact" and item.get("executable")
                                         and item.get("target", {}).get("name") == "profile_defaults"]
                                require(len(paths) == 1 and paths[0].is_file() and paths[0].is_relative_to(target),
                                        "Cargo did not report exactly one executable in the selected target directory")
                                binary = paths[0]
                            require(binary.is_file(), "successful compiler did not produce its requested executable")
                            compiles = [event for event in built["executions"] if event["result"] == "0"
                                        and Path(event["path"]).resolve() == (toolchain / "bin/rustc").resolve()
                                        and any(arg.endswith(".rs") for arg in event["argv"][1:])]
                            require(len(compiles) == 1, "expected exactly one observed Rust source compilation")
                            actual = profile_arguments(compiles[0]["argv"])
                            record["actual_compiler_argv"] = compiles[0]["argv"]
                            record["actual_codegen"] = actual
                            for key in ("debug-assertions", "overflow-checks"):
                                wanted = [value.split("=", 1)[1] for value in flags if value.startswith(key + "=")]
                                require(actual.get(key, []) == wanted, "actual Rust profile request differs from the predeclared case")
                            record["binary"] = identity(binary)
                            result, out, err = command(case_name + "-" + engine + "-run", [binary], case, env, timeout=10)
                            record["runtime_executed"] = True
                            record["runtime"] = result["label"]
                            record["passed"] = not result.get("timeout") and check_runtime(result["status"], out, err, fixture, row["expected_panic"])
                            require(record["passed"], "runtime differs from the predeclared Rust behavior")
                        except Exception as error:
                            record["failure"] = str(error)
                        save()
                    row["passed"] = all(value["passed"] for value in row["engines"].values())
                    save()
    except Exception as error:
        report["errors"].append(str(error))
        report["failure_traceback"] = traceback.format_exc()
    finally:
        if daemon is not None:
            if daemon.poll() is None:
                try:
                    os.kill(owner, signal.SIGTERM) if owner is not None else os.killpg(daemon.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            try:
                daemon.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(daemon.pid, signal.SIGKILL)
                daemon.wait(timeout=5)
                report["errors"].append("owned daemon required forced termination")
        for handle in handles:
            handle.close()
        if socket_dir is not None:
            socket_dir.cleanup()
        if owner is not None:
            try:
                report["daemon_executions"] = exec_events((root / "daemon.trace").read_bytes())
                allowed = {(package / "bin" / name).resolve() for name in ("rustc", "lustr", "lust", "lust-core", "cargo")}
                allowed |= {(package / "lib/lust/tcc/bin/tcc").resolve()}
                allowed |= {(package / f"lib/rustlib/{TARGET}/bin" / name).resolve() for name in ("llvm-ar", "llvm-objcopy")}
                for name in ("cc", "gcc", "clang", "as", "ld", "ld.lld", "sha256sum"):
                    path = shutil.which(name, path=environment["PATH"])
                    if path:
                        allowed.add(Path(path).resolve())
                allowed |= {path.resolve() for path in Path("/usr/lib/gcc").glob("*/*/collect2")}
                observed = report["daemon_executions"] + [event for command in report["commands"]
                    if "-candidate-build" in command["label"] for event in command.get("executions", [])]
                report["unexpected_candidate_executions"] = [event for event in observed
                    if event["result"] == "0" and Path(event["path"]).resolve() not in allowed]
                require(not report["unexpected_candidate_executions"], "candidate executed an undeclared compiler or process")
                report["no_fallback_observed"] = True
            except Exception as error:
                report["errors"].append("candidate execution verification: " + str(error))
        report["expected_case_count"] = 24
        report["passed_case_count"] = sum(case["passed"] for case in report["cases"])
        report["passed"] = (len(report["cases"]) == 24 and report["passed_case_count"] == 24
                            and report.get("no_fallback_observed", False) and not report["errors"])
        save()
    print(json.dumps({key: report[key] for key in ("passed", "expected_case_count", "passed_case_count", "errors")}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        unittest.main(argv=[sys.argv[0]])
    else:
        parser = argparse.ArgumentParser(description=__doc__)
        parser.add_argument("--package", type=Path, required=True)
        parser.add_argument("--sdk", type=Path, required=True)
        parser.add_argument("--output", type=Path, required=True)
        parser.add_argument("--tools-dir", type=Path, action="append", default=[], help="additional explicit host-tool directories")
        parser.add_argument("--source-evidence", type=Path, help="retained source manifest for this exact compiler package")
        sys.exit(run(parser.parse_args()))
