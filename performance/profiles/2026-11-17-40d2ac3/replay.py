#!/usr/bin/env python3
"""Reproduce the labeled reconstruction without changing the original evidence.

This reads the retained original ZIP and the frozen strict reporter. It does not
run a compiler or turn the original failed CI profile into a passed run.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import shutil
import sys
import tempfile
import zipfile


sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def relative(name):
    path = PurePosixPath(name)
    require(name and not path.is_absolute() and ".." not in path.parts and "\\" not in name,
            "invalid evidence-relative path")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, default=ROOT.parents[1] / "archives/2026-11-17-40d2ac3-qualification.zip")
    args = parser.parse_args()
    for line in (ROOT / "SHA256SUMS").read_text().splitlines():
        expected, name = line.split("  ", 1)
        require(sha256((ROOT / relative(name)).read_bytes()) == expected,
                f"bundle checksum mismatch: {name}")
    receipt = json.loads((ROOT / "reconstruction.json").read_bytes())
    inventory = json.loads((ROOT / "original-inputs.json").read_bytes())
    require(sha256(args.archive.read_bytes()) == receipt["artifact_sha256"] == inventory["archive_sha256"],
            "original archive digest mismatch")
    reporter_path = ROOT / "source/profile-mixed-compiler.py"
    require(sha256(reporter_path.read_bytes()) == receipt["reporter_source_sha256"],
            "recorded reporter digest mismatch")
    spec = importlib.util.spec_from_file_location("recorded_profile_reporter", reporter_path)
    reporter = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reporter)
    with tempfile.TemporaryDirectory(prefix="lustr-profile-reconstruction-") as directory:
        original = Path(directory) / "original"
        original.mkdir()
        with zipfile.ZipFile(args.archive) as archive:
            require(len(archive.namelist()) == len(set(archive.namelist())), "duplicate ZIP member")
            prefix = inventory["prefix"]
            actual = {name[len(prefix):] for name in archive.namelist() if name.startswith(prefix) and not name.endswith("/")}
            require(actual == {row["path"] for row in inventory["files"]}, "original input inventory mismatch")
            for row in inventory["files"]:
                path = relative(row["path"])
                data = archive.read(prefix + str(path))
                require(len(data) == row["bytes"] and sha256(data) == row["sha256"],
                        f"original input digest or size mismatch: {path}")
                target = original / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
        for path in (original / "profiles").glob("*.jsonl"):
            reporter.read_profile(path)
        try:
            reporter.summarize(original)
        except ValueError as error:
            require(str(error) == receipt["original_validation_error"], "original profiling failure changed")
        else:
            raise ValueError("original CI profile unexpectedly passed")
        derived = Path(directory) / "derived"
        shutil.copytree(original, derived)
        path = derived / relative(receipt["changed_relative_path"])
        before = path.read_bytes()
        old, new = receipt["original_bytes"].encode(), receipt["reconstructed_bytes"].encode()
        require(sha256(before) == receipt["original_trace_sha256"] and before.count(old) == 1,
                "original trace digest or framing replacement mismatch")
        path.write_bytes(before.replace(old, new, 1))
        require(sha256(path.read_bytes()) == receipt["derived_trace_sha256"], "reconstructed trace digest mismatch")
        report = reporter.summarize(derived)
        require((json.dumps(report, indent=2) + "\n").encode() == (ROOT / "derived-summary.json").read_bytes(),
                "derived summary mismatch")
        changed = []
        for row in inventory["files"]:
            require(sha256((original / row["path"]).read_bytes()) == row["sha256"], "original input changed during replay")
            if (derived / row["path"]).read_bytes() != (original / row["path"]).read_bytes():
                changed.append(row["path"])
        require(changed == [receipt["changed_relative_path"]], "reconstruction changed unexpected inputs")
        require(json.loads((original / "run.json").read_bytes())["completed"] is False,
                "original profiling completion status changed")
        require((original / "run.json").read_bytes() == (derived / "run.json").read_bytes(),
                "reconstruction changed the original run record")
    print(json.dumps({"result": "retained derived reconstruction reproduced exactly",
                      "original_ci_profile": "failed as recorded",
                      "original_inputs_verified": len(inventory["files"]),
                      "profiles": len(report["invocations"]), "commands": len(report["modes"]),
                      "native_successful_records": sum(mode["native_emitter"]["measured_count"] for mode in report["modes"].values()),
                      "new_compiler_qualification": False, "paired_speed_evidence": False}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, zipfile.BadZipFile) as error:
        print(f"profile reconstruction rejected: {error}", file=sys.stderr)
        sys.exit(1)
