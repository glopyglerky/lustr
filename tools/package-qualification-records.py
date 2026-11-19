#!/usr/bin/env python3
"""Archive qualification records from the unchanged complete-upload roots.

This is evidence packaging, not compiler qualification. The complete artifact
remains authoritative for omitted compiled files. No symlink is dereferenced.
"""

import argparse
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tarfile


UPLOAD = "      - name: Retain source-bound results and every failure"
PREFIX = "            ${{ runner.temp }}/"
LIMIT = 480 * 1024 * 1024
COMPILED_MINIMUM = 1024 * 1024


def original_patterns(workflow):
    """Read this workflow's literal upload block; reject other YAML shapes."""
    text = workflow.read_text()
    if text.count(UPLOAD + "\n") != 1:
        raise ValueError("expected one original complete-artifact upload")
    block = text.split(UPLOAD + "\n", 1)[1]
    block = re.split(r"(?m)^      - |^  [^ ]", block, maxsplit=1)[0]
    if ("        uses: actions/upload-artifact@v4\n" not in block
            or "          name: exclusive-entry-and-shared-services\n" not in block
            or block.count("          path: |\n") != 1):
        raise ValueError("complete-upload identity or literal path block changed")
    patterns = []
    for line in block.split("          path: |\n", 1)[1].splitlines():
        if not line.strip():
            continue
        if not line.startswith(PREFIX):
            raise ValueError("complete-upload path is not a literal runner.temp child")
        pattern = line[len(PREFIX):]
        parts = pattern.split("/")
        if any(p in ("", ".", "..") for p in parts) or "\\" in pattern:
            raise ValueError("unsafe complete-upload path")
        if any(any(c in p for c in "*?[") for p in parts[:-1]):
            raise ValueError("only final-component upload globs are supported")
        patterns.append(pattern)
    if not patterns or len(patterns) != len(set(patterns)):
        raise ValueError("empty or duplicate complete-upload roots")
    return patterns


def record_name(name):
    parts = Path(name).parts
    return ("streams" in parts or Path(name).suffix.lower() in {
        ".stdout", ".stderr", ".out", ".err", ".log", ".trace", ".json",
        ".jsonl", ".ndjson", ".txt", ".sha256", ".status", ".yaml", ".yml",
        ".toml", ".lock", ".csv", ".md", ".receipt",
    } or re.search(r"(?:^|[._-])(stdout|stderr|trace|log|report|receipt)(?:$|[._-])",
                   parts[-1], re.IGNORECASE) is not None)


def compiled_format(name, header, size):
    # Record names always win, even if a program printed an entire executable.
    if record_name(name) or size < COMPILED_MINIMUM:
        return None
    if (header.startswith(b"\x7fELF") and len(header) >= 64
            and header[4] in (1, 2) and header[5] in (1, 2) and header[6] == 1
            and int.from_bytes(header[16:18], "little" if header[5] == 1 else "big") in (1, 2, 3)):
        return "ELF"
    if header.startswith(b"!<arch>\n") or header.startswith(b"!<thin>\n"):
        return "ar"
    if header.startswith(b"rust\0") and Path(name).suffix == ".rmeta":
        return "Rust metadata"
    return None


def stamp(info):
    return (info.st_dev, info.st_ino, info.st_mode, info.st_size,
            info.st_mtime_ns, info.st_ctime_ns)


class HashedReader:
    def __init__(self, source):
        self.source, self.digest, self.count = source, hashlib.sha256(), 0

    def read(self, size=-1):
        data = self.source.read(size)
        self.digest.update(data)
        self.count += len(data)
        return data


