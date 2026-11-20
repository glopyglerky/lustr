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

    def test_actual_cargo_split_request_preserves_flags_and_empty_search_inventory(self):
        # Flags and identities are from the retained ca383 cold argv; only this
        # temporary fixture's source/build paths differ from the hosted paths.
        build = self.root / "intermediates"
        argv = ["/installed/rustc", "--crate-name", campaign.CRATE, "--edition=2024",
                str(self.source.relative_to(self.root)), "--error-format=json", "--json=" + campaign.JSON_OPTIONS,
                "--crate-type", "lib", "--emit=dep-info,metadata,link", "-Z", "embed-metadata=no",
                "-C", "panic=abort", "-C", "embed-bitcode=no",
                "--check-cfg", campaign.CHECK_CFG[0], "--check-cfg", campaign.CHECK_CFG[1],
                "-C", "metadata=24b6c5ad5c09908c", "-C", "extra-filename=-8ede6661f55aba71",
                "--out-dir", str(build / "debug/build/public-leaf-fixture/8ede6661f55aba71/out"),
                "--force-warn=unused_crate_dependencies"]
        project = {"source": self.source, "build": build}
        def decode(arguments):
            return campaign.declared_compile(campaign.parse_compile({"argv": arguments, "cwd": str(self.root)}), project)
        outputs = decode(argv)
        self.assertEqual(outputs["metadata"].name, "libnative_leaf-8ede6661f55aba71.rmeta")
        self.assertEqual(campaign.parse_compile({"argv": argv, "cwd": str(self.root)})["search_dirs"], [])
        facts = campaign.parse_compile({"argv": argv, "cwd": str(self.root)})
        for choice, expected in (("no", False), (None, None), ("yes", True)):
            changed = campaign.metadata_replay_argv({"argv": argv}, choice)
            observed = campaign.parse_compile({"argv": changed, "cwd": str(self.root)})
            self.assertIs(campaign.library_options(observed)["embed_metadata"], expected)
            self.assertEqual({key: value for key, value in observed.items() if key != "unstable"},
                             {key: value for key, value in facts.items() if key != "unstable"})
        altered = []
        for value in ("embed-metadata=yes", "unknown-unstable=no"):
            changed = list(argv)
            changed[changed.index("embed-metadata=no")] = value
            altered.append(changed)
        altered.append(argv[:argv.index("-Z")] + argv[argv.index("-Z") + 2:])
        altered.append(argv[:-1] + ["--force-warn=unused_variables"])
        altered.append(argv + ["-L", "dependency=" + str(self.root / "invented-dependency")])
        altered.append(argv + ["-Z", "embed-metadata=no"])
        altered.append(argv + ["-C", "embed-metadata=no"])  # Separate upstream channel remains refused.
        for arguments in altered:
            with self.subTest(arguments=arguments):
                with self.assertRaises(AssertionError):
                    decode(arguments)

    def test_split_metadata_reference_rejects_hash_format_and_identity_mutations(self):
        # Synthetic bytes exercise rejection paths only. Qualification requires
        # the original Rust-produced reference bytes and genuine metadata reader.
        def payload(svh, stub):
            return (b"rust\0\0\0\x0c" + (40).to_bytes(8, "little")
                    + (0 if stub else 64).to_bytes(8, "little") + svh
                    + b"fixture-only crate/target/version" + (b"" if stub else b"x" * 96) + b"rust-end-file")
        full, stub = payload(b"n" * 16, False), payload(b"n" * 16, True)
        reference_full, reference_stub = payload(b"r" * 16, False), payload(b"r" * 16, True)
        evidence = campaign.compare_split_metadata(full, stub, reference_full, reference_stub)
        self.assertTrue(evidence["equal_except_crate_hash"])
        mutations = [
            (full, full, reference_full, reference_stub),
            (full, stub[:24] + b"x" * 16 + stub[40:], reference_full, reference_stub),
            (full, b"wronghdr" + stub[8:], reference_full, reference_stub),
            (full, stub[:44] + b"X" + stub[45:], reference_full, reference_stub),
            (full, stub[:-13] + b"extra" + stub[-13:], reference_full, reference_stub),
            (full, stub, reference_full, reference_stub[:24] + b"x" * 16 + reference_stub[40:]),
            (full[:16] + bytes(8) + full[24:], stub, reference_full, reference_stub),
            (full, stub[:8] + (1).to_bytes(8, "little") + stub[16:], reference_full, reference_stub),
        ]
        for index, values in enumerate(mutations):
            with self.subTest(mutation=index):
                with self.assertRaises(AssertionError):
                    campaign.compare_split_metadata(*values)

    def test_forced_unused_dependency_json_requires_one_exact_empty_result(self):
        expected = {"$message_type": "unused_extern", "lint_level": "force-warn", "unused_extern_names": []}
        self.assertEqual(campaign.unused_dependencies([expected]), expected)
        for messages in ([], [expected, expected],
                         [dict(expected, lint_level="warn")],
                         [dict(expected, unused_extern_names=["native_leaf"])],
                         [dict(expected, unused_extern_names=None)],
                         [dict(expected, unexpected=True)],
                         [dict(expected, **{"$message_type": "unused_externs"})]):
            with self.subTest(messages=messages):
                with self.assertRaisesRegex(AssertionError, "exactly one original empty"):
                    campaign.unused_dependencies(messages)


if __name__ == "__main__":
    unittest.main()
