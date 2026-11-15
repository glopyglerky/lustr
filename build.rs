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
    verify(&lust.join("bin/lust"), pin["lust_sha256"]);
    verify(&lust.join("bin/lust-core"), pin["lust_core_sha256"]);
    println!("cargo:rustc-env=LUST_BUILD_HASH={}", pin["lust_build"]);
    println!("cargo:rustc-env=LUSTR_CORE_SOURCE={}", pin["source"]);
    println!("cargo:rerun-if-changed=upstream/bluefe-core.pin");
    println!("cargo:rerun-if-changed=compiler/native/src/client.rs");
    println!("cargo:rerun-if-changed=compiler/native/src/protocol.rs");
    println!("cargo:rerun-if-env-changed=LUSTR_CORE_ROOT");
    println!("cargo:rerun-if-env-changed=LUSTR_LUST_ROOT");
}
