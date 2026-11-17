#!/usr/bin/env python3
"""Exercise the actual mixed trace writer with concurrent compiler-like processes.

The tiny Rust probe compiles the production helper verbatim. It does not build a
compiler, change the profile validator, or claim a performance measurement.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import unittest


SOURCE = Path(__file__).resolve().parents[1] / "crates/lustr-semantic/src/mixed_main.rs"
RUSTC = os.environ.get("RUSTC", "rustc")
PROBE = r'''
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "single" {
        trace(&args[2]);
        println!("original compiler result");
        return;
    }
    let worker: usize = args[1].parse().unwrap();
    let count: usize = args[2].parse().unwrap();
    let barrier = std::path::Path::new(&args[3]);
    fs::write(barrier.join(format!("ready-{worker}")), b"").unwrap();
    let start = std::time::Instant::now();
    while !barrier.join("start").exists() {
        assert!(start.elapsed().as_secs() < 15, "start barrier timed out");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    for n in 0..count {
        let identity = format!("{:016x}", worker * count + n);
        trace(&format!("request-key={identity} retained=false"));
        let phases = ["c-prepare", "output-prepare", "c-write", "libtcc", "nm", "rename", "file-sync", "directory-sync"]
            .iter().map(|phase| format!(r#"{{"phase":"{phase}","elapsed_ns":1,"completed":true}}"#))
            .collect::<Vec<_>>().join(",");
        trace(&format!(concat!(
            "lustr mixed native: unit {identity}\n",
            "lustr mixed native profile: {{\"schema\":\"lustr.mixed-native-emitter\",",
            "\"version\":1,\"native\":\"0123456789abcdef\",\"unit_identity\":\"{identity}\",",
            "\"success\":true,\"elapsed_ns\":8,\"phases\":[{phases}]}}\n"
        ), identity=identity, phases=phases));
    }
}
'''


class ConcurrentTraceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.build = tempfile.TemporaryDirectory(prefix="lustr-trace-probe-")
        cls.addClassCleanup(cls.build.cleanup)
        root = Path(cls.build.name)
        source = SOURCE.read_text()
        start = source.index("fn trace(value: &str) {")
        helper = source[start:source.index("\n}", start) + 2]
        probe = root / "trace.rs"
        probe.write_text("use std::fs;\n" + helper + "\n" + PROBE)
        cls.binary = root / "trace"
        subprocess.run([RUSTC, "--edition=2024", str(probe), "-o", str(cls.binary)], check=True)
        print("Production trace helper SHA-256:", hashlib.sha256(helper.encode()).hexdigest())

    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix="lustr-trace-test-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.trace = self.root / "mixed.trace"
        self.environment = dict(os.environ, LUSTR_MIXED_TRACE=str(self.trace))

    def single(self, value, environment=None):
        result = subprocess.run([str(self.binary), "single", value],
                                env=environment or self.environment,
                                capture_output=True, check=True, timeout=10)
        self.assertEqual(result.stdout, b"original compiler result\n")
        self.assertEqual(result.stderr, b"")

    def test_existing_diagnostic_bytes_are_preserved(self):
        self.single("first")
        self.single("two lines\nwith an existing final newline\n")
        self.assertEqual(self.trace.read_bytes(), b"first\ntwo lines\nwith an existing final newline\n\n")

    def test_disabled_or_unwritable_trace_preserves_the_result(self):
        environment = dict(self.environment)
        environment.pop("LUSTR_MIXED_TRACE")
        self.single("disabled", environment)
        self.assertFalse(self.trace.exists())
        self.single("unwritable", dict(self.environment, LUSTR_MIXED_TRACE=str(self.root)))
        self.assertFalse(self.trace.exists())

    def test_whole_records_under_concurrent_processes(self):
        workers, count = 8, 300
        processes = []
        try:
            for worker in range(workers):
                processes.append(subprocess.Popen(
                    [str(self.binary), str(worker), str(count), str(self.root)],
                    env=self.environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE))
            deadline = time.monotonic() + 10
            while len(list(self.root.glob("ready-*"))) != workers:
                self.assertLess(time.monotonic(), deadline, "writer start barrier timed out")
                self.assertTrue(all(process.poll() is None for process in processes), "writer exited before the barrier")
                time.sleep(0.01)
            (self.root / "start").write_bytes(b"")
            for process in processes:
                stdout, stderr = process.communicate(timeout=20)
                self.assertEqual(process.returncode, 0, stderr.decode(errors="replace"))
                self.assertEqual(stdout, b"")
        finally:
            for process in processes:
                if process.poll() is None:
                    process.kill()
                process.communicate()
        lines = self.trace.read_text().splitlines()
        requests, diagnostics = [], []
        index = 0
        while index < len(lines):
            line = lines[index]
            request = re.fullmatch(r"request-key=([0-9a-f]{16}) retained=false", line)
            if request:
                requests.append(request[1])
                index += 1
                continue
            diagnostic = re.fullmatch(r"lustr mixed native: unit ([0-9a-f]{16})", line)
            self.assertIsNotNone(diagnostic, f"interleaved or unknown trace record: {line[:200]!r}")
            self.assertLess(index + 2, len(lines), "incomplete multi-line diagnostic")
            prefix = "lustr mixed native profile: "
            self.assertTrue(lines[index + 1].startswith(prefix), "emitter record separated from its diagnostic")
            emitter = json.loads(lines[index + 1][len(prefix):])
            self.assertEqual(emitter["unit_identity"], diagnostic[1])
            self.assertEqual(emitter["elapsed_ns"], sum(phase["elapsed_ns"] for phase in emitter["phases"]))
            self.assertEqual(lines[index + 2], "", "original final newline was interleaved")
            diagnostics.append(diagnostic[1])
            index += 3
        expected = Counter(f"{n:016x}" for n in range(workers * count))
        self.assertEqual(Counter(requests), expected)
        self.assertEqual(Counter(diagnostics), expected)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--rustc", default=RUSTC)
    args, remaining = parser.parse_known_args()
    SOURCE, RUSTC = args.source, args.rustc
    unittest.main(argv=[__file__, *remaining])
