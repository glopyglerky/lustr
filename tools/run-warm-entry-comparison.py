#!/usr/bin/env python3
"""Compare exact before/after public entries with pristine Lust and Cargo.

The two public entries share one source-built native runtime and Rust SDK. The
baseline entry is rebuilt from its pinned source with the same build recipe.
This measures entry changes only; it is not a second complete toolchain build.
"""
import argparse
from contextlib import ExitStack
import csv
import importlib.util
import json
import os
from pathlib import Path
import platform
import random
import shutil
import statistics
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("public_slice", HERE / "run-upstream-slice-benchmark.py")
S = importlib.util.module_from_spec(spec)
spec.loader.exec_module(S)
require = S.require
BASELINE = "28a2842635ee994db492b43f8c956eddce57a803"
SCHEMA = "lustr-warm-entry-comparison-v1"
LANES = ("baseline", "entry", "pristine", "cargo")
PHASES = ("cold-check", "no-op-check", "cold-build", "no-op-build")
SEED = 0x5741524D


def git(source, *args):
    return subprocess.check_output(["git", "-C", str(source), *args], env={
        "PATH": "/usr/local/bin:/usr/bin:/bin", "LANG": "C", "LC_ALL": "C",
        "GIT_NO_REPLACE_OBJECTS": "1"}).decode().strip()


def source_identity(source):
    require(not git(source, "status", "--porcelain", "--untracked-files=no"), "tracked source changed")
    return {"revision": git(source, "rev-parse", "HEAD"), "tree": git(source, "rev-parse", "HEAD^{tree}")}


def build_baseline(args):
    require(not args.output.exists(), "baseline build output must be new")
    args.output.mkdir(mode=0o700, parents=True)
    receipt = {"schema": SCHEMA, "status": "failed", "scope": "Pinned baseline entry plus identical candidate-built native runtime and SDK; not a complete baseline package build."}
    try:
        baseline, candidate = source_identity(args.baseline_source), source_identity(args.repository)
        require(baseline["revision"] == BASELINE, "baseline revision differs")
        shared = {}
        for path in ("compiler/native", "crates/lustr-native-core", "library", "upstream/bluefe-core.pin",
                     "src/host_runtime.rs", "tools/native-tcc-input-observer.rs", "upstream/runtime"):
            before = git(args.baseline_source, "rev-parse", f"HEAD:{path}")
            after = git(args.repository, "rev-parse", f"HEAD:{path}")
            require(before == after, f"shared runtime source differs: {path}")
            shared[path] = before
        package = args.built / "toolchain"
        sdk = (package / "bin/rustc-reference").resolve().parent.parent
        command = [str(sdk / "bin/cargo"), "build", "--manifest-path", str(args.baseline_source / "Cargo.toml"),
                   "--release", "--locked", "--offline", "--jobs", "2", "--bin", "lustr", "--features", "source-native-runtime",
                   "--target-dir", str(args.output / "target"), "--config", f'build.build-dir="{args.output / "intermediates"}"']
        env = dict(os.environ, RUSTC=str(sdk / "bin/rustc"), LUSTR_CORE_ROOT=str(sdk), LUSTR_LUST_ROOT=str(package),
                   LUSTR_NATIVE_BUILD_OUTPUT=str(args.built / "NATIVE-BUILD-RECEIPT.txt"),
                   LUSTR_HOST_SDK_RECEIPT=str(args.built / "HOST-SDK-RECEIPT.txt"))
        env.pop("LD_LIBRARY_PATH", None)
        receipt.update(baseline=baseline, candidate=candidate, shared_source_objects=shared, command=command,
                       build_environment={key:env[key] for key in ("RUSTC", "LUSTR_CORE_ROOT", "LUSTR_LUST_ROOT", "LUSTR_NATIVE_BUILD_OUTPUT", "LUSTR_HOST_SDK_RECEIPT")})
        with (args.output / "build.stdout").open("xb") as out, (args.output / "build.stderr").open("xb") as err:
            result = subprocess.run(command, cwd=args.baseline_source, env=env, stdin=subprocess.DEVNULL, stdout=out, stderr=err)
        receipt["exit"] = result.returncode
        require(result.returncode == 0, "baseline entry build failed; original streams retained")
        target = package / "bin/lustr-warm-baseline"
        require(not target.exists(), "baseline entry already exists")
        shutil.copy2(args.output / "target/release/lustr", target)
        receipt["baseline_binary"] = {"path": str(target), "sha256": S.digest(target)}
        files = [package / "bin/lustr", package / "bin/lust", package / "bin/lust-core", package / "bin/rustc-full",
                 package / "bin/rustc-reference", package / "bin/cargo", package / "lib/liblustr_native_core.so",
                 package / "lib/lustr-selection-tools.txt",
                 args.built / "NATIVE-BUILD-RECEIPT.txt", args.built / "HOST-SDK-RECEIPT.txt", args.built / "COMPONENTS.txt"]
        receipt["shared_package_sha256"] = {str(p):S.digest(p) for p in files}
        receipt["build_streams"] = {name:S.digest(args.output / name) for name in ("build.stdout", "build.stderr")}
        receipt["status"] = "completed"
    except BaseException as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        S.write_json(args.output / "receipt.json", receipt)
    print(f"Built baseline entry: {target}")


