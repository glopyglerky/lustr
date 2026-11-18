#!/usr/bin/env python3
"""Replay effective-access evidence without executing a compiler."""
import hashlib
import io
import json
from pathlib import Path
import tarfile

root = Path(__file__).resolve().parent
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = json.loads((root / "MANIFEST.json").read_bytes())
actual = {str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()}
assert actual == set(manifest["files"]) | {"MANIFEST.json"}
for name, expected in manifest["files"].items():
    data = (root / name).read_bytes()
    assert sha(data) == expected["sha256"] and len(data) == expected["bytes"], name
identity = json.loads((root / "identity.json").read_bytes())
for name, expected in identity["source_sha256"].items():
    assert sha((root / "source" / name).read_bytes()) == expected
build = json.loads((root / "helper-build.json").read_bytes())
assert build["status"] == "completed" and build["source"] == identity["product"]
assert build["binary_sha256"] == identity["helper_sha256"]
assert sha((root / "validated-source/tools/test-native-selection-cache.py").read_bytes()) == identity["test_sha256"]
report = json.loads((root / "results.json").read_bytes())
assert report["status"] == "completed" and report["success"]
assert len(report["records"]) == identity["invocations"] == 154
assert len(report["assertions"]) == identity["assertions"] == 290
assert all(a["passed"] for a in report["assertions"])
assert sha((root / "streams.tar.gz").read_bytes()) == report["public_projection"]["archive_sha256"]
with tarfile.open(fileobj=io.BytesIO((root / "streams.tar.gz").read_bytes()), mode="r:gz") as archive:
    streams = {}
    for member in archive.getmembers():
        assert member.isfile() and member.name not in streams
        assert not member.name.startswith("/") and ".." not in Path(member.name).parts
        streams[member.name] = archive.extractfile(member).read()
for row in report["records"]:
    for kind in ("stdout", "stderr"):
        assert sha(streams[row[kind + "_file"]]) == row["public_" + kind + "_sha256"]
    decoded = json.loads(streams[row["stdout_file"]])
    decoded.pop("elapsed_ns")
    assert decoded == row["selection"]
for candidate, reference in zip(report["records"][::2], report["records"][1::2]):
    assert candidate["exit"] == reference["exit"]
    assert candidate["selection"] == reference["selection"]
    assert streams[candidate["stderr_file"]] == streams[reference["stderr_file"]]
print("Verified exact access-fix source/helper, 154 invocations, 77 cached/fresh pairs and 290 assertions.")
