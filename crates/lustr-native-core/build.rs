use std::path::PathBuf;
use std::process::Command;

fn main() {
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
    print!("{}", String::from_utf8(output.stdout).unwrap());
    // Included upstream code retains its original source-library location.
    println!("cargo:rustc-env=CARGO_MANIFEST_DIR={}", source.display());
    println!("cargo:rerun-if-changed={}", source.display());
}
