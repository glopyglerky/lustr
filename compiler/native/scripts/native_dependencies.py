#!/usr/bin/env python3
"""Inspect a native binary's actual loader dependencies on Linux or Darwin.

The client and fuzz gates use this to distinguish a cold executable from one
bound to the current daemon's immutable base. Inspection errors are failures,
never evidence that an executable is standalone.
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys


def library_suffix():
    if sys.platform == "darwin":
        return ".dylib"
    if sys.platform == "linux":
        return ".so"
    raise RuntimeError(f"native dependency inspection is unsupported on {sys.platform}")


def inspect(binary):
    binary = Path(binary)
    if not binary.is_file():
        raise RuntimeError(f"native binary does not exist: {binary}")
    tool = "otool" if sys.platform == "darwin" else "readelf"
    library_suffix()  # Reject an unsupported host before choosing a tool.
    result = subprocess.run([tool, "-l" if sys.platform == "darwin" else "-d", str(binary)],
                            capture_output=True, text=True, errors="replace", timeout=10)
    if result.returncode:
        raise RuntimeError(f"{tool} could not inspect {binary}: {result.stderr.strip()}")
    if sys.platform == "darwin":
        fields = {}
        commands = 0
        for block in re.split(r"(?m)^Load command \d+\s*$", result.stdout)[1:]:
            command = re.search(r"(?m)^\s*cmd (LC_\w+)\s*$", block)
            if not command:
                raise RuntimeError(f"unreadable Mach-O load command in {binary}")
            commands += 1
            name = command[1]
            if name == "LC_RPATH" or name.endswith("_DYLIB"):
                value = re.search(r"(?m)^\s*(?:name|path) (.+) \(offset \d+\)\s*$", block)
                if not value:
                    raise RuntimeError(f"unreadable {name} value in {binary}")
                fields.setdefault(name, []).append(value[1])
        if not commands:
            raise RuntimeError(f"{binary} has no readable Mach-O load commands")
        dependencies = [value for name, values in fields.items()
                        if name.endswith("_DYLIB") and name != "LC_ID_DYLIB" for value in values]
        return dependencies, fields.get("LC_RPATH", []), fields.get("LC_ID_DYLIB", [])
    dependencies = re.findall(r"\(NEEDED\).*?\[([^]]+)\]", result.stdout)
    paths = re.findall(r"\((?:RUNPATH|RPATH)\).*?\[([^]]+)\]", result.stdout)
    return dependencies, [part for path in paths for part in path.split(":")], []


def standalone(binary):
    dependencies, _, _ = inspect(binary)
    if any("libbase-" in value for value in dependencies):
        raise RuntimeError(f"{binary} loads a daemon base: {dependencies}")


def base_dependency(binary, base_dir, owner):
    if not isinstance(owner, int) or owner <= 0:
        raise RuntimeError("a positive daemon owner PID is required")
    dependencies, paths, _ = inspect(binary)
    bases = [value for value in dependencies if "libbase-" in value]
    if len(bases) != 1:
        raise RuntimeError(f"{binary} must load exactly one daemon base, observed {bases}")
    prefix = "@rpath/" if sys.platform == "darwin" else ""
    pattern = re.escape(prefix) + rf"libbase-{owner}-[0-9a-f]{{32}}" + re.escape(library_suffix())
    if re.fullmatch(pattern, bases[0]) is None:
        raise RuntimeError(f"{binary} does not load an immutable base owned by daemon {owner}: {bases[0]}")
    base_dir = Path(base_dir).resolve()
    if not any(Path(path).is_absolute() and Path(path).resolve() == base_dir for path in paths):
        raise RuntimeError(f"{binary} has no runtime search path for {base_dir}: {paths}")
    base = base_dir / bases[0].removeprefix(prefix)
    if not base.is_file():
        raise RuntimeError(f"{binary} names a missing immutable base: {base}")
    if sys.platform == "darwin" and inspect(base)[2] != [bases[0]]:
        raise RuntimeError(f"{base} install name differs from the executable's dependency")
    return base


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--absent", action="store_true")
    parser.add_argument("--base-dir", type=Path)
    parser.add_argument("--owner", type=int)
    args = parser.parse_args()
    try:
        if args.absent:
            if args.base_dir is not None or args.owner is not None:
                raise RuntimeError("standalone inspection does not accept a base directory or owner")
            standalone(args.binary)
        else:
            if args.base_dir is None or args.owner is None:
                raise RuntimeError("daemon inspection requires --base-dir and --owner")
            print(base_dependency(args.binary, args.base_dir, args.owner))
        return 0
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"native dependency inspection failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