def package(workflow, base, output, max_bytes=LIMIT):
    patterns = original_patterns(workflow)
    base = base.resolve(strict=True)
    destination = output.resolve()
    for pattern in patterns:
        scope = base.joinpath(*pattern.split("/")[:-1]) if any(c in pattern for c in "*?[") else base / pattern
        if destination == scope or destination.is_relative_to(scope):
            raise ValueError("supplemental output overlaps an original artifact root")
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    report = {
        "schema": "lustr-qualification-records-v1", "capture_complete": False,
        "compiler_qualification": False, "workflow_sha256": hashlib.sha256(workflow.read_bytes()).hexdigest(),
        "helper_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "source": {k: os.environ.get(k) for k in ("GITHUB_SHA", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT")},
        "original_upload": "exclusive-entry-and-shared-services", "patterns": patterns,
        "roots": [], "entries": [], "errors": [],
        "symlink_policy": "Preserve link targets without dereferencing. A record-named symlink fails capture; its referent bytes are not asserted.",
        "omission_policy": "Only ELF object/executable/shared-library, ar, or Rust metadata containers of at least 1 MiB with non-record names omit regular-file bytes. Every regular file is hashed. Other formats remain included.",
        "snapshot_scope": "Each entry is checked before and after capture. This is not a coordinated cross-file snapshot or a verified download of the separate complete artifact.",
    }
    seen = set()
    archive_path = output / "records.tar.gz"
    directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW

    def capture(archive, parent, name, relative):
        if relative in seen:
            return
        seen.add(relative)
        entry = {"path": relative}
        report["entries"].append(entry)
        try:
            before = os.stat(name, dir_fd=parent, follow_symlinks=False)
            entry.update(mode=stat.S_IMODE(before.st_mode), size=before.st_size)
            tar = tarfile.TarInfo(relative)
            tar.mode = stat.S_IMODE(before.st_mode)
            tar.mtime = before.st_mtime_ns // 1_000_000_000
            if stat.S_ISLNK(before.st_mode):
                target = os.readlink(name, dir_fd=parent)
                entry.update(kind="symlink", included=True, target=target,
                             sha256=hashlib.sha256(os.fsencode(target)).hexdigest(),
                             hash_domain="link-target-bytes")
                tar.type, tar.linkname = tarfile.SYMTYPE, target
                archive.addfile(tar)
                if record_name(relative):
                    raise ValueError("record is a symlink; referent was not read")
            elif stat.S_ISDIR(before.st_mode):
                entry.update(kind="directory", included=True)
                tar.type = tarfile.DIRTYPE
                archive.addfile(tar)
                fd = os.open(name, directory_flags, dir_fd=parent)
                try:
                    if stamp(os.fstat(fd)) != stamp(before):
                        raise ValueError("directory changed before capture")
                    names = sorted(os.listdir(fd), key=os.fsencode)
                    for child in names:
                        capture(archive, fd, child, relative + "/" + child)
                    if names != sorted(os.listdir(fd), key=os.fsencode):
                        raise ValueError("directory membership changed during capture")
                finally:
                    os.close(fd)
            elif stat.S_ISREG(before.st_mode):
                entry["kind"] = "regular"
                fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
                with os.fdopen(fd, "rb") as source:
                    if stamp(os.fstat(source.fileno())) != stamp(before):
                        raise ValueError("file changed before capture")
                    kind = compiled_format(relative, source.read(64), before.st_size)
                    source.seek(0)
                    reader = HashedReader(source)
                    entry["included"] = kind is None
                    if kind:
                        entry.update(omitted="compiled-container", format=kind)
                        while reader.read(1024 * 1024):
                            pass
                    else:
                        tar.size = before.st_size
                        archive.addfile(tar, reader)
                    entry.update(sha256=reader.digest.hexdigest(), hash_domain="file-bytes")
                    if reader.count != before.st_size or stamp(os.fstat(source.fileno())) != stamp(before):
                        raise ValueError("file changed during capture")
            else:
                entry.update(kind="special", included=False, omitted="non-regular-object-not-read")
                if record_name(relative):
                    raise ValueError("record is not a regular file")
            if stamp(os.stat(name, dir_fd=parent, follow_symlinks=False)) != stamp(before):
                raise ValueError("entry changed during capture")
        except Exception as error:
            entry["error"] = f"{type(error).__name__}: {error}"
            report["errors"].append({"path": relative, "error": entry["error"]})

    try:
        with tarfile.open(archive_path, "x:gz", dereference=False) as archive:
            for pattern in patterns:
                root = {"pattern": pattern, "matches": []}
                report["roots"].append(root)
                fd = os.open(base, directory_flags)
                try:
                    parts = pattern.split("/")
                    for part in parts[:-1]:
                        child = os.open(part, directory_flags, dir_fd=fd)
                        os.close(fd)
                        fd = child
                    names = sorted((n for n in os.listdir(fd) if fnmatch.fnmatchcase(n, parts[-1])), key=os.fsencode)
                    for name in names:
                        relative = "/".join([*parts[:-1], name])
                        root["matches"].append(relative)
                        capture(archive, fd, name, relative)
                    if names != sorted((n for n in os.listdir(fd) if fnmatch.fnmatchcase(n, parts[-1])), key=os.fsencode):
                        raise OSError("upload root matches changed during capture")
                    root["state"] = "present" if names else "absent"
                except FileNotFoundError:
                    root["state"] = "absent"
                except OSError as error:
                    root["state"] = "unreadable-or-symlink-parent"
                    report["errors"].append({"path": pattern, "error": str(error)})
                finally:
                    os.close(fd)
        if not any(e.get("kind") == "regular" for e in report["entries"]):
            report["errors"].append({"error": "no regular files in original artifact roots"})
        archive_hash = hashlib.sha256()
        with archive_path.open("rb") as source:
            while data := source.read(1024 * 1024):
                archive_hash.update(data)
        report["archive"] = {"path": archive_path.name, "bytes": archive_path.stat().st_size,
                             "sha256": archive_hash.hexdigest(), "maximum_bytes": max_bytes}
        if archive_path.stat().st_size > max_bytes:
            archive_path.rename(output / "records-over-budget.tar.gz")
            report.setdefault("archive", {})["path"] = "records-over-budget.tar.gz"
            report["errors"].append({"error": "supplemental archive exceeds download budget; full original upload remains required"})
        report["capture_complete"] = not report["errors"]
    except Exception as error:
        report["errors"].append({"error": f"{type(error).__name__}: {error}"})
    finally:
        report["entries"].sort(key=lambda e: os.fsencode(e["path"]))
        regular = [e for e in report["entries"] if e.get("kind") == "regular"]
        report["regular_files"] = len(regular)
        report["included_regular_bytes"] = sum(e["size"] for e in regular if e.get("included"))
        report["omitted_regular_bytes"] = sum(e["size"] for e in regular if e.get("included") is False)
        encoded = json.dumps(report, indent=2) + "\n"
        if archive_path.exists() and archive_path.stat().st_size + len(encoded.encode()) > max_bytes:
            archive_path.rename(output / "records-over-budget.tar.gz")
            report.setdefault("archive", {})["path"] = "records-over-budget.tar.gz"
            report["errors"].append({"error": "supplemental archive plus inventory exceeds download budget"})
            report["capture_complete"] = False
        (output / "inventory.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workflow", type=Path, default=Path(".github/workflows/compiler-set.yml"))
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = package(args.workflow, args.root, args.output)
    print(json.dumps({"capture_complete": result["capture_complete"], "entries": len(result["entries"]),
                      "archive": result.get("archive"), "errors": result["errors"]}))
    return 0 if result["capture_complete"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
