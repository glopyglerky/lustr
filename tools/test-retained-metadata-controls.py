#!/usr/bin/env python3
"""Independent evaluator failure controls; these are not compiler qualification."""
import copy
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest


spec = importlib.util.spec_from_file_location("retained_metadata_campaign", Path(__file__).with_name("test-retained-native-metadata.py"))
campaign = importlib.util.module_from_spec(spec)
spec.loader.exec_module(campaign)


def row(name, number):
    return {"name": name, "native_id": [7, number]}


def work():
    return {"checked_definitions": [], "generated_definitions": [], "artifact_reused": True,
            "counts": {"checked_bodies": 0, "generated_bodies": 0, "compiled_objects": 0,
                       "reused_objects": 3, "metadata_encodes": 0, "archive_builds": 0, "sdk_loads": 1}}


class WorkControls(unittest.TestCase):
    def test_complete_noop(self):
        self.assertEqual(campaign.validate_work(work(), campaign.case("noop", noop=True)), ({}, {}))
        consumers = {"original_consumers": [{"case": name, "assertions": count} for name, count in campaign.EXPECTED_CONSUMERS],
                     "checked_runtime_results": campaign.EXPECTED_ASSERTION_TOTAL,
                     "commands": [{"name": name + "-" + suffix, "status": 0, "timed_out": False, "cleanup_incomplete": False}
                                  for name, _count in campaign.EXPECTED_CONSUMERS for suffix in ("metadata-root", "consumer-compile", "consumer-run")]}
        campaign.validate_consumers(consumers)
        for field in ("original_consumers", "commands"):
            missing = copy.deepcopy(consumers)
            missing[field].pop()
            with self.subTest(field=field), self.assertRaises(AssertionError):
                campaign.validate_consumers(missing)
        repeated = copy.deepcopy(consumers)
        repeated["original_consumers"][-1] = repeated["original_consumers"][0]
        with self.assertRaises(AssertionError):
            campaign.validate_consumers(repeated)

    def test_noop_cannot_repeat_any_work(self):
        for key in ("checked_bodies", "generated_bodies", "compiled_objects", "metadata_encodes", "archive_builds"):
            value = work()
            value["counts"][key] = 1
            with self.subTest(key=key), self.assertRaises(AssertionError):
                campaign.validate_work(value, campaign.case("noop", noop=True))
        value = work()
        campaign.validate_work(value, campaign.case("publication-recovery", artifact_cache_reuse=True, metadata_count=0, archive_count=0))
        value["counts"]["metadata_encodes"] = 1
        with self.assertRaisesRegex(AssertionError, "recovery"):
            campaign.validate_work(value, campaign.case("publication-recovery", artifact_cache_reuse=True, metadata_count=0, archive_count=0))

    def test_unchanged_body_check_and_generation_fail(self):
        for field, count in (("checked_definitions", "checked_bodies"), ("generated_definitions", "generated_bodies")):
            value = work()
            value[field] = [row("combine", 3)]
            value["counts"][count] = 1
            with self.subTest(field=field), self.assertRaisesRegex(AssertionError, "unaffected"):
                campaign.validate_work(value, campaign.case("edit", dirty=("helper",), checked=("helper",)))

    def test_required_native_work_cannot_disappear(self):
        for expected in (campaign.case("cold", checked=("helper",), check_required=("helper",)),
                         campaign.case("cold", dirty=("helper",), required=("helper",))):
            with self.subTest(expected=expected), self.assertRaises(AssertionError):
                campaign.validate_work(work(), expected)

    def test_boolean_counter_and_sdk_reload_fail(self):
        for key, invalid in (("generated_bodies", False), ("sdk_loads", 0), ("sdk_loads", 2), ("compiled_objects", -1)):
            value = work()
            value["counts"][key] = invalid
            with self.subTest(key=key, invalid=invalid), self.assertRaises(AssertionError):
                campaign.validate_work(value, campaign.case("noop", noop=True))

    def test_check_and_rejection_cannot_emit_metadata(self):
        for expected in (campaign.case("check", action="check"), campaign.case("drop", action="prepare-drop"),
                         campaign.case("invalid", status=1), campaign.case("gap", status=4)):
            value = work()
            value["counts"]["metadata_encodes"] = 1
            with self.subTest(expected=expected), self.assertRaisesRegex(AssertionError, "non-emitting"):
                campaign.validate_work(value, expected)

    def test_partial_failure_recovery_requires_one_remaining_object(self):
        expected = campaign.case("recovery", compile_dirty=("helper", "combine"), compiled_count=1)
        value = work()
        value["counts"]["compiled_objects"] = 1
        campaign.validate_work(value, expected)
        for invalid in (0, 2):
            changed = copy.deepcopy(value)
            changed["counts"]["compiled_objects"] = invalid
            with self.subTest(invalid=invalid), self.assertRaisesRegex(AssertionError, "partial-failure"):
                campaign.validate_work(changed, expected)
        value["generated_definitions"] = [row("helper", 1)]
        value["counts"]["generated_bodies"] = 1
        with self.assertRaisesRegex(AssertionError, "regenerated"):
            campaign.validate_work(value, expected)

    def test_duplicate_and_foreign_identity_inventories_fail(self):
        for rows in ([row("helper", 1), row("helper", 2)], [row("helper", 1), row("combine", 1)],
                     [{"name": "helper", "native_id": [7, True]}], [{"name": "helper", "native_id": [7, -1]}]):
            with self.subTest(rows=rows), self.assertRaises(AssertionError):
                campaign.definition_rows(rows, "control")

    def test_original_type_diagnostic_cannot_be_replaced_by_operational_error(self):
        expected = campaign.case("invalid", campaign.INVALID, status=1)
        path = Path("/qualification/consumer.rs")
        number, line = next((n, line) for n, line in enumerate(campaign.INVALID.decode().splitlines(), 1) if "{ true }" in line)
        column = line.index("true") + 1
        diagnostic = ("error[L0110]: mismatched types: expected `u64`, found `bool`\n"
                      f"  --> {path}:{number}:{column}\n   | {line}\n   | {' ' * (column - 1)}^^^^\n"
                      "   = fix: Change the value to type `u64`, or change the expected type.\n\n")
        campaign.native_type_rejection({"phase": "native-prepare", "error": diagnostic}, path, expected)
        for error, phase in (("cannot find source", "native-prepare"), (diagnostic, "publication"),
                             (diagnostic.replace("L0110", "L9999"), "native-prepare"),
                             (diagnostic.replace(f":{number}:{column}", f":{number}:1"), "native-prepare")):
            with self.subTest(error=error, phase=phase), self.assertRaises(AssertionError):
                campaign.native_type_rejection({"phase": phase, "error": error}, path, expected)


