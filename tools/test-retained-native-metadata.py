#!/usr/bin/env python3
"""Qualify the real retained native metadata session, including narrow object reuse.

Expectations below are fixed independently of the candidate. The candidate is
one process over caller-owned pipes. Original Rust readers and callers consume
completed artifacts only after each candidate response. This bounded campaign
does not qualify the public compiler, arbitrary Rust, or a speed claim.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import selectors
import shutil
import signal
import struct
import subprocess
import sys
import time
import traceback


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("native_metadata_campaign", ROOT / "tools/test-native-consumer-metadata.py")
original = importlib.util.module_from_spec(spec)
spec.loader.exec_module(original)
require, sha, file_identity = original.require, original.sha, original.file_identity
SDK_RECEIPT = original.SDK_RECEIPT
SCHEMA = "retained-native-metadata-v1"
REPORT_SCHEMA = "retained-native-metadata-report-v1"
BASE = original.SOURCE
EDIT = BASE.replace(b"value & 255u64", b"value & 511u64")
SPAN = BASE.replace(b"pub fn transform", b"\n\npub fn transform", 1)
REORDER = b"#![no_std]\n" + BASE.splitlines(keepends=True)[3] + BASE.splitlines(keepends=True)[2] + BASE.splitlines(keepends=True)[1]
STACK = b"pub fn stack_args(a:u64,b:u64,c:u64,d:u64,e:u64,f:u64,g:u64,h:u64)->u64 { a & 1u64 | b & 2u64 | c & 4u64 | d & 8u64 | e & 16u64 | f & 32u64 | g & 64u64 | h & 128u64 }\n"
INSERT = REORDER + STACK
SIGNATURE = REORDER.replace(b"combine(left: u64, right: u64) -> u64 { left | right }",
                            b"combine(left: u64, right: u64, mask: u64) -> u64 { left ^ right ^ mask }")
INVALID = REORDER.replace(b"{ value & 255u64 }", b"{ true }")
TYPE_GAP = REORDER.replace(b"combine(left: u64, right: u64) -> u64", b"combine(left: u16, right: u16) -> u16")
REORDER_EDIT = REORDER.replace(b"value & 255u64", b"value & 511u64")
CHECK_SECOND = REORDER_EDIT.replace(b"left | right", b"left ^ right")
DROP_EDIT = CHECK_SECOND.replace(b"value & 511u64", b"value & 2047u64")
DOUBLE_EDIT = REORDER.replace(b"value & 255u64", b"value & 1023u64").replace(b"left | right", b"left & right")
NAMES = frozenset(("transform", "helper", "combine"))


def case(name, source=REORDER, *, action="build", status=0, dirty=(), required=(),
         checked=(), check_required=(), noop=False, unchanged=(), mask=255,
         arity=2, stack=False, sabotage=None, combine="or", compile_dirty=None, compiled_count=None,
         artifact_cache_reuse=None, metadata_count=None, archive_count=None):
    return dict(name=name, source=source, action=action, status=status,
                dirty=frozenset(dirty), required=frozenset(required),
                checked=frozenset(checked), check_required=frozenset(check_required),
                noop=noop, unchanged=frozenset(unchanged), mask=mask, arity=arity,
                stack=stack, sabotage=sabotage, combine=combine,
                compile_dirty=frozenset(dirty if compile_dirty is None else compile_dirty), compiled_count=compiled_count,
                artifact_cache_reuse=artifact_cache_reuse, metadata_count=metadata_count, archive_count=archive_count)


# These outcomes are declared before the candidate runs. A valid u16
# signature requires narrow-integer ABI support beyond the native metadata
# caller's linked scalar path.
HISTORY = [
    case("cold", BASE, dirty=NAMES, required=NAMES, checked=NAMES, check_required=NAMES, compiled_count=3),
    case("no-op", BASE, noop=True, unchanged=NAMES),
    case("body-edit", EDIT, dirty=("helper",), required=("helper",), checked=("helper",), check_required=("helper",), unchanged=("transform", "combine"), mask=511),
    case("body-edit-no-op", EDIT, noop=True, unchanged=NAMES, mask=511),
    case("body-revert", BASE, dirty=("helper",), required=("helper",), checked=("helper",), check_required=("helper",), unchanged=("transform", "combine")),
    case("span-only", SPAN, unchanged=NAMES),
    case("reorder", REORDER, unchanged=NAMES),
    case("insert", INSERT, dirty=("stack_args",), required=("stack_args",), checked=("stack_args",), check_required=("stack_args",), unchanged=NAMES, stack=True),
    case("insert-no-op", INSERT, noop=True, unchanged=NAMES | {"stack_args"}, stack=True),
    case("remove", REORDER, unchanged=NAMES),
    case("signature", SIGNATURE, dirty=("combine",), required=("combine",), checked=("combine",), check_required=("combine",), unchanged=("transform", "helper"), arity=3, combine="xor"),
    case("signature-no-op", SIGNATURE, noop=True, unchanged=NAMES, arity=3, combine="xor"),
    case("signature-revert", REORDER, dirty=("combine",), required=("combine",), checked=("combine",), check_required=("combine",), unchanged=("transform", "helper")),
    case("invalid-private", INVALID, status=1, checked=("helper",), check_required=("helper",)),
    case("source-recovery", REORDER, dirty=("helper",), checked=("helper",), unchanged=("transform", "combine")),
    case("valid-type-gap", TYPE_GAP, status=4, checked=("combine",)),
    case("gap-recovery", REORDER, dirty=("combine",), checked=("combine",), unchanged=("transform", "helper")),
    case("check-edit", REORDER_EDIT, action="check", checked=("helper",), check_required=("helper",), mask=511),
    case("second-check-edit", CHECK_SECOND, action="check", checked=("combine",), check_required=("combine",), mask=511, combine="xor"),
    case("build-after-check", CHECK_SECOND, dirty=("helper", "combine"), required=("helper", "combine"), unchanged=("transform",), mask=511, combine="xor"),
    case("prepare-drop", DROP_EDIT, action="prepare-drop", checked=("helper",), check_required=("helper",), mask=2047, combine="xor"),
    case("build-after-drop", DROP_EDIT, dirty=("helper",), required=("helper",), unchanged=("transform", "combine"), mask=2047, combine="xor", compiled_count=1),
    case("partial-object-failure", DOUBLE_EDIT, status=3, dirty=("helper", "combine"), required=("helper", "combine"),
         checked=("helper", "combine"), check_required=("helper", "combine"), sabotage="native-object", mask=1023, combine="and", compiled_count=1),
    case("same-source-object-recovery", DOUBLE_EDIT, compile_dirty=("helper", "combine"), compiled_count=1, unchanged=("transform",), mask=1023, combine="and"),
    case("stage-failure", REORDER, status=3, dirty=("helper", "combine"), checked=("helper", "combine"), sabotage="staging"),
    case("stage-recovery", REORDER, unchanged=("transform",), artifact_cache_reuse=False, metadata_count=1, archive_count=1),
    case("publication-failure", REORDER_EDIT, status=3, dirty=("helper",), checked=("helper",), sabotage="publication", mask=511),
    case("publication-recovery", REORDER_EDIT, unchanged=("transform", "combine"), mask=511,
         artifact_cache_reuse=True, metadata_count=0, archive_count=0),
    case("recovery-no-op", REORDER_EDIT, noop=True, unchanged=NAMES, mask=511),
]
EXPECTED_CONSUMERS = [("history-" + item["name"], 65538 * (6 if item["stack"] else 4))
                      for item in HISTORY if item["status"] == 0 and item["action"] == "build"] + [
    ("fresh-fresh-comparison", 262152),
    ("sdk-retention-snapshot-cold", 262152),
    ("sdk-retention-loaded-receipt-immutable", 262152),
    ("sdk-retention-loaded-artifact-immutable", 262152),
]
EXPECTED_CONSUMER_COUNT = 25
EXPECTED_ASSERTION_TOTAL = 6815952


def validate_consumers(report):
    actual = report.get("original_consumers", [])
    require(len(EXPECTED_CONSUMERS) == EXPECTED_CONSUMER_COUNT
            and sum(count for _name, count in EXPECTED_CONSUMERS) == EXPECTED_ASSERTION_TOTAL,
            "authored consumer inventory differs from its fixed review totals")
    require([(row.get("case"), row.get("assertions")) for row in actual] == EXPECTED_CONSUMERS
            and report.get("checked_runtime_results") == EXPECTED_ASSERTION_TOTAL,
            "ordinary original consumers or runtime assertions are missing, duplicated or reclassified")
    for name, _assertions in EXPECTED_CONSUMERS:
        for suffix in ("metadata-root", "consumer-compile", "consumer-run"):
            commands = [row for row in report.get("commands", []) if row.get("name") == name + "-" + suffix]
            require(len(commands) == 1 and commands[0].get("status") == 0
                    and commands[0].get("timed_out") is False and commands[0].get("cleanup_incomplete") is False,
                    "ordinary consumer lacks one actual complete reader/compile/runtime command")


class Commands:
    """One observed command with bounded normal and failure pipe draining."""
    def __init__(self, root):
        self.root, self.records = root, []

    def run(self, name, argv, env, expected=0):
        argv = [str(value) for value in argv]
        record = {"name": name, "argv": argv, "cwd": str(self.root),
                  "environment": {key: env[key] for key in ("PATH", "LD_LIBRARY_PATH", "RUSTC", "RUSTC_WRAPPER",
                      "RUSTC_WORKSPACE_WRAPPER", "LUSTR_FORBIDDEN_LOG", "RUSTC_FORCE_RUSTC_VERSION") if key in env}}
        self.records.append(record)
        process = subprocess.Popen(argv, cwd=self.root, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   start_new_session=True)
        timed_out, incomplete = False, False
        stdout, stderr = b"", b""
        try:
            try:
                stdout, stderr = process.communicate(timeout=60)
            except subprocess.TimeoutExpired as error:
                timed_out = True
                stdout, stderr = error.output or b"", error.stderr or b""
                # A reaped leader does not establish EOF on inherited pipes.
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                try:
                    stdout, stderr = process.communicate(timeout=2)
                except subprocess.TimeoutExpired as cleanup_error:
                    incomplete = True
                    stdout = cleanup_error.output or stdout
                    stderr = cleanup_error.stderr or stderr
        except BaseException:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                process.communicate(timeout=2)
            except (subprocess.TimeoutExpired, OSError):
                pass
            raise
        finally:
            for stream in (process.stdin, process.stdout, process.stderr):
                if stream is not None:
                    stream.close()
        for label, data in (("stdout", stdout), ("stderr", stderr)):
            path = self.root / (name + "." + label)
            with path.open("xb") as target:
                target.write(data)
            record[label], record[label + "_sha256"] = path.name, sha(data)
        record.update(status=process.poll(), timed_out=timed_out, cleanup_incomplete=incomplete)
        (self.root / "commands.json").write_text(json.dumps(self.records, indent=2) + "\n")
        require(not timed_out and not incomplete and process.returncode == expected,
                f"{name}: expected exit {expected}, got {process.returncode}; exact available streams preserved")
        if expected == 0:
            require(not stderr, name + ": unexpected successful stderr; exact stream preserved")
        return stdout, stderr


def bounded_file(root, relative):
    require(isinstance(relative, str) and relative and not Path(relative).is_absolute(), "candidate artifact path is not relative")
    path = root / relative
    require(all(part not in (".", "..") for part in Path(relative).parts)
            and path.resolve(strict=True).is_relative_to(root.resolve()), "candidate artifact escaped its output")
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 128 * 1024 * 1024,
            "candidate artifact is not a bounded regular file")
    return path


def definition_rows(rows, label):
    require(isinstance(rows, list), label + " inventory is missing")
    result, identities = {}, set()
    for row in rows:
        require(isinstance(row, dict) and isinstance(row.get("name"), str), label + " has a malformed definition")
        identity = row.get("native_id")
        require(isinstance(identity, list) and len(identity) == 2 and all(type(part) is int and 0 <= part < 2**64 for part in identity),
                label + " has a malformed native identity")
        require(row["name"] not in result and tuple(identity) not in identities, label + " duplicates a definition")
        result[row["name"]] = row
        identities.add(tuple(identity))
    return result


def validate_work(report, expected):
    counts = report.get("counts")
    keys = ("checked_bodies", "generated_bodies", "compiled_objects", "reused_objects", "metadata_encodes", "archive_builds", "sdk_loads")
    require(isinstance(counts, dict) and all(type(counts.get(key)) is int and counts[key] >= 0 for key in keys), "missing actual work counters")
    require(counts["sdk_loads"] == 1, "retained request reloaded the SDK")
    checked = definition_rows(report.get("checked_definitions"), "checked")
    generated = definition_rows(report.get("generated_definitions"), "generated")
    require(counts["checked_bodies"] == len(checked) and counts["generated_bodies"] == len(generated), "work counters disagree with actual definition inventories")
    require(expected["check_required"] <= checked.keys() <= expected["checked"], "unaffected native function was checked or a required check was absent")
    require(expected["required"] <= generated.keys() <= expected["dirty"], "unaffected native function was regenerated or required generation was absent")
    require(counts["compiled_objects"] <= len(expected["compile_dirty"]), "unchanged native function was recompiled")
    if expected["compiled_count"] is not None:
        require(counts["compiled_objects"] == expected["compiled_count"], "actual native emission count differs from the declared partial-failure history")
    for expectation, actual in (("metadata_count", "metadata_encodes"), ("archive_count", "archive_builds")):
        if expected[expectation] is not None:
            require(counts[actual] == expected[expectation], "artifact recovery repeated or omitted required construction work")
    if expected["artifact_cache_reuse"] is not None:
        require(report.get("artifact_reused") is expected["artifact_cache_reuse"], "artifact recovery did not retain the required prepared generation")
    if expected["action"] != "build" or expected["status"] in (1, 4):
        require(not generated and all(counts[key] == 0 for key in ("compiled_objects", "metadata_encodes", "archive_builds")),
                "non-emitting request performed artifact work")
    if expected["noop"]:
        require(all(counts[key] == 0 for key in ("checked_bodies", "generated_bodies", "compiled_objects", "metadata_encodes", "archive_builds"))
                and report.get("artifact_reused") is True, "no-op repeated compiler, metadata or archive work")
    return checked, generated


def elf_object(data, renames=None):
    """Independent bounded ELF view: bytes, symbols and relocation destinations."""
    renames = renames or {}
    require(len(data) >= 64 and data[:7] == b"\x7fELF\x02\x01\x01", "native object is not little-endian ELF64")
    require(struct.unpack_from("<HHI", data, 16) == (1, 62, 1), "native object is not an x86_64 relocatable")
    base = struct.unpack_from("<Q", data, 40)[0]
    width, count, names_index = struct.unpack_from("<HHH", data, 58)
    require(width == 64 and 1 < count <= 4096 and 0 < names_index < count and base + count * 64 <= len(data), "invalid ELF section inventory")
    sections = [struct.unpack_from("<IIQQQQIIQQ", data, base + index * 64) for index in range(count)]

    def part(offset, size):
        require(0 <= offset <= len(data) and 0 <= size <= len(data) - offset, "ELF section exceeds real bytes")
        return data[offset:offset + size]

    def string(strings, offset):
        require(0 <= offset < len(strings), "ELF string index is absent")
        end = strings.find(b"\0", offset)
        require(end >= offset, "unterminated ELF symbol")
        return strings[offset:end].decode("utf-8", errors="strict")

    strings = part(sections[names_index][4], sections[names_index][5])
    names = [string(strings, section[0]) for section in sections]
    symbol_tables = [index for index, section in enumerate(sections) if section[1] == 2]
    require(len(symbol_tables) == 1, "native object needs exactly one symbol table")
    sym_index = symbol_tables[0]
    table = sections[sym_index]
    require(table[9] == 24 and table[5] % 24 == 0 and 0 < table[6] < count, "invalid ELF symbol table")
    symbols_blob = part(table[4], table[5])
    symbol_strings = part(sections[table[6]][4], sections[table[6]][5])
    symbols, defined, functions, undefined, normalized = [], set(), set(), set(), []
    for offset in range(0, len(symbols_blob), 24):
        name, info, other, section, value, size = struct.unpack_from("<IBBHQQ", symbols_blob, offset)
        name = string(symbol_strings, name)
        binding, kind = info >> 4, info & 15
        require(section in (0, 0xfff1, 0xfff2) or section < count, "ELF symbol names an absent section")
        target = names[section] if 0 < section < count else section
        named = renames.get(name, name)
        symbols.append((named, kind, target, value))
        if offset == 0:
            require((name, info, other, section, value, size) == ("", 0, 0, 0, 0, 0), "ELF null symbol changed")
            continue
        if section == 0:
            require(name and binding in (1, 2), "unrepresented undefined native symbol")
            undefined.add(named)
        elif binding in (1, 2):
            require(named and named not in defined, "duplicate native linker definition")
            defined.add(named)
            if kind == 2:
                require(0 < section < count and sections[section][2] & 4 and size > 0, "native function has no executable bytes")
                functions.add(named)
        normalized.append((named, binding, kind, other, target, value, size))
    allocated, relocations, external_calls = [], [], set()
    for index, section in enumerate(sections):
        if section[2] & 2:
            contents = None if section[1] == 8 else part(section[4], section[5]).hex()
            allocated.append((names[index], section[1], section[2], section[5], section[8], contents))
        if section[1] in (4, 9):
            require(section[1] == 4 and section[6] == sym_index and 0 < section[7] < count
                    and section[9] == 24 and section[5] % 24 == 0, "unrepresented native relocation format")
            for offset in range(section[4], section[4] + section[5], 24):
                location, info, addend = struct.unpack("<QQq", part(offset, 24))
                sym, kind = info >> 32, info & 0xffffffff
                require(sym < len(symbols) and location < sections[section[7]][5], "relocation has an absent target or location")
                named, sym_kind, target, value = symbols[sym]
                if target == 0 and named:
                    code = sections[section[7]]
                    code_bytes = part(code[4], code[5])
                    require(code[2] & 4 and kind in (2, 4) and addend == -4 and 0 < location
                            and location + 4 <= len(code_bytes) and code_bytes[location - 1] in (0xe8, 0xe9),
                            "native external relocation is not a represented direct scalar call")
                    external_calls.add(named)
                relocations.append((names[section[7]], location, kind, addend,
                                    ("section", target, value) if sym_kind == 3 else ("symbol", named)))
    return dict(defined=defined, functions=functions, undefined=undefined, external_calls=external_calls,
                contents=(sorted(allocated), sorted(normalized, key=repr), sorted(relocations, key=repr)))


def archive_members(data):
    require(data.startswith(b"!<arch>\n"), "candidate artifact is not an archive")
    at, long_names, members, offsets, symbol_index = 8, b"", {}, {}, None
    while at < len(data):
        header_at = at
        require(at + 60 <= len(data) and data[at + 58:at + 60] == b"`\n", "truncated archive header")
        name = data[at:at + 16].decode("ascii").rstrip()
        size_text = data[at + 48:at + 58].decode("ascii").strip()
        require(size_text.isdigit(), "invalid archive member size")
        size = int(size_text)
        start, end = at + 60, at + 60 + size
        require(end <= len(data), "truncated archive member")
        payload = data[start:end]
        if name == "//":
            require(not long_names, "duplicate archive long-name table")
            long_names = payload
        elif name == "/":
            require(symbol_index is None, "duplicate archive symbol index")
            symbol_index = payload
        else:
            if name.startswith("/"):
                require(name[1:].isdigit(), "unrepresented archive special member")
                offset = int(name[1:])
                end_name = long_names.find(b"/\n", offset)
                require(0 <= offset < len(long_names) and end_name >= offset, "invalid archive long member name")
                name = long_names[offset:end_name].decode("ascii")
            else:
                require(name.endswith("/"), "unrepresented archive member name")
                name = name[:-1]
            require(name and Path(name).name == name and name not in members, "duplicate or nonlocal archive member")
            members[name], offsets[name] = payload, header_at
        at = end + (size & 1)
        require(at <= len(data) and (not size & 1 or data[end:at] == b"\n"), "archive padding differs")
    require(at == len(data) and symbol_index is not None and len(symbol_index) >= 4, "archive lacks its actual symbol index")
    number = struct.unpack_from(">I", symbol_index)[0]
    require(4 + number * 4 <= len(symbol_index), "truncated archive symbol offsets")
    locations = struct.unpack_from(">" + "I" * number, symbol_index, 4)
    # GNU indexes contain exactly `number` terminated names. The pinned LLVM
    # ArchiveWriter also includes a zero byte in the member size when those
    # records need two-byte alignment; it is not another empty symbol name.
    # rust-lang/llvm-project 0a5579eb649d3634a9c30ac873300cc2944fa070,
    # llvm/lib/Object/ArchiveWriter.cpp: computeSymbolTableSize/writeSymbolTable.
    names, cursor = [], 4 + number * 4
    for _ in range(number):
        end = symbol_index.find(b"\0", cursor)
        require(end >= cursor, "archive symbol names differ from their count")
        names.append(symbol_index[cursor:end])
        cursor = end + 1
    padding = symbol_index[cursor:]
    require(padding == b"" or (cursor & 1 and padding == b"\0"), "archive symbol index has invalid alignment padding")
    index = {}
    for name, location in zip(names, locations):
        name = name.decode("utf-8")
        require(name and name not in index and location in offsets.values(), "archive symbol has no unique real member")
        index[name] = location
    return members, offsets, index


def elf_section(data, wanted):
    require(len(data) >= 64 and data[:7] == b"\x7fELF\x02\x01\x01", "metadata wrapper is not ELF64")
    base = struct.unpack_from("<Q", data, 40)[0]
    width, count, names_index = struct.unpack_from("<HHH", data, 58)
    require(width == 64 and 0 < names_index < count <= 4096 and base + count * 64 <= len(data), "metadata wrapper section table is invalid")
    sections = [struct.unpack_from("<IIQQQQIIQQ", data, base + index * 64) for index in range(count)]
    names = sections[names_index]
    require(names[4] + names[5] <= len(data), "metadata wrapper names exceed real bytes")
    strings = data[names[4]:names[4] + names[5]]
    found = []
    for section in sections:
        require(section[0] < len(strings), "metadata wrapper section name is absent")
        end = strings.find(b"\0", section[0])
        require(end >= section[0], "metadata wrapper section name is incomplete")
        if strings[section[0]:end] == wanted:
            require(section[4] + section[5] <= len(data), "metadata wrapper payload exceeds real bytes")
            found.append(data[section[4]:section[4] + section[5]])
    require(len(found) == 1, "metadata wrapper lacks its exact unique original section")
    return found[0]


def rlib_link_members(wrapper):
    data, at = elf_section(wrapper, b".rmeta-link"), 0

    def integer():
        nonlocal at
        value, shift = 0, 0
        while True:
            require(at < len(data) and shift < 64, "invalid rlib link integer")
            byte = data[at]
            at += 1
            value |= (byte & 127) << shift
            if byte < 128:
                return value
            shift += 7

    count = integer()
    require(0 < count <= 65535, "rlib link omitted its native object inventory")
    names = []
    for _ in range(count):
        size = integer()
        require(0 < size <= 255 and at + size <= len(data), "invalid rlib link member name")
        names.append(data[at:at + size].decode("ascii"))
        at += size
        require(at < len(data) and data[at] == 0xc1, "rlib link string lacks the original Rust sentinel")
        at += 1
    require(integer() == 0 and data[at:] == b"rust-end-file", "rlib link changed its native-library closure or framing")
    require(len(set(names)) == len(names), "rlib link duplicates a member")
    return names


def inspect_artifact(output, report, expected, previous):
    objects = definition_rows(report.get("objects"), "emitted object")
    wanted = NAMES | ({"stack_args"} if expected["stack"] else set())
    require(objects.keys() == wanted, "actual emitted function inventory differs from the original source")
    declarations = definition_rows(report.get("functions"), "completed declaration")
    require(declarations.keys() == wanted, "completed declarations do not cover the actual original source")
    for name, declaration in declarations.items():
        require(declaration["native_id"] == objects[name]["native_id"]
                and declaration.get("public") is (name != "helper"), "declaration visibility/identity differs from emitted native ownership")
        name_span = declaration.get("name_span")
        position = expected["source"].index(("fn " + name + "(").encode()) + 3
        require(name_span == [position, position + len(name)], "completed declaration name lost its exact original span")
        expected_parameters = list("abcdefgh") if name == "stack_args" else (["left", "right", "mask"] if expected["arity"] == 3 else ["left", "right"]) if name == "combine" else ["value"]
        require([parameter.get("name") for parameter in declaration.get("parameters", [])] == expected_parameters,
                "completed parameter inventory differs from the unchanged declaration")
        calls = [declarations["helper"]["native_id"]] if name == "transform" else []
        require(declaration.get("calls") == calls, "completed local call graph differs from original source")
    counts = report["counts"]
    compiled = {name for name, row in objects.items() if row.get("reused") is False}
    reused = {name for name, row in objects.items() if row.get("reused") is True}
    require(compiled | reused == wanted and not compiled & reused, "missing per-object generation/reuse fact")
    require(compiled <= expected["compile_dirty"] and counts["compiled_objects"] == len(compiled)
            and counts["reused_objects"] == len(reused), "object work differs from the declared dirty function set")
    mapping, members_expected, original_defined, renamed_defined = {}, {}, set(), set()
    for name, row in objects.items():
        native, rust = row.get("native_symbol"), row.get("rust_symbol")
        require(isinstance(native, str) and re.fullmatch(r"f_[0-9a-f]{16}", native)
                and isinstance(rust, str) and rust.startswith("_R"), "native/Rust symbol mapping is malformed")
        require(native not in mapping and rust not in mapping.values(), "native/Rust mapping duplicates emission")
        mapping[native] = rust
    for name, row in objects.items():
        before = bounded_file(output, row.get("original_path")).read_bytes()
        after = bounded_file(output, row.get("renamed_path")).read_bytes()
        require(sha(before) == row.get("original_sha256") and sha(after) == row.get("renamed_sha256"), "recorded object identity differs from its actual bytes")
        old, new = elf_object(before), elf_object(after)
        native = row["native_symbol"]
        helper = "tramp_" + native.removeprefix("f_")
        require(old["defined"] == old["functions"] == {native, helper}, "native object omitted its function or emitted unowned helpers")
        calls = {objects["helper"]["native_symbol"]} if name == "transform" else set()
        require(old["undefined"] == calls and old["external_calls"] == calls,
                "native object has unrepresented or missing actual local-call relocations")
        require(new["defined"] == new["functions"] == {row["rust_symbol"], helper}
                and new["undefined"] == new["external_calls"] == {mapping[value] for value in calls}, "symbol rewrite changed real definition/call ownership")
        require(elf_object(before, mapping)["contents"] == new["contents"], "symbol rewrite changed allocated bytes or relocation destinations")
        require(not original_defined & old["defined"] and not renamed_defined & new["defined"], "archive objects duplicate an emitted definition")
        original_defined |= old["defined"]
        renamed_defined |= new["defined"]
        member = row.get("member")
        require(isinstance(member, str) and member.endswith(".o") and member not in members_expected, "native object lacks a unique archive member")
        members_expected[member] = after
        if name in expected["unchanged"]:
            require(previous and name in previous["objects"], "unchanged object has no earlier actual generation")
            old_row = previous["objects"][name]
            require(row["reused"] is True and all(row[key] == old_row[key] for key in
                    ("native_id", "native_symbol", "rust_symbol", "member", "original_sha256", "renamed_sha256")),
                    "unchanged function did not retain its actual object and identity")
    rlib = output / "libconsumer.rlib"
    data = rlib.read_bytes()
    require(sha(data) == report.get("rlib_sha256"), "published archive differs from actual generation")
    members, offsets, index = archive_members(data)
    require(list(members) == report.get("archive_members") and set(members) == {"lib.rmeta", "lib.rmeta-link", *members_expected},
            "archive inventory differs from complete emitted instances")
    require(rlib_link_members(members["lib.rmeta-link"]) == list(members)[2:]
            and list(members)[2:] == sorted(members_expected), "rlib link inventory differs from actual canonical native object order")
    metadata = elf_section(members["lib.rmeta"], b".rmeta")
    require(len(metadata) >= 40 and metadata[:8] == b"rust\0\0\0\x0c"
            and sha(metadata) == report.get("metadata_sha256") and metadata[24:40].hex() == report.get("crate_hash_le_hex"),
            "original metadata wrapper/hash differs from the published native generation")
    for member, contents in members_expected.items():
        require(members[member] == contents, "archive object member differs from checked bytes")
    for row in objects.values():
        require(index.get(row["rust_symbol"]) == offsets[row["member"]], "archive index does not expose the actual emitted function")
    if expected["noop"]:
        require(previous and data == previous["rlib"], "no-op changed the complete artifact")
    return {"objects": objects, "rlib": data, "output": output, "source": expected["source"]}


def command_expectations(output, report, expected, llvm):
    """Bind each reported tool call to this request's actual artifact inventory."""
    commands = report.get("commands")
    require(isinstance(commands, list), "artifact command inventory is missing")
    if expected["noop"] or expected["action"] != "build" or expected["status"] in (1, 4):
        require(not commands, "non-emitting or no-op request recorded artifact tool work")
    objects = definition_rows(report.get("objects"), "artifact command object")
    by_suffix = {f'{row["native_id"][1]:016x}': row for row in objects.values()}
    result, labels = [], set()
    for row in commands:
        require(isinstance(row, dict) and isinstance(row.get("name"), str)
                and row["name"] not in labels and type(row.get("status")) is int and row["status"] == 0,
                "artifact command is duplicated, malformed or unsuccessful")
        label = row["name"]
        labels.add(label)
        if label == "archive":
            members = ["lib.rmeta", "lib.rmeta-link"] + sorted(item["member"] for item in objects.values())
            require(report.get("archive_members") == members, "archive command lacks its actual member inventory")
            argv = [str(llvm / "llvm-ar"), "crsD", "--format=gnu", "libconsumer.rlib", *members]
        else:
            match = re.fullmatch(r"objcopy-([0-9a-f]{16})", label)
            require(match is not None and match.group(1) in by_suffix, "object command has no emitted owner")
            suffix = match.group(1)
            require(by_suffix[suffix].get("member") == f"consumer_{suffix}.o", "object command member changed")
            argv = [str(llvm / "llvm-objcopy"), "--redefine-syms=symbols.map",
                    f"native_{suffix}.o", f"consumer_{suffix}.o"]
        # Rust Command's debug form is a sequence of quoted strings in this
        # fixed caller, with one explicit current_dir and no environment edits.
        rendered = row.get("command")
        match = re.fullmatch(r'cd ("(?:[^"\\]|\\.)*") && (.*)', rendered) if isinstance(rendered, str) else None
        require(match is not None, "artifact command omitted its exact working directory")
        cwd, rest, actual_argv = json.loads(match.group(1)), match.group(2), []
        while rest:
            value, end = json.JSONDecoder().raw_decode(rest)
            require(isinstance(value, str), "artifact command argument is not a complete string")
            actual_argv.append(value)
            require(end == len(rest) or rest[end] == " ", "artifact command argument separator changed")
            rest = rest[end:].lstrip(" ")
        require(cwd == str(output / "staging") and actual_argv == argv,
                "artifact command changed its request, executable, arguments or working directory")
        for stream in ("stdout", "stderr"):
            contents = bounded_file(output, label + "." + stream).read_bytes()
            require(sha(contents) == row.get(stream + "_sha256"), "artifact command stream differs from actual bytes")
            require(not contents, "artifact utility emitted unexpected output")
        result.append({"name": label, "argv": argv, "cwd": cwd, "status": 0})
    require(sum(row["name"] == "archive" for row in result) == report["counts"]["archive_builds"],
            "archive command inventory differs from the claimed construction count")
    return result


