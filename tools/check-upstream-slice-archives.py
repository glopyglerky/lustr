#!/usr/bin/env python3
"""Replay original, completed upstream-slice archives without executing them.

The source-built runner verifies its evidence immediately. This independent
repository gate prevents original evidence from disappearing or being silently
replaced after GitHub Actions artifacts expire.
"""

import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import sys
import tempfile
import zipfile


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("upstream_slice", ROOT / "tools/run-upstream-slice-benchmark.py")
slice_bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(slice_bench)


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def archive_sha256(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def replay_manifest(path):
    manifest = json.loads(path.read_text())
    require(manifest.get("schema") == "lustr-public-slice-archive-v1", "unsupported completed slice manifest")
    require(manifest["repository"] == "glopyglerky/lustr", "unexpected source repository")
    require(manifest["status"] == "completed", "failed slice cannot be relabeled completed")
    require(manifest["campaign_schema"] == "lustr-public-slice-v1", "unexpected slice schema")
    require(all(len(manifest[key]) == 40 and all(c in "0123456789abcdef" for c in manifest[key])
                for key in ("head_sha", "measured_checkout", "measured_tree")), "invalid source identity")
    archive = path.parent / manifest["archive"]
    require(archive.parent.resolve() == path.parent.resolve(), "archive escapes its manifest directory")
    require(archive.stat().st_size == manifest["archive_bytes"], "original archive size changed")
    require(archive_sha256(archive) == manifest["archive_sha256"], "original archive checksum changed")

    with zipfile.ZipFile(archive) as source, tempfile.TemporaryDirectory(prefix="lustr-public-slice-") as directory:
        entries = source.infolist()
        require(len(entries) == manifest["archive_files"], "original file count changed")
        require(len(entries) <= 10000, "unbounded archive inventory")
        require(sum(entry.file_size for entry in entries) <= 64 * 1024 * 1024, "unbounded archive content")
        seen = set()
        for entry in entries:
            name = PurePosixPath(entry.filename)
            require(entry.filename not in seen, "duplicate original archive member")
            seen.add(entry.filename)
            require(not name.is_absolute() and not any(part in ("", ".", "..") for part in name.parts)
                    and not entry.filename.startswith("/") and "\\" not in entry.filename,
                    "original archive member escapes extraction")
            require(not stat.S_ISLNK(entry.external_attr >> 16), "original archive contains a symlink")
            require(not entry.is_dir(), "unexpected directory entry in original archive")
            target = Path(directory).joinpath(*name.parts)
            target.parent.mkdir(parents=True, exist_ok=True)
            with source.open(entry) as data, target.open("xb") as output:
                shutil.copyfileobj(data, output)

        report = slice_bench.replay(Path(directory))
        require(report["schema"] == manifest["campaign_schema"], "original report schema changed")
        require(report["status"] == manifest["status"], "original completion status changed")
        require(report["candidate_revision_and_tree"] ==
                [manifest["measured_checkout"], manifest["measured_tree"]], "measured checkout/tree changed")
        require(report["host"]["runner"]["GITHUB_RUN_ID"]["value"] == str(manifest["workflow_run"]),
                "original workflow run identity changed")
        require(report["host"]["runner"]["GITHUB_SHA"]["value"] == manifest["measured_checkout"],
                "runner checkout identity changed")
        require(report["raw_sha256"] == manifest["report_raw_sha256"], "original raw-event digest changed")
        require(report["pairs"] == manifest["pairs"], "original pair count changed")
        require(len(report["samples"]) == manifest["timings"], "original timing count changed")
        require(len(report["summaries"]) == manifest["comparisons"], "original comparison count changed")
    return manifest


def main():
    paths = sorted((ROOT / "performance/slices").glob("*/manifest.json"))
    completed = [path for path in paths if json.loads(path.read_text()).get("schema") == "lustr-public-slice-archive-v1"]
    require(completed, "no completed upstream-slice archives retained")
    for path in completed:
        result = replay_manifest(path)
        print(f"{path.parent.name}: verified original ZIP, {result['timings']} timings, "
              f"{result['comparisons']} comparisons; no compiler executed")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, zipfile.BadZipFile) as exc:
        print(f"upstream-slice archive verification failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
