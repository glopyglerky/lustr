#!/usr/bin/env python3
"""Real metadata/native value-call, rejection, profile and link controls."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--rustc", type=Path, required=True)
parser.add_argument("--qualifier", type=Path, required=True)
parser.add_argument("--out", type=Path, required=True)
args = parser.parse_args()
repo = Path(__file__).resolve().parent.parent
args.out.mkdir()
records = []


def run(name, command, expected=0, env=None):
    command = list(map(str, command))
    answer = subprocess.run(command, capture_output=True, timeout=60,
                            env=os.environ | (env or {}))
    (args.out / f"{name}.stdout").write_bytes(answer.stdout)
    (args.out / f"{name}.stderr").write_bytes(answer.stderr)
    records.append({"name": name, "command": command, "status": answer.returncode,
                    "expected": expected})
    assert answer.returncode == expected, (name, answer.returncode, answer.stderr.decode(errors="replace"))
    return answer


def peer(name, text, panic="abort"):
    source = args.out / f"{name}.rs"
    source.write_text(text)
    library = args.out / f"lib{name}.rlib"
    run(f"{name}-compile", [args.rustc, "--edition=2024", "--crate-type=rlib",
        "--crate-name=peer", f"-Cpanic={panic}", "-Copt-level=0", "-Cdebuginfo=0",
        "-Cembed-bitcode=no", source, "-o", library])
    return library


def native(name, source, library, expected=0, output=None, env=None):
    output = output or args.out / f"{name}.native"
    answer = run(name, [args.qualifier, source, library, output, args.out / f"{name}-state"], expected, env)
    return output, answer


def reference(name, source, library, expected=0):
    output = args.out / f"{name}.rust"
    answer = run(f"{name}-reference-compile", [args.rustc, "--edition=2024",
        "--crate-name=consumer", "-Cpanic=abort", "-Copt-level=0", "-Cdebuginfo=0",
        "--extern", f"peer={library}", source, "-o", output], expected)
    return output, answer


passed = False
try:
    fixture = repo / "tests/shared-imports"
    original = (fixture / "peer.rs").read_text()
    library = peer("peer", original)
    native_output, compiled = native("good", fixture / "consumer.rs", library)
    assert b"consumer-rust-typeck=0 consumer-rust-analysis=0" in compiled.stderr
    assert b"imported-generation-reuse=refused previous-artifact=preserved" in compiled.stderr
    actual = run("good-run", [native_output]).stdout
    rust_output, _ = reference("good", fixture / "consumer.rs", library)
    assert actual == run("good-reference-run", [rust_output]).stdout == b"shared nominal calls: 65541\n"
    successful = native_output.read_bytes()

    malformed = args.out / "malformed.rs"
    malformed.write_text("extern crate peer; fn main( {\n")
    _, answer = native("malformed", malformed, library, 3, native_output)
    assert b"unclosed delimiter" in answer.stderr and native_output.read_bytes() == successful
    rejected, answer = reference("malformed", malformed, library, 1)
    assert b"unclosed delimiter" in answer.stderr and not rejected.exists()
    invalid_archive = args.out / "libinvalid.rlib"
    invalid_archive.write_bytes(b"invalid archive control\n")
    _, answer = native("invalid-archive", fixture / "consumer.rs", invalid_archive, 3, native_output)
    assert b"Failed(read imported archive" in answer.stderr and native_output.read_bytes() == successful

    for name, code in [("move-rejected", b"E0382"), ("type-rejected", b"E0308"),
                       ("inherent-rejected", b"E0116"), ("orphan-rejected", b"E0117")]:
        native(name, fixture / f"{name}.rs", library, 1, native_output)
        assert native_output.read_bytes() == successful
        rejected, answer = reference(name, fixture / f"{name}.rs", library, 1)
        assert code in answer.stderr and not rejected.exists()

    local, _ = native("local-trait", fixture / "local-trait.rs", library)
    baseline, _ = reference("local-trait", fixture / "local-trait.rs", library)
    assert run("local-trait-run", [local]).stdout == run("local-trait-reference-run", [baseline]).stdout
    pointer, _ = native("function-pointer", fixture / "function-pointer.rs", library)
    baseline, _ = reference("function-pointer", fixture / "function-pointer.rs", library)
    assert run("function-pointer-run", [pointer]).stdout == run("function-pointer-reference-run", [baseline]).stdout == b"shared function pointers: 65541\n"

    # Field structure is not authority for imported auto traits: real negative
    # upstream impls must be refused before native checks the consumer.
    for trait in ["Send", "Sync", "Unpin"]:
        name = f"negative-{trait.lower()}"
        dependency = peer(name, "#![feature(negative_impls)]\n" + original + f"\nimpl !{trait} for Word {{}}\n")
        _, answer = native(name, fixture / "consumer.rs", dependency, 2, native_output)
        assert b"negative or unresolved imported" in answer.stderr and native_output.read_bytes() == successful
        reference(name, fixture / "consumer.rs", dependency)
        bound = args.out / f"{name}-bound.rs"
        bound.write_text(f"extern crate peer; fn require<T: {trait}>() {{}} fn main() {{ require::<peer::Word>(); }}\n")
        unrepresented, _ = native(f"{name}-bound", bound, dependency, 2)
        assert not unrepresented.exists()
        rejected, answer = reference(f"{name}-bound", bound, dependency, 1)
        assert b"E0277" in answer.stderr and not rejected.exists()

    # These are valid inputs whose shared bridge is not yet integrated. The
    # profile is never rewritten to make them pass.
    unrepresented, answer = native("unwind", fixture / "consumer.rs", library, 2,
                                  env={"LUSTR_QUALIFICATION_PANIC": "unwind"})
    assert b"Unintegrated" in answer.stderr and not unrepresented.exists()
    copied = peer("copy", original.replace("#[repr(transparent)]", "#[derive(Clone, Copy)]\n#[repr(transparent)]"))
    unrepresented, answer = native("copy", fixture / "consumer.rs", copied, 2)
    assert b"Unintegrated" in answer.stderr and not unrepresented.exists()
    reference("copy", fixture / "consumer.rs", copied)
    narrow = peer("narrow", original.replace("u64", "u32").replace("6364136223846793005", "1664525").replace("1442695040888963407", "1013904223"))
    narrow_source = args.out / "narrow-consumer.rs"
    narrow_source.write_text("extern crate peer; fn main(){let x=peer::step(peer::Word(7)); println!(\"{}\",x.0);}\n")
    unrepresented, answer = native("narrow", narrow_source, narrow, 2)
    assert b"Unintegrated" in answer.stderr and not unrepresented.exists()
    reference("narrow", narrow_source, narrow)
    for name, addition in [
        ("external-method", "impl Word { pub fn value(self)->u64 { self.0 } }"),
        ("external-clone", "impl Clone for Word { fn clone(&self)->Self { Self(self.0) } }")]:
        dependency = peer(name, original + "\n" + addition + "\n")
        unrepresented, answer = native(name, fixture / "consumer.rs", dependency, 2)
        assert b"Unintegrated" in answer.stderr and not unrepresented.exists()
        reference(name, fixture / "consumer.rs", dependency)

    for name in ["LUST_FRONT_STATE", "LUST_FRONT_ARENA"]:
        answer = run(f"image-guard-{name.lower()}", [args.qualifier, "--front-image-control"],
                     env={name: str(args.out / "refused-image")})
        assert b"front-image entry guards: 4 passed" in answer.stderr
    assert b"front-image client guards: 4 passed" in run("image-client-guards", [args.qualifier, "--front-client-control"]).stderr

    # Emission owns its exact imported objects, not their original pathname.
    snapshot, _ = native("removed-snapshot", fixture / "consumer.rs", library,
                         env={"LUSTR_QUALIFICATION_REMOVE_SNAPSHOT": "1"})
    assert run("removed-snapshot-run", [snapshot]).stdout == b"shared nominal calls: 65541\n"

    # The unchanged consumer sees a real dependency edit through fresh
    # authoritative metadata. This does not claim retained import reuse.
    edit_source = args.out / "edit-consumer.rs"
    edit_source.write_text("extern crate peer; fn main(){let x=peer::step(peer::Word(7)); println!(\"{}\",x.0);}\n")
    edited = peer("edited", original.replace("1442695040888963407", "1442695040888963408"))
    outputs = []
    identities = []
    for name, dependency in [("dependency-old", library), ("dependency-new", edited), ("dependency-revert", library)]:
        binary, answer = native(name, edit_source, dependency)
        identities.append(next(line for line in answer.stderr.splitlines() if line.startswith(b"imported-generation=")))
        actual = run(f"{name}-run", [binary]).stdout
        baseline, _ = reference(name, edit_source, dependency)
        assert actual == run(f"{name}-reference-run", [baseline]).stdout
        outputs.append(actual)
    assert outputs[0] == outputs[2] and outputs[0] != outputs[1]
    assert identities[0] == identities[2] and identities[0] != identities[1]

    # An actual link I/O failure preserves a prior successful executable.
    directory = args.out / "link-output"
    directory.mkdir()
    old = directory / "program"
    shutil.copyfile(native_output, old)
    old.chmod(0o755)
    directory.chmod(0o555)
    try:
        native("link-failed", fixture / "consumer.rs", library, 3, old)
        assert old.read_bytes() == successful
    finally:
        directory.chmod(0o755)
    recovered, _ = native("link-recovered", fixture / "consumer.rs", library, output=old)
    assert run("link-recovered-run", [recovered]).stdout == b"shared nominal calls: 65541\n"
    _, answer = native("object-failed", fixture / "consumer.rs", library, 3, old,
                       env={"LUSTR_QUALIFICATION_OBJECT_FAILURE": "1"})
    assert b"cannot materialize imported object" in answer.stderr and old.read_bytes() == successful
    recovered, _ = native("object-recovered", fixture / "consumer.rs", library, output=old)
    assert run("object-recovered-run", [recovered]).stdout == b"shared nominal calls: 65541\n"
    passed = True
finally:
    summary = {"schema": 1, "passed": passed, "cases": records,
               "qualifier_sha256": hashlib.sha256(args.qualifier.read_bytes()).hexdigest(),
               "reference_identity": subprocess.check_output([str(args.rustc), "-vV"]).decode(),
               "limitations": ["fresh imported sessions only", "transparent u64 non-Copy aggregates",
                   "Linux x86_64 abort profile", "no consumer Rust metadata output",
                   "normal public routing is unchanged", "no performance claim"]}
    (args.out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(f"passed {len(records)} real compiler/runtime controls")
