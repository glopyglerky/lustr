#!/usr/bin/env python3
"""Bind the actual tracked inputs of the bounded metadata qualification job."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess


ROOT = Path(__file__).resolve().parents[1]
INPUTS = [
    "compiler/native", "crates/lustr-native-core", "crates/lustr-metadata-encoder",
    "crates/lustr-metadata-sdk", "crates/lustr-native-metadata-qualification",
    "tools/test-native-consumer-metadata.py", "tools/test-native-metadata-trace.py",
    "tools/native-metadata-source-manifest.py", "tools/build-tinycc-pic.rs",
    "tools/check-exclusive-coverage.py", "tools/build-metadata-sdk-receipt.py",
    "upstream/bluefe-core.pin", ".github/workflows/compiler-set.yml",
]


def git(*arguments):
    return subprocess.check_output(["git", *arguments], cwd=ROOT)


def capture():
    if git("rev-parse", "--show-object-format").strip() != b"sha1":
        raise ValueError("unrepresented Git object format")
    git("diff", "--cached", "--quiet", "HEAD", "--", *INPUTS)
    files, gitlinks = [], []
    for record in git("ls-files", "--stage", "-z", "--", *INPUTS).split(b"\0"):
        if not record:
            continue
        header, name = record.split(b"\t", 1)
        mode, blob, stage = header.decode("ascii").split()
        if stage != "0":
            raise ValueError("unresolved tracked input")
        relative = os.fsdecode(name)
        path = ROOT / relative
        item = {"path": relative, "mode": mode, "git_object": blob}
        if mode == "160000":
            # The test-fixture gitlink is not initialized or consumed by this
            # bounded crate. Record its identity without claiming file bytes.
            gitlinks.append(item)
            continue
        before = path.lstat()
        if mode in ("100644", "100755"):
            if not stat.S_ISREG(before.st_mode) or bool(before.st_mode & 0o111) != (mode == "100755"):
                raise ValueError("tracked regular source mode changed: " + relative)
            data = path.read_bytes()
        elif mode == "120000":
            if not stat.S_ISLNK(before.st_mode):
                raise ValueError("tracked symlink was replaced: " + relative)
            data = os.fsencode(os.readlink(path))
        else:
            raise ValueError("unrepresented tracked source mode: " + mode)
        after = path.lstat()
        stamp = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns)
        if stamp(before) != stamp(after):
            raise ValueError("tracked source changed during capture: " + relative)
        actual_blob = hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()
        if actual_blob != blob:
            raise ValueError("actual source differs from its Git index object: " + relative)
        item.update(size=len(data), sha256=hashlib.sha256(data).hexdigest())
        files.append(item)
    if not files:
        raise ValueError("tracked source input inventory is empty")
    head, tree, native = git("rev-parse", "HEAD", "HEAD^{tree}", "HEAD:compiler/native").decode("ascii").splitlines()
    return {"schema": "native-metadata-source-inputs-v1", "head": head, "tree": tree,
            "native_tree": native, "files": files, "gitlinks": gitlinks}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("record", "verify"))
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    current = capture()
    if args.mode == "record":
        with args.manifest.open("x") as output:
            json.dump(current, output, indent=2)
            output.write("\n")
    elif json.loads(args.manifest.read_text()) != current:
        raise ValueError("source files, symlinks, gitlinks or Git generation changed during qualification")
    print(json.dumps({"mode": args.mode, "files": len(current["files"]), "gitlinks": len(current["gitlinks"]),
                      "head": current["head"], "tree": current["tree"]}))


if __name__ == "__main__":
    main()
