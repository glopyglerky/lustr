use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub(crate) fn program(root: &Path, name: &str) -> PathBuf {
    root.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

pub(crate) fn environment(root: &Path) -> Vec<(OsString, OsString)> {
    // The assembled mixed toolchain provides a direct Cargo compiler entry.
    // Cargo already supplies a complete rustc invocation: going through the
    // public lustr/rustc launcher only adds an exec for every crate. Preserve
    // the public launcher for standalone subset and compiler commands.
    let direct = program(root, "rustc-cargo");
    let cargo_rustc = if direct.is_file() { direct } else { program(root, "rustc") };
    [("CARGO", "cargo"), ("LUST_CARGO", "cargo"), ("RUSTC", "rustc"), ("RUSTDOC", "rustdoc")]
        .into_iter()
        .map(|(variable, executable)| {
            let executable = if variable == "RUSTC" { cargo_rustc.clone() } else { program(root, executable) };
            (variable.into(), executable.into_os_string())
        })
        .collect()
}

pub(crate) fn validate(root: &Path) -> Result<(), String> {
    for name in ["cargo", "rustc", "rustdoc"] {
        let path = program(root, name);
        if !path.is_file() {
            return Err(format!("BlueFe toolchain is missing {}", path.display()));
        }
    }
    Ok(())
}
