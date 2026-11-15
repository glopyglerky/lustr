//! Bounded Linux native-delivery checks. Generated output stays in an explicit
//! build filesystem; this tool changes no installed/default toolchain.
use serde_json::json;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::Instant;

#[allow(dead_code)]
#[path = "../src/native_expansion.rs"]
mod native_expansion;
#[allow(dead_code)]
#[path = "../src/owned_child.rs"]
mod owned_child;

fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        std::os::unix::fs::symlink(fs::read_link(source)?, destination)?;
    } else if metadata.is_dir() {
        fs::create_dir(destination)?;
        for item in fs::read_dir(source)? {
            let item = item?;
            copy_tree(&item.path(), &destination.join(item.file_name()))?;
        }
        fs::set_permissions(destination, metadata.permissions())?;
    } else if metadata.is_file() {
        fs::copy(source, destination)?;
    } else {
        return Err(std::io::Error::other("nonregular delivery input"));
    }
    Ok(())
}

fn sandbox(root: &Path, package: &str) -> Vec<OsString> {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // A remapped source name is not a host path; callers run from the source
    // checkout and it must agree with Cargo.toml before any build is started.
    let source = if source.is_dir() { source } else { std::env::current_dir().unwrap() };
    assert!(source.join("src/main.rs").is_file());
    let mut args: Vec<OsString> = [
        "-c",
        "10-13",
        "bwrap",
        "--die-with-parent",
        "--new-session",
        "--unshare-pid",
        "--ro-bind",
        "/usr",
        "/usr",
        "--ro-bind",
        "/etc",
        "/etc",
        "--symlink",
        "usr/bin",
        "/bin",
        "--symlink",
        "usr/lib",
        "/lib",
        "--symlink",
        "usr/lib64",
        "/lib64",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--tmpfs",
        "/tmp",
        "--ro-bind",
    ]
    .map(OsString::from)
    .to_vec();
    args.extend([
        source.into_os_string(),
        "/lustr".into(),
        "--ro-bind".into(),
        root.join("artifacts/upstream-toolchain").into_os_string(),
        "/core".into(),
        "--ro-bind".into(),
        std::env::var_os("LUSTR_PRISTINE_ROOT").expect("LUSTR_PRISTINE_ROOT must identify the pinned pristine runtime"),
        "/lust".into(),
        "--ro-bind".into(),
        root.join("artifacts").join(package).into_os_string(),
        "/package".into(),
        "--bind".into(),
        root.join("build-v3").into_os_string(),
        "/build".into(),
    ]);
    args.extend(
        [
            "--clearenv",
            "--setenv",
            "PATH",
            "/core/bin:/usr/bin:/bin",
            "--setenv",
            "HOME",
            "/build/home",
            "--setenv",
            "TMPDIR",
            "/build/tmp",
            "--setenv",
            "CARGO_HOME",
            "/build/cargo-home",
            "--setenv",
            "CARGO_TARGET_DIR",
            "/build/target-driver",
            "--setenv",
            "RUSTC",
            "/core/bin/rustc",
            "--setenv",
            "RUSTDOC",
            "/core/bin/rustdoc",
            "--setenv",
            "LUSTR_CORE_ROOT",
            "/core",
            "--setenv",
            "LUSTR_LUST_ROOT",
            "/lust",
            "--setenv",
            "LUSTR_TEST_PACKAGE",
            "/package",
            "--setenv",
            "RUSTFLAGS",
            "--remap-path-prefix=/lustr=/src/lustr --remap-path-prefix=/build=/build/lustr",
            "--chdir",
            "/lustr",
        ]
        .map(OsString::from),
    );
    args
}