def retained_trace(data, prefixes, candidate, candidate_argv, allowed, root_pid=None, root_exit=0):
    """Account for complete calls within evaluator-observed response prefixes.

    A prefix is captured only after reading readiness or a request response.
    No later trace rewrite, split syscall across a response boundary, extra
    allowed child, or candidate-authored counter can hide repeated tool work.
    Delayed exit/signal notifications are retained without permitting later work.
    """
    require(prefixes, "retained trace lacks externally observed response boundaries")
    ends, previous = [], 0
    for boundary in prefixes:
        prefix = boundary["data"]
        require(isinstance(prefix, bytes) and len(prefix) >= previous and data.startswith(prefix),
                "retained trace changed an already observed response prefix")
        require(not prefix or prefix.endswith(b"\n"), "retained trace response prefix ends inside a record")
        ends.append((boundary["name"], len(prefix), boundary["commands"]))
        previous = len(prefix)
    ends.append(("after-final-response", len(data), []))
    require(not data or data.endswith(b"\n"), "retained trace ends inside a record")
    pending, directories, statuses, tool_pids = {}, {}, {}, set()
    candidate_pid, initial_count, offset = None, 0, 0
    attempts, intervals = [], []
    for interval, end, wanted in ends:
        observed, changed_dirs = [], set()
        for raw_line in data[offset:end].decode("utf-8", errors="strict").splitlines():
            if not raw_line.strip():
                continue
            prefix = re.fullmatch(r'\s*(?:(?:\[pid\s+(\d+)\]|(\d+))\s+)?(.*)', raw_line)
            require(prefix is not None, "unparsed retained trace prefix")
            pid, record = prefix.group(1) or prefix.group(2) or "entry", prefix.group(3)
            resumed = re.fullmatch(r'<\.\.\. (execve|chdir) resumed>(.*)', record)
            if resumed:
                require(pid in pending and pending[pid][0] == resumed.group(1), "unmatched retained syscall resume")
                record = pending.pop(pid)[1] + resumed.group(2)
            elif (record.startswith("execve(") or record.startswith("chdir(")) and record.endswith(" <unfinished ...>"):
                require(pid not in pending, "retained trace overwrote an unfinished syscall")
                pending[pid] = (record.split("(", 1)[0], record.removesuffix(" <unfinished ...>"))
                continue
            elif record.startswith("--- SIG"):
                require(re.fullmatch(r'--- SIG[A-Z0-9]+ .* ---', record) is not None, "malformed retained signal record")
                continue
            elif record.startswith("+++"):
                require(pid not in pending, "retained process exited with an unfinished syscall")
                exited = re.fullmatch(r'\+\+\+ exited with ([0-9]+) \+\+\+', record)
                killed = re.fullmatch(r'\+\+\+ killed by (SIG[A-Z0-9]+)(?: \(core dumped\))? \+\+\+', record)
                require((exited is not None or killed is not None) and pid not in statuses,
                        "unparsed or duplicated retained process exit")
                statuses[pid] = int(exited.group(1)) if exited else killed.group(1)
                continue
            else:
                require(pid not in pending, "retained syscall started before its pending call resumed")
            require(interval != "after-final-response", "artifact work occurred after the final response")
            if record.startswith("chdir("):
                change = re.fullmatch(r'chdir\(("(?:[^"\\]|\\.)*")\)\s+=\s+(0|-1 [A-Z][A-Z0-9_]* \([^\n]*\))', record)
                require(change is not None and change.group(2) == "0", "failed or unparsed artifact working-directory change")
                directory = json.loads(change.group(1))
                require(pid != candidate_pid and pid not in directories and pid not in tool_pids
                        and directory in {row["cwd"] for row in wanted}, "unowned artifact working-directory change")
                directories[pid] = directory
                changed_dirs.add(pid)
                continue
            require(record.startswith("execve("), "unparsed retained trace record: " + record)
            complete = re.fullmatch(r'execve\((.*)\)\s+=\s+(0|-1 [A-Z][A-Z0-9_]* \([^\n]*\))', record)
            require(complete is not None, "incomplete retained execve record")
            filename, consumed = json.JSONDecoder().raw_decode(complete.group(1))
            rest = complete.group(1)[consumed:]
            require(isinstance(filename, str) and rest.startswith(", "), "retained executable path is incomplete")
            argv, consumed = json.JSONDecoder().raw_decode(rest[2:])
            require(isinstance(argv, list) and all(isinstance(arg, str) for arg in argv)
                    and rest[2 + consumed:].startswith(", "), "retained executable arguments are incomplete")
            raw = Path(filename)
            require(raw.is_absolute(), "retained executable is not absolute")
            actual = raw.resolve(strict=True)
            require(actual in allowed, "retained candidate attempted an undeclared process: " + str(actual))
            status = 0 if complete.group(2) == "0" else -1
            attempt = {"request": interval, "pid": pid, "path": str(actual), "argv": argv, "exec_status": status}
            attempts.append(attempt)
            if actual == candidate:
                require(initial_count == 0 and interval == ends[0][0] and status == 0
                        and argv == candidate_argv and filename == str(candidate)
                        and (root_pid is None or pid == str(root_pid)), "retained trace changed its single initial candidate")
                initial_count += 1
                candidate_pid = pid
            else:
                require(initial_count == 1 and pid != candidate_pid and pid not in tool_pids and pid in changed_dirs,
                        "artifact execution lacks a unique observed request-owned child")
                require(status == 0, "artifact executable attempt failed")
                tool_pids.add(pid)
                attempt["cwd"] = directories[pid]
                observed.append({"argv": argv, "cwd": directories[pid], "status": 0})
                require(argv and argv[0] == filename, "artifact executable differs from argv zero")
        require(not pending, "retained syscall crosses an observed response boundary")
        require(changed_dirs <= tool_pids, "working-directory change lacks an accounted artifact execution")
        require(observed == [{key: row[key] for key in ("argv", "cwd", "status")} for row in wanted],
                "actual artifact executions differ from the request's exact command inventory")
        intervals.append({"name": interval, "start": offset, "end": end, "commands": observed})
        offset = end
    require(initial_count == 1 and statuses.get(candidate_pid) == root_exit, "candidate trace lacks its actual expected exit")
    require(all(statuses.get(pid) == 0 for pid in tool_pids), "artifact child exit was absent or unsuccessful")
    return {"complete": True, "attempts": attempts, "intervals": intervals, "exit_statuses": statuses}


