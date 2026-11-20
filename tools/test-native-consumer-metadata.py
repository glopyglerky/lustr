#!/usr/bin/env python3
"""Qualify genuine metadata from the native completed-record producer.

The candidate receives the SDK directory for immutable artifacts and LLVM
utilities. It must not invoke or link its Rust compiler to produce the native
artifact. Independent original readers and callers run only after that artifact
exists. Default qualification requires a completely accounted exec trace;
--functional-only records that missing proof explicitly.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import subprocess
import sys
import traceback


ROOT = Path(__file__).resolve().parents[1]
SDK_RECEIPT = "4e400b263bf9923616dea44ead2d77213703177d3bfaf6becd6a19a4c5a657a7"
CFG_VERSION = "1.102.0-nightly (5b7b37aef 2026-11-08)"
ENCODING_CONTROLS = [
    "missing-emitted-instance", "duplicate-emitted-instance", "foreign-emitted-instance",
    "malformed-emitted-symbol", "duplicate-emitted-symbol", "duplicate-definition",
    "duplicate-call-edge", "foreign-call-edge", "empty-rlib-link-members",
    "duplicate-rlib-link-member", "malformed-rlib-link-member",
]
SOURCE = b"""#![no_std]
pub fn transform(value: u64) -> u64 { helper(value) ^ 85u64 }
fn helper(value: u64) -> u64 { value & 255u64 }
pub fn combine(left: u64, right: u64) -> u64 { left | right }
"""
CALLER = b"""fn main() {
    let transform: fn(u64) -> u64 = consumer::transform;
    let combine: fn(u64, u64) -> u64 = consumer::combine;
    for value in (0u64..65536).chain([u64::MAX, 1u64 << 63]) {
        assert_eq!(consumer::transform(value), (value & 255) ^ 85);
        assert_eq!(transform(value), (value & 255) ^ 85);
        assert_eq!(consumer::combine(value, 0x101), value | 0x101);
        assert_eq!(combine(value, 0x101), value | 0x101);
    }
    println!("rust-metadata-results=262152");
}
"""
INVALID_PRIVATE = b"#![no_std]\npub fn visible(value:u64)->u64 { value }\nfn hidden(value:u64)->u64 { true }\n"


def require(condition, reason):
    if not condition:
        raise AssertionError(reason)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_identity(path):
    path = Path(path)
    with path.open("rb") as source:
        before = os.fstat(source.fileno())
        digest = hashlib.file_digest(source, "sha256").hexdigest()
        after = os.fstat(source.fileno())
    stamp = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns)
    require(stamp(before) == stamp(after), "tool changed during identity read: " + str(path))
    return {"path": str(path), "sha256": digest, "size": after.st_size}


class Commands:
    def __init__(self, root):
        self.root, self.records = root, []

    def run(self, name, argv, env, expected=0):
        argv = [str(value) for value in argv]
        record = {"name": name, "argv": argv, "cwd": str(self.root),
                  "environment": {name: env[name] for name in ["PATH", "LD_LIBRARY_PATH", "RUSTC", "RUSTC_WRAPPER",
                      "RUSTC_WORKSPACE_WRAPPER", "LUSTR_FORBIDDEN_LOG", "RUSTC_FORCE_RUSTC_VERSION"] if name in env}}
        self.records.append(record)
        process = subprocess.Popen(argv, cwd=self.root, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = process.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        for stream, data in [("stdout", stdout), ("stderr", stderr)]:
            path = self.root / (name + "." + stream)
            with path.open("xb") as target:
                target.write(data)
            record[stream] = path.name
            record[stream + "_sha256"] = sha(data)
        record.update(status=process.returncode, timed_out=timed_out)
        (self.root / "commands.json").write_text(json.dumps(self.records, indent=2) + "\n")
        require(not timed_out and process.returncode == expected,
                f"{name}: expected exit {expected}, got {process.returncode}; exact streams preserved")
        if expected == 0:
            require(not stderr, name + ": unexpected successful stderr; exact stream preserved")
        return stdout, stderr


def traced_executables(path, allowed, candidate):
    """Account for every traced attempt, including interleaved split calls.

    The fixed campaign invokes absolute existing paths. An unknown syscall
    format, missing resume, failed undeclared attempt or shortened path cannot
    become proof that no other compiler was invoked.
    """
    pending, attempts, executed = {}, [], []
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        prefix = re.fullmatch(r'\s*(?:(?:\[pid\s+(\d+)\]|(\d+))\s+)?(.*)', line)
        require(prefix is not None, "unparsed candidate trace prefix")
        pid, record = prefix.group(1) or prefix.group(2) or "entry", prefix.group(3)
        require("execveat(" not in record and not record.startswith("<... execveat resumed>"),
                "unrepresented execveat command in candidate trace")
        resumed = re.fullmatch(r'<\.\.\. execve resumed>(.*)', record)
        if resumed:
            require(pid in pending, "candidate trace has an unmatched execve resume")
            record = pending.pop(pid) + resumed.group(1)
        elif record.startswith("execve(") and record.endswith(" <unfinished ...>"):
            require(pid not in pending, "candidate trace overwrote an unfinished execve")
            pending[pid] = record.removesuffix(" <unfinished ...>")
            continue
        elif not record.startswith("execve("):
            require(re.fullmatch(r'\+\+\+ exited with [0-9]+ \+\+\+', record) is not None
                    or re.fullmatch(r'\+\+\+ killed by SIG[A-Z0-9]+(?: \(core dumped\))? \+\+\+', record) is not None
                    or re.fullmatch(r'--- SIG[A-Z0-9]+ .* ---', record) is not None,
                    "unparsed candidate trace record: " + record)
            require(pid not in pending or record.startswith("--- SIG"),
                    "candidate exited with an unfinished execve")
            continue
        else:
            require(pid not in pending, "candidate trace started a second execve before resume")
        complete = re.fullmatch(r'execve\((.*)\)\s+=\s+(0|-1 [A-Z][A-Z0-9_]* \([^\n]*\))', record)
        require(complete is not None, "incomplete or unparsed candidate execve record: " + record)
        filename = re.match(r'("(?:[^"\\]|\\.)*"),\s', complete.group(1))
        require(filename is not None, "candidate execve path is missing or shortened")
        try:
            raw = Path(json.loads(filename.group(1)))
            require(raw.is_absolute(), "candidate execve path is not absolute")
            actual = raw.resolve(strict=True)
        except (ValueError, OSError) as error:
            raise AssertionError("candidate execve path cannot be accounted for") from error
        require(actual in allowed, "candidate attempted an undeclared process: " + str(actual))
        status = 0 if complete.group(2) == "0" else -1
        attempts.append({"pid": pid, "path": str(actual), "status": status})
        if status == 0:
            executed.append(str(actual))
    require(not pending, "candidate trace ends with an unfinished execve")
    require(executed and executed[0] == str(candidate)
            and executed.count(str(candidate)) == 1,
            "candidate trace lacks its exact single initial qualifier execution")
    return {"complete": True, "attempts": attempts, "executed": executed}


def source_error(stderr, source, code):
    messages = [json.loads(line) for line in stderr.splitlines() if line]
    errors = [message for message in messages if message.get("level") == "error" and isinstance(message.get("code"), dict)]
    require(any(message["code"]["code"] == code and any(span.get("is_primary") and
                Path(span["file_name"]).resolve() == source.resolve() for span in message.get("spans", [])) for message in errors),
            "original rejection lacks the required source-bound diagnostic " + code)
    require(not any("internal compiler error" in message.get("message", "").lower() for message in messages),
            "original compiler crash cannot count as source rejection")


def native_private_rejection(result, source, output):
    # Frozen observed native diagnostic from controls-v3/invalid-private in
    # evidence/native-consumer-records-20261120, also derived from the pinned
    # check::mismatch L0110 path and diag::render. Only its absolute path varies.
    expected = (
        "error[L0110]: mismatched types: expected `u64`, found `bool`\n"
        f"  --> {source}:3:29\n"
        "   | fn hidden(value:u64)->u64 { true }\n"
        "   |                             ^^^^\n"
        "   = fix: Change the value to type `u64`, or change the expected type.\n\n"
    )
    require(source.read_bytes() == INVALID_PRIVATE, "invalid-private source changed")
    require(result.get("phase") == "native-prepare" and result.get("error") == expected,
            "native private-body rejection lacks its exact type diagnostic and original source span")
    require(not any((output / name).exists() for name in
                    ("libconsumer.rlib", "staging/consumer.rmeta", "staging/native.o")),
            "rejected private body published metadata or an object")


def campaign(args, root, report, commands):
    sdk, candidate, clang = args.sdk.resolve(strict=True), args.candidate.resolve(strict=True), args.clang.resolve(strict=True)
    receipt = args.receipt.resolve(strict=True)
    require(sha(receipt.read_bytes()) == SDK_RECEIPT, "receipt is not the exact reviewed construction output")
    sdk_record = json.loads(receipt.read_bytes())
    tools = {"candidate": file_identity(candidate), "clang": file_identity(clang)}
    for name, expected in sdk_record["sdk_tools"].items():
        actual = file_identity(sdk / expected["path"])
        require(actual["sha256"] == expected["sha256"] and actual["size"] == expected["size"], "SDK tool differs: " + name)
        tools[name] = actual
    llvm = sdk / "lib/rustlib/x86_64-unknown-linux-gnu/bin"
    for name in ["llvm-ar", "llvm-objcopy"]:
        tools[name] = file_identity(llvm / name)
    report["tool_identities"] = tools
    report["receipt_sha256"] = SDK_RECEIPT
    report["source_sha256"] = sha(SOURCE)
    report["test_source_sha256"] = sha(Path(__file__).read_bytes())
    report["git_head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    source = root / "consumer.rs"
    source.write_bytes(SOURCE)
    env = os.environ.copy()
    env.pop("RUSTC_FORCE_RUSTC_VERSION", None)
    # Host Clang remains the original research backend. The caller selects it
    # explicitly; the candidate's unchanged Command("clang") finds this name.
    traps = root / "compiler-traps"
    traps.mkdir()
    (traps / "clang").symlink_to(clang)
    trap = traps / "rustc"
    trap.write_text('#!/bin/sh\nprintf "%s\\n" "$0" >> "$LUSTR_FORBIDDEN_LOG"\nexit 91\n')
    trap.chmod(0o755)
    for name in ["cargo", "rustc-full", "rustc-reference"]:
        (traps / name).symlink_to(trap)
    forbidden = root / "forbidden-compiler.log"
    native_env = {**env, "PATH": str(traps) + os.pathsep + env["PATH"], "RUSTC": str(trap),
                  "RUSTC_WRAPPER": str(trap), "RUSTC_WORKSPACE_WRAPPER": str(trap),
                  "LUSTR_FORBIDDEN_LOG": str(forbidden)}
    commands.run("compiler-trap-control", [trap], native_env, expected=91)
    require(forbidden.read_bytes() == (str(trap) + "\n").encode(), "compiler trap control did not execute")
    forbidden.rename(root / "compiler-trap-control.log")
    report["controls"].append("compiler-trap-control")
    rustc = sdk / "bin/rustc"
    original, _ = commands.run("oracle-identity", [rustc, "-vV"], env)
    require(b"commit-hash: 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494\n" in original, "wrong independent compiler")
    commands.run("clang-identity", [clang, "--version"], env)
    allowed = {candidate, clang, (llvm / "llvm-ar").resolve(), (llvm / "llvm-objcopy").resolve()}
    strace = None if args.functional_only else shutil.which("strace")
    require(args.functional_only or strace is not None, "actual exec trace requires strace")
    report["candidate_invocations"] = []

    def produce(name, source=source, overrides=None, expected=0, preserved=None, used_receipt=receipt):
        output = root / name
        output.mkdir()
        if preserved is not None:
            (output / "libconsumer.rlib").write_bytes(preserved)
        argv = [candidate, source, output, used_receipt, sdk]
        trace = root / (name + ".trace")
        if strace:
            argv = [strace, "-f", "-s", "65535", "-e", "trace=execve,execveat", "-o", trace, *argv]
        chosen_env = {**native_env, **(overrides or {})}
        stdout, _ = commands.run(name, argv, chosen_env, expected=expected)
        require(not forbidden.exists(), name + ": candidate invoked a Rust compiler trap")
        result = json.loads((output / "report.json").read_bytes())
        require(result["passed"] == (expected == 0), name + ": report/status disagreement")
        execution = traced_executables(trace, allowed, candidate) if strace else None
        report["candidate_invocations"].append({"name": name, "status": expected, "phase": result["phase"], "execution": execution,
            "report_sha256": sha((output / "report.json").read_bytes())})
        if expected == 0:
            require(stdout == ("native metadata artifact: " + str(output / "libconsumer.rlib") + "\n").encode(), "unexpected candidate stdout")
        return output, result

    output, native = produce("accepted")
    require(native["native_source_sha256"] == sha(SOURCE), "native producer changed the original source")
    backend_commands = list((output / "staging").glob(".lustr-object-*/backend.command"))
    require(len(backend_commands) == 1, "native generation lacks its one captured backend command")
    backend = shlex.split(backend_commands[0].read_text())
    require(backend[:3] == ["clang", "--no-default-config", "-fno-stack-protector"],
            "native metadata backend did not fix the represented mitigation/configuration context")
    report["controls"].append("explicit-native-backend-context")
    require(native["encoding_controls"] == ENCODING_CONTROLS, "actual native-record mutation controls are incomplete")
    report["controls"].extend(ENCODING_CONTROLS)
    require([function["name"] for function in native["functions"]] == ["transform", "helper", "combine"], "complete declaration inventory changed")
    require([function["public"] for function in native["functions"]] == [True, False, True], "actual source visibility changed")
    require(native["functions"][0]["calls"] == [native["functions"][1]["native_id"]], "private helper call is missing from native records")
    require({tuple(row["native_id"]) for row in native["symbol_mapping"]} == {tuple(row["native_id"]) for row in native["functions"]}, "emitted identity mapping is incomplete")
    require(native["archive_members"] == ["lib.rmeta", "lib.rmeta-link", "consumer.o"], "archive inventory differs")
    require(native["archive_controls"] == ["missing-object-member", "duplicate-object-member"],
            "actual malformed-archive controls are incomplete")
    report["controls"].extend(native["archive_controls"])
    report["controls"].append("native-record-object-and-archive-join")
    root_output, _ = commands.run("original-metadata-root", [rustc, "-Zls=root", output / "staging/consumer.rmeta"], env)
    spec = importlib.util.spec_from_file_location("metadata_sdk_construction", ROOT / "tools/build-metadata-sdk-receipt.py")
    construction = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(construction)
    original_root = construction.parse_root(root_output)
    require(original_root["stable_crate_id_hex"] == native["stable_crate_id"] == "4d850957defa2873", "original stable crate identity differs")
    require(original_root["svh_le_hex"] == native["crate_hash_le_hex"], "original reader sees a different native crate hash")
    require([(d["name"], d["kind_ordinal"], d["is_private"], d["host_hash_le_hex"]) for d in original_root["dependencies"]]
            == [("core", 2, False, None), ("compiler_builtins", 2, True, None)], "original reader sees the wrong SDK closure")
    report["controls"].append("original-metadata-root")
    rlib = output / "libconsumer.rlib"
    caller = root / "caller.rs"
    caller.write_bytes(CALLER)
    program = root / "caller"
    commands.run("caller-compile", [rustc, "--edition=2024", "-Cpanic=abort", "-Copt-level=0", "-Dwarnings", caller, "--extern", "consumer=" + str(rlib), "-o", program], env)
    stdout, _ = commands.run("caller-run", [program], env)
    require(stdout == b"rust-metadata-results=262152\n", "caller results do not match exact independent assertions")
    report["controls"].append("direct-and-function-value-results")
    report["checked_runtime_results"] = 262152
    # A disabled mitigation must not be omitted or represented as stronger
    # protection. This separate no_std consumer requests stronger protection;
    # its small dependency closure avoids the original five-errors-per-kind
    # reporting cap masking the required consumer-specific diagnostic.
    protected_caller = root / "stronger-mitigation.rs"
    protected_caller.write_bytes(b"#![no_std]\npub fn invoke(value:u64)->u64 { consumer::transform(value) }\n")
    protected_output = root / "stronger-mitigation.rmeta"
    _, stderr = commands.run("stronger-mitigation-refused", [rustc, "--edition=2024", "-Cpanic=abort",
        "-Zstack-protector=all", "-Dpartial_stack_protector", "--crate-type=rlib", "--emit=metadata", "--error-format=json",
        protected_caller, "--extern", "consumer=" + str(rlib), "-o", protected_output], env, expected=1)
    diagnostics = [json.loads(line) for line in stderr.splitlines() if line]
    require(any(message.get("level") == "error" and isinstance(message.get("code"), dict)
                and message["code"]["code"] == "partial_stack_protector"
                and "consumer" in message.get("message", "") for message in diagnostics),
            "stronger mitigation refusal lacks the original dependency-specific diagnostic")
    require(not protected_output.exists(), "incompatible mitigation published caller metadata")
    report["controls"].append("stronger-mitigation-refused")
    for name, text, code in [
        ("private-call", b"fn main() { let _ = consumer::helper(1); }\n", "E0603"),
        ("private-function-value", b"fn main() { let _: fn(u64)->u64 = consumer::helper; }\n", "E0603"),
        ("wrong-argument-type", b"fn main() { let _ = consumer::transform(true); }\n", "E0308"),
        ("wrong-arity", b"fn main() { let _ = consumer::combine(1); }\n", "E0061"),
        ("wrong-function-value-type", b"fn main() { let _: fn(u32)->u64 = consumer::transform; }\n", "E0308"),
    ]:
        file = root / (name + ".rs")
        file.write_bytes(text)
        _, stderr = commands.run(name, [rustc, "--edition=2024", "-Cpanic=abort", "--error-format=json", "--emit=metadata", file,
                                      "--extern", "consumer=" + str(rlib), "-o", root / (name + ".rmeta")], env, expected=1)
        source_error(stderr, file, code)
        require(not (root / (name + ".rmeta")).exists(), "rejected caller published metadata")
        report["controls"].append(name)
    invalid = root / "invalid-private.rs"
    invalid.write_bytes(INVALID_PRIVATE)
    rejected_output, rejected = produce("invalid-private-body", source=invalid, expected=1)
    native_private_rejection(rejected, invalid, rejected_output)
    _, stderr = commands.run("invalid-private-oracle", [rustc, "--edition=2024", "--crate-type=rlib", "--emit=metadata", "--error-format=json", invalid,
        "-o", root / "invalid-private.rmeta"], env, expected=1)
    source_error(stderr, invalid, "E0308")
    report["controls"].append("invalid-private-body")
    good = rlib.read_bytes()
    failed_output, rejected = produce("publication-failure", expected=1, preserved=good)
    require(rejected["phase"] == "publication" and "publish metadata archive" in rejected["error"], "failure did not reach actual publication boundary")
    require((failed_output / "libconsumer.rlib").read_bytes() == good, "failed publication replaced the good artifact")
    report["controls"].append("good-artifact-preserved-on-publication-failure")
    _, recovered = produce("valid-recovery")
    require(recovered["native_source_sha256"] == sha(SOURCE), "valid recovery changed the original program")
    report["controls"].append("valid-recovery")
    for name, value in [("empty-version-refused", ""), ("different-version-refused", "other-version")]:
        _, rejected = produce(name, overrides={"RUSTC_FORCE_RUSTC_VERSION": value}, expected=1)
        require(rejected["phase"] == "setup" and "RUSTC_FORCE_RUSTC_VERSION" in rejected["error"], "unrepresented crate identity override was admitted")
        report["controls"].append(name)
    _, exact = produce("exact-version-context", overrides={"RUSTC_FORCE_RUSTC_VERSION": CFG_VERSION})
    require(exact["stable_crate_id"] == native["stable_crate_id"], "equivalent version context changed original stable identity")
    report["controls"].append("exact-version-context")
    changed = root / "changed-receipt.json"
    changed.write_bytes(receipt.read_bytes() + b" ")
    _, rejected = produce("changed-receipt-refused", used_receipt=changed, expected=1)
    require(rejected["phase"] == "setup" and "reviewed exact identity" in rejected["error"], "unbound SDK receipt was admitted")
    report["controls"].append("changed-receipt-refused")
    require(source.read_bytes() == SOURCE and sha(receipt.read_bytes()) == SDK_RECEIPT, "qualification input changed")
    for name, identity in tools.items():
        require(file_identity(identity["path"]) == identity, "qualification tool changed: " + name)
    report["artifact_sha256"] = sha(good)
    report["compiler_traps_unused"] = not forbidden.exists()
    report["exec_trace_qualified"] = bool(report["candidate_invocations"]) and all(
        item["execution"] is not None and item["execution"]["complete"] for item in report["candidate_invocations"])
    report["functional_passed"] = True
    report["passed"] = report["exec_trace_qualified"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--sdk", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, default=ROOT / "docs/compiler-set/evidence/metadata-sdk-receipt-20261120/receipt.json")
    parser.add_argument("--clang", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--functional-only", action="store_true", help="local evidence only: leaves actual execution tracing explicitly unqualified")
    args = parser.parse_args()
    root = args.output.absolute()
    root.mkdir(mode=0o700)
    commands = Commands(root)
    report = {"schema": "native-consumer-metadata-campaign-v1", "passed": False, "functional_passed": False,
              "exec_trace_qualified": False, "full_compiler_qualification": False, "retained_public_integration": False,
              "functional_only": args.functional_only, "controls": []}
    try:
        campaign(args, root, report, commands)
    except Exception as error:
        report["failure"] = str(error)
        (root / "failure.txt").write_text(traceback.format_exc())
    report["commands"] = commands.records
    (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: value for key, value in report.items() if key in ["passed", "functional_passed", "exec_trace_qualified", "controls", "failure"]}))
    return 0 if report["functional_passed"] and (args.functional_only or report["exec_trace_qualified"]) else 1


if __name__ == "__main__":
    sys.exit(main())