fn run(root: &Path, label: &str, args: &[OsString]) -> ExitStatus {
    use std::io::Write;
    let mut log = fs::OpenOptions::new().create_new(true).write(true).open(root.join("logs").join(format!("{label}.log"))).unwrap();
    println!("Starting {label}");
    let start = Instant::now();
    let result = owned_child::output(Command::new("taskset").args(args)).unwrap();
    log.write_all(&result.stdout).unwrap();
    log.write_all(&result.stderr).unwrap();
    log.sync_all().unwrap();
    let receipt = json!({"command": std::iter::once(OsString::from("taskset")).chain(args.iter().cloned()).collect::<Vec<_>>(), "elapsed_seconds": start.elapsed().as_secs_f64(), "success": result.status.success(), "exit": result.status.code()});
    let mut receipt_file =
        fs::OpenOptions::new().create_new(true).write(true).open(root.join("private").join(format!("{label}.json"))).unwrap();
    receipt_file.write_all(serde_json::to_string_pretty(&receipt).unwrap().as_bytes()).unwrap();
    receipt_file.sync_all().unwrap();
    println!("{label}: {}", result.status);
    print!("{}", String::from_utf8_lossy(&result.stderr));
    print!("{}", String::from_utf8_lossy(&result.stdout));
    result.status
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(args.len() >= 3, "native_qualification ROOT PACKAGE assemble|build|test [UNIQUE_LABEL] [TEST_FILTER]");
    let root = Path::new(&args[0]).canonicalize().unwrap();
    let package = &args[1];
    assert!(!package.is_empty() && package.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    match args[2].as_str() {
        "seal-library" => {
            let runtime = root.join("artifacts").join(package);
            let expanded = runtime.join("lib/lustr-expanded");
            fs::create_dir_all(expanded.join("bin")).unwrap();
            fs::create_dir_all(expanded.join("lib/lust")).unwrap();
            fs::hard_link(runtime.join("bin/lust-core"), expanded.join("bin/lust-core")).unwrap();
            std::os::unix::fs::symlink("../../../lustr", expanded.join("lib/lust/library")).unwrap();
            std::os::unix::fs::symlink("../../../lust/tcc", expanded.join("lib/lust/tcc")).unwrap();
            let library = runtime.join("lib/lustr");
            let inputs = native_expansion::inputs(&library).unwrap();
            let output = Command::new("sha256sum").current_dir(&library).args(inputs).output().unwrap();
            assert!(output.status.success());
            fs::write(library.join("sources.sha256"), output.stdout).unwrap();
        }
        "assemble" => {
            let destination = root.join("artifacts").join(package);
            copy_tree(&root.join("artifacts/native-toolchain-v20"), &destination).unwrap();
            fs::create_dir(destination.join("lib/lustr")).unwrap();
            fs::create_dir(destination.join("lib/lustr/std")).unwrap();
            copy_tree(&std::env::current_dir().unwrap().join("library/lustr/std/src"), &destination.join("lib/lustr/std/src")).unwrap();
            std::os::unix::fs::symlink("../lust/library/deps", destination.join("lib/lustr/deps")).unwrap();
            println!("Prepared {package}; build must install the matching entry before qualification");
        }
        operation @ ("build" | "build-examples" | "test" | "measure") => {
            let label = &args[3];
            assert!(label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
            let mut command = sandbox(&root, package);
            if operation == "test" {
                command.extend(["--setenv".into(), "LUSTR_TEST_OUTPUT".into(), format!("/build/contract {label} under spaces").into()]);
            }
            if operation == "measure" {
                command.extend(["--ro-bind".into(), root.join("artifacts/matched-native-baseline").into_os_string(), "/matched".into()]);
                command.extend([
                    "timeout".into(),
                    "--kill-after=5".into(),
                    "240".into(),
                    "/build/target-driver/release/examples/native_measurement".into(),
                    format!("/build/{label}").into(),
                ]);
                let status = run(&root, label, &command);
                std::process::exit(status.code().unwrap_or(2));
            }
            command.push("/core/bin/cargo".into());
            command.push(if operation == "build-examples" { "build".into() } else { operation.into() });
            command.extend(
                ["--config", "build.build-dir=\"/build/cargo-build-driver\"", "--locked", "--offline", "--jobs", "2"].map(OsString::from),
            );
            if operation == "build" {
                command.extend(["--release".into(), "--bin".into(), "lustr".into()]);
            } else if operation == "build-examples" {
                command.extend(["--release".into(), "--examples".into()]);
            } else {
                if let Some(filter) = args.get(4) {
                    command.extend(["--test".into(), "compiler_contract".into(), filter.into()]);
                }
                command.extend(["--", "--include-ignored", "--test-threads=1"].map(OsString::from));
            }
            let status = run(&root, label, &command);
            if !status.success() {
                std::process::exit(status.code().unwrap_or(2));
            }
            if operation == "build" {
                let compiled = root.join("build-v3/target-driver/release/lustr");
                for name in ["lustr", "rustc"] {
                    fs::copy(&compiled, root.join("artifacts").join(package).join("bin").join(name)).unwrap();
                }
            }
        }
        _ => panic!("unknown qualification operation"),
    }
}
