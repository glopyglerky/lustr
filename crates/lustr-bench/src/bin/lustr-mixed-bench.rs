//! Independent Rust test oracle and historical evidence parser controls.
//! Acquisition of the retired whole-rustc package is disabled. Recorded
//! campaign readers live in tools/report-mixed-performance.py.
#![forbid(unsafe_code)]

#[cfg(test)]
#[path = "../native_evidence.rs"]
mod native_evidence;
#[cfg(test)]
#[path = "../order.rs"]
mod order;
#[cfg(test)]
#[path = "../stats.rs"]
mod stats;

use nix::sys::signal::Signal;
use serde_json::json;
use std::{ffi::OsString, os::unix::process::ExitStatusExt, path::Path, process::Command};
#[cfg(test)]
use std::{fs, path::PathBuf};

type R<T> = Result<T, Box<dyn std::error::Error>>;
const RUSTC_FLAGS: &[&str] = &[
    "--edition=2024",
    "--target=x86_64-unknown-linux-gnu",
    "-C",
    "opt-level=0",
    "-C",
    "debuginfo=0",
    "-C",
    "panic=unwind",
    "-C",
    "overflow-checks=on",
    "-C",
    "debug-assertions=on",
];
const TEST_RECEIPT: &str = "lustr benchmark rustc-test: ";

fn rustc_arguments(command: &mut Command, source: &Path, output: &Path, operation: &str) {
    command.arg(source).args(RUSTC_FLAGS);
    command
        .arg(match operation {
            "build" => "--emit=link",
            "check" => "--emit=metadata",
            "test" => "--test",
            _ => unreachable!("fixed native reference operation"),
        })
        .arg("-o")
        .arg(output);
}

fn child_status(command: &mut Command, phase: &str) -> R<i32> {
    let argv: Vec<_> = std::iter::once(command.get_program()).chain(command.get_args()).map(OsString::from).collect();
    let status = command.status()?;
    eprintln!("{TEST_RECEIPT}{}", json!({"phase":phase,"argv":argv,"exit":status.code(),"signal":status.signal()}));
    if let Some(signal) = status.signal() {
        // Preserve a failed child as a failed/signal result, never execute the
        // test after compiler failure and never retry through another engine.
        nix::sys::signal::raise(Signal::try_from(signal)?)?;
        return Err("signalled child unexpectedly returned".into());
    }
    Ok(status.code().ok_or("missing child exit status")?)
}

/// The outer existing recorder times this complete compile-and-test command.
/// This adapter adds no clock or estimator; both actual child commands and
/// outcomes remain in its raw stderr alongside the compiler/test diagnostics.
fn rustc_test(arguments: &[OsString]) -> R<i32> {
    require(arguments.len() == 3, "internal rustc test requires RUSTC SOURCE NEW_TEST_BINARY")?;
    let source = Path::new(&arguments[1]);
    let output = Path::new(&arguments[2]);
    require(!output.exists(), "test output must be fresh")?;
    let mut compile = Command::new(&arguments[0]);
    rustc_arguments(&mut compile, source, output, "test");
    let code = child_status(&mut compile, "compile")?;
    if code != 0 {
        return Ok(code);
    }
    require(output.is_file(), "rustc did not publish the test binary")?;
    child_status(Command::new(output).args(["--test-threads=1", "--color", "never"]), "execute")
}

fn require(condition: bool, message: impl Into<String>) -> R<()> {
    if condition { Ok(()) } else { Err(message.into().into()) }
}

fn main() -> R<()> {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    if arguments.first().is_some_and(|value| value == "--native-rustc-test") {
        std::process::exit(rustc_test(&arguments[1..])?);
    }
    Err("Retired whole-rustc acquisition is disabled; this binary only provides the independent --native-rustc-test oracle".into())
}

#[cfg(test)]
fn native_source(literal: &str, signature: bool) -> String {
    let ty = if signature { "u32" } else { "u64" };
    format!(
        "fn kernel(x:{ty})->{ty}{{x ^ {literal}}}\nfn main(){{println!(\"{{}}\",kernel(42));}}\n#[test]\nfn scalar_contract(){{assert_eq!(kernel(42),59);}}\n"
    )
}

#[cfg(test)]
mod reference_tests {
    use super::*;

    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn scratch() -> Scratch {
        let path = std::env::temp_dir().join(format!(
            "lustr-rustc-reference-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Scratch(path)
    }
    fn rustc() -> OsString {
        std::env::var_os("LUSTR_BENCH_TEST_RUSTC").or_else(|| std::env::var_os("RUSTC")).unwrap_or_else(|| "rustc".into())
    }

    #[test]
    fn reference_build_check_edits_and_rejection_execute_real_rustc() {
        let directory = scratch();
        let source = directory.0.join("main.rs");
        for (literal, signature, expected) in [("17", false, b"59\n"), ("19", false, b"57\n"), ("17", true, b"59\n")] {
            fs::write(&source, native_source(literal, signature)).unwrap();
            let binary = directory.0.join("program");
            let mut compile = Command::new(rustc());
            rustc_arguments(&mut compile, &source, &binary, "build");
            let output = compile.output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            let result = Command::new(&binary).output().unwrap();
            assert!(result.status.success());
            assert_eq!(result.stdout, expected);
            assert!(result.stderr.is_empty());
            let mut check = Command::new(rustc());
            rustc_arguments(&mut check, &source, &directory.0.join("metadata.rmeta"), "check");
            assert!(check.output().unwrap().status.success());
        }
        fs::write(&source, native_source("true", false)).unwrap();
        let metadata = directory.0.join("invalid.rmeta");
        let mut check = Command::new(rustc());
        rustc_arguments(&mut check, &source, &metadata, "check");
        let result = check.output().unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(!metadata.exists());
        assert!(String::from_utf8_lossy(&result.stderr).contains("E0277"));
    }

    #[test]
    fn reference_test_adapter_executes_once_and_preserves_failure() {
        let directory = scratch();
        let source = directory.0.join("tests.rs");
        let marker = directory.0.join("executed");
        for (name, passes, expected) in [("pass", true, 0), ("fail", false, 101)] {
            fs::write(&source, format!(
                "#[test] fn scalar_contract() {{ use std::io::Write; let mut f=std::fs::OpenOptions::new().create(true).append(true).open({marker:?}).unwrap(); f.write_all(b\"x\").unwrap(); assert!({passes}); }}\n"
            )).unwrap();
            assert_eq!(rustc_test(&[rustc(), source.clone().into_os_string(), directory.0.join(name).into_os_string()]).unwrap(), expected);
        }
        assert_eq!(fs::read(&marker).unwrap(), b"xx", "each accepted test binary must execute exactly once");
        fs::write(&source, native_source("true", false)).unwrap();
        let invalid = directory.0.join("invalid-test");
        assert_eq!(rustc_test(&[rustc(), source.into_os_string(), invalid.clone().into_os_string()]).unwrap(), 1);
        assert!(!invalid.exists());
        assert_eq!(fs::read(&marker).unwrap(), b"xx", "a failed compile must not execute or retry a test");
    }
}
