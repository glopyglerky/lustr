#!/usr/bin/env python3
"""Check the literal workflow path filters that protect compiler inputs.

These controls check trigger coverage, not branch protection or compiler success.
They deliberately accept only the simple positive path lists used here.
"""
import fnmatch
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]


def filters(name, event, text=None):
    if text is None:
        text = (ROOT / ".github/workflows" / name).read_text()
    trigger = text.split("\non:\n", 1)[1].split("\npermissions:", 1)[0]
    event_block = re.search(rf"(?m)^  {event}:\n(.*?)(?=^  \w|\Z)", trigger, re.S)
    if event_block is None:
        raise ValueError(f"{name}: missing {event} trigger")
    if re.search(r"(?m)^    paths-ignore\s*:", event_block[1]):
        raise ValueError(f"{name}: paths-ignore cannot provide positive trigger coverage")
    path_blocks = re.findall(r"(?m)^    paths:\n(.*?)(?=^    \S|\Z)", event_block[1], re.S)
    if len(path_blocks) != 1:
        raise ValueError(f"{name}: expected exactly one positive paths block")
    lines = [line for line in path_blocks[0].splitlines() if line.strip()]
    if any(re.fullmatch(r"      - '[^']+'", line) is None for line in lines):
        raise ValueError(f"{name}: unsupported positive path-list syntax")
    paths = [line.removeprefix("      - '").removesuffix("'") for line in lines]
    if not paths or any(path.startswith("!") for path in paths):
        raise ValueError(f"{name}: expected positive literal path filters")
    return paths


def matches_path(path, pattern):
    """Match the simple literal/*/terminal-** patterns used in these workflows."""
    parts = pattern.split("/")
    if any(re.fullmatch(r"[A-Za-z0-9._*-]+", part) is None for part in parts):
        raise ValueError(f"unsupported workflow path pattern: {pattern}")
    recursive = parts[-1] == "**"
    prefix = parts[:-1] if recursive else parts
    if any("**" in part for part in prefix):
        raise ValueError(f"only a terminal ** component is supported: {pattern}")
    components = path.split("/")
    if len(components) < len(prefix) or (not recursive and len(components) != len(prefix)):
        return False
    return all(fnmatch.fnmatchcase(component, part) for component, part in zip(components, prefix))


class WorkflowInputControls(unittest.TestCase):
    def assert_covered(self, workflow, event, paths):
        patterns = filters(workflow, event)
        for path in paths:
            with self.subTest(workflow=workflow, event=event, path=path):
                self.assertTrue(any(matches_path(path, pattern) for pattern in patterns),
                                f"{path} does not trigger {workflow} ({event})")

    def test_paths_ignore_cannot_replace_positive_trigger_coverage(self):
        text = (ROOT / ".github/workflows/compiler-set.yml").read_text()
        for event in ("push", "pull_request"):
            with self.subTest(event=event):
                with self.assertRaisesRegex(ValueError, "paths-ignore"):
                    filters("compiler-set.yml", event, text.replace("    paths:\n", "    paths-ignore:\n"))

    def test_single_star_cannot_claim_recursive_source_coverage(self):
        text = (ROOT / ".github/workflows/compiler-set.yml").read_text()
        path = "crates/new/src/lib.rs"
        for event in ("push", "pull_request"):
            with self.subTest(event=event):
                original = filters("compiler-set.yml", event, text)
                changed = filters("compiler-set.yml", event, text.replace("'crates/**'", "'crates/*'"))
                self.assertTrue(any(matches_path(path, pattern) for pattern in original))
                self.assertFalse(any(matches_path(path, pattern) for pattern in changed))
        self.assertTrue(matches_path("tools/test-warm-applicability.py", "tools/*warm*"))
        self.assertFalse(matches_path("tools/nested/test-warm-applicability.py", "tools/*warm*"))

    def test_compiler_set_covers_the_complete_source_guard_input_families(self):
        families = ["Cargo.toml", "Cargo.lock", "build.rs", ".cargo/config.toml", ".gitmodules",
                    "rust-toolchain", "rust-toolchain.toml", "examples/native_package.rs",
                    "examples/new.rs", "library/new/src/lib.rs", "src/new.rs",
                    "tests/new.rs", "compiler/native/src/new.rs", "crates/new/Cargo.toml",
                    "crates/new/src/lib.rs", "upstream/bluefe-core.pin", "crates/lustr-semantic/rust-toolchain.toml",
                    "upstream/new-import-schema.json", "tools/new-source-check.py",
                    ".github/workflows/compiler-set.yml", "tools/test-workflow-inputs.py",
                    "tools/test-retired-acquisition.py"]
        for event in ("push", "pull_request"):
            self.assert_covered("compiler-set.yml", event, families)

    def test_historical_replay_changes_trigger_their_own_controls(self):
        common = ["tools/run-upstream-slice-benchmark.py", "tools/report-mixed-performance.py",
                  "tools/capture-benchmark-host.py", "tools/test-workflow-inputs.py"]
        for workflow in ("warm-entry-performance.yml", "warm-native-performance.yml"):
            self.assert_covered(workflow, "pull_request", common + ["tools/run-warm-entry-comparison.py"])
        self.assert_covered("warm-native-performance.yml", "pull_request",
                            ["tools/check-warm-applicability.py", "tools/test-warm-applicability.py"])

    def test_native_probe_controls_cover_their_input_schema_and_package(self):
        self.assert_covered("native-probe-tests.yml", "pull_request",
                            ["crates/lustr-bench/Cargo.toml", "crates/lustr-bench/Cargo.lock",
                             ".gitmodules", "rust-toolchain", "rust-toolchain.toml",
                             "crates/lustr-bench/sample-spec.json", "tools/test-retired-acquisition.py",
                             "tools/test-workflow-inputs.py", "tools/assess-current-performance.py",
                             "tools/qualify-retained-results.py", "upstream/retained-scenario-results.json"])


if __name__ == "__main__":
    unittest.main()
