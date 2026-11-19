#!/usr/bin/env python3
"""Qualify pinned retained histories through public Lustr with independent verdicts.

Reuse the donor's fixture ordering, overlays, daemon lifecycle and old controls.
Do not reuse its rule that two rejections of valid Rust are an unsupported pass.
Observe check, build and runtime separately. Each requested executable runs once.
This finite qualification never selects production ownership or measures speed.
"""
import argparse
from concurrent.futures import ProcessPoolExecutor
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import stat
import subprocess
import sys
import types


ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def tree_id(directory):
    """Compute Git's tree identity from actual fixture bytes, modes and links."""
    entries = []
    for path in directory.iterdir():
        info = path.lstat()
        if stat.S_ISDIR(info.st_mode):
            mode, value, key = b"40000", tree_id(path), os.fsencode(path.name) + b"/"
        else:
            require(stat.S_ISREG(info.st_mode) or stat.S_ISLNK(info.st_mode), "unsupported fixture file kind")
            data = os.fsencode(os.readlink(path)) if path.is_symlink() else path.read_bytes()
            mode = b"120000" if path.is_symlink() else b"100755" if info.st_mode & 0o111 else b"100644"
            value = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
            key = os.fsencode(path.name)
        entries.append((key, mode + b" " + os.fsencode(path.name) + b"\0" + bytes.fromhex(value)))
    data = b"".join(value for _, value in sorted(entries))
    return hashlib.sha1(b"tree " + str(len(data)).encode() + b"\0" + data).hexdigest()


def load_inventory(source, manifest):
    inventory = json.loads(manifest.read_text())
    require(inventory["schema"] == "lustr-retained-scenario-inventory-v1", "unknown inventory schema")
    pin = re.search(r'^revision = "([0-9a-f]{40})"$', (source / "upstream/lust.pin").read_text(), re.M)
    require(pin is not None and inventory["upstream_revision"] == pin[1], "scenario inventory differs from the Lust pin")
    rows = inventory["scenarios"]
    require(rows and len({row["name"] for row in rows}) == len(rows), "empty or duplicated scenario inventory")
    require(all(re.fullmatch(r"[A-Za-z0-9_]+", row["name"]) and row["status"] in ("required", "unresolved") for row in rows), "invalid scenario identity or status")
    expected = {row["name"] for row in rows} | {row["name"] for row in inventory["integration_histories"]}
    directory = source / "compiler/native/tests/daemon"
    require({p.name for p in directory.iterdir() if p.is_dir()} == expected, "scenario directories changed without inventory review")
    for row in rows:
        require(tree_id(directory / row["name"]) == row["upstream_tree"], row["name"] + ": fixture differs from pinned upstream")
        require(type(row["historical_unsupported_versions"]) is int and row["historical_unsupported_versions"] >= 0, "invalid historical count")
        require((row["status"] == "required") == (row["historical_unsupported_versions"] == 0), "historical unresolved coverage was silently admitted")
    require(digest(source / "compiler/native/scripts/test-daemon-scenarios.py") == inventory["scenario_loader_sha256"], "scenario loader changed without review")
    for name, versions in inventory["runtime_signals"].items():
        require(name in expected, "signal expectation names an unknown scenario")
        for version, signals in versions.items():
            require((directory / name / version).exists() and signals == ["SIGABRT"], "invalid explicit program-signal expectation")
    for name, versions in inventory["panic_thread_id_versions"].items():
        require(name in expected and versions and len(versions) == len(set(versions)), "invalid panic-header inventory")
        require(all((directory / name / version).is_dir() for version in versions), "panic-header inventory names an unknown version")
    return inventory


def completed(result, allowed_signals=()):
    require(type(result.get("exit")) is int and (0 <= result["exit"] <= 255 or -result["exit"] in allowed_signals) and not result.get("timed_out"), "interrupted, signalled or missing process result")


def source_rejection(result, cwd, cargo=False):
    """Require structured source diagnostics from a semantic-only oracle check.

    A nonzero build can be a linker, SDK, manifest or tool failure. None proves
    invalid Rust. The separately invoked check must report an error whose
    primary span is in the unchanged fixture's source tree.
    """
    completed(result)
    if result["exit"] != (101 if cargo else 1):
        return False
    try:
        records = [json.loads(line) for line in result["stdout" if cargo else "stderr"].splitlines() if line.strip()]
        if cargo:
            records = [row["message"] for row in records if row.get("reason") == "compiler-message"]
        errors = [row for row in records if row.get("level") == "error"]
        source_errors = []
        for error in errors:
            if re.fullmatch(r"aborting due to \d+ previous errors?(?:; \d+ warnings? emitted)?", error.get("message", "")):
                continue
            spans = [span for span in error.get("spans", []) if span.get("is_primary") is True]
            anchored = False
            for span in spans:
                path = Path(span["file_name"])
                path = (Path(cwd) / path).resolve() if not path.is_absolute() else path.resolve()
                if path.is_relative_to(Path(cwd).resolve()) and path.is_file():
                    anchored = True
            if not anchored:
                return False
            source_errors.append(error)
        return bool(source_errors)
    except (ValueError, KeyError, TypeError):
        return False


