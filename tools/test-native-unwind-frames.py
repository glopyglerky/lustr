#!/usr/bin/env python3
"""Exercise scalar native CFI from a completed public-library campaign.

This gate requires GDB's Python support and named frame-unwinder debug trace.
Every native frame must actually select ``dwarf2`` and the selected frame's PC
must match the observed walk. A private copy with one incorrect CFA operand
must retain its ordinary runtime result but fail the same frame-sequence check.
Neither a frame-pointer walk nor counting FDE records can satisfy this gate.

The tested scope is the existing total, acyclic scalar fixture. This does not
qualify Rust source-level debug information, exception propagation or cleanups.
The trace's ``yes`` result means selection, not just an attempted unwinder:
https://sourceware.org/pipermail/gdb-cvs/2021-June/050976.html
Frame cache invalidation and Python frame access:
https://sourceware.org/gdb/current/onlinedocs/gdb.html/Frames-In-Python.html
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import struct
import subprocess
import sys


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def span(data, offset, size):
    require(0 <= offset <= len(data) and 0 <= size <= len(data) - offset,
            "ELF/archive field is outside its input")
    return data[offset:offset + size]


def word(data, offset, fmt):
    return struct.unpack(fmt, span(data, offset, struct.calcsize(fmt)))


def string(data, offset):
    require(0 <= offset < len(data), "string offset is outside its table")
    end = data.find(b"\0", offset)
    require(end >= offset, "unterminated ELF string")
    return data[offset:end].decode("ascii")


class Elf:
    """Only the ELF64 little-endian x86-64 objects used by this fixed gate."""

    def __init__(self, data):
        self.data = data
        require(data[:7] == b"\x7fELF\x02\x01\x01", "expected ELF64 little-endian input")
        header = word(data, 16, "<HHIQQQIHHHHHH")
        self.kind, machine = header[:2]
        require(machine == 62 and self.kind in (1, 2, 3), "expected x86-64 ELF")
        shoff, shentsize, shnum, shstrndx = header[5], header[10], header[11], header[12]
        require(shentsize == 64 and 0 < shstrndx < shnum, "unrepresented ELF section table")
        self.sections = []
        for index in range(shnum):
            entry = word(data, shoff + index * shentsize, "<IIQQQQIIQQ")
            self.sections.append(dict(zip(
                ("name_offset", "kind", "flags", "address", "offset", "size", "link", "info", "align", "entry_size"), entry),
                index=index))
        names = self.bytes(self.sections[shstrndx])
        for section in self.sections:
            section["name"] = string(names, section["name_offset"])
        self.symbols = []
        symtab = self.section(".symtab")
        require(symtab["entry_size"] == 24 and symtab["size"] % 24 == 0, "unrepresented ELF symbols")
        names = self.bytes(self.sections[symtab["link"]])
        for index in range(symtab["size"] // 24):
            name, info, other, section, value, size = word(data, symtab["offset"] + index * 24, "<IBBHQQ")
            self.symbols.append({"name": string(names, name), "info": info, "section": section,
                                 "value": value, "size": size, "index": index})

    def section(self, name):
        matches = [section for section in self.sections if section["name"] == name]
        require(len(matches) == 1, "missing or duplicate ELF section " + name)
        return matches[0]

    def bytes(self, section):
        return span(self.data, section["offset"], section["size"])

    def symbol(self, name):
        matches = [symbol for symbol in self.symbols if symbol["name"] == name and symbol["info"] & 15 == 2]
        require(len(matches) == 1 and matches[0]["size"] > 0, "missing or duplicate function " + name)
        return matches[0]

    def body(self, symbol):
        section = self.sections[symbol["section"]]
        return span(self.data, section["offset"] + symbol["value"] - section["address"], symbol["size"])


def archive_members(data):
    require(data.startswith(b"!<arch>\n"), "expected ordinary GNU archive")
    position, names, result = 8, b"", {}
    while position < len(data):
        header = span(data, position, 60)
        require(header[58:] == b"`\n", "invalid archive member header")
        name = header[:16].decode("ascii").rstrip()
        size = int(header[48:58].decode("ascii").strip())
        contents = span(data, position + 60, size)
        if name == "//":
            names = contents
        elif name not in ("/", "/SYM64/"):
            if name.startswith("/"):
                offset = int(name[1:])
                end = names.find(b"/\n", offset)
                require(0 <= offset < end, "invalid archive long name")
                name = names[offset:end].decode("ascii")
            else:
                require(name.endswith("/"), "unrepresented archive member name")
                name = name[:-1]
            require(name not in result, "duplicate archive member")
            result[name] = contents
        position += 60 + size + size % 2
    require(position == len(data), "invalid archive alignment")
    return result


def frame_records(elf):
    section = elf.section(".eh_frame")
    data = elf.bytes(section)
    position, records = 0, {}
    while position < len(data):
        length, = word(data, position, "<I")
        if length == 0:
            require(not any(data[position:]), "data follows EH-frame terminator")
            break
        require(length != 0xffffffff and length >= 4, "unrepresented EH-frame record")
        record = span(data, position, length + 4)
        cie, = word(record, 4, "<I")
        records[position] = {"offset": position, "data": record, "cie": cie}
        position += length + 4
    return section, records


def native_frame(elf, symbol):
    section, records = frame_records(elf)
    cies = [record for record in records.values() if record["cie"] == 0]
    fdes = [record for record in records.values() if record["cie"] != 0]
    require(len(cies) == len(fdes) == 1, "native object must contain one CIE and one FDE")
    cie, fde = cies[0], fdes[0]
    require(cie["offset"] == 0 and fde["offset"] + 4 - fde["cie"] == 0,
            "native FDE does not refer to its actual CIE")
    require(cie["data"][8:].startswith(b"\x01zR\0\x01\x78\x10\x01\x1b"),
            "native CIE is not the qualified pcrel/sdata4 form")
    initial, size = word(fde["data"], 8, "<II")
    require(initial == 0 and size == symbol["size"], "native FDE range differs from its body")
    return cie["data"], fde["data"]


def joined_frame(elf, symbol, template):
    section, records = frame_records(elf)
    matches = []
    for position, record in records.items():
        if record["cie"] == 0 or len(record["data"]) < 17:
            continue
        cie = records.get(position + 4 - record["cie"])
        if cie is None or cie["data"] != template[0]:
            continue
        delta, size = word(record["data"], 8, "<iI")
        initial = section["address"] + position + 8 + delta
        if initial == symbol["value"]:
            require(size == symbol["size"] and record["data"][16:] == template[1][16:],
                    "linked CFI/range differs from the genuine native object")
            matches.append({"offset": section["offset"] + position, "size": size,
                            "instructions": record["data"][16:].hex(), "record_count": len(records)})
    require(len(matches) == 1, "missing or duplicate linked native FDE")
    return matches[0]


def verify_linked_body(native, linked, name):
    source, target = native.symbol(name), linked.symbol(name)
    require(source["size"] == target["size"], "link changed native body size")
    expected = bytearray(native.body(source))
    for section in native.sections:
        if section["kind"] != 4 or section["info"] != source["section"]:
            continue
        require(section["entry_size"] == 24 and section["size"] % 24 == 0, "unrepresented text relocations")
        for position in range(0, section["size"], 24):
            offset, info, addend = word(native.bytes(section), position, "<QQq")
            require(source["value"] <= offset < source["value"] + source["size"], "relocation outside native body")
            require(info & 0xffffffff in (2, 4), "unrepresented native call relocation")
            destination = linked.symbol(native.symbols[info >> 32]["name"])
            offset -= source["value"]
            value = destination["value"] + addend - target["value"] - offset
            require(-(1 << 31) <= value < (1 << 31), "native call relocation out of range")
            expected[offset:offset + 4] = struct.pack("<i", value)
    require(bytes(expected) == linked.body(target), "tested executable does not contain the genuine native body")
    return target, joined_frame(linked, target, native_frame(native, source))


def corrupt_cfa(data, frame):
    # The qualification is deliberately tied to TinyCC's actual instruction
    # sequence: augmentation length, advance over push, CFA offset, saved rbp,
    # advance over mov, CFA register. Change only the offset operand.
    at = frame["offset"] + 16
    require(data[at:at + 9] == b"\0\x41\x0e\x10\x86\x02\x43\x0d\x06",
            "native CFI shape changed; review the mutation before qualification")
    mutated = bytearray(data)
    mutated[at + 3] = 0x7f
    require([i for i, (old, new) in enumerate(zip(data, mutated)) if old != new] == [at + 3],
            "CFI control changed more than its declared operand")
    return bytes(mutated), {"file_offset": at + 3, "before": 16, "after": 127,
                           "operation": "DW_CFA_def_cfa_offset"}


GDB_SCRIPT = r'''
set pagination off
set confirm off
set startup-with-shell off
set auto-load off
set debuginfod enabled off
set print demangle off
set print asm-demangle off
set language c
python
import gdb, json
config = json.loads(CONFIG_JSON)
gdb.execute("file " + json.dumps(config["binary"]))
gdb.execute("break *" + config["symbol"])
gdb.execute("run")
start = int(gdb.parse_and_eval("$pc"))
entry = int(gdb.parse_and_eval("(void *)&" + config["symbol"]))
if start != entry:
    raise gdb.GdbError("inferior did not stop at the native function entry")
if config["offset"]:
    gdb.execute("tbreak *" + hex(start + config["offset"]))
    gdb.execute("continue")
stop = int(gdb.parse_and_eval("$pc"))
if stop != entry + config["offset"]:
    raise gdb.GdbError("inferior did not stop at the required native instruction")
# Observe a fresh frame generation without resuming the inferior. Named trace
# selections bind the selected unwinder to each native frame, including on GDB
# releases which cannot disable individual architecture unwinders.
gdb.invalidate_cached_frames()
gdb.write("LUSTR_UNWIND_TRACE_BEGIN\n", gdb.STDERR)
gdb.execute("set debug frame 1")
frames, error = [], None
try:
    frame = gdb.newest_frame()
    for unused in range(3):
        if frame is None:
            break
        frames.append({"pc": int(frame.pc()), "name": frame.name(), "level": frame.level()})
        if len(frames) < 3:
            frame = frame.older()
except gdb.error as exc:
    error = str(exc)
gdb.execute("set debug frame 0")
gdb.write("LUSTR_UNWIND_TRACE_END\n", gdb.STDERR)
gdb.write("LUSTR_UNWIND_RESULT=" + json.dumps({"entry": entry, "stop": stop,
    "frames": frames, "error": error}) + "\n")
gdb.execute("kill")
end
'''


def selected_unwinders(data):
    """Require accepted sniffers and bind final frame descriptions to PCs.

    GDB's frame_unwind_try_unwinder prints 'yes' only after its sniffer accepts.
    Nested sniffer calls use deeper indentation, so their selection cannot be
    attributed to the caller's frame. Frame descriptions additionally expose
    the selected frame_info::unwind and the PC of that same frame.
    """
    begin, end = b"LUSTR_UNWIND_TRACE_BEGIN\n", b"LUSTR_UNWIND_TRACE_END\n"
    require(data.count(begin) == data.count(end) == 1, "GDB did not delimit one fresh frame trace")
    trace = data.split(begin, 1)[1]
    require(end in trace, "GDB frame trace ended before it began")
    trace = trace.split(end, 1)[0].decode()
    contexts, selections = {}, []
    for line in trace.splitlines():
        match = re.fullmatch(r'( *)\[frame\] (frame_unwind_find_by_frame|frame_unwind_try_unwinder): (.*)', line)
        if not match:
            continue
        indent, function, message = match.groups()
        depth = len(indent)
        frame = re.fullmatch(r"this_frame=(-?\d+)", message)
        attempt = re.fullmatch(r'trying unwinder "([^"]+)"', message)
        if function == "frame_unwind_find_by_frame" and frame:
            contexts[depth] = {"level": int(frame[1]), "attempt": None}
        elif function == "frame_unwind_try_unwinder" and attempt:
            require(depth in contexts, "unwinder attempt has no frame context")
            contexts[depth]["attempt"] = attempt[1]
        elif function == "frame_unwind_try_unwinder" and message == "yes":
            require(depth in contexts and contexts[depth]["attempt"] is not None,
                    "selected unwinder has no matching attempt")
            selections.append({"level": contexts[depth]["level"], "name": contexts[depth]["attempt"]})
            contexts[depth]["attempt"] = None
    frames = sorted(set((int(level), kind, name, int(pc, 16)) for level, kind, name, pc in re.findall(
        r'\{level=(-?\d+),type=(\w+_FRAME),unwinder="([^"]+)",pc=(0x[0-9a-f]+),', trace)))
    require(selections and any(item["level"] == 0 for item in selections),
            "GDB did not report the selected native unwinder")
    return {"selections": selections, "frames": [dict(zip(("level", "type", "name", "pc"), frame)) for frame in frames]}


def used_dwarf(observation, level):
    selected = [item["name"] for item in observation["unwinder_trace"]["selections"] if item["level"] == level]
    if not selected or any(name != "dwarf2" for name in selected):
        return False
    expected_pc = observation["frames"][level]["pc"]
    return any(item == {"level": level, "type": "NORMAL_FRAME", "name": "dwarf2", "pc": expected_pc}
               for item in observation["unwinder_trace"]["frames"])


def frame_sequence(observation, symbols, offset):
    require(observation["stop"] == observation["entry"] + offset, "wrong native stop position")
    slide = observation["entry"] - symbols[0]["value"]
    frames = observation["frames"]
    return observation["error"] is None and len(frames) == 3 and [frame["level"] for frame in frames] == [0, 1, 2] and all(
        symbol["value"] + slide <= frame["pc"] < symbol["value"] + symbol["size"] + slide
        for symbol, frame in zip(symbols, frames)) and all(used_dwarf(observation, level) for level in (0, 1))


def run(argv, output, name, timeout=40):
    log = output / (name + ".log")
    with log.open("xb") as stream:
        process = subprocess.Popen([str(arg) for arg in argv], stdin=subprocess.DEVNULL,
                                   stdout=stream, stderr=subprocess.STDOUT, start_new_session=True,
                                   env=dict(os.environ, LANG="C", LC_ALL="C", DEBUGINFOD_URLS=""))
        try:
            status = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise AssertionError("timed out: " + name)
    require(status == 0, f"{name} exited {status}; see {log.name}")
    return log.read_bytes()


def walk(gdb, binary, symbol, offset, output, label):
    config = {"binary": str(binary), "symbol": symbol, "offset": offset}
    script = output / (label + ".gdb")
    script.write_text(GDB_SCRIPT.replace("CONFIG_JSON", repr(json.dumps(config))))
    data = run([gdb, "--quiet", "--nx", "--batch", "-x", script], output, label)
    records = [json.loads(line[len(b"LUSTR_UNWIND_RESULT="):]) for line in data.splitlines()
               if line.startswith(b"LUSTR_UNWIND_RESULT=")]
    require(len(records) == 1, "GDB did not complete exactly one frame observation")
    records[0]["unwinder_trace"] = selected_unwinders(data)
    require([item["name"] for item in records[0]["unwinder_trace"]["selections"] if item["level"] == 0] == ["dwarf2"],
            "GDB did not actually select DWARF CFI for the stopped native frame")
    return records[0]


def checked_path(root, relative):
    path = root / relative
    require(path.resolve().is_relative_to(root.resolve()), "campaign path escapes its root")
    return path


def load_cases(campaign):
    report_bytes = (campaign / "report.json").read_bytes()
    report = json.loads(report_bytes)
    require(report.get("passed") is True and report.get("functional_passed") is True,
            "input public campaign has not passed")
    history = report.get("cargo_unwind_library", {})
    require(history.get("passed") is True and history.get("history") == [
        {"case": label, "compiled_objects": count, "status": status}
        for label, count, status in [("cold", 3, 0), ("noop", 0, 0), ("edit", 1, 0), ("invalid", 0, 101), ("repair", 0, 0)]
    ], "required unwind history changed or did not pass")
    require(len(report["source_identity"]) == 2 and all(re.fullmatch(r"[0-9a-f]{40}", value)
            for value in report["source_identity"]), "campaign source identity is missing")
    records = []
    for path in (campaign / "observations").glob("*/report.json"):
        data = path.read_bytes()
        record = json.loads(data)
        if record.get("context", {}).get("crate_name") != "unwind_leaf" or record.get("status") != 0:
            continue
        require(digest(data) == report["sealed_request_files"].get(str(path.relative_to(campaign / "observations"))),
                "native observation differs from the campaign seal")
        if not record.get("artifact_reused") and record["counts"]["compiled_objects"] in (1, 3):
            records.append((path, record, digest(data)))
    require(len(records) == 2, "campaign lacks unique cold and edited native records")
    cases = []
    for label, count in [("cold", 3), ("edit", 1)]:
        matches = [record for record in records if record[1]["counts"]["compiled_objects"] == count]
        require(len(matches) == 1, "missing or repeated native generation")
        path, record, record_hash = matches[0]
        require(record["context"]["profile"]["panic_abort"] is False and record["backend_helpers"] == [],
                "campaign did not use the one-body unwind route")
        archive = checked_path(path.parent, "staging/" + next(item["path"].split("/")[-1]
            for item in record["outputs"] if item["emit"] == "link"))
        archive_bytes = archive.read_bytes()
        require(digest(archive_bytes) == record["rlib_sha256"], "native archive changed")
        members = archive_members(archive_bytes)
        binary = campaign / "projects/unwind-leaf" / ("caller_" + label)
        elf = Elf(binary.read_bytes())
        functions, frame_info = [], {}
        for name in ("limit", "project"):
            objects = [item for item in record["objects"] if item["name"] == name]
            require(len(objects) == 1, "native object is missing or repeated")
            item = objects[0]
            native_bytes = members[item["member"]]
            require(digest(native_bytes) == item["renamed_sha256"], "native object changed in the archive")
            symbol, frame = verify_linked_body(Elf(native_bytes), elf, item["rust_symbol"])
            functions.append(symbol)
            frame_info[name] = frame
        mains = [symbol for symbol in elf.symbols if symbol["name"].endswith("_11caller_" + label + "4main")]
        require(len(mains) == 1 and mains[0]["size"] > 0, "original Rust caller main is absent")
        functions.append(mains[0])
        body = elf.body(functions[0])
        require(body.startswith(b"\x55\x48\x89\xe5") and body.endswith(b"\xc9\xc3"),
                "native prologue/epilogue changed; review exact stop positions")
        cases.append({"label": label, "binary": binary, "elf": elf, "symbols": functions, "frames": frame_info,
                      "record_sha256": record_hash, "archive_sha256": digest(archive_bytes),
                      "binary_sha256": digest(elf.data), "source_sha256": record["source_sha256"]})
    return report, digest(report_bytes), cases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact-dir", type=Path, required=True, help="successful public campaign or its parent directory")
    parser.add_argument("--gdb", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new directory for immutable logs and private controls")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    result = {"schema": "lustr-native-unwind-frames-v1", "passed": False, "cases": [],
              "scope": "DWARF CFI for two native scalar frames into the original Rust caller",
              "rust_source_debug_qualified": False, "panic_cleanup_qualified": False,
              "evaluator_sha256": digest(Path(__file__).read_bytes())}
    try:
        campaign = args.artifact_dir.resolve()
        if (campaign / "campaign/report.json").is_file():
            campaign /= "campaign"
        original, report_hash, cases = load_cases(campaign)
        result.update(source_identity=original["source_identity"], campaign_report_sha256=report_hash)
        gdb = args.gdb.resolve(strict=True)
        result["gdb_sha256"] = digest(gdb.read_bytes())
        result["gdb_version"] = run([gdb, "--version"], output, "gdb-version").decode()
        for case in cases:
            label = case["label"]
            binary = output / (label + "-original")
            binary.write_bytes(case["elf"].data)
            binary.chmod(0o700)
            row = {key: case[key] for key in ("label", "record_sha256", "archive_sha256", "binary_sha256", "source_sha256")}
            row.update(stops=[], mutation=None, negative_control_rejected=False)
            result["cases"].append(row)
            expected = b"unwind leaf ok\n"
            require(run([binary], output, label + "-runtime") == expected, "original runtime changed")
            for name, offset in [("entry", 0), ("after-prologue", 4), ("return", case["symbols"][0]["size"] - 1)]:
                observation = walk(gdb, binary, case["symbols"][0]["name"], offset, output, label + "-" + name)
                require(frame_sequence(observation, case["symbols"], offset), "DWARF did not recover limit, project and Rust main at " + name)
                row["stops"].append({"name": name, "offset": offset, "observation": observation, "passed": True})
            mutated, mutation = corrupt_cfa(case["elf"].data, case["frames"]["limit"])
            changed = Elf(mutated)
            require(len(frame_records(changed)[1]) == len(frame_records(case["elf"])[1])
                    and all(changed.body(symbol) == case["elf"].body(symbol) for symbol in case["symbols"]),
                    "negative control changed code or frame-record count")
            negative = output / (label + "-corrupt-cfa")
            negative.write_bytes(mutated)
            negative.chmod(0o700)
            row["mutation"] = dict(mutation, binary_sha256=digest(mutated))
            require(run([negative], output, label + "-control-runtime") == expected, "CFI control altered ordinary runtime behavior")
            observation = walk(gdb, negative, case["symbols"][0]["name"], 4, output, label + "-control-frames")
            require(not frame_sequence(observation, case["symbols"], 4), "corrupt CFI passed the same required frame sequence")
            row.update(negative_control_rejected=True, negative_observation=observation)
            require(digest(case["binary"].read_bytes()) == case["binary_sha256"], "original campaign binary changed")
        result["passed"] = True
    except Exception as error:
        result["error"] = f"{type(error).__name__}: {error}"
        print(result["error"], file=sys.stderr)
    finally:
        result["files"] = {str(path.relative_to(output)): digest(path.read_bytes())
                           for path in output.rglob("*") if path.is_file()}
        (output / "report.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"passed": result["passed"], "report": str(output / "report.json")}))
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
