#!/usr/bin/env python3
"""Check committed public project evidence without decrypting private originals.

Reuses the existing Rust replay. No archived command is executed or file
extracted; successful private recovery remains a separately retained receipt.
"""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = {"README.md", "campaign.json", "samples.json", "replay.json", "host.json", "SHA256SUMS"}
ENCRYPTED = {"private-evidence.tar.gz.cms", "receipt.json", "status.json"}


def need(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def timing_csv(samples):
    output = io.StringIO(newline="")
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["scenario", "lane", "pair", "seconds", "order_in_pair", "stdout_sha256", "stderr_sha256"])
    count = 0
    for case in samples["cases"]:
        orders = {item["pair"]: item["lane_indices"] for item in case["order"] if not item["warmup"]}
        for row in case["records"]:
            if row["phase"] != "timed":
                continue
            writer.writerow([case["name"], row["lane"], row["pair"], repr(row["wall_seconds"]),
                             orders[row["pair"]].index(case["lanes"].index(row["lane"])),
                             row["stdout_sha256"], row["stderr_sha256"]])
            count += 1
    return output.getvalue().encode(), count


def check(directory, replay):
    manifest = read(directory / "manifest.json")
    need(manifest["schema"] == "lustr-committed-project-evidence-v1", "unknown project manifest")
    files = manifest["files"]
    actual = {path.relative_to(directory).as_posix() for path in directory.rglob("*") if path.is_file()}
    need(actual == set(files) | {"manifest.json"}, "project inventory differs")
    for name, identity in files.items():
        path = directory / name
        need(not Path(name).is_absolute() and ".." not in Path(name).parts and not path.is_symlink(), "unsafe retained path")
        data = path.read_bytes()
        need(len(data) == identity["bytes"] and sha(data) == identity["sha256"], "retained file identity differs")
    public = directory / "public"
    with zipfile.ZipFile(directory / "artifacts/public-original.zip") as archive:
        need(len(archive.namelist()) == len(PUBLIC) and set(archive.namelist()) == PUBLIC, "public archive inventory differs")
        need(all(archive.read(name) == (public / name).read_bytes() for name in PUBLIC), "public original bytes differ")
    sums = {}
    for line in (public / "SHA256SUMS").read_text().splitlines():
        checksum, name = line.split("  ", 1)
        need(name in PUBLIC - {"SHA256SUMS"} and name not in sums, "public checksum inventory differs")
        sums[name] = checksum
        need(sha((public / name).read_bytes()) == checksum, "public checksum differs")
    need(set(sums) == PUBLIC - {"SHA256SUMS"}, "public checksums incomplete")
    with zipfile.ZipFile(directory / "artifacts/encrypted-original.zip") as archive:
        need(len(archive.namelist()) == len(ENCRYPTED) and set(archive.namelist()) == ENCRYPTED, "encrypted archive inventory differs")
        for name in ("receipt.json", "status.json"):
            need(archive.read(name) == (directory / "retention" / name).read_bytes(), "retention original bytes differ")
        receipt = json.loads(archive.read("receipt.json"))
        cipher = archive.read("private-evidence.tar.gz.cms")
        need(len(cipher) == receipt["ciphertext_bytes"] and sha(cipher) == receipt["ciphertext_sha256"], "ciphertext identity differs")
    metadata = read(directory / "github-artifacts.json")["artifacts"]
    for key, name in (("public_artifact", "public-original.zip"), ("encrypted_artifact", "encrypted-original.zip")):
        matches = [item for item in metadata if item["id"] == manifest[key]]
        need(len(matches) == 1, "missing original artifact metadata")
        data = (directory / "artifacts" / name).read_bytes()
        item = matches[0]
        need(item["size_in_bytes"] == len(data) and item["digest"] == "sha256:" + sha(data), "GitHub artifact binding differs")
        need(item["workflow_run"]["head_sha"] == manifest["head"] and item["workflow_run"]["id"] == manifest["run"], "GitHub source binding differs")
    campaign = read(public / "campaign.json")
    need(campaign["status"] == "passed" and campaign["revision"] == manifest["checkout"], "public source identity differs")
    samples = read(public / "samples.json")
    rendered, count = timing_csv(samples)
    need(count == manifest["timed_commands"] and rendered == (directory / "timings.csv").read_bytes(), "timing CSV does not reproduce")
    result = subprocess.run([str(replay), str(public / "samples.json")], capture_output=True, timeout=60, check=False)
    need(result.returncode == 0, "existing statistical replay failed")
    replayed = json.loads(result.stdout)
    need(replayed == read(public / "replay.json") and replayed["verified_comparisons"] == manifest["comparisons"], "public replay receipt does not reproduce")
    recovery = read(directory / "recovery/result.json")
    need(recovery["status"] == "passed" and recovery["source_identity"]["revision"] == manifest["checkout"]
         and recovery["source_identity"]["git_tree"] == manifest["tree"], "retained recovery source binding differs")
    return count, replayed["verified_comparisons"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--replay", type=Path, required=True)
    args = parser.parse_args()
    replay = args.replay.resolve(strict=True)
    manifests = sorted((ROOT / "performance/projects").glob("*/manifest.json"))
    need(bool(manifests), "no retained project evidence")
    total = [0, 0]
    for manifest in manifests:
        count, comparisons = check(manifest.parent, replay)
        total[0] += count
        total[1] += comparisons
        print(f"{manifest.parent.name}: original public/encrypted ZIPs verified; {count} timings and {comparisons} comparisons replayed")
    print(f"Public evidence verified: {total[0]} timings, {total[1]} comparisons. Private originals were not decrypted or freshly requalified by this check.")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, subprocess.TimeoutExpired, zipfile.BadZipFile) as error:
        raise SystemExit(f"retained project evidence rejected: {error}")
