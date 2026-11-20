use std::path::PathBuf;
use std::process::Command;

const COMMIT: &str = "2d20b42b883dcfc014f3c2c56081eeb6091c2179";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    let rustc = std::env::var_os("RUSTC").expect("Cargo must set RUSTC");
    let query = |args: &[&str]| {
        // Cargo adds rustc-dev's link stubs to its loader search path. The SDK
        // compiler must use its own runtime loader paths for these queries.
        let output = Command::new(&rustc).env_remove("LD_LIBRARY_PATH").args(args).output().expect("query build compiler");
        assert!(output.status.success(), "build compiler query {args:?} failed: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).expect("build compiler response must be UTF-8")
    };
    let version = query(&["-vV"]);
    assert!(
        version.lines().any(|line| line == format!("commit-hash: {COMMIT}")),
        "independent Rust inspection oracle requires rustc commit {COMMIT}; got {version}"
    );
    let host = version.lines().find_map(|line| line.strip_prefix("host: ")).expect("compiler host");
    let release = version.lines().find_map(|line| line.strip_prefix("release: ")).expect("compiler release");
    println!("cargo:rustc-env=LUSTR_BUILD_RUSTC_HOST={host}");
    println!("cargo:rustc-env=LUSTR_BUILD_RUSTC_RELEASE={release}");
    println!("cargo:rustc-env=LUSTR_BUILD_RUSTC_COMMIT={COMMIT}");
    let sysroot = PathBuf::from(query(&["--print", "sysroot"]).trim_end_matches('\n'));
    let sysroot = sysroot.canonicalize().expect("compiler sysroot must exist");
    println!("cargo:rustc-env=LUSTR_BUILD_SYSROOT={}", sysroot.to_str().expect("toolchain location must be UTF-8"));
    // rustc-dev supplies rustc_driver as a dynamic library. The probe is an
    // installed host tool, so it must also find that compiler's shared runtime.
    if host.contains("linux") || host.contains("darwin") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
        for directory in [sysroot.join("lib"), sysroot.join("lib/rustlib").join(host).join("lib")] {
            let directory = directory.to_str().expect("toolchain location must be UTF-8");
            assert!(!directory.contains(['\n', '\r', ',']), "unsupported toolchain path for linker rpath");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
        }
    }
}
