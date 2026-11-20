//! Record source-package tools used by public Cargo target selection.
#![allow(dead_code)]
use std::{fs, io::Write, path::Path, process::Command};
#[path = "host-sdk.rs"]
mod host_sdk;

pub fn write(package: &Path, source: &str, host: &str) {
    let paths = ["bin/cargo", "bin/lustr", "bin/rustc"].map(std::path::PathBuf::from);
    assert_eq!(package.join("bin/lustr").canonicalize().unwrap(), package.join("bin/rustc").canonicalize().unwrap());
    let mut record = format!("lustr-native-selection-toolchain-v2\nsource={source}\nhost={host}\n");
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
    let output = Command::new(package.join("bin/lustr")).arg("--lustr-version").output().unwrap();
    assert!(output.status.success());
    let version = std::str::from_utf8(&output.stdout).unwrap();
    let host = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        _ => panic!("native package host is unsupported"),
    };
    let pins = include_str!("../upstream/bluefe-core.pin");
    let pin = |key: &str| pins.lines().find_map(|line| line.strip_prefix(&format!("{key} = \""))).unwrap().trim_end_matches('"');
    assert!(
        version.contains(&format!("declared Rust source {}", pin("source"))),
        "receipt requires the native entry's declared source identity"
    );
    if host == "x86_64-unknown-linux-gnu" {
        assert_eq!(host_sdk::sha256(&package.join("bin/cargo")).unwrap(), pin("cargo_sha256"));
    }
    write(package, pin("source"), host);
}
