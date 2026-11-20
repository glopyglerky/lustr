#!/usr/bin/env python3
"""Refusal controls for the construction receipt, not compiler coverage tests."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


TOOL = Path(__file__).with_name("build-metadata-sdk-receipt.py")
spec = importlib.util.spec_from_file_location("metadata_sdk_receipt", TOOL)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

# Original -Zls=root output of the fixed source at the declared SDK revision.
ROOT = b"""Crate info:
name consumer
hash 4d9f7a5849640ef99a473e0f882bcf56 stable_crate_id StableCrateId(5585881185830709363)
proc_macro false
triple x86_64-unknown-linux-gnu
edition 2024
symbol_mangling_version V0
required_panic_strategy Some(Abort) panic_in_drop_strategy Unwind
has_global_allocator false has_alloc_error_handler false has_panic_handler false has_default_lib_allocator false
compiler_builtins false needs_allocator false needs_panic_runtime false no_builtins false panic_runtime false profiler_runtime false
=External Dependencies=
1 core-b13a99ac1c8a27b3 hash 0662df1217edf122fa8f956981d11136 host_hash None kind Unconditional public
2 compiler_builtins-fb8df4e3730e7261 hash 08c29915e675074ade0248590ef645e6 host_hash None kind Unconditional private


"""
CFG_VERSION = "1.102.0-nightly (5b7b37aef 2026-11-08)"


class Controls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="lustr-sdk-receipt-control-")
        self.directory = Path(self.temporary.name).resolve()

    def tearDown(self):
        self.temporary.cleanup()

    def test_original_reader_facts_and_svh_byte_order(self):
        root = module.parse_root(ROOT)
        self.assertEqual(root["stable_crate_id_hex"], "4d850957defa2873")
        self.assertEqual([(d["name"], d["kind_ordinal"], d["is_private"]) for d in root["dependencies"]],
                         [("core", 2, False), ("compiler_builtins", 2, True)])
        self.assertEqual(root["dependencies"][0]["svh_le_hex"], "3611d18169958ffa22f1ed1712df6206")
        self.assertEqual(root["dependencies"][1]["svh_le_hex"], "e645f60e594802de4a0775e61599c208")
        self.assertEqual(module.KINDS, {"MacrosOnly": 0, "Conditional": 1, "Unconditional": 2})

    def test_missing_or_reordered_dependency_numbers_fail(self):
        for changed in [ROOT.replace(b"1 core-", b"2 core-"), ROOT.replace(b"2 compiler_", b"3 compiler_")]:
            with self.assertRaisesRegex(ValueError, "numbering"):
                module.parse_root(changed)

    def test_duplicate_dependency_name_fails(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            module.parse_root(ROOT.replace(b"2 compiler_builtins-", b"2 core-"))

    def test_unknown_dependency_kind_and_host_context_fail(self):
        for changed in [ROOT.replace(b"kind Unconditional", b"kind Unknown"),
                        ROOT.replace(b"host_hash None", b"host_hash Some(0)")]:
            with self.assertRaisesRegex(ValueError, "dependency row"):
                module.parse_root(changed)

    def test_extra_reader_output_cannot_be_ignored(self):
        for changed in [b"warning: partial output\n" + ROOT, ROOT + b"unexpected final data\n"]:
            with self.assertRaises(ValueError):
                module.parse_root(changed)

    def test_missing_root_field_and_wide_stable_id_fail(self):
        for changed in [ROOT.replace(b"proc_macro false\n", b""),
                        ROOT.replace(b"5585881185830709363", b"18446744073709551616")]:
            with self.assertRaises(ValueError):
                module.parse_root(changed)

    def header_file(self):
        root = module.parse_root(ROOT)
        data = bytearray(1024)
        data[:8] = module.METADATA_HEADER
        data[8:16] = (600).to_bytes(8, "little")
        data[16:24] = (700).to_bytes(8, "little")
        data[24:40] = bytes.fromhex(root["svh_le_hex"])
        version = ("rustc " + CFG_VERSION).encode()
        data[40] = len(version)
        data[41:41 + len(version)] = version
        data[41 + len(version)] = 0xc1
        path = self.directory / "reference.rmeta"
        path.write_bytes(data)
        return path, root

    def test_header_requires_original_hash_version_and_positions(self):
        path, root = self.header_file()
        original = path.read_bytes()
        self.assertEqual(module.check_header(path, root, CFG_VERSION)["root_position"], 600)
        for offset in [7, 24, 41, 41 + len(("rustc " + CFG_VERSION).encode())]:
            changed = bytearray(original)
            changed[offset] ^= 1
            path.write_bytes(changed)
            with self.assertRaises(ValueError):
                module.check_header(path, root, CFG_VERSION)
        changed = bytearray(original)
        changed[8:16] = (2048).to_bytes(8, "little")
        path.write_bytes(changed)
        with self.assertRaisesRegex(ValueError, "positions"):
            module.check_header(path, root, CFG_VERSION)

    def test_rlib_or_truncated_metadata_is_not_admitted(self):
        path, root = self.header_file()
        for data in [b"!<arch>\n", module.METADATA_HEADER]:
            path.write_bytes(data)
            with self.assertRaises(ValueError):
                module.check_header(path, root, CFG_VERSION)

    def test_artifact_receipt_changes_with_actual_bytes(self):
        path = self.directory / "dependency.rmeta"
        path.write_bytes(b"first")
        first = module.artifact(self.directory, Path(path.name))
        path.write_bytes(b"other")
        self.assertNotEqual(first, module.artifact(self.directory, Path(path.name)))

    def test_sdk_artifact_cannot_escape_through_a_symlink(self):
        inside = self.directory / "sdk"
        inside.mkdir()
        (self.directory / "outside").write_bytes(b"not an SDK input")
        (inside / "escape").symlink_to(self.directory / "outside")
        with self.assertRaisesRegex(ValueError, "escapes"):
            module.artifact(inside, Path("escape"))

    def test_failed_original_command_is_recorded_once_without_retry(self):
        commands = module.Commands(self.directory)
        source = "from pathlib import Path; import sys; p=Path('attempt'); p.write_bytes((p.read_bytes() if p.exists() else b'')+b'one'); sys.stdout.buffer.write(b'\\x00original'); sys.stderr.buffer.write(b'original failure'); sys.exit(101)"
        with self.assertRaisesRegex(ValueError, "failed"):
            commands.run("failure", [sys.executable, "-c", source])
        self.assertEqual(len(commands.records), 1)
        self.assertEqual(commands.records[0]["status"], 101)
        self.assertEqual((self.directory / "00-failure.stdout").read_bytes(), b"\x00original")
        self.assertEqual((self.directory / "00-failure.stderr").read_bytes(), b"original failure")
        self.assertEqual((self.directory / "attempt").read_bytes(), b"one")
        self.assertFalse((self.directory / "receipt.json").exists())

    def test_unpinned_sdk_fails_before_any_original_compiler_command(self):
        sdk = self.directory / "sdk"
        (sdk / "bin").mkdir(parents=True)
        (sdk / "lib").mkdir()
        (sdk / "bin/rustc").write_bytes(b"different rustc")
        (sdk / "lib/librustc_driver-f21b5e769b80ce6a.so").write_bytes(b"different driver")
        (sdk / "lib/libLLVM.so.23.1-rust-1.102.0-nightly").write_bytes(b"different llvm")
        commands = module.Commands(self.directory)
        with self.assertRaisesRegex(ValueError, "differs from its pin"):
            module.build(sdk, self.directory, commands)
        self.assertEqual(commands.records, [])

    def test_cli_has_no_user_source_input(self):
        result = subprocess.run([sys.executable, str(TOOL), "--sdk", str(self.directory),
                                 "--output", str(self.directory / "new"), "--source", "user.rs"],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(result.returncode, 2)
        self.assertFalse((self.directory / "new").exists())

    def test_existing_output_is_preserved(self):
        marker = self.directory / "receipt.json"
        marker.write_bytes(b"existing evidence")
        result = subprocess.run([sys.executable, str(TOOL), "--sdk", str(self.directory),
                                 "--output", str(self.directory)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(marker.read_bytes(), b"existing evidence")
        self.assertIn(b"preserved without modification", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
