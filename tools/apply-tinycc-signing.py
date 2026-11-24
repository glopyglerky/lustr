#!/usr/bin/env python3
"""Apply the declared Mach-O signing repair to an exact pinned TinyCC copy."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DECLARATION = ROOT / "upstream/tinycc-macho-signing.json"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def apply(source, receipt):
    declaration = json.loads(DECLARATION.read_text())
    patch = ROOT / declaration["patch"]
    if digest(patch.read_bytes()) != declaration["patch_sha256"]:
        raise RuntimeError("TinyCC integration patch differs from its declaration")
    path = source / declaration["file"]
    if path.is_symlink() or not path.is_file():
        raise RuntimeError("TinyCC Mach-O input must be a regular source file")
    before = digest(path.read_bytes())
    if before == declaration["original_sha256"]:
        # Archived source can be beneath an unrelated worktree. Fence Git
        # discovery at its parent and discard inherited repository overrides;
        # a source checkout at cwd still discovers its own repository normally.
        env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        env["GIT_CEILING_DIRECTORIES"] = str(source.parent)
        for flags in [["--check"], []]:
            subprocess.run(["git", "apply", *flags, str(patch)], cwd=source, env=env, check=True)
        action = "applied"
    elif before == declaration["patched_sha256"]:
        action = "already-applied"
    else:
        raise RuntimeError("TinyCC Mach-O source is neither the pinned original nor its declared integration")
    after = digest(path.read_bytes())
    if after != declaration["patched_sha256"]:
        raise RuntimeError("TinyCC patched source differs from its declaration")
    result = dict(declaration, action=action, observed_before_sha256=before,
                  observed_after_sha256=after, declaration_sha256=digest(DECLARATION.read_bytes()),
                  application_tool_sha256=digest(Path(__file__).read_bytes()))
    receipt.parent.mkdir(parents=True, exist_ok=True)
    receipt.write_text(json.dumps(result, indent=2) + "\n")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    result = apply(args.source.resolve(strict=True), args.receipt)
    print(json.dumps({key: result[key] for key in ["revision", "action", "observed_after_sha256"]}))


if __name__ == "__main__":
    main()
