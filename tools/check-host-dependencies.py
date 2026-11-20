#!/usr/bin/env python3
"""Reject a service-manager dependency in the active compiler/build sources.

This source and lockfile gate supplements actual native execution. It does not
claim to prove every dynamically constructed process name or every host library.
Historical evidence and upstream documentation are outside its mutation scope.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]
# New runtime/package directories must not silently escape this check. Scan
# source and build inputs wherever they live; only immutable evidence and prose
# are excluded. The guard and its planted negative controls name forbidden APIs
# without depending on them.
HISTORICAL = ("docs/", "performance/")
AUDIT_TOOLS = {"tools/check-host-dependencies.py", "tools/test-host-dependencies.py"}
UNIT_EXTENSIONS = {".service", ".socket", ".timer", ".target", ".path", ".slice",
                   ".scope", ".mount", ".automount", ".swap", ".device"}
SOURCE_EXTENSIONS = {".rs", ".c", ".h", ".cc", ".cpp", ".py", ".sh", ".bash",
                     ".yml", ".yaml", ".toml", ".lock", ".ps1", ".bat", ".cmd",
                     ".mk", ".cmake", ".nix", ".spec", ".install", ".pc"} | UNIT_EXTENSIONS
BUILD_NAMES = {"makefile", "gnumakefile", "cmakelists.txt", "meson.build",
               "meson_options.txt", "pkgbuild", ".gitmodules"}
FORBIDDEN = re.compile(
    rb"(?<![A-Za-z0-9_])(?:libsystemd(?:[-_][A-Za-z0-9_-]+)?|systemd(?:[-_][A-Za-z0-9_-]+)?|"
    rb"systemctl|journalctl|loginctl|sd_notify[a-z_0-9]*|sd_listen_fds[a-z_0-9]*|"
    rb"sd_(?:bus|event|journal|device|id128|hwdb|boot|path|machine|login|pid|peer|uid|seat|session|get)[a-z_0-9]*|"
    rb"sd-notify|NOTIFY_SOCKET|LISTEN_FDS|LISTEN_PID|LISTEN_FDNAMES)(?![A-Za-z0-9_])",
    re.IGNORECASE,
)


def source_input(path):
    name = path.name.lower()
    if path.suffix.lower() in SOURCE_EXTENSIONS or name in BUILD_NAMES:
        return True
    if name in {"dockerfile", "containerfile"} or name.startswith(("dockerfile.", "containerfile.")):
        return True
    # Executable entry scripts often have no extension. Reading only the prefix
    # avoids loading unrelated generated binaries merely to classify them.
    with path.open("rb") as file:
        return file.read(2) == b"#!"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=ROOT, capture_output=True, check=True,
    )
    paths = sorted(set(result.stdout.decode().rstrip("\0").split("\0")))
    violations = []
    checked = []
    packages = set()
    for name in paths:
        if name in AUDIT_TOOLS or name.startswith(HISTORICAL):
            continue
        path = ROOT / name
        if not path.is_file() or not source_input(path):
            continue
        data = path.read_bytes()
        checked.append(name)
        if path.suffix.lower() in UNIT_EXTENSIONS:
            violations.append({"path": name, "line": 1, "token": "service-manager unit file"})
        if path.name == "Cargo.lock":
            lock = tomllib.loads(data.decode())
            packages.update(p["name"] for p in lock.get("package", []))
        for match in FORBIDDEN.finditer(data):
            violations.append({"path": name, "line": data[:match.start()].count(b"\n") + 1,
                               "token": match.group().decode()})
    # A missing checkout or a misplaced invocation must not pass vacuously.
    required = {"Cargo.toml", "compiler/native/Cargo.lock", "compiler/native/src/client.rs",
                "compiler/native/src/daemon.rs", "src/host_runtime.rs", ".github/workflows/ci.yml"}
    missing = sorted(required - set(checked))
    report = {"schema": 1, "checked_files": len(checked), "lockfile_packages": sorted(packages),
              "violations": violations, "missing_required_files": missing,
              "passed": not violations and not missing}
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
