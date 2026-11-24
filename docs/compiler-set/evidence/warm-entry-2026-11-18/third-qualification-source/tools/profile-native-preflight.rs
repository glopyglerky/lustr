//! Profile the public entry's Cargo target preflight without compiling a program.
//!
//! This deliberately includes the production parser, selector and child owner.
//! It is a component diagnostic, not a compiler benchmark. See the associated
//! native-preflight measurement note for build and execution instructions.
#![allow(dead_code)]

#[path = "../compiler/native/src/protocol.rs"]
mod protocol;
#[path = "../src/full.rs"]
mod full;
#[path = "../src/owned_child.rs"]
mod owned_child;

use std::{io::Write, path::PathBuf, process::{Command, ExitCode}, time::Instant};

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let phase = arguments.next().expect("PHASE PACKAGE_BIN NATIVE_ARGUMENTS...");
    let parent = PathBuf::from(arguments.next().expect("PACKAGE_BIN"));
    assert!(parent.is_absolute(), "PACKAGE_BIN must be absolute");
    if phase == "prime-selection-tools" {
        return match full::prime_selection_tools(&parent) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(2)
            }
        };
    }
    let args: Vec<_> = arguments.collect();
    let Some(protocol::Invocation::Compile { req, run: false, .. }) = protocol::parse_args(&args) else {
        eprintln!("expected the same native check/build arguments as the public entry");
        return ExitCode::from(2);
    };
    let Some(manifest) = req.session.krate.manifest_path.as_ref() else {
        eprintln!("this diagnostic requires a manifest-based native request");
        return ExitCode::from(2);
    };

    if phase == "selection" {
        let start = Instant::now();
        let result = full::select(&req, &parent, &args);
        let elapsed_ns = start.elapsed().as_nanos();
        let record = match &result {
            Ok(selection) => serde_json::json!({
                "phase": phase, "elapsed_ns": elapsed_ns, "success": true,
                "manifest": selection.manifest, "target": selection.name,
            }),
            Err(error) => serde_json::json!({
                "phase": phase, "elapsed_ns": elapsed_ns, "success": false,
                "error": error.to_string(),
            }),
        };
        println!("{record}");
        return if result.is_ok() { ExitCode::SUCCESS } else { ExitCode::from(2) };
    }

    // Match full::cargo and the metadata/pkgid argv in full::select exactly.
    // These individual observations cannot be subtracted from another run's
    // whole-entry duration to attribute a performance difference.
    let mut command = Command::new(parent.join("cargo"));
    command.env("RUSTC", parent.join("rustc-full")).env("RUSTDOC", parent.join("rustdoc"));
    if std::env::var_os("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT").is_none() {
        command.env("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "1");
    }
    match phase.as_str() {
        "metadata" => {
            command.args(["metadata", "--format-version=1", "--manifest-path"]).arg(manifest).args(req.session.cargo_net.args());
        }
        "pkgid" => {
            // An explicit benchmark package is separate from native argv so
            // this helper does not implement Cargo's package-spec grammar.
            let package = std::env::var_os("LUSTR_PROFILE_PACKAGE").expect("pkgid requires LUSTR_PROFILE_PACKAGE");
            command.args(["pkgid", "--manifest-path"]).arg(manifest).arg("--package").arg(package).args(req.session.cargo_net.args());
        }
        _ => {
            eprintln!("phase must be selection, metadata or pkgid");
            return ExitCode::from(2);
        }
    }
    let start = Instant::now();
    let result = owned_child::output(&mut command);
    let elapsed_ns = start.elapsed().as_nanos();
    match result {
        Ok(output) => {
            let success = output.status.success();
            std::io::stderr().write_all(&output.stderr).unwrap();
            println!("{}", serde_json::json!({
                "phase": phase, "elapsed_ns": elapsed_ns, "success": success,
                "exit": output.status.code(),
                "stdout": String::from_utf8_lossy(&output.stdout),
            }));
            if success { ExitCode::SUCCESS } else { ExitCode::from(2) }
        }
        Err(error) => {
            println!("{}", serde_json::json!({
                "phase": phase, "elapsed_ns": elapsed_ns, "success": false,
                "error": error.to_string(),
            }));
            ExitCode::from(2)
        }
    }
}
