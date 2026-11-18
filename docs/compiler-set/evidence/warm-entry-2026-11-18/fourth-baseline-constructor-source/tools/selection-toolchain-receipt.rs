//! Record source-package tools used by public Cargo target selection.
#![allow(dead_code)]
use std::{fs, io::Write, path::Path, process::Command};
#[path = "host-sdk.rs"]
mod host_sdk;

pub fn write(package: &Path, source: &str, host: &str) {
    let mut paths = ["bin/cargo", "bin/rustc-full", "bin/rustc-reference", "bin/rustdoc"].map(std::path::PathBuf::from).to_vec();
    for entry in fs::read_dir(package.join("lib")).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if name.contains(".so") || name.ends_with(".dylib") {
            paths.push(Path::new("lib").join(name));
        }
    }
    paths.sort();
    let mut record = format!("lustr-selection-toolchain-v1\nsource={source}\nhost={host}\n");
    for path in paths {
        record.push_str(&format!("{}  {}\n", host_sdk::sha256(&package.join(&path)).unwrap(), path.display()));
    }
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(package.join("lib/lustr-selection-tools.txt")).unwrap();
    file.write_all(record.as_bytes()).unwrap();
    file.sync_all().unwrap();
}

// This explicit helper supports verified existing source packages in the
// selector regression gate. Normal package construction calls write directly.
fn main() {
    let package = std::env::args_os().nth(1).expect("PACKAGE_ROOT");
    let package = Path::new(&package);
    let output = Command::new(package.join("bin/rustc-reference")).arg("-vV").output().unwrap();
    assert!(output.status.success());
    let version = std::str::from_utf8(&output.stdout).unwrap();
    let host = version.lines().find_map(|line| line.strip_prefix("host: ")).unwrap();
    let pins = include_str!("../upstream/bluefe-core.pin");
    let pin = |key: &str| pins.lines().find_map(|line| line.strip_prefix(&format!("{key} = \""))).unwrap().trim_end_matches('"');
    host_sdk::check_compiler_version(version, pin("source"), host).unwrap();
    if host == "x86_64-unknown-linux-gnu" {
        assert_eq!(host_sdk::sha256(&package.join("bin/cargo")).unwrap(), pin("cargo_sha256"));
        assert_eq!(host_sdk::sha256(&package.join("bin/rustc-reference")).unwrap(), pin("rustc_sha256"));
    }
    write(package, pin("source"), host);
}
