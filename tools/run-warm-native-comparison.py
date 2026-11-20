#!/usr/bin/env python3
"""Historical complete-package comparison and evidence replay.

The whole-rustc constructor belongs to the frozen source commits. Current native
packages do not satisfy this historical harness and must never be substituted.

This campaign has a separate schema and package proof. The existing entry-only
comparison and its identical-native-source guard remain unchanged.
"""
import argparse
from contextlib import ExitStack
import csv
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import re
import shutil
import statistics
import subprocess
import sys
import tarfile
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("native_entry_helpers", HERE / "run-warm-entry-comparison.py")
B = importlib.util.module_from_spec(spec)
spec.loader.exec_module(B)
S, require = B.S, B.require
SCHEMA = "lustr-warm-native-packages-v1"
BUILD_SCHEMA = "lustr-complete-source-package-v1"
GATE_SCHEMA = "lustr-complete-package-gates-v2"
BASELINE = "3c378a11d60149a3ec99fa49ea1b336e9406fca8"
BASELINE_TREE = "0256606d5793bfb890130f05fae013d0c5c5f9e1"
TCC = "43c7708b85681a2fd4451c8a541af4494a8919b2"
CARGO_LAYOUT_OPT_OUT = "__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT"
NATIVE_ORACLE_RELEASE = "1.99.0"
NATIVE_ORACLE_COMMIT = "b940084d7eb6a299eb4bfeb8e34901bc051e7ac4"
LANES, PHASES, SEED = B.LANES, B.PHASES, B.SEED
PUBLIC = ("baseline", "entry")
CONSTRUCTOR_INPUTS = ("tools/build-mixed-toolchain.rs", "src/host_runtime.rs", "tools/host-sdk.rs",
                      "src/native_expansion.rs", "src/owned_child.rs", "tools/selection-toolchain-receipt.rs")
RECEIPTS = ("BUILD-PROFILE.txt", "COMPONENTS.txt", "NATIVE-BUILD-RECEIPT.txt", "HOST-SDK-RECEIPT.txt")
PACKAGE_FILES = ("bin/lustr", "bin/lust", "bin/lust-core", "bin/rustc-full", "bin/cargo-lustr", "bin/cargo",
                 "bin/rustc-reference", "bin/rustdoc", "lib/liblustr_native_core.so", "lib/lustr-native.id",
                 "lib/lustr-selection-tools.txt", "lib/lustr/sources.sha256")
IDENTITY_KEYS = ("tool_paths", "daemon_paths", "package_roots", "compiler_and_runner_sha256", "fixture_files",
                 "candidate_identity", "baseline_identity", "package_builds", "package_gates", "build_receipts", "rustc_version")
orders, summaries, parity = B.orders, B.summaries, B.parity
priming_summary, retained_proof, edit_variants = B.priming_summary, B.retained_proof, B.edit_variants
git, source_identity = B.git, B.source_identity


def binding(path, root):
    return {"file": str(path.relative_to(root)), "sha256": S.digest(path), "bytes": path.stat().st_size}


def evidence(root, item):
    path = Path(item["file"])
    require(not path.is_absolute() and ".." not in path.parts and not (root / path).is_symlink()
            and (root / path).resolve().is_relative_to(root.resolve()), "invalid evidence path")
    data = (root / path).read_bytes()
    require(hashlib.sha256(data).hexdigest() == item["sha256"] and len(data) == item["bytes"], "evidence changed")
    return data


def snapshot(root):
    """Own files and external links, without following a linked directory tree.

    Source-library directory links are separately bound by source Git objects;
    external SDK/TinyCC identities are recorded in their own explicit receipts.
    """
    result = {}
    for path in sorted(root.rglob("*")):
        key = str(path.relative_to(root))
        if path.is_symlink():
            value = {"link": os.readlink(path), "resolved": str(path.resolve(strict=True))}
            if path.is_file(): value["sha256"] = S.digest(path)
            else: require(path.is_dir(), "package link points to a special file")
            result[key] = value
        elif path.is_file(): result[key] = {"sha256": S.digest(path)}
        else: require(path.is_dir(), "package contains a special file")
    return result


def native_identity(source):
    return {path: git(source, "rev-parse", "HEAD:" + path) for path in
            ("compiler/native", "library", "crates/lustr-native-core", "upstream/lust.integration",
             "upstream/lust-integration.patch", "upstream/lust-integration.qualification", "upstream/bluefe-core.pin")}


def build_environment(args):
    env = {"PATH": str(args.sdk / "bin") + ":/usr/local/bin:/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
           "HOME": str(Path.home()), "CARGO_HOME": str(args.cargo_home), "CARGO_NET_OFFLINE": "true",
           "CARGO_BUILD_JOBS": "2", "RUSTC": str(args.sdk / "bin/rustc"), "GIT_NO_REPLACE_OBJECTS": "1",
           "GIT_ALTERNATE_OBJECT_DIRECTORIES": str(args.pristine_source / ".git/objects")}
    return env


def archive_file(source, target, root):
    target.parent.mkdir(parents=True, exist_ok=True)
    require(not target.exists(), "evidence destination already exists")
    shutil.copyfile(source, target)
    return binding(target, root)


def build_package(args):
    require(args.lane in PUBLIC and not args.output.exists() and not args.built.exists(), "new build/evidence scope required")
    require(args.built != args.output and not args.built.is_relative_to(args.output), "package and portable receipt scopes must be separate")
    args.output.mkdir(mode=0o700, parents=True)
    receipt = {"schema": BUILD_SCHEMA, "status": "failed", "lane": args.lane, "source_path": str(args.source),
               "built": str(args.built), "package": str(args.built / "toolchain"), "files": {}}
    recorder = S.Recorder(args.output, args.timeout)
    try:
        source = source_identity(args.source)
        if args.lane == "baseline":
            require(source == {"revision": BASELINE, "tree": BASELINE_TREE}, "complete baseline source differs")
        require(source_identity(args.pristine_source) == {"revision": S.PIN, "tree": S.TREE}, "pristine source differs")
        receipt.update(source=source, native_source=native_identity(args.source), sdk=str(args.sdk), tcc=str(args.tcc))
        native = args.source / "compiler/native"
        receipt["differential_inputs"] = {"script_sha256": S.digest(native / "scripts/difftest.sh"),
            "case_pool_sha256": S.digest(native / "scripts/case-pool.sh"),
            "fixture_files": {part: S.inventory(native / "tests" / part) for part in ("pass", "fail", "reject")}}
        source_inputs = {}
        for name in CONSTRUCTOR_INPUTS:
            path = args.source / name
            require(not path.is_symlink() and path.read_bytes() == subprocess.check_output(
                ["git", "-C", str(args.source), "show", "HEAD:" + name]), "constructor source differs from Git")
            item = archive_file(path, args.output / "constructor-source" / name, args.output)
            item["git_blob"] = git(args.source, "rev-parse", "HEAD:" + name)
            source_inputs[name] = item
        receipt["constructor_sources"] = source_inputs
        environment = build_environment(args)
        _, version, _ = recorder.run([args.sdk / "bin/rustc", "-vV"], environment, args.source, args.lane, "sdk-identity")
        require(f"commit-hash: {S.RUST}" in version.decode().splitlines() and f"host: {S.TARGET}" in version.decode().splitlines(), "SDK pin/host differs")
        receipt["rustc_version"] = version.decode()
        receipt["sdk_compiler_sha256"] = S.digest(args.sdk / "bin/rustc")
        receipt["tcc_files"] = snapshot(args.tcc)
        require((args.tcc / "share/licenses/tinycc/SOURCE.txt").read_text().splitlines()[1] == "revision " + TCC, "TinyCC provenance differs")
        constructor = args.output / "constructor"
        recorder.run([args.sdk / "bin/rustc", "--edition=2024", args.source / CONSTRUCTOR_INPUTS[0], "-o", constructor],
                     environment, args.source, args.lane, "constructor-build")
        receipt["constructor_sha256"] = S.digest(constructor)
        recorder.run([constructor, args.source, args.sdk, args.tcc, args.built, "--release"],
                     environment, args.source, args.lane, "complete-package-build")
        require(source_identity(args.source) == source and native_identity(args.source) == receipt["native_source"], "source changed during construction")
        package = args.built / "toolchain"
        require(all((package / name).is_file() for name in PACKAGE_FILES), "incomplete package")
        require(all(not (package / name).is_symlink() for name in ("bin/lustr", "bin/lust", "bin/lust-core", "lib/liblustr_native_core.so")), "native package binary was substituted with a link")
        for name in RECEIPTS:
            receipt["files"][name] = archive_file(args.built / name, args.output / "package-receipts" / name, args.output)
        for path in sorted((args.built / "logs").iterdir()):
            require(path.is_file() and not path.is_symlink(), "invalid constructor log")
            receipt["files"]["logs/" + path.name] = archive_file(path, args.output / "package-receipts/logs" / path.name, args.output)
        tools = package / "lib/lustr-selection-tools.txt"
        receipt["files"]["selection-tools"] = archive_file(tools, args.output / "package-receipts/selection-tools.txt", args.output)
        seeds = list((package / "lib/lustr-selection-verified").glob("tools-*.json"))
        require(len(seeds) == 1 and seeds[0].is_file() and not seeds[0].is_symlink(), "own constructor seed missing or ambiguous")
        require(seeds[0].name == B.selection_seed(seeds[0].read_bytes(), package / "bin/lustr", tools.read_bytes()), "own constructor seed differs")
        receipt["seed"] = {"path": str(seeds[0]), **archive_file(seeds[0], args.output / "package-receipts/selection-seed.json", args.output)}
        receipt["package_files"] = snapshot(package)
        receipt["status"] = "completed"
    except BaseException as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        recorder.raw.close()
        receipt["commands"] = recorder.records
        receipt["raw_sha256"] = S.digest(args.output / "raw.jsonl")
        S.write_json(args.output / "receipt.json", receipt)
    try:
        verify_package(args.output, args.lane, live=True)
    except BaseException as error:
        receipt.update(status="failed", error=f"{type(error).__name__}: {error}")
        S.write_json(args.output / "receipt.json", receipt)
        raise
    print(f"Built independent complete {args.lane} package from {source['revision']}")


