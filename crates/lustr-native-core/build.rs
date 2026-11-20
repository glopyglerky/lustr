use std::path::PathBuf;
use std::process::Command;

fn main() {
    assert!(
        std::env::var_os("CARGO_FEATURE_EXCLUSIVE_NATIVE").is_some()
            || std::env::var_os("CARGO_FEATURE_NATIVE_QUALIFICATION").is_some(),
        "select exclusive-native for the product or native-qualification for bounded research"
    );
    // Product feature unification must never import the research SDK or select
    // the separate qualification composition. Reject it before querying a
    // compiler, building native dependencies or emitting a product receipt.
    assert!(
        std::env::var_os("CARGO_FEATURE_EXCLUSIVE_NATIVE").is_none()
            || (std::env::var_os("CARGO_FEATURE_NATIVE_QUALIFICATION").is_none()
                && std::env::var_os("CARGO_FEATURE_UPSTREAM_SEMANTIC").is_none()),
        "exclusive-native cannot be combined with native-qualification or upstream-semantic"
    );
    if std::env::var_os("CARGO_FEATURE_UPSTREAM_SEMANTIC").is_some() {
        let compiler = std::env::var_os("RUSTC").unwrap();
        let query = |args: &[&str]| {
            let out = Command::new(&compiler).env_remove("LD_LIBRARY_PATH").args(args).output().unwrap();
            assert!(out.status.success(), "cannot inspect imported compiler identity");
            String::from_utf8(out.stdout).unwrap()
        };
        let identity = query(&["-vV"]);
        let commit = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494";
        assert!(identity.lines().any(|line| line == format!("commit-hash: {commit}")), "upstream semantic service requires {commit}");
        let root = PathBuf::from(query(&["--print", "sysroot"]).trim()).canonicalize().unwrap();
        println!("cargo:rustc-env=LUSTR_IMPORTED_SYSROOT={}", root.display());
        println!("cargo:rustc-env=LUSTR_IMPORTED_COMMIT={commit}");
        println!("cargo:rustc-link-search=native={}", root.join("lib").display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", root.join("lib").display());
    }
    let source = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../compiler/native").canonicalize().unwrap();
    let helper = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("upstream-build-helper");
    let status = Command::new(std::env::var_os("RUSTC").unwrap())
        .env_remove("LD_LIBRARY_PATH")
        .arg("--edition=2024")
        .arg(source.join("build.rs"))
        .arg("-o")
        .arg(&helper)
        .status()
        .unwrap();
    assert!(status.success());
    let output = Command::new(helper).env("CARGO_MANIFEST_DIR", &source).output().unwrap();
    assert!(output.status.success(), "unchanged upstream build helper failed");
    // The helper also builds a standalone executable. A cdylib has no
    // `lust-core` bin target and must not receive its Mach-O stack argument.
    for line in String::from_utf8(output.stdout).unwrap().lines() {
        if !line.starts_with("cargo:rustc-link-arg-bin=") {
            println!("{line}");
        }
    }
    // Included upstream code retains its original source-library location.
    println!("cargo:rustc-env=CARGO_MANIFEST_DIR={}", source.display());
    println!("cargo:rerun-if-changed={}", source.display());
}
