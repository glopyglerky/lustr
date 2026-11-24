#!/usr/bin/env python3
"""Retain one failed, owned native crate fixture without changing its result."""

import argparse
import json
import os
from pathlib import Path
import stat
import tarfile
import tempfile


def owned_directory(path):
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid():
        raise ValueError("evidence and case directories must be real directories owned by this user")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--name", required=True)
    parser.add_argument("--reason", required=True)
    parser.add_argument("--daemon-log", type=Path)
    args = parser.parse_args()
    if not args.name or args.name in (".", "..") or any(c in args.name for c in "/\\\0"):
        raise ValueError("invalid fixture name")
    owned_directory(args.case)
    if args.output.resolve().is_relative_to(args.case.resolve()):
        raise ValueError("failure output cannot be inside the case being archived")
    args.output.mkdir(mode=0o700, parents=True, exist_ok=True)
    owned_directory(args.output)
    # Never replace an earlier capture or recursively remove a caller's path.
    destination = Path(tempfile.mkdtemp(prefix=args.name + "-", dir=args.output))
    record = {
        "schema": "lustr-crate-failure-v1",
        "fixture": args.name,
        "failure": args.reason,
        "capture_complete": False,
        "skipped_special_files": [],
        "limits": "A best-effort snapshot of the failed fixture. A live daemon may still append logs; this is not a coordinated final snapshot.",
    }

    def ordinary_files(info):
        if info.isfile() or info.isdir() or info.issym() or info.islnk():
            return info
        record["skipped_special_files"].append(info.name)
        return None

    try:
        # Keep dot directories such as .lust-cache inside the archive. The CI
        # uploader can retain the non-hidden archive without exposing unrelated
        # runner dotfiles. Do not follow symlinks or read sockets/FIFOs/devices.
        with tarfile.open(destination / "case.tar.gz", mode="x:gz", dereference=False) as archive:
            archive.add(args.case, arcname="case", filter=ordinary_files)
            if args.daemon_log:
                try:
                    info = args.daemon_log.lstat()
                except FileNotFoundError:
                    record["external_daemon_log"] = "absent"
                else:
                    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid():
                        raise ValueError("external daemon log must be a regular file owned by this user")
                    archive.add(args.daemon_log, arcname="external-daemon.log", recursive=False, filter=ordinary_files)
                    record["external_daemon_log"] = "captured"
        record["capture_complete"] = True
    except Exception as error:
        record["capture_error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        (destination / "failure.json").write_text(json.dumps(record, indent=2) + "\n")
        print(f"native crate failure evidence: {destination}")


if __name__ == "__main__":
    main()
