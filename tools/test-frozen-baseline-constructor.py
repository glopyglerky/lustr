#!/usr/bin/env python3
"""Exercise frozen-constructor orchestration, not Rust compiler qualification."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("baseline", Path(__file__).with_name("build-regression-baseline.py"))
B = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(B)


class FrozenConstructorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="lustr frozen constructor ")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.source.mkdir()
        self.tcc = self.root / "tcc"
        self.tcc.mkdir()
        self.output, self.evidence = self.root / "output", self.root / "evidence"
        (self.source / "tools").mkdir()
        (self.source / B.CONSTRUCTOR).write_text("// frozen constructor fixture\n")
        self.git("init", "--quiet")
        self.git("add", ".")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "fixture")
        self.revision = self.git("rev-parse", "HEAD").strip()
        # A process-level constructor fixture lets these controls run before the
        # pinned Rust SDK is installed. It does not compile or qualify Rust.
        self.compiler = self.root / "compiler"
        constructor = (
            f"#!{sys.executable}\n"
            "import pathlib, sys\n"
            "assert len(sys.argv) == 5 and sys.argv[2] == '--current-sdk'\n"
            "output = pathlib.Path(sys.argv[4])\n"
            "output.mkdir()\n"
            "(output/'called').write_text('frozen constructor')\n"
        )
        self.compiler.write_text(
            f"#!{sys.executable}\n"
            "import pathlib, sys\n"
            "if sys.argv[1:] == ['-vV']:\n print('fixture compiler, not Rust'); sys.exit(0)\n"
            "assert sys.argv[1] == '--edition=2024' and sys.argv[3] == '-o'\n"
            "assert pathlib.Path(sys.argv[2]).read_text() == '// frozen constructor fixture\\n'\n"
            f"output=pathlib.Path(sys.argv[4]); output.write_text({constructor!r}); output.chmod(0o755)\n"
        )
        self.compiler.chmod(0o755)

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.source), *args], stderr=subprocess.PIPE).decode()

    def run_build(self):
        return B.build(self.source, self.tcc, self.output, self.evidence, str(self.compiler), self.revision)

    def receipt(self):
        return json.loads((self.evidence / "receipt.json").read_text())

    def test_own_frozen_constructor_runs_without_new_setup_or_profile_flags(self):
        receipt = self.run_build()
        self.assertEqual(receipt["status"], "completed")
        self.assertEqual(receipt["baseline"]["revision"], self.revision)
        archived = self.evidence / "constructor-source.rs"
        self.assertEqual(archived.read_bytes(), (self.source / B.CONSTRUCTOR).read_bytes())
        self.assertEqual(receipt["baseline"]["constructor_sha256"], hashlib.sha256(archived.read_bytes()).hexdigest())
        self.assertEqual([row["label"] for row in receipt["commands"]], ["compiler-version", "constructor-build", "constructor-run"])
        self.assertEqual(receipt["commands"][1]["argv"][2], str(archived))
        self.assertEqual(receipt["commands"][2]["argv"], [str(self.evidence / "build-mixed-toolchain"), str(self.source), "--current-sdk", str(self.tcc), str(self.output)])
        self.assertEqual((self.output / "called").read_text(), "frozen constructor")
        for row in receipt["commands"]:
            self.assertEqual(row["exit"], 0)
            for item in row["streams"].values():
                data = (self.evidence / item["file"]).read_bytes()
                self.assertEqual(item["sha256"], hashlib.sha256(data).hexdigest())
                self.assertEqual(item["bytes"], len(data))

    def test_dirty_source_stops_before_build(self):
        (self.source / B.CONSTRUCTOR).write_text("// current constructor must not be substituted\n")
        with self.assertRaisesRegex(ValueError, "clean exact frozen"):
            self.run_build()
        self.assertEqual(self.receipt()["commands"], [])
        self.assertFalse(self.output.exists())

    def test_different_revision_stops_before_build(self):
        self.revision = "0" * 40
        with self.assertRaisesRegex(ValueError, "clean exact frozen"):
            self.run_build()
        self.assertEqual(self.receipt()["status"], "failed")
        self.assertEqual(self.receipt()["commands"], [])

    def test_constructor_build_failure_preserves_status_and_streams(self):
        text = self.compiler.read_text().replace("assert sys.argv[1] ==", "print('cannot build frozen constructor', file=sys.stderr); sys.exit(17)\nassert sys.argv[1] ==")
        self.compiler.write_text(text)
        with self.assertRaisesRegex(RuntimeError, "constructor-build failed with status 17"):
            self.run_build()
        receipt = self.receipt()
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(receipt["commands"][-1]["exit"], 17)
        self.assertEqual((self.evidence / "constructor-build.stderr").read_text(), "cannot build frozen constructor\n")
        self.assertFalse(self.output.exists())

    def test_constructor_runtime_failure_is_not_accepted_as_counterexample(self):
        text = self.compiler.read_text().replace("output.mkdir()", "print('unrelated package setup failure', file=sys.stderr); sys.exit(39)")
        self.compiler.write_text(text)
        with self.assertRaisesRegex(RuntimeError, "constructor-run failed with status 39"):
            self.run_build()
        self.assertEqual(self.receipt()["commands"][-1]["exit"], 39)
        self.assertFalse(self.output.exists())
        self.assertIn("unrelated package setup failure", (self.evidence / "constructor-run.stderr").read_text())

    def test_missing_compiler_preserves_startup_failure(self):
        self.compiler.unlink()
        with self.assertRaises(FileNotFoundError):
            self.run_build()
        row = self.receipt()["commands"][-1]
        self.assertEqual(row["label"], "compiler-version")
        self.assertIn("startup_error", row)
        self.assertNotIn("exit", row)
        self.assertEqual(set(row["streams"]), {"stdout", "stderr"})

    def test_existing_output_is_preserved(self):
        self.output.mkdir()
        sentinel = self.output / "keep"
        sentinel.write_text("previous attempt")
        with self.assertRaisesRegex(ValueError, "must both be new"):
            self.run_build()
        self.assertEqual(sentinel.read_text(), "previous attempt")
        self.assertFalse(self.evidence.exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
