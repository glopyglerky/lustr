#!/usr/bin/env python3
"""Exercise preparation, checking and consumption of owned imported code plans."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--rustc", type=Path, required=True)
parser.add_argument("--qualifier", type=Path, required=True)
parser.add_argument("--out", type=Path, required=True)
args = parser.parse_args()
args.out.mkdir()
records = []


def run(name, command, expected=0, env=None):
    command = list(map(str, command))
    environment = os.environ | (env or {})
    result = subprocess.run(command, capture_output=True, timeout=60, env=environment)
    (args.out / f"{name}.stdout").write_bytes(result.stdout)
    (args.out / f"{name}.stderr").write_bytes(result.stderr)
    records.append({"name": name, "command": command, "environment": env or {},
                    "status": result.returncode, "expected": expected})
    assert result.returncode == expected, (name, result.returncode, result.stderr.decode(errors="replace"))
    return result


def native(name, source, expected=0, action="build", failure=None, output=None, extra=None):
    output = output or args.out / "consumer.native"
    env = {"LUSTR_QUALIFICATION_ACTION": action} | (extra or {})
    if failure:
        env["LUSTR_QUALIFICATION_ADAPTER_FAILURE"] = failure
    state = args.out / f"{name}-state"
    result = run(name, [args.qualifier, source, library, output, state], expected, env)
    return output, state, result


passed = False
try:
    peer = args.out / "peer.rs"
    peer.write_text("#![no_std]\n#[repr(transparent)]pub struct Word(pub i8);\n"
                    "pub fn add(value:Word,offset:i8)->Word{Word(value.0.wrapping_add(offset))}\n")
    library = args.out / "libpeer.rlib"
    run("peer", [args.rustc, "--edition=2024", "--crate-name=peer", "--crate-type=rlib",
                 "-Cpanic=abort", "-Copt-level=0", "-Cdebuginfo=0", "-Cembed-bitcode=no",
                 peer, "-o", library])
    source = args.out / "consumer.rs"
    source.write_text("extern crate peer;fn main(){let mut count=0u32;"
                      "for left in -128i16..128 {for right in -128i16..128 {"
                      "let actual=peer::add(peer::Word(left as i8),right as i8).0;"
                      "assert_eq!(actual,(left+right) as i8);count+=1;}}"
                      "println!(\"import emission: {} results\",count);}\n")
    invalid = args.out / "invalid.rs"
    invalid.write_text("extern crate peer;fn main(){peer::add(peer::Word(1),true);}\n")
    rust = args.out / "consumer.rust"
    run("reference", [args.rustc, "--edition=2024", "--crate-name=consumer", "-Cpanic=abort",
                      "-Copt-level=0", "-Cdebuginfo=0", "--extern", f"peer={library}", source, "-o", rust])
    expected = b"import emission: 65536 results\n"
    assert run("reference-run", [rust]).stdout == expected
    output, state, compiled = native("build", source)
    assert b"integer-adapters=1" in compiled.stderr
    assert b"imported-emission-transfer=once dependencies=preserved" in compiled.stderr
    assert run("build-run", [output]).stdout == expected
    successful = output.read_bytes()

    # Each control would fail if its owned LLVM invocation ran. Checks and
    # dropped Ready requests must succeed without materializing any object.
    for action in ["check", "abandon"]:
        for failure in [None, "missing", "failed", "malformed", "empty"]:
            _, state, result = native(f"{action}-{failure or 'normal'}", source, action=action, failure=failure)
            assert b"shared request:" in result.stderr
            assert b"imported-emission-transfer=once dependencies=preserved" in result.stderr
            assert output.read_bytes() == successful and not list(state.rglob("*.o"))
        absent = args.out / f"{action}.must-not-exist"
        _, state, _ = native(f"{action}-no-output", source, action=action, failure="failed", output=absent)
        assert not absent.exists() and not list(state.rglob("*.o"))

    for action in ["build", "check", "abandon"]:
        _, state, result = native(f"invalid-{action}", invalid, 1, action=action, failure="failed")
        assert b"shared request:" not in result.stderr
        assert b"LLVM adapter emission" not in result.stderr
        assert output.read_bytes() == successful and not list(state.rglob("*.o"))
    rejected = args.out / "invalid.rust"
    result = run("invalid-reference", [args.rustc, "--edition=2024", "-Cpanic=abort",
                 "--extern", f"peer={library}", invalid, "-o", rejected], 1)
    assert b"E0308" in result.stderr and not rejected.exists()

    for failure, diagnostic in [("missing", b"start imported LLVM adapter emission"),
                                ("failed", b"imported LLVM adapter emission failed"),
                                ("malformed", b"invalid imported LLVM adapter object"),
                                ("empty", b"invalid imported LLVM adapter object")]:
        _, state, result = native(f"build-{failure}", source, 3, failure=failure)
        assert b"shared request:" in result.stderr and diagnostic in result.stderr
        assert output.read_bytes() == successful and not list(state.rglob("*.o"))

    _, _, result = native("already-transferred", source, 3,
                          extra={"LUSTR_QUALIFICATION_TAKE_EMISSION": "1"})
    assert b"imported emission ownership failed" in result.stderr
    assert output.read_bytes() == successful
    _, _, result = native("object-publication-failed", source, 3,
                          extra={"LUSTR_QUALIFICATION_OBJECT_FAILURE": "1"})
    assert b"cannot materialize imported object" in result.stderr
    assert output.read_bytes() == successful
    recovered, _, _ = native("recovered", source,
                            extra={"LUSTR_QUALIFICATION_REMOVE_SNAPSHOT": "1"})
    assert run("recovered-run", [recovered]).stdout == expected
    passed = True
finally:
    summary = {"schema": 1, "scope": "owned imported emission lifecycle; no public routing or performance claim",
               "passed": passed, "qualifier_sha256": hashlib.sha256(args.qualifier.read_bytes()).hexdigest(),
               "cases": records}
    (args.out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(f"passed {len(records)} imported emission controls")