def verify_commands(root, commands, phases, lane):
    require([row["phase"] for row in commands] == phases, "required commands missing or reordered")
    for index, row in enumerate(commands):
        require(row["kind"] == "command" and row["id"] == index and row["lane"] == lane and row["expected_exit"] == row["exit"] == 0
                and row["signal"] is None and not row["timed_out"] and not row["output_limit"] and not row.get("launch_error"), "build or qualification command failed")
        for name in ("stdout", "stderr"): evidence(root, row[name])


def verify_package(root, lane, live=False):
    receipt = json.loads((root / "receipt.json").read_text())
    require(receipt.get("schema") == BUILD_SCHEMA and receipt.get("status") == "completed" and receipt.get("lane") == lane, "incomplete or mismatched complete package receipt")
    if lane == "baseline": require(receipt["source"] == {"revision": BASELINE, "tree": BASELINE_TREE}, "complete baseline pin differs")
    require(S.digest(root / "raw.jsonl") == receipt["raw_sha256"] and [json.loads(line) for line in (root / "raw.jsonl").read_text().splitlines()] == receipt["commands"], "constructor command log differs")
    verify_commands(root, receipt["commands"], ["sdk-identity", "constructor-build", "complete-package-build"], lane)
    source, sdk, tcc, built, package = map(Path, (receipt["source_path"], receipt["sdk"], receipt["tcc"], receipt["built"], receipt["package"]))
    require(package == built / "toolchain", "package escaped constructor output")
    commands = receipt["commands"]
    constructor = Path(commands[1]["argv"][-1])
    require(commands[0]["argv"] == [str(sdk / "bin/rustc"), "-vV"]
            and commands[1]["argv"] == [str(sdk / "bin/rustc"), "--edition=2024", str(source / CONSTRUCTOR_INPUTS[0]), "-o", str(constructor)]
            and commands[2]["argv"] == list(map(str, [constructor, source, sdk, tcc, built])) + ["--release"], "not the source's normal full release constructor")
    require(all(row["cwd"] == str(source) and row["environment"] == commands[0]["environment"] for row in commands), "constructor environment/caller differs")
    env = commands[0]["environment"]
    require(env == {"PATH": str(sdk / "bin") + ":/usr/local/bin:/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
        "HOME": env["HOME"], "CARGO_HOME": env["CARGO_HOME"], "CARGO_NET_OFFLINE": "true", "CARGO_BUILD_JOBS": "2",
        "RUSTC": str(sdk / "bin/rustc"), "GIT_NO_REPLACE_OBJECTS": "1", "GIT_ALTERNATE_OBJECT_DIRECTORIES": env["GIT_ALTERNATE_OBJECT_DIRECTORIES"]}, "constructor environment contains other compiler controls")
    require(evidence(root, commands[0]["stdout"]).decode() == receipt["rustc_version"]
            and f"commit-hash: {S.RUST}" in receipt["rustc_version"].splitlines()
            and f"host: {S.TARGET}" in receipt["rustc_version"].splitlines(), "constructor SDK pin differs")
    require(set(receipt["constructor_sources"]) == set(CONSTRUCTOR_INPUTS), "constructor source closure incomplete")
    for item in receipt["constructor_sources"].values():
        data = evidence(root, item)
        require(hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() == item["git_blob"], "constructor source Git blob differs")
    files = receipt["files"]
    for item in files.values(): evidence(root, item)
    require(b"Cargo artifact profile: release\n" in evidence(root, files["BUILD-PROFILE.txt"]), "complete package not release-built")
    source_log = evidence(root, files["logs/source-revision.stdout"]).decode().splitlines()
    require(source_log == [receipt["source"]["revision"], receipt["source"]["tree"]], "constructor built another source revision")
    for name in ("source-revision", "source-identity", "native-build", "entry-build", "cargo-entry-build", "native-library-build", "mixed-build", "preverify-selection-tools"):
        require(evidence(root, files[f"logs/{name}.status"]) == b"exit status: 0\n", "required normal constructor stage failed")
    expected = json.dumps(str(package / "bin/lustr")) + ' "--lustr-prime-selection-tools"\n'
    require(evidence(root, files["logs/preverify-selection-tools.command"]).decode() == expected, "seed was not produced by its own normal constructor")
    require(evidence(root, files["logs/preverify-selection-tools.stdout"]) == evidence(root, files["logs/preverify-selection-tools.stderr"]) == b"", "constructor seed emitted unexpected diagnostics")
    seed_name = B.selection_seed(evidence(root, receipt["seed"]), package / "bin/lustr", evidence(root, files["selection-tools"]))
    require(receipt["seed"]["path"] == str(package / "lib/lustr-selection-verified" / seed_name), "seed installed in another package")
    manifest = receipt["package_files"]
    for name in PACKAGE_FILES: require("sha256" in manifest[name], "package component missing")
    require(all("link" not in manifest[name] for name in ("bin/lustr", "bin/lust", "bin/lust-core", "lib/liblustr_native_core.so")), "native package component was linked from another scope")
    require(manifest["lib/lustr-selection-tools.txt"]["sha256"] == files["selection-tools"]["sha256"]
            and manifest["lib/lustr-selection-verified/" + seed_name]["sha256"] == receipt["seed"]["sha256"], "installed seed/receipt bytes differ")
    hashes = {path: digest for digest, path in re.findall(r"^([0-9a-f]{64})  (.+)$", evidence(root, files["COMPONENTS.txt"]).decode(), re.M)}
    for name in ("lustr", "lust", "lust-core", "rustc-full", "cargo-lustr"):
        require(hashes.get(str(package / "bin" / name)) == manifest["bin/" + name]["sha256"], "installed binary differs from constructor hash")
    native_hashes = re.findall(rb"^cargo:rustc-env=LUST_BUILD_HASH=([0-9a-f]{16})$", evidence(root, files["NATIVE-BUILD-RECEIPT.txt"]), re.M)
    require(len(native_hashes) == 1, "native build identity is ambiguous")
    if live:
        require(source_identity(source) == receipt["source"] and native_identity(source) == receipt["native_source"], "package source changed")
        require(snapshot(package) == manifest and snapshot(tcc) == receipt["tcc_files"], "complete package or TinyCC changed")
        require(S.digest(sdk / "bin/rustc") == receipt["sdk_compiler_sha256"], "SDK compiler changed")
        require(all(S.digest(built / name) == files[name]["sha256"] for name in RECEIPTS), "build receipt changed")
    return receipt


