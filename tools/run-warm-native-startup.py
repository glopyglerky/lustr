#!/usr/bin/env python3
"""Optional fixed startup diagnostic after a completed full-package campaign.

The real native clients execute only --version. This returns before request
parsing or daemon work. Results do not change the primary campaign or establish
an additive overhead decomposition, optimization benefit or causal explanation.
"""
import argparse
import csv
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import statistics
import sys
import tomllib

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("complete_native", HERE / "run-warm-native-comparison.py")
N = importlib.util.module_from_spec(spec)
spec.loader.exec_module(N)
S, require = N.S, N.require
SCHEMA = "lustr-native-startup-diagnostic-v1"
LANES = ("baseline", "pristine")
PAIRS = 96
SEED = 0x5354415254


def build_hash(data):
    values = re.findall(rb"^cargo:rustc-env=LUST_BUILD_HASH=([0-9a-f]{16})$", data, re.M)
    require(len(values) == 1, "one actual emitted native build identity is required")
    return values[0].decode()


def emitted_receipt(intermediates):
    root = intermediates / "release/build"
    require(root.is_dir() and not root.is_symlink(), "pristine release build-output directory missing")
    matches = []
    for directory, dirs, files in os.walk(root):
        dirs[:] = sorted(name for name in dirs if name not in ("out", "fingerprint"))
        require(not any((Path(directory) / name).is_symlink() for name in dirs), "pristine build-output traversal contains a link")
        for name in sorted(files):
            if name not in ("output", "stdout"): continue
            path = Path(directory) / name
            require(path.is_file() and not path.is_symlink(), "pristine build output is not ordinary")
            if b"cargo:rustc-env=LUST_BUILD_HASH=" in path.read_bytes(): matches.append(path)
    require(len(matches) == 1, "pristine native build output is missing or ambiguous")
    build_hash(matches[0].read_bytes())
    return matches[0]


