//! Historical plan data remains inspectable without executing the plan.
use std::ffi::OsString;

fn words(input: &[&str]) -> Vec<OsString> {
    input.iter().map(OsString::from).collect()
}

fn toolchain_root() -> &'static str {
    if cfg!(windows) { r"C:\not installed toolchain" } else { "/not installed toolchain" }
}

fn request(input: &[&str]) -> cargo_lust::Request {
    let cargo_lust::Action::Execute(request) = cargo_lust::parse(&words(input)).unwrap() else { panic!() };
    request
}

fn programs() -> cargo_lust::Programs {
    cargo_lust::Programs { cargo: "unused cargo".into(), lust: "unused lust".into(), explicit_cargo: false, cranelift_backend: None }
}

#[test]
fn frontend_plan_preserves_cargo_and_compiler_argument_boundaries() {
    let input = request(&[
        "--bluefe-root",
        toolchain_root(),
        "--frontend-jobs",
        "4",
        "rustc",
        "-p",
        "package with spaces",
        "--lib",
        "--",
        "-Cdebuginfo=2",
        "--cfg",
        "project_setting=\"kept\"",
    ]);
    let selected = cargo_lust::plan(&input, programs()).unwrap();
    assert_eq!(
        selected.args,
        words(&[
            "rustc",
            "-p",
            "package with spaces",
            "--lib",
            "--",
            "-Cdebuginfo=2",
            "--cfg",
            "project_setting=\"kept\"",
            "-Zunstable-options",
            "--jobs-frontend=4",
        ])
    );
    assert!(selected.environment_remove.is_empty());
    assert_eq!(selected.environment.len(), 4);
    assert!(selected.experimental);
}

#[test]
fn frontend_option_requires_explicit_toolchain_and_target_command() {
    for input in [
        vec!["--bluefe-root", toolchain_root(), "--frontend-jobs", "0", "rustc"],
        vec!["--bluefe-root", toolchain_root(), "--frontend-jobs=-1", "rustc"],
        vec!["--bluefe-root", toolchain_root(), "--frontend-jobs"],
        vec!["--frontend-jobs", "4", "rustc"],
        vec!["--bluefe-root", toolchain_root(), "--frontend-jobs=4", "check"],
        vec!["--bluefe-root", toolchain_root(), "--frontend-jobs=4", "--engine", "cranelift", "rustc"],
        vec!["--bluefe-root", toolchain_root(), "--frontend-jobs=4", "--frontend-jobs=2", "rustc"],
    ] {
        assert!(cargo_lust::parse(&words(&input)).is_err(), "{input:?}");
    }
    for flags in [vec!["--jobs-frontend=2"], vec!["--jobs-frontend", "2"], vec!["-Zthreads=2"], vec!["-Z", "threads=2"]] {
        let mut input = vec!["--bluefe-root", toolchain_root(), "--frontend-jobs=4", "rustc", "--"];
        input.extend(flags);
        assert!(cargo_lust::parse(&words(&input)).is_err(), "{input:?}");
    }
    let mut forged = request(&["--bluefe-root", toolchain_root(), "rustc"]);
    forged.frontend_jobs = Some(0);
    assert!(cargo_lust::plan(&forged, programs()).is_err());
}

#[test]
fn frontend_plan_is_pure_and_inserts_only_one_argument_boundary() {
    let selected =
        cargo_lust::plan(&request(&["--bluefe-root", toolchain_root(), "--frontend-jobs=1", "rustc", "--profile", "check"]), programs())
            .unwrap();
    assert_eq!(selected.args, words(&["rustc", "--profile", "check", "--", "-Zunstable-options", "--jobs-frontend=1"]));
    let regular = request(&["rustc", "--", "--frontend-jobs=4"]);
    assert_eq!(regular.frontend_jobs, None);
    assert_eq!(cargo_lust::plan(&regular, programs()).unwrap().args, regular.args);
}

#[cfg(unix)]
#[test]
fn frontend_preserves_every_nonzero_argument_byte() {
    use std::os::unix::ffi::OsStringExt;
    for byte in 1..=255 {
        let argument = OsString::from_vec(vec![b'x', byte, b'y']);
        let mut input = words(&["--bluefe-root", toolchain_root(), "--frontend-jobs=4", "rustc", "--"]);
        input.push(argument.clone());
        let cargo_lust::Action::Execute(request) = cargo_lust::parse(&input).unwrap() else { panic!() };
        let plan = cargo_lust::plan(&request, programs()).unwrap();
        assert_eq!(plan.args[2], argument);
        assert_eq!(plan.args.len(), 5);
    }
}
