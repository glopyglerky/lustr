#!/usr/bin/env python3
"""Process-boundary controls for the actual Lustr entry.

Synthetic children verify selection and failure propagation, not Rust semantics.
The optional native-runtime argument separately exercises a real native build.
"""
import argparse
import atexit
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import time
import subprocess
import sys
import tempfile


def executable(path, text):
    path.write_text(text)
    path.chmod(0o755)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--entry", type=Path, required=True)
    parser.add_argument("--native-runtime", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--require-daemon", action="store_true", help="fail unless real retained public daemon controls execute")
    args = parser.parse_args()
    entry = args.entry.resolve(strict=True)
    assert not args.require_daemon or args.native_runtime, "--require-daemon needs --native-runtime"
    outcomes = []
    completed = False
    entry_hash = hashlib.sha256(entry.read_bytes()).hexdigest()
    runner_hash = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    native_hashes = ({name: hashlib.sha256((args.native_runtime / "bin" / name).read_bytes()).hexdigest()
                     for name in ["lust", "lust-core"]} if args.native_runtime else None)

    def save():
        report = {"schema": "lustr-exclusive-entry-controls-v2", "status": "completed" if completed else "failed",
                  "entry_sha256": entry_hash,
                  "runner_sha256": runner_hash,
                  "controls": outcomes, "daemon_required": args.require_daemon,
                  "scope": "synthetic entry failures and named real native witnesses; no whole-Rust or speed qualification",
                  "full_rust_coverage": False}
        if native_hashes is not None:
            report["native_helper_sha256"] = native_hashes
        if args.output:
            args.output.write_text(json.dumps(report, indent=2) + "\n")

    atexit.register(save)
    with tempfile.TemporaryDirectory(prefix="lustr-exclusive-") as temporary:
        root = Path(temporary)
        package = root / "package"
        binary = package / "bin"
        binary.mkdir(parents=True)
        shutil.copy2(entry, binary / "lustr")
        (binary / "rustc").symlink_to("lustr")
        source = root / "main.rs"
        source.write_text("fn main() { println!(\"42\"); }\n")
        log = root / "executed"
        trap = '#!/bin/sh\nprintf "%s\\n" "FORBIDDEN:$0" >> "$TEST_LOG"\nexit 0\n'
        for name in ["rustc-full", "rustc-reference", "lustr-native-worker", "cargo"]:
            executable(binary / name, trap)
        # The old diagnostic retry path required this source-library marker.
        marker = package / "lib/lustr/std/src/platform.rs"
        marker.parent.mkdir(parents=True)
        marker.write_text("// Test-only old expansion marker.\n")
        env = dict(os.environ)
        env.update(LUST_NO_DAEMON="1", TEST_LOG=str(log), RUSTC=str(binary / "rustc-full"))
        for name in ["LUST_LIBRARY", "LUST_SOURCES_READY", "LUSTR_TRACE_PATH", "LUSTR_NATIVE_DIAGNOSTICS", "LUST_PROBE_FAKE_DIR", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "MIMALLOC_PURGE_DELAY", "LUST_DAEMON_LOCK_FD"]:
            env.pop(name, None)
        env["LUST_CACHE_DIR"] = str(root / "cache")

        def run(label, arguments, expected, expected_native, diagnostic=None):
            log.write_text("")
            result = subprocess.run([str(binary / "lustr"), *map(str, arguments)], env=env, cwd=root, capture_output=True)
            events = log.read_text().splitlines()
            assert result.returncode == expected, (label, result.returncode, result.stderr.decode(errors="replace"))
            assert events == expected_native, (label, events)
            if diagnostic is not None:
                assert diagnostic.encode() in result.stderr, (label, result.stderr)
            outcomes.append({"case": label, "status": result.returncode, "children": events})
            return result

        for status in [0, 1, 2, 7, 101]:
            executable(binary / "lust", f'#!/bin/sh\necho native >> "$TEST_LOG"\necho "std::env::consts:: deliberate native result" >&2\nexit {status}\n')
            run(f"native-status-{status}", ["build", source], status, ["native"], "deliberate native result")
        executable(binary / "lust", '#!/bin/sh\necho native >> "$TEST_LOG"\nkill -TERM $$\n')
        run("native-signal", ["check", source], -signal.SIGTERM, ["native"])
        for label, arguments in [
            ("release-delegation", ["build", source, "--release"]),
            ("oracle-delegation", ["check", source, "--rustc"]),
            ("profile-delegation", ["build", source, "--profile", "release"]),
            ("metadata-complement", ["--edition=2024", source, "--emit=metadata"]),
            ("engine-rustc", ["--engine=rustc", "check"]),
            ("engine-cranelift", ["--engine", "cranelift", "build"]),
            ("toolchain-selection", ["+stable", "build"]),
            ("bluefe-selection", ["--bluefe-root", package, "check"]),
            ("cargo-build", ["build", "--workspace"]),
        ]:
            run(label, arguments, 2, [], "integration is incomplete")
        executable(binary / "lust-core", '#!/bin/sh\necho native-core >> "$TEST_LOG"\nexit 0\n')
        run("native-test-retained", ["test", source], 0, ["native-core"])
        run("native-release-cc-retained", ["build", source, "--release", "--target", "x86_64-unknown-linux-gnu"], 0, ["native-core"])
        env["RUSTC_WRAPPER"] = str(binary / "rustc-full")
        run("configured-wrapper-refused", ["check", source], 2, [], "configured external compiler wrapper")
        env.pop("RUSTC_WRAPPER")
        for value in ["", str(root / "probe")]:
            env["LUST_PROBE_FAKE_DIR"] = value
            for command in ["build", "test"]:
                run(f"external-probe-mode-{command}-{'empty' if not value else 'set'}", [command, source], 2, [], "internal compiler-probe mode")
        env.pop("LUST_PROBE_FAKE_DIR")
        saved_env = env
        snapshot = root / "child-environment.json"
        env = {"PATH": os.defpath, "HOME": str(root), "LANG": "C", "LC_ALL": "C", "PYTHONCOERCECLOCALE": "0",
               "LUST_NO_DAEMON": "1", "LUST_CACHE_DIR": str(root / "cache"), "TEST_LOG": str(log),
               "TEST_ENVFILE": str(snapshot), "RUSTC": "caller-rustc-value", "RUSTC_WRAPPER": "", "CUSTOM_EMPTY": "",
               "CUSTOM_CONTEXT": "caller-context", "__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT": "0"}
        executable(binary / "lust", f'#!{sys.executable}\nimport json,os\nwith open(os.environ["TEST_ENVFILE"],"w") as f: json.dump(dict(os.environ),f)\nwith open(os.environ["TEST_LOG"],"a") as f: f.write("native\\n")\n')
        run("complete-cold-child-environment-preserved", ["check", source], 0, ["native"])
        actual = json.loads(snapshot.read_text())
        assert actual == env, "native process environment changed: " + ",".join(sorted(key for key in actual.keys() | env.keys() if actual.get(key) != env.get(key)))
        env = saved_env
        (binary / "lust").unlink()
        run("native-spawn-failure-is-terminal", ["check", source], 2, [], "cannot execute")
        executable(binary / "lust", '#!/bin/sh\necho native >> "$TEST_LOG"\nexit 0\n')
        library = package / "lib" / ("liblustr_native_core.dylib" if sys.platform == "darwin" else "liblustr_native_core.so")
        library.write_bytes(b"invalid dynamic library")
        run("native-library-identity-failure-is-terminal", ["check", source], 2, [], "native compiler identity is missing")
        (package / "lib/lustr-native.id").write_text("0" * 64 + "\n")
        run("native-library-load-failure-is-terminal", ["check", source], 2, [], "cannot load")
        library.unlink()
        (package / "lib/lustr-native.id").unlink()
        # Resolve a daemon endpoint through a regular file: transport admission
        # fails before any compiler is launched, on socket-restricted hosts too.
        blocked = root / "not-a-directory"
        blocked.write_text("transport sentinel")
        env.pop("LUST_NO_DAEMON")
        env["LUST_DAEMON_SOCKET"] = str(blocked / "socket")
        run("native-transport-failure-is-terminal", ["check", source], 3, [])
        env["LUST_NO_DAEMON"] = "1"
        env.pop("LUST_DAEMON_SOCKET")
        # A query carrying compilation options must never become a successful
        # no-op, and target queries must not advertise unimplemented targets.
        for flag in [["--emit=obj"], ["-o", str(root / "unwritten")], [str(source)], ["--target", "wasm32-unknown-unknown"]]:
            run("query-refuses-" + flag[0], ["--print=cfg", *flag], 2, [], "integration is incomplete")
        run("query-refuses-source-dependent-name", ["--print=crate-name", "-"], 2, [], "explicit crate name")
        log.write_text("")
        query = subprocess.run([str(binary / "lustr"), "--print=crate-name", "--crate-name", "named", "-"], input=b'#![crate_name="source"]\n', env=env, cwd=root, capture_output=True)
        assert query.returncode == 2 and not log.read_text() and not query.stdout
        outcomes.append({"case": "query-refuses-nonempty-source", "status": 2})
        manifest = root / "Cargo.toml"
        manifest.write_text('[package]\nname="host-probe"\nversion="0.1.0"\nedition="2024"\n')
        for label, dependencies, kinds, members in [
            ("dependency-host-probe", [{"name": "dependency"}], ["bin"], ["p"]),
            ("build-script-host-probe", [], ["custom-build"], ["p"]),
            ("proc-macro-host-probe", [], ["proc-macro"], ["p"]),
            ("workspace-host-probe", [], ["bin"], ["p", "q"]),
        ]:
            metadata = {"workspace_members": members, "packages": [{"id": "p", "dependencies": dependencies, "targets": [{"kind": kinds}]}]}
            payload = json.dumps(metadata)
            executable(binary / "cargo", '#!/bin/sh\necho metadata >> "$TEST_LOG"\ncat <<\'JSON\'\n' + payload + '\nJSON\n')
            run(label, ["check", source, "--manifest-path", manifest], 2, ["metadata"], "native Cargo integration is incomplete")
        manifest.unlink()

        if args.native_runtime:
            native = args.native_runtime.resolve(strict=True)
            for name in ["lust", "lust-core", "cargo"]:
                (binary / name).unlink()
                if name == "cargo":
                    (binary / name).symlink_to(native / "bin" / name)
                else:
                    shutil.copy2(native / "bin" / name, binary / name)
            (package / "lib/lust").symlink_to(native / "lib/lust", target_is_directory=True)
            artifact = root / "native-output"
            log.write_text("")
            result = subprocess.run([str(binary / "lustr"), "build", str(source), "-o", str(artifact)], env=env, cwd=root, capture_output=True)
            assert result.returncode == 0, result.stderr.decode(errors="replace")
            program = subprocess.run([str(artifact)], capture_output=True)
            assert (program.returncode, program.stdout) == (0, b"42\n")
            assert not log.read_text(), "stock compiler was used for the native witness"
            outcomes.append({"case": "real-native-build-and-run", "status": 0, "stdout": "42\n"})
            result = subprocess.run([str(binary / "lustr"), "run", str(source)], env=env, cwd=root, capture_output=True)
            assert (result.returncode, result.stdout) == (0, b"42\n"), result.stderr.decode(errors="replace")
            assert not log.read_text()
            outcomes.append({"case": "real-native-run-once", "status": 0})
            source.write_text("#[test] fn passes() { assert_eq!(2 + 2, 4); }\n")
            result = subprocess.run([str(binary / "lustr"), "test", str(source), "--json"], env=env, cwd=root, capture_output=True)
            assert result.returncode == 0, result.stderr.decode(errors="replace")
            assert not log.read_text()
            outcomes.append({"case": "real-native-test", "status": 0})
            result = subprocess.run([str(binary / "lustr"), "test", str(source), "--rustc"], env=env, cwd=root, capture_output=True)
            assert result.returncode == 2 and not log.read_text(), result.stderr.decode(errors="replace")
            outcomes.append({"case": "real-native-test-rejects-oracle-flag", "status": 2})
            # The packaged helpers themselves carry the same no-delegation
            # boundary; invoking them directly must not restore an old escape.
            for helper in ["lust", "lust-core"]:
                for flags in [["build", source, "--release"], ["check", source, "--rustc"]]:
                    result = subprocess.run([str(binary / helper), *map(str, flags)], env=env, cwd=root, capture_output=True)
                    assert result.returncode == 2 and not log.read_text(), (helper, flags, result.stderr)
                    outcomes.append({"case": "real-helper-refuses-" + helper + flags[-1], "status": 2})
                for value in ["", str(root / "probe")]:
                    result = subprocess.run([str(binary / helper), "build", str(source)], env=dict(env, LUST_PROBE_FAKE_DIR=value), cwd=root, capture_output=True)
                    assert result.returncode == 2 and not log.read_text(), (helper, result.stderr)
                    outcomes.append({"case": "real-helper-probe-" + helper + ("-set" if value else "-empty"), "status": 2})

            # Preserve the caller's compile-time environment through both the
            # direct and retained paths. Tool steering belongs to subprocesses.
            semantic = root / "environment.rs"
            semantic_names = ["RUSTC", "RUSTC_WRAPPER", "__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "MIMALLOC_PURGE_DELAY", "LUST_DAEMON_LOCK_FD"]
            semantic.write_text('fn main() { println!("' + ':'.join(['{}'] * len(semantic_names)) + '",' + ','.join(
                'option_env!("' + name + '").unwrap_or("unset")' for name in semantic_names) + '); }\n')

            def semantic_output(context):
                return (':'.join(context.get(name, "unset") for name in semantic_names) + '\n').encode()

            for mode in ["set", "unset"]:
                context = dict(env)
                if mode == "unset":
                    context.pop("RUSTC", None)
                result = subprocess.run([str(binary / "lustr"), "build", str(semantic), "-o", str(artifact)], env=context, cwd=root, capture_output=True)
                assert result.returncode == 0, result.stderr
                program = subprocess.run([str(artifact)], capture_output=True)
                expected = semantic_output(context)
                assert program.returncode == 0 and program.stdout == expected and not log.read_text(), program
                outcomes.append({"case": "real-direct-semantic-environment-" + mode, "status": 0})

            # Cargo resolves the real feature graph, while only native bin
            # artifacts are advertised by the information query contract.
            cargo_project = root / "cargo-project"
            (cargo_project / "src").mkdir(parents=True)
            cargo_manifest = cargo_project / "Cargo.toml"
            cargo_manifest.write_text('[package]\nname="native-cargo-control"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[features]\ndefault=["nested"]\nnested=["leaf"]\nleaf=[]\n')
            cargo_source = cargo_project / "src/main.rs"
            cargo_source.write_text('#[cfg(not(feature="leaf"))] compile_error!("feature graph lost");\nfn main() { println!("42"); }\n')
            context = dict(env, RUSTC=str(binary / "lustr"), CARGO_NET_OFFLINE="true")
            metadata = subprocess.run([str(binary / "cargo"), "metadata", "--format-version=1", "--manifest-path", str(cargo_manifest)], env=context, cwd=cargo_project, capture_output=True)
            assert metadata.returncode == 0, metadata.stderr
            nodes = json.loads(metadata.stdout)["resolve"]["nodes"]
            assert set(nodes[0]["features"]) == {"default", "nested", "leaf"}, nodes
            result = subprocess.run([str(binary / "lustr"), "build", str(cargo_source), "--manifest-path", str(cargo_manifest), "--offline", "-o", str(artifact)], env=env, cwd=cargo_project, capture_output=True)
            assert result.returncode == 0 and not log.read_text(), result.stderr
            program = subprocess.run([str(artifact)], capture_output=True)
            assert (program.returncode, program.stdout) == (0, b"42\n"), program
            outcomes.append({"case": "real-native-cargo-feature-graph", "status": 0, "features": sorted(nodes[0]["features"])})
            cargo_source.write_text(semantic.read_text())
            for layout in [None, "0"]:
                context = dict(env)
                if layout is not None:
                    context["__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT"] = layout
                result = subprocess.run([str(binary / "lustr"), "build", str(cargo_source), "--manifest-path", str(cargo_manifest), "--offline", "-o", str(artifact)], env=context, cwd=cargo_project, capture_output=True)
                assert result.returncode == 0 and not log.read_text(), result.stderr
                program = subprocess.run([str(artifact)], capture_output=True)
                assert program.returncode == 0 and program.stdout == semantic_output(context), program
                outcomes.append({"case": "real-native-cargo-environment-" + ("unset" if layout is None else "custom"), "status": 0})

            # Exercise the actual packaged library, separately from the raw
            # helper process witnesses above. A failed library request is final.
            native_library = native / "lib" / library.name
            assert native_library.is_file(), "native runtime needs its constructed compiler library"
            library.symlink_to(native_library)
            (package / "lib/lustr-native.id").symlink_to(native / "lib/lustr-native.id")
            trace = root / "library-trace.jsonl"
            context = dict(env, LUSTR_TRACE_PATH=str(trace))
            source.write_text('fn main() { println!("42"); }\n')
            for label, text, expected_status in [
                ("build", source.read_text(), 0),
                ("reject", 'fn main() { let value: i32 = "wrong"; }\n', 1),
            ]:
                source.write_text(text)
                result = subprocess.run([str(binary / "lustr"), "build", str(source), "-o", str(artifact)], env=context, cwd=root, capture_output=True)
                assert result.returncode == expected_status and not log.read_text(), result.stderr
                program = subprocess.run([str(artifact)], capture_output=True)
                assert (program.returncode, program.stdout) == (0, b"42\n"), program
                outcomes.append({"case": "real-native-library-" + label, "status": expected_status})
            library_events = [json.loads(line) for line in trace.read_text().splitlines()]
            assert len(library_events) == 2 and all(event["entry"] == "native-library" for event in library_events), library_events
            library.unlink()
            (package / "lib/lustr-native.id").unlink()

            daemon_socket = root / "retained.sock"
            try:
                probe = socket.socket(socket.AF_UNIX)
                probe.bind(str(daemon_socket))
                probe.close()
                daemon_socket.unlink()
            except OSError as error:
                assert not args.require_daemon, f"required retained daemon cannot bind: {error}"
                outcomes.append({"case": "real-retained-daemon", "status": "unavailable", "reason": error.strerror})
            else:
                context = dict(env, LUST_DAEMON_SOCKET=str(daemon_socket), LUST_DAEMON_IDLE="10")
                context.pop("LUST_NO_DAEMON", None)
                trace = root / "retained-trace.jsonl"
                context["LUSTR_TRACE_PATH"] = str(trace)
                daemon = subprocess.Popen([str(binary / "lust-core"), "daemon"], env=context, cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
                try:
                    deadline = time.monotonic() + 10
                    while not daemon_socket.exists() and daemon.poll() is None and time.monotonic() < deadline:
                        time.sleep(0.02)
                    assert daemon_socket.exists() and daemon.poll() is None, "owned daemon failed to start"
                    original = 'fn main() { println!("42"); }\n'
                    source.write_text(original)
                    for label, text, status, expected in [
                        ("initial", original, 0, b"42\n"),
                        ("body-edit", original.replace('"42"', '"43"'), 0, b"43\n"),
                        ("reject", 'fn main() { let value: i32 = "wrong"; }\n', 1, b"43\n"),
                        ("revert", original, 0, b"42\n"),
                    ]:
                        source.write_text(text)
                        result = subprocess.run([str(binary / "lustr"), "build", str(source), "-o", str(artifact)], env=context, cwd=root, capture_output=True)
                        assert result.returncode == status, (label, result.stderr)
                        program = subprocess.run([str(artifact)], capture_output=True)
                        assert program.returncode == 0 and program.stdout == expected and not log.read_text(), (label, program)
                        assert daemon.poll() is None, "retained compiler terminated"
                        outcomes.append({"case": "real-retained-" + label, "status": status})
                    result = subprocess.run([str(binary / "lustr"), "build", str(semantic), "-o", str(artifact)], env=context, cwd=root, capture_output=True)
                    assert result.returncode == 0, result.stderr
                    program = subprocess.run([str(artifact)], capture_output=True)
                    assert program.stdout == semantic_output(context) and not log.read_text()
                    for layout in [None, "0"]:
                        request_context = dict(context)
                        if layout is not None:
                            request_context["__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT"] = layout
                        result = subprocess.run([str(binary / "lustr"), "build", str(cargo_source), "--manifest-path", str(cargo_manifest), "--offline", "-o", str(artifact)], env=request_context, cwd=cargo_project, capture_output=True)
                        assert result.returncode == 0 and not log.read_text(), result.stderr
                        program = subprocess.run([str(artifact)], capture_output=True)
                        assert program.returncode == 0 and program.stdout == semantic_output(request_context), program
                        outcomes.append({"case": "real-retained-cargo-environment-" + ("unset" if layout is None else "custom"), "status": 0})
                    events = [json.loads(line) for line in trace.read_text().splitlines()]
                    assert len(events) == 7 and all(event["entry"] == "daemon" for event in events), events
                    outcomes.append({"case": "real-retained-semantic-environment", "status": 0, "retained_responses": len(events)})
                finally:
                    daemon.terminate()
                    try:
                        daemon.communicate(timeout=5)
                    except subprocess.TimeoutExpired:
                        daemon.kill()
                        daemon.communicate()
    completed = True
    save()
    atexit.unregister(save)
    unavailable = sum(item["status"] == "unavailable" for item in outcomes)
    print(json.dumps({"passed": len(outcomes) - unavailable, "unavailable": unavailable, "full_rust_coverage": False}))


if __name__ == "__main__":
    main()
