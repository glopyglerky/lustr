//! Explicit SDK-bootstrapped qualification helper. Application compilation
//! below uses the actual native driver, never rustc acceptance or fallback.
#![allow(dead_code, unused_imports)]
include!("../crates/lustr-native-core/src/lib.rs");

// This example composes the native sources directly. Cargo's library-only
// link directive does not propagate without a Rust library dependency.
#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

fn put(path: &std::path::Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}

fn source(value: &str) -> String {
    format!("fn value() -> i64 {{ {value} }}\nfn main() {{ println!(\"{{}}\", value()); }}\n")
}

fn build(root: &std::path::Path, session: &mut driver::Session, file: &std::path::Path, label: &str, expected: i32) -> std::path::PathBuf {
    let output = root.join(format!("{label}.elf"));
    assert!(!output.exists());
    let request = driver::Request {
        cmd: driver::Cmd::Build, session: driver::SessionKey::synthetic(file, false),
        file: file.to_owned(), out: output.clone(), json: false, timings: true,
        debug: false, emit_c: None, max_errors: 0, run_args: None,
        harness: false, cc: false, client: Default::default(),
    };
    put(&root.join(format!("{label}.source.rs")), &std::fs::read(file).unwrap());
    put(&root.join(format!("{label}.request.bytes")), &protocol::encode(&request));
    let response = driver::compile(&request, session);
    put(&root.join(format!("{label}.stderr")), response.stderr.as_bytes());
    put(&root.join(format!("{label}.status")), format!("{}\n", response.code).as_bytes());
    assert_eq!(response.code, expected, "{label}: {}", response.stderr);
    assert!(!response.ran, "the compiler must not execute qualification applications");
    if expected != 0 { assert!(!output.exists()); }
    output
}

fn closure(root: &std::path::Path, label: &str, output: &std::path::Path, image: bool) {
    let result = std::process::Command::new("readelf").args(["-W", "-d"]).arg(output).output().unwrap();
    put(&root.join(format!("{label}.dynamic.txt")), &result.stdout);
    assert!(result.status.success(), "{label}: {}", String::from_utf8_lossy(&result.stderr));
    let text = String::from_utf8(result.stdout).unwrap();
    if image {
        assert!(text.contains("libbase-"), "{label} must exercise the real image link: {text}");
        assert!(text.contains("(RPATH)") || text.contains("(RUNPATH)"));
    } else {
        assert!(!text.contains("libbase-") && !text.contains("(RPATH)") && !text.contains("(RUNPATH)"), "{label}: {text}");
    }
}

fn main() {
    raise_stack_limit();
    let root = std::path::PathBuf::from(std::env::args_os().nth(1).expect("NEW_OUTPUT_DIRECTORY"));
    std::fs::create_dir(&root).unwrap();
    let file = root.join("input.rs");
    std::fs::write(&file, source("11")).unwrap();
    let key = driver::SessionKey::synthetic(&file, false);
    let mut one = driver::Session::new(root.join("objects-one"), false, key.clone());
    one.cache_tried = true;
    let once = build(&root, &mut one, &file, "one-shot-default", 0);
    assert!(one.caches.output == driver::OutputKind::Standalone);
    one.settle(); drop(one);

    let mut session = driver::Session::new(root.join("objects-session"), true, key);
    session.cache_tried = true;
    session.caches.base_dir = Some(root.join("bases"));
    assert!(session.caches.output == driver::OutputKind::SessionImage);
    let first = build(&root, &mut session, &file, "image-default", 0);
    std::fs::write(&file, source("22")).unwrap();
    let overlay = build(&root, &mut session, &file, "image-overlay", 0);
    assert!(!session.caches.overlays.is_empty(), "must exercise a populated image overlay before switching");

    session.set_output(driver::OutputKind::Standalone);
    assert!(session.caches.objects.is_none() && session.caches.overlays.is_empty());
    std::fs::write(&file, source("true")).unwrap();
    build(&root, &mut session, &file, "standalone-reject", 1);
    std::fs::write(&file, source("33")).unwrap();
    let standalone = build(&root, &mut session, &file, "standalone-recovery", 0);
    assert!(session.caches.overlays.is_empty());
    let text = std::fs::read_to_string(root.join("standalone-recovery.stderr")).unwrap();
    assert!(!text.contains("link libbase.so"));
    assert_eq!(text.lines().filter(|line| line.split_whitespace().next() == Some("link")).count(), 1);

    session.set_output(driver::OutputKind::SessionImage);
    std::fs::write(&file, source("true")).unwrap();
    build(&root, &mut session, &file, "image-return-reject", 1);
    std::fs::write(&file, source("44")).unwrap();
    let returned = build(&root, &mut session, &file, "image-return-recovery", 0);
    std::fs::write(&file, source("55")).unwrap();
    let returned_overlay = build(&root, &mut session, &file, "image-return-overlay", 0);
    assert!(!session.caches.overlays.is_empty());
    session.settle(); drop(session);

    // Every distinct native application executes ONCE, after all layout
    // changes and native session retirement. Earlier images must still find
    // their original bases; no retry or compiler fallback is permitted.
    for (label, output, expected, image) in [
        ("one-shot-default", once, "11\n", false),
        ("image-default", first, "11\n", true),
        ("image-overlay", overlay, "22\n", true),
        ("standalone-recovery", standalone, "33\n", false),
        ("image-return-recovery", returned, "44\n", true),
        ("image-return-overlay", returned_overlay, "55\n", true),
    ] {
        closure(&root, label, &output, image);
        let result = std::process::Command::new(output).output().unwrap();
        put(&root.join(format!("{label}.run.stdout")), &result.stdout);
        put(&root.join(format!("{label}.run.stderr")), &result.stderr);
        put(&root.join(format!("{label}.run.status")), format!("{}\n", result.status).as_bytes());
        assert!(result.status.success(), "{label}: failed application must not replay");
        assert_eq!(result.stdout, expected.as_bytes(), "{label}");
    }
    println!("native constructor defaults, populated overlays, image/standalone/image transitions, rejection/recovery and retained image lifetimes passed; performance and native crate coverage remain open");
}