class Session:
    def __init__(self, name, candidate, source, root, receipt, sdk, env, allowed, campaign):
        self.root, self.source, self.campaign = root, source, campaign
        self.candidate, self.allowed = candidate, allowed
        self.buffer, self.index, self.last_success = b"", 0, None
        self.eof, self.closed, self.extra_bytes = False, False, 0
        self.max_object_serial, self.blocked_object = 0, None
        self.transcript = []
        self.record = {"name": name, "requests": self.transcript, "exec_trace_qualified": False,
                       "socket_transport_qualified": False, "argv": [], "exit": None}
        campaign["sessions"].append(self.record)
        root.mkdir()
        self.trace = root / "candidate.trace"
        self.trace_prefixes = []
        self.llvm = sdk / "lib/rustlib/x86_64-unknown-linux-gnu/bin"
        self.candidate_argv = [str(candidate), "--retained", str(source), str(root), str(receipt), str(sdk)]
        self.stdout_log = (root / "candidate.stdout").open("xb")
        self.stderr_log = (root / "candidate.stderr").open("xb")
        strace = shutil.which("strace")
        require(strace is not None, "real retained qualification requires strace")
        argv = [strace, "-f", "-s", "65535", "-e", "trace=execve,execveat,chdir,fchdir,socket,socketpair", "-o", str(self.trace),
                str(candidate), "--retained", str(source), str(root), str(receipt), str(sdk)]
        self.record["argv"] = argv
        self.process = subprocess.Popen(argv, cwd=campaign["working_directory"], env=env,
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=self.stderr_log, start_new_session=True)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        try:
            self.ready = self.line()
            require(self.ready.get("event") == "ready" and self.ready.get("schema") == SCHEMA
                    and type(self.ready.get("session_id")) is int and self.ready["session_id"] >= 0
                    and self.ready.get("sdk_receipt_sha256") == SDK_RECEIPT, "candidate did not establish its real retained session")
            self.record["ready"] = self.ready
            self.capture_trace("ready", [])
        except BaseException:
            self.abort()
            raise

    def capture_trace(self, name, commands):
        prefix = self.trace.read_bytes()
        self.trace_prefixes.append({"name": name, "data": prefix, "commands": commands})
        self.record.setdefault("trace_boundaries", []).append(
            {"name": name, "bytes": len(prefix), "sha256": sha(prefix), "commands": commands})

    def __enter__(self):
        return self

    def __exit__(self, kind, error, stack):
        if kind is not None:
            self.record["request_failure"] = str(error)
        try:
            self.finish()
        except Exception as finish_error:
            self.record["finish_failure"] = str(finish_error)
            self.abort()
            if kind is None:
                raise
        return False

    def abort(self):
        if self.closed:
            return
        try:
            self.kill_group()
            self.drain(time.monotonic() + 2)
        finally:
            self.close_handles()

    def kill_group(self):
        # Descendant writers can remain after the strace leader exits.
        try:
            os.killpg(self.process.pid, signal.SIGKILL)
            self.record["cleanup_group_kill"] = "sent"
        except ProcessLookupError:
            self.record["cleanup_group_kill"] = "group-absent"

    def drain(self, deadline):
        while time.monotonic() < deadline:
            if self.eof and self.process.poll() is not None:
                return True
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            if self.eof:
                time.sleep(min(0.01, remaining))
                continue
            for _key, _mask in self.selector.select(min(0.05, remaining)):
                data = os.read(self.process.stdout.fileno(), 65536)
                if data:
                    self.stdout_log.write(data)
                    self.extra_bytes += len(data)
                else:
                    self.eof = True
                    self.selector.unregister(self.process.stdout)
        return self.eof and self.process.poll() is not None

    def close_handles(self):
        self.record["exit"] = self.process.poll()
        self.record["stdout_eof_observed"] = self.eof
        self.record["extra_stdout_bytes"] = self.extra_bytes
        for stream in (self.process.stdin, self.process.stdout, self.stdout_log, self.stderr_log):
            if stream is not None:
                try:
                    stream.close()
                except OSError as error:
                    self.record.setdefault("close_errors", []).append(str(error))
        self.selector.close()
        self.closed = True

    def line(self):
        deadline = time.monotonic() + 60
        while b"\n" not in self.buffer:
            remaining = deadline - time.monotonic()
            require(remaining > 0, "retained candidate response timed out")
            events = self.selector.select(remaining)
            require(events, "retained candidate response timed out")
            data = os.read(self.process.stdout.fileno(), 65536)
            if not data:
                self.eof = True
                self.selector.unregister(self.process.stdout)
                raise AssertionError("retained candidate exited before its required response")
            self.stdout_log.write(data)
            self.stdout_log.flush()
            self.buffer += data
            require(len(self.buffer) <= 16 * 1024 * 1024, "retained candidate response exceeded the bounded protocol")
        line, self.buffer = self.buffer.split(b"\n", 1)
        value = json.loads(line)
        require(isinstance(value, dict), "retained candidate emitted a non-object response")
        return value

    def request(self, expected, previous):
        output = self.root / expected["name"]
        output.mkdir()
        self.source.write_bytes(expected["source"])
        (output / "original-source.rs").write_bytes(expected["source"])
        if expected["sabotage"] == "staging":
            (output / "staging").write_bytes(b"evaluator-owned staging conflict\n")
        if expected["sabotage"] == "publication":
            require(previous is not None, "publication failure needs an actual earlier successful artifact")
            (output / "libconsumer.rlib").write_bytes(previous["rlib"])
        if expected["sabotage"] == "native-object":
            require(self.max_object_serial > 0 and self.blocked_object is None, "partial-emission control lacks its observed serial baseline")
            self.blocked_object = self.root / "native-objects" / f"native-library-{self.max_object_serial + 2}.o"
            self.blocked_object.mkdir()
        request = {"id": expected["name"], "action": expected["action"], "output": expected["name"]}
        record = {"request": request, "source_sha256": sha(expected["source"]), "expected_status": expected["status"],
                  "source": str(output / "original-source.rs"), "response": None}
        self.transcript.append(record)
        (output / "request.json").write_text(json.dumps(request, indent=2) + "\n")
        start = time.monotonic_ns()
        self.process.stdin.write(json.dumps(request).encode() + b"\n")
        self.process.stdin.flush()
        response = self.line()
        record["elapsed_ns"] = time.monotonic_ns() - start
        record["response"] = response
        self.index += 1
        require(response.get("id") == request["id"] and response.get("status") == expected["status"],
                "retained response differs from the predeclared request outcome")
        require(response.get("report") == expected["name"] + "/report.json", "retained response named another request's report")
        path = bounded_file(self.root, response["report"])
        data = path.read_bytes()
        result = json.loads(data)
        record["report_sha256"] = sha(data)
        require(result.get("schema") == REPORT_SCHEMA and result.get("id") == request["id"]
                and result.get("status") == expected["status"] and result.get("phase") == response.get("phase")
                and result.get("session_id") == self.ready["session_id"] and result.get("request_index") == self.index,
                "retained report belongs to a different request, session or outcome")
        require(result.get("source_path") == str(self.source) and result.get("sdk_receipt_sha256") == SDK_RECEIPT,
                "retained report changed its original source or SDK context")
        require(result.get("source_sha256") == sha(expected["source"]), "retained report is not bound to the attempted original source")
        require(self.source.read_bytes() == expected["source"], "candidate changed the original source")
        if previous:
            require((previous["output"] / "libconsumer.rlib").read_bytes() == previous["rlib"], "request changed the earlier durable artifact")
        validate_work(result, expected)
        self.capture_trace(expected["name"], command_expectations(output, result, expected, self.llvm))
        if expected["action"] == "prepare-drop":
            require(result.get("phase") == "prepared-dropped", "prepare/drop control did not drop an actual prepared continuation")
        for row in result.get("objects", []):
            serial = row.get("native_object_serial")
            require(type(serial) is int and serial > 0, "actual native object lacks its emission serial")
            self.max_object_serial = max(self.max_object_serial, serial)
        serial_counter = result.get("native_next_object_serial")
        require(type(serial_counter) is int and serial_counter >= self.max_object_serial,
                "native object serial counter lost an observed emission")
        # This native field is the last consumed serial, including a failed
        # write, rather than the next available object pathname.
        self.max_object_serial = serial_counter
        durable_build = expected["status"] == 0 and expected["action"] == "build"
        if durable_build:
            self.last_success = request["id"]
        require(result.get("last_successful_request") == self.last_success, "failed or check-only request replaced the last durable success")
        if not durable_build and expected["sabotage"] != "publication":
            require(not (output / "libconsumer.rlib").exists(), "failed or check-only request published a new archive")
        if expected["sabotage"] == "publication":
            require(result.get("phase") == "publication" and (output / "libconsumer.rlib").read_bytes() == previous["rlib"],
                    "actual failed publication replaced the retained good artifact or failed at a different boundary")
        if expected["sabotage"] == "staging":
            require(result.get("phase") == "staging" and (output / "staging").read_bytes() == b"evaluator-owned staging conflict\n",
                    "staging conflict did not remain a real terminal boundary failure")
        if expected["sabotage"] == "native-object":
            require(result.get("phase") == "native-emission" and self.blocked_object.is_dir(),
                    "partial object failure did not reach the genuine native output boundary")
            require(not any(self.blocked_object.iterdir()), "candidate wrote inside the evaluator's native-output conflict")
            self.blocked_object.rmdir()
            self.blocked_object = None
        (self.root / "transcript.json").write_text(json.dumps(self.record, indent=2) + "\n")
        return output, result

    def finish(self):
        deadline = time.monotonic() + 20
        observed = False
        try:
            if self.process.stdin and not self.process.stdin.closed:
                self.process.stdin.close()
            observed = self.drain(deadline - 2)
            if not observed:
                self.record["timed_out"] = True
                self.kill_group()
                self.drain(deadline)
        except BaseException:
            self.kill_group()
            self.drain(deadline)
            raise
        finally:
            self.close_handles()
        require(observed and self.record["exit"] == 0 and not self.record.get("close_errors"),
                "retained candidate did not provide bounded EOF and clean exit")
        require(not self.buffer and not self.extra_bytes, "retained candidate emitted unaccounted protocol output")
        require(not (self.root / "candidate.stderr").read_bytes(), "retained candidate emitted unexpected stderr")
        execution = retained_trace(self.trace.read_bytes(), self.trace_prefixes, self.candidate,
                                   self.candidate_argv, self.allowed, root_pid=self.ready["session_id"])
        # Socket calls and unknown syscall formats fail complete trace accounting.
        self.record.update(execution=execution, exec_trace_qualified=True, socket_transport_qualified=True,
                           trace_sha256=sha(self.trace.read_bytes()),
                           stdout_sha256=sha((self.root / "candidate.stdout").read_bytes()),
                           stderr_sha256=sha((self.root / "candidate.stderr").read_bytes()))
        (self.root / "transcript.json").write_text(json.dumps(self.record, indent=2) + "\n")