def verify_pair(roots, live=False):
    pair = {lane: verify_package(roots[lane], lane, live) for lane in PUBLIC}
    before, after = pair["baseline"], pair["entry"]
    require(before["package"] != after["package"] and before["built"] != after["built"]
            and before["source_path"] != after["source_path"], "complete packages share their source/build scope")
    for key in ("sdk", "sdk_compiler_sha256", "rustc_version", "tcc", "tcc_files"):
        require(before[key] == after[key], "complete packages used different SDK/TinyCC identities")
    require(evidence(roots["baseline"], before["files"]["HOST-SDK-RECEIPT.txt"]) == evidence(roots["entry"], after["files"]["HOST-SDK-RECEIPT.txt"]), "complete host SDK receipts differ")
    for name in ("bin/cargo", "bin/rustc-reference", "bin/rustdoc"):
        require(before["package_files"][name]["sha256"] == after["package_files"][name]["sha256"], "Rust reference tool bytes differ")
    return pair


def native_differential_environment(environment, oracle_sdk):
    # The frozen native corpus follows Rust 1.99, independently of the newer
    # SDK required to build the complete package and its imported full core.
    # case-pool.sh invokes literal `rustc`, so RUSTC alone does not select it.
    return dict(environment, PATH=str(oracle_sdk / "bin") + ":" + environment["PATH"],
                RUSTC=str(oracle_sdk / "bin/rustc"), DIFFTEST_JOBS="2", DIFFTEST_RUSTC_CACHE="off")


def verify_native_oracle(oracle, live=False):
    require(isinstance(oracle, dict) and set(oracle) == {"sdk", "rustc_version", "compiler_sha256", "files"},
            "native differential oracle identity missing or incomplete")
    require(all(isinstance(oracle[key], str) for key in ("sdk", "rustc_version", "compiler_sha256")),
            "native differential oracle identity fields differ")
    sdk = Path(oracle["sdk"])
    require(sdk.is_absolute(), "native differential oracle SDK must be absolute")
    lines = oracle["rustc_version"].splitlines()
    for expected in (f"release: {NATIVE_ORACLE_RELEASE}", f"commit-hash: {NATIVE_ORACLE_COMMIT}", f"host: {S.TARGET}"):
        require([line for line in lines if line.startswith(expected.split(":", 1)[0] + ":")] == [expected],
                "native differential oracle release/source/host differs")
    require(lines and lines[0].startswith(f"rustc {NATIVE_ORACLE_RELEASE} ("), "native differential oracle version differs")
    files = oracle["files"]
    require(isinstance(files, dict) and files, "native differential oracle files missing")
    for name, item in files.items():
        require(isinstance(name, str) and name and isinstance(item, dict)
                and set(item) in ({"sha256"}, {"link", "resolved"}, {"link", "resolved", "sha256"}),
                "native differential oracle file binding differs")
        path = Path(name)
        require(not path.is_absolute() and ".." not in path.parts and str(path) == name
                and ("sha256" not in item or isinstance(item["sha256"], str) and re.fullmatch(r"[0-9a-f]{64}", item["sha256"]))
                and ("link" not in item or isinstance(item["link"], str) and isinstance(item["resolved"], str)
                     and Path(item["resolved"]).is_absolute()),
                "native differential oracle file binding differs")
    require(files.get("bin/rustc", {}).get("sha256") == oracle["compiler_sha256"]
            and re.fullmatch(r"[0-9a-f]{64}", oracle["compiler_sha256"]), "native differential oracle compiler binding differs")
    for pattern in ("lib/librustc_driver-*.so", "lib/libLLVM*.so*", f"lib/rustlib/{S.TARGET}/lib/libstd-*.rlib"):
        require(any(Path(name).match(pattern) and "sha256" in item for name, item in files.items()),
                "native differential oracle runtime inventory incomplete")
    if live:
        require(snapshot(sdk) == files, "native differential oracle changed")
    return oracle


def verify_native_oracle_pair(gates):
    require(gates["baseline"]["native_oracle"] == gates["entry"]["native_oracle"],
            "complete packages used different native differential oracle identities")


def qualify_package(args):
    require(not args.output.exists(), "new package qualification scope required")
    package_build = verify_package(args.build_receipt, args.lane, live=True)
    args.output.mkdir(mode=0o700, parents=True)
    recorder = S.Recorder(args.output, args.timeout)
    receipt = {"schema": GATE_SCHEMA, "status": "failed", "lane": args.lane, "source": package_build["source"], "output": str(args.output),
               "package": package_build["package"], "build_receipt_sha256": S.digest(args.build_receipt / "receipt.json"),
               "scope": "Gate-profile source unit tests, original ordinary native differential corpus with the installed release binaries, and the complete public entry contract. Full sanitizer/fuzz/platform CI remains separate."}
    try:
        source, sdk, package = map(Path, (package_build["source_path"], package_build["sdk"], package_build["package"]))
        # Direct native tests bypass the public entry, which supplies the
        # pinned Cargo's compatibility layout for Lust's build-script parser.
        # Apply the same setting to both gates, without changing either source
        # package, its constructor or any measured command.
        env = dict(package_build["commands"][0]["environment"], LUST_TCC_HOME=package_build["tcc"],
                   **{CARGO_LAYOUT_OPT_OUT: "1"})
        oracle_sdk = args.native_oracle_sdk
        require(oracle_sdk != sdk, "native differential oracle must be separate from the package SDK")
        diff_env = native_differential_environment(env, oracle_sdk)
        oracle_files = snapshot(oracle_sdk)
        _, oracle_version, oracle_error = recorder.run([oracle_sdk / "bin/rustc", "-vV"],
            diff_env, source, args.lane, "native-oracle-identity")
        receipt["native_oracle"] = {"sdk": str(oracle_sdk), "rustc_version": oracle_version.decode(),
                                    "compiler_sha256": S.digest(oracle_sdk / "bin/rustc"), "files": oracle_files}
        require(not oracle_error, "native differential oracle identity emitted stderr")
        verify_native_oracle(receipt["native_oracle"], live=True)
        work = args.output / "work"
        work.mkdir()
        native = source / "compiler/native"
        test_target = work / "native-tests"
        command = [sdk / "bin/cargo", "test", "--manifest-path", native / "Cargo.toml", "--locked", "--offline", "--profile", "gate",
                   "--jobs", "1", "--bins", "--target-dir", test_target, "--config", f'build.build-dir="{work / "native-intermediates"}"',
                   "--message-format=json", "--", "--test-threads=1"]
        _, out, _ = recorder.run(command, env, source, args.lane, "native-unit-tests")
        artifacts = [json.loads(line) for line in out.splitlines() if line.startswith(b'{')]
        executables = {row["target"]["name"]: Path(row["executable"]) for row in artifacts
                       if row.get("reason") == "compiler-artifact" and row.get("profile", {}).get("test") and row.get("executable")}
        require(set(executables) == {"lust", "lust-core"}, "native unit binaries missing or ambiguous")
        receipt["unit_binaries"] = {name: {"path": str(path), "sha256": S.digest(path), "profile": "gate"} for name, path in executables.items()}
        require(len(re.findall(rb"test result: ok\. [1-9][0-9]* passed; 0 failed;", out)) == 2, "both native unit suites must execute")
        if args.lane == "entry":
            _, oracle_out, _ = recorder.run([executables["lust-core"], "no_change_check_", "--nocapture", "--test-threads=1"],
                dict(env, LUST_CHECK_RESOLVE="1"), source, args.lane, "no-change-resolve-oracle")
            require(b"test result: ok. 2 passed; 0 failed;" in oracle_out, "focused oracle tests did not both execute")
        exported = work / "native-corpus"
        exported.mkdir()
        archive = work / "native-source.tar"
        recorder.run(["git", "-C", source, "archive", "--format=tar", "HEAD:compiler/native", "--output", archive],
                     env, source, args.lane, "native-corpus-export")
        with tarfile.open(archive) as source_archive:
            source_archive.extractall(exported, filter="data")
        (exported / "target").mkdir()
        (exported / "target/release").symlink_to(package / "bin", target_is_directory=True)
        receipt["differential"] = {"source_tree": package_build["native_source"]["compiler/native"],
                                   "export_archive_sha256": S.digest(archive),
                                   "installed_lust": {"path": str(package / "bin/lust"), "sha256": S.digest(package / "bin/lust")},
                                   "installed_core": {"path": str(package / "bin/lust-core"), "sha256": S.digest(package / "bin/lust-core")},
                                   "script_sha256": S.digest(exported / "scripts/difftest.sh"),
                                   "case_pool_sha256": S.digest(exported / "scripts/case-pool.sh"),
                                   "fixture_files": {part: S.inventory(exported / "tests" / part) for part in ("pass", "fail", "reject")}}
        _, diff_out, _ = recorder.run(["bash", exported / "scripts/difftest.sh"], diff_env, exported, args.lane, "ordinary-native-differential")
        count = sum(sum(name.endswith(".rs") and "/" not in name for name in files) for files in receipt["differential"]["fixture_files"].values())
        summaries_found = re.findall(rb"^difftest: ([0-9]+) passed, ([0-9]+) failed$", diff_out, re.M)
        require(summaries_found == [(str(count).encode(), b"0")], "native differential corpus was incomplete or failed")
        receipt["differential"]["passed"] = count
        entry_env = dict(env, LUSTR_CORE_ROOT=str(sdk), LUSTR_LUST_ROOT=str(package), LUSTR_TEST_PACKAGE=str(package),
                         LUSTR_NATIVE_BUILD_OUTPUT=str(Path(package_build["built"]) / "NATIVE-BUILD-RECEIPT.txt"),
                         LUSTR_HOST_SDK_RECEIPT=str(Path(package_build["built"]) / "HOST-SDK-RECEIPT.txt"),
                         LUSTR_TEST_OUTPUT=str(work / "entry-contract"))
        _, entry_out, _ = recorder.run([sdk / "bin/cargo", "test", "--manifest-path", source / "Cargo.toml", "--release",
            "--features", "source-native-runtime", "--test", "compiler_contract", "--locked", "--offline", "--jobs", "2",
            "--target-dir", Path(package_build["built"]) / "work/entry-target", "--config",
            f'build.build-dir="{Path(package_build["built"]) / "work/entry-intermediates"}"', "--", "--ignored", "--test-threads=1"],
            entry_env, source, args.lane, "complete-public-entry-contract")
        require(re.search(rb"test result: ok\. [1-9][0-9]* passed; 0 failed;", entry_out), "public entry contract did not execute")
        require(verify_package(args.build_receipt, args.lane, live=True) == package_build, "qualification changed its release package")
        verify_native_oracle(receipt["native_oracle"], live=True)
        receipt["status"] = "completed"
    except BaseException as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        recorder.raw.close()
        receipt["commands"] = recorder.records
        receipt["raw_sha256"] = S.digest(args.output / "raw.jsonl")
        S.write_json(args.output / "receipt.json", receipt)
    try:
        verify_gates(args.output, package_build, S.digest(args.build_receipt / "receipt.json"), live=True)
    except BaseException as error:
        receipt.update(status="failed", error=f"{type(error).__name__}: {error}")
        S.write_json(args.output / "receipt.json", receipt)
        raise
    print(f"Qualified source/package lane {args.lane}; broader native CI remains separate")


