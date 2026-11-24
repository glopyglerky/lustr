#!/usr/bin/env python3
"""Retired compiler acquisition must fail before using supplied paths or tools.

Historical replay readers and explicitly synthetic record models remain useful.
These controls do not compile Rust or qualify compiler performance.
"""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
ENTRIES = {
    "run-upstream-slice-benchmark.py": (),
    "run-warm-entry-comparison.py": ("build-baseline", "build-selector-gate", "measure"),
    "run-warm-native-comparison.py": ("build-package", "qualify-package", "measure"),
    "run-warm-entry-diagnostics.py": ("run",),
    "run-warm-native-startup.py": ("run",),
    "measure-coordinator-project.py": ("--prepare-only",),
}


class RetiredAcquisitionControls(unittest.TestCase):
    def test_retired_commands_refuse_before_creating_output_or_invoking_a_tool(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            tool = root / "sentinel"
            marker = root / "called"
            tool.write_text(f"#!{sys.executable}\nfrom pathlib import Path\nPath({str(marker)!r}).write_text('invoked')\n")
            tool.chmod(0o755)
            env = dict(os.environ, RUSTC=str(tool), CARGO=str(tool))
            for filename, modes in ENTRIES.items():
                for mode in modes or (None,):
                    output = root / "output"
                    argv = [sys.executable, "-B", str(ROOT / "tools" / filename)]
                    if mode is not None:
                        argv.append(mode)
                    argv += ["--output", str(output), "--built", str(root), "--repository", str(ROOT), "--sdk", str(root)]
                    result = subprocess.run(argv, env=env, capture_output=True, timeout=20)
                    with self.subTest(filename=filename, mode=mode):
                        self.assertEqual(result.returncode, 2, result.stderr)
                        self.assertIn(b"acquisition is disabled", result.stderr)
                        self.assertEqual(result.stdout, b"")
                        self.assertFalse(output.exists())
                        self.assertFalse(marker.exists())

    def test_historical_record_models_have_no_process_acquisition_sink(self):
        spec = importlib.util.spec_from_file_location("retired_slice", ROOT / "tools/run-upstream-slice-benchmark.py")
        model = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(model)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            recorder = model.Recorder(root, 1)
            with recorder.raw:
                with self.assertRaisesRegex(ValueError, "acquisition is disabled"):
                    recorder.run([sys.executable, "-c", "raise SystemExit(0)"], {}, root, "test", "disabled")
                with self.assertRaisesRegex(ValueError, "acquisition is disabled"):
                    model.Daemon(recorder, root / "tool", {}, root, "test")
                self.assertEqual(recorder.records, [])

    def test_retired_standalone_mixed_driver_launchers_are_absent(self):
        for name in ("mixed-compiler-qualification.rs", "mixed-check-qualification.rs",
                     "mixed-library-qualification.rs", "mixed-scalar-qualification.rs"):
            with self.subTest(name=name):
                self.assertFalse((ROOT / "tools" / name).exists())


if __name__ == "__main__":
    unittest.main()
