//! Read-only Rust-compatible discovery for the current native package.
//!
//! This is deliberately smaller than rustc's query surface. It reports native
//! capabilities. Library filenames describe only the packaged scalar-library contract.
use std::{ffi::OsString, io::Read, path::{Path, PathBuf}, process::Command};

pub fn host_proc_macro_compiler(bin: &Path, host: &str) -> Option<PathBuf> {
    if host != "x86_64-unknown-linux-gnu" {
        return None;
    }
    let compiler = bin.parent()?.join("libexec/lustr-host-proc-macro/bin/lustr-rustc-host-proc-macro");
    compiler.is_file().then_some(compiler)
}

fn information_query(args: &[OsString]) -> bool {
    args.iter().any(|arg| arg == "--print" || arg.to_str().is_some_and(|s| s.starts_with("--print=")))
}

fn native_print(kind: &str) -> bool {
    matches!(kind, "file-names" | "sysroot" | "split-debuginfo" | "crate-name" | "cfg")
}

pub fn needs_rust_query(args: &[OsString], host: &str) -> bool {
    if !information_query(args) {
        return false;
    }
    args.windows(2).any(|pair| {
        pair[0] == "--target" && pair[1] != host
            || pair[0] == "--print" && pair[1].to_str().is_none_or(|kind| !native_print(kind))
            || matches!(pair[0].to_str(), Some("-C" | "--codegen"))
                && pair[1].to_str().is_none_or(|option| discovery_codegen_option(option).is_err())
    }) || args.iter().filter_map(|arg| arg.to_str()).any(|arg| {
        arg.strip_prefix("--target=").is_some_and(|target| target != host)
            || arg.strip_prefix("--print=").is_some_and(|kind| !native_print(kind))
            || arg.starts_with("-Z")
            || arg.strip_prefix("-C").filter(|option| !option.is_empty())
                .or_else(|| arg.strip_prefix("--codegen="))
                .is_some_and(|option| discovery_codegen_option(option).is_err())
    })
}

pub fn answer(args: &[OsString], bin: &Path, host: &str) -> Option<Result<(), String>> {
    if args == ["-vV"] || args == ["--version", "--verbose"] {
        println!("lustr {}", env!("CARGO_PKG_VERSION"));
        println!("binary: lustr\ncommit-hash: unknown\ncommit-date: unknown");
        println!("host: {host}\nrelease: {}", env!("LUSTR_DECLARED_RUST_RELEASE"));
        println!("lustr-native-build: {}", env!("LUST_BUILD_HASH"));
        println!("lustr-declared-rust-source: {}", env!("LUSTR_CORE_SOURCE"));
        println!("lustr-support: partial-native; Rust release names the declared semantics, not full coverage");
        return Some(Ok(()));
    }
    if !information_query(args) {
        return None;
    }
    Some(query(args, bin, host))
}

fn discovery_codegen_option(option: &str) -> Result<(), String> {
    // These options affect emission and linking. None of the queries below
    // computes an artifact, its linker arguments, or its relocation policy.
    // Rust's target cfg describes the target's default relocation model.
    match option.split_once('=') {
        Some(("link-arg" | "link_arg", _)) => Ok(()),
        Some(("relocation-model" | "relocation_model", "default" | "static" | "pic" | "pie"
            | "dynamic-no-pic" | "ropi" | "rwpi" | "ropi-rwpi")) => Ok(()),
        _ => Err(format!("the native information query does not represent codegen option {option:?}")),
    }
}

