#!/usr/bin/env python3
"""Qualify native-only reuse after real complete Rust recompilation.

The previous Rust analysis and objects must stay invalidated. This checks the
actual executable, the selected native objects, and last-success publication.
"""
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import sys


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def main():
    compiler = Path(sys.argv[1]).resolve(strict=True)
    work = Path(sys.argv[2]).absolute()
    work.mkdir()
    (work / "src").mkdir()
    (work / "Cargo.toml").write_text(
        "[package]\nname='native-reuse'\nversion='0.1.0'\nedition='2024'\n"
        "[profile.dev]\ndebug=0\n"
    )
    linker = work / "controlled-linker"
    linker.write_text('#!/bin/sh\nif [ -e "$(dirname "$0")/fail-link" ]; then exit 75; fi\nexec cc "$@"\n')
    linker.chmod(0o755)
    trace = work / "mixed.trace"
    environment = dict(os.environ)
    environment.update({
        "LUSTR_MIXED_TRACE": str(trace),
        "LUSTR_MIXED_CACHE_DIR": str(work / "mixed-state"),
        "LUST_CACHE_DIR": str(work / "native-cache"),
        "LUST_DAEMON_SOCKET": str(work / "native.sock"),
        "LUST_DAEMON_IDLE": "2s",
    })
    for key in ("LUST_LIBRARY", "LD_LIBRARY_PATH", "LUSTR_MIXED_PROFILE_DIR"):
        environment.pop(key, None)

    def source(literal=17, label="a"):
        return (
            f"fn kernel(x:u64)->u64{{x ^ {literal}}}\n"
            "fn stable(x:u64)->u64{x.wrapping_add(5)}\n"
            f'fn main(){{println!("{{}} {{}} {label}",kernel(42),stable(10));}}\n'
        )

    def build(label, text, *, expected=True, output=None, reuse=None, generated=None, extra=()):
        (work / "src/main.rs").write_text(text)
        trace.write_text("")
        command = [str(compiler), "build", "--offline", "--jobs", "1", "--target-dir", str(work / "target"),
                   "--config", f'target.x86_64-unknown-linux-gnu.linker={json.dumps(str(linker))}', *extra]
        result = subprocess.run(command, cwd=work, env=environment, capture_output=True, timeout=120)
        (work / f"{label}.stdout").write_bytes(result.stdout)
        (work / f"{label}.stderr").write_bytes(result.stderr)
        (work / f"{label}.status").write_text(str(result.returncode) + "\n")
        observed = trace.read_text()
        (work / f"{label}.trace").write_text(observed)
        require((result.returncode == 0) == expected, f"{label}: compiler status {result.returncode}: {result.stderr!r}")
        require((work / "src/main.rs").read_text() == text, "compiler rewrote the original source")
        if reuse is not None:
            require("reused-rust-objects=1" not in observed, f"{label}: stale Rust work was reused")
            require(observed.count("native-object-reused=") == reuse, f"{label}: wrong native reuse count: {observed}")
        if generated is not None:
            require(observed.count("lustr mixed native:") == generated, f"{label}: wrong native compilation count: {observed}")
        if output is not None:
            execution = subprocess.run([work / "target/debug/native-reuse"], capture_output=True, timeout=15)
            require(execution.returncode == 0 and execution.stdout == output, f"{label}: incorrect executable: {execution}")
        return observed

    def certificate():
        states = [(path, json.loads(path.read_text())) for path in (work / "mixed-state").glob("*/current.json")]
        states = [(path, value) for path, value in states if {k["name"] for k in value["kernels"]} == {"kernel", "stable"}]
        require(len(states) == 1, "ambiguous native reuse certificate")
        path, state = states[0]
        require(state["version"] == 5, "missing native object identity version")
        for kernel in state["kernels"]:
            canonical = (kernel["prefix"] + kernel["body"] + kernel["suffix"]).encode()
            require(kernel["native_request"]["source_sha256"] == "sha256=" + hashlib.sha256(canonical).hexdigest(),
                    "certificate did not bind complete canonical native source")
            require(kernel["native_request"]["symbol"] == kernel["symbol"], "certificate lost the Rust symbol")
            require(kernel["object_sha256"] == "sha256=" + hashlib.sha256(selected(state, kernel).read_bytes()).hexdigest(),
                    "certificate did not bind native object bytes")
        return path, state

    def selected(state, kernel):
        return Path(kernel.get("object", str(Path(state["directory"]) / kernel["identity"] / "native-link.o")))

    def snapshot(state):
        return {selected(state, kernel): selected(state, kernel).read_bytes() for kernel in state["kernels"]}

    def unchanged(saved):
        require(all(path.read_bytes() == value for path, value in saved.items()), "reuse changed a previous successful object")

    build("cold", source(), output=b"59 15 a\n", reuse=0, generated=2)
    build("literal", source(19), output=b"57 15 a\n", generated=1)
    certificate_path, prior = certificate()
    saved = snapshot(prior)
    build("rust-context", source(19, "b"), output=b"57 15 b\n", reuse=2, generated=0)
    unchanged(saved)
    certificate_path, current = certificate()
    old_inodes = {(path.stat().st_dev, path.stat().st_ino) for path in saved}
    new_inodes = {(path.stat().st_dev, path.stat().st_ino) for path in (selected(current, k) for k in current["kernels"])}
    require(old_inodes.isdisjoint(new_inodes), "new native object aliases its previous generation")

    # Change executable code without breaking ELF structure: the source and
    # export still match, so the retained SHA-256 must reject these bytes.
    kernel = next(k for k in current["kernels"] if k["name"] == "stable")
    object_path = selected(current, kernel)
    damaged = bytearray(object_path.read_bytes())
    sections, = struct.unpack_from("<Q", damaged, 40)
    count, = struct.unpack_from("<H", damaged, 60)
    for index in range(count):
        header = sections + index * 64
        kind, flags, _, offset, size = struct.unpack_from("<IQQQQ", damaged, header + 4)
        if kind == 1 and flags & 4 and size:
            damaged[offset] ^= 1
            break
    else:
        raise AssertionError("missing executable object section")
    object_path.write_bytes(damaged)
    build("damaged-object", source(19, "c"), output=b"57 15 c\n", reuse=1, generated=1)

    certificate_path, current = certificate()
    current["kernels"][0]["native_request"]["rust_abi"] += "invalid"
    certificate_path.write_text(json.dumps(current))
    build("changed-abi-record", source(19, "d"), output=b"57 15 d\n", reuse=1, generated=1)

    certificate_path, current = certificate()
    current["kernels"][0]["native_request"]["source_sha256"] = "sha256=" + "0" * 64
    certificate_path.write_text(json.dumps(current))
    build("changed-source-record", source(19, "e"), output=b"57 15 e\n", reuse=1, generated=1)

    certificate_path, current = certificate()
    saved = snapshot(current)
    certificate_bytes = certificate_path.read_bytes()
    (work / "fail-link").touch()
    build("failed-link", source(19, "f"), expected=False, reuse=2, generated=0)
    unchanged(saved)
    require(certificate_path.read_bytes() == certificate_bytes, "failed link published a new certificate")
    (work / "fail-link").unlink()
    build("link-recovery", source(19, "f"), output=b"57 15 f\n", reuse=2, generated=0)
    unchanged(saved)
    certificate()
    build("changed-profile", source(19, "g"), output=b"57 15 g\n", reuse=0, generated=2,
          extra=("--config", "profile.dev.overflow-checks=false"))
    print("complete Rust analysis, unchanged scalar object reuse, canonical source/ABI/byte identities, corruption rejection, "
          "independent generations, failed-link recovery and profile invalidation: passed")


if __name__ == "__main__":
    main()