def consumer_source(expected):
    combine_type = "fn(u64,u64,u64)->u64" if expected["arity"] == 3 else "fn(u64,u64)->u64"
    combine_args = "value,0x101,0xf0" if expected["arity"] == 3 else "value,0x101"
    combine_operator = {"or": "|", "xor": "^", "and": "&"}[expected["combine"]]
    combine_result = "value ^ 0x101 ^ 0xf0" if expected["arity"] == 3 else f"value {combine_operator} 0x101"
    stack_setup, stack_asserts = "", ""
    if expected["stack"]:
        stack_setup = "let stack:fn(u64,u64,u64,u64,u64,u64,u64,u64)->u64=consumer::stack_args;"
        stack_asserts = """
        let args:[u64;8]=core::array::from_fn(|n|value.rotate_left((n*7) as u32).wrapping_add((n as u64)*0x102030405));
        let result=args.iter().enumerate().fold(0u64,|result,(n,value)|result | (value & (1u64<<n)));
        assert_eq!(consumer::stack_args(args[0],args[1],args[2],args[3],args[4],args[5],args[6],args[7]),result);
        assert_eq!(stack(args[0],args[1],args[2],args[3],args[4],args[5],args[6],args[7]),result);
"""
    assertions = 65538 * (6 if expected["stack"] else 4)
    source = f"""fn main() {{
    let transform:fn(u64)->u64=consumer::transform;
    let combine:{combine_type}=consumer::combine;
    {stack_setup}
    for value in (0u64..65536).chain([u64::MAX,1u64<<63]) {{
        assert_eq!(consumer::transform(value),(value & {expected['mask']}) ^ 85);
        assert_eq!(transform(value),(value & {expected['mask']}) ^ 85);
        assert_eq!(consumer::combine({combine_args}),{combine_result});
        assert_eq!(combine({combine_args}),{combine_result});
        {stack_asserts}
    }}
    println!("retained-metadata-results={assertions}");
}}
"""
    return source.encode(), assertions


