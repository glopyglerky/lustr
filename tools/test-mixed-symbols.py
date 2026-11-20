#!/usr/bin/env python3
"""Qualify actual TinyCC objects against the production reader and GNU nm.

This also executes the production build helper to test cache identity changes.
It does not substitute for the full source-built compiler qualification.
"""
import argparse
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
RUSTC = os.environ.get("RUSTC", "rustc")
TCC = str(Path(os.environ.get("LUST_TCC_HOME", ROOT / "compiler/native/third_party/tcc")) / "bin/tcc")
NM = "nm"
EXPORT = "lustr_mixed_0123456789abcdef"
FUNCTION = f"unsigned long long {EXPORT}(unsigned long long a)"


def command(args, **kwargs):
    return subprocess.run(list(map(str, args)), capture_output=True, timeout=30, **kwargs)


def u16(data, at):
    return struct.unpack_from("<H", data, at)[0]


def u32(data, at):
    return struct.unpack_from("<I", data, at)[0]


def u64(data, at):
    return struct.unpack_from("<Q", data, at)[0]


class MixedSymbolTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="lustr-mixed-symbols-")
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name)
        cls.reader = cls.root / "check-mixed-symbols"
        build = command([RUSTC, "--edition=2024", "-O", ROOT / "tools/check-mixed-symbols.rs", "-o", cls.reader])
        assert build.returncode == 0, build.stderr
        cls.objects = {}
        cases = {
            "closed": "static unsigned long long inner(unsigned long long a){return a ^ 123;} " + FUNCTION + "{return inner(a);}",
            "data": "static unsigned long long value = 17; " + FUNCTION + "{return a ^ value;}",
            "strong": "extern unsigned long long external(unsigned long long); " + FUNCTION + "{return external(a);}",
            "weak": "extern unsigned long long external(unsigned long long) __attribute__((weak)); " + FUNCTION + "{return external ? external(a) : a;}",
            "strong-data": "extern unsigned long long external; " + FUNCTION + "{return a ^ external;}",
            "weak-data": "extern unsigned long long external __attribute__((weak)); " + FUNCTION + "{return &external ? a ^ external : a;}",
        }
        for label, source in cases.items():
            c = cls.root / f"{label}.c"
            c.write_text(source)
            obj = cls.root / f"{label}.o"
            result = command([TCC, "-w", "-fno-asynchronous-unwind-tables", "-c", c, "-o", obj])
            assert result.returncode == 0, result.stderr
            cls.objects[label] = obj
        cls.closed = cls.objects["closed"].read_bytes()
        cls.base = u64(cls.closed, 40)
        cls.count = u16(cls.closed, 60)
        cls.symsection = next(cls.base + n * 64 for n in range(cls.count) if u32(cls.closed, cls.base + n * 64 + 4) == 2)
        cls.symat = u64(cls.closed, cls.symsection + 24)
        cls.symsize = u64(cls.closed, cls.symsection + 32)
        cls.strsection = cls.base + u32(cls.closed, cls.symsection + 40) * 64
        cls.strat = u64(cls.closed, cls.strsection + 24)
        cls.export_index = next(i for i in range(cls.symsize // 24)
                                if cls.closed[cls.strat + u32(cls.closed, cls.symat + i * 24):].split(b"\0", 1)[0] == EXPORT.encode())
        cls.export_at = cls.symat + cls.export_index * 24

    def candidate(self, path, expected=EXPORT):
        result = command([self.reader, "check", path, expected])
        self.assertIn(result.returncode, (0, 1), result.stderr)
        self.assertEqual(result.stdout, b"")
        return result

    def original(self, path):
        return command([NM, "--undefined-only", "--format=posix", path])

    def mutate(self, label, edit, diagnostic=None, imports=False):
        data = bytearray(self.closed)
        edit(data)
        path = self.root / f"mutation-{label}.o"
        path.write_bytes(data)
        candidate = self.candidate(path)
        self.assertEqual(candidate.returncode, 1, label)
        if diagnostic:
            self.assertIn(diagnostic.encode(), candidate.stderr, label)
        if imports:
            original = self.original(path)
            self.assertEqual(original.returncode, 0, original.stderr)
            self.assertNotEqual(original.stdout, b"", label)
        return path

    def test_actual_tinycc_closed_objects_match_nm(self):
        for label in ("closed", "data"):
            with self.subTest(label=label):
                old = self.original(self.objects[label])
                self.assertEqual((old.returncode, old.stdout), (0, b""), old.stderr)
                new = self.candidate(self.objects[label])
                self.assertEqual((new.returncode, new.stderr), (0, b""))

    def test_actual_tinycc_strong_weak_function_and_data_imports_match_nm(self):
        for label in ("strong", "weak", "strong-data", "weak-data"):
            with self.subTest(label=label):
                old = self.original(self.objects[label])
                self.assertEqual(old.returncode, 0, old.stderr)
                self.assertIn(b"external", old.stdout)
                new = self.candidate(self.objects[label])
                self.assertEqual(new.returncode, 1)
                self.assertIn(b"native object requires unrepresented symbols: external", new.stderr)

    def test_local_undefined_symbol_is_not_skipped(self):
        at = self.symat + 2 * 24  # The real TinyCC local inner function.
        self.assertEqual(self.closed[at + 4] >> 4, 0)
        self.mutate("local-import", lambda data: struct.pack_into("<H", data, at + 6, 0), "unrepresented symbols", imports=True)

    def test_undefined_zero_name_and_unknown_binding_are_rejected(self):
        def unknown(data):
            data[self.export_at + 4] = 0xF2
            struct.pack_into("<H", data, self.export_at + 6, 0)
        self.mutate("unknown-import", unknown, "unrepresented symbols")
        def unnamed(data):
            struct.pack_into("<I", data, self.export_at, 0)
            struct.pack_into("<H", data, self.export_at + 6, 0)
        self.mutate("unnamed-import", unnamed, "unrepresented symbols")

    def test_missing_weak_duplicate_or_nonfunction_export_is_rejected(self):
        result = self.candidate(self.objects["closed"], "another_symbol")
        self.assertEqual(result.returncode, 1)
        self.assertIn(b"missing its expected", result.stderr)
        for label, info in (("weak-export", 0x22), ("data-export", 0x11), ("unknown-kind", 0x1F), ("unknown-binding", 0x32)):
            with self.subTest(label=label):
                self.mutate(label, lambda data, info=info: data.__setitem__(self.export_at + 4, info))
        self.mutate("zero-name", lambda data: struct.pack_into("<I", data, self.export_at, 0), "unnamed ELF")
        def duplicate(data):
            at = self.symat + 2 * 24
            data[at:at + 24] = data[self.export_at:self.export_at + 24]
        self.mutate("duplicate-export", duplicate)

    def test_common_and_absolute_nonfile_definitions_are_outside_scalar_contract(self):
        for label, section in (("common", 0xFFF2), ("absolute", 0xFFF1), ("extended", 0xFFFF)):
            with self.subTest(label=label):
                path = self.mutate(label, lambda data, section=section: struct.pack_into("<H", data, self.export_at + 6, section), "unsupported ELF symbol")
                if label != "extended":
                    old = self.original(path)
                    self.assertEqual((old.returncode, old.stdout), (0, b""), old.stderr)

    def test_truncated_header_tables_strings_and_symbol_records_fail_without_panics(self):
        for length in sorted({0, 6, 7, 8, 15, 16, 63, 64, self.base + self.count * 64 - 1,
                              self.symat + 23, self.symat + self.symsize - 1, len(self.closed) - 1}):
            with self.subTest(length=length):
                self.mutate(f"truncated-{length}", lambda data, length=length: data.__delitem__(slice(length, None)))

    def test_malformed_header_and_section_contracts_fail_closed(self):
        mutations = [
            ("endian", 5, "B", 2), ("machine", 18, "H", 183), ("type", 16, "H", 3),
            ("section-offset", 40, "Q", 0xFFFFFFFFFFFFFFFF), ("section-count", 60, "H", 0),
            ("section-stride", 58, "H", 63), ("names-index", 62, "H", self.count),
            ("null-section", self.base + 4, "I", 1),
            ("sym-kind", self.symsection + 4, "I", 11), ("sym-entry", self.symsection + 56, "Q", 23),
            ("sym-size", self.symsection + 32, "Q", self.symsize - 1),
            ("sym-link", self.symsection + 40, "I", self.count),
            ("sym-info", self.symsection + 44, "I", 0),
            ("bad-string-table", self.strsection + 4, "I", 1),
            ("string-offset", self.export_at, "I", 0xFFFFFFFF),
            ("unknown-section", self.base + 64 + 4, "I", 0xFFFFFFFF),
            ("section-address", self.base + 64 + 16, "Q", 1),
            ("symbol-section", self.export_at + 6, "H", self.count),
            ("symbol-value", self.export_at + 8, "Q", 0xFFFFFFFFFFFFFFFF),
            ("symbol-size", self.export_at + 16, "Q", 0),
            ("symbol-visibility", self.export_at + 5, "B", 4),
            ("null-symbol", self.symat + 4, "B", 0x10),
        ]
        for label, offset, format_, value in mutations:
            with self.subTest(label=label):
                self.mutate(label, lambda data, offset=offset, format_=format_, value=value: struct.pack_into("<" + format_, data, offset, value))
        self.mutate("unterminated-strings", lambda data: data.__setitem__(self.strat + u64(data, self.strsection + 32) - 1, 65))
        self.mutate("overlapping-tables", lambda data: struct.pack_into("<Q", data, self.strsection + 24, self.symat))
        self.mutate("second-symtab", lambda data: struct.pack_into("<I", data, self.base + 2 * 64 + 4, 2))

    def test_relocations_cannot_hide_unknown_symbols_or_escape_sections(self):
        section = next(self.base + n * 64 for n in range(self.count) if u32(self.closed, self.base + n * 64 + 4) == 4)
        at = u64(self.closed, section + 24)
        for label, offset, format_, value in [
            ("relocation-link", section + 40, "I", 0),
            ("relocation-target", section + 44, "I", self.count),
            ("relocation-entry", section + 56, "Q", 16),
            ("relocation-place", at, "Q", 0xFFFFFFFFFFFFFFFF),
            ("relocation-kind", at + 8, "I", 0xFFFFFFFF),
            ("relocation-null-symbol", at + 12, "I", 0),
            ("relocation-file-symbol", at + 12, "I", 1),
            ("relocation-symbol", at + 12, "I", self.symsize // 24),
        ]:
            with self.subTest(label=label):
                self.mutate(label, lambda data, offset=offset, format_=format_, value=value: struct.pack_into("<" + format_, data, offset, value))

    def test_missing_nonregular_oversized_objects_are_rejected(self):
        self.assertEqual(self.candidate(self.root / "missing.o").returncode, 1)
        self.assertEqual(self.candidate(self.root).returncode, 1)
        link = self.root / "linked.o"
        link.symlink_to(self.objects["closed"])
        self.assertEqual(self.candidate(link).returncode, 1)
        huge = self.root / "huge.o"
        with huge.open("wb") as stream:
            stream.truncate(64 * 1024 * 1024 + 1)
        self.assertEqual(self.candidate(huge).returncode, 1)

    def test_production_build_identity_requires_and_hashes_host_validator_and_dependency_sources(self):
        repository = self.root / "identity-repo"
        native = repository / "compiler/native"
        owned = repository / "crates/lustr-native-core/src"
        output = self.root / "identity-output"
        native.mkdir(parents=True)
        owned.mkdir(parents=True)
        host = repository / "src/host_runtime.rs"
        host.parent.mkdir()
        shutil.copyfile(ROOT / "src/host_runtime.rs", host)
        # The production helper hashes these external completion/artifact inputs
        # as well as the original host and symbol-validator sources.
        library_inputs = (
            Path("src/library_contract.rs"), Path("src/library_request.rs"),
            Path("crates/lustr-native-core/src/native_library.rs"),
            Path("crates/lustr-native-core/src/library_history.rs"),
            Path("crates/lustr-native-core/src/native_completion.rs"),
            Path("crates/lustr-native-core/src/library_artifact.rs"),
            Path("crates/lustr-native-core/src/metadata_archive.rs"),
            Path("crates/lustr-native-core/src/public_library.rs"),
            Path("crates/lustr-native-core/src/unit_object.rs"),
            Path("crates/lustr-native-core/src/unit_publication.rs"),
            Path("upstream/metadata-artifact-tools.json"),
            Path("docs/compiler-set/evidence/metadata-sdk-receipt-20261120/receipt.json"),
        )
        library_trees = (Path("crates/lustr-metadata-encoder/src"), Path("crates/lustr-metadata-sdk/src"))
        for relative in library_inputs:
            destination = repository / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, destination)
        for relative in library_trees:
            shutil.copytree(ROOT / relative, repository / relative)
        output.mkdir()
        helper = self.root / "native-build-helper"
        build = command([RUSTC, "--edition=2024", ROOT / "compiler/native/build.rs", "-o", helper])
        self.assertEqual(build.returncode, 0, build.stderr)
        for filename in ("mixed_unit.rs", "mixed_symbols.rs", "native_cache_inputs.rs"):
            shutil.copyfile(ROOT / "crates/lustr-native-core/src" / filename, owned / filename)
        lockfiles = (native / "Cargo.lock", owned.parent / "Cargo.lock")
        for lock in lockfiles:
            shutil.copyfile(ROOT / lock.relative_to(repository), lock)
        environment = dict(os.environ, CARGO_MANIFEST_DIR=str(native), OUT_DIR=str(output),
                           LUST_TCC_HOME=str(Path(TCC).resolve().parents[1]), CARGO_CFG_TARGET_OS="linux")
        environment.pop("DEP_MIMALLOC_INCLUDE_DIR", None)
        def identity():
            result = command([helper], env=environment)
            self.assertEqual(result.returncode, 0, result.stderr)
            lines = result.stdout.decode().splitlines()
            self.assertIn(f"cargo:rerun-if-changed={native}/../../crates/lustr-native-core/src/mixed_symbols.rs", lines)
            self.assertIn(f"cargo:rerun-if-changed={native}/../../src/host_runtime.rs", lines)
            self.assertIn(f"cargo:rerun-if-changed={native}/Cargo.lock", lines)
            self.assertIn(f"cargo:rerun-if-changed={native}/../../crates/lustr-native-core/Cargo.lock", lines)
            for relative in (*library_inputs, *library_trees):
                self.assertIn(f"cargo:rerun-if-changed={native}/../../{relative}", lines)
            return next(line for line in lines if line.startswith("cargo:rustc-env=LUST_BUILD_HASH="))
        original = identity()
        for relative in library_inputs:
            with self.subTest(library_input=relative):
                path = repository / relative
                original_bytes = path.read_bytes()
                path.write_bytes(original_bytes + b"\n// public library identity mutation probe\n")
                self.assertNotEqual(identity(), original)
                path.write_bytes(original_bytes)
                self.assertEqual(identity(), original)
                path.unlink()
                missing_input = command([helper], env=environment)
                self.assertNotEqual(missing_input.returncode, 0)
                self.assertIn(b"public library source input is required", missing_input.stderr)
                path.write_bytes(original_bytes)
                self.assertEqual(identity(), original)
        for relative in library_trees:
            with self.subTest(library_tree=relative):
                path = repository / relative / "lib.rs"
                original_bytes = path.read_bytes()
                path.write_bytes(original_bytes + b"\n// metadata dependency identity mutation probe\n")
                self.assertNotEqual(identity(), original)
                path.write_bytes(original_bytes)
                self.assertEqual(identity(), original)
        for lock in lockfiles:
            with self.subTest(lock=lock.relative_to(repository)):
                lock_bytes = lock.read_bytes()
                lock.write_bytes(lock_bytes + b"\n# dependency identity mutation probe\n")
                self.assertNotEqual(identity(), original)
                lock.write_bytes(lock_bytes)
                self.assertEqual(identity(), original)
                lock.unlink()
                missing_lock = command([helper], env=environment)
                self.assertNotEqual(missing_lock.returncode, 0)
                self.assertIn(b"native compiler dependency lockfile is required", missing_lock.stderr)
                lock.write_bytes(lock_bytes)
                self.assertEqual(identity(), original)
        host_bytes = host.read_bytes()
        host.write_bytes(host_bytes + b"\n// host identity mutation probe\n")
        self.assertNotEqual(identity(), original)
        host.write_bytes(host_bytes)
        self.assertEqual(identity(), original)
        host.unlink()
        missing_host = command([helper], env=environment)
        self.assertNotEqual(missing_host.returncode, 0)
        self.assertIn(b"native host adapter is required", missing_host.stderr)
        host.write_bytes(host_bytes)
        source = owned / "mixed_symbols.rs"
        original_bytes = source.read_bytes()
        source.write_bytes(original_bytes + b"\n// identity mutation probe\n")
        self.assertNotEqual(identity(), original)
        source.write_bytes(original_bytes)
        self.assertEqual(identity(), original)
        source.unlink()
        missing = command([helper], env=environment)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn(b"mixed object symbol validator is required", missing.stderr)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", default=RUSTC)
    parser.add_argument("--tcc", default=TCC)
    parser.add_argument("--nm", default=NM)
    args, remaining = parser.parse_known_args()
    RUSTC, TCC, NM = args.rustc, args.tcc, args.nm
    unittest.main(argv=[__file__, *remaining])
