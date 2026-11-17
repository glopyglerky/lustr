#!/usr/bin/env python3
"""Read-only reconciliation after authenticated decryption and archive verification.

Reuses the reviewed exporter at its ORIGINAL LOGICAL paths. Reads of private
evidence are relocated; omitted compiler/receipt hashes come from recorded
guards, never fabricated files. This does not requalify absent compiler bytes.
No private key, extraction, writes, compilation or campaign execution occurs.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import types

sys.dont_write_bytecode = True
PUBLIC = {"README.md", "campaign.json", "host.json", "replay.json", "samples.json"}
SCENARIOS = ("clean-dependency-build", "primed-root-build", "primed-root-check", "no-op", "compile-plus-test")


class CheckFailure(Exception):
    pass


def need(value, label):
    if not value:
        raise CheckFailure(label)


def sha(path):
    need(path.is_file() and not path.is_symlink(), "ordinary-file-required")
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def git(repo, *arguments):
    # A local refs/replace entry must not substitute another object while the
    # evidence still names the original commit identity.
    result = subprocess.run(["git", "--no-replace-objects", "-C", str(repo), *arguments], capture_output=True, check=False)
    need(result.returncode == 0, "recorded-source-unavailable")
    return result.stdout


def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


def source_projector(repo, identity, frozen):
    """Load the recorded exporter, never a substituted working-tree script."""
    revision = identity["revision"]
    need(type(revision) is str and re.fullmatch(r"[0-9a-f]{40}", revision), "invalid-recorded-source-revision")
    oid = lambda suffix: git(repo, "rev-parse", "--verify", revision + suffix).decode("ascii").strip()
    need(oid("^{commit}") == revision, "recorded-source-is-not-a-commit")
    coordinator = oid(":crates/lustr")
    need(coordinator == identity["coordinator_git_tree"], "recorded-coordinator-tree-differs")
    expected = {}
    for row in git(repo, "ls-tree", "-rz", revision + ":crates/lustr").split(b"\0"):
        if not row:
            continue
        metadata, name = row.split(b"\t", 1)
        mode, kind, checksum = metadata.split()
        need(kind == b"blob" and mode in (b"100644", b"100755"), "unsupported-frozen-source-entry")
        expected[name] = (mode, checksum.decode("ascii"))
    actual = {os.fsencode(path.relative_to(frozen)): path for path in frozen.rglob("*") if path.is_file()}
    need(set(actual) == set(expected), "frozen-source-git-inventory-differs")
    for name, path in actual.items():
        mode = b"100755" if path.stat().st_mode & 0o111 else b"100644"
        need((mode, git_blob(path.read_bytes())) == expected[name], "frozen-source-git-blob-differs")
    path = "tools/measure-coordinator-project.py"
    source = git(repo, "show", revision + ":" + path)
    blob = oid(":" + path)
    need(git_blob(source) == blob, "exporter-git-blob-differs")
    projector = types.ModuleType("recorded_coordinator_projection")
    projector.__file__ = str(repo / path)
    exec(compile(source, "<recorded-coordinator-exporter>", "exec"), projector.__dict__)
    need(tuple(projector.SCENARIOS) == SCENARIOS, "unsupported-recorded-exporter-contract")
    return projector, {
        "revision": revision, "git_tree": oid("^{tree}"), "coordinator_git_tree": coordinator,
        "exporter_git_blob": blob, "exporter_sha256": hashlib.sha256(source).hexdigest(),
        "exporter_loaded_from_recorded_revision": True,
        "campaign_exporter_runtime_sha256": None,
        "runner_source_git_tree": oid(":crates/lustr-bench"),
        "runner_recorded_binary_sha256": identity["runner_sha256"],
        "runner_binary_freshly_rehashed": False,
        "runner_binary_source_equivalence": "not re-established; original self-hash and CI source provenance retained",
    }


def relocated(logical, original, restored):
    logical = Path(logical)
    need(logical.is_relative_to(original), "attempted-original-host-read")
    relative = logical.relative_to(original)
    need(".." not in relative.parts, "attempted-relative-path-escape")
    return restored / relative


def reconcile(args):
    restored, public, repo = (path.resolve(strict=True) for path in (args.restored, args.public, args.repository))
    private = restored / "private"
    receipt = read(args.receipt)
    need(receipt["schema"] == "lustr-encrypted-evidence-receipt-v1", "retention-receipt-schema")
    manifest_path = restored / "MANIFEST.json"
    need(sha(manifest_path) == receipt["encrypted_manifest_sha256"], "encrypted-manifest-binding")
    manifest = read(manifest_path)
    need(manifest["schema"] == "lustr-private-evidence-manifest-v1", "private-manifest-schema")
    # Reject ancestor links before any manifest-directed file reads. The
    # recovered directory must contain only the verified ordinary-file tree.
    need(private.is_dir() and not private.is_symlink(), "private-directory-required")
    actual = set()
    for path in private.rglob("*"):
        need(not path.is_symlink(), "private-symlink")
        if path.is_file():
            actual.add(path.relative_to(restored).as_posix())
        else:
            need(path.is_dir(), "private-special-file")
    observed = {}
    for row in manifest["files"]:
        name = Path(row["name"])
        need(not name.is_absolute() and ".." not in name.parts and name.parts[0] == "private", "private-file-scope")
        path = restored / name
        need(row["name"] not in observed and sha(path) == row["sha256"] and path.stat().st_size == row["bytes"], "private-file-digest-or-size")
        need(stat.S_IMODE(path.stat().st_mode) == row["mode"], "private-file-mode")
        observed[row["name"]] = row
    need(actual == set(observed), "private-inventory-differs")
    need(len(observed) == receipt["original_file_count"] and sum(row["bytes"] for row in observed.values()) == receipt["original_file_bytes"], "private-inventory-totals")

    sums = {}
    for line in (public / "SHA256SUMS").read_text().splitlines():
        checksum, name = line.split("  ", 1)
        need(name in PUBLIC and name not in sums and re.fullmatch(r"[0-9a-f]{64}", checksum), "public-manifest-schema")
        need(sha(public / name) == checksum, "public-file-digest")
        sums[name] = checksum
    need(set(sums) == PUBLIC and {p.name for p in public.iterdir()} == PUBLIC | {"SHA256SUMS"}, "public-inventory-differs")
    identity, result, host = (read(private / name) for name in ("identity.json", "campaign-result.json", "host.json"))
    need(all(type(identity.get(key)) is str and re.fullmatch(r"[0-9a-f]{64}", identity[key])
             for key in ("runner_sha256", "replay_sha256", "gate_sha256", "cargo_sha256", "reference_rustc_sha256")),
         "invalid-recorded-tool-identity")
    samples, campaign = (read(public / name) for name in ("samples.json", "campaign.json"))
    need(result["status"] == campaign["status"] == "passed", "campaign-not-successful")
    need(all(campaign.get(key) == value for key, value in identity.items()), "campaign-identity-differs")
    need((public / "host.json").read_bytes() == (private / "host.json").read_bytes(), "public-host-differs")
    need(sums["samples.json"] == result["public_samples_sha256"], "private-result-public-samples-binding")

    frozen = private / "snapshot"
    projector, source_identity = source_projector(repo, identity, frozen)
    need([case["name"] for case in samples["cases"]] == list(SCENARIOS) == result["scenarios"], "scenario-inventory-differs")
    source_rows = {}
    for path in sorted(frozen.rglob("*")):
        if path.is_file():
            source_rows[os.fsencode(path.relative_to(frozen)).hex()] = f"{path.stat().st_mode:o}:{sha(path)}"
    tree = hashlib.sha256(json.dumps(source_rows, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    regenerated, original_roots, job_counts = [], set(), set()
    stream_count = 0
    for case in samples["cases"]:
        name = case["name"]
        directory = private / name
        report_path = directory / "report.json"
        report = read(report_path)
        need(report["runner_sha256"] == identity["runner_sha256"], "recorded-runner-identity-disagrees")
        original_spec = directory / "spec.original.json"
        need(sha(report_path) == case["full_private_report_sha256"] == result["original_report_sha256"][name], "original-report-binding")
        need(sha(original_spec) == case["original_spec_sha256"] == report["spec_sha256"], "original-spec-binding")
        need(original_spec.read_bytes() == (private / "specs" / (name + ".json")).read_bytes(), "prepared-spec-bytes-differ")
        need(read(original_spec) == report["spec"], "embedded-spec-differs")
        need(report["source_before"] == report["source_after"] == report["final_source_observation"], "recorded-source-guards-differ")
        source = report["source_before"]
        need(source["files_hex_paths"] == source_rows and source["tree_sha256"] == tree, "frozen-source-tree-differs")
        need(source["cargo_lock_sha256"] == sha(frozen / "Cargo.lock"), "frozen-lockfile-differs")
        run_spec = report["spec"]
        old_private = Path(run_spec["source"]).parent
        need(Path(run_spec["source"]).name == "snapshot" and old_private.name == "private" and old_private.is_absolute()
             and ".." not in old_private.parts, "original-private-scope")
        need(run_spec["revision"] == identity["revision"], "scenario-source-revision-differs")
        job_counts.add(int(run_spec["lanes"][0]["env"]["CARGO_BUILD_JOBS"]))
        original_roots.add(str(old_private))
        recorded = {os.fsdecode(bytes.fromhex(key)): value for key, value in source["external_inputs"].items()}
        for lane in run_spec["lanes"]:
            for tool in lane["tools"]:
                need(tool["path"] not in recorded or recorded[tool["path"]] == tool["sha256"], "tool-guard-identity-disagrees")
                recorded[tool["path"]] = tool["sha256"]
        need(all(re.fullmatch(r"[0-9a-f]{64}", value) for value in recorded.values()), "recorded-input-digest-shape")
        package = Path(run_spec["native_state"]["toolchain"])
        sdk = Path(run_spec["lanes"][0]["env"]["CARGO"]).parent.parent
        gate = Path(run_spec["lanes"][0]["gates"][0]["command"]["argv"][0])
        for path, checksum in ((sdk / "bin/cargo", identity["cargo_sha256"]),
                               (package / "bin/cargo", identity["cargo_sha256"]),
                               (sdk / "bin/rustc", identity["reference_rustc_sha256"]),
                               (gate, identity["gate_sha256"])):
            need(recorded.get(str(path)) == checksum, "recorded-tool-identity-disagrees")
        for component, checksum in identity["compiler_components_sha256"].items():
            need(recorded.get(str(package / "bin" / component)) == checksum, "recorded-component-identity-disagrees")
        for filename, checksum in identity["constructor_receipts_sha256"].items():
            need(recorded.get(str(package.parent / filename)) == checksum, "recorded-constructor-identity-disagrees")

        # The exporter keeps its original path comparisons. Only actual reads
        # move to the verified restored tree; foreign paths can never be read.
        def physical(logical):
            return relocated(logical, old_private, private)

        class OriginalPath(type(Path())):
            def open(self, mode="r", *arguments, **keywords):
                need(mode in ("r", "rb"), "archive-read-only")
                return physical(self).open(mode, *arguments, **keywords)

            def read_bytes(self):
                return physical(self).read_bytes()

            def stat(self, *arguments, **keywords):
                return physical(self).stat(*arguments, **keywords)

        def archived_digest(logical):
            logical = Path(logical)
            if logical.is_relative_to(old_private):
                return sha(physical(logical))
            need(str(logical) in recorded, "unrecorded-external-digest")
            return recorded[str(logical)]

        real_path, real_digest = projector.Path, projector.digest
        try:
            projector.Path, projector.digest = OriginalPath, archived_digest
            projected = projector.project_report(OriginalPath(old_private / name / "report.json"))
        finally:
            projector.Path, projector.digest = real_path, real_digest
        need(projected == case, "public-case-does-not-reproduce")
        regenerated.append(projected)
        stream_count += len(report["records"]) * 2
    need(len(original_roots) == 1, "mixed-original-campaign-roots")
    need(len(job_counts) == 1, "mixed-campaign-job-budgets")
    need(set(result["original_report_sha256"]) == set(projector.SCENARIOS), "original-report-inventory-differs")
    rebuilt = {"schema": 1, "pairs": projector.PAIRS, "warmup_pairs": projector.WARMUPS,
               "seed": projector.SEED, "bootstrap_resamples": projector.RESAMPLES,
               "runner_sha256": identity["runner_sha256"], "cases": regenerated}
    need(encoded(rebuilt) == (public / "samples.json").read_bytes(), "public-samples-bytes-do-not-reproduce")
    need(projector.public_readme(rebuilt, identity, host).encode() == (public / "README.md").read_bytes(), "public-readme-bytes-do-not-reproduce")
    expected_campaign = {**identity, "schema": "lustr-coordinator-campaign-v1", "scenarios": list(SCENARIOS),
                         "status": "passed", "project_profiles": "unchanged dev and test defaults", "jobs": job_counts.pop(),
                         "pairs": projector.PAIRS, "warmup_pairs": projector.WARMUPS,
                         "measurement": "wall time only; detached daemon CPU, memory and energy are not measured",
                         "privacy": "Allowlist projection. Original report/spec/output SHA256 values identify private originals, not these rewritten bytes."}
    need(campaign == expected_campaign, "public-campaign-does-not-reproduce")
    replay_path = args.replay.resolve(strict=True)
    replay_hash = sha(replay_path)
    replay = subprocess.run([str(replay_path), str(public / "samples.json")], capture_output=True, check=False, timeout=60)
    need(replay.returncode == 0, "existing-statistical-replay-failed")
    replayed = json.loads(replay.stdout)
    need(replayed["verified_comparisons"] == len(projector.SCENARIOS), "statistical-replay-incomplete")
    need(replayed == read(public / "replay.json"), "public-replay-receipt-does-not-reproduce")
    source_identity.update(replay_expected_source_git_tree=source_identity["runner_source_git_tree"],
                           replay_recorded_binary_sha256=identity["replay_sha256"], replay_local_binary_sha256=replay_hash,
                           replay_matches_recorded_binary=replay_hash == identity["replay_sha256"],
                           replay_binary_source_equivalence="not re-established by this check")
    return {"status": "passed", "all_archived_originals_freshly_hashed": True,
            "original_file_count": len(observed), "scenario_count": len(regenerated),
            "original_report_count": len(regenerated), "raw_stream_count": stream_count,
            "public_cases_semantically_identical": True, "samples_and_readme_byte_identical": True,
            "verified_statistical_comparisons": replayed["verified_comparisons"],
            "replay_json_semantically_identical": True, "campaign_json_semantically_identical": True,
            "source_identity": source_identity, "compiler_binaries_freshly_rehashed": False,
            "omitted_external_input_scope": "Recorded identity consistency only; absent compiler, runtime and constructor bytes were not requalified."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("restored", "receipt", "public", "repository", "replay"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(reconcile(args), sort_keys=True, indent=2))
        return 0
    except Exception as error:
        category = str(error) if isinstance(error, CheckFailure) else "reconciliation-operation-failed"
        print(json.dumps({"status": "failed", "check": category}, sort_keys=True))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
