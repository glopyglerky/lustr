//! Build a local development toolchain from checked lustr source and the pinned
//! upstream Rust SDK. No predecessor lustr binary or historical package is used.
//! SDK, TinyCC and source library references remain explicit local dependencies.
use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};

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
    let version = checked("sdk-version", Command::new(sdk.join("bin/rustc")).arg("-vV"), &logs);
    let pin = fs::read_to_string(repo.join("upstream/bluefe-core.pin")).unwrap();
    let source = pin.lines().find_map(|line| line.strip_prefix("source = \"").and_then(|line| line.strip_suffix('"'))).unwrap();
    assert!(
        std::str::from_utf8(&version).unwrap().lines().any(|line| line == format!("commit-hash: {source}")),
        "select the matching pinned Rust SDK before building native components"
    );
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
    native.args(["--bin", "lust", "--bin", "lust-core"]).env("LUST_TCC_HOME", &tcc);
    checked("native-build", &mut native, &logs);
    let mut receipts = Vec::new();
    native_receipts(&work.join("native-intermediates").join(profile.directory()).join("build"), &mut receipts);
    assert_eq!(receipts.len(), 1, "one fresh native build receipt");
    let receipt = &receipts[0];
    let native_output = fs::read_to_string(receipt).unwrap();
    let hash = native_output.lines().find_map(|line| line.strip_prefix("cargo:rustc-env=LUST_BUILD_HASH=")).unwrap();
    assert!(hash.len() == 16 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
    let package = output.join("toolchain");
    fs::create_dir(&package).unwrap();
    fs::create_dir(package.join("bin")).unwrap();
    fs::create_dir(package.join("lib")).unwrap();
    for entry in fs::read_dir(sdk.join("lib")).unwrap() {
        let entry = entry.unwrap();
        symlink(entry.path(), package.join("lib").join(entry.file_name())).unwrap();
    }
    for entry in fs::read_dir(sdk.join("bin")).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if !["rustc", "rustc-full", "rustc-reference", "lustr", "lust", "lust-core", "cargo-lustr"].iter().any(|reserved| name == *reserved)
        {
            symlink(entry.path(), package.join("bin").join(name)).unwrap();
        }
    }
    symlink(sdk.join("bin/rustc"), package.join("bin/rustc-reference")).unwrap();
    fs::create_dir(package.join("lib/lust")).unwrap();
    symlink(repo.join("compiler/native/library"), package.join("lib/lust/library")).unwrap();
    symlink(&tcc, package.join("lib/lust/tcc")).unwrap();
    symlink(repo.join("library/lustr"), package.join("lib/lustr")).unwrap();
    symlink(&tcc, package.join("lib/tcc")).unwrap();
    for name in ["lust", "lust-core"] {
        copy(&work.join("native-target").join(profile.directory()).join(name), &package.join("bin").join(name));
    }
    let mut entry = cargo(&sdk, &repo, "Cargo.toml", &work, "entry", profile);
    entry
        .args(["--bin", "lustr", "--features", "source-native-runtime"])
        .env("LUSTR_CORE_ROOT", &sdk)
        .env("LUSTR_LUST_ROOT", &package)
        .env("LUSTR_NATIVE_BUILD_OUTPUT", receipt);
    checked("entry-build", &mut entry, &logs);
    copy(&work.join("entry-target").join(profile.directory()).join("lustr"), &package.join("bin/lustr"));
    symlink("lustr", package.join("bin/rustc")).unwrap();
    let mut coordinator = cargo(&sdk, &repo, "crates/lustr/Cargo.toml", &work, "coordinator", profile);
    coordinator.args(["--bin", "cargo-lustr"]);
    checked("cargo-entry-build", &mut coordinator, &logs);
    copy(&work.join("coordinator-target").join(profile.directory()).join("cargo-lustr"), &package.join("bin/cargo-lustr"));
    let mut library = cargo(&sdk, &repo, "crates/lustr-native-core/Cargo.toml", &work, "native-library", profile);
    library.env("LUST_TCC_HOME", &tcc);
    checked("native-library-build", &mut library, &logs);
    let library = package.join("lib/liblustr_native_core.so");
    copy(&work.join("native-library-target").join(profile.directory()).join("liblustr_native_core.so"), &library);
    let identity = checked("hash-native-library", Command::new("sha256sum").arg(&library), &logs);
    let identity = std::str::from_utf8(&identity).unwrap().split_whitespace().next().unwrap();
    record(&package.join("lib/lustr-native.id"), format!("{identity}\n").as_bytes());
    let mut mixed = cargo(&sdk, &repo, "crates/lustr-semantic/Cargo.toml", &work, "mixed", profile);
    mixed.args(["--features", "full-core", "--bin", "lustr-mixed"]).env("LUSTR_NATIVE_BUILD_HASH", hash);
    checked("mixed-build", &mut mixed, &logs);
    copy(&work.join("mixed-target").join(profile.directory()).join("lustr-mixed"), &package.join("bin/rustc-full"));
    // rustc-full resolves current_exe to this directory, alongside its matching
    // native core and the original compiler used for non-UTF8 OS arguments.
    record(&output.join("NATIVE-BUILD-RECEIPT.txt"), native_output.as_bytes());
    let mut manifest = format!(
        "Local source-built development candidate; release admission and broad mixed performance remain pending.\nReferenced source={}, SDK={}, TinyCC={}\nNative build={}\nCargo artifact profile={}\n",
        repo.display(),
        sdk.display(),
        tcc.display(),
        hash,
        profile.directory()
    );
    for name in ["lustr", "rustc-full", "lust", "lust-core", "cargo-lustr"] {
        let result = checked(&format!("hash-{name}"), Command::new("sha256sum").arg(package.join("bin").join(name)), &logs);
        manifest.push_str(std::str::from_utf8(&result).unwrap());
    }
    record(&output.join("COMPONENTS.txt"), manifest.as_bytes());
    for p in [package.join("bin"), package.join("lib"), package, logs, output.clone()] {
        fs::File::open(p).unwrap().sync_all().unwrap();
    }
    println!("Source-built local toolchain: {}", output.join("toolchain/bin/lustr").display());
    println!("Build caches are owned by this output scope. Retain artifacts and logs, then remove its work directory after qualification.");
}
