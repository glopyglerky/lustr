//! Ordinary Cargo check, retained original checking and fresh Rust metadata.
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};
fn put(p: &Path, s: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    let mut f = fs::OpenOptions::new().write(true).create_new(true).open(p).unwrap();
    f.write_all(s.as_bytes()).unwrap();
}
fn retain(p: &Path, to: &Path) {
    fs::copy(p, to).unwrap();
    fs::set_permissions(to, fs::Permissions::from_mode(0o644)).unwrap();
}
fn metadata_files(root: &Path, found: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            metadata_files(&entry.path(), found);
        } else if kind.is_file()
            && entry.file_name().to_str().unwrap().starts_with("libchecked_lib-")
            && entry.path().extension().is_some_and(|e| e == "rmeta")
        {
            found.push(entry.path());
        }
    }
}
fn main() {
    unsafe extern "C" {
        fn prctl(option: i32, ...) -> i32;
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }
    assert_eq!(unsafe { prctl(36, 1usize, 0usize, 0usize, 0usize) }, 0);
    let mut a = std::env::args_os().skip(1);
    let work = PathBuf::from(a.next().unwrap());
    let compiler = PathBuf::from(a.next().unwrap());
    let rustc = PathBuf::from(a.next().unwrap());
    assert!(a.next().is_none());
    fs::create_dir(&work).unwrap();
    let work = work.canonicalize().unwrap();
    retain(&compiler, &work.join("actual-entry-bytes"));
    let driver = compiler.with_file_name("rustc-full").canonicalize().unwrap();
    retain(&driver, &work.join("actual-mixed-driver-bytes"));
    retain(&driver.with_file_name("lust-core"), &work.join("actual-native-core-bytes"));
    put(&work.join("Cargo.toml"), "[workspace]\nmembers=['app','checked-lib','macro-source','portable']\nresolver='3'\n");
    put(&work.join("checked-lib/Cargo.toml"), "[package]\nname='checked-lib'\nversion='0.1.0'\nedition='2024'\n");
    let library = "pub fn boundary(x:i64)->i64{x ^ 17}\n";
    let lib = work.join("checked-lib/src/lib.rs");
    put(&lib, library);
    put(&work.join("portable/Cargo.toml"), "[package]\nname='portable'\nversion='0.1.0'\nedition='2024'\n");
    put(&work.join("portable/src/lib.rs"), "#![no_std]\nextern crate alloc;\npub fn general()->i64{alloc::vec![0i64].iter().sum()}\n");
    put(&work.join("macro-source/Cargo.toml"), "[package]\nname='macro-source'\nversion='0.1.0'\nedition='2024'\n[lib]\nproc-macro=true\n");
    let mac = "use proc_macro::TokenStream;\n#[proc_macro]\npub fn general(_:TokenStream)->TokenStream{\"async fn general()->i64{checked_lib::boundary(2)}\".parse().unwrap()}\n";
    put(&work.join("macro-source/src/lib.rs"), mac);
    put(
        &work.join("app/Cargo.toml"),
        "[package]\nname='checked-app'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nchecked-lib={path='../checked-lib'}\nmacro-source={path='../macro-source'}\nportable={path='../portable'}\n",
    );
    let source = "use macro_source::general;\ngeneral!();\nfn kernel(x:i64)->i64{x ^ 17}\nfn main(){let _future=general();println!(\"{}\",kernel(checked_lib::boundary(portable::general())));}\n";
    let app = work.join("app/src/main.rs");
    put(&app, source);
    let trace = work.join("mixed.trace");
    let check = |label: &str, ok: bool| {
        fs::write(&trace, "").unwrap();
        let before = (fs::read(&app).unwrap(), fs::read(&lib).unwrap());
        let mut command = Command::new(&compiler);
        command
            .current_dir(&work)
            .args(["check", "--offline", "--jobs", "2", "-vv", "-p", "checked-app", "--target-dir"])
            .arg(work.join("target"))
            .arg("--config")
            .arg(format!("build.build-dir={:?}", work.join("intermediates")))
            .env("LUSTR_MIXED_TRACE", &trace)
            .env("LUSTR_MIXED_CACHE_DIR", work.join("mixed-state"))
            .env("LUST_CACHE_DIR", work.join("native-cache"))
            .env("LUST_DAEMON_SOCKET", work.join("native.sock"))
            .env("LUST_DAEMON_IDLE", "2s")
            .env("LUST_STATS", "1")
            .env_remove("CARGO_PROFILE_DEV_DEBUG")
            .env_remove("LUST_LIBRARY")
            .env_remove("LD_LIBRARY_PATH");
        let out = command.output().unwrap();
        fs::write(work.join(format!("{label}.stdout")), &out.stdout).unwrap();
        fs::write(work.join(format!("{label}.stderr")), &out.stderr).unwrap();
        fs::write(work.join(format!("{label}.status")), format!("{}\n", out.status)).unwrap();
        fs::copy(&trace, work.join(format!("{label}.trace"))).unwrap();
        assert_eq!(before, (fs::read(&app).unwrap(), fs::read(&lib).unwrap()));
        assert_eq!(out.status.success(), ok, "{label}: {}", String::from_utf8_lossy(&out.stderr));
        fs::read_to_string(&trace).unwrap()
    };
    let cold = check("cold", true);
    assert!(cold.matches("native-checked-definitions=1").count() >= 2, "missing native library/app checks: {cold}");
    assert!(!cold.contains("native-definition-omitted="));
    let metadata = || {
        // Metadata may live in Cargo's separate build-dir rather than the
        // target directory. Find the actual compiler output in these two
        // request-owned output scopes; never manufacture a target/deps copy.
        let mut files = Vec::new();
        metadata_files(&work.join("target"), &mut files);
        metadata_files(&work.join("intermediates"), &mut files);
        assert_eq!(files.len(), 1);
        files[0].clone()
    };
    let original = fs::read(metadata()).unwrap();
    retain(&metadata(), &work.join("original.rmeta"));
    fs::write(&app, source.replace("x ^ 17", "x ^ 19")).unwrap();
    let warm = check("body", true);
    assert!(warm.contains("reused-check-context=1 full-analysis=0 native-check-edits=1"), "no retained app check: {warm}");
    assert!(!warm.contains("native-definition-omitted="));
    fs::write(&lib, library.replace("x ^ 17", "x ^ 19")).unwrap();
    let warmlib = check("library-body", true);
    assert!(warmlib.contains("reused-check-context=1 full-analysis=0 native-check-edits=1"), "no retained library check: {warmlib}");
    let fresh = fs::read(metadata()).unwrap();
    assert_ne!(original, fresh, "source edit retained old Rust metadata");
    retain(&metadata(), &work.join("edited.rmeta"));
    // The matching original compiler consumes the genuinely new metadata.
    put(&work.join("consume.rs"), "pub fn consume()->i64{checked_lib::boundary(0)}\n");
    let out = Command::new(&rustc)
        .args(["--edition=2024", "--crate-type=lib", "--emit=metadata"])
        .arg(work.join("consume.rs"))
        .arg("--extern")
        .arg(format!("checked_lib={}", metadata().display()))
        .arg("-o")
        .arg(work.join("consumed.rmeta"))
        .env_remove("LD_LIBRARY_PATH")
        .output()
        .unwrap();
    fs::write(work.join("consume.stdout"), out.stdout).unwrap();
    fs::write(work.join("consume.stderr"), &out.stderr).unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    fs::write(&app, source.replace("x ^ 17", "x ^ true")).unwrap();
    let bad = check("invalid", false);
    assert!(!bad.contains("reused-check-context=1"));
    fs::write(&app, source).unwrap();
    check("recovery", true);
    fs::write(&lib, library.replace("->i64", "->u64").replace("x ^ 17", "(x ^ 17) as u64")).unwrap();
    let signature = check("signature", false);
    assert!(!signature.contains("reused-check-context=1"), "signature edit reused stale checking");
    fs::write(&lib, library).unwrap();
    check("signature-recovery", true);
    let macrofile = work.join("macro-source/src/lib.rs");
    fs::write(&macrofile, mac.replace("->i64", "->u64")).unwrap();
    check("macro-invalid", false);
    fs::write(&macrofile, mac).unwrap();
    check("macro-recovery", true);
    let warned = format!("{source}fn unused_warning(){{}}\n");
    fs::write(&app, &warned).unwrap();
    check("warning-cold", true);
    fs::write(&app, warned.replace("x ^ 17", "x ^ 19")).unwrap();
    let warning = check("warning-edit", true);
    assert!(fs::read_to_string(work.join("warning-edit.stderr")).unwrap().contains("function `unused_warning` is never used"));
    assert!(!warning.contains("reused-check-context=1"));
    fs::write(&app, format!("#![deny(dead_code)]\n{warned}")).unwrap();
    check("warning-denied", false);
    let mut daemons = 0;
    loop {
        let mut status = 0;
        let pid = unsafe { waitpid(-1, &mut status, 0) };
        if pid < 0 {
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(10));
            break;
        }
        assert_eq!(status, 0, "owned daemon {pid}");
        daemons += 1;
    }
    assert!(daemons > 0);
    fs::write(work.join("owned-daemons.status"), format!("{daemons} actual idle exit0; no remaining children\n")).unwrap();
    println!(
        "ordinary debug-profile Cargo check; native retained app/library edits; fresh Rust metadata consumed by original compiler; real proc macro/core/alloc; signature/macro/type rejection and recovery; warning/deny parity: actual0"
    );
}
