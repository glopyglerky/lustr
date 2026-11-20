//! A development host receipt records the selected SDK bytes. It cannot admit a
//! release or change the separately frozen upstream Rust source identity.
#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "../src/host_runtime.rs"]
mod host_runtime;

pub fn sha256(path: &Path) -> Result<String, String> {
    let output = host_runtime::sha256_command().arg("--").arg(path).output().map_err(|e| format!("hash {}: {e}", path.display()))?;
    if !output.status.success() {
        return Err(format!("cannot hash {}: {}", path.display(), String::from_utf8_lossy(&output.stderr)));
    }
    let hash = std::str::from_utf8(&output.stdout).ok().and_then(|s| s.split_whitespace().next()).unwrap_or("");
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("SHA256 command returned an invalid digest".into());
    }
    Ok(hash.to_ascii_lowercase())
}

pub fn check_compiler_version(version: &str, source: &str, host: &str) -> Result<(), String> {
    for (field, expected) in [("commit-hash", source), ("host", host)] {
        let values: Vec<_> = version.lines().filter_map(|line| line.strip_prefix(&format!("{field}: "))).collect();
        if values != [expected] {
            return Err(format!("selected Rust SDK {field} must be {expected}; observed {values:?}"));
        }
    }
    Ok(())
}

pub fn snapshot(sdk: &Path, source: &str, host: &str) -> Result<String, String> {
    let version = Command::new(sdk.join("bin/rustc")).arg("-vV").output().map_err(|e| format!("query selected Rust SDK: {e}"))?;
    if !version.status.success() {
        return Err("selected Rust SDK cannot report its identity".into());
    }
    check_compiler_version(std::str::from_utf8(&version.stdout).map_err(|_| "invalid Rust SDK identity")?, source, host)?;
    let mut files = vec![PathBuf::from("bin/rustc"), PathBuf::from("bin/cargo")];
    let mut drivers = 0;
    for entry in std::fs::read_dir(sdk.join("lib")).map_err(|e| format!("inspect SDK libraries: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("non-UTF8 SDK library name")?;
        if name.starts_with("librustc_driver-") && name.ends_with(std::env::consts::DLL_SUFFIX) {
            drivers += 1;
            files.push(Path::new("lib").join(name));
        } else if name.starts_with("libLLVM") && entry.path().is_file() {
            files.push(Path::new("lib").join(name));
        }
    }
    if drivers != 1 {
        return Err(format!("selected Rust SDK must contain one native rustc driver; found {drivers}"));
    }
    files.sort();
    let mut text = format!("lustr-host-sdk-v1\nsource={source}\nhost={host}\nqualification=development-source-build\n");
    for file in files {
        text.push_str(&format!("{}  {}\n", sha256(&sdk.join(&file))?, file.display()));
    }
    Ok(text)
}

pub fn verify_receipt(sdk: &Path, receipt: &Path, source: &str, host: &str, build_rustc: &Path) -> Result<(), String> {
    let recorded = std::fs::read_to_string(receipt).map_err(|e| format!("read host SDK receipt: {e}"))?;
    if recorded != snapshot(sdk, source, host)? {
        return Err("selected SDK differs from its explicit development host receipt".into());
    }
    if sha256(build_rustc)? != sha256(&sdk.join("bin/rustc"))? {
        return Err("build compiler differs from the selected host SDK".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdk_receipt_requires_the_pinned_source_and_exact_host() {
        let version = "rustc 1.0\ncommit-hash: pinned\nhost: aarch64-apple-darwin\n";
        assert!(check_compiler_version(version, "pinned", "aarch64-apple-darwin").is_ok());
        assert!(check_compiler_version(version, "other", "aarch64-apple-darwin").is_err());
        assert!(check_compiler_version(version, "pinned", "x86_64-unknown-linux-gnu").is_err());
        assert!(check_compiler_version(&format!("{version}host: aarch64-apple-darwin\n"), "pinned", "aarch64-apple-darwin").is_err());
        assert!(check_compiler_version("", "pinned", "aarch64-apple-darwin").is_err());
    }

    #[test]
    fn sha256_matches_the_published_empty_digest() {
        let path = std::env::temp_dir().join(format!("lustr-host-sdk-digest-{}", std::process::id()));
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        drop(file);
        let digest = sha256(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(digest, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }
}
