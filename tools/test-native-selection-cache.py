#!/usr/bin/env python3
"""Exercise the public package-selector component against the unchanged Cargo path.

This is a selector regression gate, not a native compiler or performance gate.
The supplied profiler includes the production parser, selector, and child owner.
Ordinary manifest requests now bind in the retained native resolver; this
component remains the public binder for explicit Cargo package specifications.
Run it with a matched package containing real Cargo and Rust executables.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profiler", type=Path, required=True)
    parser.add_argument("--package-bin", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    profiler = args.profiler.resolve(strict=True)
    package_bin = args.package_bin.resolve(strict=True)
    root = args.output.absolute()
    root.mkdir(parents=True, exist_ok=False)
    project = root / "project"
    (project / "src").mkdir(parents=True)
    for directory in ["home", "cargo-home", "tmp", "native-cache", "caller"]:
        (root / directory).mkdir()
    environment = {
        "PATH": str(package_bin) + os.pathsep + "/usr/local/bin:/usr/bin:/bin",
        "HOME": str(root / "home"),
        "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
        "CARGO_HOME": str(root / "cargo-home"),
        "CARGO_TARGET_DIR": str(root / "target"),
        "CARGO_BUILD_BUILD_DIR": str(root / "intermediates"),
        "CARGO_NET_OFFLINE": "true", "CARGO_TERM_COLOR": "never",
        "TMPDIR": str(root / "tmp"),
        "LUST_CACHE_DIR": str(root / "native-cache"),
        "LUSTR_SELECTION_DIAGNOSTICS": str(root / "selection-diagnostics.jsonl"),
        "LUSTR_PROFILE_SELECTION_DETAIL": "1",
    }
    manifest = project / "Cargo.toml"
    source = project / "src/main.rs"
    other = project / "src/other.rs"
    manifest_text = ('[package]\nname="selection-cache-regression"\nversion="0.1.0"\n'
                     'edition="2024"\n[workspace]\n[[bin]]\nname="first"\npath="src/main.rs"\n'
                     '[[bin]]\nname="second"\npath="src/other.rs"\n')
    manifest.write_text(manifest_text)
    source.write_text("fn main() {}\n")
    other.write_text("fn main() {}\n")
    native_args = ["check", str(source), "--manifest-path", str(manifest), "--offline", "--locked", "--bin", "first"]
    records = []
    assertions = []
    sequence = 0
    completed = False
    failure = None

    def save():
        (root / "results.json").write_text(json.dumps({
            "schema": "lustr-selection-cache-regression-v1",
            "scope": "actual public Cargo target selection only; no native compile or performance claim",
            "profiler_sha256": hashlib.sha256(profiler.read_bytes()).hexdigest(),
            "records": records, "assertions": assertions,
            "status": "failed" if failure is not None else "completed" if completed else "incomplete",
            "error": failure,
            "success": completed and failure is None and all(item["passed"] for item in assertions),
        }, indent=2) + "\n")

    def require(condition, description):
        assertions.append({"description": description, "passed": bool(condition)})
        save()
        if not condition:
            raise AssertionError(description)

    def invoke(label, *, cwd=project, extra_env=None, selected_args=None, bin_path=package_bin, uncached=False, timeout=30):
        nonlocal sequence
        sequence += 1
        env = dict(environment)
        env.update(extra_env or {})
        if uncached:
            env["LUSTR_NO_SELECTION_CACHE"] = "1"
        diagnostic = Path(env["LUSTR_SELECTION_DIAGNOSTICS"])
        diagnostic.write_text("")
        command = [str(profiler), "selection", str(bin_path)] + (native_args if selected_args is None else selected_args)
        prefix = f"{sequence:03d}-{label}"
        try:
            completed = subprocess.run(command, cwd=cwd, env=env, capture_output=True, timeout=timeout)
        except subprocess.TimeoutExpired as error:
            stdout, stderr = error.stdout or b"", error.stderr or b""
            (root / (prefix + ".stdout")).write_bytes(stdout)
            (root / (prefix + ".stderr")).write_bytes(stderr)
            records.append({"label": label, "sequence": sequence, "argv": command, "cwd": str(cwd),
                            "uncached": uncached, "exit": None, "status": "timeout", "timeout_seconds": timeout,
                            "diagnostics": diagnostic.read_text(), "stdout_file": prefix + ".stdout",
                            "stderr_file": prefix + ".stderr", "stdout_sha256": hashlib.sha256(stdout).hexdigest(),
                            "stderr_sha256": hashlib.sha256(stderr).hexdigest()})
            save()
            raise
        (root / (prefix + ".stdout")).write_bytes(completed.stdout)
        (root / (prefix + ".stderr")).write_bytes(completed.stderr)
        events = [json.loads(line) for line in diagnostic.read_text().splitlines()]
        decoded = json.loads(completed.stdout)
        semantic = {key: value for key, value in decoded.items() if key != "elapsed_ns"}
        if decoded.get("success"):
            selected = decoded["selection"]
            assert selected["manifest"] == decoded["manifest"] and selected["name"] == decoded["target"]
            assert selected["package"]["id"] and selected["target"]["kind"] and selected["target"]["src_path"]
        record = {"label": label, "sequence": sequence, "argv": command, "cwd": str(cwd),
                  "uncached": uncached, "exit": completed.returncode, "selection": semantic,
                  "diagnostics": events, "stdout_file": prefix + ".stdout", "stderr_file": prefix + ".stderr",
                  "stdout_sha256": hashlib.sha256(completed.stdout).hexdigest(),
                  "stderr_sha256": hashlib.sha256(completed.stderr).hexdigest()}
        records.append(record)
        save()
        return record, completed.stderr

    def pair(label, *, expected=0, event=None, **kwargs):
        candidate, candidate_stderr = invoke(label, **kwargs)
        reference, reference_stderr = invoke(label + "-uncached", uncached=True, **kwargs)
        require(candidate["exit"] == reference["exit"] == expected, label + " preserves exit status")
        require(candidate["selection"] == reference["selection"], label + " preserves target identity or rejection")
        require(candidate_stderr == reference_stderr, label + " preserves exact Cargo error output")
        if event is not None:
            allowed = (event,) if isinstance(event, str) else event
            require(len(candidate["diagnostics"]) == 1 and
                    candidate["diagnostics"][0].get("selection_cache") in allowed,
                    label + " observes " + " or ".join(allowed))
        return candidate

    def warm(label="warm", **kwargs):
        pair(label + "-seed", **kwargs)
        return pair(label, event="hit", **kwargs)

    try:
        setup = subprocess.run([str(package_bin / "cargo"), "generate-lockfile", "--offline", "--manifest-path", str(manifest)],
                               cwd=project, env=environment, capture_output=True, timeout=30)
        (root / "setup.stdout").write_bytes(setup.stdout)
        (root / "setup.stderr").write_bytes(setup.stderr)
        require(setup.returncode == 0, "real Cargo lockfile setup succeeds")
        warm("unchanged-request")
        source.write_text("fn main() { let _value = 2 + 3; }\n")
        pair("source-body-edit", event="hit")

        caller = root / "caller"
        warm("caller-directory", cwd=caller)
        (caller / ".cargo").mkdir()
        caller_config = caller / ".cargo/config.toml"
        caller_config.write_text("malformed [\n")
        pair("new-caller-config", cwd=caller, expected=2, event="ineligible")
        pair("manifest-directory-does-not-inherit-caller", event="hit")
        caller_config.unlink()

        for filename in ["config.toml", "config"]:
            path = root / "cargo-home" / filename
            path.write_text("malformed [\n")
            pair("cargo-home-" + filename, expected=2, event="ineligible")
            path.unlink()
        warm("config-removed")

        lock = project / "Cargo.lock"
        original_lock = lock.read_bytes()
        original_stat = lock.stat()
        # --no-deps metadata does not parse Cargo.lock, but its changed bytes
        # still invalidate our cache and must agree with a fresh Cargo lookup.
        lock.write_bytes(b"x" * len(original_lock))
        os.utime(lock, ns=(original_stat.st_atime_ns, original_stat.st_mtime_ns))
        pair("same-size-mtime-lock-corruption", event="miss")
        lock.write_bytes(original_lock)
        warm("lock-restored")

        manifest.write_text(manifest_text.replace('name="first"', 'name="renamed"'))
        pair("selected-target-renamed", expected=2, event="miss")
        manifest.write_text(manifest_text)
        warm("manifest-restored")

        (project / "build.rs").write_text("fn main() {}\n")
        pair("new-build-script-selection", event="miss")
        pair("build-script-selection-remains-ineligible-for-store", event="miss")
        (project / "build.rs").unlink()
        warm("build-script-removed")

        (project / "src/bin").mkdir()
        discovered = project / "src/bin/discovered.rs"
        discovered.write_text("fn main() {}\n")
        pair("new-discovered-target", event="miss")
        warm("discovered-target-retained")
        discovered.unlink()
        pair("discovered-target-removed", event="miss")
        warm("discovery-restored")

        without_bin = native_args[:-2]
        warm("unqualified-target", selected_args=without_bin)
        other.unlink()
        other.symlink_to("main.rs")
        pair("other-target-symlink-creates-ambiguity", selected_args=without_bin, expected=2, event="miss")
        pair("explicit-bin-resolves-symlink-alias", event="miss")
        other.unlink()
        other.write_text("fn main() {}\n")
        warm("target-alias-restored")

        package_lock = root / "cargo-home/.package-cache"
        saved_package_lock = root / "cargo-home/.package-cache.saved"
        package_lock.rename(saved_package_lock)
        package_lock.mkdir()
        pair("package-lock-replaced-by-directory", event="ineligible")
        package_lock.rmdir()
        saved_package_lock.rename(package_lock)
        warm("package-lock-restored")

        pair("unknown-cargo-option", extra_env={"CARGO_UNSTABLE_REVIEW_SENTINEL": "1"}, event="ineligible")
        pair("environment-context-change", extra_env={"SELECTION_REVIEW_CONTEXT": "changed"}, event="miss")
        warm("environment-context-retained", extra_env={"SELECTION_REVIEW_CONTEXT": "changed"})
        warm("original-context-retained")

        def corrupt_entries(kind):
            entries = [entry for entry in (root / "native-cache/cargo-selection-v1").glob("*.json")
                       if not entry.name.startswith("tools-")]
            require(bool(entries), kind + " has retained records to corrupt")
            for entry in entries:
                if kind == "malformed-json":
                    entry.write_text("{corrupt\n")
                elif kind == "checksum-mismatch":
                    try:
                        record = json.loads(entry.read_text())
                    except json.JSONDecodeError:
                        # An unused context may still hold the preceding malformed
                        # record. Corrupt the regenerated current context below.
                        continue
                    record["payload"]["selection"]["package"]["id"] = "corrupted-package-id"
                    entry.write_text(json.dumps(record))
                else:
                    entry.chmod(0o666)

        for corruption in ["malformed-json", "checksum-mismatch", "nonprivate-entry"]:
            corrupt_entries(corruption)
            pair(corruption, event="miss")
            warm(corruption + "-recovered")

        manifest.write_text(manifest_text + "[features]\noptional=[]\n")
        pair("unadmitted-manifest-syntax", event="ineligible")
        manifest.write_text(manifest_text)
        warm("admitted-manifest-restored")

        # A separate package projection permits tool replacement without touching
        # the caller's actual toolchain. Libraries remain identical symlinks.
        projection = root / "package"
        projection_bin = projection / "bin"
        projection_bin.mkdir(parents=True)
        projection_lib = projection / "lib"
        projection_lib.mkdir()
        for original in (package_bin.parent / "lib").iterdir():
            if original.name != "lustr-selection-verified":
                (projection_lib / original.name).symlink_to(original, target_is_directory=original.is_dir())
        for name in ["cargo", "lustr", "rustc"]:
            original = package_bin / name
            if not original.exists():
                continue
            if name == "cargo":
                shutil.copy2(original.resolve(), projection_bin / name)
            else:
                (projection_bin / name).symlink_to(original)
        warm("projected-tools", bin_path=projection_bin)
        replacement = projection_bin / "cargo.replacement"
        shutil.copy2(projection_bin / "cargo", replacement)
        replacement.replace(projection_bin / "cargo")
        pair("cargo-replaced-identically", bin_path=projection_bin, event="miss")
        warm("replacement-tools-retained", bin_path=projection_bin)
        mode = (projection_bin / "cargo").stat().st_mode
        (projection_bin / "cargo").chmod(mode & ~0o222)
        warm("cargo-readonly-executable", bin_path=projection_bin)
        (projection_bin / "cargo").chmod(mode & ~0o111)
        pair("cargo-execute-permission-removed", bin_path=projection_bin, expected=2, event="ineligible")
        (projection_bin / "cargo").chmod(mode)
        warm("cargo-execute-permission-restored", bin_path=projection_bin)

        # Rust query identity is the package's native entry. Replacing its
        # bytes invalidates certificates without depending on an SDK driver.
        query = projection_bin / "lustr"
        query_source = query.resolve()
        query.unlink()
        shutil.copy2(query_source, query)
        rustc_alias = projection_bin / "rustc"
        rustc_alias.unlink()
        rustc_alias.symlink_to("lustr")
        pair("native-query-replaced-identically", bin_path=projection_bin, event="miss")
        warm("native-query-replacement-retained", bin_path=projection_bin)
        with query.open("ab") as stream:
            stream.write(b"native-query-mutated")
        pair("native-query-bytes-changed", bin_path=projection_bin, event=("miss", "ineligible"))
        shutil.copy2(query_source, query)
        warm("native-query-bytes-restored", bin_path=projection_bin)

        # These paths are controlled copies or symlinks in the projection. A
        # cache preflight must not wait for a FIFO writer before Cargo executes.
        projected_cargo = projection_bin / "cargo"
        saved_cargo = projection_bin / "cargo.regular"
        projected_cargo.rename(saved_cargo)
        try:
            os.mkfifo(projected_cargo, 0o700)
            pair("cargo-replaced-by-fifo", bin_path=projection_bin, expected=2, event="ineligible", timeout=3)
        finally:
            projected_cargo.unlink(missing_ok=True)
            saved_cargo.rename(projected_cargo)
        warm("cargo-fifo-removed", bin_path=projection_bin)

        projected_receipt = projection_lib / "lustr-selection-tools.txt"
        receipt_target = projected_receipt.readlink()
        projected_receipt.unlink()
        try:
            os.mkfifo(projected_receipt, 0o600)
            pair("tool-receipt-replaced-by-fifo", bin_path=projection_bin, event="ineligible", timeout=3)
        finally:
            projected_receipt.unlink(missing_ok=True)
            projected_receipt.symlink_to(receipt_target)

        # A native executable header is not proof that this is the qualified
        # Cargo implementation. This delegate reads an external file that is
        # deliberately absent from Cargo's admitted selection inputs.
        compiler = shutil.which("cc")
        require(compiler is not None, "C compiler exists for unknown-native-tool control")
        external = root / "external-tool-state"
        wrapper_source = root / "unknown-cargo.c"
        wrapper_source.write_text(
            '#include <unistd.h>\n#include <stdio.h>\n'
            'int main(int argc, char **argv) { (void)argc; if (access('
            + json.dumps(str(external)) + ', F_OK) == 0) {'
            'fputs("external tool state rejects metadata\\n", stderr); return 2; } '
            'argv[0]=' + json.dumps(str(package_bin / "cargo"))
            + '; execv(argv[0], argv); return 127; }\n')
        compiled = subprocess.run([compiler, str(wrapper_source), "-O2", "-o", str(projection_bin / "cargo")],
                                  capture_output=True, timeout=30)
        (root / "unknown-cargo-build.stdout").write_bytes(compiled.stdout)
        (root / "unknown-cargo-build.stderr").write_bytes(compiled.stderr)
        require(compiled.returncode == 0, "unknown native Cargo delegate compiles")
        for index in range(3):
            pair("unknown-native-cargo-" + str(index), bin_path=projection_bin, event=("miss", "ineligible"))
        external.touch()
        pair("unknown-native-cargo-external-change", bin_path=projection_bin, expected=2, event=("miss", "ineligible"))
        completed = True
        save()
        print(json.dumps({"success": True, "invocations": len(records), "assertions": len(assertions), "report": str(root / "results.json")}))
    except BaseException as error:
        failure = f"{type(error).__name__}: {error}"
        save()
        print(json.dumps({"success": False, "error": str(error), "report": str(root / "results.json")}), file=sys.stderr)
        raise


if __name__ == "__main__":
    main()