def compare_runtime(actual, expected, allowed_signals=(), panic_thread_id=False):
    completed(actual, allowed_signals)
    completed(expected, allowed_signals)
    require(actual["exit"] == expected["exit"], "program exit differs from Rust")
    require(actual["stdout"] == expected["stdout"], "program stdout differs from Rust")
    actual_stderr, expected_stderr = actual["stderr"], expected["stderr"]
    if panic_thread_id:
        # Only independently reviewed panic versions allow normalization, and
        # only the single main-thread panic header's runtime thread identifier.
        require(expected["exit"] in (101, -signal.SIGABRT), "panic normalization requires the declared failed program outcome")
        header = rb"(?m)^thread 'main' \([0-9]+\) panicked at ([^\r\n]+:\d+:\d+):$"
        actual_stderr, actual_count = re.subn(header, rb"thread 'main' (TID) panicked at \1:", actual_stderr)
        expected_stderr, expected_count = re.subn(header, rb"thread 'main' (TID) panicked at \1:", expected_stderr)
        require(actual_count == expected_count == 1, "reviewed panic version is missing its one canonical panic header")
    require(actual_stderr == expected_stderr, "program stderr differs from Rust")


def compare_version(observed, expected_ops, allowed_signals=(), panic_thread_id=False):
    checks, builds = observed["checks"], observed["builds"]
    require(set(checks) == ({"cold", "warm"} if "check" in expected_ops else {"cold"}), "missing or unexpected check result")
    require(set(builds) == ({"cold", "warm"} if "run" in expected_ops else {"cold"}), "missing or unexpected build result")
    oracle = observed["oracle_build"]
    completed(oracle)
    require(oracle.get("tool") in ("rustc", "cargo"), "Rust build oracle tool identity is missing")
    for result in list(checks.values()) + [entry["compile"] for entry in builds.values()]:
        completed(result)
    if oracle["exit"] != 0:
        require(oracle["exit"] == (1 if oracle["tool"] == "rustc" else 101), "Rust build oracle failed with an unexpected status")
        semantic = observed.get("oracle_semantic_check")
        require(semantic is not None, "failed Rust build has no separate semantic-only oracle check")
        completed(semantic)
        require(semantic.get("source_rejection") is True, "Rust oracle failed without a structured semantic source rejection")
        for result in list(checks.values()) + [entry["compile"] for entry in builds.values()]:
            require(result["exit"] == 1 and re.search(rb"error\[L\d+\]", result["stderr"]), "invalid Rust was not rejected by a native diagnostic")
        require(all("runtime" not in entry and not entry["artifact_exists"] for entry in builds.values()), "rejected build produced or executed an artifact")
        require("oracle_runtime" not in observed, "rejected Rust oracle executed an artifact")
        return "rejected"
    require("oracle_runtime" in observed, "accepted Rust oracle has no runtime result")
    for result in checks.values():
        require(result["exit"] == 0, "candidate check rejected Rust accepted in the pinned supported history")
    for entry in builds.values():
        require(entry["compile"]["exit"] == 0, "candidate build rejected Rust accepted in the pinned supported history")
        require(entry["artifact_exists"] and entry.get("artifact_sha256") and "runtime" in entry, "accepted build has no bound executable result")
        compare_runtime(entry["runtime"], observed["oracle_runtime"], allowed_signals, panic_thread_id)
    return "executed"


def stop(process):
    if process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    process.wait(timeout=5)


