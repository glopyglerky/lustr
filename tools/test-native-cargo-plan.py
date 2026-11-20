#!/usr/bin/env python3
"""Check native Cargo planning, exact results and retained planning reuse.

The installed package is copied into a test-only query-spy package. Cargo itself
keeps its pinned bytes; the spy forwards only information requests to the exact
public Lustr executable and refuses compiler invocations. Independent Rust runs
only in explicitly recorded oracle commands. Default execution requires a real
retained daemon and syscall tracing. --cold-only reports the narrower local
controls without claiming retained reuse or performance.
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
import sys
import time


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def process_identity(pid, executable):
    """Linux identity of the actual owned executable, not its tracer."""
    try:
        if Path(f"/proc/{pid}/exe").resolve(strict=True) != executable.resolve(strict=True):
            return None
        fields = Path(f"/proc/{pid}/stat").read_text().rsplit(") ", 1)[1].split()
        return None if fields[0] == "Z" else (pid, fields[19])
    except (OSError, IndexError):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--sdk", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cold-only", action="store_true")
    args = parser.parse_args()
    source, original, sdk = [p.resolve(strict=True) for p in [args.source, args.package, args.sdk]]
    output = args.output.absolute()
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema": "lustr-native-cargo-planning-controls-v1", "passed": False,
              "scope": "cold-only" if args.cold_only else "actual retained compiler and traced Cargo/tool-hash reuse",
              "performance_claim": False, "full_rust_coverage": False, "commands": [], "controls": [],
              "entry_sha256": digest(original / "bin/lustr"), "core_sha256": digest(original / "bin/lust-core"),
              "runner_sha256": digest(__file__), "oracle_sha256": digest(sdk / "bin/rustc")}
    daemon = None
    daemon_process = None
    handles = []

    def save():
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    def require(condition, message):
        if not condition:
            raise AssertionError(message)

    try:
        pins = dict(re.findall(r'^([a-z0-9_]+) = "([^\n]+)"$', (source / "upstream/bluefe-core.pin").read_text(), re.M))
        require(digest(sdk / "bin/rustc") == pins["rustc_sha256"], "independent Rust oracle is not pinned")
        require(digest(original / "bin/cargo") == digest(sdk / "bin/cargo") == pins["cargo_sha256"], "Cargo is not pinned")
        report.update(declared_rust_source=pins["source"], source_pin_sha256=digest(source / "upstream/bluefe-core.pin"),
                      cargo_sha256=digest(original / "bin/cargo"), original_query_receipt_sha256=digest(original / "lib/lustr-selection-tools.txt"))
        package = output / "query-spy-package"
        binary = package / "bin"
        binary.mkdir(parents=True)
        (package / "lib/lust").mkdir(parents=True)
        for name in ["lust", "lust-core"]:
            shutil.copy2(original / "bin" / name, binary / name)
        shutil.copy2(original / "bin/lustr", binary / "lustr-entry")
        (binary / "cargo").symlink_to((original / "bin/cargo").resolve())
        for name in ["library", "tcc"]:
            (package / "lib/lust" / name).symlink_to((original / "lib/lust" / name).resolve())
        (package / "lib/tcc").symlink_to((original / "lib/tcc").resolve())
        queries = output / "compiler-queries.jsonl"
        queries.write_text("")
        # These are the three exact information queries of the pinned Cargo
        # for this fixed Linux package. Adding --print to a compiler request
        # does not make it an information-only operation.
        query = ["-", "--crate-name", "___", "--print=file-names",
                 "--crate-type", "bin", "--crate-type", "rlib", "--crate-type", "dylib", "--crate-type", "cdylib",
                 "--crate-type", "staticlib", "--crate-type", "proc-macro", "--print=sysroot", "--print=split-debuginfo",
                 "--print=crate-name", "--print=cfg", "-Wwarnings"]
        allowed_queries = [["-vV"], query, query[:4] + ["--target", "x86_64-unknown-linux-gnu"] + query[4:]]
        spy = (f"#!{sys.executable}\nimport json,os,sys\nargs=sys.argv[1:]\n"
               f"allowed=args in {allowed_queries!r}\n"
               'allowed=allowed and ("-" not in args or sys.stdin.buffer.read(1)==b"")\n'
               f"with open({str(queries)!r},'a') as f: f.write(json.dumps(dict(argv=args,query_only=allowed,control=os.environ.get('LUSTR_QUERY_CONTROL'))) + '\\n')\n"
               'if not allowed: print("compiler invocation refused by qualification query spy",file=sys.stderr);sys.exit(47)\n'
               f"os.execv({str(binary / 'lustr-entry')!r},['lustr',*args])\n")
        (binary / "lustr").write_text(spy)
        (binary / "lustr").chmod(0o755)
        (binary / "rustc").symlink_to("lustr")
        receipt = f"lustr-native-selection-toolchain-v2\nsource={pins['source']}\nhost=x86_64-unknown-linux-gnu\n"
        receipt += "".join(f"{digest(package / name)}  {name}\n" for name in ["bin/cargo", "bin/lustr", "bin/rustc"])
        (package / "lib/lustr-selection-tools.txt").write_text(receipt)
        report["query_package"] = {name: digest(binary / name) for name in ["lustr-entry", "lustr", "rustc", "lust", "lust-core", "cargo"]}
        project = output / "project"
        for name in ["src", "util/src", "unused/src"]:
            (project / name).mkdir(parents=True)
        manifest = project / "Cargo.toml"
        manifest.write_text('[package]\nname="plan-control"\nversion="0.1.0"\nedition="2024"\n'
                            '[workspace]\nmembers=["util","unused"]\nresolver="3"\n'
                            '[features]\ndefault=[]\nfast=["renamed/fast"]\n'
                            '[dependencies]\nrenamed={package="util",path="util",default-features=false}\n')
        (project / "util/Cargo.toml").write_text('[package]\nname="util"\nversion="0.1.0"\nedition="2024"\n[features]\ndefault=[]\nfast=[]\n')
        (project / "util/src/lib.rs").write_text('#[cfg(feature="fast")] pub fn value() -> i32 { 22 }\n#[cfg(not(feature="fast"))] pub fn value() -> i32 { 11 }\n')
        (project / "unused/Cargo.toml").write_text('[package]\nname="unused"\nversion="0.1.0"\nedition="2024"\n')
        (project / "unused/src/lib.rs").write_text('compile_error!("unselected workspace package must not be analyzed");\n')
        main_source = project / "src/main.rs"
        main_source.write_text('fn main() { println!("{}", renamed::value()); }\n')
        for name in ["home", "cargo-home", "tmp"]:
            (output / name).mkdir()
        environment = {"PATH": "/usr/local/bin:/usr/bin:/bin", "HOME": str(output / "home"),
                       "CARGO_HOME": str(output / "cargo-home"), "CARGO_TARGET_DIR": str(output / "cargo-target"),
                       "CARGO_BUILD_BUILD_DIR": str(output / "cargo-build"), "CARGO_NET_OFFLINE": "true",
                       "TMPDIR": str(output / "tmp"), "LANG": "C", "LC_ALL": "C", "RUST_BACKTRACE": "0",
                       "LUST_CACHE_DIR": str(output / "native-cache"), "LUST_DAEMON_IDLE": "120"}
        report["environment"] = environment
        traces = output / "daemon-exec.trace"
        traced = not args.cold_only
        strace = shutil.which("strace") if traced else None
        require(not traced or strace is not None, "retained process proof requires strace")
        trace_prefix = lambda path: [strace, "-qq", "-f", "-e", "trace=execve", "-s", "4096", "-o", str(path)] if traced else []

        def execute(argv, context, role, trace_process=False):
            prefix = f"{len(report['commands']):03d}-{role}"
            trace = output / (prefix + ".exec.trace")
            actual = trace_prefix(trace) + list(map(str, argv)) if trace_process else list(map(str, argv))
            process = subprocess.Popen(actual, cwd=project, env=context, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            timed_out = False
            try:
                stdout, stderr = process.communicate(timeout=60)
            except subprocess.TimeoutExpired:
                timed_out = True
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                stdout, stderr = process.communicate(timeout=5)
            result = subprocess.CompletedProcess(actual, process.returncode, stdout, stderr)
            (output / (prefix + ".stdout")).write_bytes(result.stdout)
            (output / (prefix + ".stderr")).write_bytes(result.stderr)
            report["commands"].append({"argv": actual, "role": role, "cwd": str(project), "exit": result.returncode,
                                       "stdout_file": prefix + ".stdout", "stderr_file": prefix + ".stderr", "timed_out": timed_out,
                                       "environment_changes": {k: v for k, v in context.items() if environment.get(k) != v}})
            save()
            require(not timed_out, role + ": owned process group timed out and was terminated")
            return result, trace

        # The spy must refuse compilation even if it contains a print flag.
        rejected, _ = execute([binary / "lustr", "src/main.rs", "--print=link-args"], dict(environment, LUSTR_QUERY_CONTROL="nonquery"), "query-spy-refusal")
        require(rejected.returncode == 47 and rejected.stdout == b"", "query spy admitted a compilation-class request")
        report["controls"].append({"case": "query-spy-refuses-compilation-with-print", "passed": True})

        if traced:
            endpoint = output / "daemon.sock"
            require(len(os.fsencode(endpoint)) <= 107, "choose a shorter output path for the real Unix socket")
            environment["LUST_DAEMON_SOCKET"] = str(endpoint)
            handles = [(output / "daemon.stdout").open("wb"), (output / "daemon.stderr").open("wb")]
            argv = trace_prefix(traces) + [str(binary / "lust-core"), "daemon"]
            daemon = subprocess.Popen(argv, cwd=project, env=environment, stdout=handles[0], stderr=handles[1], start_new_session=True)
            report["daemon_argv"] = argv
            deadline = time.monotonic() + 10
            while daemon.poll() is None and time.monotonic() < deadline:
                children = Path(f"/proc/{daemon.pid}/task/{daemon.pid}/children").read_text().split()
                identities = [identity for child in children if (identity := process_identity(int(child), binary / "lust-core")) is not None]
                require(len(identities) <= 1, "tracer started more than one native daemon")
                if identities and endpoint.exists():
                    daemon_process = identities[0]
                    break
                time.sleep(0.02)
            require(daemon_process is not None and daemon.poll() is None, "owned traced daemon failed to start")
            report["daemon_process_identity"] = daemon_process

        def candidate(label, flags=(), no_plan_processes=False):
            context = dict(environment)
            if args.cold_only:
                context["LUST_NO_DAEMON"] = "1"
            owner_trace = output / (label + ".owner.jsonl")
            context["LUSTR_TRACE_PATH"] = str(owner_trace)
            before = traces.stat().st_size if traces.exists() else 0
            artifact = project / "native-out"
            artifact.unlink(missing_ok=True)
            require(daemon_process is None or process_identity(daemon_process[0], binary / "lust-core") == daemon_process, "original native daemon exited before request")
            source_hashes = {str(path.relative_to(project)): digest(path) for path in project.rglob("*") if path.is_file() and (path.suffix == ".rs" or path.name == "Cargo.toml")}
            result, client_trace = execute([binary / "lustr-entry", "build", "src/main.rs", "--manifest-path", "Cargo.toml", "--offline", *flags, "-o", artifact], context, label, trace_process=traced)
            require(result.returncode == 0 and artifact.is_file(), f"{label}: native build failed: {result.stderr!r}")
            require(daemon_process is None or process_identity(daemon_process[0], binary / "lust-core") == daemon_process, "original native daemon exited during request")
            require(all(digest(project / name) == checksum for name, checksum in source_hashes.items()), "candidate changed a source or manifest")
            events = [json.loads(line) for line in owner_trace.read_text().splitlines()]
            require(len(events) == 1 and events[0]["engine"] == "lust", "request has no exclusive native owner")
            if traced:
                require(events[0]["entry"] == "daemon", "request was not served by retained daemon")
            artifact_hash = digest(artifact)
            actual, _ = execute([artifact], context, label + "-program")
            require(digest(artifact) == artifact_hash, "native executable changed while running")
            oracle_context = dict(environment, RUSTC=str(sdk / "bin/rustc"))
            oracle, _ = execute([sdk / "bin/cargo", "build", "-q", "--offline", "--bin", "plan-control", *flags, "--message-format=json"], oracle_context, label + "-oracle-build")
            require(oracle.returncode == 0, f"independent oracle build failed: {oracle.stderr!r}")
            messages = [json.loads(line) for line in oracle.stdout.splitlines()]
            executables = [row["executable"] for row in messages if row.get("reason") == "compiler-artifact" and row.get("executable") and "bin" in row.get("target", {}).get("kind", [])]
            require(len(executables) == 1, "oracle did not bind one executable")
            oracle_hash = digest(executables[0])
            expected, _ = execute([executables[0]], environment, label + "-oracle-program")
            require(digest(executables[0]) == oracle_hash, "oracle executable changed while running")
            require((actual.returncode, actual.stdout, actual.stderr) == (expected.returncode, expected.stdout, expected.stderr), label + ": exact program result differs")
            require(all(digest(project / name) == checksum for name, checksum in source_hashes.items()), "oracle compared different source or manifests")
            record = {"case": label, "passed": True, "source_sha256": source_hashes, "artifact_sha256": artifact_hash, "oracle_artifact_sha256": oracle_hash, "stdout": actual.stdout.decode()}
            if traced:
                trace_text = traces.read_bytes()[before:] + client_trace.read_bytes()
                programs = [os.fsdecode(p) for p in re.findall(rb'execve\("([^"\n]+)"', trace_text)]
                counts = {"cargo_processes": sum(Path(p).name == "cargo" for p in programs),
                          "tool_hash_processes": sum(Path(p).name in ("sha256sum", "shasum") for p in programs)}
                record.update(counts)
                if no_plan_processes:
                    require(counts == {"cargo_processes": 0, "tool_hash_processes": 0}, f"{label}: retained planning did repeated external work: {counts}")
            report["controls"].append(record)
            save()

        # A fresh explicit package must resolve its lockfile through normal
        # Cargo planning before pkgid; the query spy still forbids compilation.
        require(not (project / "Cargo.lock").exists(), "fresh package control already has a lockfile")
        candidate("explicit-package-fresh", ["--package", "plan-control@0.1.0"])
        require((project / "Cargo.lock").is_file(), "explicit package planning did not resolve its lockfile")
        candidate("initial")
        candidate("unchanged", no_plan_processes=True)
        main_source.write_text('fn main() { println!("{}", renamed::value() + 5); }\n')
        candidate("body-edit", no_plan_processes=True)
        candidate("feature-selection", ["--features", "fast"])
        candidate("feature-recovery", no_plan_processes=True)
        # Cargo owns the package-spec grammar. This path still performs the
        # public package binding before the same native unit compiler; its
        # planning work is reported, not covered by the warm zero-process claim.
        candidate("explicit-package", ["--package", "plan-control@0.1.0"])
        events = [json.loads(line) for line in queries.read_text().splitlines()]
        cargo_events = [row for row in events if row["control"] is None]
        require(cargo_events and all(row["query_only"] for row in cargo_events), "Cargo attempted a compiler/probe request")
        require(len(events) == len(cargo_events) + 1 and events[0]["control"] == "nonquery" and not events[0]["query_only"], "query spy negative control was not recorded")
        report.update(query_count=len(cargo_events), query_spy_sha256=digest(binary / "lustr"), passed=True)
    except Exception as error:
        report["failure"] = repr(error)
    finally:
        if daemon is not None:
            if daemon.poll() is None:
                try:
                    os.killpg(daemon.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            try:
                daemon.wait(timeout=5)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(daemon.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                daemon.wait(timeout=5)
                report.update(passed=False, cleanup_failure="owned daemon did not terminate")
        # A broken public entry might start a replacement daemon in its own
        # session. Stop only core daemon PIDs observed in this test's syscall
        # traces whose current executable still matches the private package.
        if "binary" in locals():
            core = binary / "lust-core"
            for trace in output.glob("*.trace"):
                for match in re.finditer(rb'(?m)^\s*(\d+)\s+execve\("([^"\n]+)"', trace.read_bytes()):
                    pid = int(match[1])
                    if os.fsdecode(match[2]) != str(core) or process_identity(pid, core) is None:
                        continue
                    try:
                        arguments = Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")
                        if len(arguments) > 1 and arguments[1] == b"daemon":
                            if os.getpgid(pid) == pid:
                                os.killpg(pid, signal.SIGKILL)
                            else:
                                os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    except OSError as error:
                        report.update(passed=False, cleanup_failure=str(error))
        for handle in handles:
            handle.close()
        save()
    print(json.dumps({key: report.get(key) for key in ["passed", "scope", "failure", "query_count", "controls"]}, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
