#!/usr/bin/env python3
"""Construct an SDK metadata receipt from a fixed reference, never user source.

This tool belongs to compiler construction/qualification. It is not imported by
the product or invoked to select an owner or encode a consumer request.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tomllib


RUST_REVISION = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494"
TARGET = "x86_64-unknown-linux-gnu"
METADATA_HEADER = b"rust\x00\x00\x00\x0c"
KINDS = {"MacrosOnly": 0, "Conditional": 1, "Unconditional": 2}
# Exact fixed construction reference previously read by the original decoder.
REFERENCE = b"""#![no_std]
pub fn transform(value: u64) -> u64 { helper(value) ^ 85u64 }
fn helper(value: u64) -> u64 { value & 255u64 }
pub fn combine(left: u64, right: u64) -> u64 { left | right }
"""
PROFILE_ARGS = ["--edition=2024", "--crate-name", "consumer", "--crate-type=rlib",
                "--emit=metadata,link", "--target=" + TARGET,
                "-Cmetadata=native_consumer_v1", "-Cpanic=abort", "-Copt-level=0",
                "-Cdebuginfo=0", "-Coverflow-checks=on", "-Cdebug-assertions=on"]
NAME = r"(?P<name>[A-Za-z_][A-Za-z_0-9]*)(?P<extra>-[0-9a-f]{16})?"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def svh_bytes(display):
    require(re.fullmatch(r"[0-9a-f]{32}", display), "malformed original SVH")
    # Svh::Display uses Fingerprint::as_u128; its serialized bytes are LE.
    return int(display, 16).to_bytes(16, "little").hex()


def parse_root(data):
    lines = data.decode("utf-8", errors="strict").splitlines()
    while lines and not lines[-1]:
        lines.pop()
    require(len(lines) >= 11 and lines[0] == "Crate info:", "missing original metadata root")
    patterns = [
        "name " + NAME,
        r"hash (?P<svh>[0-9a-f]{32}) stable_crate_id StableCrateId\((?P<stable>[0-9]+)\)",
        r"proc_macro (?P<proc_macro>true|false)",
        r"triple (?P<target>[A-Za-z0-9_-]+)",
        r"edition (?P<edition>2015|2018|2021|2024)",
        r"symbol_mangling_version (?P<symbol_mangling>V0|Legacy|Hashed)",
        r"required_panic_strategy (?P<required_panic>None|Some\(Abort\)|Some\(Unwind\)) panic_in_drop_strategy (?P<panic_in_drop>Abort|Unwind)",
        r"has_global_allocator (?P<global_allocator>true|false) has_alloc_error_handler (?P<alloc_error_handler>true|false) has_panic_handler (?P<panic_handler>true|false) has_default_lib_allocator (?P<default_lib_allocator>true|false)",
        r"compiler_builtins (?P<compiler_builtins>true|false) needs_allocator (?P<needs_allocator>true|false) needs_panic_runtime (?P<needs_panic_runtime>true|false) no_builtins (?P<no_builtins>true|false) panic_runtime (?P<panic_runtime>true|false) profiler_runtime (?P<profiler_runtime>true|false)",
    ]
    root = {}
    for line, pattern in zip(lines[1:10], patterns):
        match = re.fullmatch(pattern, line)
        require(match is not None, "unrecognized original metadata root field: " + line)
        root.update(match.groupdict())
    require(lines[10] == "=External Dependencies=", "missing original dependency table")
    stable = int(root.pop("stable"))
    require(0 <= stable < 1 << 64, "StableCrateId exceeds its original width")
    root["stable_crate_id_hex"] = f"{stable:016x}"
    root["extra_filename"] = root.pop("extra") or ""
    root["svh_le_hex"] = svh_bytes(root["svh"])
    dependencies, names = [], set()
    for line in lines[11:]:
        # This fixed Linux target has no host/proc-macro dependency context.
        match = re.fullmatch(r"(?P<number>[1-9][0-9]*) " + NAME +
                             r" hash (?P<svh>[0-9a-f]{32}) host_hash None kind (?P<kind>MacrosOnly|Conditional|Unconditional) (?P<privacy>private|public)", line)
        require(match is not None, "unrepresented original dependency row: " + line)
        row = match.groupdict()
        require(int(row["number"]) == len(dependencies) + 1, "dependency numbering is not ordered and complete")
        require(row["name"] not in names, "duplicate dependency name")
        names.add(row["name"])
        dependencies.append({"crate_num": int(row["number"]), "name": row["name"],
                             "svh_le_hex": svh_bytes(row["svh"]), "host_hash_le_hex": None,
                             "kind": row["kind"], "kind_ordinal": KINDS[row["kind"]],
                             "is_private": row["privacy"] == "private",
                             "extra_filename": row["extra"] or ""})
    root["dependencies"] = dependencies
    return root


def artifact(root, relative):
    path = root / relative
    resolved = path.resolve(strict=True)
    require(resolved.is_relative_to(root), "SDK artifact escapes its root")
    with resolved.open("rb") as source:
        before = os.fstat(source.fileno())
        require(before.st_size > 0, "empty SDK artifact")
        digest = hashlib.file_digest(source, "sha256").hexdigest()
        after = os.fstat(source.fileno())
    stamp = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns)
    require(stamp(before) == stamp(after), "SDK artifact changed while hashing")
    require(path.resolve(strict=True) == resolved, "SDK artifact target changed while hashing")
    return {"path": str(relative), "size": before.st_size, "sha256": digest}


def check_header(path, root, cfg_version):
    with path.open("rb") as source:
        data = source.read(512)
    require(data.startswith(METADATA_HEADER), "not the pinned version-12 metadata input")
    require(data[24:40].hex() == root["svh_le_hex"], "metadata header SVH differs from original reader")
    version = ("rustc " + cfg_version).encode("utf-8")
    # The pinned version string uses the single-byte usize/LEB128 form.
    require(len(version) < 128 and data[40:41] == bytes([len(version)]), "unrepresented metadata version-string length")
    require(data[41:41 + len(version)] == version and data[41 + len(version)] == 0xc1,
            "metadata version string differs from the declared SDK")
    size = path.stat().st_size
    positions = [int.from_bytes(data[start:start + 8], "little") for start in [8, 16]]
    require(all(40 < position < size for position in positions), "metadata root positions are outside the artifact")
    return {"magic_hex": data[:8].hex(), "svh_le_hex": data[24:40].hex(),
            "rustc_version": version.decode(), "root_position": positions[0],
            "unhashed_root_position": positions[1]}


class Commands:
    def __init__(self, output):
        self.output, self.records = output, []

    def run(self, name, argv):
        prefix = f"{len(self.records):02d}-{name}"
        record = {"argv": [str(arg) for arg in argv], "cwd": str(self.output),
                  "environment": {"PATH": os.defpath, "LANG": "C", "LC_ALL": "C"}}
        (self.output / (prefix + ".command.json")).write_text(json.dumps(record, indent=2) + "\n")
        process = subprocess.Popen(record["argv"], cwd=self.output, env=record["environment"],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = process.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        for name, data in [("stdout", stdout), ("stderr", stderr)]:
            (self.output / (prefix + "." + name)).write_bytes(data)
            record[name] = prefix + "." + name
            record[name + "_sha256"] = sha(data)
        record.update(status=process.returncode, timed_out=timed_out)
        self.records.append(record)
        (self.output / (prefix + ".status.json")).write_text(json.dumps(record, indent=2) + "\n")
        require(not timed_out and process.returncode == 0, f"{prefix} failed; original command and streams preserved")
        require(not stderr, f"{prefix} produced unexpected diagnostics")
        return stdout


def build(sdk, output, commands):
    repo = Path(__file__).resolve().parents[1]
    pin_bytes = (repo / "upstream/bluefe-core.pin").read_bytes()
    pin = tomllib.loads(pin_bytes.decode("utf-8"))
    require(pin["source"] == RUST_REVISION, "source pin differs from reviewed metadata reader")
    paths = {"rustc": Path("bin/rustc"), "driver": Path("lib") / pin["driver_file"],
             "llvm": Path("lib") / pin["llvm_file"]}
    before = {name: artifact(sdk, path) for name, path in paths.items()}
    for name in paths:
        require(before[name]["sha256"] == pin[name + "_sha256"], "SDK " + name + " differs from its pin")
    lib = Path("lib/rustlib") / TARGET / "lib"
    pairs = {}
    for name in ["core", "compiler_builtins"]:
        matches = sorted((sdk / lib).glob("lib" + name + "-*.rmeta"))
        require(len(matches) == 1, "SDK metadata artifact is missing or ambiguous: " + name)
        metadata = matches[0].relative_to(sdk)
        code = metadata.with_suffix(".rlib")
        pairs[name] = {"artifact": artifact(sdk, metadata), "link_artifact": artifact(sdk, code)}
    rustc = sdk / paths["rustc"]
    version = commands.run("rustc-version", [rustc, "-vV"]).decode("utf-8").splitlines()
    require(version and version[0].startswith("rustc "), "missing compiler version")
    for field, expected in [("commit-hash", RUST_REVISION), ("host", TARGET)]:
        require([line for line in version if line.startswith(field + ": ")] == [field + ": " + expected],
                "SDK " + field + " does not match the construction context")
    cfg_version = version[0][len("rustc "):]
    source = output / "reference.rs"
    source.write_bytes(REFERENCE)
    commands.run("reference-build", [rustc, source, *PROFILE_ARGS, "--sysroot", sdk, "--out-dir", output])
    reference_path = output / "libconsumer.rmeta"
    reference = parse_root(commands.run("reference-root", [rustc, "-Zls=root", reference_path]))
    require(reference["name"] == "consumer" and reference["target"] == TARGET and reference["edition"] == "2024"
            and reference["required_panic"] == "Some(Abort)" and reference["symbol_mangling"] == "V0"
            and reference["proc_macro"] == "false", "fixed reference context changed")
    dependencies = reference["dependencies"]
    require([dep["name"] for dep in dependencies] == ["core", "compiler_builtins"], "fixed no_std dependency closure changed")
    require([(dep["kind_ordinal"], dep["is_private"], dep["host_hash_le_hex"]) for dep in dependencies]
            == [(2, False, None), (2, True, None)], "fixed no_std dependency facts changed")
    for dep in dependencies:
        pair = pairs[dep["name"]]
        metadata = sdk / pair["artifact"]["path"]
        root = parse_root(commands.run(dep["name"] + "-root", [rustc, "-Zls=root", metadata]))
        for field in ["name", "extra_filename", "svh_le_hex"]:
            require(root[field] == dep[field], "loaded SDK root differs from reference dependency: " + field)
        require(root["target"] == TARGET and root["proc_macro"] == "false", "SDK dependency target/kind differs")
        dep.update(pair)
        dep["stable_crate_id_hex"] = root["stable_crate_id_hex"]
        dep["metadata_header"] = check_header(metadata, root, cfg_version)
        dep["original_root"] = root
    reference_header = check_header(reference_path, reference, cfg_version)
    require(source.read_bytes() == REFERENCE, "fixed construction source changed")
    require((repo / "upstream/bluefe-core.pin").read_bytes() == pin_bytes, "SDK pin changed during construction")
    for name, prior in before.items():
        require(artifact(sdk, paths[name]) == prior, "SDK tool changed during construction: " + name)
    for pair in pairs.values():
        for prior in pair.values():
            require(artifact(sdk, Path(prior["path"])) == prior, "SDK dependency changed during construction")
    return {"schema": "lustr-native-metadata-sdk-v1", "rust_revision": RUST_REVISION,
            "target": TARGET, "metadata_version": 12, "cfg_version": cfg_version,
            "dependencies": dependencies, "sdk_tools": before,
            "source_pin_sha256": sha(pin_bytes), "construction_tool_sha256": sha(Path(__file__).read_bytes()),
            "construction_only": True, "user_source_read": False, "full_compiler_qualification": False,
            "profile_arguments": PROFILE_ARGS,
            "reference": {"source_sha256": sha(REFERENCE), "metadata_header": reference_header,
                          "metadata": artifact(output, Path("libconsumer.rmeta")),
                          "rlib": artifact(output, Path("libconsumer.rlib")), "original_root": reference},
            "original_source_contract": {"revision": RUST_REVISION,
                "root_reader": "compiler/rustc_metadata/src/rmeta/decoder.rs::list_crate_metadata",
                "dependency_kinds": "compiler/rustc_crate_store/src/cstore.rs::CrateDepKind",
                "svh_display": "compiler/rustc_data_structures/src/svh.rs",
                "svh_bytes": "compiler/rustc_data_structures/src/fingerprint.rs::to_le_bytes",
                "header": "compiler/rustc_metadata/src/rmeta/mod.rs"}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sdk", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new construction evidence directory")
    args = parser.parse_args()
    sdk = args.sdk.resolve(strict=True)
    output = args.output.absolute()
    require(not output.exists(), "construction output already exists; preserved without modification")
    require(not output.parent.resolve(strict=True).is_relative_to(Path(__file__).resolve().parents[1]),
            "construction output must stay outside the source checkout")
    output.mkdir(mode=0o700)
    output = output.resolve(strict=True)
    commands = Commands(output)
    report = {"schema": "lustr-metadata-sdk-construction-v1", "passed": False,
              "user_source_read": False, "full_compiler_qualification": False}
    try:
        receipt = build(sdk, output, commands)
        data = (json.dumps(receipt, indent=2, sort_keys=True) + "\n").encode()
        temporary = output / "receipt.json.pending"
        with temporary.open("xb") as target:
            target.write(data)
            target.flush()
            os.fsync(target.fileno())
        temporary.rename(output / "receipt.json")
        report.update(passed=True, receipt_sha256=sha(data))
    except Exception as error:
        report["failure"] = str(error)
    report["commands"] = commands.records
    (output / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps({k: v for k, v in report.items() if k != "commands"}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
