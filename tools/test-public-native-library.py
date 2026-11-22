#!/usr/bin/env python3
"""Qualify the installed retained scalar library through real pinned Cargo.

This is a bounded public artifact integration campaign, not full Rust coverage
or a performance claim. Cargo determines the actual compiler arguments. Native
work observations are source instrumentation, corroborated by immutable objects;
process, artifact, metadata-reader and runtime checks are independent evidence.
Reference protocols: Rust 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494
rustc_errors/src/json.rs, rustc_interface/src/passes.rs, and rustc_metadata/src
rmeta/encoder.rs and locator.rs; Cargo submodule c07ac2c4d12237effc4b94f4543769b421790045,
src/compiler/mod.rs (split-metadata emission and paired dependency paths).
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parents[1]


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "tools" / filename)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


retained = module("public_library_retained_readers", "test-retained-native-metadata.py")
construction = module("public_library_sdk_reader", "build-metadata-sdk-receipt.py")
require, sha, file_identity = retained.require, retained.sha, retained.file_identity
TARGET = "x86_64-unknown-linux-gnu"
CRATE = "native_leaf"
NAMES = {"project", "limit", "merge"}
BASE = b"""#![no_std]
pub fn project(input: u64) -> u64 { limit(input) ^ 165u64 }
fn limit(input: u64) -> u64 { input & 63u64 }
pub fn merge(left: u64, right: u64) -> u64 { left ^ right }
"""
EDIT = BASE.replace(b"input & 63u64", b"input & 127u64")
INVALID = EDIT.replace(b"input & 127u64", b"true")
OTHER = BASE.replace(b"input & 63u64", b"input & 255u64").replace(b"left ^ right", b"left | right")
ASSERTIONS_PER_CONSUMER = 65538 * 4
HISTORY = [
    ("cargo-cold", "first", BASE, 63, "xor", "cold"),
    ("cargo-noop", "first", BASE, 63, "xor", "noop"),
    ("private-body-edit", "first", EDIT, 127, "xor", "edit"),
    ("body-edit-noop", "first", EDIT, 127, "xor", "noop"),
    ("typed-invalid", "first", INVALID, 127, "xor", "invalid"),
    ("source-repair", "first", EDIT, 127, "xor", "repair"),
    ("repair-noop", "first", EDIT, 127, "xor", "noop"),
    ("different-unit", "second", OTHER, 255, "or", "cold"),
    ("first-unit-revisit", "first", EDIT, 127, "xor", "noop"),
]
SUCCESS_CASES = [row[0] for row in HISTORY if row[-1] != "invalid"]
EXPECTED_ASSERTIONS = len(SUCCESS_CASES) * ASSERTIONS_PER_CONSUMER
EVICTION_MASKS = [255, 511, 1023, 127]
EVICTION_ASSERTIONS = len(EVICTION_MASKS) * ASSERTIONS_PER_CONSUMER
METADATA_TRANSITIONS = [
    ("metadata-explicit-no", "no"),
    ("metadata-default", None),
    ("metadata-explicit-yes", "yes"),
    ("metadata-restored-no", "no"),
]
METADATA_TRANSITION_ASSERTIONS = len(METADATA_TRANSITIONS) * ASSERTIONS_PER_CONSUMER
VERSION_REFUSAL = "incompatible-public-library-version"
RAW_REQUEST_OFFSET = len(HISTORY) + len(EVICTION_MASKS) + 1
JSON_OPTIONS = "diagnostic-rendered-ansi,artifacts,future-incompat,unused-externs-silent"
CHECK_CFG = ["cfg(docsrs,test)", "cfg(feature, values())"]


def write_new(path, data):
    path = Path(path)
    with path.open("xb") as target:
        target.write(data if isinstance(data, bytes) else data.encode())


def json_write(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def process_identity(pid, executable):
    try:
        if Path(f"/proc/{pid}/exe").resolve(strict=True) != Path(executable).resolve(strict=True):
            return None
        fields = Path(f"/proc/{pid}/stat").read_text().rsplit(") ", 1)[1].split()
        if fields[0] == "Z":
            return None
        return [pid, fields[19]]
    except (OSError, IndexError):
        return None


def trace_events(data, cwd, *, application=None):
    """Parse complete observed process events; shortened or split tails fail.

    Calls are ordered by their entry, so a child running before its parent's
    clone-resume still inherits that parent's observed working directory.
    """
    require(not data or data.endswith(b"\n"), "trace ends inside a record")
    pending, events, exits, undecoded = {}, [], {}, {}
    for offset, line in enumerate(data.decode("utf-8", errors="strict").splitlines()):
        if not line.strip():
            continue
        match = re.fullmatch(r"\s*(?:(?:\[pid\s+(\d+)\]|(\d+))\s+)?(.*)", line)
        require(match is not None, "unparsed process trace prefix")
        pid, record = match.group(1) or match.group(2) or "entry", match.group(3)
        if record.startswith("--- SIG"):
            require(re.fullmatch(r"--- SIG[A-Z0-9]+ .* ---", record) is not None, "unparsed signal")
            continue
        if record.startswith("+++"):
            require(pid not in pending and pid not in exits, "duplicate exit or unfinished syscall at exit")
            ended = re.fullmatch(r"\+\+\+ exited with (\d+) \+\+\+", record)
            killed = re.fullmatch(r"\+\+\+ killed by (SIG[A-Z0-9]+)(?: \(core dumped\))? \+\+\+", record)
            require(ended is not None or killed is not None, "unparsed process exit")
            exits[pid] = int(ended.group(1)) if ended else killed.group(1)
            if pid in undecoded:
                start = undecoded.pop(pid)
                events.append((start, pid, "undecoded_terminal_stop", "", exits[pid]))
            continue
        require(pid not in undecoded, "syscall follows an undecoded stop before exit")
        if record == "???( <unfinished ...>":
            require(pid not in pending and pid not in exits, "undecoded stop changed an observed identity")
            undecoded[pid] = offset
            continue
        resumed = re.fullmatch(r"<\.\.\. ([a-z0-9_]+) resumed>(.*)", record)
        if resumed:
            require(pid in pending and pending[pid][1] == resumed.group(1), "unmatched syscall resume")
            start, _, before = pending.pop(pid)
            record = before + resumed.group(2)
        else:
            require(pid not in pending, "syscall replaced an unfinished call")
            start = offset
            if record.endswith(" <unfinished ...>"):
                syscall = record.split("(", 1)[0]
                require(syscall in {"execve", "execveat", "chdir", "fchdir", "clone", "clone3", "fork", "vfork", "exit_group"},
                        "unknown unfinished syscall")
                pending[pid] = (start, syscall, record.removesuffix(" <unfinished ...>"))
                continue
        complete = re.fullmatch(r"([a-z0-9_]+)\((.*)\)\s+=\s+(.*)", record)
        require(complete is not None, "unparsed complete syscall: " + record)
        events.append((start, pid, complete.group(1), complete.group(2), complete.group(3)))
    require(not pending and not undecoded, "syscall crosses an externally completed request boundary")
    directories, parents, attempts, changes = {}, {}, [], []
    groups, group_exits, terminal_stops = {}, {}, []
    root_pid = None
    for _, pid, call, arguments, result in sorted(events):
        directories.setdefault(pid, str(cwd))
        if call in {"clone", "clone3", "fork", "vfork"}:
            child = re.match(r"([1-9][0-9]*)(?:\s|$)", result)
            if child:
                child = child.group(1)
                require(child not in parents, "process creation reused an observed live identity")
                parents[child] = pid
                directories[child] = directories[pid]
                if call in {"clone", "clone3"} and re.search(r"\bflags=[^,}]*\bCLONE_THREAD\b", arguments):
                    groups[child] = groups.get(pid, pid)
            else:
                require(re.fullmatch(r"-1 [A-Z][A-Z0-9_]* \([^\n]*\)", result) is not None, "unparsed failed process creation")
            continue
        if call == "exit_group":
            require(result == "?", "unexpected exit_group result")
            group_exits[groups.get(pid, pid)] = arguments
            continue
        if call == "undecoded_terminal_stop":
            leader = groups.get(pid)
            # strace 6.8 emits this record when it cannot read the syscall
            # registers. Require a known non-leader, a preceding group exit,
            # and actual matching terminal events. Retain the unparsed stop
            # separately from the executable inventory.
            require(leader is not None and leader != pid and group_exits.get(leader) == "0"
                    and result == exits.get(leader) == 0,
                    "undecoded stop outside a completed observed thread-group shutdown")
            terminal_stops.append({"tid": pid, "leader": leader, "status": result})
            continue
        require(call not in {"execveat", "fchdir"}, "unrepresented executable or cwd descriptor operation")
        if call == "chdir":
            path = json.loads(arguments)
            require(isinstance(path, str) and result == "0", "failed or malformed working-directory change")
            directories[pid] = str((Path(directories[pid]) / path).resolve())
            changes.append({"pid": pid, "cwd": directories[pid]})
            continue
        require(call == "execve", "unexpected selected syscall: " + call)
        filename, end = json.JSONDecoder().raw_decode(arguments)
        rest = arguments[end:]
        require(isinstance(filename, str) and rest.startswith(", "), "shortened executable path")
        argv, end = json.JSONDecoder().raw_decode(rest[2:])
        rest = rest[end + 2:]
        require(isinstance(argv, list) and argv and all(isinstance(v, str) for v in argv)
                and rest.startswith(", "), "shortened executable arguments")
        environment, end = json.JSONDecoder().raw_decode(rest[2:])
        require(not rest[end + 2:].strip() and isinstance(environment, list)
                and all(isinstance(v, str) and "=" in v for v in environment), "shortened exec environment; tracing requires -v")
        env = {}
        for value in environment:
            key, value = value.split("=", 1)
            require(key not in env, "duplicate environment key")
            env[key] = value
        require(Path(filename).is_absolute() or (
            application is not None and pid == root_pid and argv == [filename]
            and (Path(directories[pid]) / filename).resolve() == application.resolve()
        ), "unbound relative executable")
        success = result == "0"
        require(success or re.fullmatch(r"-1 [A-Z][A-Z0-9_]* \([^\n]*\)", result) is not None,
                "unparsed executable result")
        if root_pid is None:
            root_pid = pid
        attempts.append({"pid": pid, "filename": filename, "argv": argv, "environment": env,
                         "cwd": directories[pid], "exec_result": result, "succeeded": success})
    require(attempts and root_pid is not None, "trace lacks actual initial execution")
    return {"root_pid": root_pid, "attempts": attempts, "parents": parents, "exits": exits,
            "cwd_changes": changes, "terminal_undecoded_stops": terminal_stops}


def frozen_prefix(previous, current):
    require(current.startswith(previous), "an observed process trace prefix was rewritten")
    require(not current or current.endswith(b"\n"), "response trace prefix is incomplete")


def parse_compile(attempt, *, executable=False):
    args = attempt["argv"][1:]
    result = {"codegen": {}, "unstable": {}, "check_cfg": [], "search_dirs": []}
    position = 0
    while position < len(args):
        arg = args[position]
        position += 1
        def next_value():
            nonlocal position
            require(position < len(args), "missing actual compiler argument")
            value = args[position]
            position += 1
            return value
        if arg == "-C" or arg.startswith("-C"):
            option = next_value() if arg == "-C" else arg[2:]
            key, separator, value = option.partition("=")
            require(separator and key not in result["codegen"], "duplicate or valueless codegen setting")
            result["codegen"][key] = value
        elif arg == "-Z" or arg.startswith("-Z"):
            option = next_value() if arg == "-Z" else arg[2:]
            key, separator, value = option.partition("=")
            require(separator and key not in result["unstable"], "duplicate or valueless unstable setting")
            result["unstable"][key] = value
        elif arg == "-L" or arg.startswith("-L"):
            option = next_value() if arg == "-L" else arg[2:]
            require(option.startswith("dependency="), "unrepresented library search class")
            result["search_dirs"].append(str((Path(attempt["cwd"]) / option[11:]).resolve()))
        elif arg.startswith("--"):
            flag, equal, value = arg.partition("=")
            value = value if equal else next_value()
            if flag == "--check-cfg":
                result["check_cfg"].append(value)
            elif flag == "--extern" and executable:
                result.setdefault(flag, []).append(value)
            else:
                require(flag not in result, "duplicate actual compiler flag")
                result[flag] = value
        else:
            require(not arg.startswith("-") and "source" not in result, "unexpected compiler positional/short flag")
            result["source"] = str((Path(attempt["cwd"]) / arg).resolve())
    require("source" in result, "compiler invocation has no real source")
    return result


def library_options(facts):
    require(set(facts["unstable"]) <= {"embed-metadata"}
            and "embed-metadata" not in facts["codegen"], "unrepresented metadata option channel")
    choice = facts["unstable"].get("embed-metadata")
    require(choice in {None, "yes", "no"}, "unrepresented metadata embedding value")
    require(facts.get("--force-warn") == "unused_crate_dependencies", "missing actual forced dependency lint")
    return {"embed_metadata": None if choice is None else choice == "yes",
            "force_warn_unused_crate_dependencies": True}


def metadata_replay_argv(invocation, choice):
    # Deliberate direct-request variation, never a rewrite of Cargo's invocation.
    # The captured base must contain precisely the real split option pair.
    argv = list(invocation["argv"])
    require(argv.count("-Z") == 1, "captured Cargo unstable option inventory changed")
    at = argv.index("-Z")
    require(argv[at + 1] == "embed-metadata=no", "captured Cargo metadata option changed")
    if choice is None:
        del argv[at:at + 2]
    else:
        argv[at + 1] = "embed-metadata=" + choice
    return argv


def declared_compile(facts, project, *, check=False):
    require(facts.get("--crate-name") == CRATE and facts.get("--edition") == "2024"
            and facts.get("--crate-type") in {"lib", "rlib"}, "Cargo changed the declared leaf identity")
    emits = "dep-info,metadata" if check else "dep-info,metadata,link"
    require(facts.get("--emit") == emits, "Cargo did not request the declared leaf artifacts")
    require(facts.get("--error-format") == "json" and facts.get("--json") == JSON_OPTIONS
            and facts["check_cfg"] == CHECK_CFG, "Cargo diagnostic/check-cfg request differs from pinned source")
    require(facts["source"] == str(project["source"]), "Cargo did not compile the unchanged selected source")
    codegen = facts["codegen"]
    require(set(codegen) == {"panic", "embed-bitcode", "metadata", "extra-filename"}
            and codegen["panic"] == "abort" and codegen["embed-bitcode"] == "no",
            "Cargo supplied an unrepresented or rewritten leaf profile")
    require(re.fullmatch(r"[0-9a-f]{16}", codegen["metadata"]) is not None
            and re.fullmatch(r"-[0-9a-f]{16}", codegen["extra-filename"]) is not None,
            "Cargo metadata or output disambiguator is malformed")
    require(facts["unstable"] == ({} if check else {"embed-metadata": "no"})
            and facts.get("--force-warn") == "unused_crate_dependencies",
            "Cargo split-metadata or forced-lint request differs from pinned source")
    allowed = {"codegen", "unstable", "check_cfg", "search_dirs", "source", "--crate-name", "--edition", "--crate-type",
               "--emit", "--error-format", "--json", "--out-dir", "--force-warn"}
    # This real dependency-free unit in pinned Cargo's new build layout has no
    # -L dependency argument (ca383 cold invocation). Preserve that actual fact.
    require(set(facts) == allowed and facts["search_dirs"] == [], "unknown actual Cargo compiler argument")
    out = Path(facts["--out-dir"]).resolve()
    require(out.is_relative_to(project["build"]), "Cargo compiler output escaped its configured build directory")
    suffix = codegen["extra-filename"]
    outputs = {"metadata": out / f"lib{CRATE}{suffix}.rmeta", "dep-info": out / f"{CRATE}{suffix}.d"}
    if not check:
        outputs["link"] = out / f"lib{CRATE}{suffix}.rlib"
    return outputs


def information_query(argv):
    args = argv[1:]
    if args == ["-vV"]:
        return True
    standard = ["-", "--crate-name", "___", "--print=file-names", "--crate-type", "bin",
                "--crate-type", "rlib", "--crate-type", "dylib", "--crate-type", "cdylib",
                "--crate-type", "staticlib", "--crate-type", "proc-macro", "--print=sysroot",
                "--print=split-debuginfo", "--print=crate-name", "--print=cfg", "-Wwarnings"]
    return args == standard or args == standard[:4] + ["--target", TARGET] + standard[4:]


def inspect_processes(parsed, package, expected_root, root_argv, projects, *, daemon=False, complete=None, application=None):
    """Classify exact product/query/planning executables, never compiler names alone."""
    if complete is None:
        complete = not daemon
    entry = (package / "bin/lustr").resolve()
    core = (package / "bin/lust-core").resolve()
    cargo = (package / "bin/cargo").resolve()
    llvm = package / "lib/rustlib" / TARGET / "bin"
    tools = {(llvm / name).resolve(): name for name in ("llvm-ar", "llvm-objcopy")}
    tcc = package / "lib/lust/tcc/bin/tcc"
    hashes = {Path(shutil.which("sha256sum")).resolve()}
    actual = parsed["attempts"]
    require(actual[0]["filename"] == str(expected_root) and actual[0]["argv"] == list(map(str, root_argv))
            and actual[0]["succeeded"], "trace initial execution differs from the actual requested command")
    compiles, artifacts, plans, runs = [], [], [], []
    manifests = {str(row["manifest"]) for row in projects.values()}
    for index, attempt in enumerate(actual):
        path = Path(attempt["filename"])
        resolved = path.resolve()
        argv = attempt["argv"]
        if not attempt["succeeded"]:
            # Rust Command PATH lookup may try the fixed hash executable in
            # earlier PATH directories. No failed compiler/tool attempt is hidden.
            require(path.name == "sha256sum" and " ENOENT " in attempt["exec_result"]
                    and len(argv) == 2 and argv[0] == "sha256sum"
                    and argv[1] in {str(package / "bin" / name) for name in ("cargo", "lustr", "rustc")}
                    and str(path.parent) in attempt["environment"].get("PATH", "").split(os.pathsep),
                    "undeclared or failed product executable attempt: " + str(path))
            continue
        if index == 0:
            # A direct replay starts at the compiler itself; Cargo normally
            # starts above it. Count that already validated root invocation,
            # without requiring it to have a parent or treating Cargo/daemon
            # startup and read-only compiler queries as body compilations.
            if not daemon and resolved == entry and not information_query(argv):
                compiles.append(attempt)
            continue
        require(argv and (argv[0] == str(path) or (resolved in hashes and argv[0] == "sha256sum")),
                "executable and argv[0] differ")
        if resolved == entry:
            if information_query(argv):
                pass
            else:
                require(not daemon, "native owner launched another compiler request while serving one")
                compiles.append(attempt)
        elif resolved == core:
            require(argv[1:] == ["--lustr-print-cfg"], "product spawned a fresh compiler/daemon")
        elif resolved == cargo:
            args = argv[1:]
            require("--manifest-path" in args and args[args.index("--manifest-path") + 1] in manifests,
                    "nested Cargo planning did not bind an owned manifest")
            if args and args[0] == "metadata":
                allowed = {"--format-version", "--manifest-path", "--filter-platform", "--offline",
                           "--locked", "--no-default-features", "--no-deps"}
                i = 1
                while i < len(args):
                    flag = args[i]
                    require(flag in allowed, "nested metadata command changed its graph contract")
                    i += 2 if flag in {"--format-version", "--manifest-path", "--filter-platform"} else 1
                require(args[args.index("--format-version") + 1] == "1", "nested Cargo metadata format differs")
            else:
                require((args and args[0] in {"build", "check"} and args[1:4] == ["--unit-graph", "-Z", "unstable-options"])
                        and ("--lib" in args or "--bin" in args) and "--package" in args,
                        "product nested Cargo attempted compilation or an unrepresented operation")
                allowed = {"--manifest-path", "--package", "--lib", "--bin", "--offline", "--locked", "--no-default-features", "--target"}
                if "--bin" in args:
                    manifest = args[args.index("--manifest-path") + 1]
                    name = args[args.index("--bin") + 1]
                    require("--lib" not in args and any(str(row["manifest"]) == manifest
                            and row.get("binary_target") == name for row in projects.values()),
                            "nested binary unit planning selected another target")
                i = 4
                while i < len(args):
                    flag = args[i]
                    require(flag in allowed, "nested unit-graph command changed its selected unit")
                    i += 2 if flag in {"--manifest-path", "--package", "--target", "--bin"} else 1
            require(attempt["environment"].get("RUSTC") == str(package / "bin/lustr")
                    and not attempt["environment"].get("RUSTC_WRAPPER")
                    and not attempt["environment"].get("RUSTC_WORKSPACE_WRAPPER"),
                    "nested planning changed its native compiler query owner")
            plans.append(attempt)
        elif resolved in hashes:
            require(len(argv) == 2 and argv[1] in {str(package / "bin" / name) for name in ("cargo", "lustr", "rustc")},
                    "product hash utility operated outside exact planning tools")
        elif resolved in tools:
            require(daemon, "artifact work escaped the retained native owner")
            artifacts.append(attempt)
        elif path == tcc and daemon:
            scratch = Path(parsed["attempts"][0]["environment"]["TMPDIR"]) / ("lust-daemon-" + parsed["root_pid"])
            require(len(argv) == 8 and argv[1:6] == ["-B" + str(package / "lib/lust/tcc/lib/tcc"),
                    "-c", "-w", "-fno-asynchronous-unwind-tables", "-o"],
                    "unexpected native C compiler options")
            c_source, obj = Path(argv[-1]), Path(argv[-2])
            require(c_source.is_absolute() and c_source == obj.with_suffix(".c")
                    and obj.suffix == ".o" and obj.parent.parent == scratch
                    and re.fullmatch(r"[0-9]+(?:r[0-9]+)?", obj.parent.name)
                    and re.fullmatch(r"u[0-9]+\.o", obj.name),
                    "native C compiler left its retained per-unit object inputs")
            artifacts.append(attempt)
        elif (not daemon and application is not None and
              (Path(attempt["cwd"]) / path).resolve() == application.resolve() and argv == [str(path)]):
            runs.append(attempt)
        else:
            raise AssertionError("undeclared product executable: " + str(path))
        require(attempt["pid"] in parsed["parents"] or attempt in runs and attempt["pid"] == parsed["root_pid"],
                "child execution has no observed process ancestry")
    # Every finite child must actually exit successfully. The public compiler
    # may report a declared source/refusal error; its status is checked by caller.
    for attempt in actual[1:]:
        if attempt["succeeded"] and attempt not in compiles:
            status = parsed["exits"].get(attempt["pid"])
            require(status == 0 or (not complete and status is None),
                    "query/planning/artifact child exit was missing or failed")
    return {"compiles": compiles, "artifacts": artifacts, "plans": plans, "runs": runs}


def diagnostic(messages, source, text, code, *, tagged=True):
    errors = [row for row in messages if row.get("level") == "error" and isinstance(row.get("code"), dict)]
    require(len(errors) == 1 and errors[0]["code"].get("code") == code,
            "typed invalid source lacks exactly one authoritative type error")
    error = errors[0]
    require((error.get("$message_type") == "diagnostic" if tagged else error.get("$message_type") in {None, "diagnostic"})
            and isinstance(error.get("rendered"), str) and isinstance(error.get("children"), list),
            "diagnostic is not the pinned Cargo/rustc JSON shape")
    start = text.index(b"true")
    line = text[:start].count(b"\n") + 1
    column = start - (text.rfind(b"\n", 0, start) + 1) + 1
    spans = [span for span in error.get("spans", []) if span.get("is_primary")]
    require(len(spans) == 1, "type error has no unique primary span")
    span = spans[0]
    shown = Path(span.get("file_name", ""))
    require((source.parent.parent / shown).resolve() == source.resolve() if not shown.is_absolute() else shown.resolve() == source.resolve(),
            "type error names another source")
    require({key: span.get(key) for key in ("byte_start", "byte_end", "line_start", "line_end", "column_start", "column_end")}
            == {"byte_start": start, "byte_end": start + 4, "line_start": line, "line_end": line,
                "column_start": column, "column_end": column + 4}, "type error changed the exact primary source span")
    if code == "L0110":
        require(error.get("message") == "mismatched types: expected `u64`, found `bool`",
                "native checker changed its exact type mismatch")
    else:
        require(code == "E0308" and error.get("message") == "mismatched types"
                and span.get("label") == "expected `u64`, found `bool`",
                "original Rust type mismatch differs from its pinned diagnostic")
    require(not any("internal compiler error" in row.get("message", "").lower() for row in messages),
            "implementation failure replaced a source rejection")
    return error


def dep_info(data, outputs, source, cwd, *, modules=(), environment=()):
    require(b"\r" not in data and data.endswith(b"\n"), "dep-info has malformed record endings")
    rules = {}
    observed_environment = []
    for line in data.decode().splitlines():
        if not line:
            continue
        if line.startswith("# env-dep:"):
            observed_environment.append(line.removeprefix("# env-dep:"))
            continue
        require(not line.startswith("#"), "scalar leaf emitted unobserved environment/checksum dependencies")
        target, separator, prerequisites = line.partition(":")
        require(separator and target not in rules, "dep-info has a malformed or duplicate target")
        target = target.replace("\\ ", " ")
        prerequisites = re.findall(r"(?:\\ |[^ ])+", prerequisites.strip())
        prerequisites = [value.replace("\\ ", " ") for value in prerequisites]
        resolve = lambda value: str((Path(cwd) / value).resolve())
        rules[resolve(target)] = [resolve(value) for value in prerequisites]
    inputs = [str(path) for path in (source, *modules)]
    expected = {str(path): inputs for path in outputs.values()}
    expected.update(dict.fromkeys(inputs, []))
    require(rules == expected, "dep-info targets or actual source prerequisites differ from requested artifacts")
    require(observed_environment == list(environment), "dep-info environment reads differ from the actual request")
    return rules


def source_rows(rows, label):
    require(isinstance(rows, list), label + " inventory is missing")
    result = {}
    for row in rows:
        require(isinstance(row, dict) and row.get("name") in NAMES and row["name"] not in result,
                label + " has a missing, foreign or repeated definition")
        native_id = row.get("native_id")
        require(isinstance(native_id, list) and len(native_id) == 2 and all(type(value) is int for value in native_id),
                label + " native identity is incomplete")
        result[row["name"]] = row
    return result


def witness(root, relative):
    require(isinstance(relative, str) and relative and not Path(relative).is_absolute(), "unbound object witness")
    path = root / relative
    require(not path.is_symlink() and path.resolve(strict=True).is_relative_to(root.resolve()), "object witness escaped owned request")
    return path.read_bytes()


def split_metadata(full, stub):
    # Pinned Rust encoder2694-2705/2800-2819 writes a header-only stub with
    # unhashed-position zero and the same SVH as the separately emitted rmeta.
    for label, data in (("full", full), ("stub", stub)):
        require(len(data) > 52 and data[:8] == b"rust\0\0\0\x0c"
                and data.endswith(b"rust-end-file"), label + " metadata has an invalid pinned header or end marker")
        root = int.from_bytes(data[8:16], "little")
        require(40 <= root < len(data) - 13, label + " metadata has an invalid root/header position")
    unhashed = int.from_bytes(full[16:24], "little")
    require(40 <= unhashed < len(full) - 13 and stub[16:24] == bytes(8),
            "split metadata changed full/stub unhashed positions")
    require(stub[24:40] == full[24:40], "split metadata stub has a different crate hash")
    require(len(stub) < len(full), "split metadata embedded a full payload instead of a stub")


def compare_split_metadata(full, stub, reference_full, reference_stub):
    split_metadata(full, stub)
    split_metadata(reference_full, reference_stub)
    # Only the SVH differs: native checked inputs replace Rust's HIR hash.
    # All other stub bytes must equal the real original producer's output.
    require(stub[:24] == reference_stub[:24] and stub[40:] == reference_stub[40:],
            "split metadata stub format/identity differs from original Rust")
    return {"full_sha256": sha(full), "stub_sha256": sha(stub),
            "reference_full_sha256": sha(reference_full), "reference_stub_sha256": sha(reference_stub),
            "crate_hash_le_hex": full[24:40].hex(), "reference_crate_hash_le_hex": reference_full[24:40].hex(),
            "equal_except_crate_hash": True}


def unused_dependencies(messages):
    # json.rs EmitTyped::UnusedExtern uses singular snake_case, even though
    # an older comment in rustc_errors/lib.rs spells the discriminator plural.
    rows = [row for row in messages if row.get("$message_type") in {"unused_extern", "unused_externs"}
            or "unused_extern_names" in row]
    expected = {"$message_type": "unused_extern", "lint_level": "force-warn", "unused_extern_names": []}
    require(rows == [expected], "forced unused-dependency JSON must contain exactly one original empty result")
    return rows[0]


def inspect_artifact(report_dir, record, outputs, source, source_bytes, kind, previous, *, embed_metadata=False):
    objects = source_rows(record.get("objects"), "native object")
    require(set(objects) == NAMES, "objects do not match original source declarations")
    mapping = {row["native_symbol"]: row["rust_symbol"] for row in objects.values()}
    require(len(mapping) == 3 and len(set(mapping.values())) == 3, "symbol mapping duplicates an emitted instance")
    expected_members = {}
    for name, row in objects.items():
        before = witness(report_dir, row.get("original_path"))
        after = witness(report_dir, row.get("renamed_path"))
        require(sha(before) == row.get("original_sha256") and sha(after) == row.get("renamed_sha256"), "object witness hash is false")
        old, new = retained.elf_object(before), retained.elf_object(after)
        native = row["native_symbol"]
        require(re.fullmatch(r"f_[0-9a-f]{16}", native) is not None, "native symbol is not its stable definition")
        helper = "tramp_" + native[2:]
        calls = {objects["limit"]["native_symbol"]} if name == "project" else set()
        require(old["defined"] == old["functions"] == {native, helper} and old["undefined"] == old["external_calls"] == calls,
                "actual TinyCC object definitions or calls differ from source")
        require(new["defined"] == new["functions"] == {row["rust_symbol"], helper}
                and new["undefined"] == new["external_calls"] == {mapping[value] for value in calls}
                and retained.elf_object(before, mapping)["contents"] == new["contents"],
                "symbol renaming changed original code or ownership")
        member = row.get("member")
        require(isinstance(member, str) and Path(member).name == member and member.endswith(".o") and member not in expected_members,
                "duplicate or unsafe object archive member")
        expected_members[member] = after
        unchanged = kind in {"noop", "options"} or (kind in {"edit", "repair"} and name != "limit")
        if unchanged:
            require(previous is not None and row.get("reused") is True, "unchanged object was not retained")
            old_row = previous["objects"][name]
            for key in ("native_id", "native_symbol", "rust_symbol", "native_object_serial", "original_sha256", "renamed_sha256", "member"):
                require(key in row and row[key] == old_row[key], "unchanged object's generation/identity changed: " + key)
        if kind == "edit" and name == "limit":
            require(previous and row.get("reused") is False
                    and row["native_id"] == previous["objects"][name]["native_id"]
                    and row["native_object_serial"] != previous["objects"][name]["native_object_serial"]
                    and row["original_sha256"] != previous["objects"][name]["original_sha256"],
                    "private edit did not replace only its actual native object")
    archive = outputs["link"].read_bytes()
    metadata = outputs["metadata"].read_bytes()
    members, offsets, symbols = retained.archive_members(archive)
    require(list(members) == ["lib.rmeta", "lib.rmeta-link", *sorted(expected_members)]
            and retained.rlib_link_members(members["lib.rmeta-link"]) == sorted(expected_members),
            "real archive inventory is incomplete, duplicated or out of order")
    embedded = retained.elf_section(members["lib.rmeta"], b".rmeta")
    if embed_metadata is False:
        split_metadata(metadata, embedded)
    else:
        require(embedded == metadata and metadata[:8] == b"rust\0\0\0\x0c",
                "default/explicit embedding did not retain the full genuine metadata")
    for member, data in expected_members.items():
        require(members[member] == data, "published archive substituted different native object bytes")
    for row in objects.values():
        require(symbols.get(row["rust_symbol"]) == offsets[row["member"]], "archive index omits an actual emitted symbol")
    dep_info(outputs["dep-info"].read_bytes(), outputs, source, source.parent.parent)
    if kind == "noop":
        require(previous is not None and archive == previous["archive"] and metadata == previous["metadata"], "no-op changed real artifacts")
    counts = record.get("counts", {})
    checked = source_rows(record.get("checked_definitions"), "checked definition")
    generated = source_rows(record.get("generated_definitions"), "generated definition")
    require(counts.get("checked_bodies") == len(checked) and counts.get("generated_bodies") == len(generated),
            "work observation counts differ from real definition observations")
    compiled = {name for name, row in objects.items() if row.get("reused") is False}
    reused = {name for name, row in objects.items() if row.get("reused") is True}
    require(compiled | reused == NAMES and not compiled & reused and counts.get("compiled_objects") == len(compiled)
            and counts.get("reused_objects") == len(reused), "native work observations contradict immutable object generations")
    if kind == "noop":
        require(not checked and not generated and not compiled and counts.get("metadata_encodes") == counts.get("archive_builds") == 0
                and record.get("artifact_reused") is True, "real no-op repeated native or artifact work")
    elif kind == "cold":
        require(set(checked) == set(generated) == compiled == NAMES and counts.get("metadata_encodes") == counts.get("archive_builds") == 1,
                "cold leaf did not perform its complete owned work")
    elif kind == "edit":
        require(set(checked) == set(generated) == compiled == {"limit"} and counts.get("metadata_encodes") == counts.get("archive_builds") == 1,
                "private edit repeated unrelated native work or skipped changed artifact work")
    elif kind == "repair":
        require(set(checked) <= {"limit"} and set(generated) <= {"limit"} and compiled <= {"limit"}, "repair lost unchanged retained definitions")
    return {"objects": objects, "archive": archive, "metadata": metadata, "counts": counts}


def validate_context(record, facts, source_bytes, owner, *, prepared=True):
    context = record.get("context", {})
    require(record.get("schema") == "lustr-public-scalar-library-v1" and record.get("owner_pid") == owner[0]
            and str(record.get("owner_start")) == owner[1], "library report changed the actual retained owner")
    expected = {"crate_name": facts["--crate-name"], "metadata": facts["codegen"]["metadata"],
                "extra_filename": facts["codegen"]["extra-filename"], "source_path": facts["source"],
                "target": TARGET, "edition": "2024", "emits": facts["--emit"].split(","),
                "check_cfg": facts["check_cfg"], "search_dirs": facts["search_dirs"],
                "sdk_receipt_sha256": retained.SDK_RECEIPT,
                "profile": {"panic_abort": True, "overflow_checks": True, "debug_assertions": True}}
    for key, value in expected.items():
        require(context.get(key) == value, "public request context changed or omitted " + key)
    require(record.get("library_options") == library_options(facts),
            "public request lost actual metadata/lint options outside its native session key")
    require(record.get("counts", {}).get("sdk_loads") == 1, "native owner reloaded immutable SDK facts")
    require((record.get("source_sha256") == sha(source_bytes) if prepared else record.get("source_sha256") in {None, sha(source_bytes)})
            and isinstance(record.get("native_context"), str), "report does not bind original source/native session context")
    return context


def cache_observation(record, live_serials=()):
    value = record.get("cache_history", {})
    require(value.get("policy") == {"inactive_entries": 2, "inactive_bytes": 16 * 1024 * 1024,
            "owner_start_bound": True, "scope": "each native-object, native-definition, and artifact cache"},
            "actual owner cache policy differs from its predeclared startup budget")
    native, artifacts = value.get("native", {}), value.get("artifacts", {})
    for counts, entries, size in ((native, "inactive_objects", "inactive_object_bytes"),
                                  (native, "inactive_definitions", "inactive_definition_bytes"),
                                  (artifacts, "inactive_entries", "inactive_bytes")):
        require(type(counts.get(entries)) is int and 0 <= counts[entries] <= 2
                and type(counts.get(size)) is int and 0 <= counts[size] <= 16 * 1024 * 1024,
                "retained inactive cache exceeds its real configured resource policy")
    for counts, key in ((native, "active_objects"), (native, "active_definitions"), (artifacts, "active_entries")):
        require(type(counts.get(key)) is int and counts[key] >= 0, "active cache observation is incomplete")
    evicted = native.get("evicted_object_serials")
    require(isinstance(evicted, list) and all(type(serial) is int for serial in evicted)
            and len(evicted) == len(set(evicted)) and not set(evicted).intersection(live_serials),
            "cache retirement discarded a live object or has invalid observations")
    require(native.get("retired_definition_ids") == [], "unchanged declaration inventory lost a live native definition")
    require(isinstance(artifacts.get("evicted_original_serials"), list)
            and isinstance(artifacts.get("evicted_renamed"), list), "artifact history retirement observations are absent")
    return value


class Commands:
    def __init__(self, root, report, save):
        self.root, self.report, self.save = root, report, save
        self.root.mkdir()

    def run(self, label, argv, cwd, env, expected=0, traced=False):
        number = len(self.report["commands"])
        stem = self.root / f"{number:03d}-{label}"
        trace = stem.with_suffix(".trace")
        argv = list(map(str, argv))
        actual = trace_command(trace) + argv if traced else argv
        record = {"label": label, "argv": argv, "actual_argv": actual, "cwd": str(cwd), "environment": env,
                  "role": "product" if traced else "independent-evaluator"}
        self.report["commands"].append(record)
        process = subprocess.Popen(actual, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        timed_out, incomplete = False, False
        stdout, stderr = b"", b""
        try:
            try:
                stdout, stderr = process.communicate(timeout=90)
            except subprocess.TimeoutExpired as error:
                timed_out = True
                stdout, stderr = error.output or b"", error.stderr or b""
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                try:
                    stdout, stderr = process.communicate(timeout=3)
                except subprocess.TimeoutExpired as error:
                    incomplete = True
                    stdout, stderr = error.output or stdout, error.stderr or stderr
        finally:
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            for stream in (process.stdout, process.stderr):
                stream.close()
            for name, data in (("stdout", stdout), ("stderr", stderr)):
                path = Path(str(stem) + "." + name)
                write_new(path, data)
                record[name] = str(path.relative_to(self.root.parent))
                record[name + "_sha256"] = sha(data)
            record.update(status=process.poll(), timed_out=timed_out, cleanup_incomplete=incomplete)
            if traced and trace.exists():
                record["trace"] = str(trace.relative_to(self.root.parent))
                record["trace_sha256"] = sha(trace.read_bytes())
            self.save()
        require(not timed_out and not incomplete and process.returncode == expected,
                f"{label}: expected status {expected}, got {process.returncode}; original streams retained")
        return stdout, stderr, trace


def trace_command(path):
    return [shutil.which("strace"), "-q", "-f", "-v", "-s", "65535",
            "-e", "trace=execve,execveat,chdir,fchdir,clone,clone3,fork,vfork,exit_group", "-o", str(path)]


def file_snapshot(outputs):
    result = {}
    for emit, path in outputs.items():
        if hasattr(path, "fileno"):
            # Cargo may unlink an intermediate before the next compiler call.
            # The open descriptor still observes the actual published inode,
            # so its old bytes remain independently checked after unlinking.
            stat = os.fstat(path.fileno())
            position = path.tell()
            path.seek(0)
            data = path.read()
            path.seek(position)
            name = str(path.name)
        else:
            stat, data, name = path.stat(), path.read_bytes(), str(path)
        result[emit] = {"path": name, "sha256": sha(data), "size": stat.st_size,
                        "inode": stat.st_ino, "mtime_ns": stat.st_mtime_ns}
    return result


def delivered_artifacts(artifact, outputs):
    require(artifact.get("target", {}).get("name") == CRATE and not artifact.get("executable"),
            "Cargo reported the wrong library target")
    paths = [Path(path) for path in artifact.get("filenames", [])]
    require(len(paths) == 2, "Cargo omitted or added a delivered library artifact")
    delivered = {}
    # Pinned nightly Cargo's uplift_to leaves rlibs in the intermediate output
    # directory unless its temporary test opt-out is set; this campaign's
    # explicit environment does not set it. Rmeta is never uplifted either.
    for emit, extension in (("link", ".rlib"), ("metadata", ".rmeta")):
        matches = [path for path in paths if path.suffix == extension]
        require(len(matches) == 1 and matches[0].resolve() == outputs[emit].resolve()
                and matches[0].read_bytes() == outputs[emit].read_bytes(),
                "Cargo's delivered filename differs from compiler output")
        delivered[emit] = matches[0]
    return delivered


def command_inventory(record, outputs, llvm):
    objects = source_rows(record.get("objects", []), "artifact object")
    members = sorted(row["member"] for row in objects.values())
    wanted, labels = [], set()
    for command in record.get("commands", []):
        label, argv, cwd = command.get("name"), command.get("argv"), command.get("cwd")
        require(isinstance(label, str) and label not in labels and command.get("status") == 0
                and isinstance(argv, list) and all(isinstance(value, str) for value in argv)
                and isinstance(cwd, str) and Path(cwd).is_absolute(), "artifact command is incomplete or repeated")
        labels.add(label)
        if label == "archive":
            require(len(argv) == 6 + len(members) and argv[0] == str(llvm / "llvm-ar")
                    and argv[1:3] == ["crsD", "--format=gnu"] and Path(argv[3]).name == argv[3]
                    and argv[3].endswith(".rlib") and argv[4:] == ["lib.rmeta", "lib.rmeta-link", *members],
                    "archive command does not construct the actual complete member inventory")
        else:
            matched = re.fullmatch(r"objcopy-([0-9a-f]{16})", label)
            require(matched is not None, "unknown artifact utility command")
            rows = [row for row in objects.values() if f'{row["native_id"][1]:016x}' == matched.group(1)]
            require(len(rows) == 1 and argv == [str(llvm / "llvm-objcopy"), "--redefine-syms=symbols.map",
                    "native_" + matched.group(1) + ".o", rows[0]["member"]],
                    "object utility command does not own its actual instance")
        wanted.append({"argv": argv, "cwd": cwd})
    require(sum(row.get("name") == "archive" for row in record.get("commands", []))
            == record.get("counts", {}).get("archive_builds", 0), "archive work observation differs from command inventory")
    return wanted


def reconcile_artifact_calls(actual, wanted):
    require([{"argv": row["argv"], "cwd": row["cwd"]} for row in actual] == wanted,
            "actual per-request artifact processes differ from complete tool command inventory")


def consumer(mask, operator):
    source = f"""fn main() {{
    let project: fn(u64)->u64 = {CRATE}::project;
    let merge: fn(u64,u64)->u64 = {CRATE}::merge;
    for value in (0u64..65536).chain([u64::MAX,1u64<<63]) {{
        assert_eq!({CRATE}::project(value), (value & {mask}) ^ 165);
        assert_eq!(project(value), (value & {mask}) ^ 165);
        assert_eq!({CRATE}::merge(value,0x101), value {operator} 0x101);
        assert_eq!(merge(value,0x101), value {operator} 0x101);
    }}
    println!("public-library-results={ASSERTIONS_PER_CONSUMER}");
}}
"""
    return source.encode()


def oracle_case(commands, sdk, env, case, attempt, outputs, mask, operation, report):
    rustc = sdk / "bin/rustc"
    root_text, _, _ = commands.run(case.name + "-original-reader", [rustc, "-Zls=root", outputs["metadata"]], case, env)
    actual_root = construction.parse_root(root_text)
    oracle = case / "original-rust"
    oracle.mkdir()
    argv = list(attempt["argv"][1:])
    at = argv.index("--out-dir")
    argv[at + 1] = str(oracle)
    _, oracle_stderr, _ = commands.run(case.name + "-original-producer", [rustc, *argv], attempt["cwd"], env)
    original_messages = [json.loads(line) for line in oracle_stderr.splitlines() if line]
    original_unused = unused_dependencies(original_messages)
    expected_oracle_outputs = {emit: oracle / path.name for emit, path in outputs.items()}
    notifications = {row.get("emit"): Path(row.get("artifact", "")) for row in original_messages if row.get("$message_type") == "artifact"}
    require(notifications == expected_oracle_outputs, "original Rust artifact protocol differs from fixed leaf contract")
    original_root_text, _, _ = commands.run(case.name + "-reference-reader", [rustc, "-Zls=root", expected_oracle_outputs["metadata"]], case, env)
    original_root = construction.parse_root(original_root_text)
    compare_lang_items(commands, rustc, env, case, outputs["metadata"], expected_oracle_outputs["metadata"])
    actual_members, _, _ = retained.archive_members(outputs["link"].read_bytes())
    reference_members, _, _ = retained.archive_members(expected_oracle_outputs["link"].read_bytes())
    full, reference_full = outputs["metadata"].read_bytes(), expected_oracle_outputs["metadata"].read_bytes()
    embedded = retained.elf_section(actual_members["lib.rmeta"], b".rmeta")
    reference_embedded = retained.elf_section(reference_members["lib.rmeta"], b".rmeta")
    options = library_options(parse_compile(attempt))
    if options["embed_metadata"] is False:
        metadata_evidence = {"layout": "split", **compare_split_metadata(full, embedded, reference_full, reference_embedded)}
    else:
        require(embedded == full and reference_embedded == reference_full,
                "default/explicit full embedding differs from each original full metadata payload")
        metadata_evidence = {"layout": "embedded", "full_sha256": sha(full), "embedded_sha256": sha(embedded),
                             "reference_full_sha256": sha(reference_full),
                             "reference_embedded_sha256": sha(reference_embedded)}
    # Native code and metadata hashes are expected to differ. Actual crate
    # identity, profile/ABI context and dependency identities must agree.
    for key in original_root:
        if key not in {"hash", "svh", "svh_le_hex"}:
            require(actual_root.get(key) == original_root[key], "genuine reader differs from original Rust context: " + key)
    dep_info(expected_oracle_outputs["dep-info"].read_bytes(), expected_oracle_outputs, Path(attempt["cwd"]) / parse_compile(attempt)["source"], attempt["cwd"])
    caller = case / "independent-caller.rs"
    program = case / "independent-caller"
    write_new(caller, consumer(mask, "^" if operation == "xor" else "|"))
    # Cargo1974-1983 supplies the linkable path and, only for split metadata,
    # the corresponding rmeta. The original locator resolves rmeta first.
    dependencies = ["--extern", CRATE + "=" + str(outputs["link"])]
    if options["embed_metadata"] is False:
        dependencies += ["--extern", CRATE + "=" + str(outputs["metadata"])]
    _, consumer_stderr, _ = commands.run(case.name + "-consumer-compile",
        [rustc, "--edition=2024", "-Cpanic=abort", "-Copt-level=0", "-Dwarnings",
         "--error-format=json", "--json=unused-externs-silent", "--force-warn=unused_crate_dependencies",
         caller, *dependencies, "-o", program], case, env)
    consumer_unused = unused_dependencies([json.loads(line) for line in consumer_stderr.splitlines() if line])
    stdout, stderr, _ = commands.run(case.name + "-consumer-run", [program], case, env)
    require(stdout == f"public-library-results={ASSERTIONS_PER_CONSUMER}\n".encode() and not stderr,
            "unchanged independent consumer returned incorrect results")
    report["checked_runtime_results"] += ASSERTIONS_PER_CONSUMER
    report["original_consumers"].append({"case": case.name, "assertions": ASSERTIONS_PER_CONSUMER,
        "crate_root": actual_root, "reference_crate_root": original_root, "source_sha256": sha(caller.read_bytes()),
        "metadata_artifacts": metadata_evidence, "original_producer_unused_dependencies": original_unused,
        "original_consumer_unused_dependencies": consumer_unused, "executable": file_identity(program)})
    return actual_root


def compare_lang_items(commands, rustc, env, case, native, original):
    actual, _, _ = commands.run(case.name + "-lang-items", [rustc, "-Zls=lang_items", native], case, env)
    expected, _, _ = commands.run(case.name + "-original-lang-items", [rustc, "-Zls=lang_items", original], case, env)
    require(b"eh_personality = <missing>" in expected and actual == expected,
            "native metadata lost the original missing language item record")


def project_fixture(root, name, package, source_name, crate_name=CRATE, *, explicit_strip=True):
    path = root / name
    source = path / source_name
    source.parent.mkdir(parents=True)
    manifest = path / "Cargo.toml"
    strip = 'strip = "none"\n' if explicit_strip else ""
    write_new(manifest, f'''[package]
name = "{package}"
version = "0.1.0"
edition = "2024"

[lib]
name = "{crate_name}"
path = "{source_name}"

[profile.dev]
panic = "abort"
debug = 0
opt-level = 0
debug-assertions = true
overflow-checks = true
incremental = false
{strip}''')
    return {"root": path, "manifest": manifest, "source": source, "target": path / "target",
            "build": path / "intermediates", "package": package}


def qualify_dependency_checks(root, package, sdk, commands, environment, oracle_env, projects, owner, report):
    consumer, provider = projects["consumer"], projects["provider"]
    base = b"#![no_std]\npub fn call(x: u64) -> u64 { provider::mask(x) }\n"
    edit = b"#![no_std]\npub fn call(x: u64) -> u64 { provider::mask(x & 7) }\n"
    leaf = b"#![no_std]\npub fn mask(x: u64) -> u64 { x & 31 }\n"
    repair = b"#![no_std]\npub fn call(x: u64) -> u64 { provider::mask(x & 7, 63) }\n"
    cases = [
        ("cold", base, leaf, 0, 2),
        ("noop", base, None, 0, 1),
        ("consumer-edit", edit, None, 0, 1),
        ("provider-body", None, leaf.replace(b"31", b"63"), 0, 2),
        ("provider-signature", None, b"#![no_std]\npub fn mask(x: u64, bits: u64) -> u64 { x & bits }\n", 101, 2),
        ("repair", repair, None, 0, 1),
        ("unused", b"#![no_std]\npub fn call(x: u64) -> u64 { x }\n", None, 0, 1),
    ]
    env = dict(environment, CARGO_TARGET_DIR=str(consumer["target"]), CARGO_BUILD_BUILD_DIR=str(consumer["build"]))
    independent = dict(oracle_env, CARGO_HOME=environment["CARGO_HOME"], CARGO_TARGET_DIR=str(consumer["root"] / "oracle-target"),
                       CARGO_BUILD_BUILD_DIR=str(consumer["root"] / "oracle-build"), RUSTC=str(sdk / "bin/rustc"))
    flags = ["check", "--lib", "--offline", "--message-format=json"]
    argv = [package / "bin/cargo", *flags]
    previous = None
    provider_svh = None
    result = {"passed": False, "history": []}
    report["dependency_checks"] = result
    for label, consumer_source, provider_source, status, request_count in cases:
        if consumer_source is not None:
            consumer["source"].write_bytes(consumer_source)
        if provider_source is not None:
            provider["source"].write_bytes(provider_source)
        before = set((root / "observations").glob("*/report.json"))
        _, _, trace = commands.run("dependency-" + label, argv, consumer["root"], env, expected=status, traced=True)
        events = trace_events(trace.read_bytes(), consumer["root"])
        compiled = inspect_processes(events, package, argv[0], argv, projects)["compiles"]
        require(len(compiled) == request_count, "dependency check did not issue the expected actual compiler requests")
        facts = [parse_compile(event) for event in compiled]
        by_name = {fact["--crate-name"]: fact for fact in facts}
        consumer_facts = by_name["consumer"]
        provider_path = Path(consumer_facts["--extern"].removeprefix("provider="))
        out = Path(consumer_facts["--out-dir"])
        suffix = consumer_facts["codegen"]["extra-filename"]
        metadata = out / f"libconsumer{suffix}.rmeta"
        require(consumer_facts["--extern"] == f"provider={provider_path}"
                and provider_path.is_relative_to(consumer["build"]) and provider_path.is_file()
                and out.is_relative_to(consumer["build"]), "consumer did not receive the actual provider metadata")
        records = [json.loads(path.read_text()) for path in set((root / "observations").glob("*/report.json")) - before]
        require(len(records) == request_count and set(by_name) == {r["context"]["crate_name"] for r in records},
                "Cargo metadata work did not reach the one native owner")
        for record in records:
            name = record["context"]["crate_name"]
            validate_context(record, by_name[name], projects[name]["source"].read_bytes(), owner)
            require(all(record["counts"][key] == 0 for key in ("compiled_objects", "generated_bodies", "archive_builds"))
                    and not record["commands"] and not record["objects"], "dependency check performed codegen")
        native = next(record for record in records if record["context"]["crate_name"] == "consumer")
        counts = native["counts"]
        require(counts["checked_bodies"] == (0 if label == "noop" else 1)
                and counts["metadata_encodes"] == (0 if label == "noop" or status else 1),
                "retained dependency check reused stale state or repeated unchanged work")
        commands.run("dependency-" + label + "-original", [sdk / "bin/cargo", *flags],
                     consumer["root"], independent, expected=status)
        if status:
            require(native["status"] == 1 and not native["outputs"]
                    and "L0105" in native["native_diagnostics"], "changed provider signature escaped native checking")
        else:
            require(native["status"] == 0, "successful dependency request did not publish metadata")
            original = next((consumer["root"] / "oracle-build").rglob(f"libconsumer{suffix}.rmeta"))
            roots = []
            for engine, artifact in (("native", metadata), ("original", original)):
                data, _, _ = commands.run("dependency-" + label + "-" + engine + "-reader",
                                            [sdk / "bin/rustc", "-Zls=root", artifact], consumer["root"], oracle_env)
                roots.append(construction.parse_root(data))
            expected = roots[1]["dependencies"]
            actual = roots[0]["dependencies"]
            require([dict(row, svh_le_hex=None) for row in actual] == [dict(row, svh_le_hex=None) for row in expected],
                    "consumer metadata changed the pinned dependency closure")
            if label == "unused":
                require([row["name"] for row in actual] == ["core", "compiler_builtins"], "unused provider entered consumer metadata")
            else:
                svh = next(row["svh_le_hex"] for row in actual if row["name"] == "provider")
                require(svh == provider_path.read_bytes()[24:40].hex(), "consumer metadata names a stale provider")
                if label == "provider-body":
                    require(svh != provider_svh, "provider body change did not invalidate consumer metadata")
                provider_svh = svh
            if label == "noop":
                require(metadata.read_bytes() == previous and native["artifact_reused"],
                        "warm check regenerated unchanged metadata bytes")
            previous = metadata.read_bytes()
            if label == "repair":
                caller = root / "cases" / "dependency-third.rs"
                write_new(caller, b"#![no_std]\npub fn third(x: u64) -> u64 { consumer::call(x) }\n")
                commands.run("dependency-original-third", [sdk / "bin/rustc", "--edition=2024", "--crate-type=lib",
                             "--emit=metadata", "-Cpanic=abort", "--extern", f"consumer={metadata}",
                             "-L", f"dependency={provider_path.parent}", caller, "-o", caller.with_suffix(".rmeta")],
                             consumer["root"], oracle_env)
        result["history"].append({"case": label, "status": status, "consumer_counts": counts})
    require(not list(consumer["build"].rglob("*.rlib")) and not list(consumer["build"].rglob("*.o")),
            "dependency metadata checks emitted link artifacts")
    result["passed"] = True


def qualify_dependency_links(root, package, sdk, commands, environment, oracle_env, projects, owner,
                             daemon_trace, daemon_argv, prefixes, report):
    consumer, provider = projects["consumer"], projects["provider"]
    build = consumer["root"] / "link-intermediates"
    env = dict(environment, CARGO_BUILD_BUILD_DIR=str(build), CARGO_TARGET_DIR=str(consumer["root"] / "link-target"))
    argv = [package / "bin/cargo", "build", "--lib", "--offline", "--message-format=json"]
    consumer_source = b"#![no_std]\npub fn call(x: u64) -> u64 { provider::mask(x) }\n"
    result = {"passed": False, "history": [], "runtime_assertions": 0}
    report["dependency_links"] = result
    preceding_archive = None
    provider_symbol = None
    for label, mask, expected_requests in (("cold", 31, 2), ("noop", 31, 1), ("provider-edit", 63, 2)):
        if label != "noop":
            provider["source"].write_text(f"#![no_std]\npub fn mask(x: u64) -> u64 {{ x & {mask} }}\n")
        if label != "provider-edit":
            consumer["source"].write_bytes(consumer_source)
        before = set((root / "observations").glob("*/report.json"))
        _, _, trace = commands.run("dependency-link-" + label, argv, consumer["root"], env, traced=True)
        events = trace_events(trace.read_bytes(), consumer["root"])
        compiled = inspect_processes(events, package, argv[0], argv, projects)["compiles"]
        require(len(compiled) == expected_requests, "Cargo link graph executed a different compiler inventory")
        facts = {fact["--crate-name"]: fact for fact in map(parse_compile, compiled)}
        consumer_facts = facts["consumer"]
        provider_meta = Path(consumer_facts["--extern"].removeprefix("provider="))
        consumer_out = Path(consumer_facts["--out-dir"])
        consumer_suffix = consumer_facts["codegen"]["extra-filename"]
        consumer_meta = consumer_out / f"libconsumer{consumer_suffix}.rmeta"
        consumer_archive = consumer_meta.with_suffix(".rlib")
        require(consumer_facts["--extern"] == f"provider={provider_meta}" and provider_meta.is_file()
                and provider_meta.is_relative_to(build) and consumer_out.is_relative_to(build),
                "Cargo did not supply the actual provider metadata in the link request")
        witnesses = set((root / "observations").glob("*/report.json")) - before
        records = sorted((json.loads(witness.read_text()) for witness in witnesses), key=lambda row: row["request_index"])
        require(len(records) == expected_requests and set(facts) == {row["context"]["crate_name"] for row in records},
                "Cargo link recording did not reach the same retained native owner")
        for row in records:
            name = row["context"]["crate_name"]
            validate_context(row, facts[name], projects[name]["source"].read_bytes(), owner)
            require(row["status"] == 0 and len(row["objects"]) == 1, "native link did not emit exactly its one source body")
        native = next(row for row in records if row["context"]["crate_name"] == "consumer")
        counts = native["counts"]
        expected_work = 0 if label == "noop" else 1
        require(all(counts[name] == expected_work for name in ("checked_bodies", "generated_bodies", "compiled_objects",
                                                               "metadata_encodes", "archive_builds")),
                "retained dependency link skipped changed work or regenerated unchanged work")
        wanted = [{"argv": command["argv"], "cwd": command["cwd"]} for row in records for command in row["commands"]]
        prefix = daemon_trace.read_bytes()
        frozen_prefix(prefixes[-1]["bytes"], prefix)
        owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        actual = owner_commands["artifacts"][prefixes[-1]["artifact_count"]:]
        reconcile_artifact_calls(actual, wanted)
        prefixes.append({"name": "dependency-link-" + label, "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
        for row in records:
            if row["context"]["crate_name"] == "provider":
                provider_symbol = row["objects"][0]["rust_symbol"]
        archive = consumer_archive.read_bytes()
        members, _, _ = retained.archive_members(archive)
        obj = retained.elf_object(members[native["objects"][0]["member"]])
        require(obj["undefined"] == obj["external_calls"] == {provider_symbol},
                "consumer object does not call the actual Rust symbol in provider")
        data, _, _ = commands.run("dependency-link-" + label + "-reader",
                                   [sdk / "bin/rustc", "-Zls=root", consumer_meta], consumer["root"], oracle_env)
        roots = construction.parse_root(data)
        require([row["name"] for row in roots["dependencies"]] == ["core", "provider", "compiler_builtins"]
                and roots["dependencies"][1]["svh_le_hex"] == provider_meta.read_bytes()[24:40].hex(),
                "linked consumer metadata did not bind the actual provider")
        if label == "noop":
            require(archive == preceding_archive and native["artifact_reused"], "warm link rebuilt an unchanged archive")
        else:
            case = root / "cases" / ("dependency-link-" + label)
            case.mkdir()
            caller = case / "caller.rs"
            program = case / "caller"
            write_new(caller, f"""fn main() {{
    let values = (0u64..=65_535).chain([1u64 << 63, (1u64 << 63) + 31, u64::MAX, u64::MAX - 32]);
    let mut checked = 0;
    for x in values {{
        assert_eq!(consumer::call(x), x & {mask});
        checked += 1;
    }}
    println!("cross-crate results={{checked}}");
}}
""")
            commands.run("dependency-link-" + label + "-runtime-link",
                         [sdk / "bin/rustc", "--edition=2024", "-Cpanic=abort", "-Dwarnings",
                          "--extern", f"consumer={consumer_meta}", "--extern", f"consumer={consumer_archive}",
                          "-L", f"dependency={provider_meta.parent}", caller, "-o", program], case, oracle_env)
            stdout, stderr, _ = commands.run("dependency-link-" + label + "-runtime", [program], case, oracle_env)
            require(stdout == b"cross-crate results=65540\n" and not stderr,
                    "actual linked scalar calls did not produce the expected results")
            result["runtime_assertions"] += 65_540
        preceding_archive = archive
        result["history"].append({"case": label, "consumer_counts": counts, "archive_sha256": sha(archive)})
    require(result["runtime_assertions"] == 131_080, "cross-crate runtime denominator changed")
    result["passed"] = True


def qualify_executable(root, package, sdk, commands, environment, oracle_env, projects, owner,
                       daemon_trace, daemon_argv, prefixes, report, *, check=False, module=False):
    consumer, provider = projects["consumer"], projects["provider"]
    consumer["binary_target"] = "consumer"
    source = consumer["root"] / "src/main.rs"
    mode, emit = ("check", "metadata") if check else ("run", "link")
    group = "cargo-executable-check" if check else "cargo-executable"
    if module:
        group += "-module"
        mode_directory = mode + "-module"
        body = source.with_name("computation.rs")
        body_prefix = b"pub fn execute() -> u64 { consumer::call("
        body_suffix = b") }\n"
        source.write_bytes(b'mod computation;\nfn main() { println!("{}", computation::execute()); }\n')
    else:
        mode_directory = mode
        body = source
        body_prefix = b'fn main() { println!("{}", consumer::call('
        body_suffix = b')); }\n'
    target = consumer["root"] / f"binary-{mode_directory}-target"
    build = consumer["root"] / f"binary-{mode_directory}-build"
    application = target / "debug/consumer"
    env = dict(environment, CARGO_TARGET_DIR=str(target), CARGO_BUILD_BUILD_DIR=str(build))
    independent = dict(oracle_env, CARGO_HOME=environment["CARGO_HOME"],
                       CARGO_TARGET_DIR=str(consumer["root"] / f"binary-{mode_directory}-oracle-target"),
                       CARGO_BUILD_BUILD_DIR=str(consumer["root"] / f"binary-{mode_directory}-oracle-build"), RUSTC=str(sdk / "bin/rustc"))
    flags = [mode, "--bin", "consumer", "--offline", "--message-format=json"]
    argv = [package / "bin/cargo", *flags]
    cases = [
        ("cold", b"255", 31, 31, 3, False),
        ("noop", b"255", None, 31, 1, True),
        ("edit", b"49", None, 17, 1, False),
        ("invalid", b"false", None, None, 1, None),
        ("repair", b"49", None, 17, 1, True),
        ("provider-edit", None, 63, 49, 3, False),
    ]
    result = {"passed": False, "history": []}
    report[group.replace("-", "_")] = result
    unit_directory, previous_units = None, {}
    for label, arg, mask, value, request_count, reused in cases:
        if arg is not None:
            body.write_bytes(body_prefix + arg + body_suffix)
        if mask is not None:
            provider["source"].write_text(f"#![no_std]\npub fn mask(x: u64) -> u64 {{ x & {mask} }}\n")
        status = 0 if value is not None else 101
        reused = False if check and not status else reused
        before = set((root / "observations").glob("*/report.json"))
        stdout, stderr, trace = commands.run(group + "-" + label, argv, consumer["root"], env, expected=status, traced=True)
        records = sorted(
            (json.loads(path.read_text()) for path in set((root / "observations").glob("*/report.json")) - before
             if path.parent.name.startswith("request-")), key=lambda row: row["request_index"])
        executable_reports = [json.loads(path.read_text()) for path in set((root / "observations").glob("*/report.json")) - before
                              if path.parent.name.startswith("executable-")]
        expected_application = None if check else application
        events = trace_events(trace.read_bytes(), consumer["root"], application=expected_application)
        observed = inspect_processes(events, package, argv[0], argv, projects, application=expected_application)
        require(len(observed["compiles"]) == request_count and len(records) == request_count - 1,
                "Cargo invoked a different native executable compilation graph")
        actual = parse_compile(observed["compiles"][-1], executable=True)
        require(actual["--crate-type"] == "bin" and actual["--emit"] == f"dep-info,{emit}"
                and actual["source"] == str(source) and len(actual["--extern"]) == (2 if check else 4),
                "normal Cargo executable request changed its artifact/source boundary")
        inputs = {row.split("=", 1)[0]: set() for row in actual["--extern"]}
        for row in actual["--extern"]:
            alias, artifact = row.split("=", 1)
            require(Path(artifact).is_relative_to(build) and Path(artifact).is_file(),
                    "executable request lost a real archive or metadata file")
            inputs[alias].add(Path(artifact).suffix)
        expected_inputs = {".rmeta"} if check else {".rlib", ".rmeta"}
        require(inputs == dict.fromkeys(("consumer", "provider"), expected_inputs), "actual Cargo executable dependencies differ")
        prefix = daemon_trace.read_bytes()
        frozen_prefix(prefixes[-1]["bytes"], prefix)
        owners = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        actual_tools = owners["artifacts"][prefixes[-1]["artifact_count"]:]
        wanted = [{"argv": command["argv"], "cwd": command["cwd"]} for row in records for command in row["commands"]]
        native_tools = [tool for tool in actual_tools if tool["filename"] == str(package / "lib/lust/tcc/bin/tcc")]
        require(not native_tools or (status == 0 and not reused and
                len({tool["argv"][-2] for tool in native_tools}) == len(native_tools)),
                "native object work ran for reused or rejected executable work, or repeated a unit")
        if check:
            require(not actual_tools and not wanted and not observed["runs"] and not list(build.rglob("*.o"))
                    and not list(build.rglob("*.rlib")) and not application.exists(),
                    "metadata-only executable check performed native emission or execution")
        else:
            require(label != "cold" or native_tools, "cold executable did not compile native object units")
        if native_tools:
            directories = {Path(tool["argv"][-2]).parent for tool in native_tools}
            require(len(directories) == 1, "executable objects came from several native sessions")
            unit_directory = next(iter(directories))
        units = {}
        if not check:
            require(unit_directory is not None, "no executable object directory was bound by the cold trace")
            for path in sorted(unit_directory.glob("u*.o")):
                observed_file = path.stat()
                units[path.name] = (observed_file.st_ino, observed_file.st_ctime_ns, sha(path.read_bytes()))
        changed_units = [name for name, generation in units.items() if previous_units.get(name) != generation]
        require(check or (units and not ((reused or status) and changed_units)),
                "reused or rejected executable changed native object generations")
        if not check and label == "edit":
            require(units.keys() == previous_units.keys() and 0 < len(changed_units) < len(units),
                    "executable body edit did not reuse unchanged native object generations")
        previous_units = units
        reconcile_artifact_calls([tool for tool in actual_tools if tool not in native_tools], wanted)
        prefixes.append({"name": group + "-" + label, "bytes": prefix, "artifact_count": len(owners["artifacts"])})
        original_stdout, _, _ = commands.run(group + "-" + label + "-original",
                                              [sdk / "bin/cargo", *flags], consumer["root"], independent, expected=status)
        if status:
            require(not executable_reports and not observed["runs"] and b"L0110" in stdout + stderr,
                    "an invalid executable ran or escaped the native checker")
        else:
            require(len(executable_reports) == 1, "missing unique retained executable observation")
            if not check:
                require(len(observed["runs"]) == 1 and stdout.splitlines()[-1] == original_stdout.splitlines()[-1] == str(value).encode(),
                        "native and original executables did not run exactly the matching computation")
            record = executable_reports[0]
            require(record["schema"] == "lustr-public-executable-v1" and record["owner_pid"] == owner[0]
                    and record["emit"] == emit and record["artifact_reused"] is reused and record["unused_extern_names"] == ["provider"],
                    "retained executable artifact state or actual unused-extern result changed")
            expected_bodies = 0 if label == "noop" else 1 + int(module and label in {"cold", "provider-edit"})
            require(record["checked_bodies"] == expected_bodies,
                    "executable did not retain the expected native check work")
            outputs = {kind: Path(filename) for kind, filename in zip(("dep-info", emit), record["outputs"])}
            dep_info(outputs["dep-info"].read_bytes(), outputs, source, consumer["root"], modules=[body] if module else [])
            if check:
                original_files = [Path(filename) for row in map(json.loads, original_stdout.splitlines())
                    if row["reason"] == "compiler-artifact" and row["target"]["kind"] == ["bin"]
                    for filename in row["filenames"]]
                require(len(original_files) == 1 and original_files[0].read_bytes() == outputs[emit].read_bytes() == b"",
                        "metadata-only binary output differs from the pinned compiler contract")
            else:
                require(outputs[emit].read_bytes() == application.read_bytes(),
                        "Cargo ran another executable than the native published artifact")
        result["history"].append({"case": label, "status": status, "runtime_value": None if check else value,
                                  "artifact_reused": reused, "tinycc_processes": len(native_tools),
                                  "changed_native_objects": changed_units})
    result["passed"] = True


def qualify_expansion_inputs(root, package, sdk, commands, environment, oracle_env, projects, owner,
                             daemon_trace, daemon_argv, prefixes, report, *, check=False):
    project = projects["consumer"]
    source = project["root"] / "src/main.rs"
    included = source.with_name("answer.txt")
    source.write_bytes(b'fn main() { println!("{}:{}:{}", consumer::call(255), include_str!("answer.txt"),'
                       b' env!("LUSTR_TEST_COMPILE_VALUE")); }\n')
    source_bytes = source.read_bytes()
    mode, emit = ("check", "metadata") if check else ("run", "link")
    name = "expansion-" + mode
    target, build = (project["root"] / f"{name}-{suffix}" for suffix in ("target", "build"))
    application = target / "debug/consumer"
    env = dict(environment, CARGO_TARGET_DIR=str(target), CARGO_BUILD_BUILD_DIR=str(build))
    independent = dict(oracle_env, CARGO_HOME=environment["CARGO_HOME"], RUSTC=str(sdk / "bin/rustc"),
                       CARGO_TARGET_DIR=str(project["root"] / f"{name}-oracle-target"),
                       CARGO_BUILD_BUILD_DIR=str(project["root"] / f"{name}-oracle-build"))
    flags = [mode, "--bin", "consumer", "--offline", "--message-format=json"]
    argv = [package / "bin/cargo", *flags]
    baseline_mask = projects["provider"]["source"].read_bytes()
    require(b"x & 63" in baseline_mask, "expansion fixture did not retain the preceding provider generation")
    result = {"passed": False, "history": []}
    report["cargo_executable_expansion_inputs" + ("_check" if check else "")] = result
    cases = [
        ("cold", b"blue", "first", b"63:blue:first"),
        ("include-edit", b"red", "first", b"63:red:first"),
        ("env-edit", b"red", "second=value", b"63:red:second=value"),
        ("missing-include", None, "second=value", None),
        ("repair", b"blue", "third", b"63:blue:third"),
    ]
    for label, content, setting, wanted_output in cases:
        key = "LUSTR_TEST_COMPILE_VALUE"
        if content is None:
            included.unlink()
        elif not included.exists() or included.read_bytes() != content:
            included.write_bytes(content)
        status = 0 if wanted_output is not None else 101
        before = set((root / "observations").glob("*/report.json"))
        stdout, stderr, trace = commands.run(name + "-" + label, argv, project["root"],
            dict(env, **{key: setting}), expected=status, traced=True)
        original, _, _ = commands.run(name + "-" + label + "-original", [sdk / "bin/cargo", *flags],
            project["root"], dict(independent, **{key: setting}), expected=status)
        records = [json.loads(path.read_text()) for path in sorted(set((root / "observations").glob("*/report.json")) - before)]
        libraries = sorted((item for item in records if item["schema"] == "lustr-public-scalar-library-v1"), key=lambda item: item["request_index"])
        binaries = [item for item in records if item["schema"] == "lustr-public-executable-v1"]
        expected_application = None if check else application
        events = trace_events(trace.read_bytes(), project["root"], application=expected_application)
        observed = inspect_processes(events, package, argv[0], argv, projects, application=expected_application)
        require(len(observed["compiles"]) == (3 if label == "cold" else 1)
                and source.read_bytes() == source_bytes, "an expansion input did not invalidate the unchanged root")
        prefix = daemon_trace.read_bytes()
        frozen_prefix(prefixes[-1]["bytes"], prefix)
        owned = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        calls = owned["artifacts"][prefixes[-1]["artifact_count"]:]
        wanted = [{"argv": command["argv"], "cwd": command["cwd"]} for item in libraries for command in item["commands"]]
        reconcile_artifact_calls([call for call in calls if Path(call["filename"]).name != "tcc"], wanted)
        prefixes.append({"name": name + "-" + label, "bytes": prefix, "artifact_count": len(owned["artifacts"])})
        if check:
            require(not calls and not observed["runs"] and not application.exists(), "checking expansion inputs emitted code")
        if status:
            require(not binaries and not observed["runs"] and not calls and b"answer.txt" in stdout + stderr,
                    "missing include emitted or ran code")
        else:
            require(len(binaries) == 1, "missing retained expansion input observation")
            if not check:
                require(len(observed["runs"]) == 1 and stdout.splitlines()[-1] == original.splitlines()[-1] == wanted_output,
                        "compiled expansion values did not match pinned Rust")
            record = binaries[0]
            require(record["owner_pid"] == owner[0] and record["emit"] == emit and not record["artifact_reused"],
                    "included file escaped retained checking")
            outputs = {kind: Path(filename) for kind, filename in zip(("dep-info", emit), record["outputs"])}
            dep_info(outputs["dep-info"].read_bytes(), outputs, source, project["root"],
                     modules=[included], environment=[f"{key}={setting}"])
        result["history"].append({"case": label, "status": status,
                                  "runtime_value": wanted_output.decode() if wanted_output is not None and not check else None})
    result["passed"] = True


def qualify_executable_extern_uses(root, package, sdk, commands, environment, oracle_env, projects,
                                   owner, daemon_trace, daemon_argv, prefixes, report):
    project = projects["consumer"]
    source = project["root"] / "src/main.rs"
    target = project["root"] / "extern-use-target"
    build = project["root"] / "extern-use-build"
    application = target / "debug/consumer"
    env = dict(environment, CARGO_TARGET_DIR=str(target), CARGO_BUILD_BUILD_DIR=str(build))
    argv = [package / "bin/cargo", "run", "--bin", "consumer", "--offline", "--message-format=json"]
    result = {"passed": False, "history": []}
    report["cargo_executable_extern_uses"] = result
    for label, declaration, unused in [
        ("direct-import", b"use provider::mask;", []),
        ("remove-import", b"", ["provider"]),
        ("glob-import", b"use provider::*;", []),
        ("absolute-import", b"use ::provider::mask;", []),
        ("extern-item", b"extern crate provider as retained_provider;", []),
        ("local-shadow", b"mod provider { pub fn mask(x: u64) -> u64 { x } } use provider::mask;", ["provider"]),
    ]:
        source.write_bytes(declaration + b'\nuse consumer::call;\nfn main() { println!("{}", call(255)); }\n')
        before = set((root / "observations").glob("*/report.json"))
        stdout, _, trace = commands.run("extern-use-" + label, argv, project["root"], env, traced=True)
        writes = [json.loads(path.read_bytes()) for path in set((root / "observations").glob("*/report.json")) - before]
        observed = inspect_processes(trace_events(trace.read_bytes(), project["root"], application=application),
            package, argv[0], argv, projects, application=application)
        require(len(observed["compiles"]) == (3 if label == "direct-import" else 1)
                and len(observed["runs"]) == 1 and stdout.splitlines()[-1] == b"63",
                "module import fixture changed its normal Cargo graph or runtime value")
        reports = [record for record in writes if record["schema"] == "lustr-public-executable-v1"]
        require(len(reports) == 1 and reports[0]["owner_pid"] == owner[0]
                and reports[0]["unused_extern_names"] == unused, "resolved module import did not update the extern-use lint")
        prefix = daemon_trace.read_bytes()
        frozen_prefix(prefixes[-1]["bytes"], prefix)
        owned = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        library_reports = sorted((record for record in writes if record["schema"] == "lustr-public-scalar-library-v1"),
                                 key=lambda record: record["request_index"])
        wanted = [{"argv": item["argv"], "cwd": item["cwd"]} for record in library_reports for item in record["commands"]]
        reconcile_artifact_calls([item for item in owned["artifacts"][prefixes[-1]["artifact_count"]:]
                                  if Path(item["filename"]).name != "tcc"], wanted)
        prefixes.append({"name": "extern-use-" + label, "bytes": prefix, "artifact_count": len(owned["artifacts"])})
        oracle_out = root / "cases" / ("extern-use-oracle-" + label)
        oracle_out.mkdir()
        actual = observed["compiles"][-1]["argv"]
        args = list(actual[1:])
        args[args.index("--emit=dep-info,link")] = "--emit=dep-info,metadata"
        args[args.index("--out-dir") + 1] = str(oracle_out)
        _, stderr, _ = commands.run("extern-use-" + label + "-original", [sdk / "bin/rustc", *args], project["root"], oracle_env)
        unused_records = [row["unused_extern_names"] for row in map(json.loads, stderr.splitlines())
                          if row.get("$message_type") == "unused_extern"]
        require(unused_records == [unused], "native extern uses differ from the pinned Rust resolver")
        result["history"].append({"case": label, "unused_extern_names": unused})
    result["passed"] = True


def qualify_integer_signatures(root, package, sdk, commands, environment, oracle_env, projects,
                               daemon_argv, daemon_trace, prefixes, owner, report):
    crate = "typed_leaves"
    target_name = "typed-leaves"
    project = project_fixture(root / "projects", "integers", target_name, "src/lib.rs", crate, explicit_strip=False)
    projects["integers"] = project
    project["binary_target"] = target_name
    producer = b"""#![no_std]
