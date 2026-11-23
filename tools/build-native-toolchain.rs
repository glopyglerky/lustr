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
fn observe(label: &str, command: &mut Command, logs: &Path) -> std::io::Result<std::process::Output> {
    record(&logs.join(format!("{label}.command")), format!("{command:?}\n").as_bytes());
    if !command.get_envs().any(|(name, _)| name == "LD_LIBRARY_PATH") {
        command.env_remove("LD_LIBRARY_PATH");
    }
    let output = command.output();
    let status = match &output {
        Ok(output) => {
            record(&logs.join(format!("{label}.stdout")), &output.stdout);
            record(&logs.join(format!("{label}.stderr")), &output.stderr);
            output.status.to_string()
        }
        Err(error) => {
            record(&logs.join(format!("{label}.stdout")), &[]);
            record(&logs.join(format!("{label}.stderr")), error.to_string().as_bytes());
            "unlaunched".into()
        }
    };
    record(&logs.join(format!("{label}.status")), format!("{status}\n").as_bytes());
    println!("{label}: {status}");
    output
}
fn checked(label: &str, command: &mut Command, logs: &Path) -> Vec<u8> {
    let output = observe(label, command, logs).unwrap();
    assert!(output.status.success(), "{label}: {}", String::from_utf8_lossy(&output.stderr));
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
// This governs construction of the compiler package, never a user's Rust
// project. The source manifest and selected SDK must determine compiler inputs;
// implicit Cargo settings cannot replace them with an unrecorded compiler,
// target, source tree, feature cfg or optimization profile.
fn construction_inputs(repo: &Path) -> Result<(), String> {
    for (name, value) in std::env::vars_os() {
        let Some(name) = name.to_str() else { continue };
        let changes_build = matches!(
            name,
            "RUSTFLAGS"
                | "CARGO_ENCODED_RUSTFLAGS"
                | "RUSTC_BOOTSTRAP"
                | "RUSTC_WRAPPER"
                | "RUSTC_WORKSPACE_WRAPPER"
                | "CARGO_BUILD_RUSTC"
                | "CARGO_BUILD_RUSTC_WRAPPER"
                | "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"
                | "CARGO_BUILD_RUSTFLAGS"
                | "CARGO_BUILD_TARGET"
        ) || name.starts_with("CARGO_PROFILE_")
            || name.starts_with("CARGO_TARGET_") && name != "CARGO_TARGET_DIR"
            || name.starts_with("CARGO_SOURCE_")
            || name.starts_with("CARGO_REGISTRIES_") && name.ends_with("_INDEX")
            || name == "CARGO_REGISTRY_INDEX"
            || name.starts_with("CARGO_UNSTABLE_");
        if changes_build && !value.is_empty() {
            return Err(format!("unreviewed compiler-construction override {name}; use the reviewed source manifests and selected SDK"));
        }
    }
    let mut directories: Vec<_> = repo.ancestors().map(|directory| directory.join(".cargo")).collect();
    let home = std::env::var_os("CARGO_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")));
    if let Some(home) = home {
        // Every Cargo child below runs from the repository. A relative
        // CARGO_HOME is resolved there, not in this constructor's caller cwd.
        directories.push(if home.is_absolute() { home } else { repo.join(home) });
    }
    for directory in directories {
        for name in ["config", "config.toml"] {
            let path = directory.join(name);
            match fs::symlink_metadata(&path) {
                Ok(_) => return Err(format!("unreviewed compiler-construction Cargo configuration: {}", path.display())),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(format!("cannot inspect compiler-construction Cargo configuration {}: {error}", path.display())),
            }
        }
    }
    Ok(())
}
fn cargo(sdk: &Path, repo: &Path, manifest: &str, package: &str, work: &Path, name: &str, profile: BuildProfile) -> Command {
    let mut command = Command::new(sdk.join("bin/cargo"));
    command
        .current_dir(repo)
        .args(["build", "--manifest-path"])
        .arg(repo.join(manifest))
        .args(["--package", package])
        .args(["--locked", "--offline", "--jobs", "2", "--target-dir"])
        .arg(work.join(format!("{name}-target")))
        .arg("--config")
        .arg(format!("build.build-dir={:?}", work.join(format!("{name}-intermediates"))))
        .env("RUSTC", sdk.join("bin/rustc"));
    profile.configure(&mut command);
    command
}
fn main() {
    let mut args = std::env::args_os().skip(1).peekable();
    let repo = PathBuf::from(args.next().expect(
        "REPOSITORY RUST_SDK TINYCC NEW_OUTPUT [--release|--debug] [--rustdoc-source PINNED_RUST_REPOSITORY] [--proc-macro-source PINNED_RUST_REPOSITORY]",
    )).canonicalize().unwrap();
    construction_inputs(&repo).unwrap_or_else(|error| panic!("{error}"));
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
    let profile = match args.peek() {
        Some(flag) if flag == "--release" => {
            args.next();
            BuildProfile::Release
        }
        Some(flag) if flag == "--debug" => {
            args.next();
            BuildProfile::Development
        }
        _ => BuildProfile::Development,
    };
    let mut rustdoc_source = None;
    let mut proc_macro_source = None;
    while let Some(flag) = args.next() {
        let source = if flag == "--rustdoc-source" {
            &mut rustdoc_source
        } else if flag == "--proc-macro-source" {
            &mut proc_macro_source
        } else {
            panic!("unknown source component option {flag:?}");
        };
        assert!(source.is_none(), "source component option repeated");
        *source = Some(PathBuf::from(args.next().expect("pinned Rust source repository"))
            .canonicalize().unwrap());
    }
    assert!(output.is_absolute());
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
            "crates/lustr-metadata-encoder",
            "crates/lustr-metadata-sdk",
            "docs/compiler-set/evidence/metadata-sdk-receipt-20261120/receipt.json",
            "tools/install-native-metadata-sdk.py",
            "upstream",
            "tools/build-native-toolchain.rs",
            "tools/selection-toolchain-receipt.rs",
            "tools/host-sdk.rs",
            "tools/host-proc-macro.rs",
            "tools/host-crate-interface.rs",
            "tools/host-freestanding.rs",
            "tools/lust-upstream-check.rs",
            "tools/check-exclusive-coverage.py",
        ]),
        &logs,
    );
    assert!(source_status.is_empty(), "commit the exact compiler/build inputs before constructing a source-identified package");
    checked(
        "exclusive-coverage-admission",
        Command::new("python3").arg(repo.join("tools/check-exclusive-coverage.py")).arg("--repository").arg(&repo),
        &logs,
    );
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
    if rustdoc_source.is_some() {
        assert_eq!(host, "x86_64-unknown-linux-gnu", "the source-built Rustdoc component is available on Linux x86_64");
    }
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
    let mut native = cargo(&sdk, &repo, "compiler/native/Cargo.toml", "lust", &work, "native", profile);
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
    // Cargo is the build orchestrator and package selector. The bounded
    // metadata SDK adds only core/builtins data and artifact-tool dependencies;
    // no Rust compiler, compiler driver or research caller is packaged.
    symlink(sdk.join("bin/cargo"), package.join("bin/cargo")).unwrap();
    fs::create_dir(package.join("lib/lust")).unwrap();
    symlink(repo.join("compiler/native/library"), package.join("lib/lust/library")).unwrap();
    symlink(&tcc, package.join("lib/lust/tcc")).unwrap();
    symlink(&tcc, package.join("lib/tcc")).unwrap();
    for name in ["lust", "lust-core"] {
        copy(&work.join("native-target").join(profile.directory()).join(name), &package.join("bin").join(name));
    }
    let mut entry = cargo(&sdk, &repo, "Cargo.toml", "lustr-compiler", &work, "entry", profile);
    entry
        .args(["--bin", "lustr", "--features", "source-native-runtime"])
        .env("LUSTR_CORE_ROOT", &sdk)
        .env("LUSTR_LUST_ROOT", &package)
        .env("LUSTR_NATIVE_BUILD_OUTPUT", receipt)
        .env("LUSTR_HOST_SDK_RECEIPT", &host_receipt);
    checked("entry-build", &mut entry, &logs);
    copy(&work.join("entry-target").join(profile.directory()).join("lustr"), &package.join("bin/lustr"));
    symlink("lustr", package.join("bin/rustc")).unwrap();
    let mut library = cargo(&sdk, &repo, "crates/lustr-native-core/Cargo.toml", "lustr-native-core", &work, "native-library", profile);
    library.args(["--lib", "--features", "exclusive-native"]).env("LUST_TCC_HOME", &tcc);
    checked("native-library-build", &mut library, &logs);
    let library = package.join("lib").join(host_runtime::NATIVE_LIBRARY);
    copy(&work.join("native-library-target").join(profile.directory()).join(host_runtime::NATIVE_LIBRARY), &library);
    let identity = checked("hash-native-library", host_runtime::sha256_command().arg(&library), &logs);
    let identity = std::str::from_utf8(&identity).unwrap().split_whitespace().next().unwrap();
    record(&package.join("lib/lustr-native.id"), format!("{identity}\n").as_bytes());
    selection_toolchain::write(&package, source, host);
    let mut runtime_identities = Vec::new();
    if host == "x86_64-unknown-linux-gnu" {
        let component = package.join("libexec/lustr-host-proc-macro");
        let bin = component.join("bin");
        fs::create_dir_all(&bin).unwrap();
        symlink(sdk.join("lib"), component.join("lib")).unwrap();
        let compiler = bin.join("lustr-rustc-host-proc-macro");
        let mut flags = std::ffi::OsString::from("-Clink-arg=-Wl,-rpath,$ORIGIN/../lib\x1f-Lnative=");
        flags.push(sdk.join("lib"));
        checked("host-proc-macro-build",
            cargo(&sdk, &repo, "crates/lustr-native-core/Cargo.toml", "lustr-native-core", &work, "host-compiler", profile)
                .args(["--features", "upstream-semantic", "--example", "host-proc-macro"])
                .env("LUST_TCC_HOME", &tcc).env("CARGO_ENCODED_RUSTFLAGS", flags), &logs);
        copy(&work.join("host-compiler-target").join(profile.directory()).join("examples/host-proc-macro"),
            &compiler);
        let companion_version = checked("host-proc-macro-version", Command::new(&compiler).arg("-vV"), &logs);
        assert_eq!(companion_version, version, "the macro compiler must use the declared SDK");
        let licenses = component.join("licenses");
        fs::create_dir(&licenses).unwrap();
        for name in ["LICENSE-MIT", "LICENSE-APACHE"] {
            copy(&repo.join("crates/lustr-metadata-encoder").join(name), &licenses.join(name));
        }
        let notice = component.join("SOURCE.txt");
        record(&notice, format!(
            "https://github.com/rust-lang/rust\nRevision {source}\nPinned Rust driver for host macros, external declarations and composed Cargo units\nSelected Lust bodies use the matching native compiler and TinyCC in the same process.\n",
        ).as_bytes());
        let identity = checked("hash-host-proc-macro",
            host_runtime::sha256_command().arg(&compiler).arg(notice), &logs);
        let digest = std::str::from_utf8(&identity).unwrap().split_whitespace().next().unwrap();
        record(&component.join("metadata-interface.id"), format!("{digest}\n").as_bytes());
        runtime_identities.push(identity);
        fs::File::open(&compiler).unwrap().sync_all().unwrap();
        for path in [licenses, bin, component, package.join("libexec")] {
            fs::File::open(path).unwrap().sync_all().unwrap();
        }
        checked("metadata-sdk-package", Command::new("python3").arg(repo.join("tools/install-native-metadata-sdk.py"))
            .arg(&repo).arg(&sdk).arg(&package), &logs);
        for tool in ["llvm-ar","llvm-objcopy"] {
            checked(&format!("installed-{tool}"),Command::new(package.join("lib/rustlib/x86_64-unknown-linux-gnu/bin").join(tool)).arg("--version"),&logs);
        }
        checked("prime-selection-tools", Command::new(package.join("bin/lustr")).arg("--lustr-prime-selection-tools"), &logs);
        let runtime = package.join("lib/liblustr_test_runtime.a");
        checked("native-test-runtime", Command::new(sdk.join("bin/rustc"))
            .env_remove("LD_LIBRARY_PATH")
            .args(["--edition=2024", "--crate-type=staticlib", "-Cpanic=unwind", "-Copt-level=2", "--crate-name=lustr_test_runtime"])
            .arg(repo.join("tools/native-test-runtime.rs")).arg("-o").arg(&runtime), &logs);
        runtime_identities.push(checked("hash-native-test-runtime", host_runtime::sha256_command().arg(runtime), &logs));
        let runtime_directory = package.join("lib/lustr-cargo-runtime");
        assert!(!runtime_directory.exists());
        let runtime_build = observe("prime-cargo-runtime",
            Command::new(package.join("bin/lust-core")).arg("--lustr-prime-cargo-runtime"), &logs);
        if runtime_directory.is_dir() {
            if runtime_build.is_ok_and(|output| output.status.success()) {
                let mut objects: Vec<_> = fs::read_dir(&runtime_directory).unwrap()
                    .map(|entry| entry.unwrap().path())
                    .filter(|path| path.extension().is_some_and(|ext| ext == "o")).collect();
                objects.sort();
                assert_eq!(objects.len(), 2);
                for (index, object) in objects.iter().enumerate() {
                    let receipt = object.with_extension("receipt");
                    for path in [object, &receipt] {
                        fs::File::open(path).unwrap().sync_all().unwrap();
                    }
                    runtime_identities.push(checked(&format!("hash-cargo-runtime-{index}"),
                        host_runtime::sha256_command().arg(object).arg(receipt), &logs));
                }
                fs::File::open(runtime_directory).unwrap().sync_all().unwrap();
            } else {
                fs::remove_dir_all(&runtime_directory).unwrap();
            }
        }
        let licenses = package.join("lib/lustr-test-licenses");
        fs::create_dir(&licenses).unwrap();
        for name in ["LICENSE-MIT", "LICENSE-APACHE"] {
            copy(&repo.join("crates/lustr-metadata-encoder").join(name), &licenses.join(name));
        }
        record(&licenses.join("SOURCE.txt"), format!("https://github.com/rust-lang/rust\nRevision {source}\nlibrary/test and its standard library runtime dependencies\n").as_bytes());
    }
    if let Some(proc_macro_source) = proc_macro_source {
        assert_eq!(host, "x86_64-unknown-linux-gnu", "the macro artifact adapter requires Linux x86_64");
        let component_work = work.join("proc-macro-expansion");
        fs::create_dir(&component_work).unwrap();
        let checkout = component_work.join("rust-source");
        checked("proc-macro-source-copy", Command::new("git")
            .args(["clone", "--local", "--shared", "--no-checkout", "--"])
            .arg(proc_macro_source).arg(&checkout), &logs);
        checked("proc-macro-source-pin", Command::new("git").current_dir(&checkout)
            .args(["checkout", "--detach", source]), &logs);
        construction_inputs(&checkout).unwrap_or_else(|error| panic!("{error}"));
        let patch = repo.join("upstream/proc-macro-expansion.patch");
        checked("proc-macro-patch", Command::new("git").current_dir(&checkout)
            .arg("apply").arg(&patch), &logs);

        let manifests = repo.join("tools/proc-macro-expansion");
        for name in ["client", "server"] {
            let destination = component_work.join(name);
            fs::create_dir(&destination).unwrap();
            for file in ["Cargo.toml", "Cargo.lock"] {
                copy(&manifests.join(name).join(file), &destination.join(file));
            }
        }
        fs::create_dir(component_work.join("client/src")).unwrap();
        copy(&manifests.join("client/src/main.rs"), &component_work.join("client/src/main.rs"));
        let mut flags = std::ffi::OsString::from("-Clink-arg=-Wl,-rpath,$ORIGIN/../lib\x1f-Lnative=");
        flags.push(sdk.join("lib"));
        checked("proc-macro-server-build", cargo(&sdk, &component_work, "server/Cargo.toml", "proc-macro-srv-cli",
            &work, "proc-macro-server", profile).args(["--bin", "rust-analyzer-proc-macro-srv", "--features", "in-rust-tree"])
            .env("CARGO_ENCODED_RUSTFLAGS", &flags).env("LD_LIBRARY_PATH", sdk.join("lib")), &logs);
        checked("proc-macro-client-build", cargo(&sdk, &component_work, "client/Cargo.toml", "lustr-expand-proc-macro",
            &work, "proc-macro-client", profile).args(["--bin", "lustr-expand-proc-macro"])
            .env("CARGO_ENCODED_RUSTFLAGS", &flags).env("LD_LIBRARY_PATH", sdk.join("lib")), &logs);

        let component = package.join("libexec/lustr-macro-expansion");
        fs::create_dir_all(component.join("bin")).unwrap();
        symlink(sdk.join("lib"), component.join("lib")).unwrap();
        let server = component.join("bin/rust-analyzer-proc-macro-srv");
        let client = component.join("bin/lustr-expand-proc-macro");
        copy(&work.join("proc-macro-server-target").join(profile.directory()).join("rust-analyzer-proc-macro-srv"), &server);
        copy(&work.join("proc-macro-client-target").join(profile.directory()).join("lustr-expand-proc-macro"), &client);
        checked("installed-proc-macro-server", Command::new(&server).arg("--version")
            .env("RUST_ANALYZER_INTERNALS_DO_NOT_USE", "1"), &logs);
        let licenses = component.join("licenses");
        fs::create_dir(&licenses).unwrap();
        for name in ["COPYRIGHT", "LICENSE-MIT", "LICENSE-APACHE"] {
            copy(&checkout.join(name), &licenses.join(name));
        }
        for name in ["client", "server"] {
            copy(&manifests.join(name).join("Cargo.lock"), &licenses.join(format!("{name}.Cargo.lock")));
        }
        let notice = component.join("SOURCE.txt");
        record(&notice, format!(concat!(
            "https://github.com/rust-lang/rust\nRevision {source}\n",
            "Rust Analyzer procedural macro worker and client with upstream/proc-macro-expansion.patch\n",
            "The native adapter represents direct function-like calls with source token trees and expansion output containing call-site identifiers, represented literals, punctuation and visible delimiter groups.\n",
        ), source = source).as_bytes());
        runtime_identities.push(checked("hash-proc-macro-expansion", host_runtime::sha256_command()
            .arg(server).arg(client).arg(&patch).arg(notice), &logs));
        for path in [licenses, component.join("bin"), component, package.join("libexec")] {
            fs::File::open(path).unwrap().sync_all().unwrap();
        }
    }
    if let Some(rustdoc_source) = rustdoc_source {
        let checkout = work.join("rustdoc-source");
        checked("rustdoc-source-copy", Command::new("git")
            .args(["clone", "--local", "--shared", "--no-checkout", "--"])
            .arg(rustdoc_source).arg(&checkout), &logs);
        checked("rustdoc-source-pin", Command::new("git").current_dir(&checkout)
            .args(["checkout", "--detach", source]), &logs);
        construction_inputs(&checkout).unwrap_or_else(|error| panic!("{error}"));
        let patch = repo.join("upstream/rustdoc-test-builders.patch");
        checked("rustdoc-patch", Command::new("git").current_dir(&checkout)
            .arg("apply").arg(&patch), &logs);
        let mut flags = std::ffi::OsString::from("-Clink-arg=-Wl,-rpath,$ORIGIN/../lib\x1f-Lnative=");
        flags.push(sdk.join("lib"));
        checked("rustdoc-build", cargo(&sdk, &checkout, "Cargo.toml", "rustdoc-tool", &work, "rustdoc", profile)
            .env("CARGO_ENCODED_RUSTFLAGS", flags)
            .env("LD_LIBRARY_PATH", sdk.join("lib"))
            .env("DOC_RUST_LANG_ORG_CHANNEL", "https://doc.rust-lang.org/nightly"), &logs);

        let component = package.join("libexec/lustr-rustdoc");
        fs::create_dir_all(component.join("bin")).unwrap();
        symlink(sdk.join("lib"), component.join("lib")).unwrap();
        let rustdoc = component.join("bin/rustdoc");
        let validator = component.join("bin/lustr-rustdoc-compile-fail");
        copy(&work.join("rustdoc-target").join(profile.directory()).join("rustdoc-tool-binary"), &rustdoc);
        fs::hard_link(&rustdoc, &validator).unwrap();
        symlink("../libexec/lustr-rustdoc/bin/rustdoc", package.join("bin/rustdoc")).unwrap();
        checked("installed-rustdoc-version", Command::new(&rustdoc).arg("--version"), &logs);
        let validator_version = checked("installed-rustdoc-validator-version", Command::new(&validator).arg("-vV"), &logs);
        assert_eq!(validator_version, version, "Rustdoc's validator must use the declared SDK compiler");
        let licenses = component.join("licenses");
        fs::create_dir(&licenses).unwrap();
        for name in ["COPYRIGHT", "LICENSE-MIT", "LICENSE-APACHE", "Cargo.lock"] {
            copy(&checkout.join(name), &licenses.join(name));
        }
        record(&component.join("SOURCE.txt"), format!(concat!(
            "https://github.com/rust-lang/rust\nRevision {source}\n",
            "Patch upstream/rustdoc-test-builders.patch\n",
            "Rustdoc uses the matching SDK for documentation analysis and compile-fail validation.\n",
            "Other test bodies request the installed Lustr compiler.\n",
        ), source = source).as_bytes());
        runtime_identities.push(checked("hash-rustdoc", host_runtime::sha256_command()
            .arg(&rustdoc).arg(&patch).arg(component.join("SOURCE.txt")), &logs));
        for path in [licenses, component.join("bin"), component, package.join("libexec")] {
            fs::File::open(path).unwrap().sync_all().unwrap();
        }
    }
    let mixed_route = if host == "x86_64-unknown-linux-gnu" {
        "native Lust and the pinned Rust host proc-macro compiler; general complementary compilation unavailable"
    } else {
        "exclusive native Lust; complementary compilation unavailable"
    };
    record(&output.join("NATIVE-BUILD-RECEIPT.txt"), native_output.as_bytes());
    let mut manifest = format!(
        "Partial native development build; complementary compilation and full-replacement release are unavailable.\nReferenced source={}, SDK={}, TinyCC={}\nNative build={}\nCargo artifact profile={}\nHost={}\nRoute={}\nSDK receipt={}\n",
        repo.display(),
        sdk.display(),
        tcc.display(),
        hash,
        profile.directory(),
        host,
        mixed_route,
        host_receipt.display()
    );
    for name in ["lustr", "lust", "lust-core"] {
        let result = checked(&format!("hash-{name}"), host_runtime::sha256_command().arg(package.join("bin").join(name)), &logs);
        manifest.push_str(std::str::from_utf8(&result).unwrap());
    }
    for result in runtime_identities {
        manifest.push_str(std::str::from_utf8(&result).unwrap());
    }
    record(&output.join("COMPONENTS.txt"), manifest.as_bytes());
    for p in [package.join("bin"), package.join("lib"), package, logs, output.clone()] {
        fs::File::open(p).unwrap().sync_all().unwrap();
    }
    println!("Source-built local toolchain: {}", output.join("toolchain/bin/lustr").display());
    println!("Build caches are owned by this output scope. Retain artifacts and logs, then remove its work directory after qualification.");
}
