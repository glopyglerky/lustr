use std::{collections::BTreeMap, path::Path, process::Command};

fn verify(path: &Path, expected: &str) {
    let result = Command::new("sha256sum").arg(path).output().expect("sha256sum is required on the build host");
    assert!(result.status.success(), "cannot hash compiler input");
    assert_eq!(
        String::from_utf8(result.stdout).unwrap().split_whitespace().next(),
        Some(expected),
        "compiler input does not match the frozen core pin"
    );
}

fn main() {
    assert_eq!(std::env::var("CARGO_CFG_TARGET_OS").as_deref(), Ok("linux"));
    assert_eq!(std::env::var("CARGO_CFG_TARGET_ARCH").as_deref(), Ok("x86_64"));
    let pin: BTreeMap<_, _> = include_str!("upstream/bluefe-core.pin")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let (k, v) = l.split_once('=').expect("invalid core pin");
            (k.trim(), v.trim().trim_matches('"'))
        })
        .collect();
    let core = std::env::var_os("LUSTR_CORE_ROOT").expect("LUSTR_CORE_ROOT must name the frozen native core");
    let lust = std::env::var_os("LUSTR_LUST_ROOT").expect("LUSTR_LUST_ROOT must name the pristine Lust runtime");
    let core = Path::new(&core);
    let lust = Path::new(&lust);
    assert!(core.is_absolute() && lust.is_absolute());
    let compiler = if core.join("bin/rustc-full").is_file() { core.join("bin/rustc-full") } else { core.join("bin/rustc") };
    verify(&compiler, pin["rustc_sha256"]);
    verify(Path::new(&std::env::var_os("RUSTC").unwrap()), pin["rustc_sha256"]);
    verify(&core.join("bin/cargo"), pin["cargo_sha256"]);
    verify(&core.join("lib").join(pin["driver_file"]), pin["driver_sha256"]);
    verify(&core.join("lib").join(pin["llvm_file"]), pin["llvm_sha256"]);
    if cfg!(feature = "source-native-runtime") {
        // A fresh, source-guarded build supplies Cargo's actual native build
        // receipt. The frozen upstream Rust pin remains unchanged. The default
        // feature set still verifies the historical complete package exactly.
        let receipt = std::env::var_os("LUSTR_NATIVE_BUILD_OUTPUT").expect("select the native Cargo build receipt");
        let receipt = Path::new(&receipt);
        assert!(receipt.is_absolute());
        let output = std::fs::read_to_string(receipt).expect("read native Cargo build receipt");
        let hashes: Vec<_> = output.lines().filter_map(|line| line.strip_prefix("cargo:rustc-env=LUST_BUILD_HASH=")).collect();
        assert_eq!(hashes.len(), 1, "one native compiler identity is required");
        let hash = hashes[0];
        assert!(hash.len() == 16 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(lust.join("bin/lust-core").is_file());
        let client = Command::new(lust.join("bin/lust")).arg("--version").output().expect("query selected native client");
        assert!(client.status.success());
        assert!(String::from_utf8(client.stdout).unwrap().contains(&format!("(build {})", &hash[..8])));
        println!("cargo:rustc-env=LUST_BUILD_HASH={hash}");
        println!("cargo:rerun-if-changed={}", receipt.display());
        println!("cargo:rerun-if-changed={}", lust.join("bin/lust").display());
        println!("cargo:rerun-if-changed={}", lust.join("bin/lust-core").display());
    } else {
        verify(&lust.join("bin/lust"), pin["lust_sha256"]);
        verify(&lust.join("bin/lust-core"), pin["lust_core_sha256"]);
        println!("cargo:rustc-env=LUST_BUILD_HASH={}", pin["lust_build"]);
    }
    println!("cargo:rustc-env=LUSTR_CORE_SOURCE={}", pin["source"]);
    println!("cargo:rerun-if-changed=upstream/bluefe-core.pin");
    println!("cargo:rerun-if-changed=compiler/native/src/client.rs");
    println!("cargo:rerun-if-changed=compiler/native/src/protocol.rs");
    println!("cargo:rerun-if-env-changed=LUSTR_CORE_ROOT");
    println!("cargo:rerun-if-env-changed=LUSTR_LUST_ROOT");
    println!("cargo:rerun-if-env-changed=LUSTR_NATIVE_BUILD_OUTPUT");
}