def build_selector_gate(args):
    require(not args.output.exists(), "selector build output must be new")
    args.output.mkdir(mode=0o700, parents=True)
    source = args.repository / "tools/profile-native-preflight.rs"
    package = args.built / "toolchain"
    sdk = (package / "bin/rustc-reference").resolve().parent.parent
    native_receipt = (args.built / "NATIVE-BUILD-RECEIPT.txt").read_text()
    build_hash, = [line.split("=", 2)[2] for line in native_receipt.splitlines() if line.startswith("cargo:rustc-env=LUST_BUILD_HASH=")]
    require(len(build_hash) == 16 and all(char in "0123456789abcdef" for char in build_hash), "native build identity differs")
    manifest = args.output / "Cargo.toml"
    manifest.write_text('[package]\nname="lustr-selection-gate"\nversion="0.0.0"\nedition="2024"\n'
                        '[workspace]\n[dependencies]\nserde_json="=1.0.154"\nsha2="=0.11.1"\n'
                        '[[bin]]\nname="profile-native-preflight"\npath=' + json.dumps(str(source)) + '\n'
                        '[profile.release]\nopt-level=3\n')
    env = dict(os.environ, RUSTC=str(sdk / "bin/rustc"), LUST_BUILD_HASH=build_hash)
    env.pop("LD_LIBRARY_PATH", None)
    receipt = {"schema":SCHEMA, "status":"failed", "scope":"Production parser/selector/child owner component only; no compiler timing claim.",
               "source":source_identity(args.repository), "native_build_hash":build_hash, "commands":[],
               "source_sha256":{str(args.repository / path):S.digest(args.repository / path) for path in
                                ("tools/profile-native-preflight.rs", "src/full.rs", "src/selection_cache.rs", "src/owned_child.rs", "src/host_runtime.rs", "compiler/native/src/protocol.rs")}}
    commands = [[str(sdk / "bin/cargo"), "generate-lockfile", "--offline", "--manifest-path", str(manifest)],
                [str(sdk / "bin/cargo"), "build", "--release", "--locked", "--offline", "--manifest-path", str(manifest),
                 "--target-dir", str(args.output / "target"), "--config", f'build.build-dir="{args.output / "intermediates"}"']]
    try:
        for index, command in enumerate(commands):
            with (args.output / f"build-{index}.stdout").open("xb") as out, (args.output / f"build-{index}.stderr").open("xb") as err:
                result = subprocess.run(command, cwd=args.output, env=env, stdin=subprocess.DEVNULL, stdout=out, stderr=err)
            receipt["commands"].append({"argv":command, "exit":result.returncode})
            require(result.returncode == 0, "selector component build failed; original streams retained")
        binary = args.output / "target/release/profile-native-preflight"
        receipt.update(status="completed", binary_sha256=S.digest(binary), manifest_sha256=S.digest(manifest), lock_sha256=S.digest(args.output / "Cargo.lock"))
    except BaseException as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        S.write_json(args.output / "receipt.json", receipt)
    print(f"Built production selector gate: {binary}")


def orders(pairs):
    require(12 <= pairs <= 96 and pairs % 4 == 0, "pairs must be a multiple of four in 12..96")
    rng, schedule = random.Random(SEED), []
    while len(schedule) < pairs:
        lanes, rotations = list(LANES), list(range(4))
        rng.shuffle(lanes)
        rng.shuffle(rotations)
        schedule.extend(lanes[n:] + lanes[:n] for n in rotations)
    return schedule


def summaries(samples, pairs):
    require(len(samples) == pairs * len(LANES) * len(PHASES), "missing or extra samples")
    values = {}
    for row in samples:
        key = (row["sample"], row["lane"], row["phase"])
        require(key not in values and type(key[0]) is int and 0 <= key[0] < pairs and key[1] in LANES and key[2] in PHASES
                and S.STATS.positive(row["seconds"]), "invalid or duplicate sample")
        values[key] = row["seconds"]
    result = []
    for phase in PHASES:
        for baseline, candidate in (("baseline", "entry"), ("pristine", "baseline"), ("pristine", "entry"), ("cargo", "entry")):
            paired = [(values[i, baseline, phase], values[i, candidate, phase]) for i in range(pairs)]
            result.append({"phase":phase, "baseline":baseline, "candidate":candidate,
                           "analysis":S.STATS.summarize(paired, 10000, SEED)})
    return result


