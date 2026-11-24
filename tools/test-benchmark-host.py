#!/usr/bin/env python3
"""Check host evidence under absent and constrained Linux observations."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("capture_host", Path(__file__).with_name("capture-benchmark-host.py"))
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)


class HostEvidenceTests(unittest.TestCase):
    def test_missing_fields_remain_unknown_without_leaking_environment(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            record = h.collect(root, root, {"SECRET_TOKEN": "must-not-be-captured", "HOSTNAME": "private-host"})
        self.assertIsNone(record["cpu_model"]["value"])
        self.assertIn("unavailable_reason", record["cpu_model"])
        self.assertIsNone(record["physical_memory_bytes"]["value"])
        serialized = json.dumps(record)
        self.assertNotIn("must-not-be-captured", serialized)
        self.assertNotIn("private-host", serialized)

    def test_container_facts_preserve_distinct_memory_and_cpu_limits(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "cpuinfo").write_text("processor : 0\nmodel name : Example CPU\n")
            (root / "meminfo").write_text("MemTotal: 2048 kB\n")
            (root / "fs/cgroup").mkdir(parents=True)
            (root / "fs/cgroup/cpu.max").write_text("150000 100000\n")
            (root / "fs/cgroup/memory.max").write_text("1048576\n")
            with patch.object(h.os, "sched_getaffinity", return_value={3, 1}), patch.object(h.os, "cpu_count", return_value=8):
                record = h.collect(root, root, {"GITHUB_RUN_ID": "42"})
        self.assertEqual(record["physical_memory_bytes"]["value"], 2097152)
        self.assertEqual(record["cgroup_v2_memory_max"]["value"], "1048576")
        self.assertEqual(record["cgroup_v2_cpu_max"]["value"], "150000 100000")
        self.assertEqual(record["logical_cpus"]["value"], 8)
        self.assertEqual(record["process_affinity"]["value"], [1, 3])
        self.assertEqual(record["runner"]["GITHUB_RUN_ID"]["value"], "42")

    def test_malformed_memory_and_unavailable_affinity_are_unknown(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root / "meminfo").write_text("MemTotal: 2048 unexpected-unit\n")
            with patch.object(h.os, "sched_getaffinity", side_effect=OSError("denied")):
                record = h.collect(root, root, {})
        self.assertIsNone(record["physical_memory_bytes"]["value"])
        self.assertIsNone(record["process_affinity"]["value"])


if __name__ == "__main__":
    unittest.main()
