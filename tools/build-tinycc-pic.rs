//! Build the pinned TinyCC archive for shared-library use without editing the
//! source checkout or substituting another backend. The copied source receives
//! the separately declared, exact-source Mach-O signing integration.
use std::{
    fs,
    io::Write,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::Command,
};
const REVISION: &str = "43c7708b85681a2fd4451c8a541af4494a8919b2";
fn put(path: &Path, bytes: &[u8]) {
    let mut f = fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();
    f.write_all(bytes).unwrap();
    f.sync_all().unwrap();
}
fn run(label: &str, command: &mut Command, logs: &Path) -> Vec<u8> {
    put(&logs.join(format!("{label}.command")), format!("{command:?}\n").as_bytes());
    let out = command.output().unwrap();
    put(&logs.join(format!("{label}.stdout")), &out.stdout);
    put(&logs.join(format!("{label}.stderr")), &out.stderr);
    put(&logs.join(format!("{label}.status")), format!("{}\n", out.status).as_bytes());
    assert!(out.status.success(), "{label}: {}", String::from_utf8_lossy(&out.stderr));
    println!("{label}: {}", out.status);
    out.stdout
}
fn main() {
    let mut args = std::env::args_os().skip(1);
    let upstream = PathBuf::from(args.next().expect("PINNED_TINYCC_CHECKOUT NEW_OUTPUT")).canonicalize().unwrap();
    let output = PathBuf::from(args.next().unwrap());
    assert!(args.next().is_none() && output.is_absolute() && !output.exists());
    assert!(!output.parent().unwrap().canonicalize().unwrap().starts_with(&upstream));
    fs::DirBuilder::new().mode(0o700).create(&output).unwrap();
    let output = output.canonicalize().unwrap();
    let logs = output.join("logs");
    let source = output.join("source");
    let install = output.join("install");
    fs::create_dir(&logs).unwrap();
    fs::create_dir(&source).unwrap();
    let revision =
        run("source-identity", Command::new("git").arg("-C").arg(&upstream).args(["rev-parse", &format!("{REVISION}^{{commit}}")]), &logs);
    assert_eq!(String::from_utf8(revision).unwrap().trim(), REVISION);
    let archive = output.join("source.tar");
    run(
        "source-archive",
        Command::new("git").arg("-C").arg(&upstream).args(["archive", "--format=tar", REVISION, "--output"]).arg(&archive),
        &logs,
    );
    run("source-extract", Command::new("tar").args(["-xf"]).arg(&archive).arg("--directory").arg(&source), &logs);
    // Snapshot the exact reviewed applicator and declaration into this build;
    // invoking the constructor from another directory cannot select other code.
    let integration = output.join("integration");
    fs::create_dir(&integration).unwrap();
    fs::create_dir(integration.join("tools")).unwrap();
    fs::create_dir(integration.join("upstream")).unwrap();
    let repair = integration.join("tools/apply-tinycc-signing.py");
    put(&repair, include_bytes!("apply-tinycc-signing.py"));
    put(&integration.join("upstream/tinycc-macho-signing.json"), include_bytes!("../upstream/tinycc-macho-signing.json"));
    put(&integration.join("upstream/tinycc-macho-signing.patch"), include_bytes!("../upstream/tinycc-macho-signing.patch"));
    let receipt = logs.join("macho-signing-source.json");
    run("macho-signing-integration", Command::new("python3").arg(repair).arg(&source).arg("--receipt").arg(&receipt), &logs);
    // Keep upstream's normal -O2 optimization, adding the relocation property
    // required when libtcc.a is embedded in the native compiler shared library.
    run(
        "configure",
        Command::new("./configure")
            .current_dir(&source)
            .arg(format!("--prefix={}", install.display()))
            .arg("--extra-cflags=-Wall -O2 -fPIC"),
        &logs,
    );
    run("build", Command::new("make").current_dir(&source).arg("--jobs=2"), &logs);
    run("install", Command::new("make").current_dir(&source).arg("install"), &logs);
    let license = install.join("share/licenses/tinycc");
    fs::create_dir_all(&license).unwrap();
    fs::copy(source.join("COPYING"), license.join("COPYING")).unwrap();
    fs::copy(receipt, license.join("lustr-macho-signing.json")).unwrap();
    put(
        &license.join("SOURCE.txt"),
        format!("https://github.com/TinyCC/tinycc\nrevision {REVISION}\nDeclared Lustr Mach-O signing integration: lustr-macho-signing.json; build flags -Wall -O2 -fPIC.\n")
            .as_bytes(),
    );
    // Link the complete real archive before any expensive compiler build. This
    // detects the R_X86_64_PC32/non-PIC failure without fabricating a C probe.
    run(
        "shared-link-check",
        Command::new("cc")
            .args(["-shared", "-Wl,--whole-archive"])
            .arg(install.join("lib/libtcc.a"))
            .args(["-Wl,--no-whole-archive", "-ldl", "-lpthread", "-lm", "-o"])
            .arg(output.join("libtcc-shared-check.so")),
        &logs,
    );
    let hashes = run("component-hashes", Command::new("sha256sum").arg(install.join("bin/tcc")).arg(install.join("lib/libtcc.a")), &logs);
    let mut info=format!("TinyCC upstream {REVISION}; original source archived with Git; declared Mach-O signing integration in logs/macho-signing-source.json; -Wall -O2 -fPIC; jobs2.\nShared link check passed; its library is compile-only/never executed. Hosted C dependency, not an owned Rust implementation or whole-stack memory-safety claim.\n").into_bytes();
    info.extend_from_slice(&hashes);
    put(&output.join("COMPONENTS.txt"), &info);
    for p in [logs, install.clone(), output] {
        fs::File::open(p).unwrap().sync_all().unwrap();
    }
    println!("TinyCC PIC installation: {}", install.display());
}