def parity(comparisons):
    result = {}
    for phase in ("no-op-check", "no-op-build"):
        row, = [x for x in comparisons if x["phase"] == phase and x["baseline"] == "pristine" and x["candidate"] == "entry"]
        a = row["analysis"]
        # Exact non-regression target. An interval overlapping one is inconclusive,
        # not evidence of equivalence. Keep the measured ratio even when it passes.
        result[phase] = {"meets_target":a["bootstrap_95"][0] >= 1.0,
                         "slowdown_median":a["candidate_median"] / a["baseline_median"],
                         "speed_ratio":a["geometric_ratio"], "speed_ratio_bootstrap_95":a["bootstrap_95"]}
    return {"criterion":"Both warm modes: lower conditional paired-bootstrap 95% speed ratio >= 1.0; no equivalence claim from overlapping intervals.",
            "meets_target":all(row["meets_target"] for row in result.values()), "modes":result}


def priming_summary(records, pairs):
    rows = [row for row in records if row["kind"] == "excluded-prime"]
    require(len(rows) == pairs * len(LANES) * 2 * 3, "missing or extra priming commands")
    values = {}
    for row in rows:
        key = (row["sample"], row["lane"], row["operation"], row["ordinal"])
        require(key not in values and 0 <= key[0] < pairs and key[1] in LANES and key[2] in ("check","build")
                and key[3] in (1,2,3) and S.STATS.positive(row["seconds"]), "invalid priming command")
        values[key] = row["seconds"]
    return [{"lane":lane, "operation":operation, "ordinal":ordinal,
             "median_seconds":statistics.median(values[sample,lane,operation,ordinal] for sample in range(pairs))}
            for operation in ("check","build") for lane in LANES for ordinal in (1,2,3)]


