#!/usr/bin/env python3
"""Replay archived race evidence; executable payloads are not in this bundle."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((HERE / "manifest.json").read_text())
archive = (HERE / "records.zip").read_bytes()
assert sha(archive) == manifest["archive_sha256"] and len(archive) == manifest["archive_bytes"]
assert manifest["binary_payloads"] == "local-only; omitted from compact archive"
with zipfile.ZipFile(HERE / "records.zip") as z:
    assert len(z.namelist()) == len(set(z.namelist()))
    files = {name: z.read(name) for name in z.namelist()}
assert set(files) == set(manifest["files"])
for name, data in files.items():
    assert not Path(name).is_absolute() and ".." not in Path(name).parts
    assert sha(data) == manifest["files"][name]["sha256"] and len(data) == manifest["files"][name]["bytes"]
obj = lambda name: json.loads(files[name])
expected = {"build-guard", "candidate-guard", "default-guard", "run-fixed-schedule", "corrected-local-replay",
            "build-final-guard", "candidate-final-guard", "default-final-guard"}
actual = {name.split("/")[2] for name in files if name.startswith("sourcecheck/commands/")}
assert actual == expected
for name in expected:
    prefix = "sourcecheck/commands/" + name + "/"
    status = obj(prefix + "status.json")
    assert status["returncode"] == (1 if name in {"default-guard", "default-final-guard", "run-fixed-schedule"} else 0)
    assert not status["timed_out"] and status["start_error"] is None
    for stream in ["stdout", "stderr"]:
        assert sha(files[prefix + stream]) == status[stream + "_sha256"]
raw_failure = files["sourcecheck/commands/run-fixed-schedule/stderr"].decode()
assert 'else "files: 0 (0 lines, 0 functions), findings: 0 in 0 files, parse errors: 0")' in files["fixed/runner.py"].decode()
assert "AssertionError" in raw_failure and 'File "' in raw_failure and "verify(args.root)" in raw_failure
assert sha(files["fixed/runner.py"]) == "c7e9dce988deb4d52911fe9ba34da4645fcd505d3d855c94c586dc9c8d7156c7"
assert obj("fixed/report.json")["status"] == "incomplete"
local = obj("sourcecheck/commands/corrected-local-replay/stdout")
assert local["verified"] and local["binary_payloads_rehashed"] and local["original_postcheck_failure_preserved"]
identity = obj("sourcecheck/corrected-replay-identity.json")
assert identity["runner_sha256"] == sha(files["corrected-replay-runner.py"])
assert identity["original_runner_sha256"] == sha(files["fixed/runner.py"])
for label, source_dir, receipt in [("before", "source-before", "identities.json"), ("final", "source-final", "final-identities.json")]:
    recorded = obj("sourcecheck/" + receipt)
    for name, digest in recorded["source"].items():
        assert sha(files["sourcecheck/" + source_dir + "/" + name]) == digest
    assert recorded["native_tree"] == "a518ff5fe17f549cda90e50990caaf987659a9f3"
assert "candidate source identity only" in files["sourcecheck/commands/candidate-final-guard/stdout"].decode()
assert "nonempty integration requires exact reviewed qualification" in files["sourcecheck/commands/default-final-guard/stderr"].decode()
assert manifest["prior_evidence_archive_sha256"] == "2d03052a61241740dbe9da525c71e262bcc9dca10adecdb7dcc3d62b5f0d54ef"
with tempfile.TemporaryDirectory(prefix="lust-port-race-replay-") as scratch:
    root = Path(scratch)
    for name, data in files.items():
        p = root / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(data)
    spec = importlib.util.spec_from_file_location("race_replay", root / "corrected-replay-runner.py")
    module = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(module)
    module.verify(root / "fixed", preserved_postcheck_failure=True, binary_receipts_only=True)
print(json.dumps({"archive_verified": True, "files": len(files), "recorded_commands": 19,
    "executed_candidate": "ca032fe45fff0931dab84958296a5e761fae68b2",
    "binary_payloads": manifest["binary_payloads"], "portable_binary_payloads_rehashed": False,
    "original_postcheck_failure_preserved": True, "historical_scheduling_proven": False}, indent=2))
