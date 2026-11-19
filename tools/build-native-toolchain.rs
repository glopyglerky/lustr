//! Build the partial native Lustr entry and runtime from checked source.
//! The pinned Rust SDK bootstraps the compiler; it is never a packaged fallback.
//! No complete Rust-replacement package is admitted by this constructor.
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};

#[path = "../src/host_runtime.rs"]
mod host_runtime;
#[path = "host-sdk.rs"]
mod host_sdk;
#[allow(dead_code)]
#[path = "../src/native_expansion.rs"]
mod native_expansion;
#[allow(dead_code)]
#[path = "../src/owned_child.rs"]
mod owned_child;
#[path = "selection-toolchain-receipt.rs"]
mod selection_toolchain;

#[derive(Clone, Copy)]
enum BuildProfile {
    Development,
    Release,
}
impl BuildProfile {
    fn directory(self) -> &'static str {
        match self {
            Self::Development => "debug",
            Self::Release => "release",
        }
    }
    fn configure(self, command: &mut Command) {
        if matches!(self, Self::Release) {
            command.arg("--release");
        }
    }
}

fn record(path: &Path, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}
fn checked(label: &str, command: &mut Command, logs: &Path) -> Vec<u8> {
    record(&logs.join(format!("{label}.command")), format!("{command:?}\n").as_bytes());
    let output = command.env_remove("LD_LIBRARY_PATH").output().unwrap();
    record(&logs.join(format!("{label}.stdout")), &output.stdout);
    record(&logs.join(format!("{label}.stderr")), &output.stderr);
    record(&logs.join(format!("{label}.status")), format!("{}\n", output.status).as_bytes());
    assert!(output.status.success(), "{label}: {}", String::from_utf8_lossy(&output.stderr));
    println!("{label}: {}", output.status);
    output.stdout
}
fn copy(source: &Path, target: &Path) {
    fs::copy(source, target).unwrap();
    fs::File::open(target).unwrap().sync_all().unwrap();
}
fn copy_sources(source: &Path, target: &Path) {
    fs::create_dir(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        let destination = target.join(entry.file_name());
        if kind.is_dir() {
            copy_sources(&entry.path(), &destination);
        } else {
            assert!(kind.is_file(), "installed adapter source must be regular: {}", entry.path().display());
            copy(&entry.path(), &destination);
        }
    }
    fs::File::open(target).unwrap().sync_all().unwrap();
}
fn source_library(repo: &Path, package: &Path) {
    let library = package.join("lib/lustr");
    fs::create_dir(&library).unwrap();
    copy_sources(&repo.join("library/lustr/std"), &library.join("std"));
    symlink("../lust/library/deps", library.join("deps")).unwrap();
    let expanded = package.join("lib/lustr-expanded");
    fs::create_dir_all(expanded.join("bin")).unwrap();
    fs::create_dir_all(expanded.join("lib/lust")).unwrap();
    symlink("../../../lustr", expanded.join("lib/lust/library")).unwrap();
    symlink("../../../lust/tcc", expanded.join("lib/lust/tcc")).unwrap();
    // The executable path selects the expanded support prefix, while its inode
    // must retain the original core's wire identity.
    fs::hard_link(package.join("bin/lust-core"), expanded.join("bin/lust-core")).unwrap();
    for name in ["lustr", "cargo"] {
        symlink(format!("../../../bin/{name}"), expanded.join("bin").join(name)).unwrap();
    }
}
fn seal_source_library(package: &Path, logs: &Path) -> String {
    let library = package.join("lib/lustr");
    let inputs = native_expansion::inputs(&library).unwrap();
    let manifest = checked("seal-native-sources", host_runtime::sha256_command().current_dir(&library).arg("--").args(&inputs), logs);
    record(&library.join("sources.sha256"), &manifest);
    native_expansion::sealed(&package.join("bin"), &package.join("source-library-identity.sock"))
        .expect("installed source library must satisfy the runtime's seal and core identity checks")
        .identity
}
fn native_receipts(directory: &Path, receipts: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if !path.file_name().is_some_and(|name| name == "out" || name == "fingerprint") {
                native_receipts(&path, receipts);
            }
        } else if path.file_name().is_some_and(|name| name == "output" || name == "stdout")
            && fs::read_to_string(&path).is_ok_and(|text| text.contains("cargo:rustc-env=LUST_BUILD_HASH="))
        {
            receipts.push(path);
        }
    }
}
fn cargo(sdk: &Path, repo: &Path, manifest: &str, work: &Path, name: &str, profile: BuildProfile) -> Command {
    let mut command = Command::new(sdk.join("bin/cargo"));
    command
        .current_dir(repo)
        .args(["build", "--manifest-path"])
        .arg(repo.join(manifest))
        .args(["--locked", "--offline", "--jobs", "2", "--target-dir"])
        .arg(work.join(format!("{name}-target")))
        .arg("--config")
        .arg(format!("build.build-dir={:?}", work.join(format!("{name}-intermediates"))))
        .env("RUSTC", sdk.join("bin/rustc"));
    profile.configure(&mut command);
    command
}
fn main() {
    let mut args = std::env::args_os().skip(1);
    let repo = PathBuf::from(args.next().expect("REPOSITORY RUST_SDK TINYCC NEW_OUTPUT [--release|--debug]")).canonicalize().unwrap();
    let sdk = args.next().unwrap();
    let sdk = if sdk == "--current-sdk" {
        let query = Command::new("rustc").args(["--print", "sysroot"]).env_remove("LD_LIBRARY_PATH").output().unwrap();
        assert!(query.status.success(), "cannot resolve the selected Rust SDK");
        PathBuf::from(String::from_utf8(query.stdout).unwrap().trim_end_matches(['\n', '\r']))
    } else {
        PathBuf::from(sdk)
    }
    .canonicalize()
    .unwrap();
    let tcc = PathBuf::from(args.next().unwrap()).canonicalize().unwrap();
    let output = PathBuf::from(args.next().unwrap());
    let profile = match args.next() {
        None => BuildProfile::Development,
        Some(flag) if flag == "--debug" => BuildProfile::Development,
        Some(flag) if flag == "--release" => BuildProfile::Release,
        Some(_) => panic!("expected --release or --debug after NEW_OUTPUT"),
    };
    assert!(args.next().is_none() && output.is_absolute());
    assert!(!output.exists(), "output must be new; existing data is preserved");
    assert!(!output.parent().unwrap().canonicalize().unwrap().starts_with(&repo), "keep generated output outside source");
    fs::DirBuilder::new().mode(0o700).create(&output).unwrap();
    let output = output.canonicalize().unwrap();
    let work = output.join("work");
    let logs = output.join("logs");
    fs::create_dir(&work).unwrap();
    fs::create_dir(&logs).unwrap();
    record(
        &output.join("BUILD-PROFILE.txt"),
        format!(
            "Cargo artifact profile: {}\nEach component retains its manifest's profile settings. The imported Rust SDK is unchanged.\nSelecting release builds optimized components; it does not establish performance qualification or release admission.\n",
            profile.directory()
        )
        .as_bytes(),
    );
    checked("source-revision", Command::new("git").arg("-C").arg(&repo).args(["rev-parse", "HEAD", "HEAD^{tree}"]), &logs);
    let source_status = checked(
        "source-status",
        Command::new("git").arg("-C").arg(&repo).args([
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            "Cargo.toml",
            "Cargo.lock",
            "build.rs",
            "src",
            "compiler/native",
            "crates/lustr-native-core",
            "library",
            "upstream",
            "tools/build-native-toolchain.rs",
            "tools/selection-toolchain-receipt.rs",
            "tools/host-sdk.rs",
            "tools/lust-upstream-check.rs",
        ]),
        &logs,
    );
    assert!(source_status.is_empty(), "commit the exact compiler/build inputs before constructing a source-identified package");
    let version = checked("sdk-version", Command::new(sdk.join("bin/rustc")).arg("-vV"), &logs);
    let pin = fs::read_to_string(repo.join("upstream/bluefe-core.pin")).unwrap();
    let source = pin.lines().find_map(|line| line.strip_prefix("source = \"").and_then(|line| line.strip_suffix('"'))).unwrap();
    assert!(
        std::str::from_utf8(&version).unwrap().lines().any(|line| line == format!("commit-hash: {source}")),
        "select the matching pinned Rust SDK before building native components"
    );
    let host = std::str::from_utf8(&version).unwrap().lines().find_map(|line| line.strip_prefix("host: ")).expect("SDK host triple");
    let expected_host = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        _ => panic!("the primary native runtime has no adapter for this build host"),
    };
    assert_eq!(host, expected_host, "select an SDK for this native build host");
    let host_receipt = output.join("HOST-SDK-RECEIPT.txt");
    record(&host_receipt, host_sdk::snapshot(&sdk, source, host).expect("record selected host SDK identity").as_bytes());
    let guard = work.join("source-guard");
    checked(
        "guard-build",
        Command::new(sdk.join("bin/rustc")).args(["--edition=2024"]).arg(repo.join("tools/lust-upstream-check.rs")).arg("-o").arg(&guard),
        &logs,
    );
    let mut check = Command::new(&guard);
    check.current_dir(&repo).arg("--candidate");
    if std::env::var_os("GIT_INDEX_FILE").is_some() {
        check.arg("--index");
    }
    checked("source-identity", &mut check, &logs);
    let mut native = cargo(&sdk, &repo, "compiler/native/Cargo.toml", &work, "native", profile);
    native.args(["--bin", "lust", "--bin", "lust-core", "--features", "exclusive-native"]).env("LUST_TCC_HOME", &tcc);
    checked("native-build", &mut native, &logs);
    let mut receipts = Vec::new();
    native_receipts(&work.join("native-intermediates").join(profile.directory()).join("build"), &mut receipts);
    assert_eq!(receipts.len(), 1, "one fresh native build receipt");
    let receipt = &receipts[0];
    let native_output = fs::read_to_string(receipt).unwrap();
    assert!(native_output.lines().any(|line| line == "cargo:rustc-env=LUST_EXCLUSIVE_NATIVE=1"));
    let hash = native_output.lines().find_map(|line| line.strip_prefix("cargo:rustc-env=LUST_BUILD_HASH=")).unwrap();
    assert!(hash.len() == 16 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
    let package = output.join("toolchain");
    fs::create_dir(&package).unwrap();
    fs::write(
        package.join("PARTIAL-NATIVE-COMPILER"),
        "Native Lustr development build. Complementary compilation and complete-replacement release are unavailable.\n",
    )
    .unwrap();
    fs::create_dir(package.join("bin")).unwrap();
    fs::create_dir(package.join("lib")).unwrap();
    // Cargo is used only for native package/target discovery. No rustc,
    // rustdoc, compiler driver or LLVM runtime is copied into this package.
    symlink(sdk.join("bin/cargo"), package.join("bin/cargo")).unwrap();
    fs::create_dir(package.join("lib/lust")).unwrap();
    symlink(repo.join("compiler/native/library"), package.join("lib/lust/library")).unwrap();
    symlink(&tcc, package.join("lib/lust/tcc")).unwrap();
    symlink(&tcc, package.join("lib/tcc")).unwrap();
    for name in ["lust", "lust-core"] {
        copy(&work.join("native-target").join(profile.directory()).join(name), &package.join("bin").join(name));
    }
    source_library(&repo, &package);
    let mut entry = cargo(&sdk, &repo, "Cargo.toml", &work, "entry", profile);
    entry
        .args(["--bin", "lustr", "--features", "source-native-runtime"])
        .env("LUSTR_CORE_ROOT", &sdk)
        .env("LUSTR_LUST_ROOT", &package)
        .env("LUSTR_NATIVE_BUILD_OUTPUT", receipt)
        .env("LUSTR_HOST_SDK_RECEIPT", &host_receipt);
    checked("entry-build", &mut entry, &logs);
    copy(&work.join("entry-target").join(profile.directory()).join("lustr"), &package.join("bin/lustr"));
    symlink("lustr", package.join("bin/rustc")).unwrap();
    let mut library = cargo(&sdk, &repo, "crates/lustr-native-core/Cargo.toml", &work, "native-library", profile);
    library.args(["--features", "exclusive-native"]).env("LUST_TCC_HOME", &tcc);
    checked("native-library-build", &mut library, &logs);
    let library = package.join("lib").join(host_runtime::NATIVE_LIBRARY);
    copy(&work.join("native-library-target").join(profile.directory()).join(host_runtime::NATIVE_LIBRARY), &library);
    let identity = checked("hash-native-library", host_runtime::sha256_command().arg(&library), &logs);
    let identity = std::str::from_utf8(&identity).unwrap().split_whitespace().next().unwrap();
    record(&package.join("lib/lustr-native.id"), format!("{identity}\n").as_bytes());
    selection_toolchain::write(&package, source, host);
    if host == "x86_64-unknown-linux-gnu" {
        checked("prime-selection-tools", Command::new(package.join("bin/lustr")).arg("--lustr-prime-selection-tools"), &logs);
    }
    let source_identity = seal_source_library(&package, &logs);
    let mixed_route = "exclusive native Lust; complementary compilation unavailable";
    record(&output.join("NATIVE-BUILD-RECEIPT.txt"), native_output.as_bytes());
    let mut manifest = format!(
        "Partial native development build; complementary compilation and full-replacement release are unavailable.\nReferenced source={}, SDK={}, TinyCC={}\nNative build={}\nCargo artifact profile={}\nSource library identity={}\nHost={}\nRoute={}\nSDK receipt={}\n",
        repo.display(),
        sdk.display(),
        tcc.display(),
        hash,
        profile.directory(),
        source_identity,
        host,
        mixed_route,
        host_receipt.display()
    );
    for name in ["lustr", "lust", "lust-core"] {
        let result = checked(&format!("hash-{name}"), host_runtime::sha256_command().arg(package.join("bin").join(name)), &logs);
        manifest.push_str(std::str::from_utf8(&result).unwrap());
    }
    record(&output.join("COMPONENTS.txt"), manifest.as_bytes());
    for p in [package.join("bin"), package.join("lib"), package, logs, output.clone()] {
        fs::File::open(p).unwrap().sync_all().unwrap();
    }
    println!("Source-built local toolchain: {}", output.join("toolchain/bin/lustr").display());
    println!("Build caches are owned by this output scope. Retain artifacts and logs, then remove its work directory after qualification.");
}