class Campaign(S.Campaign):
    def __init__(self, args):
        super().__init__(args)
        self.tools["baseline"] = args.built / "toolchain/bin/lustr-warm-baseline"
        self.report.update(schema=SCHEMA, phases=list(PHASES), lanes=list(LANES), seed=SEED,
                           scope="Identical public slice with pinned baseline and candidate public entries, pristine Lust and Rust/Cargo.",
                           limits="Entry-only before/after comparison with identical native runtime and SDK. Broader compiler qualification remains separate.")

    def compile(self, lane, operation, state, env, phase, fresh=False, trace=False):
        if lane != "baseline" or not trace:
            return super().compile(lane, operation, state, env, phase, fresh, trace)
        local = dict(env)
        trace_path = state / f"entry-{len(self.recorder.records)}.trace"
        local["LUSTR_TRACE_PATH"] = str(trace_path)
        result, _, _ = self.recorder.run(self.command(lane, operation, state), local, self.fixture, lane, phase)
        rows = [json.loads(line) for line in trace_path.read_bytes().splitlines()]
        expected = "upstream-process" if "LUST_NO_DAEMON" in env else "daemon"
        require(rows == [{"engine":"lust", "entry":expected}], "baseline public entry did not use the required native route")
        archived = self.output / "commands" / f"{result['id']:05}.entry.trace"
        shutil.copyfile(trace_path, archived)
        self.recorder.event({"kind":"entry-route", "command":result["id"], "phase":phase, "records":rows,
                             "trace":{"file":str(archived.relative_to(self.output)), "sha256":S.digest(archived), "bytes":archived.stat().st_size}})
        binary = state / "program" if operation == "build" else None
        if binary is not None:
            require(binary.is_file() and not binary.is_symlink(), "baseline build did not produce an ordinary executable")
            self.built_artifacts[binary] = {"build_command":result["id"], "binary_sha256":S.digest(binary)}
            self.recorder.event({"kind":"build-identity", "binary":str(binary), **self.built_artifacts[binary]})
        return result, binary

    def daemon_tool(self, lane):
        return self.tools["pristine"] if lane == "pristine" else self.tools["integrated"]

    def qualification(self, sockets, stage):
        self.recorder.event({"kind":"qualification-start", "stage":stage})
        for lane in ("cargo", "pristine", "baseline", "entry"):
            state = self.output / "work" / stage / "runtime" / lane
            env = self.environment(state, sockets / (lane + ".sock"), True)
            _, binary = self.compile(lane, "build", state, env, "gate-build", trace=True)
            self.runtime(lane, binary, env, "gate-runtime")
        for lane in ("pristine", "baseline", "entry"):
            for operation in ("check", "build"):
                state = self.output / "work" / stage / ("retained-" + operation) / lane
                env = dict(self.environment(state, sockets / (lane + ".sock")), LUST_STATS="1")
                selection_trace = state / "selection-events.jsonl"
                if lane == "entry":
                    env["LUSTR_SELECTION_DIAGNOSTICS"] = str(selection_trace)
                with S.Daemon(self.recorder, self.daemon_tool(lane), env, self.fixture, lane, f"{stage}-{operation}-{lane}") as daemon:
                    self.compile(lane, operation, state, env, "gate-warm-prime", trace=True)
                    if lane == "entry":
                        for _ in range(3):
                            self.compile(lane, operation, state, env, "gate-selection-prime")
                        before = selection_trace.read_bytes()
                        command, _ = self.compile(lane, operation, state, env, "gate-selection-hit")
                        after = selection_trace.read_bytes()
                        require(after.startswith(before), "selection diagnostics were replaced")
                        rows = [json.loads(line) for line in after[len(before):].splitlines()]
                        require(rows == [{"selection_cache":"hit"}], "warm candidate did not reuse exact target selection")
                        archive = self.output / "commands" / f"{command['id']:05}.selection.trace"
                        archive.write_bytes(after[len(before):])
                        self.recorder.event({"kind":"selection-cache", "command":command["id"], "operation":operation,
                                             "trace":{"file":str(archive.relative_to(self.output)), "sha256":S.digest(archive), "bytes":archive.stat().st_size}})
                    command, _, stderr = self.recorder.run(self.command(lane, operation, state) + ["--timings"],
                                                          env, self.fixture, lane, "gate-retained-" + operation)
                    route = S.retained_check(stderr)
                    self.recorder.event({"kind":"retained-route", "command":command["id"], "lane":lane, "operation":operation, "route":list(route)})
                    if operation == "build":
                        binary = state / "program"
                        self.built_artifacts[binary] = {"build_command":command["id"], "binary_sha256":S.digest(binary)}
                        self.recorder.event({"kind":"build-identity", "binary":str(binary), **self.built_artifacts[binary]})
                        self.runtime(lane, state / "program", env, "gate-retained-runtime")
                    daemon.alive()
        reject = self.output / "rejected.rs"
        for lane in LANES:
            state = self.output / "work" / stage / "rejection" / lane
            env = self.environment(state, sockets / (lane + ".sock"), True)
            command = ([self.reference, reject, "--edition=2024", "--emit=metadata", "-o", state / "reject.rmeta", *S.FLAGS]
                       if lane == "cargo" else [self.tools[lane], "check", reject, *S.FLAGS])
            _, out, err = self.recorder.run(command, env, self.fixture, lane, "gate-reject", 1)
            require(b"E0308" in out + err or b"L0110" in out + err, "wrong rejection diagnostic")
        self.recorder.event({"kind":"qualification", "stage":stage, "completed":True})

    def measure_group(self, sample, order, operation, sockets):
        self.recorder.event({"kind":"group-start", "sample":sample, "operation":operation, "order":order})
        with ExitStack() as stack:
            daemons, states, environments = {}, {}, {}
            for lane in order:
                state = self.output / "work" / f"{sample:03}-{operation}" / lane
                require(not state.exists(), "cold state was reused")
                env = self.environment(state, sockets / (lane + ".sock"), True)
                states[lane] = state
                result, binary = self.compile(lane, operation, state, env, "cold-" + operation)
                self.timing(sample, lane, "cold-" + operation, [result])
                if binary is not None:
                    self.runtime(lane, binary, env, "cold-build-runtime")
            for lane in order:
                env = self.environment(states[lane], sockets / (lane + ".sock"))
                environments[lane] = env
                if lane != "cargo":
                    daemons[lane] = stack.enter_context(S.Daemon(self.recorder, self.daemon_tool(lane), env, self.fixture, lane, f"{sample}-{operation}-{lane}"))
                for ordinal in range(1,4):
                    result, _ = self.compile(lane, operation, states[lane], env, "excluded-prime", fresh=lane == "cargo")
                    self.recorder.event({"kind":"excluded-prime", "sample":sample, "lane":lane, "operation":operation,
                                         "ordinal":ordinal, "seconds":result["seconds"], "command":result["id"]})
            for lane in order:
                if lane in daemons:
                    daemons[lane].alive()
                result, binary = self.compile(lane, operation, states[lane], environments[lane], "no-op-" + operation, fresh=lane == "cargo")
                self.timing(sample, lane, "no-op-" + operation, [result])
                if binary is not None:
                    self.runtime(lane, binary, environments[lane], "no-op-build-runtime")
                if lane in daemons:
                    daemons[lane].alive()
        self.recorder.event({"kind":"group-end", "sample":sample, "operation":operation})

    def execute(self):
        require(platform.system() == "Linux" and platform.machine() == "x86_64", "Linux x86-64 campaign required")
        require("Cargo artifact profile: release\n" in (self.args.built / "BUILD-PROFILE.txt").read_text(), "release candidate required")
        require(b"Finished `release` profile" in self.args.pristine_build_log.read_bytes(), "release pristine build required")
        require(source_identity(self.args.pristine_source) == {"revision":S.PIN, "tree":S.TREE}, "pristine source differs")
        self.report["candidate_identity"] = source_identity(self.args.repository)
        before = json.loads(self.args.baseline_receipt.read_text())
        require(before["status"] == "completed" and before["baseline"]["revision"] == BASELINE, "baseline build not completed")
        require(before["candidate"] == self.report["candidate_identity"], "baseline composition uses another candidate")
        require(before["baseline_binary"]["sha256"] == S.digest(self.tools["baseline"]), "baseline binary differs")
        require(all(S.digest(path) == value for path, value in before["shared_package_sha256"].items()), "shared package differs")
        self.report["baseline_composition"] = before
        self.report["host"] = S.HOST.collect()
        self.report["fixture_preparation"] = S.prepare_fixture(self.args.repository, self.args.pristine_source, self.fixture)
        self.report["tool_paths"] = {key:str(value) for key,value in self.tools.items()}
        files = list(self.tools.values()) + [Path(path) for path in before["shared_package_sha256"]] + [self.reference, self.args.pristine_lust.with_name("lust-core"),
                 self.args.baseline_receipt, self.args.pristine_build_log, Path(__file__), HERE / "run-upstream-slice-benchmark.py",
                 HERE / "report-mixed-performance.py", HERE / "capture-benchmark-host.py"]
        identities = {str(path):S.digest(path) for path in files}
        self.report["compiler_and_runner_sha256"] = identities
        receipts = self.output / "build-receipts"
        receipts.mkdir()
        self.report["build_receipts"] = {}
        for path in (self.args.baseline_receipt, self.args.pristine_build_log, self.args.built / "COMPONENTS.txt",
                     self.args.built / "NATIVE-BUILD-RECEIPT.txt", self.args.built / "BUILD-PROFILE.txt",
                     self.args.built / "toolchain/lib/lustr-selection-tools.txt"):
            shutil.copyfile(path, receipts / path.name)
            self.report["build_receipts"][path.name] = {"file":str(Path("build-receipts") / path.name), "sha256":S.digest(path)}
        with tempfile.TemporaryDirectory(prefix="lustr-warm-") as temporary:
            sockets = Path(temporary)
            env = self.environment(self.output / "work/lockfile", sockets / "unused.sock", True)
            _, out, _ = self.recorder.run([self.reference, "-vV"], env, self.output, "cargo", "rust-identity")
            require(f"commit-hash: {S.RUST}" in out.decode().splitlines(), "Rust pin differs")
            self.report["rustc_version"] = out.decode()
            self.recorder.run([self.tools["cargo"], "generate-lockfile", "--offline", "--manifest-path", self.fixture / "Cargo.toml"],
                              env, self.fixture, "cargo", "lockfile")
            source = S.inventory(self.fixture)
            self.report["fixture_files"] = source
            reject = self.output / "rejected.rs"
            reject.write_text(S.REJECTION)
            self.report["rejection_input"] = {"file":"rejected.rs", "sha256":S.digest(reject)}
            self.recorder.event({"kind":"identities", **{key:self.report[key] for key in
                                 ("tool_paths", "compiler_and_runner_sha256", "fixture_files", "candidate_identity", "baseline_composition", "build_receipts", "rustc_version")}})
            self.qualification(sockets, "initial")
            self.report["orders"] = orders(self.args.pairs)
            for sample, order in enumerate(self.report["orders"]):
                for operation in ("check", "build"):
                    self.measure_group(sample, order, operation, sockets)
                    require(S.inventory(self.fixture) == source, "fixture changed during measurement")
            self.qualification(sockets, "final")
            require(S.inventory(self.fixture) == source and reject.read_text() == S.REJECTION, "input changed")
        require(all(S.digest(path) == value for path,value in identities.items()), "compiler/runner changed")
        require(source_identity(self.args.repository) == self.report["candidate_identity"]
                and source_identity(self.args.pristine_source) == {"revision":S.PIN, "tree":S.TREE}, "source changed during comparison")
        self.report.update(samples=self.samples, summaries=summaries(self.samples, self.args.pairs), status="completed")
        self.report["warm_target"] = parity(self.report["summaries"])
        self.report["priming_summary"] = priming_summary(self.recorder.records, self.args.pairs)


