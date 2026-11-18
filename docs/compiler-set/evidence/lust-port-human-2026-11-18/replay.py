#!/usr/bin/env python3
"""Verify preserved hosted/local lust-port outcomes without rerunning them."""
import hashlib
import json
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
m = json.loads((HERE / "manifest.json").read_text())
archive = HERE / "records.zip"
assert hashlib.sha256(archive.read_bytes()).hexdigest() == m["archive_sha256"]
assert archive.stat().st_size == m["archive_bytes"]
with zipfile.ZipFile(archive) as z:
    assert len(z.namelist()) == len(set(z.namelist()))
    files = {name: z.read(name) for name in z.namelist()}
assert set(files) == set(m["files"])
for name, data in files.items():
    assert not name.startswith("/") and ".." not in Path(name).parts
    assert hashlib.sha256(data).hexdigest() == m["files"][name]["sha256"]
    assert len(data) == m["files"][name]["bytes"]
obj = lambda name: json.loads(files[name])
for name in m["commands"]:
    base = f"commands/{name}/"
    s = obj(base + "status.json")
    expected = 1 if name in {"source-guard-default", "final-source-guard-default"} else 0
    assert s["returncode"] == expected and not s["timed_out"], name
    for stream in ["stdout", "stderr"]:
        assert hashlib.sha256(files[base + stream]).hexdigest() == s[stream + "_sha256"]

original = obj("original-source-receipt.json")
instrumented = obj("instrumented-source-receipt.json")
assert original["source_commit"] == "ff9c0de9d2a2c89edd8df48e74e64a5c9483f232"
assert original["executed_hosted_commit"] == "c3513db6bc32a57ec4ae48bc0e0c504582d3a5b5"
assert original["source_tree"] == original["executed_hosted_tree"] == "52ab272cbed5d88b7126d35da5a7f7ff2607cc24"
assert instrumented["source_commit"] == "fc94bc2b892a7e34e3133d4c6fb1e19a2e3ee8b0"
assert original["sdk"] == instrumented["sdk"]
assert "b940084d7eb6a299eb4bfeb8e34901bc051e7ac4" in original["sdk"]["rustc_verbose"]
for label, receipt in [("original", original), ("instrumented", instrumented)]:
    for name, digest in receipt["source_files"].items():
        assert hashlib.sha256(files[f"source/{label}/{name}"]).hexdigest() == digest
assert not instrumented["assertions_changed"]
assert instrumented["scanner_executable_unchanged"] and not instrumented["native_compiler_behavior_changed"]
assert instrumented["scanner_executable_sha256"] == "5cb69d5e5d797afd3da06657f4d90fa4347ea89a60b430d79da628e4e22dc7b6"
source = "compiler/native/tools/lust-port/tests/scan_fixture.rs"
before = files["source/original/" + source].decode()
after = files["source/instrumented/" + source].decode()
start = after.index('    eprintln!(\n        "lust-port human report, fixture ')
end = after.index('    );\n', start) + len('    );\n')
assert after[:start] + after[end:] == before

hosted = files["hosted/full-job.log"]
assert len(hosted) == 167361
assert hashlib.sha256(hosted).hexdigest() == "83177df25fb74c79539c2d5197bc16c0c0223c7d0b06fbd5e20b6a8ac720a140"
assert hashlib.sha256(files["hosted/artifact.zip"]).hexdigest() == "0093ad3c49171757f0ca9c6e8cbf884216677338eac786b5b19439b0f8d16478"
assert b'assertion failed: text.contains("attrs.unsupported")' in hosted
assert b"test result: FAILED. 4 passed; 1 failed" in hosted
ordinary = files["commands/original-tests/stdout"].decode()
assert "test result: ok. 8 passed; 0 failed" in ordinary
assert "test result: ok. 5 passed; 0 failed" in ordinary
diagnostic = files["commands/instrumented-human-test/stdout"].decode()
assert "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out" in diagnostic
raw = files["commands/instrumented-human-test/stderr"].decode()
assert "status: exit status: 0" in raw
begin = raw.index("\nstdout:\n") + len("\nstdout:\n")
end = raw.index("\nstderr:\n", begin)
report = raw[begin:end]
assert report.encode() == files["instrumented-human-report.txt"]
assert report.count("attrs.unsupported") == 2
assert "files: 7 (18 lines, 6 functions), findings: 3 in 3 files, parse errors: 1" in report
assert "findings 3 (active 1, target-specific 1, test-only 1)" in report
assert "lust-port: 7 files, 3 findings, 1 parse errors" in raw[end:]
assert "candidate source identity only" in files["commands/final-source-guard-index/stdout"].decode()
assert not m["hosted_failure_resolved"] and not m["scanner_fix_claimed"]
print(json.dumps({"verified": True, "archive_sha256": m["archive_sha256"],
    "files": len(files), "commands": len(m["commands"]),
    "hosted_tool_tests": {"passed": 12, "failed": 1},
    "original_local_tests": {"passed": 13, "failed": 0},
    "instrumented_local_test": {"passed": 1, "failed": 0, "filtered": 4},
    "captured_feature_occurrences": 2, "hosted_failure_resolved": False,
    "scanner_fix_claimed": False}, indent=2))
