#!/usr/bin/env python3
"""Qualify the exact TinyCC signing helper and native output publication.

Linux controls exercise argv/status handling from the declared patched C
helper. Real Mach-O signing and native signing-failure publication controls run
only on Darwin. The existing upstream odd-path retained scenario remains the
whole-compiler regression; these component controls do not claim that result.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--tcc", type=Path, required=True, help="TinyCC install prefix")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source, tcc = [p.resolve(strict=True) for p in [args.source, args.tcc]]
    output = args.output.absolute()
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema": "lustr-tinycc-signing-controls-v1", "passed": False,
              "platform": platform.system(), "full_compiler_qualification": False,
              "darwin_execution": platform.system() == "Darwin",
              "runner_sha256": digest(__file__), "commands": [], "controls": []}

    def save():
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    def require(condition, message):
        if not condition:
            raise AssertionError(message)

    def run(label, argv, env=None, expected=0, cwd=None):
        stem = f"{len(report['commands']):02d}-{label}"
        result = subprocess.run(list(map(str, argv)), cwd=cwd or output, env=env, capture_output=True, timeout=30)
        (output / (stem + ".stdout")).write_bytes(result.stdout)
        (output / (stem + ".stderr")).write_bytes(result.stderr)
        report["commands"].append({"label": label, "argv": list(map(str, argv)),
                                   "cwd": str(cwd or output), "exit": result.returncode,
                                   "stdout": stem + ".stdout", "stderr": stem + ".stderr",
                                   "environment_overrides": {} if env is None else {
                                       key: value for key, value in env.items() if os.environ.get(key) != value}})
        save()
        require(result.returncode == expected, f"{label}: unexpected status {result.returncode}: {result.stderr!r}")
        return result

    try:
        declaration = json.loads((ROOT / "upstream/tinycc-macho-signing.json").read_text())
        require(digest(source / "tccmacho.c") == declaration["patched_sha256"], "test source is not the exact declared patch")
        require(digest(ROOT / declaration["patch"]) == declaration["patch_sha256"], "declared patch changed")
        receipt = json.loads((tcc / "share/licenses/tinycc/lustr-macho-signing.json").read_text())
        for field in ["revision", "original_sha256", "patched_sha256", "patch_sha256"]:
            require(receipt[field] == declaration[field], "installed TinyCC receipt differs: " + field)
        report.update(declaration=declaration, installed_receipt=receipt,
                      tcc_sha256=digest(tcc / "bin/tcc"), archive_sha256=digest(tcc / "lib/libtcc.a"))
        patch = ROOT / declaration["patch"]
        apply_tool = ROOT / "tools/apply-tinycc-signing.py"
        original_dir = output / "original-source"
        original_dir.mkdir()
        shutil.copyfile(source / "tccmacho.c", original_dir / "tccmacho.c")
        git_env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        git_env["GIT_CEILING_DIRECTORIES"] = str(original_dir.parent)
        run("recover-pinned-original", ["git", "apply", "-R", patch], git_env, cwd=original_dir)
        original = (original_dir / "tccmacho.c").read_bytes()
        require(hashlib.sha256(original).hexdigest() == declaration["original_sha256"],
                "applicator controls did not recover the exact pinned input")
        for layout in ["standalone", "checkout-root", "nested-worktree"]:
            boundary = output / ("apply-" + layout)
            boundary.mkdir()
            if layout != "standalone":
                run("init-" + layout, ["git", "init", "-q", boundary])
            copy = boundary / "nested/source" if layout == "nested-worktree" else boundary
            copy.mkdir(parents=True, exist_ok=True)
            (copy / "tccmacho.c").write_bytes(original)
            receipt_path = boundary / "application.json"
            # These inherited overrides must never redirect the child applicator.
            inherited = dict(os.environ, GIT_DIR=str(boundary / "unrelated.git"),
                             GIT_WORK_TREE=str(original_dir), GIT_INDEX_FILE=str(boundary / "wrong-index"))
            run("apply-" + layout, [sys.executable, apply_tool, copy, "--receipt", receipt_path], inherited)
            require(digest(copy / "tccmacho.c") == declaration["patched_sha256"], "wrong patched source: " + layout)
            require(json.loads(receipt_path.read_text())["action"] == "applied", "initial application was skipped")
            report["controls"].append({"case": "apply-" + layout, "passed": True})
        run("repeat-application", [sys.executable, apply_tool, copy, "--receipt", receipt_path])
        require(json.loads(receipt_path.read_text())["action"] == "already-applied", "repeat application lost source identity")
        report["controls"].append({"case": "repeat-application", "passed": True})
        (copy / "tccmacho.c").write_bytes(original + b"\n/* unreviewed change */\n")
        altered = digest(copy / "tccmacho.c")
        refused = boundary / "refused-application.json"
        run("refuse-altered-input", [sys.executable, apply_tool, copy, "--receipt", refused], expected=1)
        require(digest(copy / "tccmacho.c") == altered and not refused.exists(), "altered input was modified or certified")
        report["controls"].append({"case": "refuse-altered-input", "passed": True})
        require(digest(original_dir / "tccmacho.c") == declaration["original_sha256"], "inherited Git context changed another source")
        text = (source / "tccmacho.c").read_text()
        start = text.index("\n#ifdef CONFIG_CODESIGN\n#include <spawn.h>")
        end = text.index("\n/* In order", start)
        helper = text[start:end]
        require(text.count("ret = tcc_codesign_file(s1, filename);") == 1, "Mach-O output does not call the tested helper")
        fake_bin = output / "fake-bin"
        fake_bin.mkdir()
        signer = fake_bin / "codesign"
        signer.write_text(f"#!{sys.executable}\n" +
                          "import base64,json,os,signal,sys\n"
                          "with open(os.environ['SIGN_TEST_LOG'],'a') as f:\n"
                          " f.write(json.dumps([base64.b64encode(os.fsencode(x)).decode() for x in sys.argv[1:]])+'\\n')\n"
                          "mode=os.environ.get('SIGN_TEST_MODE','success')\n"
                          "if mode=='signal': os.kill(os.getpid(),signal.SIGTERM)\n"
                          "sys.exit(7 if mode=='exit' else 0)\n")
        signer.chmod(0o755)
        prefix = r"""
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <errno.h>
#include <sys/wait.h>
typedef int TCCState;
static TCCState test_state;
static void test_enter_state(TCCState *s1) { if (s1 != &test_state) abort(); }
static int errors, wait_calls;
static int test_error(const char *format, ...) {
    va_list args; va_start(args, format); vfprintf(stderr, format, args); va_end(args);
    fputc('\n', stderr); ++errors; return -1;
}
static pid_t observed_waitpid(pid_t pid, int *status, int options) {
    pid_t result;
    ++wait_calls;
    if (WAIT_MODE == 1 && wait_calls == 1) { errno = EINTR; return -1; }
    result = waitpid(pid, status, options);
    if (WAIT_MODE == 2 && result == pid) { errno = ECHILD; return -1; }
    return result;
}
#define waitpid observed_waitpid
#define tcc_error_noabort (test_enter_state(s1), test_error)
#define CONFIG_CODESIGN 1
"""
        suffix = r"""
