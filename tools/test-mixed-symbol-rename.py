#!/usr/bin/env python3
"""Check the in-process scalar-object rename against GNU tools and real linking.

These are correctness checks, not a compiler benchmark. Application code and
relocation records must remain byte-identical, including for long Rust symbols.
"""
import argparse
import json
import os
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
RUSTC = os.environ.get("RUSTC", "rustc")
TCC = str(Path(os.environ.get("LUST_TCC_HOME", ROOT / "compiler/native/third_party/tcc")) / "bin/tcc")
NM = "nm"
OBJCOPY = "objcopy"
EXPORT = "lustr_mixed_0123456789abcdef"
LIMIT = 64 * 1024 * 1024


def command(args, **kwargs):
    return subprocess.run(list(map(str, args)), capture_output=True, timeout=60, **kwargs)


def number(data, at, width):
    return int.from_bytes(data[at:at + width], "little")


def layout(data):
    base, count = number(data, 40, 8), number(data, 60, 2)
    sections = [base + index * 64 for index in range(count)]
    symbols = next(at for at in sections if number(data, at + 4, 4) == 2)
    strings = base + number(data, symbols + 40, 4) * 64
    table, size = number(data, symbols + 24, 8), number(data, symbols + 32, 8)
    names_at = number(data, strings + 24, 8)
    names = data[names_at:names_at + number(data, strings + 32, 8)]
    symbols_by_name = {}
    for at in range(table, table + size, 24):
        offset = number(data, at, 4)
        name = names[offset:].split(b"\0", 1)[0]
        symbols_by_name[name] = at
    return sections, symbols, strings, symbols_by_name


class MixedSymbolRenameTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="lustr-mixed-rename-")
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name)
        cls.probe = cls.root / "rename-probe"
        source = cls.root / "probe.rs"
        module = json.dumps(str(ROOT / "crates/lustr-semantic/src/mixed_symbol_rename.rs"))
        source.write_text(f'''#[path = {module}]
mod rename;
fn main() -> std::process::ExitCode {{
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 4);
    let replacement = std::fs::read_to_string(&args[2]).unwrap();
    match rename::rename(std::path::Path::new(&args[0]), &args[1], &replacement, std::path::Path::new(&args[3])) {{
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {{ eprintln!("{{error}}"); std::process::ExitCode::from(1) }}
    }}
}}
''')
        built = command([RUSTC, "--edition=2024", "-O", source, "-o", cls.probe])
        assert built.returncode == 0, built.stderr.decode()
        cls.objects = {}
        for label, prefix, body in [
            ("closed", "static unsigned long long value=17; static unsigned long long inner(unsigned long long a){return a ^ 123;}",
             "return inner(a) ^ value;"),
            ("import", "extern unsigned long long external(unsigned long long);", "return external(a);"),
        ]:
            source = cls.root / f"{label}.c"
            source.write_text(prefix + f"unsigned long long {EXPORT}(unsigned long long a){{{body}}}\n")
            obj = cls.root / f"{label}.o"
            compiled = command([TCC, "-w", "-fno-asynchronous-unwind-tables", "-c", source, "-o", obj])
            assert compiled.returncode == 0, compiled.stderr.decode()
            cls.objects[label] = obj
        cls.original = cls.objects["closed"].read_bytes()
        cls.sections, cls.symheader, cls.strheader, cls.names = layout(cls.original)

    def setUp(self):
        self.case = Path(tempfile.mkdtemp(dir=self.root))

    def rename(self, replacement, source=None, expected=EXPORT, output=None):
        name = self.case / "replacement.txt"
        name.write_text(replacement)
        output = output or self.case / "renamed.o"
        result = command([self.probe, source or self.objects["closed"], expected, name, output])
        self.assertIn(result.returncode, (0, 1), result.stderr)
        self.assertEqual(result.stdout, b"")
        self.assertFalse(list(self.case.glob(".lustr-symbol-*.tmp")), "temporary file leaked")
        return result, output

    def require_rejected(self, replacement, source=None, expected=EXPORT, output=None):
        result, output = self.rename(replacement, source, expected, output)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertFalse(output.exists(), "failed validation published an object")
        return result

    def check_bytes(self, actual, replacement):
        before = self.original
        sections, symheader, strheader, names = layout(actual)
        self.assertEqual(sections, self.sections)
        self.assertEqual(symheader, self.symheader)
        self.assertEqual(strheader, self.strheader)
        export = self.names[EXPORT.encode()]
        self.assertEqual(names[replacement.encode()], export, "export's symbol index changed")
        old_len = number(before, self.strheader + 32, 8)
        if replacement == EXPORT:
            self.assertEqual(actual, before)
            return
        self.assertNotIn(EXPORT.encode(), names, "old export remains in the symbol table")
        allowed = set(range(strheader + 24, strheader + 40)) | set(range(export, export + 4))
        self.assertTrue(all(before[index] == actual[index] for index in range(len(before)) if index not in allowed),
                        "code, relocations or another original object byte changed")
        start = number(actual, strheader + 24, 8)
        self.assertGreaterEqual(start, len(before))
        self.assertEqual(number(actual, export, 4), old_len)
        self.assertEqual(actual[start:start + old_len],
                         before[number(before, strheader + 24, 8):number(before, strheader + 24, 8) + old_len])
        self.assertEqual(actual[start + old_len:], replacement.encode() + b"\0")
        for at in sections:
            if at != strheader and number(before, at + 4, 4) in (1, 4):
                start, size = number(before, at + 24, 8), number(before, at + 32, 8)
                self.assertEqual(actual[start:start + size], before[start:start + size], "code/data/relocations changed")

    def linked(self, obj, name, label):
        source = self.case / f"{label}.rs"
        source.write_text(f'''unsafe extern "C" {{ #[link_name = {json.dumps(name)}] fn selected(value:u64)->u64; }}
fn main() {{ for value in [0u64, 1, 42, u64::MAX] {{ assert_eq!(unsafe {{ selected(value) }}, value ^ 106); }} }}
''')
        executable = self.case / label
        built = command([RUSTC, "--edition=2024", source, "-C", f"link-arg={obj}", "-o", executable])
        self.assertEqual(built.returncode, 0, built.stderr)
        executed = command([executable])
        self.assertEqual((executed.returncode, executed.stdout, executed.stderr), (0, b"", b""))

    def test_short_equal_and_growing_names_preserve_bytes_and_match_objcopy_and_rust(self):
        for index, replacement in enumerate(("x", EXPORT, "_RNvC15renamed_example12scalar_entry", "_RNvC" + "long_name_" * 600)):
            with self.subTest(length=len(replacement)):
                output = self.case / f"candidate-{index}.o"
                result, output = self.rename(replacement, output=output)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.check_bytes(output.read_bytes(), replacement)
                oracle = self.case / f"oracle-{index}.o"
                original = command([OBJCOPY, "--redefine-sym", f"{EXPORT}={replacement}", self.objects["closed"], oracle])
                self.assertEqual(original.returncode, 0, original.stderr)
                a = command([NM, "--format=posix", "-a", "-S", output])
                b = command([NM, "--format=posix", "-a", "-S", oracle])
                self.assertEqual((a.returncode, a.stdout), (b.returncode, b.stdout), (a.stderr, b.stderr))
                self.linked(output, replacement, f"candidate_run_{index}")
                self.linked(oracle, replacement, f"oracle_run_{index}")
        self.assertEqual(self.objects["closed"].read_bytes(), self.original, "renaming modified the input")

    def test_invalid_names_missing_export_and_name_collisions_do_not_publish(self):
        for name in ("", "has\0nul", "inner", "value"):
            with self.subTest(name=name):
                self.require_rejected(name)
        self.require_rejected("new_name", expected="missing_export")
        # File/section names are not additional function exports.
        self.require_rejected("new_name", expected="inner")

    def test_imports_and_malformed_actual_objects_do_not_publish(self):
        self.require_rejected("new_name", source=self.objects["import"])
        export = self.names[EXPORT.encode()]
        mutations = [
            ("truncated", lambda b: b.__delitem__(slice(0, len(b) - 63))),
            ("machine", lambda b: struct.pack_into("<H", b, 18, 183)),
            ("out-of-range-table", lambda b: struct.pack_into("<Q", b, 40, 2**64 - 1)),
            ("bad-name-offset", lambda b: struct.pack_into("<I", b, export, 2**32 - 1)),
            ("weak-export", lambda b: b.__setitem__(export + 4, 0x22)),
            ("import-export", lambda b: struct.pack_into("<H", b, export + 6, 0)),
            ("duplicate-export", lambda b: b.__setitem__(slice(self.names[b"inner"], self.names[b"inner"] + 24), b[export:export + 24])),
        ]
        relocation = next(at for at in self.sections if number(self.original, at + 4, 4) == 4)
        relocation_data = number(self.original, relocation + 24, 8)
        mutations.append(("bad-relocation-index", lambda b: struct.pack_into("<I", b, relocation_data + 12, 2**32 - 1)))
        for label, edit in mutations:
            with self.subTest(label=label):
                data = bytearray(self.original)
                edit(data)
                path = self.case / f"{label}.o"
                path.write_bytes(data)
                self.require_rejected("new_name", source=path)

    def test_existing_output_including_symlinks_is_never_replaced(self):
        target = self.case / "existing.o"
        target.write_bytes(b"do not replace")
        result, _ = self.rename("new_name", output=target)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(target.read_bytes(), b"do not replace")
        for index, destination in enumerate((target, self.case / "missing-target")):
            link = self.case / f"link-{index}.o"
            link.symlink_to(destination)
            result, _ = self.rename("new_name", output=link)
            self.assertEqual(result.returncode, 1)
            self.assertTrue(link.is_symlink())
        result, _ = self.rename("new_name", output=self.objects["closed"])
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.objects["closed"].read_bytes(), self.original)

    def test_new_output_has_an_independent_inode(self):
        result, output = self.rename("new_name")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotEqual(output.stat().st_ino, self.objects["closed"].stat().st_ino)
        output.write_bytes(b"changed only the output")
        self.assertEqual(self.objects["closed"].read_bytes(), self.original)

    def test_missing_nonregular_and_oversized_input_fail_before_publication(self):
        link = self.case / "source-link.o"
        link.symlink_to(self.objects["closed"])
        large = self.case / "oversized.o"
        with large.open("wb") as stream:
            stream.truncate(LIMIT + 1)
        for source in (self.case / "missing.o", self.case, link, large):
            with self.subTest(source=source.name):
                self.require_rejected("new_name", source=source)

    def test_object_growth_is_bounded_and_failed_output_directory_is_untouched(self):
        large = self.case / "full.o"
        large.write_bytes(self.original)
        with large.open("r+b") as stream:
            stream.truncate(LIMIT)
        self.require_rejected("new_name", source=large)
        self.require_rejected("new_name", output=self.case / "missing" / "output.o")
        self.assertFalse((self.case / "missing").exists())


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", default=RUSTC)
    parser.add_argument("--tcc", default=TCC)
    parser.add_argument("--nm", default=NM)
    parser.add_argument("--objcopy", default=OBJCOPY)
    args, remaining = parser.parse_known_args()
    RUSTC, TCC, NM, OBJCOPY = args.rustc, args.tcc, args.nm, args.objcopy
    unittest.main(argv=[__file__, *remaining])
