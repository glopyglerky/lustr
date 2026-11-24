#!/usr/bin/env python3
"""Check the actual C types and behavior of native erased drop callbacks.

The C compiler checks every emitted drop declaration against void (*)(void *).
This catches the incompatible indirect calls even with a C compiler whose UBSan
does not check function types. --sanitize keeps address/undefined checks too.
The existing full native differential and platform gates remain required.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_FIXTURES = (
    "dyn_box_drop", "arc_any_downcast", "weak_dyn_rc", "unsized_refcell_dyn_drop",
    "mpsc_p04", "panic_any_string_only", "thread_result_alias",
    "default_panic_unwinds", "dyn_fnonce_box", "unsized_arc_mutex_dyn_counter",
)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lust", required=True, type=Path)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--cc", default=os.environ.get("SAN_CC", "cc"))
    parser.add_argument("--cxx", default="c++")
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--fixture", action="append", help="existing pass fixture stem")
    parser.add_argument("--sanitize", action="store_true")
    parser.add_argument("--unwind-tables", action="store_true")
    args = parser.parse_args()
    args.lust = args.lust.resolve(strict=True)
    args.rustc = args.rustc.resolve(strict=True)
    args.output = args.output.resolve()
    # Evidence directories have one owner and are never reused by another run.
    args.output.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env.update(LUST_NO_DAEMON="1", LUST_NO_CACHE="1", RUST_BACKTRACE="0")
    env["LUST_CACHE_DIR"] = str(args.output / "native-cache")
    (args.output / "tmp").mkdir()
    env["TMPDIR"] = str(args.output / "tmp")
    if args.unwind_tables:
        env["LUST_UNWIND_TABLES"] = "1"
    report = {"schema": 1, "lust_sha256": digest(args.lust),
              "lust_core_sha256": digest(args.lust.with_name("lust-core")),
              "rustc_sha256": digest(args.rustc), "sanitizers": args.sanitize,
              "unwind_tables": args.unwind_tables,
              "commands": [], "cases": [], "passed": False}
    objects = args.output / "objects"
    objects.mkdir()
    # Clang treats discarded pointer qualifiers as an incompatible-pointer
    # diagnostic; GCC gives that same defect its own warning. Reject default
    # C diagnostics on both hosts instead of depending on one warning name.
    flags = ["-std=gnu11", "-O1", "-Werror"]
    if args.unwind_tables:
        flags += ["-fexceptions", "-fno-strict-aliasing", "-fwrapv"]
    if args.sanitize:
        flags += ["-fsanitize=address,undefined", "-fno-sanitize-recover=undefined"]
        env["ASAN_OPTIONS"] = "detect_leaks=1:allocator_may_return_null=1"

    def run(label, command, required=True):
        result = subprocess.run(list(map(str, command)), cwd=args.output,
                                env=env, capture_output=True, timeout=120)
        (args.output / f"{label}.stdout").write_bytes(result.stdout)
        (args.output / f"{label}.stderr").write_bytes(result.stderr)
        report["commands"].append({"label": label, "argv": list(map(str, command)),
                                   "exit_code": result.returncode})
        if required and result.returncode:
            raise RuntimeError(f"{label} failed with exit {result.returncode}: "
                               + result.stderr.decode(errors="replace")[:4000])
        return result

    try:
        run("native-version", [args.lust, "--version"])
        run("rust-version", [args.rustc, "-vV"])
        run("cc-version", [args.cc, "--version"])
        for fixture in args.fixture or DEFAULT_FIXTURES:
            if not re.fullmatch(r"[A-Za-z0-9_]+", fixture):
                raise ValueError("fixture must be a pass-fixture stem")
            source = ROOT / "compiler/native/tests/pass" / (fixture + ".rs")
            prefix = args.output / fixture
            native = args.output / (fixture + ".native")
            oracle = args.output / (fixture + ".rust")
            compiled = args.output / (fixture + ".cbin")
            run(fixture + "-native-build",
                [args.lust, "build", source, "-o", native, "--emit-c", prefix])
            run(fixture + "-rust-build",
                [args.rustc, "--edition=2024", "-A", "warnings", source, "-o", oracle])
            units = sorted(args.output.glob(fixture + ".*.c"))
            if not units:
                raise RuntimeError("native compilation emitted no C")
            unit_objects = []
            checked = 0
            for unit in units:
                text = unit.read_text()
                names = set(re.findall(r"\bvoid\s+(drop_[A-Za-z0-9_]+)\s*\(", text))
                # Include the actual runtime String vtable target, not a presumed
                # helper name. Old source names the incompatible typed function.
                names.update(re.findall(
                    r"lust_vt_string\[0\]\s*=\s*\(const void \*\)([A-Za-z0-9_]+)\s*;", text))
                # The threaded exception catcher likewise needs an exact ABI.
                names.update(re.findall(r"lust_unw_try\(([A-Za-z0-9_]+), h\)", text))
                checks = "\n".join(
                    '_Static_assert(__builtin_types_compatible_p(__typeof__(&'
                    + name + '), void (*)(void *)), "erased callback type: '
                    + name + '");' for name in sorted(names))
                annotated = text + "\n" + checks + "\n"
                key = hashlib.sha256(annotated.encode()).hexdigest()
                object_file = objects / (key + ".o")
                if not object_file.exists():
                    checked_unit = objects / (key + ".c")
                    checked_unit.write_text(annotated)
                    run(fixture + "-cc-" + key[:12],
                        [args.cc, *flags, "-c", checked_unit, "-o", object_file])
                checked += len(names)
                unit_objects.append(object_file)
            if not checked:
                raise RuntimeError("fixture exercised no erased callback type checks")
            linker = args.cc
            link_flags = flags
            if args.unwind_tables:
                eh_source = args.output / (fixture + ".eh.cc")
                eh_object = objects / (digest(eh_source) + ".eh.o")
                link_flags = [f for f in flags if f not in {
                    "-std=gnu11"}]
                if not eh_object.exists():
                    run(fixture + "-eh-build", [args.cxx, *link_flags, "-c", eh_source, "-o", eh_object])
                unit_objects.append(eh_object)
                linker = args.cxx
            run(fixture + "-cc-link", [linker, *link_flags, *unit_objects, "-lm", "-o", compiled])
            expected = run(fixture + "-rust-run", [oracle], required=False)
            sanitized = run(fixture + "-cc-run", [compiled], required=False)
            executions = [("C", sanitized)]
            if not args.unwind_tables:
                executions.append(("native", run(fixture + "-native-run", [native], required=False)))
            # With unwind tables, the .eh.cc unit supplies exception handling.
            # As in upstream's EH lane, only the separately linked C/C++ output
            # is the executable under test; the emitter's TCC temporary is not.
            for mode, result in executions:
                if (result.returncode, result.stdout) != (expected.returncode, expected.stdout):
                    raise RuntimeError(f"{fixture} {mode} output/status differs from Rust")
            if re.search(rb"[0-9]: runtime error|Sanitizer", sanitized.stderr):
                raise RuntimeError(f"{fixture} sanitizer rejected generated C")
            case = {"fixture": fixture, "source_sha256": digest(source),
                    "checked_declarations": checked, "c_units": len(units),
                    "exit_code": expected.returncode, "passed": True}
            report["cases"].append(case)
            print(json.dumps(case), flush=True)
        report["passed"] = True
    except (RuntimeError, ValueError, OSError, subprocess.TimeoutExpired) as error:
        report["error"] = str(error)
        print(str(error), file=sys.stderr)
    finally:
        (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
