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

    # Actual upstream integer and nominal signatures, including signed values,
    # zero arguments, primitive returns and eight arguments crossing the ABI's
    # register/stack boundary. Each consumer comparison uses wider arithmetic.
    integer_library = peer("integer-peer", (fixture / "integer-peer.rs").read_text())
    integer_output, integer_compile = native("integer-signatures", fixture / "integer-consumer.rs", integer_library)
    assert b"consumer-rust-typeck=0 consumer-rust-analysis=0" in integer_compile.stderr
    assert b"integer-adapters=30" in integer_compile.stderr
    adapter_source = (args.out / "integer-signatures-state/integer-adapters.ll").read_text()
    assert adapter_source.count("define ") == 30
    assert "i8 zeroext noundef" in adapter_source and "i16 signext noundef" in adapter_source
    baseline, _ = reference("integer-signatures", fixture / "integer-consumer.rs", integer_library)
    assert run("integer-signatures-run", [integer_output]).stdout == run("integer-signatures-reference-run", [baseline]).stdout == b"shared integer signatures: 262720 value and pointer results\n"
    integer_successful = integer_output.read_bytes()
    for name, body, code in [
        ("integer-signedness", "peer::combine_u64(peer::U64(1), 2i64, peer::U64(3), 4, 5, 6, 7, 8);", b"E0308"),
        ("integer-arity", "peer::combine_u64(peer::U64(1), 2, peer::U64(3), 4, 5, 6, 7);", b"E0061"),
        ("integer-move", "let value = peer::U64(1); peer::combine_u64(value, 2, value, 4, 5, 6, 7, 8);", b"E0382"),
        ("integer-pointer", "let _: fn(peer::U64, i64, peer::U64, u64, u64, u64, u64, u64) -> peer::U64 = peer::combine_u64;", b"E0308"),
    ]:
        source = args.out / f"{name}.rs"
        source.write_text(f"extern crate peer; fn main() {{ {body} }}\n")
        native(name, source, integer_library, 1, integer_output)
        assert integer_output.read_bytes() == integer_successful
        rejected, answer = reference(name, source, integer_library, 1)
        assert code in answer.stderr and not rejected.exists()

    # These are actual failed llc executions or invalid emitted outputs, not a
    # compiler-selection result. Every failure preserves the prior executable.
    for failure, diagnostic in [("missing", b"start imported LLVM adapter emission"),
                                ("failed", b"imported LLVM adapter emission failed"),
                                ("malformed", b"invalid imported LLVM adapter object"),
                                ("empty", b"invalid imported LLVM adapter object")]:
        _, answer = native(f"adapter-{failure}", fixture / "integer-consumer.rs", integer_library,
                           3, integer_output, {"LUSTR_QUALIFICATION_ADAPTER_FAILURE": failure})
        assert diagnostic in answer.stderr and integer_output.read_bytes() == integer_successful
        assert b"shared request:" in answer.stderr
        assert (args.out / f"adapter-{failure}-state/integer-adapters.ll").exists()
    # Native rejection precedes adapter emission. An intentionally failed LLVM
    # command therefore cannot replace this actual move diagnostic with status 3.
    invalid = args.out / "integer-move.rs"
    native("adapter-invalid-before-emission", invalid, integer_library, 1, integer_output,
           {"LUSTR_QUALIFICATION_ADAPTER_FAILURE": "failed"})
    assert integer_output.read_bytes() == integer_successful
    integer_recovered, _ = native("adapter-recovered", fixture / "integer-consumer.rs", integer_library,
        output=integer_output, env={"LUSTR_QUALIFICATION_REMOVE_SNAPSHOT": "1"})
    assert run("adapter-recovered-run", [integer_recovered]).stdout == b"shared integer signatures: 262720 value and pointer results\n"

    # Scalar-only dependencies use the same owned fact/ABI path. No synthetic
    # nominal declaration is required to make the compiler load the service.
    scalar_library = peer("scalars", "#![no_std]\npub fn add(a:i64,b:i64)->i64{a.wrapping_add(b)}\npub fn seed()->i64{-17}\n")
    scalar_source = args.out / "scalar-consumer.rs"
    scalar_source.write_text("extern crate peer; fn main(){let f:fn(i64,i64)->i64=peer::add; let value=f(i64::MAX,peer::seed());assert_eq!(value,i64::MAX-17);println!(\"{}\",value);}\n")
    scalar_output, _ = native("scalars", scalar_source, scalar_library)
    baseline, _ = reference("scalars", scalar_source, scalar_library)
    assert run("scalars-run", [scalar_output]).stdout == run("scalars-reference-run", [baseline]).stdout
    inline_library = peer("inline-body", "#![no_std]\n#[inline(always)]pub fn add(a:i64,b:i64)->i64{a.wrapping_add(b)}\npub fn seed()->i64{-17}\n")
    _, answer = native("inline-body", scalar_source, inline_library, 2, native_output)
    assert b"requires complementary body materialization" in answer.stderr and native_output.read_bytes() == successful
    assert b"shared request:" not in answer.stderr
    baseline, _ = reference("inline-body", scalar_source, inline_library)
    assert run("inline-body-reference-run", [baseline]).stdout == run("inline-body-original-run", [scalar_output]).stdout
    checked_library = peer("checked-add", "#![no_std]\npub fn add(a:i64,b:i64)->i64{a+b}\npub fn seed()->i64{-17}\n")
    _, answer = native("checked-add", scalar_source, checked_library, 2, native_output)
    assert b"requires additional shared runtime or dependency records" in answer.stderr and native_output.read_bytes() == successful
    assert b"shared request:" not in answer.stderr
    baseline, _ = reference("checked-add", scalar_source, checked_library)
    assert run("checked-add-reference-run", [baseline]).stdout == run("checked-add-original-run", [scalar_output]).stdout

    # The real native C classifier puts this seventh nominal argument on the
    # stack by value. That pointer-mode adapter is not represented yet; it must
    # be refused before emission rather than guessed from the nominal's size.
    stacked = peer("stack-nominal", "#![no_std]\n#[repr(transparent)]pub struct Word(pub u64);\npub fn stack(a:u64,b:u64,c:u64,d:u64,e:u64,f:u64,g:Word)->Word{Word(a.wrapping_add(b).wrapping_add(c).wrapping_add(d).wrapping_add(e).wrapping_add(f).wrapping_add(g.0))}\n")
    stack_source = args.out / "stack-nominal-consumer.rs"
    stack_source.write_text("extern crate peer;fn main(){assert_eq!(peer::stack(1,2,3,4,5,6,peer::Word(7)).0,28);}\n")
    _, answer = native("stack-nominal", stack_source, stacked, 2, native_output)
    assert b"represented direct or single-integer cast modes" in answer.stderr and native_output.read_bytes() == successful
    assert b"shared request:" not in answer.stderr
    baseline, _ = reference("stack-nominal", stack_source, stacked)
    run("stack-nominal-reference-run", [baseline])

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
    represented, _ = native("narrow", narrow_source, narrow)
    baseline, _ = reference("narrow", narrow_source, narrow)
    assert run("narrow-run", [represented]).stdout == run("narrow-reference-run", [baseline]).stdout
    wide = peer("wide", original.replace("u64", "u128"))
    unrepresented, answer = native("wide", narrow_source, wide, 2)
    assert b"Unintegrated" in answer.stderr and not unrepresented.exists()
    reference("wide", narrow_source, wide)
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
               "limitations": ["fresh imported sessions only", "transparent 8/16/32/64-bit and pointer-width non-Copy integer aggregates",
                   "Linux x86_64 abort profile", "no consumer Rust metadata output",
                   "normal public routing is unchanged", "no performance claim"]}
    (args.out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(f"passed {len(records)} real compiler/runtime controls")
