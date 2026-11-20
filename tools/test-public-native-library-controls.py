#!/usr/bin/env python3
"""Mutation controls for the public library evaluator, executed only in CI."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("public_library_evaluator", ROOT / "tools/test-public-native-library.py")
campaign = importlib.util.module_from_spec(spec)
spec.loader.exec_module(campaign)


def exec_line(pid, path, argv, env=("PATH=/usr/bin:/bin",), result="0"):
    return f"{pid} execve({json.dumps(path)}, {json.dumps(argv)}, {json.dumps(list(env))}) = {result}\n"


class ProcessEvidence(unittest.TestCase):
    def test_split_creation_binds_child_to_actual_parent(self):
        data = exec_line(10, "/owned/cargo", ["/owned/cargo", "build"])
        data += "10 clone(child_stack=NULL, flags=CLONE_VM|CLONE_VFORK|SIGCHLD <unfinished ...>\n"
        data += '11 chdir("/owned/stage") = 0\n'
        data += exec_line(11, "/owned/tool", ["/owned/tool", "object.o"])
        data += "10 <... clone resumed>) = 11\n11 +++ exited with 0 +++\n10 +++ exited with 0 +++\n"
        parsed = campaign.trace_events(data.encode(), Path("/owned"))
        self.assertEqual(parsed["parents"], {"11": "10"})
        self.assertEqual(parsed["attempts"][1]["cwd"], "/owned/stage")
        self.assertEqual(parsed["exits"], {"11": 0, "10": 0})

    def test_unfinished_execution_cannot_cross_response(self):
        data = exec_line(10, "/owned/cargo", ["/owned/cargo"])
        data += '11 execve("/owned/tool", ["/owned/tool"], ["PATH=/bin"] <unfinished ...>\n'
        with self.assertRaisesRegex(AssertionError, "crosses"):
            campaign.trace_events(data.encode(), Path("/owned"))

    def test_shortened_environment_cannot_prove_ownership(self):
        data = b'10 execve("/owned/cargo", ["/owned/cargo"], 0x123 /* 14 vars */) = 0\n'
        with self.assertRaises((AssertionError, ValueError)):
            campaign.trace_events(data, Path("/owned"))

    def test_failed_compiler_attempt_is_preserved(self):
        data = exec_line(10, "/owned/cargo", ["/owned/cargo"])
        data += exec_line(10, "/hidden/rustc", ["/hidden/rustc", "input.rs"], result="-1 ENOENT (No such file or directory)")
        parsed = campaign.trace_events(data.encode(), Path("/owned"))
        self.assertFalse(parsed["attempts"][1]["succeeded"])
        self.assertEqual(parsed["attempts"][1]["filename"], "/hidden/rustc")

    def test_late_prefix_rewrite_is_refused(self):
        with self.assertRaisesRegex(AssertionError, "rewritten"):
            campaign.frozen_prefix(b"complete original\n", b"complete modified\n")

    def test_extra_allowlisted_artifact_call_is_refused(self):
        call = {"argv": ["/owned/llvm-ar", "crsD", "--format=gnu", "owned.rlib", "owned.o"], "cwd": "/owned/stage"}
        campaign.reconcile_artifact_calls([call], [call])
        with self.assertRaisesRegex(AssertionError, "actual per-request"):
            campaign.reconcile_artifact_calls([call, call], [call])

    def test_descriptor_execution_is_not_silently_skipped(self):
        data = exec_line(10, "/owned/cargo", ["/owned/cargo"])
        data += '10 execveat(3, "", ["rustc"], ["PATH=/bin"], AT_EMPTY_PATH) = 0\n'
        with self.assertRaisesRegex(AssertionError, "unrepresented"):
            campaign.trace_events(data.encode(), Path("/owned"))


class ArtifactAndDiagnosticEvidence(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.source = self.root / "implementation" / "leaf entry.rs"
        self.source.parent.mkdir()
        self.source.write_bytes(campaign.INVALID)
        self.outputs = {"link": self.root / "libnative_leaf-x.rlib",
                        "metadata": self.root / "libnative_leaf-x.rmeta",
                        "dep-info": self.root / "native_leaf-x.d"}

    def tearDown(self):
        self.directory.cleanup()

    def dependency_bytes(self):
        source = str(self.source).replace(" ", "\\ ")
        return ("".join(f"{path}: {source}\n\n" for path in self.outputs.values()) + source + ":\n").encode()

    def test_exact_dep_info_tracks_actual_source_and_all_outputs(self):
        actual = campaign.dep_info(self.dependency_bytes(), self.outputs, self.source, self.root)
        self.assertEqual(actual[str(self.outputs["metadata"])], [str(self.source)])

    def test_missing_metadata_dependency_target_is_refused(self):
        data = self.dependency_bytes()
        line = f"{self.outputs['metadata']}: {str(self.source).replace(' ', chr(92) + ' ')}\n\n".encode()
        with self.assertRaisesRegex(AssertionError, "targets"):
            campaign.dep_info(data.replace(line, b""), self.outputs, self.source, self.root)

    def test_invented_manifest_dependency_is_refused(self):
        data = self.dependency_bytes().replace(b"\n\n", b" Cargo.toml\n\n", 1)
        with self.assertRaisesRegex(AssertionError, "prerequisites"):
            campaign.dep_info(data, self.outputs, self.source, self.root)

    def diagnostic(self):
        text = campaign.INVALID
        start = text.index(b"true")
        line = text[:start].count(b"\n") + 1
        column = start - text.rfind(b"\n", 0, start)
        return {"$message_type": "diagnostic", "level": "error", "code": {"code": "L0110", "explanation": None},
                "message": "mismatched types: expected `u64`, found `bool`", "children": [], "rendered": "error\n",
                "spans": [{"file_name": str(self.source), "is_primary": True, "byte_start": start, "byte_end": start + 4,
                           "line_start": line, "line_end": line, "column_start": column, "column_end": column + 4}]}

    def test_primary_span_is_bound_to_exact_invalid_expression(self):
        error = self.diagnostic()
        campaign.diagnostic([error], self.source, campaign.INVALID, "L0110")
        error["spans"][0]["byte_start"] += 1
        with self.assertRaisesRegex(AssertionError, "exact primary"):
            campaign.diagnostic([error], self.source, campaign.INVALID, "L0110")

    def test_extra_coded_error_cannot_ride_alongside_expected_error(self):
        expected = self.diagnostic()
        extra = self.diagnostic()
        extra["code"]["code"] = "L9999"
        with self.assertRaisesRegex(AssertionError, "exactly one"):
            campaign.diagnostic([expected, extra], self.source, campaign.INVALID, "L0110")

    def test_generic_refusal_is_not_a_type_error(self):
        error = self.diagnostic()
        error["code"]["code"] = "integration-incomplete"
        with self.assertRaisesRegex(AssertionError, "authoritative type"):
            campaign.diagnostic([error], self.source, campaign.INVALID, "L0110")


if __name__ == "__main__":
    unittest.main()