def expected_version(manifest, output):
    version = tomllib.loads(manifest.decode())["package"]["version"]
    require(isinstance(version, str) and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?", version), "native package version is invalid")
    return f"lust {version} (build {build_hash(output)[:8]})\n".encode()


def orders():
    return [list(LANES if sample % 2 == 0 else reversed(LANES)) for sample in range(PAIRS)]


def environment(package, state):
    # The same held environment is used by both commands. No daemon/socket or
    # native-cache controls are supplied; --version exits before consulting them.
    env = S.controlled_environment(package, state, state / "unused.sock")
    for key in ("LUST_DAEMON_SOCKET", "LUST_DAEMON_IDLE", "LUST_CACHE_DIR"): del env[key]
    return env


def summarize(samples):
    require(len(samples) == PAIRS * 2, "all 192 startup observations are required")
    require([(row["sample"], row["lane"]) for row in samples] == [(i, lane) for i, order in enumerate(orders()) for lane in order], "startup order differs")
    values = {(row["sample"], row["lane"]): row["seconds"] for row in samples}
    require(all(S.STATS.positive(value) for value in values.values()), "invalid startup duration")
    return S.STATS.summarize([(values[i, "pristine"], values[i, "baseline"]) for i in range(PAIRS)], 10000, SEED)


def render(report):
    lines = ["# Native client startup diagnostic", "", "Status: " + report["status"] + ".", "", __doc__.strip(), ""]
    if report["status"] != "completed": return "\n".join(lines + ["No completed diagnostic result. Captured failures and observations remain retained.", ""])
    a = report["summary"]
    lines += ["Frozen complete-package baseline native client versus pristine Lust, under one controlled caller directory and held environment.", "",
        "| Command | Baseline native client | Pristine native client |", "| --- | ---: | ---: |",
        f"| `--version` median | {a['candidate_median']*1000:.6f} ms | {a['baseline_median']*1000:.6f} ms |", "",
        f"Paired pristine/baseline geometric ratio: {a['geometric_ratio']:.6f}; conditional bootstrap 95% interval [{a['bootstrap_95'][0]:.6f}, {a['bootstrap_95'][1]:.6f}]. Values above one favor the baseline native client.", "",
        "The fixed sequence retains 96 alternating pairs (192 observations) after exactly three excluded primes per client. Every command must return zero, exact source/build-receipt-derived version stdout and empty stderr. No daemon or socket is started or contacted.", "",
        "The same 100-microsecond polling recorder used by the primary campaign measures complete process wall time. Startup, process/image placement and measurement behavior remain combined. These observations cannot be subtracted from the warm-command results as a component attribution. There is no diagnostic pass/fail speed threshold.", "",
        "The original 384 timings, lane order and strict Lust target remain authoritative and unchanged. A completed primary target miss is eligible for this optional diagnostic; an incomplete primary campaign is not.", ""]
    return "\n".join(lines)


def run(args):
    require(args.output.is_absolute() and not args.output.exists(), "new absolute diagnostic output required")
    args.output.mkdir(parents=True, mode=0o700)
    recorder = S.Recorder(args.output, 30)
    report = {"schema": SCHEMA, "status": "failed", "pairs": PAIRS, "orders": orders(), "samples": [], "summary": None}
    failure = None
    try:
        primary = N.verify(args.campaign)
        before_report = S.digest(args.campaign / "report.json")
        package_root = (args.campaign / primary["build_receipts"]["baseline-build"]["file"]).parent
        package = N.verify_package(package_root, "baseline", live=True)
        require(package == primary["package_builds"]["baseline"], "baseline package differs from completed campaign")
        baseline_source = Path(package["source_path"])
        pristine_source = Path(primary["pristine_source"])
        require(N.source_identity(pristine_source) == {"revision": S.PIN, "tree": S.TREE}, "pristine source differs")
        pristine_receipt = emitted_receipt(args.pristine_intermediates)
        binaries = {"baseline": Path(package["package"]) / "bin/lust", "pristine": Path(primary["tool_paths"]["pristine"])}
        paths = [*binaries.values(), binaries["baseline"].with_name("lust-core"), binaries["pristine"].with_name("lust-core")]
        for path in paths: require(S.digest(path) == primary["compiler_and_runner_sha256"][str(path)], "native binary differs from completed campaign")
        manifests = {"baseline": baseline_source / "compiler/native/Cargo.toml", "pristine": pristine_source / "Cargo.toml"}
        receipt_files = {"baseline": package_root / package["files"]["NATIVE-BUILD-RECEIPT.txt"]["file"], "pristine": pristine_receipt}
        expected = {lane: expected_version(manifests[lane].read_bytes(), receipt_files[lane].read_bytes()) for lane in LANES}
        provenance = args.output / "provenance"
        provenance.mkdir()
        shutil.copytree(package_root, provenance / "baseline-build")
        bindings = {"source-report": N.archive_file(args.campaign / "report.json", provenance / "source-report.json", args.output)}
        for lane in LANES:
            bindings[lane + "-manifest"] = N.archive_file(manifests[lane], provenance / (lane + "-Cargo.toml"), args.output)
            bindings[lane + "-native-build"] = N.archive_file(receipt_files[lane], provenance / (lane + "-native-build.txt"), args.output)
        bindings["pristine-build-log"] = N.archive_file(args.campaign / primary["build_receipts"]["pristine-build"]["file"], provenance / "pristine-build.log", args.output)
        state = args.output / "work/state"
        for path in (state, state / "tmp", state / "cargo-home"): path.mkdir(parents=True, exist_ok=True)
        cwd = args.campaign / "fixture"
        env = environment(Path(package["package"]), state)
        fingerprints = {str(path): S.digest(path) for path in paths + list(manifests.values()) + list(receipt_files.values()) +
            [Path(__file__), HERE / "run-warm-native-comparison.py", HERE / "run-upstream-slice-benchmark.py", HERE / "report-mixed-performance.py"]}
        report.update(source_campaign_sha256=before_report, baseline_source=package["source"], pristine_source={"revision": S.PIN, "tree": S.TREE},
            binaries={lane: str(path) for lane, path in binaries.items()}, files_sha256=fingerprints, bindings=bindings,
            baseline_build_receipt_sha256=S.digest(package_root / "receipt.json"), pristine_emitted_receipt_path=str(pristine_receipt),
            pristine_intermediates=str(args.pristine_intermediates), expected_stdout={lane: value.decode() for lane, value in expected.items()},
            cwd=str(cwd), state=str(state), environment=env, fixture_files=S.inventory(cwd))
        recorder.event({"kind": "startup-identities", **{key: report[key] for key in IDENTITY_KEYS}})
        for ordinal in range(1, 4):
            for lane in LANES:
                command, out, err = recorder.run([binaries[lane], "--version"], env, cwd, lane, "excluded-startup-prime")
                require(out == expected[lane] and err == b"", "startup prime output differs from source/build identity")
                recorder.event({"kind": "startup-prime", "lane": lane, "ordinal": ordinal, "command": command["id"], "seconds": command["seconds"]})
        for sample, order in enumerate(orders()):
            for lane in order:
                command, out, err = recorder.run([binaries[lane], "--version"], env, cwd, lane, "startup-version")
                require(out == expected[lane] and err == b"", "startup output differs from source/build identity")
                row = {"sample": sample, "lane": lane, "command": command["id"], "seconds": command["seconds"]}
                report["samples"].append(row)
                recorder.event({"kind": "startup-timing", **row})
        after = {path: S.digest(path) for path in fingerprints}
        report["files_sha256_after"] = after
        require(after == fingerprints, "startup compiler/source/receipt changed")
        require(S.digest(args.campaign / "report.json") == before_report and S.inventory(cwd) == report["fixture_files"], "primary campaign changed")
        require(N.verify_package(package_root, "baseline", live=True) == package
                and N.source_identity(pristine_source) == report["pristine_source"], "native package or pristine source changed")
        recorder.event({"kind": "startup-final-identities", "files_sha256": after, "source_campaign_sha256": before_report})
        report.update(status="completed", summary=summarize(report["samples"]))
    except BaseException as error:
        failure = error
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
    finally:
        recorder.raw.close()
        report["raw_sha256"] = S.digest(args.output / "raw.jsonl")
        S.write_json(args.output / "report.json", report)
        (args.output / "README.md").write_text(render(report))
        with (args.output / "samples.csv").open("w", newline="") as output:
            writer = csv.DictWriter(output, fieldnames=("sample", "lane", "command", "seconds"))
            writer.writeheader(); writer.writerows(report["samples"])
    if failure: raise failure
    try: result = verify(args.output)
    except BaseException as error:
        report.update(status="failed", error=f"portable replay: {type(error).__name__}: {error}")
        S.write_json(args.output / "report.json", report)
        (args.output / "README.md").write_text(render(report))
        raise
    print(render(result))


IDENTITY_KEYS = ("source_campaign_sha256", "baseline_source", "pristine_source", "binaries", "files_sha256", "bindings",
    "baseline_build_receipt_sha256", "pristine_emitted_receipt_path", "pristine_intermediates", "expected_stdout", "cwd", "state", "environment", "fixture_files")


def verify(output):
    report = json.loads((output / "report.json").read_text())
    require(report.get("schema") == SCHEMA and report.get("status") == "completed" and report.get("pairs") == PAIRS, "startup diagnostic is incomplete")
    require(S.digest(output / "raw.jsonl") == report["raw_sha256"], "startup raw log changed")
    records = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
    require(records[0] == {"kind": "startup-identities", **{key: report[key] for key in IDENTITY_KEYS}}, "startup identity differs")
    source = json.loads(N.evidence(output, report["bindings"]["source-report"]))
    require(report["bindings"]["source-report"]["sha256"] == report["source_campaign_sha256"] and source.get("schema") == N.SCHEMA
            and source.get("status") == "completed" and source.get("pairs") == 24, "startup source campaign is not completed")
    package_root = output / "provenance/baseline-build"
    package = N.verify_package(package_root, "baseline")
    require(S.digest(package_root / "receipt.json") == report["baseline_build_receipt_sha256"] == source["build_receipts"]["baseline-build"]["sha256"]
            and package == source["package_builds"]["baseline"] and package["source"] == report["baseline_source"], "startup baseline full package differs")
    require(report["pristine_source"] == {"revision": S.PIN, "tree": S.TREE}, "startup pristine pin differs")
    require(report["binaries"] == {"baseline": str(Path(package["package"]) / "bin/lust"), "pristine": source["tool_paths"]["pristine"]}, "startup native entry differs")
    for path in [Path(value) for value in report["binaries"].values()]:
        for identity in (path, path.with_name("lust-core")):
            require(report["files_sha256"][str(identity)] == source["compiler_and_runner_sha256"][str(identity)], "startup binary is not the qualified compiler")
    emitted = Path(report["pristine_emitted_receipt_path"])
    require(emitted.is_relative_to(Path(report["pristine_intermediates"]) / "release/build") and emitted.name in ("output", "stdout"), "pristine build hash was not taken from Cargo's emitted output")
    for lane in LANES:
        manifest = N.evidence(output, report["bindings"][lane + "-manifest"])
        native = N.evidence(output, report["bindings"][lane + "-native-build"])
        require(expected_version(manifest, native).decode() == report["expected_stdout"][lane], "expected version is not source/build-receipt-derived")
    require(report["bindings"]["baseline-native-build"]["sha256"] == package["files"]["NATIVE-BUILD-RECEIPT.txt"]["sha256"]
            and report["files_sha256"][str(emitted)] == report["bindings"]["pristine-native-build"]["sha256"], "startup native receipt binding differs")
    log = N.evidence(output, report["bindings"]["pristine-build-log"])
    require(report["bindings"]["pristine-build-log"]["sha256"] == source["build_receipts"]["pristine-build"]["sha256"] and b"Finished `release` profile" in log, "pristine release build log differs")
    require(report["environment"] == environment(Path(package["package"]), Path(report["state"]))
            and report["fixture_files"] == source["fixture_files"], "startup shared environment/fixture differs")
    require(report["files_sha256_after"] == report["files_sha256"] and records[-1] == {"kind": "startup-final-identities",
        "files_sha256": report["files_sha256_after"], "source_campaign_sha256": report["source_campaign_sha256"]}, "startup final identities differ")
    commands, samples, primes = {}, [], []
    for index, row in enumerate(records[1:-1], 1):
        if row["kind"] == "command":
            require(row["id"] == index and row["id"] not in commands and row["lane"] in LANES
                    and row["argv"] == [report["binaries"][row["lane"]], "--version"]
                    and row["environment"] == report["environment"] and row["cwd"] == report["cwd"]
                    and row["expected_exit"] == row["exit"] == 0 and row["signal"] is None
                    and not row["timed_out"] and not row["output_limit"] and not row.get("launch_error"), "failed or mismatched startup command")
            require(N.evidence(output, row["stdout"]) == report["expected_stdout"][row["lane"]].encode() and N.evidence(output, row["stderr"]) == b"", "startup output differs")
            commands[row["id"]] = row
        elif row["kind"] in ("startup-prime", "startup-timing"):
            command = commands.get(row["command"])
            phase = "excluded-startup-prime" if row["kind"] == "startup-prime" else "startup-version"
            require(command is not None and command["id"] == index - 1 and command["phase"] == phase
                    and command["lane"] == row["lane"] and command["seconds"] == row["seconds"] and S.STATS.positive(row["seconds"]), "startup timing/prime binding differs")
            (primes if row["kind"] == "startup-prime" else samples).append({key: value for key, value in row.items() if key != "kind"})
        else: raise ValueError("startup diagnostic contains an unrelated operation")
    require(len(commands) == 198 and len(primes) == 6 and [(row["ordinal"], row["lane"]) for row in primes] == [(n, lane) for n in range(1, 4) for lane in LANES], "startup primes/commands are incomplete")
    require([row["phase"] for row in commands.values()] == ["excluded-startup-prime"] * 6 + ["startup-version"] * 192, "startup phase order differs")
    require(samples == report["samples"] and report["orders"] == orders(), "startup observations or schedule differ")
    S.STATS.compare(summarize(samples), report["summary"], "startup diagnostic statistics")
    require((output / "README.md").read_text() == render(report), "startup diagnostic table differs")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    live = commands.add_parser("run")
    for name in ("campaign", "pristine-intermediates", "output"): live.add_argument("--" + name, type=Path, required=True)
    replay = commands.add_parser("verify")
    replay.add_argument("output", type=Path)
    args = parser.parse_args()
    for key, value in vars(args).items():
        if isinstance(value, Path): setattr(args, key, value.resolve())
    if args.command == "run": run(args)
    else: verify(args.output); print("Verified 192 startup observations and six excluded primes; no performance admission")


if __name__ == "__main__":
    main()