def verify_gates(root, build, build_digest, live=False):
    receipt = json.loads((root / "receipt.json").read_text())
    lane = build["lane"]
    require(receipt.get("schema") == GATE_SCHEMA and receipt.get("status") == "completed" and receipt.get("lane") == lane
            and receipt["source"] == build["source"] and receipt["package"] == build["package"]
            and receipt["build_receipt_sha256"] == build_digest, "package correctness evidence is missing or belongs to another build")
    require(S.digest(root / "raw.jsonl") == receipt["raw_sha256"] and [json.loads(line) for line in (root / "raw.jsonl").read_text().splitlines()] == receipt["commands"], "package correctness raw log differs")
    phases = ["native-oracle-identity", "native-unit-tests"] + (["no-change-resolve-oracle"] if lane == "entry" else []) + ["native-corpus-export", "ordinary-native-differential", "complete-public-entry-contract"]
    verify_commands(root, receipt["commands"], phases, lane)
    commands = {row["phase"]: row for row in receipt["commands"]}
    source, sdk, package, work = map(Path, (build["source_path"], build["sdk"], build["package"], str(Path(receipt["output"]) / "work")))
    environment = dict(build["commands"][0]["environment"], LUST_TCC_HOME=build["tcc"],
                       **{CARGO_LAYOUT_OPT_OUT: "1"})
    native_oracle = verify_native_oracle(receipt.get("native_oracle"), live)
    oracle_sdk = Path(native_oracle["sdk"])
    require(oracle_sdk != sdk, "native differential oracle must be separate from the package SDK")
    diff_env = native_differential_environment(environment, oracle_sdk)
    identity = commands["native-oracle-identity"]
    require(identity["argv"] == [str(oracle_sdk / "bin/rustc"), "-vV"] and identity["environment"] == diff_env
            and identity["cwd"] == str(source) and evidence(root, identity["stdout"]).decode() == native_oracle["rustc_version"]
            and evidence(root, identity["stderr"]) == b"", "native differential oracle command differs")
    unit = commands["native-unit-tests"]
    expected_unit = [sdk / "bin/cargo", "test", "--manifest-path", source / "compiler/native/Cargo.toml", "--locked", "--offline", "--profile", "gate",
        "--jobs", "1", "--bins", "--target-dir", work / "native-tests", "--config", f'build.build-dir="{work / "native-intermediates"}"',
        "--message-format=json", "--", "--test-threads=1"]
    require(unit["argv"] == list(map(str, expected_unit)) and unit["environment"] == environment and unit["cwd"] == str(source), "native unit command differs")
    out = evidence(root, unit["stdout"])
    artifacts = [json.loads(line) for line in out.splitlines() if line.startswith(b'{')]
    executables = {row["target"]["name"]: row["executable"] for row in artifacts
        if row.get("reason") == "compiler-artifact" and row.get("profile", {}).get("test") and row.get("executable")}
    require(executables == {name: row["path"] for name, row in receipt["unit_binaries"].items()}, "unit executable identities differ from Cargo")
    require(len(re.findall(rb"test result: ok\. [1-9][0-9]* passed; 0 failed;", out)) == 2
            and set(receipt["unit_binaries"]) == {"lust", "lust-core"}
            and all(row["profile"] == "gate" for row in receipt["unit_binaries"].values()), "native gate-profile units did not execute")
    if lane == "entry":
        oracle = commands["no-change-resolve-oracle"]
        require(oracle["argv"] == [receipt["unit_binaries"]["lust-core"]["path"], "no_change_check_", "--nocapture", "--test-threads=1"]
                and oracle["environment"] == dict(environment, LUST_CHECK_RESOLVE="1") and oracle["cwd"] == str(source)
                and b"test result: ok. 2 passed; 0 failed;" in evidence(root, oracle["stdout"]), "fresh resolve-oracle focused tests missing")
    export = commands["native-corpus-export"]
    require(export["argv"] == list(map(str, ["git", "-C", source, "archive", "--format=tar", "HEAD:compiler/native", "--output", work / "native-source.tar"]))
            and export["environment"] == environment and export["cwd"] == str(source), "native corpus export command differs")
    differential = receipt["differential"]
    require(differential["source_tree"] == build["native_source"]["compiler/native"], "native differential source differs")
    for label, name in (("installed_lust", "lust"), ("installed_core", "lust-core")):
        require(differential[label] == {"path": str(Path(build["package"]) / "bin" / name), "sha256": build["package_files"]["bin/" + name]["sha256"]}, "differential used another release compiler")
    diff = commands["ordinary-native-differential"]
    require(diff["argv"] == ["bash", str(work / "native-corpus/scripts/difftest.sh")]
            and diff["cwd"] == str(work / "native-corpus") and diff["environment"] == diff_env,
            "original complete differential request differs")
    require(not any(key in diff["environment"] for key in ("DIFFTEST_SHARD", "PANIC", "RELEASE", "SANITIZE", "LUST_STRICT")), "differential mode was changed")
    require({key: differential[key] for key in ("script_sha256", "case_pool_sha256", "fixture_files")} == build["differential_inputs"], "differential inputs differ from the constructed source")
    count = sum(sum(name.endswith(".rs") and "/" not in name for name in files) for files in differential["fixture_files"].values())
    require(count > 0 and set(differential["fixture_files"]) == {"pass", "fail", "reject"} and differential["passed"] == count
            and re.findall(rb"^difftest: ([0-9]+) passed, ([0-9]+) failed$", evidence(root, diff["stdout"]), re.M) == [(str(count).encode(), b"0")], "full native differential count differs")
    entry = commands["complete-public-entry-contract"]
    expected_entry = [sdk / "bin/cargo", "test", "--manifest-path", source / "Cargo.toml", "--release", "--features", "source-native-runtime",
        "--test", "compiler_contract", "--locked", "--offline", "--jobs", "2", "--target-dir", Path(build["built"]) / "work/entry-target",
        "--config", f'build.build-dir="{Path(build["built"]) / "work/entry-intermediates"}"', "--", "--ignored", "--test-threads=1"]
    entry_env = dict(environment, LUSTR_CORE_ROOT=str(sdk), LUSTR_LUST_ROOT=str(package), LUSTR_TEST_PACKAGE=str(package),
        LUSTR_NATIVE_BUILD_OUTPUT=str(Path(build["built"]) / "NATIVE-BUILD-RECEIPT.txt"),
        LUSTR_HOST_SDK_RECEIPT=str(Path(build["built"]) / "HOST-SDK-RECEIPT.txt"), LUSTR_TEST_OUTPUT=str(work / "entry-contract"))
    require(entry["argv"] == list(map(str, expected_entry)) and entry["environment"] == entry_env and entry["cwd"] == str(source)
            and re.search(rb"test result: ok\. [1-9][0-9]* passed; 0 failed;", evidence(root, entry["stdout"])), "complete public entry gate missing")
    return receipt


