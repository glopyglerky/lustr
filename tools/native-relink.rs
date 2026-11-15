//! Offline recipient rebuild of every native consumer with replacement TinyCC.
//! Run `prepare KIT REFERENCE_RUNTIME NEW_WORK`, edit NEW_WORK/tinycc-...,
//! then `build KIT REFERENCE_RUNTIME NEW_WORK`. No installed toolchain changes.
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
#[path = "../src/native_expansion.rs"]
mod native_expansion;
#[allow(dead_code)]
#[path = "../src/owned_child.rs"]
mod owned_child;

const TCC: &str = "43c7708b85681a2fd4451c8a541af4494a8919b2";
const TCC_SOURCE_SHA256: &str = "8067daee0159f1259463513dac5013fc44c0c631741b60f225c4d52b2e9b8e39";

fn run(command: &mut Command) -> Vec<u8> {
    println!("{command:?}");
    let result = owned_child::output(command).unwrap();
    use std::io::Write;
    std::io::stdout().write_all(&result.stdout).unwrap();
    std::io::stderr().write_all(&result.stderr).unwrap();
    assert!(result.status.success(), "recipient command failed: {}", result.status);
    result.stdout
}

fn hash(path: &Path) -> String {
    let result = owned_child::output(Command::new("sha256sum").arg("--").arg(path)).unwrap();
    assert!(result.status.success());
    let text = std::str::from_utf8(&result.stdout).unwrap();
    let hash = &text[..64];
    assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()));
    hash.to_owned()
}

fn copy_tree(source: &Path, destination: &Path, links: &mut BTreeMap<(u64, u64), PathBuf>) {
    let meta = fs::symlink_metadata(source).unwrap();
    if meta.file_type().is_symlink() {
        std::os::unix::fs::symlink(fs::read_link(source).unwrap(), destination).unwrap();
    } else if meta.is_dir() {
        fs::create_dir(destination).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            copy_tree(&entry.path(), &destination.join(entry.file_name()), links);
        }
        fs::set_permissions(destination, meta.permissions()).unwrap();
    } else {
        assert!(meta.is_file());
        let identity = (meta.dev(), meta.ino());
        if let Some(first) = links.get(&identity) {
            fs::hard_link(first, destination).unwrap();
        } else {
            fs::copy(source, destination).unwrap();
            if meta.nlink() > 1 {
                links.insert(identity, destination.to_owned());
            }
        }
    }
}

fn quote(text: &str) -> String {
    let mut result = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn pin(path: &Path) -> BTreeMap<String, String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let (key, value) = l.split_once('=').unwrap();
            (key.trim().to_owned(), value.trim().trim_matches('"').to_owned())
        })
        .collect()
}

fn verify_reference(source: &Path, reference: &Path) {
    let pin = pin(&source.join("upstream/bluefe-core.pin"));
    for (file, key) in [("bin/rustc-full", "rustc_sha256"), ("bin/cargo", "cargo_sha256")] {
        assert_eq!(hash(&reference.join(file)), pin[key], "reference SDK mismatch");
    }
    for (file, key) in [("driver_file", "driver_sha256"), ("llvm_file", "llvm_sha256")] {
        assert_eq!(hash(&reference.join("lib").join(&pin[file])), pin[key], "reference SDK mismatch");
    }
}

fn cargo(kit: &Path, reference: &Path, work: &Path, manifest: &str, target: &str) {
    let cwd = work.join("source");
    for ancestor in cwd.ancestors() {
        for name in [".cargo/config", ".cargo/config.toml"] {
            assert!(
                !ancestor.join(name).exists(),
                "fixed host rebuild requires no discovered per-directory Cargo config: {}",
                ancestor.join(name).display()
            );
        }
    }
    let mut command = Command::new(reference.join("bin/cargo"));
    let build_dir = work.join(format!("cargo-build-{target}"));
    let config = format!("build.build-dir={}", quote(build_dir.to_str().expect("Cargo configuration requires a UTF-8 build path")));
    command
        .current_dir(cwd)
        .args([
            "build",
            "--config",
            &config,
            "--config",
            "build.target=[]",
            "--locked",
            "--offline",
            "--release",
            "--jobs",
            "2",
            "--manifest-path",
        ])
        .arg(work.join("source").join(manifest))
        .env("RUSTC", reference.join("bin/rustc-full"))
        .env("RUSTDOC", reference.join("bin/rustdoc"))
        .env("CARGO_HOME", work.join("cargo-home"))
        .env("CARGO_TARGET_DIR", work.join(format!("target-{target}")))
        .env("LUST_TCC_HOME", work.join("runtime/lib/lust/tcc"))
        .env("LUSTR_CORE_ROOT", reference)
        .env("LUSTR_LUST_ROOT", work.join("runtime"))
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER");
    // Remapping applies to ordinary compiler paths. Upstream build scripts also
    // embed operational source/TinyCC locations; rebuilt outputs are local to
    // this final work/runtime prefix and are not claimed privacy-cleared.
    let mut remap = OsString::from("--remap-path-prefix=");
    remap.push(work);
    remap.push("=/build/relink");
    command.env("CARGO_ENCODED_RUSTFLAGS", remap).env_remove("RUSTFLAGS");
    assert!(kit.join("vendor").is_dir());
    run(&mut command);
}