int main(int argc, char **argv) {
    int result;
    if (argc != 2) return 90;
    result = tcc_codesign_file(&test_state, argv[1]);
    printf("returned=%d errors=%d waits=%d\n", result, errors, wait_calls);
    return result ? 21 : 0;
}
"""
        cc = shutil.which("cc")
        require(cc is not None, "a C compiler is required for the exact helper controls")
        arch = {"x86_64": "X86_64", "arm64": "ARM64", "aarch64": "ARM64"}.get(platform.machine())
        require(arch is not None, "the full Mach-O source control needs a supported host target")
        run("compile-actual-macho-source", [cc, "-c", "-Wall", "-Werror=implicit-function-declaration",
                                           "-DCONFIG_CODESIGN", "-DTCC_TARGET_MACHO", "-DTCC_TARGET_" + arch,
                                           "-DONE_SOURCE=0", source / "tccmacho.c", "-o", output / "macho-context.o"])
        report["controls"].append({"case": "actual-macho-source-context", "passed": True,
                                   "tcc_header_sha256": digest(source / "tcc.h"),
                                   "configuration_sha256": digest(source / "config.h"),
                                   "scope": "host compilation of the actual Mach-O translation unit; no target execution"})
        helper_file = output / "helper.c"
        helper_file.write_text(prefix + helper + suffix)
        report["helper_source_sha256"] = digest(helper_file)
        for mode in range(3):
            run("compile-helper-" + str(mode), [cc, "-std=c99", "-Wall", "-Wextra", "-Werror",
                                                f"-DWAIT_MODE={mode}", helper_file, "-o", output / f"helper-{mode}"])

        def signing_case(name, filename, mode="success", helper_mode=0, missing=False):
            log = output / (name + ".signer.jsonl")
            env = dict(os.environ, PATH=str(output / "no-tools" if missing else fake_bin),
                       SIGN_TEST_LOG=str(log), SIGN_TEST_MODE=mode)
            failure = missing or mode != "success" or helper_mode == 2
            result = run(name, [output / f"helper-{helper_mode}", filename], env, 21 if failure else 0)
            rows = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
            expected_args = [base64.b64encode(os.fsencode(x)).decode() for x in ["-f", "-s", "-", "--", filename]]
            require(rows == ([] if missing else [expected_args]), name + ": signer ran more than once or argv changed")
            waits = 0 if missing else 2 if helper_mode == 1 else 1
            expected_output = f"returned={-1 if failure else 0} errors={int(failure)} waits={waits}\n".encode()
            require(result.stdout == expected_output, name + ": helper did not return its ordinary result")
            require(not (output / "SHELL_SIDE_EFFECT").exists(), "the path was interpreted as a shell command")
            report["controls"].append({"case": name, "passed": True, "signer_attempts": len(rows),
                                       "path_bytes": len(os.fsencode(filename))})
            save()

        signing_case("metacharacters", "out sp ace#é 'quoted' \"$x\";touch SHELL_SIDE_EFFECT")
        signing_case("long-argument", "nested/" * 155 + "output")
        signing_case("leading-option", "-filename")
        signing_case("non-utf8-argument", os.fsdecode(b"path-\xff"))
        signing_case("nonzero-status", "output", mode="exit")
        signing_case("signal-status", "output", mode="signal")
        signing_case("spawn-error", "output", missing=True)
        signing_case("interrupted-wait", "output", helper_mode=1)
        signing_case("wait-error", "output", helper_mode=2)

        # Compile the actual native publication module. The worker-pool seam is
        # uncalled: this driver supplies one C unit and never runs in memory.
        native = ROOT / "compiler/native/src/tcc.rs"
        quoted_path = str(native)
        hashes = "#"
        while '"' + hashes in quoted_path:
            hashes += "#"
        module_path = f'r{hashes}"{quoted_path}"{hashes}'
        driver = output / "native-publication.rs"
        driver.write_text(f"#[path = {module_path}]\nmod tcc;\n" + r"""