class Campaign(B.Campaign):
    def __init__(self, args):
        super().__init__(args)
        self.packages = {lane: (args.baseline_built if lane == "baseline" else args.built) / "toolchain" for lane in LANES}
        self.tools["baseline"] = self.packages["baseline"] / "bin/lustr"
        self.daemons = {"baseline": self.packages["baseline"] / "bin/lust", "entry": self.tools["integrated"], "pristine": self.tools["pristine"]}
        self.report.update(schema=SCHEMA, scope="Complete independently source-built baseline and candidate Lustr packages, pristine Lust and Rust/Cargo on the same public slice.",
            limits="Both package lanes use their own release native client/core/library, public entry, normal constructor seed, environment and daemon. One public slice; no broad Rust parity or release admission. Native sanitizer/fuzz/platform CI remains separate.")

    def environment_for_lane(self, lane, state, socket, cold=False):
        for folder in (state, state / "tmp", state / "cargo-home"):
            folder.mkdir(parents=True, exist_ok=True)
        return S.controlled_environment(self.packages[lane], state, socket, cold)

    def daemon_tool(self, lane):
        return self.daemons[lane]

    def execute(self):
        require(platform.system() == "Linux" and platform.machine() == "x86_64", "Linux x86-64 campaign required")
        require(self.args.pairs == 24, "this separately named campaign requires all 24 balanced pairs")
        roots = {"baseline": self.args.baseline_receipt, "entry": self.args.candidate_receipt}
        builds = verify_pair(roots, live=True)
        gate_roots = {"baseline": self.args.baseline_gates, "entry": self.args.candidate_gates}
        gates = {lane: verify_gates(gate_roots[lane], builds[lane], S.digest(roots[lane] / "receipt.json"), live=True) for lane in PUBLIC}
        verify_native_oracle_pair(gates)
        for lane in PUBLIC:
            require(builds[lane]["package"] == str(self.packages[lane]), "executed package differs from constructor")
        self.report["candidate_identity"] = source_identity(self.args.repository)
        self.report["baseline_identity"] = source_identity(self.args.baseline_source)
        require(builds["entry"]["source"] == self.report["candidate_identity"] and builds["baseline"]["source"] == self.report["baseline_identity"], "campaign source differs from its full package")
        require(source_identity(self.args.pristine_source) == {"revision": S.PIN, "tree": S.TREE}, "pristine source differs")
        require(b"Finished `release` profile" in self.args.pristine_build_log.read_bytes(), "release pristine build required")
        reference_slice = self.args.pristine_source / "bench/slices/goport-util-small"
        require(S.inventory(self.args.baseline_source / "compiler/native/bench/slices/goport-util-small") == S.inventory(reference_slice), "baseline fixture source differs")
        self.report["host"] = S.HOST.collect()
        self.report["fixture_preparation"] = S.prepare_fixture(self.args.repository, self.args.pristine_source, self.fixture)
        self.report["tool_paths"] = {key: str(value) for key, value in self.tools.items()}
        self.report["daemon_paths"] = {key: str(value) for key, value in self.daemons.items()}
        self.report["package_roots"] = {key: str(value) for key, value in self.packages.items()}
        self.report["package_builds"], self.report["package_gates"] = builds, gates
        files = list(self.tools.values()) + list(self.daemons.values()) + [self.reference, self.args.pristine_lust.with_name("lust-core"),
            self.args.pristine_build_log, Path(__file__), HERE / "run-warm-entry-comparison.py", HERE / "run-upstream-slice-benchmark.py",
            HERE / "report-mixed-performance.py", HERE / "capture-benchmark-host.py"]
        for lane in PUBLIC:
            files.extend(self.packages[lane] / name for name in PACKAGE_FILES)
        identities = {str(path): S.digest(path) for path in files}
        self.report["compiler_and_runner_sha256"] = identities
        receipts = self.output / "build-receipts"
        receipts.mkdir()
        self.report["build_receipts"] = {}
        for lane in PUBLIC:
            for label, original in (("build", roots[lane]), ("gates", gate_roots[lane])):
                target = receipts / lane / label
                target.mkdir(parents=True)
                names = ["receipt.json", "raw.jsonl", "commands"] + (["constructor-source", "package-receipts"] if label == "build" else [])
                for name in names:
                    source = original / name
                    if source.is_dir(): shutil.copytree(source, target / name)
                    else: shutil.copyfile(source, target / name)
                self.report["build_receipts"][lane + "-" + label] = binding(target / "receipt.json", self.output)
        self.report["build_receipts"]["pristine-build"] = archive_file(self.args.pristine_build_log, receipts / "pristine-build.log", self.output)
        with tempfile.TemporaryDirectory(prefix="lustr-native-") as temporary:
            sockets = Path(temporary)
            env = self.environment_for_lane("cargo", self.output / "work/lockfile", sockets / "unused.sock", True)
            _, out, _ = self.recorder.run([self.reference, "-vV"], env, self.output, "cargo", "rust-identity")
            require(f"commit-hash: {S.RUST}" in out.decode().splitlines(), "Rust pin differs")
            self.report["rustc_version"] = out.decode()
            self.recorder.run([self.tools["cargo"], "generate-lockfile", "--offline", "--manifest-path", self.fixture / "Cargo.toml"],
                              env, self.fixture, "cargo", "lockfile")
            source = S.inventory(self.fixture)
            self.report["fixture_files"] = source
            reject = self.output / "rejected.rs"
            reject.write_text(S.REJECTION)
            self.report["rejection_input"] = {"file": "rejected.rs", "sha256": S.digest(reject)}
            self.recorder.event({"kind": "identities", **{key: self.report[key] for key in IDENTITY_KEYS}})
            self.qualification(sockets, "initial")
            self.cache_edit_qualification(sockets)
            self.report["orders"] = orders(self.args.pairs)
            for sample, order in enumerate(self.report["orders"]):
                for operation in ("check", "build"):
                    self.measure_group(sample, order, operation, sockets)
                    require(S.inventory(self.fixture) == source, "fixture changed during measurement")
            self.qualification(sockets, "final")
            require(S.inventory(self.fixture) == source and reject.read_text() == S.REJECTION, "input changed")
        require(all(S.digest(path) == value for path, value in identities.items()), "compiler/runner changed")
        require(verify_pair(roots, live=True) == builds, "source or complete package changed during comparison")
        require(source_identity(self.args.pristine_source) == {"revision": S.PIN, "tree": S.TREE}, "pristine source changed")
        self.report.update(samples=self.samples, summaries=summaries(self.samples, self.args.pairs), status="completed")
        self.report["warm_target"] = parity(self.report["summaries"])
        self.report["priming_summary"] = priming_summary(self.recorder.records, self.args.pairs)



    def qualification(self, sockets, stage):
        self.recorder.event({"kind":"qualification-start", "stage":stage})
        for lane in ("cargo", "pristine", "baseline", "entry"):
            state = self.output / "work" / stage / "runtime" / lane
            env = self.environment_for_lane(lane, state, sockets / (lane + ".sock"), True)
            _, binary = self.compile(lane, "build", state, env, "gate-build", trace=True)
            self.runtime(lane, binary, env, "gate-runtime")
        for lane in ("pristine", "baseline", "entry"):
            for operation in ("check", "build"):
                state = self.output / "work" / stage / ("retained-" + operation) / lane
                env = dict(self.environment_for_lane(lane, state, sockets / (lane + ".sock")), LUST_STATS="1")
                selection_trace = state / "selection-events.jsonl"
                if lane in ("baseline", "entry"):
                    env["LUSTR_SELECTION_DIAGNOSTICS"] = str(selection_trace)
                with S.Daemon(self.recorder, self.daemon_tool(lane), env, self.fixture, lane, f"{stage}-{operation}-{lane}") as daemon:
                    self.compile(lane, operation, state, env, "gate-warm-prime", trace=True)
                    if lane in ("baseline", "entry"):
                        for _ in range(3):
                            self.compile(lane, operation, state, env, "gate-selection-prime")
                        before = selection_trace.read_bytes()
                        command, _ = self.compile(lane, operation, state, env, "gate-selection-hit")
                        after = selection_trace.read_bytes()
                        require(after.startswith(before), "selection diagnostics were replaced")
                        rows = [json.loads(line) for line in after[len(before):].splitlines()]
                        require(rows == [{"selection_cache":"hit"}], "warm public entry did not reuse exact target selection")
                        archive = self.output / "commands" / f"{command['id']:05}.selection.trace"
                        archive.write_bytes(after[len(before):])
                        self.recorder.event({"kind":"selection-cache", "lane":lane, "command":command["id"], "operation":operation,
                                             "trace":{"file":str(archive.relative_to(self.output)), "sha256":S.digest(archive), "bytes":archive.stat().st_size}})
                    command, _, stderr = self.recorder.run(self.command(lane, operation, state) + ["--timings"],
                                                          env, self.fixture, lane, "gate-retained-" + operation)
                    proof = retained_proof(stderr, operation)
                    self.recorder.event({"kind":"retained-route", "command":command["id"], "lane":lane, "operation":operation, **proof})
                    if operation == "build":
                        binary = state / "program"
                        self.built_artifacts[binary] = {"build_command":command["id"], "binary_sha256":S.digest(binary)}
                        self.recorder.event({"kind":"build-identity", "binary":str(binary), **self.built_artifacts[binary]})
                        self.runtime(lane, state / "program", env, "gate-retained-runtime")
                    daemon.alive()
        reject = self.output / "rejected.rs"
        for lane in LANES:
            state = self.output / "work" / stage / "rejection" / lane
            env = self.environment_for_lane(lane, state, sockets / (lane + ".sock"), True)
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
                env = self.environment_for_lane(lane, state, sockets / (lane + ".sock"), True)
                states[lane] = state
                result, binary = self.compile(lane, operation, state, env, "cold-" + operation)
                self.timing(sample, lane, "cold-" + operation, [result])
                if binary is not None:
                    self.runtime(lane, binary, env, "cold-build-runtime")
            for lane in order:
                env = self.environment_for_lane(lane, states[lane], sockets / (lane + ".sock"))
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

    def cache_edit_qualification(self, sockets):
        """Both complete public packages must retain their own edit sessions."""
        main = self.fixture / "src/slice_main.rs"
        original, source = main.read_bytes(), S.inventory(self.fixture)
        lanes = ("cargo", "baseline", "entry")
        states = {lane: self.output / "work/cache-edits" / lane for lane in lanes}
        envs = {lane: self.environment_for_lane(lane, states[lane], sockets / ("cache-edits-" + lane + ".sock")) for lane in lanes}
        selection_paths = {lane: states[lane] / "selection.jsonl" for lane in PUBLIC}
        entry_paths = {lane: states[lane] / "entry.jsonl" for lane in PUBLIC}
        for lane in PUBLIC:
            envs[lane].update(LUST_STATS="1", LUSTR_SELECTION_DIAGNOSTICS=str(selection_paths[lane]), LUSTR_TRACE_PATH=str(entry_paths[lane]))
        def archive(command, suffix, data):
            path = self.output / "commands" / f"{command['id']:05}.{suffix}"
            path.write_bytes(data)
            return binding(path, self.output)
        self.recorder.event({"kind": "cache-edit-start"})
        try:
            with ExitStack() as stack:
                daemons = {lane: stack.enter_context(S.Daemon(self.recorder, self.daemon_tool(lane), envs[lane], self.fixture, lane, "cache-edits")) for lane in PUBLIC}
                for lane in PUBLIC:
                    for operation in ("check", "build"):
                        for _ in range(3): self.compile(lane, operation, states[lane], envs[lane], "cache-edit-prime")
                for stage, contents in edit_variants(original).items():
                    main.write_bytes(contents)
                    snapshot = self.output / "commands" / f"cache-edit-{stage}.rs"
                    snapshot.write_bytes(contents)
                    observed = S.inventory(self.fixture)
                    require(observed == dict(source, **{"src/slice_main.rs": S.digest(main)}), "edit changed another source input")
                    self.recorder.event({"kind": "cache-edit-source", "stage": stage, "source": binding(snapshot, self.output), "fixture_files": observed})
                    commands, runs = {}, {}
                    for operation in ("check", "build"):
                        for lane in lanes:
                            state, env = states[lane], envs[lane]
                            before_selection = selection_paths[lane].read_bytes() if lane in PUBLIC else b""
                            before_entry = entry_paths[lane].read_bytes() if lane in PUBLIC else b""
                            binary = state / (f"target/{S.TARGET}/debug/slice" if lane == "cargo" else "program")
                            if operation == "build" and binary.exists(): binary.unlink()
                            phase = f"cache-edit-{stage}-{operation}"
                            expected = 101 if stage == "invalid" else 0
                            command, out, err = self.recorder.run(self.command(lane, operation, state), env, self.fixture, lane, phase, expected)
                            commands[lane + "-" + operation] = command["id"]
                            if lane in PUBLIC:
                                selection, entry = selection_paths[lane].read_bytes(), entry_paths[lane].read_bytes()
                                require(selection.startswith(before_selection) and entry.startswith(before_entry), "edit diagnostics were replaced")
                                selection, entry = selection[len(before_selection):], entry[len(before_entry):]
                                require([json.loads(line) for line in selection.splitlines()] == [{"selection_cache": "hit"}], "body edit did not retain target selection")
                                expected_route = ([{"engine": "lust-rejected", "entry": "daemon"}, {"engine": "full-rust-core", "entry": "cargo"}]
                                                  if stage == "invalid" else [{"engine": "lust", "entry": "daemon"}])
                                require([json.loads(line) for line in entry.splitlines()] == expected_route, "edit escaped its retained native compiler")
                                self.recorder.event({"kind": "cache-edit-proof", "lane": lane, "command": command["id"], "stage": stage, "operation": operation,
                                    "selection": archive(command, "edit-selection.trace", selection), "entry": archive(command, "edit-entry.trace", entry),
                                    "retained": retained_proof(err, operation) if stage == "initial" else None})
                            if stage == "invalid":
                                require(b"E0308" in out + err, "cached invalid body has the wrong rejection")
                                require(operation != "build" or not binary.exists(), "rejected edit left an executable")
                            elif lane == "cargo": S.cargo_artifact(out, operation, state / "target", False)
                            if operation == "build" and stage != "invalid":
                                require(binary.is_file() and not binary.is_symlink(), "edited build did not produce an ordinary executable")
                                built = {"build_command": command["id"], "binary_sha256": S.digest(binary)}
                                self.recorder.event({"kind": "build-identity", "binary": str(binary), **built})
                                run, stdout, stderr = self.recorder.run([binary], env, self.fixture, lane, "cache-edit-execute")
                                self.recorder.event({"kind": "runtime-identity", "command": run["id"], "binary": str(binary), **built})
                                runs[lane] = run["id"]
                                expected_output = (b"selection-cache-edited\n" if stage == "edited" else b"") + self.expected_output
                                require(stdout == expected_output and stderr == b"", "edited runtime differs from required Rust output")
                            for daemon in daemons.values(): daemon.alive()
                    self.recorder.event({"kind": "cache-edit-step", "stage": stage, "commands": commands, "runs": runs})
        finally:
            main.write_bytes(original)
        require(S.inventory(self.fixture) == source, "edit gate did not restore original source")
        self.recorder.event({"kind": "cache-edit-completed"})


