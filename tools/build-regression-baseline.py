#!/usr/bin/env python3
"""Build the frozen failed-link baseline with its own unmodified constructor.

This is a correctness counterexample, not a performance lane. Its constructor
predates release-profile and selection-seed options and builds debug artifacts.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

BASELINE = "2add988c6397ac5175fcdf89a3aac17d359bf283"
CONSTRUCTOR = "tools/build-mixed-toolchain.rs"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_identity(source, expected):
    def git(*args):
        return subprocess.check_output(["git", "-C", str(source), *args])
    revision = git("rev-parse", "HEAD").decode().strip()
    if revision != expected or git("status", "--porcelain", "--untracked-files=all"):
        raise ValueError("baseline must be the clean exact frozen revision")
    constructor = source / CONSTRUCTOR
    if constructor.is_symlink() or constructor.read_bytes() != git("show", f"{expected}:{CONSTRUCTOR}"):
        raise ValueError("baseline constructor bytes differ from the frozen source")
    return {"revision": revision, "tree": git("rev-parse", "HEAD^{tree}").decode().strip(),
            "constructor_blob": git("rev-parse", f"HEAD:{CONSTRUCTOR}").decode().strip(),
            "constructor_sha256": digest(constructor)}


def command(evidence, label, argv, cwd, receipt):
    row = {"label": label, "argv": [str(arg) for arg in argv], "cwd": str(cwd)}
    receipt["commands"].append(row)
    paths = {name: evidence / f"{label}.{name}" for name in ("stdout", "stderr")}
    try:
        with paths["stdout"].open("xb") as out, paths["stderr"].open("xb") as err:
            result = subprocess.run(row["argv"], cwd=cwd, stdin=subprocess.DEVNULL, stdout=out, stderr=err)
        row["exit"] = result.returncode
    except OSError as error:
        row["startup_error"] = str(error)
        raise
    finally:
        row["streams"] = {name: {"file": path.name, "sha256": digest(path), "bytes": path.stat().st_size}
                          for name, path in paths.items() if path.is_file()}
    if result.returncode != 0:
        raise RuntimeError(f"{label} failed with status {result.returncode}; exact streams retained")


def build(source, tcc, output, evidence, rustc="rustc", expected=BASELINE):
    source, tcc = source.resolve(), tcc.resolve()
    output, evidence = output.absolute(), evidence.absolute()
    if output.exists() or evidence.exists():
        raise ValueError("baseline output and evidence must both be new")
    evidence.mkdir(parents=True)
    receipt = {"schema": "lustr-frozen-baseline-constructor-v1", "status": "failed",
               "scope": __doc__, "source": str(source), "output": str(output), "commands": []}
    try:
        before = source_identity(source, expected)
        receipt["baseline"] = before
        archived = evidence / "constructor-source.rs"
        archived.write_bytes((source / CONSTRUCTOR).read_bytes())
        binary = evidence / "build-mixed-toolchain"
        command(evidence, "compiler-version", [rustc, "-vV"], source, receipt)
        command(evidence, "constructor-build", [rustc, "--edition=2024", archived, "-o", binary], source, receipt)
        receipt["constructor_binary_sha256"] = digest(binary)
        # The frozen constructor has exactly four arguments and no --release.
        # Never apply the current constructor's setup steps to this old entry.
        command(evidence, "constructor-run", [binary, source, "--current-sdk", tcc, output], source, receipt)
        if source_identity(source, expected) != before:
            raise ValueError("baseline source changed during construction")
        receipt["status"] = "completed"
    except Exception as error:
        receipt["error"] = str(error)
        raise
    finally:
        (evidence / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "tcc", "output", "evidence"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    build(args.source, args.tcc, args.output, args.evidence)