def render(report):
    lines = ["# Warm public-entry comparison", "", f"Status: {report['status']}.", "", report["scope"], "", report["limits"], ""]
    if report["status"] != "completed":
        return "\n".join(lines + ["No completed performance result. All original streams and failure records remain retained.", ""])
    lines += ["All times are measured end-to-end command medians in milliseconds.", "",
              "| Operation | Baseline Lustr | Candidate Lustr | Pristine Lust | rustc via Cargo |", "| --- | ---: | ---: | ---: | ---: |"]
    for phase in PHASES:
        values = [statistics.median(row["seconds"] for row in report["samples"] if row["lane"] == lane and row["phase"] == phase) * 1000 for lane in LANES]
        lines.append(f"| {phase} | " + " | ".join(f"{value:.3f}" for value in values) + " |")
    lines += ["", "Each lane then primes its new warm state with three complete commands before the measured no-op. These costs remain visible below, in milliseconds. The first native prime can include daemon initialization and selection-cache population; this table does not isolate either component.", "",
              "| Operation | Lane | First prime | Second prime | Third prime |", "| --- | --- | ---: | ---: | ---: |"]
    for operation in ("check","build"):
        for lane in LANES:
            values = [row["median_seconds"] * 1000 for row in report["priming_summary"] if row["operation"] == operation and row["lane"] == lane]
            lines.append(f"| {operation} | {lane} | " + " | ".join(f"{value:.3f}" for value in values) + " |")
    lines += ["", "Ratios are paired reference/candidate geometric means; values above one favor the candidate.", "",
              "| Operation | Reference | Candidate | Ratio | Conditional bootstrap 95% interval |", "| --- | --- | --- | ---: | --- |"]
    for row in report["summaries"]:
        a = row["analysis"]
        lines.append(f"| {row['phase']} | {row['baseline']} | {row['candidate']} | {a['geometric_ratio']:.4f} | [{a['bootstrap_95'][0]:.4f}, {a['bootstrap_95'][1]:.4f}] |")
    lines += ["", f"Warm target met: {report['warm_target']['meets_target']}.", "", report["warm_target"]["criterion"], "",
              "Intervals are conditional exploratory estimates on this runner. Overlap with one does not prove equality. Full compiler correctness and release admission are separate gates.", ""]
    return "\n".join(lines)