def elf_fixture(function="f_0000000000000001", callee="f_0000000000000002", text=b"\xe8\0\0\0\0\xc3\xc3"):
    strings = b"\0"
    indexes = {}
    for name in (function, "tramp_0000000000000001", callee):
        indexes[name] = len(strings)
        strings += name.encode() + b"\0"
    symbols = bytes(24)
    symbols += struct.pack("<IBBHQQ", indexes[function], 0x12, 0, 1, 0, 6)
    symbols += struct.pack("<IBBHQQ", indexes["tramp_0000000000000001"], 0x12, 0, 1, 6, 1)
    symbols += struct.pack("<IBBHQQ", indexes[callee], 0x10, 0, 0, 0, 0)
    relocations = struct.pack("<QQq", 1, (3 << 32) | 4, -4)
    names = b"\0.text\0.rela.text\0.symtab\0.strtab\0.shstrtab\0"
    blob = bytearray(64)
    sections = [(0,) * 10]
    for name, kind, flags, contents, link, info, alignment, entry in (
        (".text", 1, 6, text, 0, 0, 1, 0), (".rela.text", 4, 0, relocations, 3, 1, 8, 24),
        (".symtab", 2, 0, symbols, 4, 1, 8, 24), (".strtab", 3, 0, strings, 0, 0, 1, 0),
        (".shstrtab", 3, 0, names, 0, 0, 1, 0)):
        blob.extend(bytes((-len(blob)) % alignment))
        offset = len(blob)
        blob.extend(contents)
        sections.append((names.index(name.encode() + b"\0"), kind, flags, 0, offset, len(contents), link, info, alignment, entry))
    blob.extend(bytes((-len(blob)) % 8))
    base = len(blob)
    for section in sections:
        blob.extend(struct.pack("<IIQQQQIIQQ", *section))
    blob[:16] = b"\x7fELF\x02\x01\x01" + bytes(9)
    struct.pack_into("<HHIQQQIHHHHHH", blob, 16, 1, 62, 1, 0, 0, base, 0, 64, 0, 0, 64, 6, 5)
    return bytes(blob)


def ar_header(name, payload):
    header = f"{name:<16}{0:<12}{0:<6}{0:<6}{100644:<8}{len(payload):<10}`\n".encode()
    return header + payload + (b"\n" if len(payload) & 1 else b"")


def link_wrapper(payload):
    names = b"\0.rmeta-link\0.shstrtab\0"
    blob = bytearray(64)
    blob.extend(payload)
    names_at = len(blob)
    blob.extend(names)
    blob.extend(bytes((-len(blob)) % 8))
    base = len(blob)
    for section in ((0,) * 10, (1, 1, 0x80000000, 0, 64, len(payload), 0, 0, 1, 0),
                    (13, 3, 0, 0, names_at, len(names), 0, 0, 1, 0)):
        blob.extend(struct.pack("<IIQQQQIIQQ", *section))
    blob[:16] = b"\x7fELF\x02\x01\x01" + bytes(9)
    struct.pack_into("<HHIQQQIHHHHHH", blob, 16, 1, 62, 1, 0, 0, base, 0, 64, 0, 0, 64, 3, 2)
    return bytes(blob)


