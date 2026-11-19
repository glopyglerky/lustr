#!/usr/bin/env python3
"""Exercise real archive packaging and byte retention, never compiler behavior."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools/package-qualification-records.py"
SPEC = importlib.util.spec_from_file_location("qualification_records", TOOL)
PACK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACK)


class PackagingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="qualification-records-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.inputs = self.base / "inputs"
        self.inputs.mkdir()
        self.output = self.base / "output"
        self.workflow = self.base / "workflow.yml"
        self.patterns("records")

    def patterns(self, *patterns):
        self.workflow.write_text("jobs:\n  native-entry-and-services:\n    steps:\n"
            + PACK.UPLOAD + "\n        if: always()\n        uses: actions/upload-artifact@v4\n"
            + "        with:\n          name: exclusive-entry-and-shared-services\n          path: |\n"
            + "".join(PACK.PREFIX + p + "\n" for p in patterns))

    def write(self, relative, data):
        path = self.inputs / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def run_cli(self, expected=0):
        result = subprocess.run([sys.executable, TOOL, "--workflow", self.workflow,
                                 "--root", self.inputs, "--output", self.output],
                                capture_output=True, timeout=20)
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        return json.loads((self.output / "inventory.json").read_text())

    def large_elf(self):
        # Actual host executable bytes with an inert appended tail. No compiler
        # is invoked and no compiler correctness claim follows from this fixture.
        data = Path(shutil.which("true")).resolve().read_bytes()
        self.assertTrue(data.startswith(b"\x7fELF"))
        return data + b"\0" * PACK.COMPILED_MINIMUM

    def test_cli_retains_binary_streams_hidden_logs_and_receipts_exactly(self):
        compiled = self.large_elf()
        records = {
            "records/01.stdout": compiled,
            "records/01.stderr": b"\x00\xfforiginal failure\n",
            "records/02.run.out": compiled,
            "records/03.exec.trace": compiled,
            "records/.lust-cache/daemon.log": b"\xffdaemon bytes\0\n",
            "records/report.json": b'{"passed":false,"exit":3}\n',
            "records/COMPONENTS.txt": b"source=exact\r\nartifact=original\n",
            "records/source.rs": b"fn main() {}\n",
        }
        for name, data in records.items():
            self.write(name, data)
        self.write("records/compiled-program", compiled)
        self.patterns("records", "records/*.stdout", "never-created.json")
        report = self.run_cli()
        self.assertTrue(report["capture_complete"])
        self.assertFalse(report["compiler_qualification"])
        entries = {e["path"]: e for e in report["entries"]}
        self.assertEqual(len(entries), len(report["entries"]))
        self.assertEqual(report["roots"][-1]["state"], "absent")
        binary = entries["records/compiled-program"]
        self.assertFalse(binary["included"])
        self.assertEqual(binary["sha256"], hashlib.sha256(compiled).hexdigest())
        with tarfile.open(self.output / "records.tar.gz") as archive:
            self.assertNotIn("records/compiled-program", archive.getnames())
            for name, original in records.items():
                self.assertEqual(archive.extractfile(name).read(), original)
                self.assertEqual(entries[name]["sha256"], hashlib.sha256(original).hexdigest())
                self.assertEqual((self.inputs / name).read_bytes(), original)
        self.assertEqual(report["archive"]["sha256"],
                         hashlib.sha256((self.output / "records.tar.gz").read_bytes()).hexdigest())

    def test_small_compiled_files_and_unknown_large_bytes_are_preserved(self):
        small = Path(shutil.which("true")).resolve().read_bytes()
        unknown = b"\x00\xffunknown binary evidence" * 70000
        self.write("records/small-program", small)
        self.write("records/unknown.bin", unknown)
        self.run_cli()
        with tarfile.open(self.output / "records.tar.gz") as archive:
            self.assertEqual(archive.extractfile("records/small-program").read(), small)
            self.assertEqual(archive.extractfile("records/unknown.bin").read(), unknown)

    def test_elf_core_dump_is_retained_as_failure_evidence(self):
        core = bytearray(self.large_elf())
        core[16:18] = (4).to_bytes(2, "little" if core[5] == 1 else "big")
        self.write("records/core", core)
        self.run_cli()
        with tarfile.open(self.output / "records.tar.gz") as archive:
            self.assertEqual(archive.extractfile("records/core").read(), core)

    def test_symlinks_and_special_entries_do_not_read_external_targets(self):
        self.write("records/report.json", b"{}\n")
        secret = self.base / "outside-secret"
        secret.write_bytes(b"external bytes must not be archived")
        link = self.inputs / "records/linked-tool"
        link.symlink_to(secret)
        os.mkfifo(self.inputs / "records/unopened-fifo")
        report = self.run_cli()
        entries = {e["path"]: e for e in report["entries"]}
        self.assertEqual(entries["records/linked-tool"]["hash_domain"], "link-target-bytes")
        self.assertEqual(entries["records/linked-tool"]["sha256"], hashlib.sha256(os.fsencode(secret)).hexdigest())
        self.assertEqual(entries["records/unopened-fifo"]["kind"], "special")
        with tarfile.open(self.output / "records.tar.gz") as archive:
            self.assertTrue(archive.getmember("records/linked-tool").issym())
            self.assertFalse(any(m.name.endswith("outside-secret") for m in archive))

    def test_stream_symlink_cannot_claim_complete_byte_capture(self):
        self.write("records/report.json", b"{}\n")
        outside = self.base / "outside"
        outside.write_bytes(b"unread external stream")
        (self.inputs / "records/01.stdout").symlink_to(outside)
        report = self.run_cli(1)
        self.assertFalse(report["capture_complete"])
        self.assertIn("record is a symlink", json.dumps(report["errors"]))

    def test_symlink_parent_in_a_declared_glob_fails_without_traversal(self):
        outside = self.base / "outside"
        outside.mkdir()
        (outside / "secret.stdout").write_bytes(b"do not read")
        (self.inputs / "alias").symlink_to(outside, target_is_directory=True)
        self.patterns("alias/*.stdout")
        report = self.run_cli(1)
        self.assertFalse(report["capture_complete"])
        self.assertEqual(report["roots"][0]["state"], "unreadable-or-symlink-parent")
        self.assertEqual(report["entries"], [])

    def test_raw_filenames_survive_inventory_and_archive(self):
        name = os.fsdecode(b"records/raw-\xff.stdout")
        self.write(name, b"\xfe\0\n")
        report = self.run_cli()
        entry, = [e for e in report["entries"] if e["kind"] == "regular"]
        self.assertEqual(os.fsencode(entry["path"]), os.fsencode(name))
        with tarfile.open(self.output / "records.tar.gz") as archive:
            self.assertEqual(archive.extractfile(name).read(), b"\xfe\0\n")

    def test_input_mutation_is_reported_as_incomplete(self):
        source = self.write("records/01.stdout", b"original\n")
        original_read = PACK.HashedReader.read
        mutated = False

        def mutate(reader, size=-1):
            nonlocal mutated
            if not mutated:
                mutated = True
                with source.open("ab") as output:
                    output.write(b"changed\n")
            return original_read(reader, size)

        with patch.object(PACK.HashedReader, "read", mutate):
            report = PACK.package(self.workflow, self.inputs, self.output)
        self.assertFalse(report["capture_complete"])
        self.assertIn("changed during capture", json.dumps(report["errors"]))

    def test_missing_all_roots_does_not_pass(self):
        report = self.run_cli(1)
        self.assertFalse(report["capture_complete"])
        self.assertEqual(report["roots"][0]["state"], "absent")

    def test_budget_failure_keeps_full_inventory_and_original_stream(self):
        raw = b"original stdout\0\xff" * 100
        source = self.write("records/01.stdout", raw)
        report = PACK.package(self.workflow, self.inputs, self.output, max_bytes=1)
        self.assertFalse(report["capture_complete"])
        self.assertFalse((self.output / "records.tar.gz").exists())
        self.assertTrue((self.output / "records-over-budget.tar.gz").is_file())
        self.assertEqual(source.read_bytes(), raw)
        entry, = [e for e in report["entries"] if e["kind"] == "regular"]
        self.assertEqual(entry["sha256"], hashlib.sha256(raw).hexdigest())

    def test_existing_or_recursive_output_is_never_replaced(self):
        source = self.write("records/01.stdout", b"original\n")
        with self.assertRaises(ValueError):
            PACK.package(self.workflow, self.inputs, source.parent / "nested")
        self.output.mkdir()
        (self.output / "original").write_bytes(b"keep")
        with self.assertRaises(FileExistsError):
            PACK.package(self.workflow, self.inputs, self.output)
        self.assertEqual((self.output / "original").read_bytes(), b"keep")

    def test_literal_upload_parser_rejects_changed_or_escaping_shapes(self):
        for pattern in ("../outside", "absolute//child", "*/nested", "records\\outside"):
            self.patterns(pattern)
            with self.assertRaises(ValueError, msg=pattern):
                PACK.original_patterns(self.workflow)
        self.patterns("records")
        self.workflow.write_text(self.workflow.read_text().replace("path: |", "path: ${{ steps.other.outputs.paths }}"))
        with self.assertRaises(ValueError):
            PACK.original_patterns(self.workflow)

    def test_current_workflow_uses_the_unchanged_complete_roots(self):
        patterns = PACK.original_patterns(ROOT / ".github/workflows/compiler-set.yml")
        self.assertIn("retained-results", patterns)
        self.assertIn("native-cargo-plan", patterns)
        self.assertIn("lustr-native-built/logs", patterns)
        self.assertIn("lustr-shared-upstream/*.stderr", patterns)


if __name__ == "__main__":
    unittest.main()
