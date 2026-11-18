//! Cheap request-contract refusal before loading mixed compiler state.
//!
//! A refusal is never a Rust-language guess: these compiler flags make the
//! existing mixed backend ineligible. The unchanged pinned rustc executable
//! owns the complete request. Everything potentially eligible still reaches
//! the ordinary mixed compiler and its real semantic/ABI checks.

pub fn full_rust_only(args: &[String], host: &str) -> bool {
    let mut optimization = None;
    let mut debuginfo = None;
    let mut emission = None;
    let mut target = None;
    let mut crate_type = None;
    let mut index = 1; // argv[0] is rustc
    while let Some(arg) = args.get(index) {
        if arg == "-C" {
            index += 1;
            if let Some(value) = args.get(index) {
                let (key, value) = value.split_once('=').unwrap_or((value, ""));
                match key {
                    "opt-level" => optimization = Some(value),
                    "debuginfo" => debuginfo = Some(value),
                    _ => {}
                }
            }
        } else if let Some(value) = arg.strip_prefix("-C") {
            if let Some((key, value)) = value.split_once('=') {
                match key {
                    "opt-level" => optimization = Some(value),
                    "debuginfo" => debuginfo = Some(value),
                    _ => {}
                }
            }
        } else if arg == "--emit" || arg == "--target" || arg == "--crate-type" {
            let flag = arg.as_str();
            index += 1;
            if let Some(value) = args.get(index) {
                match flag {
                    "--emit" => emission = Some(value.as_str()),
                    "--target" => target = Some(value.as_str()),
                    "--crate-type" => crate_type = Some(value.as_str()),
                    _ => unreachable!(),
                }
            }
        } else if let Some(value) = arg.strip_prefix("--emit=") {
            emission = Some(value);
        } else if let Some(value) = arg.strip_prefix("--target=") {
            target = Some(value);
        } else if let Some(value) = arg.strip_prefix("--crate-type=") {
            crate_type = Some(value);
        }
        index += 1;
    }

    if optimization.is_some_and(|level| level != "0")
        || target.is_some_and(|target| target != host)
        || crate_type.is_some_and(|kind| kind != "bin" && kind != "rlib" && kind != "lib")
    {
        return true;
    }
    let metadata_only = emission.is_some_and(|outputs| {
        !outputs.is_empty() && outputs.split(',').all(|part| matches!(part.split('=').next(), Some("metadata" | "dep-info")))
    });
    if debuginfo.is_some_and(|level| level != "0" && level != "none") && !metadata_only {
        return true;
    }
    emission.is_some_and(|outputs| outputs.split(',').any(|part| !matches!(part.split('=').next(), Some("link" | "metadata" | "dep-info"))))
}

#[cfg(test)]
mod tests {
    use super::full_rust_only;

    const HOST: &str = "x86_64-unknown-linux-gnu";

    fn full(args: &[&str]) -> bool {
        let mut request = vec!["rustc".to_owned()];
        request.extend(args.iter().map(|arg| (*arg).to_owned()));
        full_rust_only(&request, HOST)
    }

    #[test]
    fn supported_mixed_requests_are_not_preempted() {
        for args in [
            vec!["--crate-type=bin", "-Copt-level=0", "-Cdebuginfo=0", "--emit=link"],
            vec!["--crate-type=rlib", "--emit=metadata,dep-info", "-Cdebuginfo=2"],
            vec!["--emit=metadata", "-C", "debuginfo=1"],
            vec!["--target", HOST, "-Copt-level=0"],
            vec!["-C", "debuginfo=2", "-C", "debuginfo=0", "--emit=link"],
        ] {
            assert!(!full(&args), "eligible request was preempted: {args:?}");
        }
    }

    #[test]
    fn unaccelerated_request_contracts_use_unchanged_rustc() {
        for args in [
            vec!["-Cdebuginfo=2", "--emit=link"],
            vec!["-C", "debuginfo=1", "--crate-type=bin"],
            vec!["-Copt-level=3", "--emit=metadata"],
            vec!["--target", "wasm32-unknown-unknown"],
            vec!["--crate-type=proc-macro"],
            vec!["--crate-type=cdylib"],
            vec!["--emit=llvm-ir"],
            vec!["--emit=metadata,mir"],
        ] {
            assert!(full(&args), "unaccelerated request paid for mixed setup: {args:?}");
        }
    }
}