def original_consumer(commands, rustc, env, output, expected, report):
    name = output.parent.name + "-" + expected["name"]
    root, _ = commands.run(name + "-metadata-root", [rustc, "-Zls=root", output / "libconsumer.rlib"], env)
    spec = importlib.util.spec_from_file_location("metadata_sdk_construction", ROOT / "tools/build-metadata-sdk-receipt.py")
    construction = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(construction)
    identity = construction.parse_root(root)
    require(identity["stable_crate_id_hex"] == "4d850957defa2873" and
            [(dep["name"], dep["kind_ordinal"], dep["is_private"], dep["host_hash_le_hex"]) for dep in identity["dependencies"]]
            == [("core", 2, False, None), ("compiler_builtins", 2, True, None)], "original reader rejected the declared crate/SDK identity")
    caller, assertions = consumer_source(expected)
    source, program = output / "independent-caller.rs", output / "independent-caller"
    source.write_bytes(caller)
    commands.run(name + "-consumer-compile", [rustc, "--edition=2024", "-Cpanic=abort", "-Copt-level=0", "-Dwarnings", source,
                    "--extern", "consumer=" + str(output / "libconsumer.rlib"), "-o", program], env)
    stdout, _ = commands.run(name + "-consumer-run", [program], env)
    require(stdout == f"retained-metadata-results={assertions}\n".encode(), "retained artifact failed the exact independent runtime assertions")
    report["checked_runtime_results"] += assertions
    report["original_consumers"].append({"case": name, "assertions": assertions, "source_sha256": sha(caller),
                                         "program_sha256": sha(program.read_bytes()), "crate_root": identity})