fn regular_files(base: &Path, directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let meta = entry.file_type().unwrap();
        if meta.is_dir() {
            regular_files(base, &entry.path(), paths);
        } else if meta.is_file() {
            paths.push(entry.path().strip_prefix(base).unwrap().to_owned());
        } else {
            assert!(meta.is_symlink());
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(args.len(), 4, "native-relink prepare|build KIT REFERENCE_RUNTIME WORK");
    let kit = Path::new(&args[1]).canonicalize().unwrap();
    let reference = Path::new(&args[2]).canonicalize().unwrap();
    let requested = std::path::absolute(&args[3]).unwrap();
    let work = if requested.exists() {
        requested.canonicalize().unwrap()
    } else {
        let name = requested.file_name().expect("work directory needs a final name");
        requested.parent().unwrap().canonicalize().unwrap().join(name)
    };
    assert!(
        !work.starts_with(&kit) && !kit.starts_with(&work) && !work.starts_with(&reference) && !reference.starts_with(&work),
        "work, kit and reference scopes must be disjoint"
    );
    let completion = work.join("relink-build-complete.sha256");
    if args[0] == "build" {
        match fs::remove_file(&completion) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => panic!("cannot invalidate previous completion: {e}"),
        }
        assert!(
            !std::env::vars_os().any(|(name, _)| name.to_str().is_some_and(|n| n.starts_with("CARGO_PROFILE_RELEASE_"))),
            "fixed release recipe rejects ambient CARGO_PROFILE_RELEASE_* overrides"
        );
    }
    verify_reference(&kit.join("source"), &reference);
    if args[0] == "prepare" {
        assert!(!work.exists(), "prepare needs a new work directory");
        let archive = kit.join(format!("tinycc-source-{TCC}.tar.gz"));
        assert_eq!(hash(&archive), TCC_SOURCE_SHA256);
        fs::create_dir(&work).unwrap();
        fs::create_dir(work.join("cargo-home")).unwrap();
        copy_tree(&kit.join("source"), &work.join("source"), &mut BTreeMap::new());
        copy_tree(&reference, &work.join("runtime"), &mut BTreeMap::new());
        let vendor = kit.join("vendor");
        fs::write(
            work.join("cargo-home/config.toml"),
            format!(
                "[source.crates-io]\nreplace-with='vendored'\n[source.vendored]\ndirectory={}\n",
                quote(vendor.to_str().expect("Cargo vendor path requires UTF-8"))
            ),
        )
        .unwrap();
        run(Command::new("tar").args(["-xzf"]).arg(archive).arg("-C").arg(&work));
        println!("Prepared sources. Edit {} then run build with the same arguments.", work.join(format!("tinycc-{TCC}")).display());
        return;
    }
    assert!(args[0] == "build" && work.join("source/upstream/bluefe-core.pin").is_file());
    let runtime = work.join("runtime");
    let tcc = work.join(format!("tinycc-{TCC}"));
    let prefix = runtime.join("lib/lust/tcc");
    let mut prefix_arg = OsString::from("--prefix=");
    prefix_arg.push(&prefix);
    run(Command::new("./configure").current_dir(&tcc).arg(prefix_arg).arg("--extra-cflags=-fPIC"));
    run(Command::new("make").current_dir(&tcc).arg("-j2"));
    run(Command::new("make").current_dir(&tcc).arg("install"));
    cargo(&kit, &reference, &work, "compiler/native/Cargo.toml", "native");
    cargo(&kit, &reference, &work, "crates/lustr-native-core/Cargo.toml", "native");
    for file in ["lust", "lust-core"] {
        fs::copy(work.join("target-native/release").join(file), runtime.join("bin").join(file)).unwrap();
    }
    fs::copy(work.join("target-native/release/liblustr_native_core.so"), runtime.join("lib/liblustr_native_core.so")).unwrap();
    let helper = work.join("upstream-build-helper");
    let helper_out = work.join("helper-out");
    fs::create_dir_all(&helper_out).unwrap();
    run(Command::new(reference.join("bin/rustc-full"))
        .args(["--edition=2024"])
        .arg(work.join("source/compiler/native/build.rs"))
        .arg("-o")
        .arg(&helper)
        .env_remove("LD_LIBRARY_PATH"));
    let build_env = run(Command::new(&helper)
        .env("CARGO_MANIFEST_DIR", work.join("source/compiler/native"))
        .env("LUST_TCC_HOME", &prefix)
        .env("CARGO_CFG_TARGET_OS", "linux")
        .env("OUT_DIR", helper_out));
    let build_env = String::from_utf8(build_env).unwrap();
    let build_hash = build_env.lines().find_map(|line| line.strip_prefix("cargo:rustc-env=LUST_BUILD_HASH=")).unwrap();
    assert!(build_hash.len() == 16 && build_hash.bytes().all(|b| b.is_ascii_hexdigit()));
    let pin_path = work.join("source/upstream/bluefe-core.pin");
    let mut pins = pin(&pin_path);
    pins.insert("lust_build".into(), build_hash.into());
    pins.insert("lust_sha256".into(), hash(&runtime.join("bin/lust")));
    pins.insert("lust_core_sha256".into(), hash(&runtime.join("bin/lust-core")));
    fs::write(&pin_path, pins.iter().map(|(k, v)| format!("{k} = {}\n", quote(v))).collect::<String>()).unwrap();
    cargo(&kit, &reference, &work, "Cargo.toml", "driver");
    for file in ["lustr", "rustc"] {
        fs::copy(work.join("target-driver/release/lustr"), runtime.join("bin").join(file)).unwrap();
    }
    cargo(&kit, &reference, &work, "crates/lustr/Cargo.toml", "coordinator");
    for file in ["cargo-lust", "cargo-lustr"] {
        fs::copy(work.join("target-coordinator/release").join(file), runtime.join("bin").join(file)).unwrap();
    }
    let component = runtime.join("lib/lustr-expanded/bin/lust-core");
    fs::remove_file(&component).unwrap();
    fs::hard_link(runtime.join("bin/lust-core"), component).unwrap();
    let mut family = vec![PathBuf::from("bin/lust"), PathBuf::from("bin/lust-core"), PathBuf::from("lib/liblustr_native_core.so")];
    regular_files(&runtime, &runtime.join("lib/lust"), &mut family);
    family.sort();
    let bytes = output_hashes(&runtime, &family);
    fs::write(runtime.join("lib/native-family.sha256"), bytes).unwrap();
    fs::write(runtime.join("lib/lustr-native.id"), format!("{}\n", hash(&runtime.join("lib/native-family.sha256")))).unwrap();
    let library = runtime.join("lib/lustr");
    let inputs = native_expansion::inputs(&library).unwrap();
    let paths: Vec<PathBuf> = inputs.into_iter().map(PathBuf::from).collect();
    fs::write(library.join("sources.sha256"), output_hashes(&library, &paths)).unwrap();
    let receipts: Vec<PathBuf> = [
        "runtime/bin/lust",
        "runtime/bin/lust-core",
        "runtime/bin/lustr",
        "runtime/bin/rustc",
        "runtime/bin/cargo-lust",
        "runtime/bin/cargo-lustr",
        "runtime/lib/liblustr_native_core.so",
        "runtime/lib/lustr-native.id",
        "runtime/lib/native-family.sha256",
        "runtime/lib/lustr/sources.sha256",
        "source/upstream/bluefe-core.pin",
    ]
    .map(PathBuf::from)
    .to_vec();
    fs::write(&completion, output_hashes(&work, &receipts)).unwrap();
    println!("Rebuilt all native consumers, root native pin, runtime namespace, source seal and hardlink. Runtime: {}", runtime.display());
    println!("Run runtime qualification before use; this does not establish release, privacy or license acceptance.");
}

fn output_hashes(root: &Path, paths: &[PathBuf]) -> Vec<u8> {
    let result = owned_child::output(Command::new("sha256sum").current_dir(root).arg("--").args(paths)).unwrap();
    assert!(result.status.success());
    result.stdout
}