def render(report):
    lines = ["# Complete native-package warm comparison", "", f"Status: {report['status']}.", "", report["scope"], "", report["limits"], ""]
    if report["status"] != "completed":
        return "\n".join(lines + ["No completed performance result. Every captured command, failure and timing remains retained.", ""])
    lines += ["Each public lane was built from its own source by its own normal release constructor and qualified before timing. Its installed selection seed, native binaries, SDK, TinyCC, package paths and owned daemons are bound separately.", "",
              f"Baseline: `{report['baseline_identity']['revision']}`. Candidate: `{report['candidate_identity']['revision']}`.", "",
              "All times below are complete-command medians in milliseconds. The fixed schedule contains 24 balanced pairs and 384 timings; none are trimmed.", "",
              "| Operation | Baseline Lustr | Candidate Lustr | Pristine Lust | rustc via Cargo |", "| --- | ---: | ---: | ---: | ---: |"]
    for phase in PHASES:
        values = [statistics.median(row["seconds"] for row in report["samples"] if row["lane"] == lane and row["phase"] == phase) * 1000 for lane in LANES]
        lines.append(f"| {phase} | " + " | ".join(f"{value:.3f}" for value in values) + " |")
    lines += ["", "Three complete priming commands precede each measured warm command. Their medians remain visible in milliseconds; these are excluded from the warm timing but never discarded.", "",
              "| Operation | Lane | First prime | Second prime | Third prime |", "| --- | --- | ---: | ---: | ---: |"]
    for operation in ("check", "build"):
        for lane in LANES:
            values = [row["median_seconds"] * 1000 for row in report["priming_summary"] if row["operation"] == operation and row["lane"] == lane]
            lines.append(f"| {operation} | {lane} | " + " | ".join(f"{value:.3f}" for value in values) + " |")
    lines += ["", "Ratios are paired reference/candidate geometric means; values above one favor the candidate.", "",
              "| Operation | Reference | Candidate | Ratio | Conditional bootstrap 95% interval |", "| --- | --- | --- | ---: | --- |"]
    for row in report["summaries"]:
        a = row["analysis"]
        lines.append(f"| {row['phase']} | {row['baseline']} | {row['candidate']} | {a['geometric_ratio']:.4f} | [{a['bootstrap_95'][0]:.4f}, {a['bootstrap_95'][1]:.4f}] |")
    lines += ["", f"Warm target met: {report['warm_target']['meets_target']}.", "", report["warm_target"]["criterion"], "",
        "Both public lanes preserve the actual retained selection route through a body edit, rejection and restoration; runtime exit status, stdout and stderr are checked against Rust. All timed builds are executed and bound to their successful build. Checks require retained no-work counters; builds require cached objects and zero new body/object work.", "",
        "Gate-profile unit executables are correctness tools, separately identified from release performance binaries. The package gates cover ordinary native differential tests and public entry contracts. Full sanitizer/fuzz/platform CI and release admission remain separate. This one adapted public slice does not establish broad compiler parity; intervals are conditional on this run.", ""]
    return "\n".join(lines)