def original_negative(commands, rustc, env, output, label, text, code, *, artifact=True):
    source = output / (label + ".rs")
    source.write_bytes(text)
    target = output / (label + ".rmeta")
    argv = [rustc, "--edition=2024", "-Cpanic=abort", "--error-format=json", "--emit=metadata", source, "-o", target]
    if artifact:
        argv += ["--extern", "consumer=" + str(output / "libconsumer.rlib")]
    else:
        argv += ["--crate-type=rlib"]
    _, stderr = commands.run(output.parent.name + "-" + output.name + "-" + label, argv, env, expected=1)
    original.source_error(stderr, source, code)
    require(not target.exists(), "original rejected request unexpectedly produced metadata")


def native_type_rejection(report, source, expected):
    text = expected["source"].decode()
    line_number, line = next((number, line) for number, line in enumerate(text.splitlines(), 1) if "{ true }" in line)
    column = line.index("true") + 1
    diagnostic = ("error[L0110]: mismatched types: expected `u64`, found `bool`\n"
                  f"  --> {source}:{line_number}:{column}\n"
                  f"   | {line}\n   | {' ' * (column - 1)}^^^^\n"
                  "   = fix: Change the value to type `u64`, or change the expected type.\n\n"
                  f"error: could not compile `{source}` due to 1 previous error(s)\n")
    # The pinned Lust driver's errors() appends this exact summary after render;
    # it is unchanged in the candidate. Keep the full diagnostic equality check.
    require(report.get("phase") == "native-prepare" and report.get("error") == diagnostic,
            "private-body failure did not preserve the exact native type diagnostic and source span")


