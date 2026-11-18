//! Real Cargo/proc-macro execution, native edits, invalidation and rejection.
use std::{
    collections::BTreeSet,
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
fn retain(source: &Path, target: &Path) {
    fs::copy(source, target).unwrap();
    fs::set_permissions(target, fs::Permissions::from_mode(0o644)).unwrap();
}
fn profile_files(directory: &Path) -> BTreeSet<String> {
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().expect("profiling names must be UTF-8"))
        .filter(|name| name.ends_with(".jsonl"))
        .collect()
}
fn main() {
    // Adopt the original Lust daemon after its short-lived compiler client
    // exits, so the qualifier observes its real idle-exit status too.
    unsafe extern "C" {
        fn prctl(option: i32, ...) -> i32;
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }
    assert_eq!(unsafe { prctl(36, 1usize, 0usize, 0usize, 0usize) }, 0);
    let mut a = std::env::args_os().skip(1);
    let work = PathBuf::from(a.next().unwrap());
    let cargo = PathBuf::from(a.next().unwrap());
    let compiler = PathBuf::from(a.next().unwrap());
    fs::create_dir(&work).unwrap();
    let work = work.canonicalize().unwrap();
    retain(&compiler, &work.join("actual-driver-bytes"));
    let driver = if compiler.file_name().is_some_and(|n| n == "lustr") {
        compiler.with_file_name("rustc-full").canonicalize().unwrap()
    } else {
        compiler.canonicalize().unwrap()
    };
    retain(&driver, &work.join("actual-mixed-driver-bytes"));
    retain(&driver.with_file_name("lust-core"), &work.join("actual-native-core-bytes"));
    put(&work.join("Cargo.toml"), "[workspace]\nmembers=['app','mixed-macro','tiny-core']\nresolver='3'\n[profile.dev]\ndebug=0\n");
    put(&work.join("tiny-core/Cargo.toml"), "[package]\nname='tiny-core'\nversion='0.1.0'\nedition='2024'\n");
    let library = "#![no_std]\nextern crate alloc;\npub fn adjustment()->u64{let v=alloc::vec::Vec::from([0u64]);v.iter().sum()}\n";
    let library_path = work.join("tiny-core/src/lib.rs");
    put(&library_path, library);
    put(&work.join("mixed-macro/Cargo.toml"), "[package]\nname='mixed-macro'\nversion='0.1.0'\nedition='2024'\n[lib]\nproc-macro=true\n");
    let macro_source = "use proc_macro::TokenStream;\n#[proc_macro]\npub fn general(_:TokenStream)->TokenStream{\"async fn general(x:u64)->u64{kernel(choose(x))}\".parse().unwrap()}\n";
    put(&work.join("mixed-macro/src/lib.rs"), macro_source);
    put(
        &work.join("app/Cargo.toml"),
        "[package]\nname='mixed-app'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nmixed-macro={path='../mixed-macro'}\ntiny-core={path='../tiny-core'}\n",
    );
    let source = concat!(
        "use mixed_macro::general;\n#[allow(non_camel_case_types)] type u64=i64;\nmod core{pub mod mem{pub fn size_of<T>()->usize{1}}}\nmod nested{pub fn kernel(x:i64)->i64{x ^ 31}}\nfn projected_size()->usize{core::mem::size_of::<u64>()}\nfn signed_alias(x:std::primitive::i64)->std::primitive::i64{if (x as u64)<0{1}else{2}}\ngeneral!();\nfn kernel(x:u64)->u64{x ^ 17}\nfn choose(x:u64)->u64{if x & 1 == 0 {x ^ 1024} else {x | 2048}}\n",
        "trait Counterfeit{fn wrapping_add(self,rhs:u64)->u64;}\nimpl Counterfeit for bool{fn wrapping_add(self,_:u64)->u64{99}}\nfn fake_wrapping(x:u64)->u64{(x==0).wrapping_add(17)}\nfn arithmetic(x:std::primitive::u64)->std::primitive::u64{x.wrapping_add(17)}\nfn rotation(x:std::primitive::u64)->std::primitive::u64{x.wrapping_sub(5).wrapping_mul(3).wrapping_neg().wrapping_shl(65).wrapping_shr(64).rotate_left(7).rotate_right(9)}\nfn signed_rotation(x:std::primitive::i64)->std::primitive::i64{x.wrapping_add(17).wrapping_sub(5).wrapping_mul(3).wrapping_neg().wrapping_shl(65).wrapping_shr(64).rotate_left(7).rotate_right(9)}\n",
        "fn main(){assert_eq!(fake_wrapping(0),99);let literal=std::env::var(\"MIXED_ARITH_LITERAL\").unwrap_or_else(|_|\"17\".into()).parse::<std::primitive::u64>().unwrap();for value in [0,1,42,std::primitive::u64::MAX]{assert_eq!(arithmetic(value),value.wrapping_add(literal));assert_eq!(rotation(value),value.wrapping_sub(5).wrapping_mul(3).wrapping_neg().wrapping_shl(65).wrapping_shr(64).rotate_left(7).rotate_right(9));}for value in [std::primitive::i64::MIN,-1,0,1,std::primitive::i64::MAX]{assert_eq!(signed_rotation(value),value.wrapping_add(17).wrapping_sub(5).wrapping_mul(3).wrapping_neg().wrapping_shl(65).wrapping_shr(64).rotate_left(7).rotate_right(9));}assert_eq!(projected_size(),1);assert_eq!(signed_alias(-1),1);assert_eq!(nested::kernel(0),31);let w=std::task::Waker::noop();let mut f=std::pin::pin!(general((42+tiny_core::adjustment()) as _));let mut cx=std::task::Context::from_waker(w);match std::future::Future::poll(f.as_mut(),&mut cx){std::task::Poll::Ready(v)=>println!(\"{v}\"),std::task::Poll::Pending=>panic!(\"unexpected pending\")}}\n"
    );
    let path = work.join("app/src/main.rs");
    put(&path, source);
    // Keep the linker command and compiler environment identical across the
    // failure and recovery requests. Only its external failure marker changes.
    let linker = work.join("controlled-linker");
    put(
        &linker,
        "#!/bin/sh\nif [ -e \"$(dirname \"$0\")/fail-link\" ]; then\n  echo 'intentional retained-link failure' >&2\n  exit 75\nfi\nexec cc \"$@\"\n",
    );
    fs::set_permissions(&linker, fs::Permissions::from_mode(0o755)).unwrap();
    let trace = work.join("mixed.trace");
    // This is only enabled by a separate profiling run. Keep one directory and
    // environment value across edits so profiling cannot change the cache key.
    let profile_directory = std::env::var_os("LUSTR_MIXED_PROFILE_DIR").map(PathBuf::from);
    let build = |label: &str, expected: bool| {
        let before = fs::read(&path).unwrap();
        let profiles_before = profile_directory.as_ref().map(|path| profile_files(path));
        let start = std::time::Instant::now();
        // A packaged lustr entry owns Cargo selection. The standalone driver form
        // exercises the identical compiler contract while constructing that package.
        let packaged = compiler.file_name().is_some_and(|n| n == "lustr");
        let mut command = Command::new(if packaged { &compiler } else { &cargo });
        if !packaged {
            command.env("RUSTC", &compiler);
        }
        let output = command
            .current_dir(&work)
            .args(["build", "--offline", "--jobs", "2", "-vv", "-p", "mixed-app", "--target-dir"])
            .arg(work.join("target"))
            .arg("--config")
            .arg(format!("build.build-dir={:?}", work.join("intermediates")))
            .arg("--config")
            .arg(format!("target.x86_64-unknown-linux-gnu.linker={linker:?}"))
            .env("LUSTR_MIXED_TRACE", &trace)
            .env("LUSTR_MIXED_CACHE_DIR", work.join("mixed-state"))
            .env("LUST_CACHE_DIR", work.join("native-cache"))
            .env("LUST_DAEMON_SOCKET", work.join("native.sock"))
            .env("LUST_DAEMON_IDLE", "2s")
            .env_remove("LUST_LIBRARY")
            .env_remove("LD_LIBRARY_PATH")
            .output()
            .unwrap();
        fs::write(work.join(format!("{label}.stdout")), &output.stdout).unwrap();
        fs::write(work.join(format!("{label}.stderr")), &output.stderr).unwrap();
        fs::write(work.join(format!("{label}.status")), format!("{} elapsed_ns={}\n", output.status, start.elapsed().as_nanos())).unwrap();
        fs::copy(&trace, work.join(format!("{label}.trace"))).unwrap();
        if let Some(directory) = &profile_directory {
            let profiles_after = profile_files(directory);
            let added = profiles_after.difference(profiles_before.as_ref().unwrap()).cloned().collect::<Vec<_>>();
            fs::write(work.join(format!("{label}.profiles")), added.join("\n") + "\n").unwrap();
        }
        assert_eq!(fs::read(&path).unwrap(), before, "compiler altered application source");
        assert_eq!(output.status.success(), expected, "{label}: Cargo {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    };
    let execute = |label: &str, expected: &[u8]| {
        let app = work.join("target/debug/mixed-app");
        retain(&app, &work.join(format!("actual-app-{label}")));
        let result = Command::new(&app).env("MIXED_ARITH_LITERAL", if label == "arithmetic-edit" { "19" } else { "17" }).output().unwrap();
        fs::write(work.join(format!("app-{label}.stdout")), &result.stdout).unwrap();
        fs::write(work.join(format!("app-{label}.stderr")), &result.stderr).unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout, expected, "{label}: wrong application result");
    };
    build("cold", true);
    execute("cold", b"1083\n");
    let cold = fs::read_to_string(&trace).unwrap();
    assert!(
        cold.contains("native-definitions=5") && cold.matches("native-definition-omitted=").count() == 5,
        "missing mixed definitions: {cold}"
    );
    fs::write(&trace, "").unwrap();
    let edited = source.replace("x ^ 17", "x ^ 19");
    fs::write(&path, &edited).unwrap();
    build("body", true);
    execute("body", b"1081\n");
    let warm = fs::read_to_string(&trace).unwrap();
    assert!(warm.contains("reused-rust-objects=1 full-analysis=0"), "body edit did not reuse checked context: {warm}");
    assert!(!warm.contains("native-definition-omitted="), "warm request ran LLVM codegen");
    assert_eq!(warm.matches("lustr mixed native:").count(), 1, "unchanged native definition was regenerated: {warm}");
    fs::write(&trace, "").unwrap();
    fs::write(&path, edited.replacen("x.wrapping_add(17)", "x.wrapping_add(19)", 1)).unwrap();
    build("arithmetic-edit", true);
    execute("arithmetic-edit", b"1081\n");
    let arithmetic = fs::read_to_string(&trace).unwrap();
    assert!(arithmetic.contains("reused-rust-objects=1 full-analysis=0"), "wrapping edit did not reuse checked context: {arithmetic}");
    assert!(!arithmetic.contains("native-definition-omitted="), "wrapping edit ran LLVM codegen");
    assert_eq!(arithmetic.matches("lustr mixed native:").count(), 1, "wrapping edit regenerated unchanged definitions: {arithmetic}");
    fs::write(&trace, "").unwrap();
    fs::write(&path, edited.replace("x ^ 19", "x ^ true")).unwrap();
    build("invalid", false);
    fs::write(&trace, "").unwrap();
    fs::write(&path, &edited).unwrap();
    build("recovery", true);
    execute("recovery", b"1081\n");
    // A failed edited link must not change the native object described by
    // the last successful certificate. Restoring that source must execute
    // its original result even when its unchanged body needs no recompilation.
    fs::write(&trace, "").unwrap();
    fs::write(work.join("fail-link"), "").unwrap();
    fs::write(&path, edited.replace("x ^ 19", "x ^ 18")).unwrap();
    build("link-failure", false);
    assert!(fs::read_to_string(work.join("link-failure.stderr")).unwrap().contains("intentional retained-link failure"));
    let failed_link = fs::read_to_string(&trace).unwrap();
    assert_eq!(failed_link.matches("lustr mixed native:").count(), 1, "failure did not follow one retained native edit: {failed_link}");
    assert!(!failed_link.contains("native-definition-omitted="), "failed edit ran full LLVM code generation");
    fs::remove_file(work.join("fail-link")).unwrap();
    fs::write(&trace, "").unwrap();
    fs::write(&path, &edited).unwrap();
    build("link-failure-recovery", true);
    execute("link-failure-recovery", b"1081\n");
    let link_recovery = fs::read_to_string(&trace).unwrap();
    assert!(
        link_recovery.contains("reused-rust-objects=1 full-analysis=0"),
        "recovery did not reuse its original certificate: {link_recovery}"
    );
    assert!(!link_recovery.contains("lustr mixed native:"), "restoring the successful source regenerated native code");
    // A successful edit publishes the new object identity, including for
    // a later request that reuses it without regenerating native code.
    fs::write(&trace, "").unwrap();
    let versioned = edited.replace("x ^ 19", "x ^ 18");
    fs::write(&path, &versioned).unwrap();
    build("versioned-object", true);
    execute("versioned-object", b"1080\n");
    fs::write(&trace, "").unwrap();
    fs::write(&path, &versioned).unwrap();
    build("versioned-object-noop", true);
    execute("versioned-object-noop", b"1080\n");
    let versioned_noop = fs::read_to_string(&trace).unwrap();
    assert!(versioned_noop.contains("reused-rust-objects=1 full-analysis=0"), "published object was not reused: {versioned_noop}");
    assert!(!versioned_noop.contains("lustr mixed native:"), "unchanged versioned object was regenerated");
    fs::write(&trace, "").unwrap();
    let signature = edited.replace("kernel(x:u64)->u64", "kernel(x:u64)->usize").replace("x ^ 19", "(x ^ 19) as usize");
    fs::write(work.join("mixed-macro/src/lib.rs"), macro_source.replace("general(x:u64)->u64", "general(x:u64)->usize")).unwrap();
    fs::write(&path, &signature).unwrap();
    build("signature", true);
    execute("signature", b"1081\n");
    let signature_trace = fs::read_to_string(&trace).unwrap();
    assert!(!signature_trace.contains("reused-rust-objects=1"), "signature change reused stale Rust work");
    assert_eq!(
        signature_trace.matches("native-object-reused=").count(),
        4,
        "unchanged native definitions were regenerated: {signature_trace}"
    );
    assert_eq!(
        signature_trace.matches("lustr mixed native:").count(),
        1,
        "changed signature did not regenerate exactly its native definition: {signature_trace}"
    );
    fs::write(&trace, "").unwrap();
    fs::write(work.join("mixed-macro/src/lib.rs"), macro_source).unwrap();
    fs::write(&path, &edited).unwrap();
    fs::write(&library_path, library.replace("[0u64]", "[1u64]")).unwrap();
    build("dependency", true);
    execute("dependency", b"2104\n");
    let dependency_trace = fs::read_to_string(&trace).unwrap();
    assert!(!dependency_trace.contains("reused-rust-objects=1"), "changed dependency reused stale Rust work");
    assert_eq!(
        dependency_trace.matches("native-object-reused=").count(),
        4,
        "dependency change regenerated identical native definitions: {dependency_trace}"
    );
    fs::write(&trace, "").unwrap();
    fs::write(work.join("mixed-macro/src/lib.rs"), macro_source.replace("kernel(choose(x))", "kernel(choose(x)) ^ 1")).unwrap();
    build("macro", true);
    execute("macro", b"2105\n");
    let macro_trace = fs::read_to_string(&trace).unwrap();
    assert!(!macro_trace.contains("reused-rust-objects=1"), "changed macro expansion reused stale Rust work");
    assert_eq!(
        macro_trace.matches("native-object-reused=").count(),
        5,
        "macro change regenerated identical native definitions: {macro_trace}"
    );
    assert!(!macro_trace.contains("lustr mixed native:"), "unchanged native source was compiled again: {macro_trace}");
    // Late Rust warnings must survive a body-only edit. Until diagnostic
    // replay is connected, a warning-bearing context uses full analysis.
    let warned = format!("{edited}\nfn unused_warning(){{}}\n");
    fs::write(&trace, "").unwrap();
    fs::write(&path, &warned).unwrap();
    build("warning-cold", true);
    execute("warning-cold", b"2105\n");
    assert!(fs::read_to_string(work.join("warning-cold.stderr")).unwrap().contains("function `unused_warning` is never used"));
    fs::write(&trace, "").unwrap();
    fs::write(&path, warned.replace("x ^ 19", "x ^ 18")).unwrap();
    build("warning-edit", true);
    execute("warning-edit", b"2104\n");
    assert!(fs::read_to_string(work.join("warning-edit.stderr")).unwrap().contains("function `unused_warning` is never used"));
    assert!(!fs::read_to_string(&trace).unwrap().contains("reused-rust-objects=1"), "warning context skipped its late diagnostics");
    fs::write(&trace, "").unwrap();
    fs::write(&path, format!("#![deny(dead_code)]\n{warned}")).unwrap();
    build("warning-denied", false);
    assert!(fs::read_to_string(work.join("warning-denied.stderr")).unwrap().contains("function `unused_warning` is never used"));
    let mut daemons = 0;
    loop {
        let mut status = 0;
        let pid = unsafe { waitpid(-1, &mut status, 0) };
        if pid < 0 {
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(10));
            break;
        }
        assert_eq!(status, 0, "owned daemon {pid} did not exit successfully");
        daemons += 1;
    }
    assert!(daemons > 0, "no native daemon was observed");
    fs::write(work.join("owned-daemons.status"), format!("{daemons} adopted native daemon(s), actual idle exit0; no remaining children\n"))
        .unwrap();
    println!(
        "real Cargo/proc macro + core/alloc no_std library + mixed execution + native warm edit + rejection/recovery + failed-link cache isolation + versioned native object reuse + signature/dependency/macro invalidation + late warning/deny preservation: actual0"
    );
}