class Observer:
    def __init__(self, config, name):
        self.config, self.name = config, name
        self.output = Path(config["output"]) / name
        self.output.mkdir()
        self.report = {"scenario": name, "passed": False, "commands": [], "versions": []}
        self.pending = {"checks": {}, "builds": {}}
        self.daemons = []
        self.daemon_logs = []
        self.oracle_executable = None
        self.source = Path(config["source"])
        self.fixture = self.source / "compiler/native/tests/daemon" / name
        self.versions = sorted(p.name for p in self.fixture.iterdir() if p.name.startswith("v") and (p.is_dir() or p.suffix == ".rs"))
        require(bool(self.versions), "empty scenario history")
        self.save()

    def save(self):
        (self.output / "report.json").write_text(json.dumps(self.report, indent=2) + "\n")

    def execute(self, argv, cwd, env, timeout=60, public=False, runtime=False):
        argv = list(map(str, argv))
        label = f"{len(self.report['commands']):04d}"
        outpath, errpath = self.output / (label + ".stdout"), self.output / (label + ".stderr")
        trace = self.output / (label + ".trace.jsonl")
        env = dict(env, RUSTC=self.config["rustc"], RUST_BACKTRACE="0")
        if public:
            env["LUSTR_TRACE_PATH"] = str(trace)
        retained = public and not env.get("LUST_NO_DAEMON")
        if retained:
            require(len(self.daemons) == 1 and self.daemons[0].poll() is None, "original retained daemon is not alive before the request")
        row = {"argv": argv, "exit": None, "timed_out": False, "stdout": outpath.name, "stderr": errpath.name}
        self.report["commands"].append(row)
        self.save()
        with outpath.open("wb") as out, errpath.open("wb") as err:
            process = subprocess.Popen(argv, cwd=cwd, env=env, stdout=out, stderr=err, start_new_session=True)
            try:
                process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                row["timed_out"] = True
                stop(process)
        row.update(exit=process.returncode, stdout_sha256=digest(outpath), stderr_sha256=digest(errpath))
        self.save()
        result = dict(row, stdout=outpath.read_bytes(), stderr=errpath.read_bytes())
        # Program signals are judged against the version's explicit semantic
        # expectation, never as an allowed compiler failure.
        completed(result, tuple(signal.valid_signals()) if runtime else ())
        if retained:
            require(self.daemons[0].poll() is None, "original retained daemon exited during the request")
        if public:
            events = [json.loads(line) for line in trace.read_text().splitlines()]
            require(len(events) == 1, "public compiler trace missing or duplicated")
            expected_engine = "lust" if row["exit"] == 0 else "lust-failed"
            expected_entries = ("native-library", "upstream-process") if env.get("LUST_NO_DAEMON") else ("daemon",)
            require(events[0].get("engine") == expected_engine and events[0].get("entry") in expected_entries, "public request used an unexpected execution owner")
            row["trace"] = events
            self.save()
        return result

    def finish_version(self):
        index = len(self.report["versions"])
        require(index < len(self.versions), "extra version result")
        version = self.versions[index]
        opsfile = self.fixture / (version.removesuffix(".rs") + ".ops")
        ops = opsfile.read_text().split() if opsfile.is_file() else ["check", "run"]
        require(ops and len(ops) == len(set(ops)) and set(ops) <= {"check", "run"}, "unknown version operations")
        signal_names = self.config.get("runtime_signals", {}).get(self.name, {}).get(version, [])
        panic_thread_id = version in self.config.get("panic_thread_id_versions", {}).get(self.name, [])
        verdict = compare_version(self.pending, ops, tuple(getattr(signal, name) for name in signal_names), panic_thread_id)
        self.report["versions"].append({"version": version, "verdict": verdict, "operations": ops, "panic_thread_id_normalized": panic_thread_id, "allowed_program_signals": signal_names})
        self.pending = {"checks": {}, "builds": {}}
        self.oracle_executable = None
        self.save()

    def run(self, argv, cwd, env, timeout=60):
        argv = list(map(str, argv))
        legacy_stderr = None
        public = argv[0] == self.config["entry"]
        if public:
            mode = "cold" if env.get("LUST_NO_DAEMON") else "warm"
            if argv[1] == "check":
                result = self.execute(argv, cwd, env, timeout, public=True)
                require(mode not in self.pending["checks"], "duplicated check result")
                self.pending["checks"][mode] = result
            else:
                require(argv[1] == "run" and "-o" in argv, "unrecognized donor request")
                target = Path(argv[argv.index("-o") + 1])
                target.unlink(missing_ok=True)
                build = self.execute([argv[0], "build", *argv[2:]], cwd, env, timeout, public=True)
                entry = {"compile": build, "artifact_exists": target.is_file()}
                require(mode not in self.pending["builds"], "duplicated build result")
                self.pending["builds"][mode] = entry
                result = build
                if build["exit"] == 0:
                    require(target.is_file(), "successful compiler did not produce its requested artifact")
                    entry["artifact_sha256"] = digest(target)
                    entry["runtime"] = self.execute([target], cwd, env, 10, runtime=True)
                    require(digest(target) == entry["artifact_sha256"], "executable changed while running")
                    result = entry["runtime"]
                    legacy_stderr = build["stderr"] + result["stderr"]
        elif argv[0] == "rustc":
            self.oracle_executable = Path(argv[argv.index("-o") + 1])
            self.oracle_executable.unlink(missing_ok=True)
            result = self.execute([self.config["rustc"], *argv[1:]], cwd, env, timeout)
            self.pending["oracle_build"] = dict(result, tool="rustc")
            if result["exit"] != 0:
                require(result["exit"] == 1, "direct Rust compiler failed outside source rejection")
                semantic_argv = [self.config["rustc"], *argv[1:]]
                semantic_argv[semantic_argv.index("-o") + 1] = str(self.output / "oracle-check.rmeta")
                semantic = self.execute([*semantic_argv, "--emit=metadata", "--error-format=json"], cwd, env, timeout)
                semantic["source_rejection"] = source_rejection(semantic, cwd)
                self.pending["oracle_semantic_check"] = semantic
                self.finish_version()
        elif argv[0] == "cargo" and argv[1] == "build":
            result = self.execute([self.config["cargo"], *argv[1:], "--message-format=json"], cwd, env, timeout)
            self.pending["oracle_build"] = dict(result, tool="cargo")
            if result["exit"] == 0:
                artifacts = [json.loads(line) for line in result["stdout"].splitlines() if line.strip()]
                executable = [row["executable"] for row in artifacts if row.get("reason") == "compiler-artifact" and row.get("executable") and "bin" in row.get("target", {}).get("kind", [])]
                require(len(executable) == 1, "Cargo oracle did not identify one selected executable")
                self.oracle_executable = Path(executable[0])
            else:
                require(result["exit"] == 101, "Cargo oracle failed with an unexpected status")
                semantic = self.execute([self.config["cargo"], "check", *argv[2:], "--message-format=json"], cwd, env, timeout)
                semantic["source_rejection"] = source_rejection(semantic, cwd, cargo=True)
                self.pending["oracle_semantic_check"] = semantic
                self.finish_version()
        else:
            require(self.oracle_executable is not None and (Path(argv[0]) == self.oracle_executable or argv[:2] == ["cargo", "run"]), "unexpected donor execution")
            require(self.oracle_executable.is_file(), "accepted Rust oracle has no executable")
            before = digest(self.oracle_executable)
            result = self.execute([self.oracle_executable], cwd, env, 10, runtime=True)
            require(before == digest(self.oracle_executable), "Rust oracle executable changed while running")
            self.pending["oracle_runtime"] = result
            self.finish_version()
        # Match the donor CLI's signal convention only for its old controls.
        # The independent observations retain the real signed process status.
        legacy_exit = 128 - result["exit"] if public and result["exit"] < 0 else result["exit"]
        return legacy_exit, result["stdout"].decode("utf-8", errors="replace"), (result["stderr"] if legacy_stderr is None else legacy_stderr).decode("utf-8", errors="replace")

    def daemon(self, argv, **kwargs):
        require(argv == [self.config["entry"], "daemon"], "unexpected scenario process")
        # The public compiler serves compile commands; the bound adjacent
        # native client owns daemon startup. Every compile still uses Lustr.
        actual = [str(Path(self.config["entry"]).with_name("lust")), "daemon"]
        # Keep a read handle while the donor removes its temporary directory.
        # This preserves startup failures and the complete native daemon log.
        self.daemon_logs.append(open(kwargs["stderr"].name, "rb"))
        process = subprocess.Popen(actual, **kwargs)
        self.daemons.append(process)
        self.report["daemon_argv"] = actual
        self.save()
        return process


