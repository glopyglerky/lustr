#!/usr/bin/env python3
"""Replay the public compact-selector evidence without executing a compiler."""
import hashlib
import io
import json
import math
from pathlib import Path
import random
import statistics
import tarfile

ROOT = Path(__file__).resolve().parent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_bytes())


def members(name):
    with tarfile.open(fileobj=io.BytesIO((ROOT / name).read_bytes()), mode="r:gz") as archive:
        files = {}
        for member in archive.getmembers():
            assert member.isfile() and member.name not in files
            assert not member.name.startswith("/") and ".." not in Path(member.name).parts
            files[member.name] = archive.extractfile(member).read()
        return files


manifest = read("MANIFEST.json")
actual = {str(p.relative_to(ROOT)) for p in ROOT.rglob("*") if p.is_file()}
assert actual == set(manifest["files"]) | {"MANIFEST.json"}
for name, expected in manifest["files"].items():
    data = (ROOT / name).read_bytes()
    assert sha(data) == expected["sha256"] and len(data) == expected["bytes"], name
identity = read("identity.json")
for lane in ("before", "after"):
    source = identity["sources"][lane]
    for path, expected in source["sha256"].items():
        assert sha((ROOT / "source" / lane / path).read_bytes()) == expected
    receipt = read(lane + "-helper-build.json")
    assert receipt["source"] == {k: source[k] for k in ("revision", "tree")}
    assert receipt["binary_sha256"] == source["helper_sha256"]

for name, report_name, fields in (
    ("selector-gate", "results.json", ("stdout_file", "stderr_file")),
    ("compact-gate", "results.json", ("stdout", "stderr")),
    ("component", "report.json", ("stdout", "stderr")),
):
    report = read(name + "/" + report_name)
    assert report["status"] == "completed"
    archive = report["public_projection"]["stream_archive"]
    assert sha((ROOT / name / archive["file"]).read_bytes()) == archive["sha256"]
    streams = members(name + "/" + archive["file"])
    assert len(streams) == archive["entries"]
    assert len(report["records"]) == identity["sets"][name]["records"]
    for row in report["records"]:
        for kind, field in zip(("stdout", "stderr"), fields):
            assert sha(streams[row[field]]) == row["public_" + kind + "_sha256"]
        assert row["exit"] is not None and row["exit"] >= 0
        if name == "selector-gate":
            decoded = json.loads(streams[row[fields[0]]])
            decoded.pop("elapsed_ns")
            assert decoded == row["selection"]
        elif name == "compact-gate" and row["kind"] == "runtime":
            assert row["exit"] == 0 and streams[row["stderr"]] == b""
            assert streams[row["stdout"]] in (b"selected-first:debug\n", b"selected-first:nodebug\n", b"selected-second\n")
            assert len(row["executable_sha256"]) == 64
        elif name == "component":
            decoded = json.loads(streams[row["stdout"]])
            assert decoded["success"] and decoded["elapsed_ns"] == row["selection_ns"]
            assert row["exit"] == 0 and row["timeout"] is None
            if row["phase"] != "prime":
                assert row["cache"] == "hit" and streams[row["stderr"]] == b""
    if "assertions" in report:
        assert len(report["assertions"]) == identity["sets"][name]["assertions"]
        assert all(item["passed"] for item in report["assertions"])

component = read("component/report.json")
plan = read("component/predeclared-plan.json")
assert component["schedule"]["measured_pairs"] == plan["measured_pairs"] == 96
assert component["schedule"]["excluded_pairs"] == plan["excluded_pairs"] == 16
assert sha((ROOT / "component/executed-runner.py").read_bytes()) == component["runner_sha256"] == plan["runner_sha256"]
measured = [row for row in component["records"] if row["phase"] == "measured"]
assert len(measured) == 192
paired = [{row["label"]: row["selection_ns"] for row in measured if row["pair"] == pair} for pair in range(96)]
assert all(set(pair) == {"before", "after"} for pair in paired)
logs = [math.log(pair["before"] / pair["after"]) for pair in paired]
rng = random.Random(91573)
bootstrap = sorted(math.exp(statistics.mean(rng.choices(logs, k=len(logs)))) for _ in range(10000))
assert math.exp(statistics.mean(logs)) == component["summary"]["paired_geometric_ratio"]
assert [bootstrap[250], bootstrap[9749]] == component["summary"]["bootstrap_95"]
for lane in ("before", "after"):
    assert statistics.median(row["selection_ns"] for row in measured if row["label"] == lane) == component["summary"][lane]["median_ns"]
fixture = members("component/fixture.tar.gz")
assert {name: sha(content) for name, content in fixture.items()} == component["fixture_sha256"]
assert component["revisions"] == {lane: identity["sources"][lane]["revision"] for lane in ("before", "after")}
assert component["helper_sha256"] == {lane: identity["sources"][lane]["helper_sha256"] for lane in ("before", "after")}
print("Verified source/build bindings, 140/265 selector gate, 103/486 compact/fallback gate, and one 96-pair component comparison.")