def lane_environment(report, lane, env):
    require(lane in LANES, "unknown command lane")
    state = Path(env["CARGO_TARGET_DIR"]).parent
    cold = "LUST_NO_DAEMON" in env or "LUST_NO_CACHE" in env
    expected = S.controlled_environment(Path(report["package_roots"][lane]), state, Path(env["LUST_DAEMON_SOCKET"]), cold)
    extra = {key: value for key, value in env.items() if key in ("LUST_STATS", "LUSTR_TRACE_PATH", "LUSTR_SELECTION_DIAGNOSTICS")}
    require(env == dict(expected, **extra), "lane environment escaped its complete package or owned caches")
    for key in ("LUSTR_TRACE_PATH", "LUSTR_SELECTION_DIAGNOSTICS"):
        if key in extra: require(Path(extra[key]).is_relative_to(state), "diagnostic path escaped its lane")
    return state, cold


def verify(output):
    report = json.loads((output / "report.json").read_text())
    require(report.get("schema") == SCHEMA and report.get("status") == "completed" and report.get("pairs") == 24, "only complete fixed 24-pair native-package campaigns have results")
    require(S.digest(output / "raw.jsonl") == report["raw_sha256"], "raw log changed")
    records = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
    def stream(record):
        path = Path(record["file"])
        require(not path.is_absolute() and ".." not in path.parts and not (output / path).is_symlink(), "invalid evidence path")
        data = (output / path).read_bytes()
        require(S.digest(output / path) == record["sha256"] and ("bytes" not in record or len(data) == record["bytes"]), "stream changed")
        return data
    identity, = [row for row in records if row["kind"] == "identities"]
    require(identity == {"kind": "identities", **{key: report[key] for key in IDENTITY_KEYS}}, "source/build identity record differs")
    require(f"commit-hash: {S.RUST}" in report["rustc_version"].splitlines(), "Rust pin differs")
    for item in report["build_receipts"].values(): stream(item)
    roots = {lane: (output / report["build_receipts"][lane + "-build"]["file"]).parent for lane in PUBLIC}
    builds = verify_pair(roots)
    require(builds == report["package_builds"] and builds["baseline"]["source"] == report["baseline_identity"]
            and builds["entry"]["source"] == report["candidate_identity"], "complete package source bindings differ")
    for lane in PUBLIC:
        root = (output / report["build_receipts"][lane + "-gates"]["file"]).parent
        require(verify_gates(root, builds[lane], report["build_receipts"][lane + "-build"]["sha256"]) == report["package_gates"][lane], "complete package gate binding differs")
        require(report["tool_paths"][lane] == str(Path(builds[lane]["package"]) / "bin/lustr")
                and report["daemon_paths"][lane] == str(Path(builds[lane]["package"]) / "bin/lust"), "public launcher or daemon belongs to another native package")
        for name in PACKAGE_FILES:
            require(report["compiler_and_runner_sha256"][str(Path(builds[lane]["package"]) / name)] == builds[lane]["package_files"][name]["sha256"], "executed package component differs")
    verify_native_oracle_pair(report["package_gates"])
    require(report["package_roots"] == {lane: builds["baseline" if lane == "baseline" else "entry"]["package"] for lane in LANES}, "package environment bindings differ")
    require(report["daemon_paths"]["pristine"] == report["tool_paths"]["pristine"]
            and report["tool_paths"]["integrated"] == report["daemon_paths"]["entry"]
            and report["tool_paths"]["cargo"] == str(Path(builds["entry"]["package"]) / "bin/cargo"), "reference/native tool bindings differ")
    require(b"Finished `release` profile" in stream(report["build_receipts"]["pristine-build"]), "pristine compiler was not release built")

    commands, retained, active, built, runtime_commands, runtime_builds, selection = {}, [], {}, {}, set(), {}, []
    edit_active, edit_done, edit_sources, edit_proofs, edit_steps = False, False, [], [], []
    variants = edit_variants((output / "fixture/src/slice_main.rs").read_bytes())
    group = None
    oracle = (output / "runtime-oracle.stdout").read_bytes()
    require(oracle and S.digest(output / "runtime-oracle.stdout") == report["runtime_oracle_sha256"], "runtime oracle changed")
    for index, row in enumerate(records):
        if row["kind"] == "command":
            require(row["id"] == index and row["id"] not in commands and row["exit"] == row["expected_exit"]
                    and row["signal"] is None and not row["timed_out"] and not row["output_limit"] and not row.get("launch_error"), "failed or unbound command")
            stream(row["stdout"]); stream(row["stderr"])
            commands[row["id"]] = row
            _, cold = lane_environment(report, row["lane"], row["environment"])
            if row["lane"] != "cargo" and not cold:
                owner = active.get(row["lane"])
                require(owner is not None and owner["socket"] == row["environment"]["LUST_DAEMON_SOCKET"], "warm command lacks its own package daemon")
                require({k:v for k,v in owner["environment"].items() if k != "LUSTR_TRACE_PATH"} ==
                        {k:v for k,v in row["environment"].items() if k != "LUSTR_TRACE_PATH"}, "warm command changed its daemon environment")
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
            runtime_builds[row["command"]] = row["build_command"]
        if row["kind"] == "group-start":
            require(group is None, "measurement groups overlap")
            group = row
        if row["kind"] == "group-end":
            require(group is not None and row["sample"] == group["sample"] and row["operation"] == group["operation"], "measurement group end differs")
            group = None
        if row["kind"] == "daemon-start":
            require(row["lane"] in report["daemon_paths"] and row["lane"] not in active, "daemon lane already active or unknown")
            _, cold = lane_environment(report, row["lane"], row["environment"])
            require(not cold and row["executable"] == report["daemon_paths"][row["lane"]]
                    and row["socket"] == row["environment"]["LUST_DAEMON_SOCKET"], "daemon executable/environment belongs to another package")
            require(all(owner["socket"] != row["socket"] for owner in active.values()), "native lanes share a daemon socket")
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
            require(retained_proof(stream(command["stderr"]), row["operation"]) == {key:row[key] for key in ("route", "counters")}, "retained proof differs")
            require(row["lane"] in active and active[row["lane"]]["environment"].get("LUST_STATS") == "1", "retained daemon not instrumented")
            retained.append((row["lane"], row["operation"]))
        if row["kind"] == "cache-edit-start":
            require(not edit_active and not edit_done and group is None, "edit gate overlaps another campaign phase")
            edit_active = True
        if row["kind"] == "cache-edit-source":
            require(edit_active and all(lane in active and active[lane]["scope"] == "cache-edits" for lane in PUBLIC), "edit has no retained owned package daemons")
            require(row["stage"] in variants and stream(row["source"]) == variants[row["stage"]], "edit source snapshot differs")
            require(row["fixture_files"] == dict(report["fixture_files"], **{"src/slice_main.rs":row["source"]["sha256"]}), "edit changed another input")
            edit_sources.append(row["stage"])
        if row["kind"] == "cache-edit-proof":
            command = commands[row["command"]]
            require(edit_active and edit_sources and row["stage"] == edit_sources[-1]
                    and row["lane"] in PUBLIC and command["lane"] == row["lane"] and command["phase"] == f"cache-edit-{row['stage']}-{row['operation']}", "edit proof is misbound")
            require(row["lane"] in active and command["environment"] == active[row["lane"]]["environment"], "edit command lost its own package daemon environment")
            require([json.loads(line) for line in stream(row["selection"]).splitlines()] == [{"selection_cache":"hit"}], "edit used fresh target selection")
            route = ([{"engine":"lust-rejected", "entry":"daemon"}, {"engine":"full-rust-core", "entry":"cargo"}]
                     if row["stage"] == "invalid" else [{"engine":"lust", "entry":"daemon"}])
            require([json.loads(line) for line in stream(row["entry"]).splitlines()] == route, "edit bypassed its native daemon")
            require(row["retained"] == (retained_proof(stream(command["stderr"]), row["operation"]) if row["stage"] == "initial" else None), "initial edit state was not retained")
            edit_proofs.append((row["stage"], row["operation"], row["lane"], row["command"]))
        if row["kind"] == "cache-edit-step":
            require(edit_active and edit_sources and row["stage"] == edit_sources[-1], "edit step escaped its source state")
            require(set(row["commands"]) == {lane + "-" + op for lane in ("cargo", "baseline", "entry") for op in ("check", "build")}, "edit omitted a compiler operation")
            for operation in ("check", "build"):
                for lane in ("cargo", "baseline", "entry"):
                    command = commands[row["commands"][lane + "-" + operation]]
                    env, fixture = command["environment"], Path(command["cwd"])
                    state = Path(env["CARGO_TARGET_DIR"]).parent
                    require(command["lane"] == lane and command["phase"] == f"cache-edit-{row['stage']}-{operation}"
                            and command["exit"] == (101 if row["stage"] == "invalid" else 0), "edited compilation status differs")
                    require(state.parts[-2:] == ("cache-edits", lane) and "LUST_NO_DAEMON" not in env and "LUST_NO_CACHE" not in env, "edit state is not retained")
                    require(env["LUST_CACHE_DIR"] == str(state / "native-cache") and env["CARGO_HOME"] == str(state / "cargo-home"), "edit cache directories differ")
                    if lane == "cargo":
                        expected = [report["tool_paths"][lane], operation, "--offline", "--locked", "--manifest-path", str(fixture / "Cargo.toml"),
                                    "--bin", "slice", "--target", S.TARGET, "--message-format=json-render-diagnostics"]
                        if row["stage"] != "invalid": S.cargo_artifact(stream(command["stdout"]), operation, state / "target", False)
                    else:
                        expected = [report["tool_paths"][lane], operation, str(fixture / "src/slice_main.rs"), "--manifest-path", str(fixture / "Cargo.toml"),
                                    "--bin", "slice", "--offline", "--locked", *S.FLAGS]
                        if operation == "build": expected += ["-o", str(state / "program")]
                        require((row["stage"], operation, lane, command["id"]) in edit_proofs, "edited public command has no cache/daemon proof")
                    require(command["argv"] == expected, "edited request changed")
                    if row["stage"] == "invalid":
                        require(b"E0308" in stream(command["stdout"]) + stream(command["stderr"]), "edited type error was not rejected")
            require(set(row["runs"]) == (set() if row["stage"] == "invalid" else {"cargo", "baseline", "entry"}), "edit runtime evidence count differs")
            for lane, command_id in row["runs"].items():
                command = commands[command_id]
                expected = (b"selection-cache-edited\n" if row["stage"] == "edited" else b"") + oracle
                require(command_id in runtime_commands and command["lane"] == lane and command["phase"] == "cache-edit-execute"
                        and runtime_builds[command_id] == row["commands"][lane + "-build"]
                        and command["exit"] == 0 and stream(command["stdout"]) == expected and stream(command["stderr"]) == b"", "edited runtime differs from Rust")
            edit_steps.append(row["stage"])
        if row["kind"] == "cache-edit-completed":
            require(edit_active and not edit_done and all(lane not in active for lane in PUBLIC), "edit daemons did not shut down cleanly")
            require(edit_sources == edit_steps == list(variants) and [(stage,op,lane) for stage,op,lane,_ in edit_proofs] ==
                    [(stage,op,lane) for stage in variants for op in ("check", "build") for lane in PUBLIC], "cache-hit edit transition sequence is incomplete")
            edit_active, edit_done = False, True
        if row["kind"] == "selection-cache":
            command = commands[row["command"]]
            require(row["lane"] in ("baseline", "entry") and command["lane"] == row["lane"] and command["phase"] == "gate-selection-hit"
                    and command["argv"][0] == report["tool_paths"][row["lane"]], "selection-cache proof uses another command")
            require([json.loads(line) for line in stream(row["trace"]).splitlines()] == [{"selection_cache":"hit"}], "selection-cache hit missing")
            selection.append((row["lane"], row["operation"]))
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
    for phase in ("gate-build", "gate-runtime", "gate-reject"):
        require([row["lane"] for row in commands.values() if row["phase"] == phase] ==
                [lane for _ in range(2) for lane in (("cargo", "pristine", "baseline", "entry") if phase != "gate-reject" else LANES)], "initial/final runtime or rejection gate omitted")
    require({row["command"] for row in records if row["kind"] == "entry-route"} ==
            {row["id"] for row in commands.values() if row["lane"] in PUBLIC and row["phase"] in ("gate-build", "gate-warm-prime")}, "required entry-route proof omitted")
    for row in commands.values():
        if row["phase"] in ("cold-build", "no-op-build", "gate-build", "gate-retained-build"):
            require(sum(value == row["id"] for value in runtime_builds.values()) == 1, "successful measured/gate build lacks one bound execution")
    require(not active, "daemon still active")
    require(group is None, "measurement group did not finish")
    require(runtime_commands == {row["id"] for row in commands.values() if row["phase"].endswith("runtime") or row["phase"] == "cache-edit-execute"}, "runtime identity missing")
    require(edit_done and not edit_active, "cache-hit edit transition qualification is missing")
    require(retained == [(lane,op) for _ in range(2) for lane in ("pristine", "baseline", "entry") for op in ("check", "build")], "missing retained check/build proof")
    require(selection == [(lane,op) for _ in range(2) for lane in ("baseline", "entry") for op in ("check", "build")], "missing initial/final selection reuse proof")
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




def main():
    if sys.argv[1:2] != ["verify"]:
        print("Retired whole-rustc acquisition is disabled. Use verify DIRECTORY for immutable historical evidence.", file=sys.stderr)
        return 2
    parser = argparse.ArgumentParser(description="Verify frozen historical evidence without compiler execution or current qualification.")
    parser.add_argument("command", choices=("verify",))
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    verify(args.output.resolve())
    print("Verified historical evidence; no compiler execution or current qualification.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