fn query(args: &[OsString], bin: &Path, host: &str) -> Result<(), String> {
    let args = args.iter().map(|arg| arg.to_str().ok_or("query arguments must be UTF-8")).collect::<Result<Vec<_>, _>>()?;
    let mut prints = Vec::new();
    let mut crate_types = Vec::new();
    let mut name = None;
    let mut target = None;
    let mut stdin = false;
    let mut cfg_options = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| args.next().ok_or_else(|| format!("missing value for {flag}"));
        match arg {
            "-" if !stdin => stdin = true,
            "--print" => prints.push(value(arg)?),
            "--crate-type" => crate_types.extend(value(arg)?.split(',')),
            "--crate-name" if name.is_none() => name = Some(value(arg)?),
            "--target" if target.is_none() => target = Some(value(arg)?),
            "--cfg" | "--check-cfg" => cfg_options.extend([arg, value(arg)?]),
            "-C" | "--codegen" => discovery_codegen_option(value(arg)?)?,
            // Cargo's target discovery sets this diagnostic level.
            "-Wwarnings" => {}
            arg if arg.starts_with("--print=") => prints.push(&arg[8..]),
            arg if arg.starts_with("--crate-type=") => crate_types.extend(arg[13..].split(',')),
            arg if arg.starts_with("--crate-name=") && name.is_none() => name = Some(&arg[13..]),
            arg if arg.starts_with("--target=") && target.is_none() => target = Some(&arg[9..]),
            arg if arg.starts_with("--cfg=") => cfg_options.extend(["--cfg", &arg[6..]]),
            arg if arg.starts_with("--check-cfg=") => cfg_options.extend(["--check-cfg", &arg[12..]]),
            arg if arg.starts_with("-C") => discovery_codegen_option(&arg[2..])?,
            arg if arg.starts_with("--codegen=") => discovery_codegen_option(&arg[10..])?,
            _ => return Err(format!("the native information query does not represent argument {arg:?}")),
        }
    }
    if target.is_some_and(|target| target != host) {
        return Err("the native information query currently describes only the built host target".into());
    }
    if name.is_none() && (prints.contains(&"file-names") || prints.contains(&"crate-name")) {
        return Err("source-dependent artifact/name discovery requires an explicit crate name".into());
    }
    if stdin {
        let mut byte = [0];
        if std::io::stdin().read(&mut byte).map_err(|error| format!("cannot inspect query input: {error}"))? != 0 {
            return Err("source-dependent information queries are not integrated; Cargo discovery requires empty input".into());
        }
    }
    let name = name.unwrap_or("rust_out");
    if name.is_empty() || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        return Err("the native information query needs an ordinary crate name".into());
    }
    if crate_types.is_empty() {
        crate_types.push("bin");
    }
    for kind in &crate_types {
        if !matches!(*kind, "bin" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro" | "lib") {
            return Err(format!("unknown crate type {kind:?}"));
        }
    }
    for print in &prints {
        if !native_print(print) {
            return Err(format!("native information query {print:?} is not integrated"));
        }
    }
    // Resolve fallible inputs before output. The native compiler owns cfg;
    // the entry does not copy SDK target tables or execute an SDK compiler.
    let cfg = if prints.contains(&"cfg") || !cfg_options.is_empty() {
        let output = Command::new(bin.join("lust-core"))
            .arg("--lustr-print-cfg")
            .args(cfg_options)
            .output()
            .map_err(|error| format!("cannot query native target facts: {error}"))?;
        if !output.status.success() {
            return Err(format!("native target query failed: {}", String::from_utf8_lossy(&output.stderr).trim_end()));
        }
        Some(String::from_utf8(output.stdout).map_err(|_| "native target facts are not UTF-8")?)
    } else {
        None
    };
    // Cargo adds the reported target library directory to the loader search
    // path for its programs and tests. It must include the Rust component's
    // matching shared runtime when that component is installed.
    let rust_driver = host_proc_macro_compiler(bin, host);
    let sysroot = rust_driver.as_deref().and_then(Path::parent).and_then(Path::parent)
        .or_else(|| bin.parent()).and_then(Path::to_str).ok_or("compiler sysroot is not UTF-8")?;
    for print in prints {
        match print {
            "file-names" => {
                for kind in &crate_types {
                    if *kind == "bin" {
                        println!("{name}");
                    } else if matches!(*kind,"lib"|"rlib") && lustr_compiler::public_library_host_supported()
                        && host == "x86_64-unknown-linux-gnu"
                        && bin.parent().is_some_and(|root|root.join("lib/lustr-metadata/sdk-receipt.json").is_file()) {
                        println!("lib{name}.rlib");
                    } else if matches!(*kind, "proc-macro" | "cdylib" | "dylib")
                        && host_proc_macro_compiler(bin, host).is_some()
                    {
                        println!("lib{name}.so");
                    } else if *kind == "staticlib" && host_proc_macro_compiler(bin, host).is_some() {
                        println!("lib{name}.a");
                    } else {
                        // Cargo's pinned target_info parser skips a type with
                        // this warning without consuming a filename for it.
                        eprintln!("warning: unsupported crate type `{kind}` for the current native Lustr package");
                    }
                }
            }
            "sysroot" => println!("{sysroot}"),
            "split-debuginfo" => println!("off"),
            "crate-name" => println!("{name}"),
            "cfg" => print!("{}", cfg.as_ref().unwrap()),
            _ => unreachable!(),
        }
    }
    Ok(())
}
