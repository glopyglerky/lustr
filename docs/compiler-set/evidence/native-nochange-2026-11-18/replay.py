#!/usr/bin/env python3
"""Verify the retained focused-test evidence without executing a compiler."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import zipfile

HERE = Path(__file__).resolve().parent
MANIFEST = json.loads((HERE / "manifest.json").read_text())
archive = HERE / "records.zip"
assert hashlib.sha256(archive.read_bytes()).hexdigest() == MANIFEST["archive_sha256"]
assert archive.stat().st_size == MANIFEST["archive_bytes"]
with zipfile.ZipFile(archive) as z:
    assert len(z.namelist()) == len(set(z.namelist()))
    records = {name: z.read(name) for name in z.namelist()}
assert set(records) == set(MANIFEST["files"])
for name, data in records.items():
    assert not name.startswith("/") and ".." not in Path(name).parts
    assert len(data) == MANIFEST["files"][name]["bytes"], name
    assert hashlib.sha256(data).hexdigest() == MANIFEST["files"][name]["sha256"], name

def obj(name):
    return json.loads(records[name])

def command(name, expected):
    base = f"commands/{name}/"
    result = obj(base + "status.json")
    assert result["returncode"] == expected and not result["timed_out"], name
    for stream in ["stdout", "stderr"]:
        assert hashlib.sha256(records[base + stream]).hexdigest() == result[stream + "_sha256"], name
    return obj(base + "command.json"), records[base + "stdout"].decode(), records[base + "stderr"].decode()

expected_codes = {name: 0 for name in MANIFEST["commands"]}
expected_codes.update({"focused-ordinary": 101, "source-guard-default": 1, "final-source-guard-default": 1})
assert set(expected_codes) == set(MANIFEST["commands"])
for name, code in expected_codes.items():
    command(name, code)

initial = obj("source-receipt.json")
corrected = obj("corrected-source-receipt.json")
assert initial["source_commit"] == "3724211b45a3f8cc2bcf8539c8d549c3c2cbae7d"
assert initial["native_tree"] == "94c20e3781d0197ec79d0e0bc7eda10621af1a3c"
assert corrected["source_commit"] == "8b68fee55b01c9bff6faa9be30612910fe6bb0f7"
assert corrected["native_tree"] == "e6eea62c77095e60eaa7b1baf77055f0708ad54e"
assert initial["sdk"] == corrected["sdk"] and initial["tinycc"] == corrected["tinycc"]
for label, receipt in [("initial", initial), ("corrected", corrected)]:
    assert receipt["source_clean"] and receipt["profile"] == "gate"
    assert receipt["compiler_jobs"] == receipt["test_threads"] == 1
    assert not receipt["performance_evidence"]
    for path, digest in receipt["source_files"].items():
        assert hashlib.sha256(records[f"source/{label}/{path}"]).hexdigest() == digest, path

first, first_out, first_err = command("focused-ordinary", 101)
ordinary, ordinary_out, _ = command("focused-corrected-ordinary", 0)
oracle, oracle_out, oracle_err = command("focused-corrected-oracle", 0)
names = [
    "driver::patch::tests::no_change_check_preserves_edit_error_restore_and_build",
    "driver::patch::tests::no_change_check_retains_new_const_observations_and_slot_updates",
]
assert f"test {names[0]} ... ok" in first_out and f"test {names[1]} ... FAILED" in first_out
assert "1 passed; 1 failed; 0 ignored; 0 measured; 387 filtered out" in first_out
assert "called `Option::unwrap()` on a `None` value" in first_err
for out in [ordinary_out, oracle_out]:
    assert all(f"test {name} ... ok" in out for name in names)
    assert "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 387 filtered out" in out
assert oracle_err == ""
assert first["argv"] == ordinary["argv"]
assert ordinary["environment_removed"] == first["environment_removed"]
assert "LUST_CHECK_RESOLVE" in ordinary["environment_removed"]
assert oracle["environment_overrides"]["LUST_CHECK_RESOLVE"] == "1"
binary = obj("corrected-test-executable.json")
assert binary["source_commit"] == corrected["source_commit"]
assert binary["sha256"] == "3b98fee1feccf3c44770cfe6859b3aa6d5e07a907e8a9b79672b523ec2a81e36"
assert oracle["argv"][0] == binary["path"]
assert binary["sha256"] == obj("post-oracle-executable.json")["sha256"]
assert obj("first-test-executable.json")["sha256"] == "ac2e94603437dfa52678b4c43f1964b957ca0b9435c3d625287f6521b2e18838"
lines = records["commands/focused-corrected-oracle/resolve.log"].decode().splitlines()
assert len(lines) == 20 and all(line.startswith("resolve oracle: ok path=") for line in lines)
paths = Counter(line.split("path=", 1)[1].split()[0] for line in lines)
assert paths == {"resolve": 3, "reuse": 7, "patch": 10}
_, guard_out, _ = command("final-source-guard-index", 0)
assert corrected["native_tree"] in guard_out and "candidate source identity only" in guard_out
_, _, default_error = command("final-source-guard-default", 1)
assert "nonempty integration requires exact reviewed qualification" in default_error

print(json.dumps({
    "verified": True, "archive_sha256": MANIFEST["archive_sha256"],
    "files": len(records), "commands": len(MANIFEST["commands"]),
    "initial_tests": {"passed": 1, "failed": 1, "filtered": 387},
    "corrected_tests_per_mode": {"passed": 2, "failed": 0, "filtered": 387},
    "oracle_observations": dict(paths), "native_tree": corrected["native_tree"],
    "full_native_admission": False, "performance_evidence": False,
}, indent=2))
