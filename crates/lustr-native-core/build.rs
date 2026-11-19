use std::path::PathBuf;
use std::process::Command;

fn main() {
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
    println!("cargo:rustc-check-cfg=cfg(lustr_tcc_input_observer)");
    // This input observer is qualified against the actual static archive and
    // child compiler, whose Linux ELF input APIs were inspected together.
    // An unknown build remains a working compiler but cannot retain artifacts.
    if std::env::var_os("CARGO_FEATURE_NATIVE_INFERENCE").is_some()
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux")
        && std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("x86_64")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu")
    {
        let home = PathBuf::from(std::env::var_os("LUST_TCC_HOME").unwrap_or_else(|| source.join("third_party/tcc").into_os_string()));
        let archive = home.join("lib/libtcc.a");
        let binary = home.join("bin/tcc");
        let hash = |p: &std::path::Path| -> Option<String> {
            let out = Command::new("sha256sum").arg("--").arg(p).output().ok()?;
            if !out.status.success() {
                return None;
            }
            Some(std::str::from_utf8(&out.stdout).ok()?.get(..64)?.to_owned())
        };
        if hash(&archive).as_deref() == Some("8f278496a5b57508f56a1307fdee8567fe4ae9e17ee00cab5fbb5b1cc8fc795c")
            && hash(&binary).as_deref() == Some("68ba40c160519df959a9531376b1762411332e4e2f6edbfd52c8a27cd03270f2")
        {
            let observer_source = source.join("../../tools/native-tcc-input-observer.rs");
            let observer = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("liblustr_tcc_inputs.so");
            let status = Command::new(std::env::var_os("RUSTC").unwrap())
                .env_remove("LD_LIBRARY_PATH")
                .args(["--edition=2024", "--crate-type=cdylib", "-Copt-level=2"])
                .arg(&observer_source)
                .arg("-o")
                .arg(&observer)
                .status()
                .unwrap();
            assert!(status.success(), "Rust native input observer build failed");
            println!("cargo:rustc-env=LUSTR_BUILT_TCC_INPUT_OBSERVER={}", observer.display());
            println!("cargo:rerun-if-changed={}", observer_source.display());
            println!("cargo:rustc-cfg=lustr_tcc_input_observer");
            println!("cargo:rustc-link-arg=-Wl,--wrap=open");
            println!("cargo:rustc-link-arg=-Wl,--wrap=dlopen");
            println!("cargo:rustc-link-arg=-Wl,--wrap=time");
        }
        println!("cargo:rerun-if-changed={}", archive.display());
        println!("cargo:rerun-if-changed={}", binary.display());
    }
}
