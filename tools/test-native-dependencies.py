#!/usr/bin/env python3
"""Exercise loader assertions against binaries produced by the actual host C linker."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1] / "compiler/native/scripts"
sys.path.insert(0, str(SCRIPTS))
from native_dependencies import base_dependency, library_suffix, standalone


class NativeDependencies(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="native-loader-")
        self.root = Path(self.temporary.name)
        self.bases = self.root / 'base space # café "quoted"'
        self.bases.mkdir()
        self.owner = os.getpid()
        self.name = f"libbase-{self.owner}-{'a' * 32}{library_suffix()}"
        self.base = self.bases / self.name
        source = self.root / "base.c"
        source.write_text("int answer(void) { return 42; }\n")
        flags = ["-dynamiclib", "-Wl,-install_name,@rpath/" + self.name] if sys.platform == "darwin" else ["-shared", "-fPIC"]
        self.command(["cc", *flags, str(source), "-o", str(self.base)])
        main = self.root / "main.c"
        main.write_text("int answer(void); int main(void) { return answer() != 42; }\n")
        self.warm = self.root / "warm"
        library = self.name.removeprefix("lib").removesuffix(library_suffix())
        self.command(["cc", str(main), "-L" + str(self.bases), "-l" + library,
                      "-Wl,-rpath," + str(self.bases), "-o", str(self.warm)])
        cold = self.root / "cold.c"
        # A matching byte string is not a loader dependency.
        cold.write_text("volatile const char *name = " + json.dumps(self.name) + ";\nint main(void) { return name[0] != 'l'; }\n")
        self.cold = self.root / "cold"
        self.command(["cc", str(cold), "-o", str(self.cold)])

    def tearDown(self):
        self.temporary.cleanup()

    def command(self, args, **kwargs):
        result = subprocess.run(args, capture_output=True, text=True, timeout=20, **kwargs)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result

    def test_native_load_commands_bind_exact_owner_and_existing_base(self):
        self.command([str(self.warm)])
        self.assertEqual(base_dependency(self.warm, self.bases, self.owner), self.base.resolve())
        with self.assertRaises(RuntimeError):
            base_dependency(self.warm, self.bases, self.owner + 1)
        with self.assertRaises(RuntimeError):
            base_dependency(self.warm, self.root, self.owner)
        with self.assertRaises(RuntimeError):
            standalone(self.warm)
        self.base.unlink()
        with self.assertRaises(RuntimeError):
            base_dependency(self.warm, self.bases, self.owner)

    def test_embedded_name_is_not_a_dependency_and_inspection_errors_fail(self):
        self.command([str(self.cold)])
        standalone(self.cold)
        with self.assertRaises(RuntimeError):
            base_dependency(self.cold, self.bases, self.owner)
        invalid = self.root / "not-a-binary"
        invalid.write_text(self.name)
        with self.assertRaises(RuntimeError):
            standalone(invalid)
        empty_path = self.root / "empty-path"
        empty_path.mkdir()
        result = subprocess.run([sys.executable, str(SCRIPTS / "native_dependencies.py"),
                                 str(self.cold), "--absent"],
                                env=dict(os.environ, PATH=str(empty_path)),
                                capture_output=True, text=True, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("native dependency inspection failed", result.stderr)


if __name__ == "__main__":
    unittest.main()
