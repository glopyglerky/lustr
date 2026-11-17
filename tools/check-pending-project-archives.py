#!/usr/bin/env python3
"""Verify retained public and sealed project ZIPs; never decrypt or execute them.

This intentionally does not mark the private originals as recovered or perform
the Rust statistical replay used by completed authenticated project campaigns.
"""

import hashlib
import json
from pathlib import Path
import sys
import zipfile


ROOT = Path(__file__).resolve().parents[1]
PUBLIC = {"README.md", "campaign.json", "replay.json", "samples.json", "host.json", "SHA256SUMS"}
SEALED = {"private-evidence.tar.gz.cms", "receipt.json", "status.json"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(manifest_file):
    meta = json.loads(manifest_file.read_text())
    require(meta["schema"] == "lustr-unrecovered-project-archive-v1", "unexpected manifest schema")
    require(meta["repository"] == "glopyglerky/lustr", "unexpected repository")
    require(meta["authenticated_recovery"] == "not-performed", "unverified private recovery claim")
    require(all(len(meta[k]) == 40 for k in ("head_sha", "measured_checkout", "measured_tree")),
            "invalid source identity")
    archives = {}
    for key in ("public", "encrypted"):
        entry = meta[key]
        require(entry["file"] == {"public": "public-original.zip", "encrypted": "encrypted-original.zip"}[key],
                "unexpected archive filename")
        path = manifest_file.parent / entry["file"]
        data = path.read_bytes()
        require(len(data) == entry["bytes"] and digest(data) == entry["sha256"],
                f"{key} original ZIP differs from its verified GitHub artifact")
        archives[key] = zipfile.ZipFile(path)
    with archives["public"] as public, archives["encrypted"] as sealed:
        require(set(public.namelist()) == PUBLIC and len(public.namelist()) == len(PUBLIC),
                "public ZIP inventory differs")
        require(set(sealed.namelist()) == SEALED and len(sealed.namelist()) == len(SEALED),
                "encrypted ZIP inventory differs")
        hashes = {}
        for line in public.read("SHA256SUMS").decode().splitlines():
            sha, name = line.split("  ", 1)
            require(name in PUBLIC - {"SHA256SUMS"} and name not in hashes,
                    "public checksum list differs")
            hashes[name] = sha
            require(digest(public.read(name)) == sha, "public file checksum differs")
        require(set(hashes) == PUBLIC - {"SHA256SUMS"}, "incomplete public checksum list")
        campaign = json.loads(public.read("campaign.json"))
        require(campaign["status"] == "passed" and campaign["revision"] == meta["measured_checkout"],
                "project measurement source/status differs")
        samples = json.loads(public.read("samples.json"))
        timed = sum(row["phase"] == "timed" for case in samples["cases"] for row in case["records"])
        replay = json.loads(public.read("replay.json"))
        require(timed == meta["timings"] and replay["verified_comparisons"] == meta["comparisons"],
                "project timing or comparison count differs")
        receipt = json.loads(sealed.read("receipt.json"))
        status = json.loads(sealed.read("status.json"))
        cipher = sealed.read("private-evidence.tar.gz.cms")
        require(receipt["schema"] == "lustr-encrypted-evidence-receipt-v1" and
                receipt["cms_content_type"] == "id-smime-ct-authEnvelopedData" and
                receipt["content_cipher"] == "AES-256-GCM", "unexpected encrypted receipt")
        require(len(cipher) == receipt["ciphertext_bytes"] and digest(cipher) == receipt["ciphertext_sha256"],
                "encrypted ciphertext receipt differs")
        require(status["campaign_status"] == "passed" and status["retention_status"] == "retained",
                "encrypted artifact is not a completed retention")
    return timed, replay["verified_comparisons"]


def main():
    paths = sorted((ROOT / "performance/projects-pending").glob("*/manifest.json"))
    require(paths, "no pending project archive manifests")
    for path in paths:
        timings, comparisons = verify(path)
        print(f"{path.parent.name}: original public and encrypted ZIPs verified; "
              f"{timings} timings, {comparisons} recorded comparisons; no private recovery")


if __name__ == "__main__":
    try:
        main()
    except (OSError, KeyError, ValueError, TypeError, zipfile.BadZipFile) as error:
        print(f"pending project archive verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
