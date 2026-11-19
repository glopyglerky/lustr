#!/usr/bin/env python3
"""Compare the source-built native Unicode implementation with pinned Rust.

Uses direct cold compilation, no daemon, no retries and no oracle substitution.
Retained-state coverage is a separate native unit test.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

RUST_SOURCE = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494"
SCALARS = 1_112_064


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def equal_files(left, right):
    with left.open("rb") as a, right.open("rb") as b:
        while True:
            av, bv = a.read(1 << 20), b.read(1 << 20)
            if av != bv:
                return False
            if not av:
                return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-core", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    native, rustc, output = args.native_core.resolve(), args.rustc.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ, LUST_NO_CACHE="1", LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(output / "cache"), TMPDIR=str(output))
    records = []
    identities = {"native_core_sha256": sha256(native), "rustc_sha256": sha256(rustc)}

    def run(name, argv, seconds=90):
        stdout, stderr = output / f"{name}.stdout", output / f"{name}.stderr"
        with stdout.open("wb") as out, stderr.open("wb") as err:
            result = subprocess.run([str(x) for x in argv], cwd=root, env=env, stdout=out, stderr=err, timeout=seconds)
        record = {"name": name, "argv": [str(x) for x in argv], "returncode": result.returncode,
                  "stdout_bytes": stdout.stat().st_size, "stdout_sha256": sha256(stdout),
                  "stderr_bytes": stderr.stat().st_size, "stderr_sha256": sha256(stderr)}
        records.append(record)
        (output / "commands.json").write_text(json.dumps(records, indent=2) + "\n")
        if result.returncode != 0:
            raise RuntimeError(f"{name} failed with {result.returncode}; see {stderr}")
        return stdout

    identity = run("rustc-identity", [rustc, "-vV"]).read_text()
    if f"commit-hash: {RUST_SOURCE}\n" not in identity:
        raise RuntimeError("qualification requires the exact declared Rust source")
    provenance = json.loads((root / "compiler/native/unicode-semantics.json").read_text())
    if provenance["rust_source"] != RUST_SOURCE:
        raise RuntimeError("Unicode provenance names a different Rust source")
    for name, expected in provenance["generated_and_retained_data"].items():
        if sha256(root / name) != expected:
            raise RuntimeError(f"Unicode provenance digest mismatch: {name}")
    sysroot = Path(run("rustc-sysroot", [rustc, "--print", "sysroot"]).read_text().strip())
    upstream_lock = sysroot / "lib/rustlib/rustc-src/rust/Cargo.lock"
    if sha256(upstream_lock) != provenance["upstream_cargo_lock_sha256"]:
        raise RuntimeError("pinned compiler dependency lockfile differs from recorded provenance")
    env["RUSTC"] = str(rustc)
    run("data-generation-check", ["bash", root / "compiler/native/scripts/gen-unicode.sh", "--check"])
    verifier = output / "verify-compiler-data"
    run("data-verifier-build", [rustc, "--edition=2024", "-O", "-C", "rpath", root / "compiler/native/tools/verify_unicode_semantics.rs", "-o", verifier])
    run("data-verifier-run", [verifier])
    fixtures = [
        ("const-runtime", root / "compiler/native/tests/pass/unicode_const_runtime_semantics.rs"),
        ("runtime-exact", root / "compiler/native/tools/unicode_runtime_exact.rs"),
    ]
    comparisons = []
    for name, fixture in fixtures:
        outputs = []
        for implementation, compiler in [("native", native), ("rust", rustc)]:
            program = output / f"{name}-{implementation}"
            argv = [compiler, "build", fixture, "-o", program] if implementation == "native" else [compiler, "--edition=2024", "-O", fixture, "-o", program]
            run(f"{name}-{implementation}-build", argv)
            outputs.append(run(f"{name}-{implementation}-run", [program], seconds=120))
        if not equal_files(*outputs):
            raise RuntimeError(f"{name}: native output differs from pinned Rust")
        for implementation in ("native", "rust"):
            if (output / f"{name}-{implementation}-run.stderr").stat().st_size:
                raise RuntimeError(f"{name}: unexpected runtime stderr from {implementation}")
        comparisons.append({"fixture": str(fixture.relative_to(root)), "fixture_sha256": sha256(fixture),
                            "bytes": outputs[0].stat().st_size, "sha256": sha256(outputs[0]), "byte_for_byte_equal": True})
    if identities != {"native_core_sha256": sha256(native), "rustc_sha256": sha256(rustc)}:
        raise RuntimeError("compiler changed during qualification")
    result = {"schema": 1, "rust_source": RUST_SOURCE, "rustc_vV": identity, "identities": identities,
              "scalar_values": SCALARS, "comparisons": comparisons, "commands": len(records),
              "limits": ["Direct cold compilation; retained behavior is covered by the separate native test.",
                         "Unicode data and exercised operations only; not a proof of all Rust semantics.",
                         "No timing or release-admission claim."]}
    (output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
