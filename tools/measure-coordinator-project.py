#!/usr/bin/env python3
"""Qualify the unchanged coordinator with the existing paired runner.

Private originals remain under private/; only the explicit allowlist projection
under public/ is suitable for publication. This script does not publish anything.
"""
import argparse
import hashlib
import importlib.util
import io
import json
import math
import os
from pathlib import Path
import re
import subprocess
import tarfile

SCENARIOS = ("clean-dependency-build", "primed-root-build", "primed-root-check", "no-op", "compile-plus-test")
LANES = ("matched-rust", "lustr-mixed")
SEED = 327849038930
PAIRS = 12
WARMUPS = 2
RESAMPLES = 10000
POLICY = "owned-lustr-mixed-v1"
ORDER = "randomized-cyclic-blocks-v1"
STREAM_KEYS = ("stdout", "stderr")
ANALYSIS_FIELDS = {"method", "pairs", "raw_ratios", "log_ratios", "geometric_ratio", "bootstrap_95",
                   "baseline_median", "candidate_median", "baseline_p95", "candidate_p95", "exact_sign_flip_p",
                   "bootstrap_resamples", "seed"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")


def command(argv, cwd=None):
    result = subprocess.run([str(arg) for arg in argv], cwd=cwd, check=True, capture_output=True)
    return result.stdout.decode("utf-8").strip()


def files_below(path, ancestors=()):
    """Keep logical paths, so rehashing observes changed library symlink targets."""
    actual = path.resolve(strict=True)
    require(actual not in ancestors, "cyclic toolchain library link")
    if actual.is_file():
        return [path]
    require(actual.is_dir(), "special toolchain runtime input")
    files = []
    for entry in sorted(path.iterdir()):
        files.extend(files_below(entry, (*ancestors, actual)))
    return files


def snapshot(repo, revision, destination):
    data = subprocess.run(["git", "-C", str(repo), "archive", "--format=tar", f"{revision}:crates/lustr"],
                          check=True, capture_output=True).stdout
    destination.mkdir()
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for entry in archive:
            relative = Path(entry.name)
            require(not relative.is_absolute() and ".." not in relative.parts and (entry.isdir() or entry.isfile()),
                    "project snapshot must contain only ordinary files/directories")
            target = destination / relative
            if entry.isdir():
                target.mkdir(exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with target.open("xb") as output:
                    output.write(archive.extractfile(entry).read())
                target.chmod(entry.mode & 0o777)


def make_spec(scenario, *, source, private, package, sdk, gate, external_inputs, revision, host, jobs):
    require(scenario in SCENARIOS, "unsupported project scenario")
    operation = "check" if scenario == "primed-root-check" else "test" if scenario == "compile-plus-test" else "build"
    tail = [operation, "--locked", "--offline", "-vv"]
    if operation == "test":
        tail += ["--", "--test-threads=1"]
    force = "src/lib.rs" if scenario in ("primed-root-build", "primed-root-check", "compile-plus-test") else None
    lanes = []
    for name, root in zip(LANES, (sdk, package)):
        candidate = name == "lustr-mixed"
        entry = package / "bin/lustr" if candidate else sdk / "bin/cargo"
        prefix = [str(entry)]
        if candidate:
            prefix += ["--bluefe-root", str(package), "--engine", "auto"]
        env = {
            "PATH": f"{sdk}/bin:/usr/bin:/bin", "HOME": str(private / "home"),
            "CARGO_HOME": str(private / "cargo-home"), "CARGO_BUILD_JOBS": str(jobs),
            "CARGO_NET_OFFLINE": "true", "CARGO_TERM_COLOR": "never",
            "CARGO": str(root / "bin/cargo"), "RUSTC": str(root / "bin/rustc"),
            "RUSTDOC": str(root / "bin/rustdoc"), "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
        }
        tools = []
        for path, version in [(root / "bin/cargo", ["--version", "--verbose"]),
                              (root / "bin/rustc", ["-vV"]), (root / "bin/rustdoc", ["--version"]),
                              (gate, ["--version"])]:
            tools.append({"path": str(path), "sha256": digest(path), "version_args": version})
        if candidate:
            tools.append({"path": str(entry), "sha256": digest(entry), "version_args": ["--lustr-version"]})
        cmd = lambda argv, exit=0: {"argv": argv, "expected_exit": exit}
        test_gate = [str(gate), "tests-via", str(entry), "--bluefe-root", str(package), "--engine", "auto"] if candidate else [str(gate), "tests"]
        lanes.append({
            "name": name, "env": env, "tools": tools,
            "command": cmd(prefix + tail), "evidence": cmd(prefix + tail),
            "selection": cmd([str(root / "bin/cargo"), "metadata", "--format-version", "1", "--locked", "--offline"]),
            "evidence_contains": ["--edition=2024", "-C debuginfo=2"],
            "normalize": [[str(private / scenario / "targets" / name), "<TARGET>"]],
            "gates": [
                {"name": "executed-tests", "kind": "tests", "compare": "stdout",
                 "command": cmd(test_gate + ["test", "--locked", "--offline", "--", "--test-threads=1"])},
                {"name": "invalid-borrow", "kind": "rejection", "compare": "stdout",
                 "command": cmd([str(gate), "reject", str(private / "reject.rs")], 1)},
            ],
        })
    return {
        "schema": 1, "resource_mode": "disabled", "source": str(source),
        "repository": "https://github.com/glopyglerky/lustr", "revision": revision,
        "package": "lustr", "root_crate": "cargo_lust", "profile": "test" if operation == "test" else "dev", "target": host, "features": [],
        "semantic_settings": {"opt-level": "0", "debug": "2", "debug-assertions": "true", "overflow-checks": "true",
                              "panic": "unwind", "incremental": "Cargo default; unchanged", "features": "Cargo defaults; unchanged"},
        "external_inputs": [str(path) for path in external_inputs],
        "scenario": scenario, "force_root": force, "snapshot_mtime_authorized": force is not None,
        "pairs": PAIRS, "warmup_pairs": WARMUPS, "seed": SEED, "bootstrap_resamples": RESAMPLES,
        "timeout_seconds": 900, "output_limit_bytes": 64 * 1024 * 1024, "lanes": lanes,
        "native_state": {"kind": POLICY, "toolchain": str(package),
                         "component_manifest": str(package.parent / "COMPONENTS.txt"),
                         "build_receipt": str(package.parent / "NATIVE-BUILD-RECEIPT.txt")},
    }


def verify_state(report, report_dir):
    state = report["native_state"]
    require(state["kind"] == POLICY and state["status"] == "passed" and state["error"] is None,
            "missing or failed owned-state qualification")
    receipts = state["receipts"]
    records = report["records"]
    prepared = []
    public = []
    epoch = 0
    live = True
    sockets = {}
    clears = []
    for index, row in enumerate(receipts):
        event = row["event"]
        if event == "prepared":
            serial = row["command"]
            require(serial == len(prepared) and serial < len(records), "missing/duplicate command state receipt")
            record = records[serial]
            lane = record["lane"]
            target = report_dir / "targets" / lane
            require(live and row["epoch"] == epoch and row["lane"] == lane and row["target"] == str(target)
                    and row["cache_scope"] == "owned-lane-target", "command state scope mismatch")
            expected = {key: str(target / name) for key, name in (("TMPDIR", "tmp"), ("XDG_CACHE_HOME", "xdg"),
                       ("LUSTR_MIXED_CACHE_DIR", "mixed-cache"), ("LUST_CACHE_DIR", "native-cache"))}
            expected.update(LUST_DAEMON_SOCKET=row["socket"], LUST_DAEMON_IDLE="120s")
            socket = Path(row["socket"])
            require(row["environment"] == expected and socket.is_absolute() and socket.name == f"{lane}.sock",
                    "cache/daemon controls differ")
            sockets.setdefault(epoch, socket.parent)
            require(sockets[epoch] == socket.parent, "socket owner changed within a retained-state epoch")
            prepared.append(serial)
            public.append({key: row[key] for key in ("event", "epoch", "command", "lane", "cache_scope")})
        elif event == "shutdown":
            require(live and row["epoch"] == epoch and row["status"] == "passed" and row["remaining_children"] == 0
                    and type(row["observed_successful_exits"]) is int and row["observed_successful_exits"] >= 0,
                    "daemon shutdown was not observed successfully")
            require(row["reason"] in ("cold-warmup", "cold-timed", "final-evidence", "final"), "unknown shutdown reason")
            live = False
            public.append({key: row[key] for key in ("event", "epoch", "reason", "status", "remaining_children", "observed_successful_exits")})
        elif event == "cleared":
            previous = receipts[index - 1] if index else {}
            require(not live and previous.get("event") == "shutdown" and previous["reason"] == row["reason"]
                    and row["epoch"] == epoch + 1 and row["entries_after_clear"] == 0
                    and row["cache_scope"] == "cargo+mixed+native+tmp+xdg", "state cleared without successful prior shutdown")
            epoch += 1
            live = True
            require(Path(row["target"]) in [report_dir / "targets" / lane for lane in LANES], "clear escaped owned targets")
            following = receipts[index + 1] if index + 1 < len(receipts) else {}
            require(following.get("event") == "prepared" and following["target"] == row["target"], "clear is not bound to its next command")
            phase = {"cold-warmup": "excluded-warmup", "cold-timed": "timed", "final-evidence": "compiler-evidence-after"}.get(row["reason"])
            require(phase is not None and records[following["command"]]["phase"] == phase, "clear occurred in the wrong phase")
            clears.append(row["reason"])
            public.append({key: row[key] for key in ("event", "epoch", "reason", "entries_after_clear", "cache_scope")})
        else:
            raise ValueError("unknown native-state receipt")
    require(len(prepared) == len(records), "incomplete command state receipts")
    require(not live and receipts[-1]["event"] == "shutdown" and receipts[-1]["reason"] == "final", "final shutdown receipt missing")
    spec = report["spec"]
    expected = []
    if spec["scenario"] == "clean-dependency-build":
        expected += ["cold-warmup"] * (WARMUPS * 2) + ["cold-timed"] * (PAIRS * 2)
    if spec["force_root"] is None:
        expected += ["final-evidence"] * 2
    require(clears == expected, "cold/warm state boundaries differ from declared scenario")
    require(sum(r["event"] == "shutdown" for r in receipts) == len(clears) + 1, "missing/duplicate shutdown receipt")
    return public


def project_report(report_path):
    """Validate private evidence first; return a deliberately narrow public view."""
    raw = report_path.read_bytes()
    report = json.loads(raw)
    spec = report["spec"]
    scenario = spec["scenario"]
    require(scenario in SCENARIOS and report["status"] == "passed", "cannot export an unqualified scenario")
    require(spec["pairs"] == PAIRS and spec["warmup_pairs"] == WARMUPS and spec["seed"] == SEED
            and spec["bootstrap_resamples"] == RESAMPLES and spec["resource_mode"] == "disabled", "campaign parameters differ")
    require([lane["name"] for lane in spec["lanes"]] == list(LANES), "campaign lanes differ")
    private = report_path.parent.parent
    expected = make_spec(scenario, source=private / "snapshot", private=private,
                         package=Path(spec["native_state"]["toolchain"]),
                         sdk=Path(spec["lanes"][0]["env"]["CARGO"]).parent.parent,
                         gate=Path(spec["lanes"][0]["gates"][0]["command"]["argv"][0]),
                         external_inputs=[Path(path) for path in spec["external_inputs"]], revision=spec["revision"],
                         host=spec["target"], jobs=int(spec["lanes"][0]["env"]["CARGO_BUILD_JOBS"]))
    require(spec == expected, "commands, gates, profile or environment differ from the fixed project campaign")
    require(report["source_before"] == report["source_after"] == report["final_source_observation"], "source guard differs")
    require(report["order_method"] == ORDER, "unknown order method")
    for row in report["order"]:
        require(set(row) == {"warmup", "pair", "lane_indices"} and type(row["warmup"]) is bool
                and type(row["pair"]) is int and isinstance(row["lane_indices"], list)
                and all(type(index) is int for index in row["lane_indices"]), "public order schema differs")
    require(digest(report_path.parent / "spec.original.json") == report["spec_sha256"], "raw spec digest differs")
    require(json.loads((report_path.parent / "spec.original.json").read_bytes()) == spec, "recorded spec differs")
    require(re.fullmatch(r"[0-9a-f]{64}", report["source_before"]["tree_sha256"]), "invalid source digest")
    require(digest(Path(spec["source"]) / "Cargo.lock") == report["source_before"]["cargo_lock_sha256"], "lockfile digest differs")
    policy = spec["native_state"]
    require(policy["kind"] == POLICY, "missing constructor-bound state policy")
    component_paths = {name: Path(policy["toolchain"]) / "bin" / name
                       for name in ("lustr", "rustc-full", "lust", "lust-core", "cargo-lustr")}
    receipt_paths = {name: Path(policy[key]) for name, key in
                     (("COMPONENTS.txt", "component_manifest"), ("NATIVE-BUILD-RECEIPT.txt", "build_receipt"))}
    for path in [*component_paths.values(), *receipt_paths.values()]:
        require(str(path) in spec["external_inputs"]
                and report["source_before"]["external_inputs"].get(os.fsencode(path).hex()) == digest(path),
                "missing or changed constructor/component guard")
    fingerprints = report["actual_cargo_fingerprints"]
    require(set(fingerprints) == set(LANES) and all(fingerprints[lane] for lane in LANES), "Cargo fingerprint receipt missing")
    state = verify_state(report, report_path.parent)
    public = []
    test_counts = []
    phases = {"identity", "dependency-selection", "compiler-evidence-before", "compiler-evidence-after", "timed", "excluded-warmup",
              "gate:executed-tests", "gate:invalid-borrow", "final-gate:executed-tests", "final-gate:invalid-borrow"}
    for serial, record in enumerate(report["records"]):
        require(record["phase"] in phases and record["lane"] in LANES, "unknown command identity")
        expected_exit = 1 if record["phase"].endswith(":invalid-borrow") else 0
        require(record["exit"] == expected_exit and record["signal"] is None and not record["timed_out"]
                and not record["output_limit"] and math.isfinite(record["wall_seconds"]) and record["wall_seconds"] > 0
                and record["resources"] == {"availability": "disabled", "usage": None}, "invalid command result")
        for stream in STREAM_KEYS:
            require(record[stream] == f"{serial:05}.{stream}", "unexpected raw stream reference")
            path = report_path.parent / record[stream]
            require(path.stat().st_size == record[f"{stream}_bytes"] and digest(path) == record[f"{stream}_sha256"],
                    "raw stream digest/size differs")
        keep = ("lane", "phase", "pair", "wall_seconds", "exit", "signal", "timed_out", "output_limit",
                "stdout_sha256", "stderr_sha256", "stdout_bytes", "stderr_bytes", "resources")
        public.append({key: record[key] for key in keep})
        if record["phase"].endswith(":executed-tests"):
            result = json.loads((report_path.parent / record["stdout"]).read_bytes())
            groups = []
            for group in result["groups"]:
                match = re.fullmatch(r"(\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", group)
                require(match is not None, "unexpected test-count syntax")
                values = dict(zip(("passed", "failed", "ignored", "measured", "filtered_out"), map(int, match.groups())))
                require(values["failed"] == 0 and values["filtered_out"] == 0, "tests were failed or filtered")
                groups.append(values)
            require(result["kind"] == "tests" and sum(row["passed"] for row in groups) > 0, "no executed project tests")
            test_counts.append({"lane": record["lane"], "phase": record["phase"], "harnesses": groups})
    require(len(test_counts) == 4 and all(row["harnesses"] == test_counts[0]["harnesses"] for row in test_counts),
            "initial/final project test counts differ")
    for warm, count in ((True, WARMUPS), (False, PAIRS)):
        rows = [row for row in report["order"] if row["warmup"] is warm]
        require(len(rows) == count and [row["pair"] for row in rows] == list(range(count))
                and all(sorted(row["lane_indices"]) == [0, 1] for row in rows), "invalid paired schedule")
        require(sum(row["lane_indices"][0] == 0 for row in rows) == count // 2, "unbalanced lane positions")
        actual = [(record["pair"], record["lane"]) for record in public
                  if record["phase"] == ("excluded-warmup" if warm else "timed")]
        expected = [(row["pair"], LANES[lane]) for row in rows for lane in row["lane_indices"]]
        require(actual == expected, "records differ from actual paired schedule")
    expected_commands = []
    for lane in spec["lanes"]:
        phases = ["identity"] * len(lane["tools"]) + ["dependency-selection", "compiler-evidence-before", "gate:executed-tests", "gate:invalid-borrow"]
        expected_commands += [(lane["name"], phase, None) for phase in phases]
    for row in report["order"]:
        expected_commands += [(LANES[lane], "excluded-warmup" if row["warmup"] else "timed", row["pair"]) for lane in row["lane_indices"]]
    for lane in LANES:
        expected_commands += [(lane, phase, None) for phase in ("compiler-evidence-after", "final-gate:executed-tests", "final-gate:invalid-borrow")]
    require([(row["lane"], row["phase"], row["pair"]) for row in public] == expected_commands,
            "identity, metadata, compiler evidence or correctness gate records missing/reordered")
    require(len(report["summaries"]) == 1 and report["summaries"][0]["baseline"] == LANES[0]
            and report["summaries"][0]["candidate"] == LANES[1], "paired comparison missing")
    comparison = report["summaries"][0]
    analysis = comparison["analysis"]
    require(set(comparison) == {"baseline", "candidate", "analysis"} and set(analysis) == ANALYSIS_FIELDS,
            "public comparison schema differs")
    require(analysis["method"] == "paired-log-v1/splitmix64/rejection-index/percentile" and analysis["pairs"] == PAIRS
            and analysis["seed"] == SEED and analysis["bootstrap_resamples"] == RESAMPLES, "public analysis identity differs")
    number = lambda value: type(value) in (int, float) and math.isfinite(value)
    for key, count in (("raw_ratios", PAIRS), ("log_ratios", PAIRS), ("bootstrap_95", 2)):
        require(isinstance(analysis[key], list) and len(analysis[key]) == count
                and all(number(value) and (key == "log_ratios" or value > 0) for value in analysis[key]),
                "public analysis arrays must contain finite numbers")
    for key in ("geometric_ratio", "baseline_median", "candidate_median", "baseline_p95", "candidate_p95"):
        require(number(analysis[key]) and analysis[key] > 0, "public analysis must contain finite positive numbers")
    require(number(analysis["exact_sign_flip_p"]) and 0 <= analysis["exact_sign_flip_p"] <= 1, "invalid exact probability")
    # Statistic values are checked against these exact public timings by the
    # repository's existing Rust replay executable, never a Python replacement.
    return {"name": scenario, "lanes": list(LANES), "records": public,
            "comparisons": [{"baseline": LANES[0], "candidate": LANES[1],
                             "analysis": {key: analysis[key] for key in sorted(ANALYSIS_FIELDS)}}],
            "order": [{key: row[key] for key in ("warmup", "pair", "lane_indices")} for row in report["order"]],
            "order_method": ORDER, "test_counts": test_counts,
            "native_state_receipts": state, "source_tree_sha256": report["source_before"]["tree_sha256"],
            "cargo_lock_sha256": report["source_before"]["cargo_lock_sha256"],
            "compiler_components_sha256": {name: digest(path) for name, path in component_paths.items()},
            "constructor_receipts_sha256": {name: digest(path) for name, path in receipt_paths.items()},
            "cargo_fingerprint_counts": {lane: len(fingerprints[lane]) for lane in LANES},
            "full_private_report_sha256": hashlib.sha256(raw).hexdigest(), "original_spec_sha256": report["spec_sha256"],
            "command_contract": {
                "cwd": "<FROZEN_COORDINATOR>", "baseline": "<MATCHED_SDK>/bin/cargo",
                "candidate": ["<SOURCE_BUILT_PACKAGE>/bin/lustr", "--bluefe-root", "<SOURCE_BUILT_PACKAGE>", "--engine", "auto"],
                "shared_cargo_arguments": spec["lanes"][0]["command"]["argv"][1:],
                "shared_metadata_arguments": ["metadata", "--format-version", "1", "--locked", "--offline"],
                "test_gate_arguments": ["test", "--locked", "--offline", "--", "--test-threads=1"],
                "test_gate_routes": {"matched-rust": "tests", "lustr-mixed": "tests-via; same package, engine auto"},
                "rejection_gate": "existing invalid-borrow source; exit 1 with structured E0502",
                "forced_root": spec["force_root"], "force_kind": "byte-preserving timestamp only" if spec["force_root"] else None,
                "profile": spec["profile"] + "; unchanged Cargo defaults", "features": "unchanged Cargo defaults",
                "shared_compiler_flag_evidence": spec["lanes"][0]["evidence_contains"],
            }}


def prepare(args, private):
    repo, package = args.repository, args.toolchain
    require(command(["git", "-C", repo, "status", "--porcelain", "--", "crates/lustr"]) == "",
            "coordinator source has uncommitted changes; freeze the intended Git revision first")
    revision = command(["git", "-C", repo, "rev-parse", "HEAD"])
    tree = command(["git", "-C", repo, "rev-parse", f"{revision}:crates/lustr"])
    reference = package / "bin/rustc-reference"
    require(reference.is_symlink(), "source-built package must identify its original Rust SDK")
    sdk = reference.resolve(strict=True).parent.parent
    for name in ("cargo", "rustdoc"):
        require((package / "bin" / name).resolve(strict=True) == (sdk / "bin" / name).resolve(strict=True), "unmatched SDK component")
    pin = re.findall(r'^source = "([0-9a-f]{40})"$', (repo / "upstream/bluefe-core.pin").read_text(), re.MULTILINE)
    require(len(pin) == 1, "missing pinned full-Rust identity")
    version = command([sdk / "bin/rustc", "-vV"])
    require(command([package / "bin/rustc", "-vV"]) == version, "candidate and reference compiler identities differ")
    fields = dict(line.split(": ", 1) for line in version.splitlines() if ": " in line)
    require(fields.get("commit-hash") == pin[0] and fields.get("host") == "x86_64-unknown-linux-gnu", "unmatched full-Rust source/host")
    require(pin[0] in command([package / "bin/lustr", "--lustr-version"]), "primary entry has a different full-Rust identity")
    require(digest(package / "bin/cargo") == digest(sdk / "bin/cargo"), "Cargo binary differs")
    for filename in ("COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt", "BUILD-PROFILE.txt"):
        require((package.parent / filename).is_file(), "missing source constructor receipt")
    require("Cargo artifact profile=release" in (package.parent / "COMPONENTS.txt").read_text(), "optimized compiler components required")
    require("Cargo artifact profile: release" in (package.parent / "BUILD-PROFILE.txt").read_text(), "constructor profile receipt differs")
    source = private / "snapshot"
    snapshot(repo, revision, source)
    for name in ("home", "cargo-home", "specs"):
        (private / name).mkdir()
    rejection = subprocess.run(["git", "-C", str(repo), "show", f"{revision}:crates/lustr-bench/fixtures/tiny/reject.rs"],
                               capture_output=True, check=True).stdout
    (private / "reject.rs").write_bytes(rejection)
    inputs = [package.parent / name for name in ("COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt", "BUILD-PROFILE.txt")]
    inputs += [package / "bin" / name for name in ("lustr", "rustc-full", "lust", "lust-core", "cargo-lustr", "rustc-reference")]
    inputs += [repo / "upstream/bluefe-core.pin", private / "reject.rs"]
    # Include the host runtime/sysroot, sealed native library and TinyCC inputs.
    # Other installed cross targets are irrelevant to this fixed host campaign.
    for path in sorted((package / "lib").iterdir()):
        if path.name != "rustlib":
            inputs += files_below(path)
    inputs += files_below(package / "lib/rustlib" / fields["host"])
    inputs = sorted(set(inputs))
    specs = {}
    for scenario in SCENARIOS:
        spec = make_spec(scenario, source=source, private=private, package=package, sdk=sdk, gate=args.gate,
                         external_inputs=inputs, revision=revision, host=fields["host"], jobs=args.jobs)
        path = private / "specs" / f"{scenario}.json"
        write_json(path, spec)
        specs[scenario] = path
    identity = {"repository": "https://github.com/glopyglerky/lustr", "revision": revision, "coordinator_git_tree": tree,
                "rust_source": pin[0], "rust_version": version, "host": fields["host"], "compiler_component_profile": "release",
                "compiler_components_sha256": {name: digest(package / "bin" / name) for name in
                                               ("lustr", "rustc-full", "lust", "lust-core", "cargo-lustr")},
                "constructor_receipts_sha256": {name: digest(package.parent / name) for name in
                                                ("COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt", "BUILD-PROFILE.txt")},
                "runner_sha256": digest(args.runner), "gate_sha256": digest(args.gate), "replay_sha256": digest(args.replay),
                "cargo_sha256": digest(sdk / "bin/cargo"), "reference_rustc_sha256": digest(sdk / "bin/rustc"),
                "native_participation": "not asserted; unchanged dev debug=2 is outside current mixed-object eligibility"}
    return specs, identity


def public_readme(data, identity, host):
    lines = ["# Unchanged coordinator performance", "",
             f"Project revision: `{identity['revision']}`. Coordinator Git tree: `{identity['coordinator_git_tree']}`.", "",
             f"Full Rust source: `{identity['rust_source']}`; host `{identity['host']}`. Compiler components use the constructor's release profile.", "",
             "The actual `crates/lustr` project retains its normal dev/test defaults, including debug=2. "
             "The reference uses the matching SDK's Cargo/rustc; Lustr uses the assembled primary entry and its matching Cargo selectors. "
             "This profile is outside current mixed-object eligibility. These results do not establish native acceleration or release admission.", "",
             f"Recorded before the campaign: {host['recorded_utc']}. CPU: {host['cpu_model']['value'] or 'unavailable'}. "
             "See host.json for observed affinity, limits and unknown host facts.", "",
             "Each case contains 12 measured pairs and two excluded warmup pairs. All raw timing values and the actual balanced execution order "
             "are retained in samples.json. No process CPU, memory or energy consumption is measured.", "",
             "| Scenario | Matched Rust median (ms) | Lustr median (ms) | Matched Rust p95 (ms) | Lustr p95 (ms) | Rust / Lustr paired ratio | Bootstrap 95% interval |",
             "| --- | ---: | ---: | ---: | ---: | ---: | --- |"]
    for case in data["cases"]:
        summary = case["comparisons"][0]["analysis"]
        lo, hi = summary["bootstrap_95"]
        lines.append(f"| `{case['name']}` | {1000 * summary['baseline_median']:.3f} | {1000 * summary['candidate_median']:.3f} | "
                     f"{1000 * summary['baseline_p95']:.3f} | {1000 * summary['candidate_p95']:.3f} | {summary['geometric_ratio']:.4f} | [{lo:.4f}, {hi:.4f}] |")
    counts = data["cases"][0]["test_counts"][0]["harnesses"]
    passed, ignored = (sum(group[key] for group in counts) for key in ("passed", "ignored"))
    lines += ["", "Ratios above one favor Lustr; below one favor matched Rust. Medians and p95 are empirical sample summaries. "
              "The paired bootstrap interval and exact sign-flip probabilities are conditional on this campaign's host and repeated workload; "
              "the individual exploratory comparisons are not adjusted for multiplicity. replay.json records verification with the repository's existing Rust statistics implementation.", "",
              "`primed-root-build`, `primed-root-check` and `compile-plus-test` force `src/lib.rs` with a byte-preserving timestamp change. "
              "They are root rebuilds, not body or signature edits. `no-op` changes no timestamps. `clean-dependency-build` clears owned "
              "Cargo/native/mixed/temp state after observed successful daemon shutdown; operating-system and dependency source caches remain warm.", "",
              f"The ordinary project test gate reported {passed} passed and {ignored} existing explicitly ignored tests. "
              "Both lanes passed matching initial/final executing tests and the structured E0502 rejection gate in every scenario. "
              "Observed compiler commands, source/tool guards, Cargo fingerprints, owned state receipts and final daemon shutdown were required. "
              "Test-count details and all command/gate exits are in samples.json.", "",
              "This directory is a derived, path-free public projection. It includes timings, order, command contracts, gate counts, component/source "
              "identities and original evidence digests. It is not a byte-identical archive of private reports, exact environments or raw command output. "
              "Original report/spec/output hashes identify private originals; SHA256SUMS independently identifies these public files. "
              "A temporary CI directory alone does not establish durable private retention.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("repository", "toolchain", "runner", "gate", "replay", "output"):
        parser.add_argument(f"--{name}", required=True, type=Path)
    parser.add_argument("--jobs", type=int, default=2)
    parser.add_argument("--prepare-only", action="store_true", help="write frozen source/specs/identity without claiming any measurements")
    args = parser.parse_args()
    for name in ("repository", "toolchain", "runner", "gate", "replay"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    require(args.output.is_absolute() and not args.output.exists(), "campaign output must be a new absolute directory")
    args.output = args.output.parent.resolve(strict=True) / args.output.name
    require(not args.output.is_relative_to(args.repository), "campaign must be outside the canonical repository")
    require(1 <= args.jobs <= len(os.sched_getaffinity(0)), "job budget must fit process affinity")
    args.output.mkdir(mode=0o700)
    private = args.output / "private"
    private.mkdir(mode=0o700)
    result = {"schema": "lustr-coordinator-campaign-v1", "status": "failed", "scenarios": list(SCENARIOS)}
    try:
        specs, identity = prepare(args, private)
        write_json(private / "identity.json", identity)
        if args.prepare_only:
            result["status"] = "prepared-only"
            return
        module = importlib.util.spec_from_file_location("benchmark_host", args.repository / "tools/capture-benchmark-host.py")
        host = importlib.util.module_from_spec(module)
        module.loader.exec_module(host)
        host_record = host.collect()
        write_json(private / "host.json", host_record)
        cases = []
        for name, spec in specs.items():
            with (private / f"{name}.runner.stdout").open("xb") as stdout, (private / f"{name}.runner.stderr").open("xb") as stderr:
                subprocess.run([str(args.runner), str(spec), str(private / name)], check=True, stdout=stdout, stderr=stderr)
            case = project_report(private / name / "report.json")
            require(case["compiler_components_sha256"] == identity["compiler_components_sha256"], "compiler components changed between scenarios")
            cases.append(case)
        data = {"schema": 1, "pairs": PAIRS, "warmup_pairs": WARMUPS, "seed": SEED, "bootstrap_resamples": RESAMPLES,
                "runner_sha256": identity["runner_sha256"], "cases": cases}
        # Validate before copying any conclusions into the publication directory.
        projection = private / "sanitized-samples.json"
        write_json(projection, data)
        replay = subprocess.run([str(args.replay), str(projection)], check=True, capture_output=True)
        replay_json = json.loads(replay.stdout)
        require(replay_json["verified_comparisons"] == len(SCENARIOS), "not every project comparison replayed")
        public = args.output / "public"
        public.mkdir()
        write_json(public / "samples.json", data)
        write_json(public / "replay.json", replay_json)
        write_json(public / "campaign.json", {**identity, **result, "status": "passed", "project_profiles": "unchanged dev and test defaults",
                                             "jobs": args.jobs, "pairs": PAIRS, "warmup_pairs": WARMUPS,
                                             "measurement": "wall time only; detached daemon CPU, memory and energy are not measured",
                                             "privacy": "Allowlist projection. Original report/spec/output SHA256 values identify private originals, not these rewritten bytes."})
        write_json(public / "host.json", host_record)
        with (public / "README.md").open("x", encoding="utf-8") as readme:
            readme.write(public_readme(data, identity, host_record))
        with (public / "SHA256SUMS").open("x") as manifest:
            for path in sorted(public.iterdir()):
                if path.name != "SHA256SUMS":
                    manifest.write(f"{digest(path)}  {path.name}\n")
        result["status"] = "passed"
        result["public_samples_sha256"] = digest(public / "samples.json")
        result["original_report_sha256"] = {case["name"]: case["full_private_report_sha256"] for case in cases}
        print(public)
    except Exception as error:
        result["error"] = str(error)
        raise
    finally:
        write_json(private / "campaign-result.json", result)


if __name__ == "__main__":
    main()
