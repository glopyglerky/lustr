//! Historical tool-selection plans are data, never production compiler routes.
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn explicit_root_plan_is_exact_manual_tool_selection_without_probes() {
    let root = std::env::temp_dir().join("not installed BlueFe toolchain");
    let args: Vec<_> = [
        OsString::from("--bluefe-root"),
        root.as_os_str().into(),
        OsString::from("plan"),
        OsString::from("build"),
        OsString::from("--"),
        OsString::from("argument with spaces"),
    ]
    .into();
    let cargo_lust::Action::Execute(request) = cargo_lust::parse(&args).unwrap() else { panic!() };
    let selected = cargo_lust::plan(&request, cargo_lust::Programs::from_environment()).unwrap();
    assert_eq!(selected.program, root.join("bin").join(format!("cargo{}", std::env::consts::EXE_SUFFIX)));
    assert_eq!(selected.args, request.args);
    assert_eq!(selected.environment.len(), 4);
    for (variable, name) in [("CARGO", "cargo"), ("LUST_CARGO", "cargo"), ("RUSTC", "rustc"), ("RUSTDOC", "rustdoc")] {
        let path = root.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        assert!(selected.environment.contains(&(variable.into(), path.into_os_string())));
    }
}

#[test]
fn historical_plan_records_the_direct_compiler_without_executing_it() {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!("lustr-direct-cargo-{}-{nonce}", std::process::id()));
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(root.clone());
    fs::create_dir_all(root.join("bin")).unwrap();
    let direct = root.join("bin").join("lustr-cargo").join(format!("rustc{}", std::env::consts::EXE_SUFFIX));
    assert_eq!(direct.file_stem().and_then(|name| name.to_str()), Some("rustc"));
    let public = root.join("bin").join(format!("rustc{}", std::env::consts::EXE_SUFFIX));
    fs::create_dir_all(direct.parent().unwrap()).unwrap();
    fs::write(&direct, b"direct compiler fixture").unwrap();
    fs::write(&public, b"public compiler fixture").unwrap();

    let arguments = [OsString::from("--bluefe-root"), root.as_os_str().into(), OsString::from("build")];
    let cargo_lust::Action::Execute(request) = cargo_lust::parse(&arguments).unwrap() else { panic!() };
    let plan = cargo_lust::plan(&request, cargo_lust::Programs::from_environment()).unwrap();
    assert!(plan.environment.contains(&(OsString::from("RUSTC"), direct.clone().into_os_string())));
    assert!(!plan.environment.contains(&(OsString::from("RUSTC"), public.clone().into_os_string())));
    fs::remove_file(&direct).unwrap();
    let plan = cargo_lust::plan(&request, cargo_lust::Programs::from_environment()).unwrap();
    assert!(plan.environment.contains(&(OsString::from("RUSTC"), public.into_os_string())));
}

#[test]
fn ambiguous_root_selections_fail_during_planning() {
    for args in [
        vec!["--bluefe-root"],
        vec!["--bluefe-root", "relative", "build"],
        vec!["--bluefe-root", "/abs", "+stable", "build"],
        vec!["--bluefe-root", "/abs", "--engine", "lust", "build"],
        vec!["--bluefe-root", "/abs", "--bluefe-root", "/abs", "build"],
    ] {
        assert!(cargo_lust::parse(&args.iter().map(OsString::from).collect::<Vec<_>>()).is_err());
    }
}

#[cfg(unix)]
#[test]
fn root_plan_preserves_every_nonzero_argument_byte() {
    use std::os::unix::ffi::OsStringExt;
    for byte in 1..=255 {
        let argument = OsString::from_vec(vec![b'x', byte, b'y']);
        let input = vec!["--bluefe-root".into(), "/nonexistent".into(), "build".into(), argument.clone()];
        let cargo_lust::Action::Execute(request) = cargo_lust::parse(&input).unwrap() else { panic!() };
        let selected = cargo_lust::plan(&request, cargo_lust::Programs::from_environment()).unwrap();
        assert_eq!(selected.args[1], argument);
    }
}
