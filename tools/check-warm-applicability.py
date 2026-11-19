#!/usr/bin/env python3
"""Reject known-inapplicable historical comparisons before constructing packages.

This is a source/evidence check, never a compiler or performance qualification.
There is no override. The original runners retain all of their admission gates.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import tomllib

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("warm_native", HERE / "run-warm-native-comparison.py")
N = importlib.util.module_from_spec(spec)
spec.loader.exec_module(N)
B = N.B
require = N.require
SCHEMA = "lustr-historical-warm-applicability-v1"
FAILURE_RECORD = "docs/compiler-set/evidence/hosted-warm-failures-20261118/summary.json"
FAILURE_SHA256 = "dd1fb16bb093680e970fc6b5c315acc322b5e1ad6687d4195586a97bf0ef7c6a"
BUILD_TOOLCHAIN = "nightly-2026-11-09"
BUILD_SOURCE = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494"
COMPLETE_BASELINE = "3c378a11d60149a3ec99fa49ea1b336e9406fca8"
COMPLETE_TREE = "0256606d5793bfb890130f05fae013d0c5c5f9e1"
ORACLE_RELEASE = "1.99.0"
ORACLE_SOURCE = "b940084d7eb6a299eb4bfeb8e34901bc051e7ac4"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def shared_sources(repository, baseline):
    return {path: {"baseline": B.git(baseline, "rev-parse", "HEAD:" + path),
                   "candidate": B.git(repository, "rev-parse", "HEAD:" + path)}
            for path in B.SHARED_SOURCE_PATHS}


def entry_preflight(args, report):
    require(report["baseline"]["revision"] == B.BASELINE, "historical entry baseline revision differs")
    shared = shared_sources(args.repository, args.baseline_source)
    report["shared_source_objects"] = shared
    changed = [path for path, objects in shared.items() if objects["baseline"] != objects["candidate"]]
    if changed:
        report.update(reason_code="shared-runtime-source-differs", changed_shared_sources=changed,
                      reason="The historical entry-only comparison requires identical shared runtime source. "
                             "Its existing build guard would reject: " + ", ".join(changed))
        return 1
    report.update(status="applicable", reason_code="shared-runtime-source-identical",
                  reason="The source-only applicability condition holds. Every original construction, "
                         "correctness, identity and measurement gate is still required.")
    return 0


def complete_preflight(args, report):
    require((N.BASELINE, N.BASELINE_TREE, N.NATIVE_ORACLE_RELEASE, N.NATIVE_ORACLE_COMMIT) ==
            (COMPLETE_BASELINE, COMPLETE_TREE, ORACLE_RELEASE, ORACLE_SOURCE),
            "complete-package campaign pins changed; its applicability requires explicit review")
    require(report["baseline"] == {"revision": COMPLETE_BASELINE, "tree": COMPLETE_TREE},
            "complete-package historical baseline source differs")
    require(os.environ.get("RUSTUP_TOOLCHAIN") == BUILD_TOOLCHAIN,
            "declared build toolchain differs or is missing; no historical result may be transferred")
    pins = {}
    for lane, source in (("baseline", args.baseline_source), ("candidate", args.repository)):
        path = source / "upstream/bluefe-core.pin"
        data = path.read_bytes()
        pin = tomllib.loads(data.decode())
        require(pin["source"] == BUILD_SOURCE, "declared " + lane + " build SDK source differs")
        pins[lane] = {"source": pin["source"], "pin_sha256": digest(data)}
    report["declared_configuration"] = {
        "build_toolchain": BUILD_TOOLCHAIN, "build_sdk": pins,
        "native_oracle": {"release": ORACLE_RELEASE, "source": ORACLE_SOURCE},
        "scope": "Declared source configuration only; no SDK was installed or executed by this preflight."}
    data = (args.repository / FAILURE_RECORD).read_bytes()
    require(digest(data) == FAILURE_SHA256, "source-bound historical failure record is missing or changed")
    failure = json.loads(data)["native_complete"]
    require(failure["packages"]["baseline"]["source"] == report["baseline"]
            and failure["conclusion"] == "failure"
            and failure["differential"]["failed_case"] == "unicode_props"
            and failure["differential"]["exit"] == 1
            and failure["differential"]["rustc_unicode_version"] == [17, 0, 0]
            and failure["differential"]["lust_unicode_version"] == [18, 0, 0]
            and failure["admitted_timing_samples"] == 0,
            "historical record does not establish this comparison's Unicode failure")
    require("commit-hash: " + BUILD_SOURCE + "\n" in failure["build_sdk"]["rustc_version"]
            and "commit-hash: " + ORACLE_SOURCE + "\n" in failure["native_oracle"]["rustc_version"],
            "historical SDK or native oracle source differs")
    proofs = failure["source_proof"] + [failure["runtime_unicode_table"]]
    for proof in proofs:
        path = proof["path"]
        require(proof["revision"] == COMPLETE_BASELINE
                and B.git(args.baseline_source, "rev-parse", "HEAD:" + path) == proof["git_blob"],
                "historical Unicode source object differs: " + path)
        actual = (args.baseline_source / path).read_bytes()
        require(digest(actual) == proof["sha256"] and len(actual) == proof["bytes"],
                "historical Unicode source bytes differ: " + path)
    report.update(reason_code="frozen-baseline-unicode-failure",
                  reason="This exact frozen baseline and declared SDK/oracle combination already failed "
                         "the unchanged Unicode differential gate. The package remains unqualified; "
                         "rebuilding it cannot qualify this historical campaign.",
                  historical_failure={"file": FAILURE_RECORD, "sha256": FAILURE_SHA256,
                                      "run_url": failure["run_url"], "job_id": failure["job_id"],
                                      "artifact": failure["artifact"],
                                      "baseline": failure["packages"]["baseline"]["source"],
                                      "differential": failure["differential"], "source_proof": proofs})
    return 1


def check(args):
    require(not args.output.exists(), "applicability output must be new; prior evidence is retained")
    args.output.mkdir(mode=0o700, parents=True)
    report = {"schema": SCHEMA, "campaign": args.campaign, "status": "blocked",
              "qualification": "unqualified", "admitted_timing_samples": 0,
              "scope": "Historical comparison applicability only. No compiler build, runtime test or timing was executed.",
              "reason_code": "unverified-applicability"}
    code = 1
    try:
        report["candidate"] = B.source_identity(args.repository)
        report["baseline"] = B.source_identity(args.baseline_source)
        report["preflight_sha256"] = digest(Path(__file__).read_bytes())
        code = (entry_preflight if args.campaign == "entry" else complete_preflight)(args, report)
    except Exception as error:
        report["reason"] = str(error)
        report["error_type"] = type(error).__name__
    report["exit"] = code
    encoded = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode()
    (args.output / "applicability.json").write_bytes(encoded)
    text = ("# Historical warm comparison applicability\n\n"
            f"Status: **{report['status']}**. Qualification: **unqualified**. Admitted timing samples: **0**.\n\n"
            + report["reason"] + "\n\n" + report["scope"] + "\n\n"
            "The frozen baseline, oracle, original admission gates and earlier failed artifacts remain unchanged. "
            "This record is neither package qualification nor a performance assessment.\n")
    (args.output / "README.md").write_text(text)
    (args.output / "SHA256SUMS").write_text(digest(encoded) + "  applicability.json\n"
                                           + digest(text.encode()) + "  README.md\n")
    print(text)
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a") as stream:
            stream.write(text + "\n")
    return code


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("campaign", choices=("entry", "complete"))
    for name in ("repository", "baseline-source", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    return check(parser.parse_args())


if __name__ == "__main__":
    sys.exit(main())
