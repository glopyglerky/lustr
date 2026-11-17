#!/usr/bin/env python3
"""Verify the committed original campaign ZIPs and replay their published data.

This checks preserved evidence, not the behavior or performance of a new build.
No recorded command is executed and no archive member is extracted.
"""
import importlib.util
from pathlib import Path
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("report_mixed", ROOT / "tools/report-mixed-performance.py")
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)


def checked_path(name):
    relative = r.Evidence.relative(name)
    path = (ROOT / str(relative)).resolve()
    r.require(path.is_relative_to(ROOT), "archive path escapes repository")
    return path


def main():
    manifest = r.json_load((ROOT / "performance/campaigns.json").read_bytes())
    r.require(manifest["schema"] == "lustr-committed-campaigns-v1", "unknown campaign manifest")
    total = dict(comparisons=0, timed_commands=0, command_outputs=0)
    for campaign in manifest["campaigns"]:
        evidence = r.Evidence(checked_path(campaign["archive"]))
        try:
            r.require(evidence.archive_sha256 == campaign["archive_sha256"], "original ZIP digest differs")
            result = r.verify(evidence)
            for key in ("checkout", "tree", "raw_sha256", "pairs", "seed"):
                r.require(result[key] == campaign[key], f"campaign identity differs: {key}")
            r.require(result["pristine_revision"] == manifest["pristine_revision"], "Lust pin differs")
            r.require(f"commit-hash: {manifest['rust_reference_revision']}\n" in result["rust_reference_version"],
                      "Rust reference differs")
            for key, rendered in (("csv", r.csv_samples(result)), ("table", r.markdown(result))):
                actual = checked_path(campaign[key]).read_bytes()
                r.require(r.sha256(actual) == campaign[key + "_sha256"], f"changed committed {key}")
                r.require(actual == rendered.encode("utf-8"), f"original archive does not replay committed {key}")
            for key in total:
                total[key] += result["verified"][key]
            print(f"{campaign['head']}: original ZIP and all outputs verified; published tables replayed")
        finally:
            evidence.close()
    print(f"Verified original evidence: {total}; no compiler or release admission")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, OverflowError, zipfile.BadZipFile) as exc:
        print(f"archived performance evidence rejected: {exc}", file=sys.stderr)
        raise SystemExit(1)
