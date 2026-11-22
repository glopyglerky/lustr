use std::{collections::BTreeMap, path::Path, process::Command};

#[path = "tools/host-sdk.rs"]
mod host_sdk;

fn verify(path: &Path, expected: &str) {
    assert_eq!(host_sdk::sha256(path).expect("hash compiler input"), expected, "compiler input does not match the frozen core pin");
}

fn main() {
    let host = std::env::var("HOST").expect("Cargo build host");
    let target = std::env::var("TARGET").expect("Cargo build target");
    assert_eq!(host, target, "build a native compiler on its destination host");
    assert!(
        matches!(target.as_str(), "x86_64-unknown-linux-gnu" | "aarch64-apple-darwin" | "x86_64-apple-darwin"),
        "the primary native runtime currently has Linux x86_64 and Darwin host adapters; other hosts need a qualified native entry; no stock-compiler coordinator is supplied"
    );
    let pin: BTreeMap<_, _> = include_str!("upstream/bluefe-core.pin")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let (k, v) = l.split_once('=').expect("invalid core pin");
            (k.trim(), v.trim().trim_matches('"'))
        })
        .collect();
    let core = std::env::var_os("LUSTR_CORE_ROOT").expect("LUSTR_CORE_ROOT must name the pinned compiler bootstrap SDK");
    let lust = std::env::var_os("LUSTR_LUST_ROOT").expect("LUSTR_LUST_ROOT must name the source-built native runtime");
    let core = Path::new(&core);
    let lust = Path::new(&lust);
    assert!(core.is_absolute() && lust.is_absolute());
    let compiler = core.join("bin/rustc");
    if target == "x86_64-unknown-linux-gnu" {
        verify(&compiler, pin["rustc_sha256"]);
        verify(Path::new(&std::env::var_os("RUSTC").unwrap()), pin["rustc_sha256"]);
        verify(&core.join("bin/cargo"), pin["cargo_sha256"]);
        verify(&core.join("lib").join(pin["driver_file"]), pin["driver_sha256"]);
        verify(&core.join("lib").join(pin["llvm_file"]), pin["llvm_sha256"]);
    } else {
        assert!(cfg!(feature = "source-native-runtime"), "Darwin requires a source-built development runtime and explicit SDK receipt");
        // The primary entry can run the compiler in-process. Reserve the same
        // main-thread stack as lust-core; Darwin cannot grow it with setrlimit.
        println!("cargo:rustc-link-arg-bin=lustr=-Wl,-stack_size,0x10000000");
        let receipt = std::env::var_os("LUSTR_HOST_SDK_RECEIPT").expect("LUSTR_HOST_SDK_RECEIPT must name the selected Darwin SDK receipt");
        let receipt = Path::new(&receipt);
        assert!(receipt.is_absolute(), "host SDK receipt must have an absolute path");
        host_sdk::verify_receipt(core, receipt, pin["source"], &target, Path::new(&std::env::var_os("RUSTC").unwrap()))
            .expect("verify the selected development host SDK");
        println!("cargo:rerun-if-changed={}", receipt.display());
    }
    assert!(cfg!(feature = "source-native-runtime"), "product builds require the source-built exclusive-native runtime");
    {
        // A fresh, source-guarded build supplies Cargo's actual native build
        // receipt, including the no-delegation product feature. The SDK is a
        // build input and never a production fallback.
        let receipt = std::env::var_os("LUSTR_NATIVE_BUILD_OUTPUT").expect("select the native Cargo build receipt");
        let receipt = Path::new(&receipt);
        assert!(receipt.is_absolute());
        let output = std::fs::read_to_string(receipt).expect("read native Cargo build receipt");
        assert!(
            output.lines().any(|line| line == "cargo:rustc-env=LUST_EXCLUSIVE_NATIVE=1"),
            "the product entry requires an exclusive native runtime"
        );
        let hashes: Vec<_> = output.lines().filter_map(|line| line.strip_prefix("cargo:rustc-env=LUST_BUILD_HASH=")).collect();
        assert_eq!(hashes.len(), 1, "one native compiler identity is required");
        let hash = hashes[0];
        assert!(hash.len() == 16 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
        for helper in ["lust", "lust-core"] {
            let identity = Command::new(lust.join("bin").join(helper))
                .arg("--lustr-runtime-identity")
                .output()
                .expect("query selected native helper policy");
            assert!(identity.status.success() && identity.stderr.is_empty(), "the native helper must report its product policy");
            assert_eq!(
                String::from_utf8(identity.stdout).unwrap(),
                format!("lustr-exclusive-native-v1\nbuild={hash}\n"),
                "native helper policy/build differs from its receipt"
            );
        }
        println!("cargo:rustc-env=LUST_BUILD_HASH={hash}");
        println!("cargo:rerun-if-changed={}", receipt.display());
        println!("cargo:rerun-if-changed={}", lust.join("bin/lust").display());
        println!("cargo:rerun-if-changed={}", lust.join("bin/lust-core").display());
    }
    // Cargo adds its target/dependency library search paths to build scripts.
    // A rustc-dev SDK can then select its target-libdir driver copy, whose
    // loader origin does not contain the pinned LLVM library. The selected
    // compiler must resolve its own SDK, as the constructor's queries do.
    let declared = Command::new(&compiler).arg("-vV").env_remove("LD_LIBRARY_PATH")
        .output().expect("query pinned bootstrap SDK identity");
    assert!(declared.status.success(), "pinned bootstrap SDK identity query failed: {}",
        String::from_utf8_lossy(&declared.stderr));
    let declared = String::from_utf8(declared.stdout).unwrap();
    assert!(declared.lines().any(|line| line == format!("commit-hash: {}", pin["source"])));
    let release = declared.lines().find_map(|line| line.strip_prefix("release: ")).expect("declared Rust semantic version");
    println!("cargo:rustc-env=LUSTR_DECLARED_RUST_RELEASE={release}");
    println!("cargo:rustc-env=LUSTR_CORE_SOURCE={}", pin["source"]);
    println!("cargo:rerun-if-changed=upstream/bluefe-core.pin");
    println!("cargo:rerun-if-changed=compiler/native/src/client.rs");
    println!("cargo:rerun-if-changed=compiler/native/src/protocol.rs");
    println!("cargo:rerun-if-changed=src/host_runtime.rs");
    println!("cargo:rerun-if-changed=tools/host-sdk.rs");
    println!("cargo:rerun-if-env-changed=LUSTR_CORE_ROOT");
    println!("cargo:rerun-if-env-changed=LUSTR_LUST_ROOT");
    println!("cargo:rerun-if-env-changed=LUSTR_NATIVE_BUILD_OUTPUT");
    println!("cargo:rerun-if-env-changed=LUSTR_HOST_SDK_RECEIPT");
}