class ArtifactControls(unittest.TestCase):
    def test_original_rlib_link_framing_and_complete_inventory(self):
        payload = b"\x02\x03a.o\xc1\x03b.o\xc1\x00rust-end-file"
        self.assertEqual(campaign.rlib_link_members(link_wrapper(payload)), ["a.o", "b.o"])
        for malformed in (payload.replace(b"\xc1", b"\xff", 1), payload.replace(b"b.o", b"a.o"),
                          payload[:-1], b"\x00\x00rust-end-file"):
            with self.subTest(payload=malformed), self.assertRaises(AssertionError):
                campaign.rlib_link_members(link_wrapper(malformed))

    def test_direct_local_call_identity_and_rename(self):
        old = elf_fixture()
        new = elf_fixture(function="_Rconsumer_public", callee="_Rconsumer_private")
        mapping = {"f_0000000000000001": "_Rconsumer_public", "f_0000000000000002": "_Rconsumer_private"}
        self.assertEqual(campaign.elf_object(old)["undefined"], {"f_0000000000000002"})
        self.assertEqual(campaign.elf_object(old, mapping)["contents"], campaign.elf_object(new)["contents"])

    def test_changed_machine_bytes_cannot_hide_behind_symbol_rename(self):
        old = elf_fixture()
        new = elf_fixture(text=b"\xe8\0\0\0\0\xc3\x90")
        self.assertNotEqual(campaign.elf_object(old)["contents"], campaign.elf_object(new)["contents"])

    def test_changed_relocation_destination_is_visible(self):
        self.assertNotEqual(campaign.elf_object(elf_fixture())["contents"],
                            campaign.elf_object(elf_fixture(callee="unowned_runtime"))["contents"])
        with self.assertRaisesRegex(AssertionError, "direct scalar call"):
            campaign.elf_object(elf_fixture(text=b"\x90\0\0\0\0\xc3\xc3"))
        unused = bytearray(elf_fixture())
        section_table = struct.unpack_from("<Q", unused, 40)[0]
        struct.pack_into("<Q", unused, section_table + 2 * 64 + 32, 0)
        actual = campaign.elf_object(bytes(unused))
        self.assertEqual(actual["undefined"], {"f_0000000000000002"})
        self.assertEqual(actual["external_calls"], set())

    def test_truncated_and_wrong_machine_objects_fail(self):
        wrong = bytearray(elf_fixture())
        struct.pack_into("<H", wrong, 18, 183)
        for data in (b"", elf_fixture()[:64], bytes(wrong)):
            with self.subTest(size=len(data)), self.assertRaises(AssertionError):
                campaign.elf_object(data)

    def test_archive_index_must_name_a_real_member(self):
        symbol = b"_Rconsumer\0"
        placeholder = struct.pack(">II", 1, 0) + symbol
        offset = 8 + len(ar_header("/", placeholder))
        archive = b"!<arch>\n" + ar_header("/", struct.pack(">II", 1, offset) + symbol) + ar_header("native.o/", b"object fixture")
        members, offsets, symbols = campaign.archive_members(archive)
        self.assertEqual(members, {"native.o": b"object fixture"})
        self.assertEqual(symbols["_Rconsumer"], offsets["native.o"])
        malformed = b"!<arch>\n" + ar_header("/", struct.pack(">II", 1, offset + 1) + symbol) + ar_header("native.o/", b"object fixture")
        with self.assertRaisesRegex(AssertionError, "real member"):
            campaign.archive_members(malformed)

    def test_duplicate_archive_members_fail(self):
        archive = b"!<arch>\n" + ar_header("/", struct.pack(">I", 0)) + ar_header("native.o/", b"a") + ar_header("native.o/", b"b")
        with self.assertRaisesRegex(AssertionError, "duplicate"):
            campaign.archive_members(archive)

    def test_artifact_path_cannot_escape_or_be_a_symlink(self):
        with tempfile.TemporaryDirectory(prefix="retained-evaluator-") as temporary:
            root = Path(temporary)
            (root / "file").write_bytes(b"fixture")
            (root / "link").symlink_to(root / "file")
            for relative in (str(root / "file"), "../absent", "link"):
                with self.subTest(relative=relative), self.assertRaises((AssertionError, FileNotFoundError)):
                    campaign.bounded_file(root, relative)

    def test_unix_socket_call_cannot_be_hidden_in_a_passing_trace(self):
        with tempfile.TemporaryDirectory(prefix="retained-trace-") as temporary:
            root = Path(temporary)
            candidate = root / "candidate"
            candidate.write_bytes(b"nonexecutable parser fixture")
            trace = root / "trace"
            trace.write_text(f'123 execve({json.dumps(str(candidate))}, ["candidate"], 0x1 /* 1 var */) = 0\n'
                             '123 socket(AF_UNIX, SOCK_STREAM, 0) = 3\n123 +++ exited with 0 +++\n')
            with self.assertRaisesRegex(AssertionError, "unparsed"):
                campaign.original.traced_executables(trace, {candidate}, candidate)


if __name__ == "__main__":
    unittest.main()