pub fn signed_narrow(x: i32) -> i32 { x ^ -7 }
pub fn signed_wide(x: i64) -> i64 { x ^ -7 }
pub fn signed_word(x: isize) -> isize { x ^ -7 }
pub fn unsigned_narrow(x: u32) -> u32 { x ^ 7 }
pub fn unsigned_wide(x: u64) -> u64 { x ^ 7 }
pub fn unsigned_word(x: usize) -> usize { x ^ 7 }
"""
    write_new(project["source"], producer)
    source = project["root"] / "src/main.rs"
    write_new(source, b"""fn main() {
    assert_eq!(typed_leaves::signed_narrow(0), -7);
    assert_eq!(typed_leaves::signed_wide(0), -7);
    assert_eq!(typed_leaves::signed_word(0), -7);
    println!("{}:{}:{}:{}:{}:{}",
        typed_leaves::signed_narrow(i32::MIN),
        typed_leaves::signed_wide(i64::MIN),
        typed_leaves::signed_word(isize::MIN),
        typed_leaves::unsigned_narrow(u32::MAX),
        typed_leaves::unsigned_wide(u64::MAX),
        typed_leaves::unsigned_word(usize::MAX));
}
""")
    build, target = project["build"], project["target"]
    application = target / "debug" / target_name
    env = dict(environment, CARGO_BUILD_BUILD_DIR=str(build), CARGO_TARGET_DIR=str(target))
    before = set((root / "observations").glob("*/report.json"))
    argv = [package / "bin/cargo", "run", "--bin", target_name, "--offline", "--message-format=json"]
    stdout, _, trace = commands.run("integer-signatures-native", argv, project["root"], env, traced=True)
    observed = inspect_processes(trace_events(trace.read_bytes(), project["root"], application=application),
        package, argv[0], argv, projects, application=application)
    require(len(observed["compiles"]) == 2 and len(observed["runs"]) == 1, "integer executable did not use one library and one binary")
    library = parse_compile(observed["compiles"][0])
    binary = parse_compile(observed["compiles"][-1], executable=True)
    require(library["codegen"].get("strip") == "debuginfo", "ordinary Cargo debug-strip request was not exercised")
    require(binary["--crate-name"] == crate and observed["compiles"][-1]["environment"].get("CARGO_BIN_NAME") == target_name,
            "Cargo target name and Rust crate identity were not exercised separately")
    records = [json.loads(path.read_bytes()) for path in set((root / "observations").glob("*/report.json")) - before]
    libraries = [record for record in records if record["schema"] == "lustr-public-scalar-library-v1"]
    require(len(libraries) == 1 and libraries[0]["owner_pid"] == owner[0], "integer library escaped native retention")
    record = libraries[0]
    outputs = {row["emit"]: Path(row["path"]) for row in record["outputs"]}
    prefix = daemon_trace.read_bytes()
    frozen_prefix(prefixes[-1]["bytes"], prefix)
    owned = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
    wanted = [{"argv": item["argv"], "cwd": item["cwd"]} for item in record["commands"]]
    reconcile_artifact_calls([item for item in owned["artifacts"][prefixes[-1]["artifact_count"]:]
                              if Path(item["filename"]).name != "tcc"], wanted)
    prefixes.append({"name": "integer-signatures-native", "bytes": prefix, "artifact_count": len(owned["artifacts"])})
    independent = root / "cases" / "integer-rust-consumer"
    independent.mkdir()
    proof = independent / "program"
    commands.run("integer-original-reader-link", [sdk / "bin/rustc", "--edition=2024", "-Cpanic=abort", source, "-o", proof,
        "--extern", f"{crate}={outputs['link']}", "--extern", f"{crate}={outputs['metadata']}"], project["root"], oracle_env)
    original, _, _ = commands.run("integer-original-reader-run", [proof], independent, oracle_env)
    expected = b"2147483641:9223372036854775801:9223372036854775801:4294967288:18446744073709551608:18446744073709551608"
    require(stdout.splitlines()[-1] == original.rstrip() == expected, "integer signedness/width was not preserved across the Rust ABI")
    sections, _, _ = commands.run("integer-strip-sections", [Path(shutil.which("readelf")), "-SW", application, outputs["link"]],
                                 independent, oracle_env)
    require(b".debug_" not in sections and b".zdebug_" not in sections, "stripped profile left debug sections in an artifact")
    report["cargo_integer_signatures"] = {"passed": True, "runtime_value": expected.decode(), "type_count": 6}


def qualify_default_executable_check(root, package, sdk, commands, environment, oracle_env, projects,
                                     daemon_argv, daemon_trace, prefixes, owner, report):
    crate = "default_check"
    target_name = "default-check"
    project_root = root / "projects" / "default-check"
    (project_root / "src").mkdir(parents=True)
    manifest, library = project_root / "Cargo.toml", project_root / "src/lib.rs"
    source = project_root / "src/main.rs"
    write_new(manifest, f'[package]\nname="{target_name}"\nversion="0.1.0"\nedition="2024"\n')
    extra_integer_types = [(b"i8", b"-7"), (b"u8", b"7"), (b"i16", b"-7"), (b"u16", b"7"),
                           (b"i128", b"-7"), (b"u128", b"7")]
    definitions = [b"pub fn mask(value: u32) -> u32 { value & 255 }"]
    definitions.extend(b"pub fn mask_%s(value: %s) -> %s { value ^ %s }" % (ty, ty, ty, mask)
                       for ty, mask in extra_integer_types)
    write_new(library, b"#![no_std]\n" + b"\n".join(definitions) + b"\n")
    checks = b" ".join(b"let _: %s = default_check::mask_%s(42%s);" % (ty, ty, ty)
                       for ty, _ in extra_integer_types)
    projects[crate] = {"manifest": manifest, "binary_target": target_name}
    target, build = project_root / "target", project_root / "build"
    env = dict(environment, CARGO_TARGET_DIR=str(target), CARGO_BUILD_BUILD_DIR=str(build))
    original_env = dict(oracle_env, RUSTC=str(sdk / "bin/rustc"), CARGO_HOME=environment["CARGO_HOME"],
                        CARGO_TARGET_DIR=str(project_root / "original-target"),
                        CARGO_BUILD_BUILD_DIR=str(project_root / "original-build"))
    env.pop("CARGO_INCREMENTAL", None)
    original_env.pop("CARGO_INCREMENTAL", None)
    flags = ["check", "--offline", "--message-format=json"]
    argv = [package / "bin/cargo", *flags]
    result = {"passed": False, "history": []}
    report["cargo_default_executable_check"] = result
    for label, argument, status in [("cold", b"42", 0), ("noop", b"42", 0), ("invalid", b"false", 101), ("repair", b"42", 0)]:
        source.write_bytes(b'fn main() { ' + checks + b' println!("{}", default_check::mask(' + argument + b')); }\n')
        before = set((root / "observations").glob("*/report.json"))
        stdout, stderr, trace = commands.run("default-check-" + label, argv, project_root, env, expected=status, traced=True)
        original, _, _ = commands.run("default-check-" + label + "-original",
            [sdk / "bin/cargo", *flags], project_root, original_env, expected=status)
        observed = inspect_processes(trace_events(trace.read_bytes(), project_root), package, argv[0], argv, projects)
        require(len(observed["compiles"]) == (2 if label == "cold" else 1) and not observed["runs"],
                "default Cargo check did not check its original library and executable graph")
        for attempt in observed["compiles"]:
            invocation = parse_compile(attempt)
            codegen = invocation["codegen"]
            require(invocation["--emit"] == "dep-info,metadata" and codegen.get("debuginfo") == "2"
                    and "panic" not in codegen and Path(codegen["incremental"]).is_relative_to(build),
                    "test did not exercise the actual default Cargo profile")
        prefix = daemon_trace.read_bytes()
        frozen_prefix(prefixes[-1]["bytes"], prefix)
        owned = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        require(len(owned["artifacts"]) == prefixes[-1]["artifact_count"] and not list(build.rglob("*.rlib"))
                and not (target / "debug" / target_name).exists(), "default Cargo check emitted link artifacts")
        prefixes.append({"name": "default-check-" + label, "bytes": prefix, "artifact_count": len(owned["artifacts"])})
        records = [json.loads(path.read_bytes()) for path in set((root / "observations").glob("*/report.json")) - before]
        binaries = [record for record in records if record["schema"] == "lustr-public-executable-v1"]
        if status:
            require(not binaries and b"L0110" in stdout + stderr, "default check lost the native type error")
        else:
            require(len(binaries) == 1 and binaries[0]["owner_pid"] == owner[0]
                    and binaries[0]["checked_bodies"] == (0 if label == "noop" else 1),
                    "default check did not keep the native body state")
        if label == "cold":
            for output, filename in [(stdout, "native"), (original, "original")]:
                paths = [Path(path) for row in map(json.loads, output.splitlines())
                    if row["reason"] == "compiler-artifact" and row["target"]["kind"] == ["lib"]
                    for path in row["filenames"] if path.endswith(".rmeta")]
                require(len(paths) == 1, "default check lost the actual library metadata output")
                root_text, _, _ = commands.run("default-check-panic-" + filename,
                    [sdk / "bin/rustc", "-Zls=root", paths[0]], project_root, oracle_env)
                require(b"required_panic_strategy None panic_in_drop_strategy Unwind" in root_text,
                        "metadata encoded a different panic profile")
                commands.run("default-check-original-consumer-" + filename,
                    [sdk / "bin/rustc", "--edition=2024", source, "--emit=metadata",
                     "--extern", f"{crate}={paths[0]}", "-o", project_root / f"independent-{filename}.rmeta"],
                    project_root, oracle_env)
        result["history"].append({"case": label, "status": status})
    result["passed"] = True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--sdk", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    package, sdk = args.package.resolve(strict=True), args.sdk.resolve(strict=True)
    root = args.output.absolute()
    root.mkdir(parents=True, exist_ok=False)
    report = {"schema": "lustr-public-native-library-campaign-v1", "passed": False,
              "functional_passed": False, "exec_trace_qualified": False, "retained_public_integration": False,
              "full_compiler_qualification": False, "full_rust_coverage": False, "performance_qualified": False,
              "socket_independent": False, "commands": [], "observed_history": [], "original_consumers": [],
              "refusals": [], "checked_runtime_results": 0,
              "native_work_evidence": "source-bound work instrumentation corroborated by immutable object generations; not an independent TinyCC profiler",
              "unqualified_scope": ["general --extern consumer ingestion",
                    "non-scalar declarations", "other profiles/targets", "full Cargo graphs", "no_std compiler", "socket-free retained host"]}
    save = lambda: json_write(root / "report.json", report)
    commands = Commands(root / "commands", report, save)
    daemon = None
    daemon_handles = []
    delivered_handles = []
    owner = None
    prefixes = []
    sealed = {}
    sealed_directories = {}
    try:
        require(sys.platform == "linux" and shutil.which("strace") and shutil.which("sha256sum"),
                "public retained evidence requires Linux strace and SHA-256 tools")
        pins = dict(re.findall(r'^([a-z0-9_]+) = "([^\n]+)"$', (ROOT / "upstream/bluefe-core.pin").read_text(), re.M))
        require(file_identity(sdk / "bin/rustc")["sha256"] == pins["rustc_sha256"]
                and file_identity(sdk / "bin/cargo")["sha256"] == pins["cargo_sha256"]
                and file_identity(package / "bin/cargo")["sha256"] == pins["cargo_sha256"], "compiler/Cargo reference identity differs from source pin")
        require((package / "bin/rustc").resolve() == (package / "bin/lustr").resolve(), "installed RUSTC is not the public Lustr entry")
        receipt = package / "lib/lustr-metadata/sdk-receipt.json"
        require(sha(receipt.read_bytes()) == retained.SDK_RECEIPT, "installed SDK facts differ from original construction")
        sdk_receipt = json.loads(receipt.read_text())
        artifacts = [row[field]["path"] for row in sdk_receipt["dependencies"] for field in ("artifact", "link_artifact")]
        llvm = package / "lib/rustlib" / TARGET / "bin"
        for relative in artifacts + [f"lib/rustlib/{TARGET}/bin/{name}" for name in ("llvm-ar", "llvm-objcopy")]:
            require(file_identity(package / relative)["sha256"] == file_identity(sdk / relative)["sha256"],
                    "installed selective SDK artifact/tool differs from original bytes: " + relative)
        tool_paths = [package / "bin" / name for name in ("lustr", "rustc", "lust", "lust-core", "cargo")]
        tool_paths += [llvm / name for name in ("llvm-ar", "llvm-objcopy")]
        tool_paths += [package / "lib/lust/tcc/bin/tcc", sdk / "bin/rustc", Path(__file__), receipt]
        report["tools"] = [file_identity(path) for path in tool_paths]
        source_identity, _, _ = commands.run("source-identity", ["git", "rev-parse", "HEAD", "HEAD^{tree}"], ROOT,
                                             {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"})
        report["source_identity"] = source_identity.decode().splitlines()
        report["evaluator_sources"] = {str(path.relative_to(ROOT)): sha(path.read_bytes()) for path in
            (Path(__file__), ROOT / "tools/test-retained-native-metadata.py", ROOT / "tools/test-native-consumer-metadata.py",
             ROOT / "tools/build-metadata-sdk-receipt.py")}
        projects_root = root / "projects"
        projects_root.mkdir()
        projects = {
            "first": project_fixture(projects_root, "first", "public-leaf-fixture", "implementation/leaf_entry.rs"),
            "second": project_fixture(projects_root, "second", "another-leaf-package", "sources/different_entry.rs"),
            "check": project_fixture(projects_root, "check", "public-leaf-check", "src/lib.rs"),
        }
        projects["consumer"] = project_fixture(projects_root, "consumer", "consumer", "src/lib.rs", "consumer")
        projects["provider"] = project_fixture(projects["consumer"]["root"], "provider", "provider", "src/lib.rs", "provider")
        with projects["consumer"]["manifest"].open("a") as manifest:
            manifest.write('\n[dependencies]\nprovider = { path = "provider" }\n')
        for project in projects.values():
            project["source"].write_bytes(BASE)
        for name in ("home", "cargo-home", "tmp", "native-cache", "observations", "traps", "cases"):
            (root / name).mkdir()
        trap_log = root / "forbidden-compilers.jsonl"
        trap = (f"#!{sys.executable}\nimport json,os,sys\n"
                "with open(os.environ['LUSTR_FORBIDDEN_LOG'],'a') as f: f.write(json.dumps({'argv':sys.argv})+'\\n')\n"
                "sys.exit(86)\n")
        for name in ("rustc", "rustc-full", "rustc-reference", "rustc-oracle", "lustr-semantic", "lustr-mixed", "clang", "gcc", "cc", "tcc"):
            path = root / "traps" / name
            write_new(path, trap)
            path.chmod(0o755)
        endpoint = root / "daemon.sock"
        require(len(os.fsencode(endpoint)) <= 107, "qualification output path exceeds the Linux socket address limit")
        environment = {"PATH": str(root / "traps") + ":/usr/bin:/bin", "HOME": str(root / "home"),
            "CARGO_HOME": str(root / "cargo-home"), "CARGO_NET_OFFLINE": "true", "CARGO_BUILD_JOBS": "1",
            "CARGO_INCREMENTAL": "0", "CARGO_TERM_COLOR": "never", "TMPDIR": str(root / "tmp"),
            "LANG": "C", "LC_ALL": "C", "RUST_BACKTRACE": "0", "RUST_MIN_STACK": "268435456",
            "LUST_CACHE_DIR": str(root / "native-cache"), "LUST_DAEMON_SOCKET": str(endpoint),
            "LUST_DAEMON_IDLE": "300", "RUSTC": str(package / "bin/rustc"),
            "LUSTR_LIBRARY_REPORT_DIR": str(root / "observations"), "LUSTR_FORBIDDEN_LOG": str(trap_log),
            "LUSTR_LIBRARY_HISTORY_ENTRIES": "2"}
        oracle_env = {"PATH": "/usr/bin:/bin", "HOME": str(root / "home"), "LANG": "C", "LC_ALL": "C",
                      "LD_LIBRARY_PATH": str(sdk / "lib"), "RUST_BACKTRACE": "0"}
        report["product_environment"] = environment
        control_log = root / "trap-control.jsonl"
        commands.run("compiler-trap-control", [root / "traps/rustc", "--qualification-control"], root,
                     dict(environment, LUSTR_FORBIDDEN_LOG=str(control_log)), expected=86)
        control = [json.loads(line) for line in control_log.read_text().splitlines()]
        require(len(control) == 1 and control[0]["argv"] == [str(root / "traps/rustc"), "--qualification-control"],
                "independent compiler trap control failed")
        report["compiler_trap_control"] = {"status": 86, "sha256": sha(control_log.read_bytes())}
        daemon_trace = root / "daemon.trace"
        daemon_argv = [str(package / "bin/lust-core"), "daemon"]
        daemon_handles = [(root / "daemon.stdout").open("wb"), (root / "daemon.stderr").open("wb")]
        daemon = subprocess.Popen(trace_command(daemon_trace) + daemon_argv, cwd=root, env=environment,
                                  stdout=daemon_handles[0], stderr=daemon_handles[1], start_new_session=True)
        deadline = time.monotonic() + 15
        while daemon.poll() is None and time.monotonic() < deadline:
            children = Path(f"/proc/{daemon.pid}/task/{daemon.pid}/children").read_text().split()
            identities = [identity for pid in children if (identity := process_identity(int(pid), package / "bin/lust-core"))]
            require(len(identities) <= 1, "tracer owns more than one candidate daemon")
            if identities and endpoint.exists():
                owner = identities[0]
                break
            time.sleep(0.02)
        require(owner is not None and daemon.poll() is None, "actual retained public owner failed to start")
        report["owner_identity"] = owner
        owner_environment = Path(f"/proc/{owner[0]}/environ").read_bytes().split(b"\0")
        require(b"LUSTR_LIBRARY_HISTORY_ENTRIES=2" in owner_environment, "actual retained owner did not receive its startup cache budget")
        report["owner_history_budget"] = {"entries": 2, "observed_in_actual_owner_environment": True}
        first_prefix = daemon_trace.read_bytes()
        parsed = trace_events(first_prefix, root)
        require(parsed["root_pid"] == str(owner[0]), "traced initial process differs from observed owner")
        inspect_processes(parsed, package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        prefixes.append({"name": "ready", "bytes": first_prefix, "artifact_count": 0})
        previous, contexts, crate_roots, saved_outputs, saved_delivered = {}, {}, {}, {}, {}

        def seal_reports():
            for relative, files in sealed_directories.items():
                directory = root / "observations" / relative
                require(sorted(str(path.relative_to(directory)) for path in directory.rglob("*") if path.is_file()) == files,
                        "completed request witness inventory changed")
            for marker in (root / "observations").glob("*/report.json"):
                directory = marker.parent
                sealed_directories.setdefault(str(directory.relative_to(root / "observations")),
                    sorted(str(path.relative_to(directory)) for path in directory.rglob("*") if path.is_file()))
            for relative, expected in sealed.items():
                require(sha((root / "observations" / relative).read_bytes()) == expected, "completed request evidence was rewritten")
            for path in (root / "observations").rglob("*"):
                if path.is_file():
                    require(not path.is_symlink(), "request witness is a symlink")
                    sealed.setdefault(str(path.relative_to(root / "observations")), sha(path.read_bytes()))

        for label, unit, source_bytes, mask, operation, kind in HISTORY:
            project = projects[unit]
            case = root / "cases" / label
            case.mkdir()
            # Writing identical bytes deliberately makes Cargo re-invoke RUSTC.
            # A Fresh line, cached diagnostic or missing report cannot pass.
            project["source"].write_bytes(source_bytes)
            write_new(case / "source.rs", source_bytes)
            write_new(case / "Cargo.toml", project["manifest"].read_bytes())
            env = dict(environment, CARGO_TARGET_DIR=str(project["target"]), CARGO_BUILD_BUILD_DIR=str(project["build"]))
            before_reports = set((root / "observations").glob("*/report.json"))
            old_files = file_snapshot({emit: path for emit, path in saved_outputs[unit].items() if path.is_file()}) if unit in saved_outputs else None
            old_delivered = file_snapshot(saved_delivered[unit]) if unit in saved_delivered else None
            cargo_argv = [package / "bin/cargo", "build", "--lib", "--offline", "--message-format=json"]
            stdout, stderr, client_trace = commands.run(label, cargo_argv, project["root"], env,
                                                       expected=101 if kind == "invalid" else 0, traced=True)
            if old_delivered is not None:
                require(file_snapshot(saved_delivered[unit]) == old_delivered,
                        "public request mutated a prior delivered artifact inode")
            require(process_identity(owner[0], package / "bin/lust-core") == owner, "public request replaced/lost retained owner")
            require(project["source"].read_bytes() == source_bytes and project["manifest"].read_bytes() == (case / "Cargo.toml").read_bytes(),
                    "product rewrote the selected source or manifest")
            parsed_client = trace_events(client_trace.read_bytes(), project["root"])
            client = inspect_processes(parsed_client, package, package / "bin/cargo", cargo_argv, projects)
            require(parsed_client["exits"].get(parsed_client["root_pid"]) == (101 if kind == "invalid" else 0),
                    "actual Cargo process exit differs from recorded command")
            require(len(client["compiles"]) == 1, "Cargo did not invoke exactly one real public leaf compilation")
            invocation = client["compiles"][0]
            require(invocation["filename"] == str(package / "bin/rustc") and invocation["environment"].get("CARGO_MANIFEST_DIR") == str(project["root"])
                    and invocation["environment"].get("CARGO_PKG_NAME") == project["package"], "Cargo request environment lost actual package facts")
            facts = parse_compile(invocation)
            outputs = declared_compile(facts, project)
            new_reports = set((root / "observations").glob("*/report.json")) - before_reports
            require(len(new_reports) == 1, "public compilation lacks exactly one completed native request record")
            report_path = next(iter(new_reports))
            record = json.loads(report_path.read_text())
            require(record.get("request_index") == len(report["observed_history"]) + 1,
                    "retained owner repeated, skipped or reset its request sequence")
            context = validate_context(record, facts, source_bytes, owner)
            require(context.get("cwd") == str(project["root"]), "retained request lost caller's actual working directory")
            if unit in contexts:
                require(record["native_context"] == contexts[unit], "body-only request changed its retained session identity")
            else:
                require(record["native_context"] not in contexts.values(), "different source/unit aliased another native session")
                contexts[unit] = record["native_context"]
            prefix = daemon_trace.read_bytes()
            frozen_prefix(prefixes[-1]["bytes"], prefix)
            daemon_parsed = trace_events(prefix, root)
            daemon_commands = inspect_processes(daemon_parsed, package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
            old_count = prefixes[-1]["artifact_count"]
            actual_tools = daemon_commands["artifacts"][old_count:]
            wanted_tools = command_inventory(record, outputs, llvm)
            reconcile_artifact_calls(actual_tools, wanted_tools)
            if kind in {"noop", "invalid"}:
                require(not actual_tools and not wanted_tools, "no-op/rejection performed artifact subprocess work")
            prefixes.append({"name": label, "bytes": prefix, "artifact_count": len(daemon_commands["artifacts"])})
            mappings = Path(f"/proc/{owner[0]}/maps").read_text()
            write_new(case / "owner-maps.txt", mappings)
            require(not any(name in mappings for name in ("librustc_driver", "rustc_codegen_llvm", "lustr_semantic")),
                    "retained product loaded a competing Rust compiler library")
            messages = [json.loads(line) for line in stdout.splitlines() if line]
            cargo_artifacts = [row for row in messages if row.get("reason") == "compiler-artifact"]
            native_stderr = witness(report_path.parent, "native.stderr")
            native_messages = [json.loads(line) for line in native_stderr.splitlines() if line]
            unused_dependencies(native_messages)
            if kind == "invalid":
                # Pinned Cargo removes selected intermediate/hardlinked outputs
                # before invoking rustc (compiler/mod.rs:449-472). It may have
                # removed the old archive; surviving bytes must not be replaced.
                require(record.get("status") == 1 and not cargo_artifacts and old_files is not None
                        and not record.get("outputs") and not any(row.get("$message_type") == "artifact" for row in native_messages),
                        "typed failure reported successful artifact publication")
                for emit, path in saved_outputs[unit].items():
                    if path.exists():
                        require(emit in old_files and sha(path.read_bytes()) == old_files[emit]["sha256"],
                                "typed failure changed an existing successful artifact")
                require(old_delivered is not None, "typed failure lacks its prior delivered artifact witness")
                forwarded = [row["message"] for row in messages if row.get("reason") == "compiler-message"]
                forwarded_error = diagnostic(forwarded, project["source"], source_bytes, "L0110", tagged=False)
                raw_error = diagnostic(native_messages, project["source"], source_bytes, "L0110")
                for field in ("message", "code", "level", "spans", "children", "rendered"):
                    require(forwarded_error.get(field) == raw_error.get(field),
                            "Cargo-forwarded native type diagnostic differs from actual response: " + field)
                require(all(record.get("counts", {}).get(key) == 0 for key in ("compiled_objects", "metadata_encodes", "archive_builds")),
                        "type rejection performed artifact work")
                oracle = case / "invalid-oracle"
                oracle.mkdir()
                argv = list(invocation["argv"][1:])
                argv[argv.index("--out-dir") + 1] = str(oracle)
                _, original_stderr, _ = commands.run(label + "-original-invalid", [sdk / "bin/rustc", *argv], project["root"], oracle_env, expected=1)
                original_messages = [json.loads(line) for line in original_stderr.splitlines() if line]
                diagnostic(original_messages, project["source"], source_bytes, "E0308")
                unused_dependencies(original_messages)
            else:
                require(record.get("status") == 0 and len(cargo_artifacts) == 1 and cargo_artifacts[0].get("fresh") is False,
                        "successful public request was not a real completed Cargo compilation")
                observed = inspect_artifact(report_path.parent, record, outputs, project["source"], source_bytes, kind, previous.get(unit))
                # Cargo may have deleted requested output paths before this
                # real invocation. Identical cached bytes must be delivered again;
                # raw-public replay below separately proves no republishing when
                # the caller has left all three successful files in place.
                output_records = record.get("outputs")
                require(isinstance(output_records, list) and len(output_records) == 3, "public artifact receipt is incomplete")
                by_emit = {row.get("emit"): row for row in output_records}
                require(set(by_emit) == set(outputs), "public artifact receipt duplicated/omitted a requested output")
                for emit, path in outputs.items():
                    row = by_emit[emit]
                    require(row.get("path") == str(path) and row.get("sha256") == sha(path.read_bytes()) and row.get("size") == path.stat().st_size,
                            "public artifact receipt differs from actual named bytes")
                artifact_messages = {row.get("emit"): Path(row.get("artifact", "")) for row in native_messages if row.get("$message_type") == "artifact"}
                require(artifact_messages == outputs, "accepted JSON artifact options did not emit the actual output notifications")
                delivered = delivered_artifacts(cargo_artifacts[0], outputs)
                saved_delivered[unit] = {emit: path.open("rb") for emit, path in delivered.items()}
                delivered_handles.extend(saved_delivered[unit].values())
                frozen = case / "artifacts"
                frozen.mkdir()
                frozen_outputs = {}
                for emit, path in outputs.items():
                    target = frozen / path.name
                    write_new(target, path.read_bytes())
                    frozen_outputs[emit] = target
                root_identity = oracle_case(commands, sdk, oracle_env, case, invocation, frozen_outputs, mask, operation, report)
                if unit in crate_roots:
                    require(root_identity["stable_crate_id_hex"] == crate_roots[unit], "body-only edit changed Rust crate identity")
                else:
                    require(root_identity["stable_crate_id_hex"] not in crate_roots.values(), "different Cargo unit metadata identity aliased")
                    crate_roots[unit] = root_identity["stable_crate_id_hex"]
                previous[unit], saved_outputs[unit] = observed, outputs
            live = [row["native_object_serial"] for row in previous.get(unit, {}).get("objects", {}).values()]
            cache_observation(record, live)
            seal_reports()
            require(not trap_log.exists(), "product invoked a forbidden compiler/emitter trap")
            report["observed_history"].append({"case": label, "unit": unit, "kind": kind, "source_sha256": sha(source_bytes),
                "invocation": invocation, "native_report": str(report_path.relative_to(root)), "native_report_sha256": sha(report_path.read_bytes()),
                "context": context, "actual_artifact_tools": actual_tools})
            save()


        # Actual bounded-history retirement, chosen before candidate execution.
        # The restored127 version is fixed, never selected from reported victims.
        project = projects["first"]
        invocation = report["observed_history"][-1]["invocation"]
        direct_env = dict(invocation["environment"])
        facts = parse_compile(invocation)
        outputs = saved_outputs["first"]
        baseline = previous["first"]
        baseline_serial = baseline["objects"]["limit"]["native_object_serial"]
        fixed_context = report["observed_history"][-1]["context"]
        eviction = {"masks": EVICTION_MASKS, "observed_history": [], "original_consumers": [],
                    "checked_runtime_results": 0, "restored_prior_serial": baseline_serial,
                    "performance_qualified": False, "entry_limit_exercised": True, "byte_limit_stress_qualified": False}
        report["eviction"] = eviction
        evicted = set()
        for index, mask in enumerate(EVICTION_MASKS):
            label = f"eviction-{index + 1}-mask-{mask}"
            if index == 3:
                require(baseline_serial in evicted, "preselected127 native generation was not evicted before restoration")
            text = EDIT.replace(b"input & 127u64", f"input & {mask}u64".encode())
            case = root / "cases" / label
            case.mkdir()
            project["source"].write_bytes(text)
            write_new(case / "source.rs", text)
            before_reports = set((root / "observations").glob("*/report.json"))
            stdout, stderr, direct_trace = commands.run(label, invocation["argv"], invocation["cwd"], direct_env, traced=True)
            require(not stdout and process_identity(owner[0], package / "bin/lust-core") == owner
                    and project["source"].read_bytes() == text, "eviction edit changed real owner/source/response protocol")
            parsed_direct = trace_events(direct_trace.read_bytes(), invocation["cwd"])
            inspect_processes(parsed_direct, package, package / "bin/rustc", invocation["argv"], projects)
            require(parsed_direct["exits"].get(parsed_direct["root_pid"]) == 0, "eviction request did not complete successfully")
            new_reports = set((root / "observations").glob("*/report.json")) - before_reports
            require(len(new_reports) == 1, "eviction request lacks one actual retained response")
            marker = next(iter(new_reports))
            record = json.loads(marker.read_text())
            context = validate_context(record, facts, text, owner)
            require(context == fixed_context and record["native_context"] == contexts["first"]
                    and record.get("request_index") == len(HISTORY) + index + 1 and record.get("status") == 0,
                    "eviction sequence changed source/metadata/profile/session context")
            require(witness(marker.parent, "native.stderr") == stderr, "eviction response witness differs from actual public bytes")
            messages = [json.loads(line) for line in stderr.splitlines() if line]
            unused_dependencies(messages)
            notices = [row for row in messages if row.get("$message_type") == "artifact"]
            require(len(notices) == 3 and {row.get("emit"): Path(row.get("artifact", "")) for row in notices} == outputs,
                    "eviction request omitted actual output notifications")
            observed = inspect_artifact(marker.parent, record, outputs, project["source"], text, "edit", previous["first"])
            history = cache_observation(record, [row["native_object_serial"] for row in observed["objects"].values()])
            evicted.update(history["native"]["evicted_object_serials"])
            prefix = daemon_trace.read_bytes()
            frozen_prefix(prefixes[-1]["bytes"], prefix)
            owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
            actual_tools = owner_commands["artifacts"][prefixes[-1]["artifact_count"]:]
            reconcile_artifact_calls(actual_tools, command_inventory(record, outputs, llvm))
            prefixes.append({"name": label, "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
            if index == 3:
                restored = observed["objects"]["limit"]
                original = baseline["objects"]["limit"]
                require(restored["native_object_serial"] != baseline_serial and restored.get("reused") is False
                        and all(restored[key] == original[key] for key in ("native_id", "native_symbol", "rust_symbol", "original_sha256", "renamed_sha256")),
                        "evicted127 restoration did not create fresh actual TinyCC output with the original bytes/identity")
                require(observed["archive"] == baseline["archive"] and observed["metadata"] == baseline["metadata"],
                        "restored source did not reproduce its original complete Rust artifact")
            frozen = case / "artifacts"
            frozen.mkdir()
            frozen_outputs = {}
            for emit, path in outputs.items():
                target = frozen / path.name
                write_new(target, path.read_bytes())
                frozen_outputs[emit] = target
            identity = oracle_case(commands, sdk, oracle_env, case, invocation, frozen_outputs, mask, "xor", eviction)
            require(identity["stable_crate_id_hex"] == crate_roots["first"], "cache eviction changed actual Rust crate identity")
            previous["first"] = observed
            seal_reports()
            require(not trap_log.exists(), "eviction request invoked a forbidden compiler")
            eviction["observed_history"].append({"case": label, "mask": mask, "source_sha256": sha(text),
                "native_report": str(marker.relative_to(root)), "cache_history": history, "actual_artifact_tools": actual_tools})
            save()
        require(len(eviction["observed_history"]) == len(eviction["original_consumers"]) == 4
                and eviction["checked_runtime_results"] == EVICTION_ASSERTIONS, "eviction history/consumer denominator is incomplete")
        eviction["passed"] = True

        # Incompatible version context must reach and be refused by the actual
        # warm owner before native preparation. Following raw no-op proves recovery.
        version = root / "cases" / "incompatible-version"
        version.mkdir()
        before_reports = set((root / "observations").glob("*/report.json"))
        before_files = file_snapshot(outputs)
        version_env = dict(direct_env, RUSTC_FORCE_RUSTC_VERSION=VERSION_REFUSAL)
        stdout, stderr, version_trace = commands.run("incompatible-version", invocation["argv"], invocation["cwd"], version_env,
                                                    expected=4, traced=True)
        require(not stdout and process_identity(owner[0], package / "bin/lust-core") == owner, "version refusal lost actual warm owner")
        parsed_version = trace_events(version_trace.read_bytes(), invocation["cwd"])
        inspect_processes(parsed_version, package, package / "bin/rustc", invocation["argv"], projects)
        require(parsed_version["exits"].get(parsed_version["root_pid"]) == 4, "actual public version refusal exit changed")
        new_reports = set((root / "observations").glob("*/report.json")) - before_reports
        require(len(new_reports) == 1, "version request was refused before the required observed warm-owner boundary")
        marker = next(iter(new_reports))
        record = json.loads(marker.read_text())
        validate_context(record, facts, EDIT, owner, prepared=False)
        require(record.get("request_index") == RAW_REQUEST_OFFSET and record.get("status") == 4
                and record.get("outcome") == "integration-incomplete" and record.get("phase") == "request-context"
                and record["native_context"] == contexts["first"], "version mismatch did not preserve its typed pre-preparation refusal")
        require(witness(marker.parent, "native.stderr") == stderr, "version refusal response differs from actual bytes")
        messages = [json.loads(line) for line in stderr.splitlines() if line]
        require(len(messages) == 1 and messages[0].get("$message_type") == "diagnostic"
                and messages[0].get("level") == "error" and messages[0].get("code") is None
                and messages[0].get("spans") == [] and messages[0].get("message") ==
                "error: RUSTC_FORCE_RUSTC_VERSION differs from pinned metadata context", "version refusal diagnostic changed its exact context cause")
        require(not record.get("commands") and not record.get("outputs") and not record.get("checked_definitions")
                and not record.get("generated_definitions")
                and all(record.get("counts", {}).get(key) == 0 for key in
                    ("checked_bodies", "generated_bodies", "compiled_objects", "metadata_encodes", "archive_builds")),
                "version refusal performed native preparation or artifact work")
        require(file_snapshot(outputs) == before_files, "incompatible version request changed last-good output bytes or publication")
        cache_observation(record, [row["native_object_serial"] for row in previous["first"]["objects"].values()])
        prefix = daemon_trace.read_bytes()
        frozen_prefix(prefixes[-1]["bytes"], prefix)
        owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        require(len(owner_commands["artifacts"]) == prefixes[-1]["artifact_count"], "version refusal executed artifact tools")
        prefixes.append({"name": "incompatible-version", "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
        seal_reports()
        report["version_refusal"] = {"passed": True, "owner_request_observed": True, "native_report": str(marker.relative_to(root)),
            "actual_environment_override": {"RUSTC_FORCE_RUSTC_VERSION": VERSION_REFUSAL}, "new_consumer_assertions": 0}
        save()

        # Observe the raw public JSON protocol once; Cargo's wrapper intentionally
        # consumes artifact notifications. Reuse the exact observed invocation.
        replay = root / "cases" / "raw-public-json-noop"
        replay.mkdir()
        project = projects["first"]
        invocation = report["observed_history"][-1]["invocation"]
        replay_env = dict(invocation["environment"])
        report["raw_replay_environment"] = {"unchanged": True,
            "jobserver": {key: replay_env[key] for key in ("CARGO_MAKEFLAGS", "MAKEFLAGS", "MFLAGS") if key in replay_env},
            "scope": "exact captured environment; no live jobserver-descriptor capability is claimed for this no-op replay"}
        before_reports = set((root / "observations").glob("*/report.json"))
        before_files = file_snapshot(saved_outputs["first"])
        stdout, stderr, replay_trace = commands.run("raw-public-json-noop", invocation["argv"], invocation["cwd"], replay_env, traced=True)
        require(not stdout, "compiler JSON protocol unexpectedly wrote ordinary stdout")
        parsed_replay = trace_events(replay_trace.read_bytes(), invocation["cwd"])
        inspect_processes(parsed_replay, package, package / "bin/rustc", invocation["argv"], projects)
        require(parsed_replay["exits"].get(parsed_replay["root_pid"]) == 0, "raw public compiler did not exit successfully")
        messages = [json.loads(line) for line in stderr.splitlines() if line]
        unused_dependencies(messages)
        notifications = [row for row in messages if row.get("$message_type") == "artifact"]
        require(len(notifications) == 3 and {row.get("emit"): Path(row.get("artifact", "")) for row in notifications} == saved_outputs["first"],
                "raw public JSON omitted, duplicated or changed a requested artifact")
        require(not any(row.get("level") == "error" for row in messages), "raw no-op emitted a compiler error")
        new_reports = set((root / "observations").glob("*/report.json")) - before_reports
        require(len(new_reports) == 1, "raw public replay did not invoke its real retained owner exactly once")
        replay_report_path = next(iter(new_reports))
        replay_record = json.loads(replay_report_path.read_text())
        require(replay_record.get("request_index") == RAW_REQUEST_OFFSET + 1, "raw replay did not advance the actual owner request sequence")
        validate_context(replay_record, parse_compile(invocation), EDIT, owner)
        require(process_identity(owner[0], package / "bin/lust-core") == owner, "raw public replay lost the actual retained owner")
        require(replay_record["native_context"] == contexts["first"] and replay_record.get("status") == 0,
                "raw public replay changed retained source ownership")
        inspect_artifact(replay_report_path.parent, replay_record, saved_outputs["first"], project["source"], EDIT, "noop", previous["first"])
        cache_observation(replay_record, [row["native_object_serial"] for row in previous["first"]["objects"].values()])
        require(file_snapshot(saved_outputs["first"]) == before_files, "raw no-op repeated final artifact publication")
        prefix = daemon_trace.read_bytes()
        frozen_prefix(prefixes[-1]["bytes"], prefix)
        parsed_owner = trace_events(prefix, root)
        owner_commands = inspect_processes(parsed_owner, package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
        require(len(owner_commands["artifacts"]) == prefixes[-1]["artifact_count"] and not replay_record.get("commands"),
                "raw no-op repeated artifact utility work")
        prefixes.append({"name": "raw-public-json-noop", "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
        seal_reports()
        report["raw_public_json"] = {"passed": True, "native_report": str(replay_report_path.relative_to(root)),
                                     "notifications": notifications, "invocation": invocation["argv"]}
        report["raw_public_history"] = [{"case": "raw-public-json-noop",
            "native_report": str(replay_report_path.relative_to(root)), "source_sha256": sha(EDIT)}]
        for label, text, expected_status, request_index in (
            ("raw-public-typed-invalid", INVALID, 1, RAW_REQUEST_OFFSET + 2),
            ("raw-public-repair", EDIT, 0, RAW_REQUEST_OFFSET + 3),
        ):
            direct = root / "cases" / label
            direct.mkdir()
            project["source"].write_bytes(text)
            write_new(direct / "source.rs", text)
            before_reports = set((root / "observations").glob("*/report.json"))
            before_files = file_snapshot(saved_outputs["first"])
            stdout, stderr, direct_trace = commands.run(label, invocation["argv"], invocation["cwd"], replay_env,
                                                        expected=expected_status, traced=True)
            require(not stdout and process_identity(owner[0], package / "bin/lust-core") == owner,
                    "raw public request changed stdout protocol or retained owner")
            parsed_direct = trace_events(direct_trace.read_bytes(), invocation["cwd"])
            inspect_processes(parsed_direct, package, package / "bin/rustc", invocation["argv"], projects)
            require(parsed_direct["exits"].get(parsed_direct["root_pid"]) == expected_status,
                    "raw public request exit differs from actual process")
            new_reports = set((root / "observations").glob("*/report.json")) - before_reports
            require(len(new_reports) == 1, "raw public request lacks its single native report")
            marker = next(iter(new_reports))
            record = json.loads(marker.read_text())
            validate_context(record, parse_compile(invocation), text, owner)
            require(record.get("request_index") == request_index and record.get("status") == expected_status
                    and record["native_context"] == contexts["first"], "raw invalid/repair request changed retained source identity")
            require(witness(marker.parent, "native.stderr") == stderr, "raw public response differs from retained witness bytes")
            direct_messages = [json.loads(line) for line in stderr.splitlines() if line]
            unused_dependencies(direct_messages)
            if expected_status:
                diagnostic(direct_messages, project["source"], text, "L0110")
                require(not record.get("outputs") and not any(row.get("$message_type") == "artifact" for row in direct_messages)
                        and all(record.get("counts", {}).get(key) == 0 for key in ("compiled_objects", "metadata_encodes", "archive_builds")),
                        "raw typed rejection performed artifact work or publication")
            else:
                previous["first"] = inspect_artifact(marker.parent, record, saved_outputs["first"], project["source"], text,
                                                     "repair", previous["first"])
                notices = [row for row in direct_messages if row.get("$message_type") == "artifact"]
                require(len(notices) == 3 and {row.get("emit"): Path(row.get("artifact", "")) for row in notices} == saved_outputs["first"],
                        "raw repair omitted actual artifact notifications")
            require(file_snapshot(saved_outputs["first"]) == before_files,
                    "raw invalid/identical-source repair changed the last successful delivered output set")
            cache_observation(record, [row["native_object_serial"] for row in previous["first"]["objects"].values()])
            prefix = daemon_trace.read_bytes()
            frozen_prefix(prefixes[-1]["bytes"], prefix)
            owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
            actual_tools = owner_commands["artifacts"][prefixes[-1]["artifact_count"]:]
            reconcile_artifact_calls(actual_tools, command_inventory(record, saved_outputs["first"], llvm))
            if expected_status:
                require(not actual_tools, "raw type rejection executed artifact tools")
            prefixes.append({"name": label, "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
            seal_reports()
            report["raw_public_history"].append({"case": label, "source_sha256": sha(text),
                "native_report": str(marker.relative_to(root)),
                "byte_identity_reference": "first-unit-revisit independently consumed generation",
                "new_consumer_assertions": 0, "actual_artifact_tools": actual_tools})
            save()
        require([row["case"] for row in report["raw_public_history"]]
                == ["raw-public-json-noop", "raw-public-typed-invalid", "raw-public-repair"],
                "raw public no-op/rejection/recovery subhistory is incomplete")

        # Separate direct-request group: Cargo's original nine invocations and
        # the existing eviction/raw histories above remain unchanged.
        transitions = {"passed": False, "request_kind": "direct replay of captured Cargo compiler argv",
                       "observed_history": [], "original_consumers": [], "checked_runtime_results": 0,
                       "malformed_option_refusals": [], "native_work_expected": "zero checks, generation and TinyCC"}
        report["metadata_transitions"] = transitions
        project = projects["first"]
        outputs = saved_outputs["first"]
        transition_baseline = previous["first"]
        transition_files = file_snapshot(outputs)
        transition_context = dict(record["context"])
        transition_start = RAW_REQUEST_OFFSET + 3
        require(record.get("request_index") == transition_start and record.get("status") == 0,
                "metadata transitions did not follow the unchanged raw history")
        captured = report["observed_history"][-1]["invocation"]
        captured_facts = parse_compile(captured)
        transition_roots = []
        reference_roots = []
        for index, (label, choice) in enumerate(METADATA_TRANSITIONS):
            if index == 3:
                # One deliberately malformed request, then the original false
                # option must recover through the same owner without a gap.
                bad_argv = metadata_replay_argv(captured, "bogus")
                before_reports = set((root / "observations").glob("*/report.json"))
                before_files = {unit: file_snapshot(paths) for unit, paths in saved_outputs.items()}
                before_delivered = {unit: file_snapshot(paths) for unit, paths in saved_delivered.items()}
                stdout, stderr, bad_trace = commands.run("metadata-bogus-refusal", bad_argv, captured["cwd"], replay_env,
                                                        expected=2, traced=True)
                parsed_bad = trace_events(bad_trace.read_bytes(), captured["cwd"])
                bad_client = inspect_processes(parsed_bad, package, package / "bin/rustc", bad_argv, projects)
                require(len(bad_client["compiles"]) == 1 and bad_client["compiles"][0]["argv"] == bad_argv
                        and bad_client["compiles"][0]["environment"] == replay_env
                        and parsed_bad["exits"].get(parsed_bad["root_pid"]) == 2 and not stdout
                        and b"incomplete" in stderr.lower(), "malformed metadata option did not remain an actual terminal refusal")
                require(set((root / "observations").glob("*/report.json")) == before_reports,
                        "malformed metadata option reached a native owner request")
                require(all(file_snapshot(saved_outputs[unit]) == before_files[unit] for unit in saved_outputs)
                        and all(file_snapshot(saved_delivered[unit]) == before_delivered[unit] for unit in saved_delivered),
                        "malformed metadata option changed a last successful output")
                prefix = daemon_trace.read_bytes()
                frozen_prefix(prefixes[-1]["bytes"], prefix)
                owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
                require(len(owner_commands["artifacts"]) == prefixes[-1]["artifact_count"]
                        and process_identity(owner[0], package / "bin/lust-core") == owner and not trap_log.exists(),
                        "malformed metadata option performed artifact work or lost its owner")
                prefixes.append({"name": "metadata-bogus-refusal", "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
                transitions["malformed_option_refusals"].append({"case": "metadata-bogus-refusal", "status": 2,
                    "invocation": bad_client["compiles"][0], "owner_request_observed": False,
                    "outputs_unchanged": True, "new_consumer_assertions": 0})
                seal_reports()
                save()
            case = root / "cases" / label
            case.mkdir()
            require(project["source"].read_bytes() == EDIT, "metadata transition changed the original source")
            write_new(case / "source.rs", EDIT)
            argv = metadata_replay_argv(captured, choice)
            before_reports = set((root / "observations").glob("*/report.json"))
            before_files = file_snapshot(outputs)
            stdout, stderr, trace = commands.run(label, argv, captured["cwd"], replay_env, traced=True)
            require(not stdout and process_identity(owner[0], package / "bin/lust-core") == owner,
                    "metadata option change replaced the owner or wrote ordinary stdout")
            parsed_client = trace_events(trace.read_bytes(), captured["cwd"])
            client = inspect_processes(parsed_client, package, package / "bin/rustc", argv, projects)
            require(len(client["compiles"]) == 1 and parsed_client["exits"].get(parsed_client["root_pid"]) == 0,
                    "metadata transition did not perform exactly one successful public invocation")
            invocation = client["compiles"][0]
            require(invocation["argv"] == argv and invocation["cwd"] == captured["cwd"]
                    and invocation["environment"] == replay_env, "metadata replay changed captured argv/environment/context")
            facts = parse_compile(invocation)
            require({key: value for key, value in facts.items() if key != "unstable"}
                    == {key: value for key, value in captured_facts.items() if key != "unstable"},
                    "metadata transition changed another original compiler argument")
            options = library_options(facts)
            require(options["embed_metadata"] is (None if choice is None else choice == "yes"),
                    "actual metadata transition differs from its preselected option")
            new_reports = set((root / "observations").glob("*/report.json")) - before_reports
            require(len(new_reports) == 1, "metadata transition lacks exactly one completed native response")
            marker = next(iter(new_reports))
            record = json.loads(marker.read_text())
            context = validate_context(record, facts, EDIT, owner)
            require(context == transition_context and record["native_context"] == contexts["first"]
                    and record.get("request_index") == transition_start + index + 1 and record.get("status") == 0,
                    "metadata request options changed native session/source ownership or request continuity")
            require(witness(marker.parent, "native.stderr") == stderr, "metadata transition response differs from retained witness")
            messages = [json.loads(line) for line in stderr.splitlines() if line]
            unused_dependencies(messages)
            notices = [row for row in messages if row.get("$message_type") == "artifact"]
            require(len(notices) == 3 and {row.get("emit"): Path(row.get("artifact", "")) for row in notices} == outputs,
                    "metadata transition omitted or changed the actual output notifications")
            output_records = record.get("outputs", [])
            require(len(output_records) == 3 and {row.get("emit") for row in output_records} == set(outputs),
                    "metadata transition output receipt is incomplete")
            for row in output_records:
                path = outputs[row["emit"]]
                require(row.get("path") == str(path) and row.get("sha256") == sha(path.read_bytes())
                        and row.get("size") == path.stat().st_size, "metadata transition receipt differs from actual bytes")
            observed = inspect_artifact(marker.parent, record, outputs, project["source"], EDIT, "options",
                                        previous["first"], embed_metadata=options["embed_metadata"])
            counts = observed["counts"]
            require(not record.get("checked_definitions") and not record.get("generated_definitions")
                    and all(counts.get(key) == 0 for key in ("checked_bodies", "generated_bodies", "compiled_objects"))
                    and counts.get("reused_objects") == 3, "metadata-only option change repeated native checking/generation/TinyCC")
            artifact_work = (counts.get("metadata_encodes"), counts.get("archive_builds"))
            require(artifact_work in {(0, 0), (1, 1)}
                    and record.get("artifact_reused") is (artifact_work == (0, 0)), "metadata artifact work observations are inconsistent")
            if index == 0:
                require(artifact_work == (0, 0) and file_snapshot(outputs) == before_files,
                        "unchanged false-option replay repeated artifact work/publication")
            elif index in (1, 2):
                require(artifact_work == (1, 1), "new tracked metadata option reused a different serializer/archive context")
            else:
                require(artifact_work == (0, 0), "restored false option regenerated its still-retained artifact variant")
            require(sha(outputs["dep-info"].read_bytes()) == transition_files["dep-info"]["sha256"],
                    "metadata option changed the unchanged dependency prerequisites/targets")
            prefix = daemon_trace.read_bytes()
            frozen_prefix(prefixes[-1]["bytes"], prefix)
            owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
            actual_tools = owner_commands["artifacts"][prefixes[-1]["artifact_count"]:]
            wanted_tools = command_inventory(record, outputs, llvm)
            reconcile_artifact_calls(actual_tools, wanted_tools)
            require([command["name"] for command in record.get("commands", [])]
                    == (["archive"] if artifact_work[1] else []), "metadata option repeated native object renaming or extra artifact work")
            prefixes.append({"name": label, "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
            cache_observation(record, [row["native_object_serial"] for row in observed["objects"].values()])
            frozen = case / "artifacts"
            frozen.mkdir()
            frozen_outputs = {}
            for emit, path in outputs.items():
                target = frozen / path.name
                write_new(target, path.read_bytes())
                frozen_outputs[emit] = target
            identity = oracle_case(commands, sdk, oracle_env, case, invocation, frozen_outputs, 127, "xor", transitions)
            require(identity["stable_crate_id_hex"] == crate_roots["first"], "metadata option changed stable crate/definition identity")
            transition_roots.append(identity["svh_le_hex"])
            reference_roots.append(transitions["original_consumers"][-1]["reference_crate_root"]["svh_le_hex"])
            if index == 3:
                require(observed["archive"] == transition_baseline["archive"] and observed["metadata"] == transition_baseline["metadata"],
                        "restored false metadata choice did not recover its original exact artifacts")
            require(project["source"].read_bytes() == EDIT and not trap_log.exists(), "metadata option rewrote source or invoked a competing compiler")
            previous["first"] = observed
            transitions["observed_history"].append({"case": label, "invocation": invocation, "library_options": options,
                "native_report": str(marker.relative_to(root)), "source_sha256": sha(EDIT), "counts": counts,
                "actual_artifact_tools": actual_tools, "crate_hash_le_hex": identity["svh_le_hex"],
                "output_files": file_snapshot(outputs)})
            seal_reports()
            save()
        require([row["case"] for row in transitions["observed_history"]] == [row[0] for row in METADATA_TRANSITIONS]
                and [row["case"] for row in transitions["original_consumers"]] == [row[0] for row in METADATA_TRANSITIONS]
                and transitions["checked_runtime_results"] == METADATA_TRANSITION_ASSERTIONS
                and len(transitions["malformed_option_refusals"]) == 1, "metadata transition/refusal/consumer denominator changed")
        require(len(set(transition_roots[:3])) == len(set(reference_roots[:3])) == 3
                and transition_roots[-1] == transition_roots[0] and reference_roots[-1] == reference_roots[0],
                "metadata option SVH identity does not match distinct upstream tracked states and restoration")
        transitions["passed"] = True

        # Exercise the metadata-only route through ordinary pinned Cargo. Use
        # different output directories from link requests and assert each
        # actual daemon response rather than accepting Cargo's Fresh result.
        project = projects["check"]
        check_history = [("cold", BASE), ("noop", BASE), ("edit", EDIT),
                         ("invalid", INVALID), ("repair", EDIT)]
        checks = {"passed": False, "observed_history": [], "original_consumers": []}
        report["metadata_only"] = checks
        check_context = None
        checked_bytes = None
        for kind, source_bytes in check_history:
            label = "metadata-check-" + kind
            case = root / "cases" / label
            case.mkdir()
            project["source"].write_bytes(source_bytes)
            write_new(case / "source.rs", source_bytes)
            env = dict(environment, CARGO_TARGET_DIR=str(project["target"]), CARGO_BUILD_BUILD_DIR=str(project["build"]))
            cargo_args = [package / "bin/cargo", "check", "--lib", "--offline", "--message-format=json"]
            before_reports = set((root / "observations").glob("*/report.json"))
            expected = 101 if kind == "invalid" else 0
            stdout, _, trace = commands.run(label, cargo_args, project["root"], env, expected=expected, traced=True)
            parsed = trace_events(trace.read_bytes(), project["root"])
            client = inspect_processes(parsed, package, package / "bin/cargo", cargo_args, projects)
            require(len(client["compiles"]) == 1 and parsed["exits"].get(parsed["root_pid"]) == expected,
                    "Cargo check did not issue a real metadata-only compiler request")
            invocation = client["compiles"][0]
            facts = parse_compile(invocation)
            outputs = declared_compile(facts, project, check=True)
            new_reports = set((root / "observations").glob("*/report.json")) - before_reports
            require(len(new_reports) == 1, "Cargo check did not use the retained native owner exactly once")
            marker = next(iter(new_reports))
            record = json.loads(marker.read_text())
            validate_context(record, facts, source_bytes, owner)
            if check_context is None:
                check_context = record["native_context"]
            require(record["native_context"] == check_context and process_identity(owner[0], package / "bin/lust-core") == owner,
                    "Cargo check lost its retained native context")
            counts = record["counts"]
            require(all(counts[key] == 0 for key in ("generated_bodies", "compiled_objects", "reused_objects", "archive_builds"))
                    and not record["commands"] and not record["objects"], "Cargo check performed native emission or archive work")
            prefix = daemon_trace.read_bytes()
            frozen_prefix(prefixes[-1]["bytes"], prefix)
            owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
            require(len(owner_commands["artifacts"]) == prefixes[-1]["artifact_count"], "Cargo check invoked artifact tools")
            prefixes.append({"name": label, "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
            messages = [json.loads(line) for line in stdout.splitlines() if line]
            cargo_artifacts = [row for row in messages if row.get("reason") == "compiler-artifact"]
            native_messages = [json.loads(line) for line in witness(marker.parent, "native.stderr").splitlines() if line]
            unused_dependencies(native_messages)
            require(not list(project["build"].rglob("*.rlib")) and not list(project["build"].rglob("*.o")),
                    "Cargo check produced a link artifact")
            rustc = sdk / "bin/rustc"
            original_dir = case / "original"
            original_dir.mkdir()
            argv = invocation["argv"][1:]
            argv[argv.index("--out-dir") + 1] = str(original_dir)
            _, original_stderr, _ = commands.run(label + "-original-producer", [rustc, *argv],
                                                   project["root"], oracle_env, expected=1 if expected else 0)
            if kind == "invalid":
                require(record["status"] == 1 and not cargo_artifacts and not record["outputs"]
                        and counts["metadata_encodes"] == 0, "invalid Cargo check published metadata")
                diagnostic(native_messages, project["source"], source_bytes, "L0110")
                diagnostic([json.loads(line) for line in original_stderr.splitlines() if line],
                           project["source"], source_bytes, "E0308")
            else:
                metadata_path = outputs["metadata"]
                require(record["status"] == 0 and len(cargo_artifacts) == 1 and cargo_artifacts[0].get("fresh") is False
                        and cargo_artifacts[0]["filenames"] == [str(metadata_path)], "Cargo check delivered the wrong artifact")
                notices = {row["emit"]: Path(row["artifact"]) for row in native_messages if row.get("$message_type") == "artifact"}
                require(notices == outputs, "Cargo check emitted the wrong artifact notices")
                dep_info(outputs["dep-info"].read_bytes(), outputs, project["source"], project["root"])
                current_bytes = metadata_path.read_bytes()
                if kind == "noop":
                    require(current_bytes == checked_bytes and counts["checked_bodies"] == counts["metadata_encodes"] == 0
                            and record["artifact_reused"], "warm Cargo check did not reuse its checked metadata")
                if kind == "edit":
                    require(current_bytes != checked_bytes and counts["metadata_encodes"] == 1
                            and set(source_rows(record["checked_definitions"], "check edit")) == {"limit"},
                            "body edit did not update metadata from only the changed body")
                checked_bytes = current_bytes
                roots = []
                for name, path in (("native", metadata_path), ("original", original_dir / metadata_path.name)):
                    root_text, _, _ = commands.run(label + "-" + name + "-reader", [rustc, "-Zls=root", path], case, oracle_env)
                    roots.append(construction.parse_root(root_text))
                require(all(roots[0].get(key) == value for key, value in roots[1].items()
                            if key not in {"hash", "svh", "svh_le_hex"}), "metadata-only Rust crate context differs from stock")
                compare_lang_items(commands, rustc, oracle_env, case, metadata_path, original_dir / metadata_path.name)
                caller = case / "caller.rs"
                write_new(caller, f"#![no_std]\npub fn call(x: u64) -> u64 {{ {CRATE}::project(x) }}\n")
                delivered = case / "libcaller.rmeta"
                commands.run(label + "-consumer", [rustc, "--edition=2024", "--crate-type=lib", "--emit=metadata",
                    "-Cpanic=abort", "--extern", CRATE + "=" + str(metadata_path), caller, "-o", delivered], case, oracle_env)
                require(delivered.is_file(), "independent consumer did not produce checked metadata")
                checks["original_consumers"].append({"case": label, "metadata_sha256": sha(delivered.read_bytes())})
            require(not trap_log.exists() and project["source"].read_bytes() == source_bytes,
                    "Cargo check rewrote source or launched a competing tool")
            checks["observed_history"].append({"case": label, "request": invocation,
                "native_report": str(marker.relative_to(root)), "counts": counts, "status": record["status"]})
            seal_reports()
            save()
        checks["passed"] = True

        qualify_dependency_checks(root, package, sdk, commands, environment, oracle_env, projects, owner, report)
        seal_reports()
        save()
        qualify_dependency_links(root, package, sdk, commands, environment, oracle_env, projects, owner,
                                 daemon_trace, daemon_argv, prefixes, report)
        seal_reports()
        save()
        qualify_executable(root, package, sdk, commands, environment, oracle_env, projects, owner,
                           daemon_trace, daemon_argv, prefixes, report)
        qualify_executable(root, package, sdk, commands, environment, oracle_env, projects, owner,
                           daemon_trace, daemon_argv, prefixes, report, check=True)
        qualify_executable(root, package, sdk, commands, environment, oracle_env, projects, owner,
                           daemon_trace, daemon_argv, prefixes, report, module=True)
        qualify_executable(root, package, sdk, commands, environment, oracle_env, projects, owner,
                           daemon_trace, daemon_argv, prefixes, report, check=True, module=True)
        qualify_expansion_inputs(root, package, sdk, commands, environment, oracle_env, projects, owner,
                                 daemon_trace, daemon_argv, prefixes, report)
        qualify_expansion_inputs(root, package, sdk, commands, environment, oracle_env, projects, owner,
                                 daemon_trace, daemon_argv, prefixes, report, check=True)
        qualify_executable_extern_uses(root, package, sdk, commands, environment, oracle_env, projects, owner,
                                      daemon_trace, daemon_argv, prefixes, report)
        qualify_integer_signatures(root, package, sdk, commands, environment, oracle_env, projects,
                                   daemon_argv, daemon_trace, prefixes, owner, report)
        qualify_default_executable_check(root, package, sdk, commands, environment, oracle_env, projects,
                                         daemon_argv, daemon_trace, prefixes, owner, report)
        seal_reports()
        save()
        # A direct archive passed where Cargo normally passes full metadata is refused.
        for label, extra in (
            ("external-crate", ["rustc", "--lib", "--", "--extern", "unused_external=" + str(saved_outputs["second"]["link"])]),
        ):
            target = root / "cases" / label
            target.mkdir()
            project = projects["first"]
            env = dict(environment, CARGO_TARGET_DIR=str(target / "target"), CARGO_BUILD_BUILD_DIR=str(target / "intermediates"))
            cargo_args = [package / "bin/cargo", *extra[:2], "--offline", "--message-format=json", *extra[2:]]
            before_reports = set((root / "observations").glob("*/report.json"))
            before_files = {unit: file_snapshot(paths) for unit, paths in saved_outputs.items()}
            stdout, stderr, trace = commands.run(label, cargo_args, project["root"], env, expected=101, traced=True)
            parsed_client = trace_events(trace.read_bytes(), project["root"])
            observed = inspect_processes(parsed_client, package, package / "bin/cargo", cargo_args, projects)
            require(len(observed["compiles"]) == 1 and parsed_client["exits"].get(parsed_client["root_pid"]) == 101,
                    "unsupported operation did not reach one actual public compiler refusal")
            require(set((root / "observations").glob("*/report.json")) == before_reports,
                    "unrepresented request was admitted to native library compilation")
            require(not list(target.rglob("*.rlib")) and not list(target.rglob("*.rmeta")),
                    "unrepresented request published an artifact")
            require(all(file_snapshot(saved_outputs[unit]) == before_files[unit] for unit in saved_outputs),
                    "unrepresented request replaced a prior successful artifact")
            messages = [json.loads(line) for line in stdout.splitlines() if line]
            require(not any(row.get("reason") == "compiler-artifact" for row in messages),
                    "Cargo reported an artifact for a refused request")
            require(b"incomplete" in stdout.lower() + stderr.lower() or b"unintegrated" in stdout.lower() + stderr.lower(),
                    "refusal did not identify the integration gap")
            prefix = daemon_trace.read_bytes()
            frozen_prefix(prefixes[-1]["bytes"], prefix)
            owner_commands = inspect_processes(trace_events(prefix, root), package, Path(daemon_argv[0]), daemon_argv, projects, daemon=True)
            require(len(owner_commands["artifacts"]) == prefixes[-1]["artifact_count"], "unrepresented request triggered artifact work")
            prefixes.append({"name": label, "bytes": prefix, "artifact_count": len(owner_commands["artifacts"])})
            require(not trap_log.exists() and process_identity(owner[0], package / "bin/lust-core") == owner,
                    "refusal invoked another compiler or replaced the retained owner")
            report["refusals"].append({"case": label, "scope": "valid unintegrated request; no compilation coverage claimed",
                                        "invocation": observed["compiles"][0]})
            seal_reports()
            save()
        require([row["case"] for row in report["observed_history"]] == [row[0] for row in HISTORY],
                "history was shortened, duplicated or reordered")
        require([row["case"] for row in report["original_consumers"]] == SUCCESS_CASES
                and report["checked_runtime_results"] == EXPECTED_ASSERTIONS,
                "independent consumer denominator changed or is incomplete")
        require(not trap_log.exists(), "compiler trap was reached")
        for identity in report["tools"]:
            require(file_identity(identity["path"]) == identity, "source/tool changed during campaign")
        seal_reports()
        report["baseline_original_consumers"] = len(report["original_consumers"]) + len(eviction["original_consumers"])
        report["baseline_checked_runtime_results"] = report["checked_runtime_results"] + eviction["checked_runtime_results"]
        require(report["baseline_original_consumers"] == 12 and report["baseline_checked_runtime_results"] == 3145824,
                "baseline independent assertion inventory changed")
        report["total_original_consumers"] = report["baseline_original_consumers"] + len(transitions["original_consumers"])
        report["total_checked_runtime_results"] = report["baseline_checked_runtime_results"] + transitions["checked_runtime_results"]
        require(report["total_original_consumers"] == 16 and report["total_checked_runtime_results"] == 4194432,
                "baseline plus explicit metadata-transition assertion inventory changed")
        report["functional_passed"] = True
    except Exception as error:
        report["failure"] = repr(error)
        report["failure_traceback"] = traceback.format_exc()
    finally:
        if daemon is not None:
            if daemon.poll() is None:
                try:
                    if owner is not None and process_identity(owner[0], package / "bin/lust-core") == owner:
                        # Signal only the owned daemon. Keep strace alive to
                        # observe the upstream controlled shutdown and final exits.
                        os.kill(owner[0], signal.SIGTERM)
                    else:
                        os.killpg(daemon.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            try:
                daemon.wait(timeout=5)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(daemon.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                try:
                    daemon.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    pass
                report["cleanup_failure"] = "owned traced daemon failed bounded termination"
        for handle in daemon_handles:
            handle.close()
        for handle in delivered_handles:
            handle.close()
        # A broken entry could have autostarted a replacement owner in a new
        # session. Stop only exact traced core daemons bound to our endpoint.
        stopped = []
        if "endpoint" in locals():
            for trace in root.rglob("*.trace"):
                for match in re.finditer(rb'(?m)^\s*(\d+)\s+execve\(("(?:[^"\\]|\\.)*")', trace.read_bytes()):
                    pid = int(match[1])
                    try:
                        path = Path(json.loads(match[2]))
                        if path.resolve() != (package / "bin/lust-core").resolve() or process_identity(pid, path) is None:
                            continue
                        arguments = Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")
                        environment_bytes = Path(f"/proc/{pid}/environ").read_bytes().split(b"\0")
                        if len(arguments) < 2 or arguments[1] != b"daemon" or os.fsencode("LUST_DAEMON_SOCKET=" + str(endpoint)) not in environment_bytes:
                            continue
                        os.kill(pid, signal.SIGTERM)
                        deadline = time.monotonic() + 2
                        while process_identity(pid, path) is not None and time.monotonic() < deadline:
                            time.sleep(0.02)
                        forced = process_identity(pid, path) is not None
                        if forced:
                            os.kill(pid, signal.SIGKILL)
                        stopped.append({"pid": pid, "forced": forced})
                    except (OSError, ValueError):
                        continue
            if stopped:
                report["unexpected_owned_daemons_stopped"] = stopped
                report["cleanup_failure"] = "an unexpected traced replacement daemon remained after public execution"
        if prefixes:
            try:
                final = (root / "daemon.trace").read_bytes()
                for boundary in prefixes:
                    frozen_prefix(boundary["bytes"], final)
                parsed = trace_events(final, root)
                require(parsed["root_pid"] == str(owner[0]) and parsed["exits"].get(str(owner[0])) in {0, "SIGTERM"},
                        "final process trace lacks actual owned termination")
                observed = inspect_processes(parsed, package, package / "bin/lust-core", [str(package / "bin/lust-core"), "daemon"], projects, daemon=True, complete=True)
                require(len(observed["artifacts"]) == prefixes[-1]["artifact_count"], "artifact work occurred after final response")
                report["daemon_trace"] = {"sha256": sha(final), "bytes": len(final), "process_tree": parsed["parents"],
                    "exits": parsed["exits"], "attempts": parsed["attempts"],
                    "terminal_undecoded_stops": parsed["terminal_undecoded_stops"],
                    "boundaries": [{"case": row["name"], "bytes": len(row["bytes"]), "sha256": sha(row["bytes"]),
                                    "artifact_count": row["artifact_count"]} for row in prefixes]}
                report["exec_trace_qualified"] = "cleanup_failure" not in report
            except Exception as error:
                report["trace_failure"] = repr(error)
        report["baseline_original_consumers"] = len(report["original_consumers"]) + len(report.get("eviction", {}).get("original_consumers", []))
        report["baseline_checked_runtime_results"] = report["checked_runtime_results"] + report.get("eviction", {}).get("checked_runtime_results", 0)
        report["total_original_consumers"] = report["baseline_original_consumers"] + len(report.get("metadata_transitions", {}).get("original_consumers", []))
        report["total_checked_runtime_results"] = report["baseline_checked_runtime_results"] + report.get("metadata_transitions", {}).get("checked_runtime_results", 0)
        report["retained_public_integration"] = bool(report["functional_passed"] and report["exec_trace_qualified"])
        report["passed"] = report["retained_public_integration"]
        report["sealed_request_files"] = sealed
        save()
    print(json.dumps({key: report[key] for key in ("passed", "functional_passed", "exec_trace_qualified",
           "retained_public_integration", "checked_runtime_results", "baseline_original_consumers", "baseline_checked_runtime_results",
           "total_original_consumers", "total_checked_runtime_results",
           "full_compiler_qualification", "performance_qualified")}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