def verify(output):
    report = json.loads((output / "report.json").read_text())
    require(report.get("schema") == SCHEMA and report.get("status") == "completed", "only complete campaigns have results")
    require(S.digest(output / "raw.jsonl") == report["raw_sha256"], "raw log changed")
    records = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
    def stream(record):
        p = Path(record["file"])
        require(not p.is_absolute() and ".." not in p.parts and not (output / p).is_symlink(), "invalid evidence path")
        data = (output / p).read_bytes()
        require(S.digest(output / p) == record["sha256"] and ("bytes" not in record or len(data) == record["bytes"]), "stream changed")
        return data
    identity, = [row for row in records if row["kind"] == "identities"]
    require(identity == {"kind":"identities", **{key:report[key] for key in
                         ("tool_paths", "compiler_and_runner_sha256", "fixture_files", "candidate_identity", "baseline_composition", "build_receipts", "rustc_version")}}, "source/build identity record differs")
    require(f"commit-hash: {S.RUST}" in report["rustc_version"].splitlines(), "Rust pin differs")
    for receipt in report["build_receipts"].values(): stream(receipt)
    before = json.loads(stream(report["build_receipts"]["receipt.json"]))
    require(before == report["baseline_composition"] and before["status"] == "completed" and before["baseline"]["revision"] == BASELINE
            and before["candidate"] == report["candidate_identity"], "baseline composition differs")
    require(before["baseline_binary"]["path"] == report["tool_paths"]["baseline"] and before["baseline_binary"]["sha256"] ==
            report["compiler_and_runner_sha256"][report["tool_paths"]["baseline"]], "baseline binary binding differs")
    selection_receipt = str(Path(report["tool_paths"]["entry"]).parent.parent / "lib/lustr-selection-tools.txt")
    require(report["build_receipts"]["lustr-selection-tools.txt"]["sha256"] == before["shared_package_sha256"][selection_receipt],
            "selection-toolchain receipt bytes differ from the executed package")
    commands, retained, active, built, runtime_commands, selection = {}, [], {}, {}, set(), []
    group = None
    oracle = (output / "runtime-oracle.stdout").read_bytes()
    require(oracle and S.digest(output / "runtime-oracle.stdout") == report["runtime_oracle_sha256"], "runtime oracle changed")
    for index, row in enumerate(records):
        if row["kind"] == "command":
            require(row["id"] == index and row["id"] not in commands and row["exit"] == row["expected_exit"]
                    and row["signal"] is None and not row["timed_out"] and not row["output_limit"] and not row.get("launch_error"), "failed or unbound command")
            stream(row["stdout"]); stream(row["stderr"])
            commands[row["id"]] = row
            if row["phase"].endswith("runtime"):
                require(stream(row["stdout"]) == oracle and stream(row["stderr"]) == b"", "runtime differs from Rust")
            if row["phase"] == "gate-reject":
                require(row["exit"] == 1 and (b"E0308" in stream(row["stdout"]) + stream(row["stderr"])
                        or b"L0110" in stream(row["stdout"]) + stream(row["stderr"])), "rejection differs")
        if row["kind"] == "build-identity":
            require(row["build_command"] in commands and commands[row["build_command"]]["exit"] == 0, "build identity lacks successful command")
            built[row["binary"]] = {key:row[key] for key in ("build_command", "binary_sha256")}
        if row["kind"] == "runtime-identity":
            require(row["binary"] in built and built[row["binary"]] == {key:row[key] for key in ("build_command", "binary_sha256")}, "runtime used another artifact")
            require(commands[row["command"]]["argv"] == [row["binary"]], "runtime command differs")
            require(row["command"] not in runtime_commands, "duplicate runtime identity")
            runtime_commands.add(row["command"])
        if row["kind"] == "group-start":
            require(group is None, "measurement groups overlap")
            group = row
        if row["kind"] == "group-end":
            require(group is not None and row["sample"] == group["sample"] and row["operation"] == group["operation"], "measurement group end differs")
            group = None
        if row["kind"] == "daemon-start":
            require(row["lane"] not in active, "daemon lane already active")
            active[row["lane"]] = row
        if row["kind"] == "daemon-stop":
            require(row["lane"] in active and active[row["lane"]]["pid"] == row["pid"]
                    and active[row["lane"]]["scope"] == row["scope"], "daemon owner differs")
            del active[row["lane"]]
            require(row["exit"] == 0 and row["error"] is None, "daemon failed")
            for item in row["streams"]: stream(item)
        if row["kind"] == "retained-route":
            command = commands[row["command"]]
            require(command["phase"] == "gate-retained-" + row["operation"], "wrong retained proof")
            require(list(S.retained_check(stream(command["stderr"]))) == row["route"], "retained proof differs")
            require(row["lane"] in active and active[row["lane"]]["environment"].get("LUST_STATS") == "1", "retained daemon not instrumented")
            retained.append((row["lane"], row["operation"]))
        if row["kind"] == "selection-cache":
            command = commands[row["command"]]
            require(command["lane"] == "entry" and command["phase"] == "gate-selection-hit"
                    and command["argv"][0] == report["tool_paths"]["entry"], "selection-cache proof uses another command")
            require([json.loads(line) for line in stream(row["trace"]).splitlines()] == [{"selection_cache":"hit"}], "selection-cache hit missing")
            selection.append(row["operation"])
        if row["kind"] == "entry-route":
            trace = [json.loads(x) for x in stream(row["trace"]).splitlines()]
            command = commands[row["command"]]
            route = "upstream-process" if "LUST_NO_DAEMON" in command["environment"] else "daemon"
            require(trace == row["records"] == [{"engine":"lust", "entry":route}], "native route differs")
        if row["kind"] == "timing" and row["phase"].startswith("no-op-") and row["lane"] != "cargo":
            require(row["lane"] in active, "warm timing had no owned daemon")
        if row["kind"] == "timing":
            require(group is not None and row["sample"] == group["sample"] and row["phase"].endswith("-" + group["operation"]), "timing escaped its group")
        if row["kind"] == "excluded-prime":
            require(group is not None and row["sample"] == group["sample"] and row["operation"] == group["operation"], "prime escaped its group")
            command = commands[row["command"]]
            require(command["phase"] == "excluded-prime" and command["lane"] == row["lane"] and command["seconds"] == row["seconds"], "prime timing differs")
    require(not active, "daemon still active")
    require(group is None, "measurement group did not finish")
    require(runtime_commands == {row["id"] for row in commands.values() if row["phase"].endswith("runtime")}, "runtime identity missing")
    require(retained == [(lane,op) for _ in range(2) for lane in ("pristine", "baseline", "entry") for op in ("check", "build")], "missing retained check/build proof")
    require(selection == ["check","build","check","build"], "missing initial/final selection reuse proof")
    samples = [dict((k,v) for k,v in row.items() if k != "kind") for row in records if row["kind"] == "timing"]
    require(samples == report["samples"], "sample log differs")
    for sample in samples:
        require(len(sample["commands"]) == 1, "timing must cover one complete public command")
        command = commands[sample["commands"][0]]
        require(command["lane"] == sample["lane"] and command["phase"] == sample["phase"] and command["seconds"] == sample["seconds"], "timing differs")
        require(command["argv"][0] == report["tool_paths"][sample["lane"]], "wrong timed entry")
        require(all(key not in command["environment"] for key in ("LUST_STATS","LUSTR_TRACE_PATH","LUSTR_SELECTION_DIAGNOSTICS")), "timed command is instrumented")
        env = command["environment"]
        cold = sample["phase"].startswith("cold-")
        require((env.get("LUST_NO_DAEMON") == "1" and env.get("LUST_NO_CACHE") == "1") if cold else
                "LUST_NO_DAEMON" not in env and "LUST_NO_CACHE" not in env, "cold/warm controls differ")
        operation = sample["phase"].split("-", 2)[-1]
        state = Path(env["CARGO_TARGET_DIR"]).parent
        fixture = Path(command["cwd"])
        require(state.parts[-2:] == (f"{sample['sample']:03}-{operation}", sample["lane"]), "lane state is shared")
        require(env["LUST_CACHE_DIR"] == str(state / "native-cache") and env["CARGO_HOME"] == str(state / "cargo-home"), "cache directory differs")
        if sample["lane"] == "cargo":
            expected = [report["tool_paths"]["cargo"], operation, "--offline", "--locked", "--manifest-path", str(fixture / "Cargo.toml"),
                        "--bin", "slice", "--target", S.TARGET, "--message-format=json-render-diagnostics"]
            S.cargo_artifact(stream(command["stdout"]), operation, state / "target", not cold)
        else:
            expected = [report["tool_paths"][sample["lane"]], operation, str(fixture / "src/slice_main.rs"), "--manifest-path", str(fixture / "Cargo.toml"),
                        "--bin", "slice", "--offline", "--locked", *S.FLAGS]
            if operation == "build": expected += ["-o", str(state / "program")]
        require(command["argv"] == expected, "timed request changed")
    require([row for row in records if row["kind"] == "qualification"] == [
        {"kind":"qualification", "stage":stage, "completed":True} for stage in ("initial", "final")], "qualification is incomplete")
    require(report["orders"] == orders(report["pairs"]), "order changed")
    groups = [row for row in records if row["kind"] == "group-start"]
    require(groups == [{"kind":"group-start", "sample":i, "operation":op, "order":order}
                       for i,order in enumerate(report["orders"]) for op in ("check", "build")], "measured schedule changed")
    require([(row["sample"], row["phase"], row["lane"]) for row in samples] == [
        (i,phase + "-" + op,lane) for i,order in enumerate(report["orders"]) for op in ("check","build")
        for phase in ("cold","no-op") for lane in order], "actual timed order differs")
    actual = summaries(samples, report["pairs"])
    S.STATS.compare(actual, report["summaries"], "warm comparison statistics")
    S.STATS.compare(parity(actual), report["warm_target"], "warm target")
    S.STATS.compare(priming_summary(records,report["pairs"]), report["priming_summary"], "excluded priming costs")
    require((output / "README.md").read_text() == render(report), "table changed")
    require(S.inventory(output / "fixture") == report["fixture_files"], "fixture changed")
    return report