def scenario_worker(payload):
    config, name = payload
    observer = Observer(config, name)
    source = Path(config["source"])
    scripts = source / "compiler/native/scripts"
    sys.path.insert(0, str(scripts))
    spec = importlib.util.spec_from_file_location("retained_donor_scenarios", scripts / "test-daemon-scenarios.py")
    module = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(module)
        module.LUST = config["entry"]
        module.RESTART = False
        module.run = observer.run
        module.subprocess = types.SimpleNamespace(Popen=observer.daemon)
        returned, problems, notes = module.scenario(str(observer.fixture))
        require(returned == name and not problems, "donor lifecycle/control failure: " + "; ".join(problems))
        require(len(observer.report["versions"]) == len(observer.versions), "missing version results")
        require(observer.pending == {"checks": {}, "builds": {}}, "unfinished version observations")
        observer.report.update(passed=True, donor_notes=notes)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        observer.report["failure"] = str(error)
    finally:
        for process in observer.daemons:
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                stop(process)
                observer.report.update(passed=False, cleanup_failure="daemon did not stop after the donor lifecycle")
        for index, stream in enumerate(observer.daemon_logs):
            with stream:
                (observer.output / f"daemon-{index}.stderr").write_bytes(stream.read())
        observer.save()
    return observer.report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT)
    parser.add_argument("--entry", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--cargo", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--scenario", action="append", default=[])
    parser.add_argument("--jobs", type=int, default=2)
    args = parser.parse_args(argv)
    require(args.jobs > 0, "jobs must be positive")
    args.source, args.output = args.source.resolve(), args.output.resolve()
    args.output.mkdir(parents=True, exist_ok=False)
    report = {"schema": "lustr-retained-results-v1", "passed": False, "status": "preparing", "scenarios": [], "full_retained_coverage": False, "full_rust_coverage": False, "release_admission": False}
    def save():
        (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    save()
    try:
        manifest = args.source / "upstream/retained-scenario-results.json"
        inventory = load_inventory(args.source, manifest)
        required = [row["name"] for row in inventory["scenarios"] if row["status"] == "required"]
        selected = args.scenario or required
        require(selected and len(selected) == len(set(selected)) and set(selected) <= set(required), "unknown, duplicated or unresolved requested scenario")
        entry, rustc, cargo = args.entry.absolute(), args.rustc.absolute(), args.cargo.absolute()
        files = [entry, entry.with_name("lust"), entry.with_name("lust-core"), rustc, cargo]
        files += sorted((entry.parent.parent / "lib").glob("liblustr_native_core.*"))
        require(len(files) >= 6, "native runtime library is missing")
        report.update(source_head=subprocess.check_output(["git", "-C", str(args.source), "rev-parse", "HEAD"], text=True).strip(), upstream_revision=inventory["upstream_revision"], inventory_sha256=digest(manifest), runner_sha256=digest(Path(__file__)), binary_sha256={str(path): digest(path) for path in files}, expected_scenarios=selected, unselected_required=sorted(set(required) - set(selected)), unresolved_scenarios=[row["name"] for row in inventory["scenarios"] if row["status"] == "unresolved"], excluded_controls=inventory["excluded_controls"], separate_integration_histories=inventory["integration_histories"], scope="Public check/build and separately executed artifacts in donor edit order. Original run-command, restart, unresolved histories and complete ownership require separate qualification. No performance claim.")
        version = subprocess.run([rustc, "-vV"], capture_output=True, check=True, timeout=30)
        (args.output / "rustc-version.stdout").write_bytes(version.stdout)
        declared = re.search(r'^source = "([0-9a-f]{40})"$', (args.source / "upstream/bluefe-core.pin").read_text(), re.M)
        require(declared is not None and f"commit-hash: {declared[1]}\n".encode() in version.stdout, "Rust oracle differs from the declared semantic SDK")
        report["rust_source"] = declared[1]
        for key in list(os.environ):
            if key.startswith(("LUST_", "LUSTR_")) or key in ("LD_LIBRARY_PATH", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
                os.environ.pop(key)
        os.environ.update(RUSTC=str(rustc), PATH=str(cargo.parent) + os.pathsep + os.environ.get("PATH", ""), RUST_BACKTRACE="0", LC_ALL="C")
        config = {"source": str(args.source), "output": str(args.output), "entry": str(entry), "rustc": str(rustc), "cargo": str(cargo), "runtime_signals": inventory["runtime_signals"], "panic_thread_id_versions": inventory["panic_thread_id_versions"]}
        report["status"] = "running"
        save()
        with ProcessPoolExecutor(max_workers=args.jobs) as pool:
            for row in pool.map(scenario_worker, [(config, name) for name in selected]):
                report["scenarios"].append({"name": row["scenario"], "passed": row["passed"], "versions": len(row["versions"]), "failure": row.get("failure"), "report": row["scenario"] + "/report.json"})
                save()
                print(json.dumps(report["scenarios"][-1]), flush=True)
        require(report["binary_sha256"] == {str(path): digest(path) for path in files}, "compiler identities changed during qualification")
        require(load_inventory(args.source, manifest) == inventory and digest(manifest) == report["inventory_sha256"], "scenario inventory changed during qualification")
        report["passed"] = len(report["scenarios"]) == len(selected) and all(row["passed"] for row in report["scenarios"])
        report["status"] = "passed" if report["passed"] else "failed"
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        report.update(status="setup-failed", failure=str(error))
    save()
    print(json.dumps({"status": report["status"], "scenarios": len(report["scenarios"]), "full_retained_coverage": False}), flush=True)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