mod pool {
    pub fn threads() -> usize { panic!("one-unit publication control must not start workers") }
    pub fn stop() { panic!("publication control must not run a program in memory") }
}
#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let source = std::path::PathBuf::from(&args[1]);
    let object = std::path::PathBuf::from(&args[2]);
    let output = std::path::PathBuf::from(&args[3]);
    tcc::compile_units(&[(object.clone(), std::fs::read_to_string(source).unwrap())], false).unwrap();
    let result = if args[4] == "shared" {
        tcc::link_shared(&[object], &output, false, Some("signing-control"))
    } else { tcc::link(&[object], &output, false) };
    match result {
        Ok(()) => println!("native-output-published"),
        Err(error) => { eprintln!("native-link-error-returned: {error}"); std::process::exit(17); }
    }
}
""")
        rustc = shutil.which("rustc")
        require(rustc is not None, "Rust is required for the actual native publication module")
        run("build-rust-version", [rustc, "-vV"])
        build_env = dict(os.environ, LUST_TCC_LIB_PATH=str(tcc / "lib/tcc"), LUST_TCC_BIN=str(tcc / "bin/tcc"))
        publication = output / "native-publication"
        run("compile-native-publication", [rustc, "--edition=2024", "-Awarnings", "-Copt-level=1",
                                           driver, "-L", "native=" + str(tcc / "lib"), "-l", "m",
                                           "-o", publication], build_env)
        report["native_module_sha256"] = digest(native)
        odd = output / "sp ace#é 'quoted' $literal;name"
        odd.mkdir()
        c_source, obj, executable = odd / "main.c", odd / "main.o", odd / "program"
        c_source.write_text('extern int puts(const char *); int main(void) { puts("signed-output"); return 0; }\n')
        run("native-executable-link", [publication, c_source, obj, executable, "executable"])
        actual = run("native-executable-result", [executable])
        require(actual.stdout == b"signed-output\n" and actual.stderr == b"", "native program result differs")
        report["controls"].append({"case": "actual-native-publication", "passed": True,
                                   "artifact_sha256": digest(executable), "format": "Mach-O" if report["darwin_execution"] else "host-native"})
        if report["darwin_execution"]:
            run("verify-native-signature", ["/usr/bin/codesign", "--verify", "--verbose=2", executable])
            old_hash = digest(executable)
            log = output / "native-failed-signing.jsonl"
            env = dict(os.environ, PATH=str(fake_bin), SIGN_TEST_LOG=str(log), SIGN_TEST_MODE="exit")
            failed = run("native-failed-signing", [publication, c_source, obj, executable, "executable"], env, 17)
            require(b"native-link-error-returned:" in failed.stderr and b"exit status 7" in failed.stderr,
                    "native signing failure did not return through the ordinary error path")
            require(digest(executable) == old_hash, "failed signing replaced the previously published executable")
            require(len(log.read_text().splitlines()) == 1, "failed native signing retried its signer")
            report["controls"].append({"case": "darwin-failed-signing-preserves-output", "passed": True})
            c_source.write_text("int answer(void) { return 73; }\n")
            library = odd / "libsigning-control.dylib"
            run("native-shared-link", [publication, c_source, obj, library, "shared"])
            run("verify-shared-signature", ["/usr/bin/codesign", "--verify", "--verbose=2", library])
            loaded = run("shared-library-result", [sys.executable, "-c",
                         "import ctypes,sys; print(ctypes.CDLL(sys.argv[1]).answer())", library])
            require(loaded.stdout == b"73\n" and loaded.stderr == b"", "signed shared library result differs")
            report["controls"].append({"case": "darwin-signed-shared-library", "passed": True})
        report["passed"] = True
    except Exception as error:
        report["failure"] = repr(error)
    finally:
        save()
    print(json.dumps({key: report.get(key) for key in ["passed", "platform", "darwin_execution", "failure", "controls"]}, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