def measure(args):
    orders(args.pairs)
    require(args.output.is_absolute() and not args.output.exists(), "new absolute output required")
    args.output.mkdir(mode=0o700, parents=True)
    campaign = Campaign(args)
    failure = None
    try:
        campaign.execute()
    except BaseException as error:
        failure = error
        campaign.report.update(status="failed", error=f"{type(error).__name__}: {error}", summaries=None, samples=campaign.samples)
    finally:
        campaign.recorder.raw.close()
        campaign.report["raw_sha256"] = S.digest(args.output / "raw.jsonl")
        S.write_json(args.output / "report.json", campaign.report)
        (args.output / "README.md").write_text(render(campaign.report))
        with (args.output / "samples.csv").open("w", newline="") as stream:
            writer = csv.DictWriter(stream, fieldnames=("sample", "lane", "phase", "seconds", "commands"))
            writer.writeheader(); writer.writerows(campaign.samples)
    if failure:
        raise failure
    report = verify(args.output)
    print(render(report))
    return 0 if report["warm_target"]["meets_target"] else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    build = commands.add_parser("build-baseline")
    for name in ("repository", "baseline-source", "built", "output"):
        build.add_argument("--" + name, type=Path, required=True)
    selector = commands.add_parser("build-selector-gate")
    for name in ("repository", "built", "output"):
        selector.add_argument("--" + name, type=Path, required=True)
    run = commands.add_parser("measure")
    for name in ("repository", "built", "baseline-receipt", "pristine-source", "pristine-lust", "pristine-build-log", "output"):
        run.add_argument("--" + name, type=Path, required=True)
    run.add_argument("--pairs", type=int, default=24)
    run.add_argument("--timeout", type=int, default=180)
    replay = commands.add_parser("verify")
    replay.add_argument("output", type=Path)
    args = parser.parse_args()
    for key, value in vars(args).items():
        if isinstance(value, Path):
            setattr(args, key, value.resolve())
    if args.command == "build-baseline":
        build_baseline(args)
        return 0
    if args.command == "build-selector-gate":
        build_selector_gate(args)
        return 0
    if args.command == "verify":
        result = verify(args.output)
        print(f"Verified {len(result['samples'])} timings; warm target met: {result['warm_target']['meets_target']}")
        return 0
    require(1 <= args.timeout <= 1800, "timeout out of range")
    return measure(args)


if __name__ == "__main__":
    raise SystemExit(main())
