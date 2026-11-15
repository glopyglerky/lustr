#![forbid(unsafe_code)]

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Native {
    pub args: Vec<String>,
    pub output: PathBuf,
}

/// This recognizes compiler requests, not source keywords. Requests requiring
/// Rust metadata or an unrepresented option stay with the full compiler core.
pub fn native_request(args: &[OsString]) -> Option<Native> {
    let mut input = None;
    let mut output = None;
    let mut settings = Vec::new();
    let mut edition = false;
    let mut crate_type = false;
    let mut emit = false;
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_str()?;
        let mut value = |name: &str| -> Option<&str> {
            if arg == name {
                index += 1;
                args.get(index)?.to_str()
            } else {
                arg.strip_prefix(name)?.strip_prefix('=')
            }
        };
        if arg == "-o" {
            if output.is_some() {
                return None;
            }
            index += 1;
            output = Some(PathBuf::from(args.get(index)?));
        } else if arg == "--edition" || arg.starts_with("--edition=") {
            if edition || value("--edition")? != "2024" {
                return None;
            }
            edition = true;
        } else if arg == "--crate-type" || arg.starts_with("--crate-type=") {
            if crate_type || value("--crate-type")? != "bin" {
                return None;
            }
            crate_type = true;
        } else if arg == "--crate-name" || arg.starts_with("--crate-name=") {
            return None;
        } else if arg == "--emit" || arg.starts_with("--emit=") {
            if emit {
                return None;
            }
            match value("--emit")? {
                "link" => {}
                // Metadata is a real artifact, even when it is empty. Do not fake it.
                _ => return None,
            }
            emit = true;
        } else if arg == "-C" || arg.starts_with("-C") {
            let flag = if arg == "-C" {
                index += 1;
                args.get(index)?.to_str()?
            } else {
                &arg[2..]
            };
            match flag {
                "panic=abort"
                | "panic=unwind"
                | "overflow-checks=on"
                | "overflow-checks=off"
                | "debug-assertions=on"
                | "debug-assertions=off" => {
                    settings.extend(["-C".to_owned(), flag.to_owned()]);
                }
                "opt-level=0" | "debuginfo=0" => {}
                _ => return None,
            }
        } else if arg.starts_with('-') || !arg.ends_with(".rs") || input.is_some() {
            return None;
        } else {
            input = Some(arg.to_owned());
        }
        index += 1;
    }
    if !edition {
        return None;
    }
    let input = input?;
    let output = output.unwrap_or_else(|| PathBuf::from(&input).file_stem().map(PathBuf::from).unwrap());
    let mut native = vec!["build".to_owned(), input, "-o".to_owned(), output.to_str()?.to_owned(), "--no-default-features".to_owned()];
    native.extend(settings);
    Some(Native { args: native, output })
}

pub fn native_command(args: &[OsString]) -> Option<Vec<OsString>> {
    let command = args.first()?.to_str()?;
    if !matches!(command, "build" | "check") || args.len() < 2 || !args[1].to_str()?.ends_with(".rs") {
        return None;
    }
    let mut full = vec![OsString::from("--edition=2024"), args[1].clone()];
    if command == "check" {
        full.extend([OsString::from("--emit=metadata"), OsString::from("-o"), OsString::from("/dev/null")]);
    }
    full.extend_from_slice(&args[2..]);
    Some(full)
}

pub fn is_native_command(args: &[OsString]) -> bool {
    args.first().is_some_and(|a| a == "build" || a == "check") && args.get(1).and_then(|a| a.to_str()).is_some_and(|a| a.ends_with(".rs"))
}

pub fn cargo_command(args: &[OsString]) -> bool {
    args.first().is_some_and(|arg| {
        matches!(arg.to_str(), Some("doctor" | "plan" | "test" | "run" | "build" | "check" | "clippy" | "fmt" | "--engine" | "--explain"))
            || arg == OsStr::new("--bluefe-root")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }
    #[test]
    fn ordinary_binary_uses_native_compiler() {
        let n = native_request(&args(&["--edition=2024", "main.rs", "-o", "out", "-C", "panic=unwind"])).unwrap();
        assert_eq!(n.args, ["build", "main.rs", "-o", "out", "--no-default-features", "-C", "panic=unwind"]);
    }
    #[test]
    fn artifact_and_configuration_contracts_are_not_dropped() {
        for tail in [
            vec!["--emit=metadata"],
            vec!["--emit=dep-info,link"],
            vec!["--extern", "dep=libdep.rlib"],
            vec!["--test"],
            vec!["-Copt-level=3"],
            vec!["--cfg", "custom"],
        ] {
            let mut a = args(&["--edition=2024", "main.rs"]);
            a.extend(args(&tail));
            assert!(native_request(&a).is_none(), "{tail:?}");
        }
        assert!(native_request(&args(&["main.rs"])).is_none());
        assert!(native_request(&args(&["--edition=2021", "main.rs"])).is_none());
    }
}