def sdk_snapshot(sdk, receipt, root):
    snapshot = root / "sdk-snapshot"
    snapshot.mkdir()
    descriptor = json.loads(receipt.read_bytes())
    for dependency in descriptor["dependencies"]:
        for field in ("artifact", "link_artifact"):
            entry = dependency[field]
            source, target = sdk / entry["path"], snapshot / entry["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            require(file_identity(target)["sha256"] == entry["sha256"], "SDK snapshot differs from exact receipt")
    for name in ("llvm-ar", "llvm-objcopy"):
        relative = Path("lib/rustlib/x86_64-unknown-linux-gnu/bin") / name
        target = snapshot / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.symlink_to((sdk / relative).resolve(strict=True))
    return snapshot


def setup_refusal(name, candidate, source, root, receipt, sdk, env, allowed, commands, report, message):
    output = root / name
    output.mkdir()
    trace = output / "candidate.trace"
    argv = [shutil.which("strace"), "-f", "-s", "65535", "-e", "trace=execve,execveat,chdir,fchdir,socket,socketpair", "-o", trace,
            candidate, "--retained", source, output, receipt, sdk]
    stdout, stderr = commands.run(name, argv, env, expected=3)
    require(not stdout and message in stderr.decode(), "unbound SDK setup did not fail before session readiness")
    require(not list(output.rglob("*.rlib")), "rejected SDK setup produced a compiler artifact")
    data = trace.read_bytes()
    execution = retained_trace(data, [{"name": "setup", "data": data, "commands": []}], candidate,
                               [str(candidate), "--retained", str(source), str(output), str(receipt), str(sdk)],
                               allowed, root_exit=3)
    report["setup_refusals"].append({"name": name, "execution": execution, "trace_sha256": sha(trace.read_bytes())})


def run_campaign(args, root, report, commands):
    candidate, sdk, receipt = [path.resolve(strict=True) for path in (args.candidate, args.sdk, args.receipt)]
    require(sha(receipt.read_bytes()) == SDK_RECEIPT, "SDK receipt is not the exact reviewed construction output")
    require(shutil.which("strace") is not None, "retained qualification requires actual syscall tracing")
    llvm = sdk / "lib/rustlib/x86_64-unknown-linux-gnu/bin"
    tools = {"candidate": file_identity(candidate)}
    descriptor = json.loads(receipt.read_bytes())
    for name, expected in descriptor["sdk_tools"].items():
        identity = file_identity(sdk / expected["path"])
        require(identity["sha256"] == expected["sha256"] and identity["size"] == expected["size"], "independent SDK tool differs: " + name)
        tools[name] = identity
    for name in ("llvm-ar", "llvm-objcopy"):
        tools[name] = file_identity(llvm / name)
    sdk_files = []
    for dependency in descriptor["dependencies"]:
        for field in ("artifact", "link_artifact"):
            entry = dependency[field]
            identity = file_identity(sdk / entry["path"])
            require(identity["sha256"] == entry["sha256"] and identity["size"] == entry["size"], "original SDK artifact differs from its receipt")
            sdk_files.append(identity)
    report["tool_identities"] = tools
    report["sdk_file_identities"] = sdk_files
    report["sdk_receipt_sha256"] = SDK_RECEIPT
    report["evaluator_sha256"] = sha(Path(__file__).read_bytes())
    report["git_head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    env = {key: value for key, value in os.environ.items() if not key.startswith("LUST_") and key not in
           ("RUSTC_FORCE_RUSTC_VERSION", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")}
    rustc = sdk / "bin/rustc"
    identity, _ = commands.run("original-compiler-identity", [rustc, "-vV"], env)
    require(b"commit-hash: 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494\n" in identity, "independent compiler revision differs")
    traps = root / "compiler-traps"
    traps.mkdir()
    trap = traps / "rustc"
    trap.write_text('#!/bin/sh\nprintf "%s\\n" "$0" >> "$LUSTR_FORBIDDEN_LOG"\nexit 91\n')
    trap.chmod(0o755)
    for name in ("cargo", "rustc-full", "rustc-reference", "clang", "clang++", "cc", "gcc", "c++", "tcc", "ld", "as"):
        (traps / name).symlink_to(trap)
    forbidden = root / "forbidden-compiler.log"
    native_env = {**env, "PATH": str(traps), "RUSTC": str(trap), "RUSTC_WRAPPER": str(trap), "RUSTC_WORKSPACE_WRAPPER": str(trap),
                  "LUSTR_FORBIDDEN_LOG": str(forbidden), "LUST_CACHE_DIR": str(root / "native-disk-cache"),
                  "LUST_NO_DAEMON": "1", "LUST_THREADS": "2"}
    commands.run("compiler-trap-control", [trap], native_env, expected=91)
    require(forbidden.read_bytes() == (str(trap) + "\n").encode(), "compiler trap control did not actually execute")
    forbidden.rename(root / "compiler-trap-control.log")
    allowed = {candidate, (llvm / "llvm-ar").resolve(), (llvm / "llvm-objcopy").resolve()}
    source = root / "consumer.rs"
    source.write_bytes(BASE)
    expected_manifest = []
    for item in HISTORY:
        expected_manifest.append({key: sha(value) if key == "source" else sorted(value) if isinstance(value, frozenset) else value
                                  for key, value in item.items()})
    (root / "predeclared-history.json").write_text(json.dumps(expected_manifest, indent=2) + "\n")
    report["history_manifest_sha256"] = sha((root / "predeclared-history.json").read_bytes())
    previous, observed, outputs = None, {}, {}
    with Session("history", candidate, source, root / "history", receipt, sdk, native_env, allowed, report) as session:
        for expected in HISTORY:
            output, result = session.request(expected, previous)
            require(not forbidden.exists(), "candidate invoked an external compiler trap")
            report["observed_history"].append(expected["name"])
            outputs[expected["name"]] = output
            if expected["status"] == 0 and expected["action"] == "build":
                artifact = inspect_artifact(output, result, expected, previous)
                original_consumer(commands, rustc, env, output, expected, report)
                if expected["name"] in ("body-revert", "signature-revert"):
                    baseline = observed["cold"]["objects"]
                    require(all(artifact["objects"][name]["original_sha256"] == baseline[name]["original_sha256"] for name in NAMES),
                            "reverted body/signature did not reproduce the original native objects")
                if expected["name"] in ("reorder", "same-source-object-recovery"):
                    original_negative(commands, rustc, env, output, "private-access", b"fn main(){let _=consumer::helper(1);}\n", "E0603")
                if expected["name"] == "remove":
                    original_negative(commands, rustc, env, output, "removed-export", b"fn main(){let _=consumer::stack_args;}\n", "E0425")
                if expected["name"] == "signature":
                    original_negative(commands, rustc, env, output, "stale-signature", b"fn main(){let _=consumer::combine(1,2);}\n", "E0061")
                observed[expected["name"]] = artifact
                previous = artifact
            elif expected["status"] == 1:
                native_type_rejection(result, source, expected)
                original_negative(commands, rustc, env, output, "original-invalid-body", expected["source"], "E0308", artifact=False)
            elif expected["status"] == 4:
                require(result.get("phase") == "native-completion"
                        and "unrepresented integer ABI or non-integer signature" in result.get("error", ""),
                        "valid type gap became a native rejection or an unrelated operational failure")
                valid = output / "independent-valid-source.rs"
                valid.write_bytes(expected["source"])
                commands.run("original-valid-type-gap", [rustc, "--edition=2024", "--crate-type=rlib", "--emit=metadata", "-Dwarnings", valid,
                             "-o", output / "independent-valid-source.rmeta"], env)
    require(report["observed_history"] == [item["name"] for item in HISTORY], "required retained history was skipped or reclassified")
    fresh_case = case("fresh-comparison", REORDER_EDIT, dirty=NAMES, required=NAMES, checked=NAMES, check_required=NAMES, mask=511, compiled_count=3)
    with Session("fresh", candidate, source, root / "fresh", receipt, sdk, native_env, allowed, report) as session:
        output, result = session.request(fresh_case, None)
        fresh = inspect_artifact(output, result, fresh_case, None)
        require(fresh["rlib"] == previous["rlib"], "fresh session differs from the final retained artifact")
        original_consumer(commands, rustc, env, output, fresh_case, report)
    snapshot = sdk_snapshot(sdk, receipt, root)
    mutable_receipt = root / "snapshot-receipt.json"
    mutable_receipt.write_bytes(receipt.read_bytes())
    sdk_previous = None
    with Session("sdk-retention", candidate, source, root / "sdk-retention", mutable_receipt, snapshot, native_env, allowed, report) as session:
        initial = case("snapshot-cold", BASE, dirty=NAMES, required=NAMES, checked=NAMES, check_required=NAMES, compiled_count=3)
        output, result = session.request(initial, None)
        sdk_previous = inspect_artifact(output, result, initial, None)
        original_consumer(commands, rustc, env, output, initial, report)
        mutable_receipt.write_bytes(receipt.read_bytes() + b" ")
        changed_receipt = case("loaded-receipt-immutable", EDIT, dirty=("helper",), required=("helper",), checked=("helper",), check_required=("helper",), unchanged=("transform", "combine"), mask=511)
        output, result = session.request(changed_receipt, sdk_previous)
        sdk_previous = inspect_artifact(output, result, changed_receipt, sdk_previous)
        original_consumer(commands, rustc, env, output, changed_receipt, report)
        setup_refusal("new-session-changed-receipt", candidate, source, root, mutable_receipt, snapshot, native_env, allowed, commands, report,
                      "SDK construction receipt differs from its reviewed exact identity")
        mutable_receipt.write_bytes(receipt.read_bytes())
        core = snapshot / descriptor["dependencies"][0]["artifact"]["path"]
        with core.open("r+b") as stream:
            stream.write(b"X")
        changed_artifact = case("loaded-artifact-immutable", BASE, dirty=("helper",), required=("helper",), checked=("helper",), check_required=("helper",), unchanged=("transform", "combine"))
        output, result = session.request(changed_artifact, sdk_previous)
        sdk_previous = inspect_artifact(output, result, changed_artifact, sdk_previous)
        original_consumer(commands, rustc, env, output, changed_artifact, report)
        setup_refusal("new-session-changed-artifact", candidate, source, root, mutable_receipt, snapshot, native_env, allowed, commands, report,
                      "metadata SDK artifact differs from its construction receipt")
    require(not forbidden.exists(), "retained candidate invoked a compiler trap")
    for name, identity in tools.items():
        require(file_identity(identity["path"]) == identity, "qualification tool changed: " + name)
    for identity in sdk_files:
        require(file_identity(identity["path"]) == identity, "original SDK artifact changed during qualification")
    require(sha(receipt.read_bytes()) == SDK_RECEIPT, "original SDK receipt changed")
    for expected in HISTORY:
        require((outputs[expected["name"]] / "original-source.rs").read_bytes() == expected["source"], "retained evidence source changed")
    report["compiler_traps_unused"] = True
    report["commands"] = commands.records
    validate_consumers(report)
    report["exec_trace_qualified"] = len(report["sessions"]) == 3 and len(report["setup_refusals"]) == 2 and all(
        session.get("exec_trace_qualified") is True for session in report["sessions"])
    report["socket_transport_qualified"] = all(session.get("socket_transport_qualified") is True for session in report["sessions"])
    report["functional_passed"] = True
    report["passed"] = report["exec_trace_qualified"] and report["socket_transport_qualified"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--sdk", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, default=ROOT / "docs/compiler-set/evidence/metadata-sdk-receipt-20261120/receipt.json")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.absolute()
    root.mkdir(mode=0o700)
    report = {"schema": "retained-native-metadata-campaign-v1", "passed": False, "functional_passed": False,
              "exec_trace_qualified": False, "socket_transport_qualified": False,
              "full_compiler_qualification": False, "retained_public_integration": False,
              "performance_qualified": False, "working_directory": str(root), "observed_history": [],
              "sessions": [], "setup_refusals": [], "original_consumers": [], "checked_runtime_results": 0,
              "expected_consumers": EXPECTED_CONSUMERS, "expected_consumer_count": EXPECTED_CONSUMER_COUNT,
              "expected_runtime_assertions": EXPECTED_ASSERTION_TOTAL,
              "unqualified_scope": ["public installed compiler route", "arbitrary metadata declarations and runtime ABI",
                                    "u16 linked ABI remains unintegrated", "cross-target emission", "performance"]}
    commands = Commands(root)
    try:
        run_campaign(args, root, report, commands)
    except Exception as error:
        report["failure"] = str(error)
        (root / "failure.txt").write_text(traceback.format_exc())
    report["commands"] = commands.records
    (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: report[key] for key in ("passed", "functional_passed", "exec_trace_qualified", "socket_transport_qualified", "observed_history", "checked_runtime_results")}
                     | ({"failure": report["failure"]} if "failure" in report else {})))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
