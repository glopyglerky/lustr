#!/usr/bin/env python3
"""Independent-package orchestration/corruption controls; synthetic children only.

No compiler is built or executed here and no generated timing is performance
evidence. Production build/gate/campaign orchestration runs with explicitly
synthetic child results, then the real portable replay verifier checks them.
"""
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("native_comparison", HERE / "run-warm-native-comparison.py")
N = importlib.util.module_from_spec(spec)
spec.loader.exec_module(N)
spec = importlib.util.spec_from_file_location("entry_controls", HERE / "test-warm-entry-comparison.py")
E = importlib.util.module_from_spec(spec)
spec.loader.exec_module(E)


def put(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data.encode() if isinstance(data, str) else data)


def fake_source(parent, name):
    source = parent / name
    source.mkdir()
    for path in N.CONSTRUCTOR_INPUTS: put(source / path, "// synthetic constructor source: " + path + "\n")
    native = source / "compiler/native"
    for group in ("pass", "fail", "reject"): put(native / "tests" / group / "synthetic.rs", "// synthetic corpus, never compiled\n")
    put(native / "scripts/difftest.sh", "# synthetic script, never executed\n")
    put(native / "scripts/case-pool.sh", "# synthetic pool, never executed\n")
    fixture = native / "bench/slices/goport-util-small"
    put(fixture / "src/slice_main.rs", "// synthetic slice\nfn main() {}\n")
    put(fixture / "prune.json", json.dumps({"spec": {"root": "slice_main.rs"}}))
    put(native / "bench/slices/driver.rs", "// synthetic driver, never compiled\n")
    return source


class SyntheticChildren:
    def __init__(self, parent):
        self.parent = parent
        self.sdk, self.tcc, self.pristine = [parent / name for name in ("synthetic-sdk", "synthetic-tcc", "synthetic-pristine")]
        for name in ("rustc", "cargo", "rustdoc"): put(self.sdk / "bin" / name, "synthetic SDK " + name + "\n")
        self.native_oracle_sdk = parent / "synthetic-native-oracle"
        for name in ("bin/rustc", "lib/librustc_driver-synthetic.so", "lib/libLLVM-synthetic.so",
                     f"lib/rustlib/{N.S.TARGET}/lib/libstd-synthetic.rlib"):
            put(self.native_oracle_sdk / name, "synthetic native oracle, never executable: " + name + "\n")
        self.native_oracle_version = (f"rustc {N.NATIVE_ORACLE_RELEASE} (synthetic identity; not an executed compiler)\n"
            f"commit-hash: {N.NATIVE_ORACLE_COMMIT}\nhost: {N.S.TARGET}\nrelease: {N.NATIVE_ORACLE_RELEASE}\n").encode()
        put(self.tcc / "bin/tcc", "synthetic TinyCC\n")
        put(self.tcc / "share/licenses/tinycc/SOURCE.txt", "Synthetic provenance for controls only\nrevision " + N.TCC + "\n")
        self.sources = {lane: fake_source(parent, "synthetic-source-" + lane) for lane in N.PUBLIC}
        shutil.copytree(self.sources["baseline"] / "compiler/native/bench", self.pristine / "bench")
        self.source_ids = {self.sources["baseline"]: {"revision": N.BASELINE, "tree": N.BASELINE_TREE},
                           self.sources["entry"]: {"revision": "c" * 40, "tree": "d" * 40}, self.pristine: {"revision": N.S.PIN, "tree": N.S.TREE}}
        self.native_ids = {source: {name: hashlib.sha1((str(source) + name).encode()).hexdigest() for name in
            ("compiler/native", "library", "crates/lustr-native-core", "upstream/lust.integration", "upstream/lust-integration.patch", "upstream/lust-integration.qualification", "upstream/bluefe-core.pin")} for source in self.sources.values()}
        self.version = f"synthetic rustc identity; not an executed compiler\ncommit-hash: {N.S.RUST}\nhost: {N.S.TARGET}\n".encode()
        self.campaign = None

    def package(self, source, built):
        package = built / "toolchain"
        for name in N.PACKAGE_FILES:
            put(package / name, "synthetic package component " + str(source) + "/" + name + "\n")
        for name, target in (("cargo", "cargo"), ("rustc-reference", "rustc"), ("rustdoc", "rustdoc")):
            (package / "bin" / name).unlink()
            (package / "bin" / name).symlink_to(self.sdk / "bin" / target)
        put(built / "BUILD-PROFILE.txt", "Cargo artifact profile: release\nSynthetic controls, never a release build.\n")
        put(built / "HOST-SDK-RECEIPT.txt", "synthetic shared SDK receipt\n")
        put(built / "NATIVE-BUILD-RECEIPT.txt", "cargo:rustc-env=LUST_BUILD_HASH=" + hashlib.sha256(str(source).encode()).hexdigest()[:16] + "\n")
        components = "Synthetic constructor output, not compiled binaries.\n"
        for name in ("lustr", "lust", "lust-core", "rustc-full", "cargo-lustr"):
            path = package / "bin" / name
            components += N.S.digest(path) + "  " + str(path) + "\n"
        put(built / "COMPONENTS.txt", components)
        names = ("source-revision", "source-identity", "native-build", "entry-build", "cargo-entry-build", "native-library-build", "mixed-build", "preverify-selection-tools")
        for name in names:
            for suffix, data in (("status", "exit status: 0\n"), ("stdout", b""), ("stderr", b""), ("command", "synthetic command\n")):
                put(built / "logs" / (name + "." + suffix), data)
        put(built / "logs/source-revision.stdout", "\n".join(self.source_ids[source].values()) + "\n")
        put(built / "logs/preverify-selection-tools.command", json.dumps(str(package / "bin/lustr")) + ' "--lustr-prime-selection-tools"\n')
        tools = (package / "lib/lustr-selection-tools.txt").read_bytes()
        payload = {"schema": "lustr-cargo-selection-v2", "tools": {"receipt": tools.decode(), "identities": [[str(package / "bin/lustr")] * 2]}}
        encoded = json.dumps(payload, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
        seed = json.dumps({"payload": payload, "sha256": hashlib.sha256(encoded).hexdigest()}).encode()
        name = N.B.selection_seed(seed, package / "bin/lustr", tools)
        put(package / "lib/lustr-selection-verified" / name, seed)

    def run(self, recorder, argv, env, cwd, lane, phase, expected=0):
        argv = list(map(str, argv))
        data = error = b""
        if phase in ("sdk-identity", "rust-identity"): data = self.version
        elif phase == "native-oracle-identity": data = self.native_oracle_version
        elif phase == "constructor-build": put(Path(argv[-1]), "synthetic constructor, never executable\n")
        elif phase == "complete-package-build": self.package(Path(argv[1]), Path(argv[-2]))
        elif phase == "native-unit-tests":
            target = Path(argv[argv.index("--target-dir") + 1])
            for name in ("lust", "lust-core"):
                executable = target / "gate/deps" / (name + "-synthetic")
                put(executable, "synthetic gate-profile test binary\n")
                data += (json.dumps({"reason": "compiler-artifact", "target": {"name": name}, "profile": {"test": True}, "executable": str(executable)}) + "\n").encode()
            data += b"test result: ok. 1 passed; 0 failed;\ntest result: ok. 2 passed; 0 failed;\n"
        elif phase == "no-change-resolve-oracle": data = b"test result: ok. 2 passed; 0 failed;\n"
        elif phase == "native-corpus-export":
            with tarfile.open(argv[-1], "w") as archive:
                for path in sorted((Path(argv[2]) / "compiler/native").iterdir()): archive.add(path, arcname=path.name)
        elif phase == "ordinary-native-differential": data = b"difftest: 3 passed, 0 failed\n"
        elif phase == "complete-public-entry-contract": data = b"test result: ok. 9 passed; 0 failed;\n"
        elif phase == "lockfile": put(self.campaign.fixture / "Cargo.lock", "# synthetic lockfile\n")
        elif phase == "gate-reject": error = b"error[E0308]: synthetic mismatch\n"
        elif phase.startswith("cache-edit-invalid-"): error = b"error[E0308]: synthetic edited mismatch\n"
        elif phase in ("gate-retained-check", "gate-retained-build"):
            error = E.PRISTINE_RETAINED_BUILD if phase.endswith("build") else E.PRISTINE_RETAINED_CHECK
        elif phase.startswith("cache-edit-initial-") and lane in N.PUBLIC:
            error = E.PRISTINE_RETAINED_BUILD if phase.endswith("build") else E.PRISTINE_RETAINED_CHECK
        elif len(argv) == 1:
            data = (b"selection-cache-edited\n" if b"selection-cache-edited" in (self.campaign.fixture / "src/slice_main.rs").read_bytes() else b"") + b"synthetic runtime output\n"
        if self.campaign and len(argv) > 1 and argv[1] in ("check", "build") and phase != "gate-reject":
            binary = None
            if lane == "cargo" and expected == 0:
                target = Path(env["CARGO_TARGET_DIR"])
                binary = target / N.S.TARGET / "debug/slice" if argv[1] == "build" else None
                data = (json.dumps({"reason": "compiler-artifact", "target": {"name": "slice", "kind": ["bin"]},
                    "profile": {"opt_level": "0", "debuginfo": 0, "debug_assertions": True, "overflow_checks": True},
                    "fresh": phase not in ("cold-check", "cold-build", "gate-build") and not phase.startswith("cache-edit-"), "executable": str(binary) if binary else None}) + "\n" +
                    json.dumps({"reason": "build-finished", "success": True}) + "\n").encode()
            elif "-o" in argv and expected == 0: binary = Path(argv[argv.index("-o") + 1])
            if binary: put(binary, "synthetic program, never executable\n")
            if "LUSTR_TRACE_PATH" in env:
                route = ([{"engine": "lust-rejected", "entry": "daemon"}, {"engine": "full-rust-core", "entry": "cargo"}] if expected == 101 else
                    [{"engine": "lust", "entry": "upstream-process" if "LUST_NO_DAEMON" in env else "daemon"}])
                with Path(env["LUSTR_TRACE_PATH"]).open("a") as stream: stream.write("".join(json.dumps(row) + "\n" for row in route))
            if "LUSTR_SELECTION_DIAGNOSTICS" in env:
                hit = phase == "gate-selection-hit" or phase.startswith("cache-edit-") and phase != "cache-edit-prime"
                with Path(env["LUSTR_SELECTION_DIAGNOSTICS"]).open("a") as stream: stream.write(json.dumps({"selection_cache": "hit" if hit else "miss"}) + "\n")
        row = {"kind": "command", "id": len(recorder.records), "argv": argv, "environment": dict(env), "cwd": str(cwd), "lane": lane, "phase": phase,
               "expected_exit": expected, "exit": expected, "signal": None, "timed_out": False, "output_limit": False, "seconds": .01}
        for name, value in (("stdout", data), ("stderr", error)):
            path = recorder.output / "commands" / f"{row['id']:05}.{name}"
            path.write_bytes(value)
            row[name] = N.binding(path, recorder.output)
        recorder.event(row)
        return row, data, error

    def git_blob(self, source, *args):
        self.assert_source(source)
        data = (source / args[-1].removeprefix("HEAD:")).read_bytes()
        return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()

    def assert_source(self, source):
        if source not in self.sources.values(): raise AssertionError("unexpected synthetic source")

    def git_show(self, argv):
        if argv[0:2] != ["git", "-C"] or argv[3] != "show": raise AssertionError("unexpected synthetic Git command")
        source = Path(argv[2]); self.assert_source(source)
        return (source / argv[4].removeprefix("HEAD:")).read_bytes()


class SyntheticDaemon:
    def __init__(self, recorder, executable, env, cwd, lane, scope):
        self.recorder = recorder
        self.row = {"kind": "daemon-start", "lane": lane, "pid": len(recorder.records) + 100, "scope": scope,
                    "executable": str(executable), "environment": dict(env), "cwd": str(cwd), "socket": env["LUST_DAEMON_SOCKET"]}
    def __enter__(self): self.recorder.event(self.row); return self
    def alive(self): pass
    def __exit__(self, *args):
        self.recorder.event({"kind": "daemon-stop", **{key: self.row[key] for key in ("lane", "pid", "scope")}, "exit": 0, "error": None, "streams": []})


def synthetic_campaign(parent):
    children = SyntheticChildren(parent)
    args = {}
    def run(recorder, *values, **kw): return children.run(recorder, *values, **kw)
    with patch.object(N, "source_identity", side_effect=lambda source: children.source_ids[source]), \
         patch.object(N, "native_identity", side_effect=lambda source: children.native_ids[source]), \
         patch.object(N, "git", side_effect=children.git_blob), patch.object(N.subprocess, "check_output", side_effect=children.git_show), \
         patch.object(N.S.Recorder, "run", run), patch.object(N.S, "Daemon", SyntheticDaemon), patch.object(N.S.HOST, "collect", return_value={"synthetic": True}):
        for lane in N.PUBLIC:
            args[lane] = SimpleNamespace(source=children.sources[lane], lane=lane, output=parent / (lane + "-build"), built=parent / (lane + "-built"),
                sdk=children.sdk, tcc=children.tcc, pristine_source=children.pristine, cargo_home=parent / "cargo-home", timeout=30)
            N.build_package(args[lane])
            N.qualify_package(SimpleNamespace(build_receipt=args[lane].output, native_oracle_sdk=children.native_oracle_sdk,
                lane=lane, output=parent / (lane + "-gates"), timeout=30))
        put(parent / "pristine-bin/lust", "synthetic pristine client\n")
        put(parent / "pristine-bin/lust-core", "synthetic pristine core\n")
        put(parent / "pristine-build.log", "Synthetic controls only; Finished `release` profile\n")
        output = parent / "campaign"
        output.mkdir()
        campaign = N.Campaign(SimpleNamespace(output=output, timeout=30, pairs=24, repository=children.sources["entry"], baseline_source=children.sources["baseline"],
            built=args["entry"].built, baseline_built=args["baseline"].built, baseline_receipt=args["baseline"].output, candidate_receipt=args["entry"].output,
            baseline_gates=parent / "baseline-gates", candidate_gates=parent / "entry-gates", pristine_source=children.pristine,
            pristine_lust=parent / "pristine-bin/lust", pristine_build_log=parent / "pristine-build.log"))
        children.campaign = campaign
        try: campaign.execute()
        finally: campaign.recorder.raw.close()
        campaign.report["raw_sha256"] = N.S.digest(output / "raw.jsonl")
        N.S.write_json(output / "report.json", campaign.report)
        (output / "README.md").write_text(N.render(campaign.report))
    return output


class PackageComparisonControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.parent = Path(cls.temporary.name)
        cls.original = synthetic_campaign(cls.parent)
    @classmethod
    def tearDownClass(cls): cls.temporary.cleanup()

    def corrupted(self, mutate):
        output = self.parent / self.id().split(".")[-1]
        shutil.copytree(self.original, output)
        rows = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
        mutate(rows)
        (output / "raw.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
        report = json.loads((output / "report.json").read_text())
        report["raw_sha256"] = N.S.digest(output / "raw.jsonl")
        N.S.write_json(output / "report.json", report)
        return output

    def test_complete_two_package_orchestration_replays_all_384_timings(self):
        report = N.verify(self.original)
        self.assertEqual(len(report["samples"]), 384)
        self.assertNotEqual(report["package_roots"]["baseline"], report["package_roots"]["entry"])

    def test_baseline_cannot_use_candidate_daemon(self):
        def mutate(rows):
            identity = next(row for row in rows if row["kind"] == "identities")
            next(row for row in rows if row["kind"] == "daemon-start" and row["lane"] == "baseline")["executable"] = identity["daemon_paths"]["entry"]
        with self.assertRaisesRegex(ValueError, "daemon executable"): N.verify(self.corrupted(mutate))

    def test_baseline_environment_cannot_borrow_candidate_tools(self):
        for variable in ("PATH", "CARGO", "RUSTC", "RUSTDOC"):
            def mutate(rows):
                baseline = next(row for row in rows if row["kind"] == "command" and row["lane"] == "baseline" and row["phase"] == "cold-check")
                candidate = next(row for row in rows if row["kind"] == "command" and row["lane"] == "entry" and row["phase"] == "cold-check")
                baseline["environment"][variable] = candidate["environment"][variable]
            with self.subTest(variable=variable):
                output = self.parent / self.id().split(".")[-1]
                if output.exists(): shutil.rmtree(output)
                with self.assertRaisesRegex(ValueError, "lane environment"): N.verify(self.corrupted(mutate))

    def test_baseline_edit_gate_cannot_be_omitted(self):
        def mutate(rows): next(row for row in rows if row["kind"] == "cache-edit-proof" and row["lane"] == "baseline")["kind"] = "omitted"
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_owned_native_lanes_cannot_share_a_socket(self):
        def mutate(rows):
            starts = [row for row in rows if row["kind"] == "daemon-start" and row["scope"] == "cache-edits"]
            starts[1]["socket"] = starts[0]["socket"]
            starts[1]["environment"]["LUST_DAEMON_SOCKET"] = starts[0]["socket"]
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_timed_command_cannot_switch_public_entry(self):
        def mutate(rows): next(row for row in rows if row["kind"] == "command" and row["phase"] == "no-op-check")["argv"][0] = "/synthetic-wrong/compiler"
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_runtime_identity_cannot_be_omitted(self):
        def mutate(rows): next(row for row in rows if row["kind"] == "runtime-identity")["kind"] = "omitted"
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_retained_route_cannot_be_omitted(self):
        def mutate(rows): next(row for row in rows if row["kind"] == "retained-route")["kind"] = "omitted"
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_timed_order_cannot_be_changed(self):
        def mutate(rows): next(row for row in rows if row["kind"] == "group-start")["order"].reverse()
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_timeout_remains_incomplete(self):
        def mutate(rows): next(row for row in rows if row["kind"] == "command" and row["phase"] == "cold-build")["timed_out"] = True
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_failed_invalid_edit_cannot_become_success(self):
        def mutate(rows):
            row = next(row for row in rows if row["kind"] == "command" and row["lane"] == "baseline" and row["phase"] == "cache-edit-invalid-build")
            row["exit"] = row["expected_exit"] = 0
        with self.assertRaises(ValueError): N.verify(self.corrupted(mutate))

    def test_constructor_receipt_refuses_entry_only_schema(self):
        root = self.parent / "wrong-build-schema"
        shutil.copytree(self.parent / "baseline-build", root)
        receipt = json.loads((root / "receipt.json").read_text()); receipt["schema"] = N.B.SCHEMA
        N.S.write_json(root / "receipt.json", receipt)
        with self.assertRaises(ValueError): N.verify_package(root, "baseline")

    def test_constructor_receipt_refuses_wrong_frozen_source(self):
        root = self.parent / "wrong-build-source"
        shutil.copytree(self.parent / "baseline-build", root)
        receipt = json.loads((root / "receipt.json").read_text()); receipt["source"]["revision"] = "f" * 40
        N.S.write_json(root / "receipt.json", receipt)
        with self.assertRaisesRegex(ValueError, "baseline pin"): N.verify_package(root, "baseline")

    def test_constructor_receipt_refuses_missing_own_seed(self):
        root = self.parent / "missing-seed"
        shutil.copytree(self.parent / "baseline-build", root)
        receipt = json.loads((root / "receipt.json").read_text())
        (root / receipt["seed"]["file"]).unlink()
        with self.assertRaises((ValueError, FileNotFoundError)): N.verify_package(root, "baseline")

    def gate_corruption(self, mutate, lane="entry", suffix=""):
        root = self.parent / (self.id().split(".")[-1] + suffix)
        shutil.copytree(self.parent / (lane + "-gates"), root)
        receipt = json.loads((root / "receipt.json").read_text())
        mutate(receipt)
        (root / "raw.jsonl").write_text("".join(json.dumps(row) + "\n" for row in receipt["commands"]))
        receipt["raw_sha256"] = N.S.digest(root / "raw.jsonl")
        N.S.write_json(root / "receipt.json", receipt)
        return root

    def verify_gate(self, root, lane="entry"):
        build_root = self.parent / (lane + "-build")
        build = N.verify_package(build_root, lane)
        return N.verify_gates(root, build, N.S.digest(build_root / "receipt.json"))

    def refuse_layout_change(self, phase, value):
        for lane in N.PUBLIC:
            with self.subTest(lane=lane):
                def mutate(receipt):
                    env = next(row for row in receipt["commands"] if row["phase"] == phase)["environment"]
                    if value is None:
                        del env[N.CARGO_LAYOUT_OPT_OUT]
                    else:
                        env[N.CARGO_LAYOUT_OPT_OUT] = value
                root = self.gate_corruption(mutate, lane, "-" + lane)
                with self.assertRaises(ValueError): self.verify_gate(root, lane)

    def test_native_units_require_recorded_cargo_layout(self):
        self.refuse_layout_change("native-unit-tests", None)

    def test_native_units_refuse_conflicting_cargo_layout(self):
        self.refuse_layout_change("native-unit-tests", "0")

    def test_native_differential_requires_recorded_cargo_layout(self):
        self.refuse_layout_change("ordinary-native-differential", None)

    def test_native_differential_refuses_conflicting_cargo_layout(self):
        self.refuse_layout_change("ordinary-native-differential", "0")

    def test_native_oracle_identity_is_required(self):
        with self.assertRaisesRegex(ValueError, "oracle identity"):
            self.verify_gate(self.gate_corruption(lambda receipt: receipt.pop("native_oracle")))

    def test_native_oracle_release_source_and_host_are_pinned(self):
        for field, expected in (("release", N.NATIVE_ORACLE_RELEASE), ("commit-hash", N.NATIVE_ORACLE_COMMIT), ("host", N.S.TARGET)):
            for mode in ("replace", "duplicate"):
                with self.subTest(field=field, mode=mode):
                    def mutate(receipt):
                        oracle = receipt["native_oracle"]
                        oracle["rustc_version"] = (oracle["rustc_version"].replace(field + ": " + expected, field + ": wrong")
                            if mode == "replace" else oracle["rustc_version"] + field + ": wrong\n")
                    with self.assertRaisesRegex(ValueError, "release/source/host"):
                        self.verify_gate(self.gate_corruption(mutate, suffix="-" + field + "-" + mode))

    def test_native_oracle_identity_command_cannot_be_omitted(self):
        def mutate(receipt):
            receipt["commands"] = [row for row in receipt["commands"] if row["phase"] != "native-oracle-identity"]
            for index, row in enumerate(receipt["commands"]): row["id"] = index
        with self.assertRaises(ValueError): self.verify_gate(self.gate_corruption(mutate))

    def test_native_oracle_identity_command_cannot_use_build_sdk(self):
        def mutate(receipt):
            next(row for row in receipt["commands"] if row["phase"] == "native-oracle-identity")["argv"][0] = str(self.parent / "synthetic-sdk/bin/rustc")
        with self.assertRaisesRegex(ValueError, "oracle command"):
            self.verify_gate(self.gate_corruption(mutate))

    def test_native_differential_requires_oracle_path_and_rustc(self):
        for lane in N.PUBLIC:
            for key in ("PATH", "RUSTC"):
                with self.subTest(lane=lane, key=key):
                    def mutate(receipt):
                        unit = next(row for row in receipt["commands"] if row["phase"] == "native-unit-tests")
                        diff = next(row for row in receipt["commands"] if row["phase"] == "ordinary-native-differential")
                        diff["environment"][key] = unit["environment"][key]
                    with self.assertRaisesRegex(ValueError, "differential request"):
                        self.verify_gate(self.gate_corruption(mutate, lane, "-" + lane + "-" + key), lane)

    def test_native_oracle_compiler_digest_must_match_inventory(self):
        def mutate(receipt): receipt["native_oracle"]["compiler_sha256"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "compiler binding"):
            self.verify_gate(self.gate_corruption(mutate))

    def test_native_oracle_runtime_inventory_cannot_be_omitted(self):
        for fragment in ("librustc_driver", "libLLVM", "libstd"):
            with self.subTest(fragment=fragment):
                def mutate(receipt):
                    files = receipt["native_oracle"]["files"]
                    del files[next(name for name in files if fragment in name)]
                with self.assertRaisesRegex(ValueError, "runtime inventory"):
                    self.verify_gate(self.gate_corruption(mutate, suffix="-" + fragment))

    def test_live_native_oracle_runtime_change_is_rejected(self):
        receipt = json.loads((self.parent / "entry-gates/receipt.json").read_text())
        oracle = receipt["native_oracle"]
        path = Path(oracle["sdk"]) / "lib/libLLVM-synthetic.so"
        original = path.read_bytes()
        try:
            path.write_bytes(original + b"changed\n")
            with self.assertRaisesRegex(ValueError, "oracle changed"):
                N.verify_native_oracle(oracle, live=True)
        finally: path.write_bytes(original)

    def test_two_packages_cannot_have_different_oracle_identities(self):
        output = self.parent / self.id().split(".")[-1]
        shutil.copytree(self.original, output)
        report = json.loads((output / "report.json").read_text())
        gate_path = output / report["build_receipts"]["entry-gates"]["file"]
        gate = json.loads(gate_path.read_text())
        gate["native_oracle"]["compiler_sha256"] = gate["native_oracle"]["files"]["bin/rustc"]["sha256"] = "f" * 64
        N.S.write_json(gate_path, gate)
        # Rebind every enclosing checksum; both receipts remain individually
        # valid. Full portable replay must still reject different oracles.
        self.assertEqual(self.verify_gate(gate_path.parent), gate)
        report["build_receipts"]["entry-gates"] = N.binding(gate_path, output)
        report["package_gates"]["entry"] = gate
        rows = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
        identity = next(row for row in rows if row["kind"] == "identities")
        identity.update({key: report[key] for key in N.IDENTITY_KEYS})
        (output / "raw.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
        report["raw_sha256"] = N.S.digest(output / "raw.jsonl")
        N.S.write_json(output / "report.json", report)
        with self.assertRaisesRegex(ValueError, "different native differential oracle"):
            N.verify(output)

    def test_native_oracle_does_not_replace_unit_entry_or_measured_sdk(self):
        report = N.verify(self.original)
        for lane in N.PUBLIC:
            build, gate = report["package_builds"][lane], report["package_gates"][lane]
            for phase in ("native-unit-tests", "complete-public-entry-contract"):
                row = next(row for row in gate["commands"] if row["phase"] == phase)
                self.assertEqual(row["environment"]["PATH"], build["commands"][0]["environment"]["PATH"])
                self.assertEqual(row["environment"]["RUSTC"], str(Path(build["sdk"]) / "bin/rustc"))
            self.assertTrue(all(not Path(path).is_relative_to(Path(gate["native_oracle"]["sdk"]))
                                for path in report["tool_paths"].values()))
        self.assertIn("commit-hash: " + N.S.RUST, report["rustc_version"])

    def test_oracle_cannot_use_unbound_unit_binary(self):
        def mutate(receipt):
            receipt["unit_binaries"]["lust-core"]["path"] = "/synthetic-wrong/unit"
            next(row for row in receipt["commands"] if row["phase"] == "no-change-resolve-oracle")["argv"][0] = "/synthetic-wrong/unit"
        with self.assertRaisesRegex(ValueError, "Cargo"): self.verify_gate(self.gate_corruption(mutate))

    def test_native_export_cannot_switch_source(self):
        def mutate(receipt): next(row for row in receipt["commands"] if row["phase"] == "native-corpus-export")["argv"][2] = "/synthetic-wrong/source"
        with self.assertRaisesRegex(ValueError, "export command"): self.verify_gate(self.gate_corruption(mutate))

    def test_native_differential_cannot_use_a_filtered_corpus(self):
        def mutate(receipt): next(row for row in receipt["commands"] if row["phase"] == "ordinary-native-differential")["argv"].append("one-case-only")
        with self.assertRaises(ValueError): self.verify_gate(self.gate_corruption(mutate))

    def test_native_differential_cannot_use_a_different_release_core(self):
        def mutate(receipt): receipt["differential"]["installed_core"]["sha256"] = "f" * 64
        with self.assertRaises(ValueError): self.verify_gate(self.gate_corruption(mutate))

    def test_public_entry_gate_cannot_be_filtered(self):
        def mutate(receipt): receipt["commands"][-1]["argv"].append("one-test-only")
        with self.assertRaises(ValueError): self.verify_gate(self.gate_corruption(mutate))


class HistoricalWorkflowControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Frozen workflow source from cb0721d49be95ec3f34a113c2354bc9d2a38bbf3.
        # These controls replay historical admission/status logic as test data;
        # the active workflow only replays evidence and never builds a compiler.
        data = (HERE / "fixtures/warm-native-workflow-cb0721d.txt").read_bytes()
        assert hashlib.sha256(data).hexdigest() == "d91ceb68a2493e78b6a87f3983782a6ff1e9f3cd78a4e56d828ef6cbd449e2bf"
        cls.text = data.decode()
        starts = list(re.finditer(r"^      - (?:name|uses): .+$", cls.text, re.M))
        cls.blocks = [cls.text[match.start():starts[i+1].start() if i+1 < len(starts) else len(cls.text)] for i, match in enumerate(starts)]
        cls.steps = {}
        for block in cls.blocks:
            identity = re.search(r"^        id: ([a-z_]+)$", block, re.M)
            if identity: cls.steps[identity.group(1)] = block

    def test_both_builds_and_all_qualification_gates_precede_timing(self):
        self.assertEqual(list(self.steps), ["baseline_build", "candidate_build", "baseline_gates", "candidate_gates", "pristine_build", "comparison"])
        for key, block in self.steps.items():
            self.assertNotIn("\n        if:", block, key)
            self.assertNotIn("continue-on-error", block, key)
        self.assertIn("ref: " + N.BASELINE, self.text)
        self.assertIn("timeout-minutes: 180", self.text)

    def test_each_builder_and_gate_has_its_own_scope(self):
        for name, lane in (("baseline", "baseline"), ("candidate", "entry")):
            build, gate = self.steps[name + "_build"], self.steps[name + "_gates"]
            self.assertIn("build-package --lane " + lane, build)
            self.assertIn(f'--built "$RUNNER_TEMP/native-{name}-built" --output "$RUNNER_TEMP/native-{name}-build"', build)
            self.assertIn("qualify-package --lane " + lane, gate)
            self.assertIn(f'--build-receipt "$RUNNER_TEMP/native-{name}-build"', gate)
            self.assertIn(f'--output "$RUNNER_TEMP/native-{name}-gates"', gate)

    def test_native_oracle_is_installed_separately_and_used_only_for_gates(self):
        self.assertIn("RUSTUP_TOOLCHAIN: nightly-2026-11-09", self.text)
        self.assertIn("rustup toolchain install 1.99.0 --profile minimal", self.text)
        for name in ("baseline", "candidate"):
            self.assertIn('--native-oracle-sdk "$(rustup run 1.99.0 rustc --print sysroot)"', self.steps[name + "_gates"])
            self.assertNotIn("native-oracle", self.steps[name + "_build"])
            self.assertIn('--sdk "$(rustc --print sysroot)"', self.steps[name + "_build"])
        self.assertNotIn("rustup default", self.text)

    def test_failed_setup_and_target_miss_artifacts_are_always_retained(self):
        upload = next(block for block in self.blocks if "actions/upload-artifact@" in block)
        self.assertIn("if: always()", upload)
        for fragment in ("native-*-build/receipt.json", "native-*-build/commands", "native-*-built/logs", "native-*-gates/commands", "native-warm-comparison/raw.jsonl", "native-warm-comparison/samples.csv"):
            self.assertIn(fragment, upload)
        for fragment in ("native-warm-measurement.status", "native-warm-assessment/assessment.json", "native-warm-assessment/README.md"):
            self.assertIn(fragment, upload)

    def test_practical_assessment_is_separate_and_has_no_ignored_failure(self):
        block, = [block for block in self.blocks if "Assess practical warm performance after full evidence replay" in block]
        self.assertIn("if: always() && steps.comparison.outputs.completed == 'true'", block)
        self.assertIn("python3 tools/assess-warm-performance.py", block)
        self.assertIn('--measurement-status "$RUNNER_TEMP/native-warm-measurement.status"', block)
        self.assertNotIn("continue-on-error", self.text)
        self.assertNotIn("|| true", block)

    def comparison_script(self):
        block = self.steps["comparison"].split("        run: |\n", 1)[1]
        return "\n".join(line[10:] if line.startswith("          ") else line for line in block.splitlines()) + "\n"

    def test_embedded_measurement_shell_is_well_formed(self):
        result = subprocess.run(["bash", "-n"], input=self.comparison_script(), text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def run_status_probe(self, completed, code):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shim = root / "bin/python3"
            shim.parent.mkdir()
            shim.write_text(f"#!{sys.executable}\n" +
                "import json, os, pathlib, sys\n" +
                "if len(sys.argv)>2 and sys.argv[2]=='measure':\n" +
                " p=pathlib.Path(sys.argv[sys.argv.index('--output')+1]); p.mkdir()\n" +
                f" (p/'report.json').write_text(json.dumps({{'status': {'completed' if completed else 'failed'!r}}}))\n" +
                f" raise SystemExit({code})\n" +
                f"os.execv({sys.executable!r},[{sys.executable!r}]+sys.argv[1:])\n")
            shim.chmod(0o755)
            env = dict(os.environ, PATH=str(shim.parent) + os.pathsep + os.environ.get("PATH", ""), RUNNER_TEMP=str(root), GITHUB_WORKSPACE=str(root / "source"), GITHUB_OUTPUT=str(root / "outputs"))
            result = subprocess.run(["bash", "-c", self.comparison_script()], env=env, capture_output=True)
            return result.returncode, (root / "outputs").read_text(), (root / "native-warm-measurement.status").read_text(), result.stderr

    def test_complete_target_miss_retains_exit_two_for_separate_assessment(self):
        code, outputs, status, error = self.run_status_probe(True, 2)
        self.assertEqual((code, outputs, status), (0, "completed=true\nmeasurement_status=2\n", "2\n"), error)

    def test_incomplete_failure_stays_failed_and_not_replayable(self):
        code, outputs, status, error = self.run_status_probe(False, 1)
        self.assertEqual((code, outputs, status), (1, "completed=false\nmeasurement_status=1\n", "1\n"), error)

    def test_unexpected_exit_is_not_normalized_despite_completed_report(self):
        for original in (1, 3, 124):
            with self.subTest(original=original):
                code, outputs, status, error = self.run_status_probe(True, original)
                self.assertEqual(code, original, error)
                self.assertEqual(status, str(original) + "\n")

    def test_zero_exit_with_incomplete_report_is_not_success(self):
        code, outputs, status, error = self.run_status_probe(False, 0)
        self.assertEqual((code, outputs, status), (1, "completed=false\nmeasurement_status=0\n", "0\n"), error)


if __name__ == "__main__":
    unittest.main()
