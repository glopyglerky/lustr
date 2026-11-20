//! Native source/declaration/body/object joining controls. This executable
//! compiles consumer bodies only through the actual native pipeline. Its first
//! tests qualify the metadata input boundary, not a completed metadata writer.
#![allow(dead_code, unused_imports)]
include!("../crates/lustr-native-core/src/lib.rs");

#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

use std::path::{Path, PathBuf};

fn args(source: &Path, output: &Path) -> Vec<String> {
    [
        source.to_str().unwrap(),
        "--crate-name",
        "consumer",
        "--edition=2024",
        "--crate-type=lib",
        "--emit=obj",
        "--target=x86_64-unknown-linux-gnu",
        "-Cmetadata=native_consumer_v1",
        "-Cpanic=abort",
        "-Copt-level=0",
        "-Cdebuginfo=0",
        "-Coverflow-checks=on",
        "-Cdebug-assertions=on",
        "-o",
        output.to_str().unwrap(),
    ]
    .iter()
    .map(|s| (*s).into())
    .collect()
}

fn put(path: &Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}

fn run(root: &Path, label: &str, command: &mut std::process::Command) -> std::process::Output {
    put(&root.join(format!("{label}.command")), format!("{command:?}\n").as_bytes());
    let result = command.output().unwrap();
    put(&root.join(format!("{label}.stdout")), &result.stdout);
    put(&root.join(format!("{label}.stderr")), &result.stderr);
    put(&root.join(format!("{label}.status")), format!("{}\n", result.status).as_bytes());
    assert!(result.status.success(), "{label}: {}", String::from_utf8_lossy(&result.stderr));
    result
}

fn duplicate_definition(object: &[u8]) -> Vec<u8> {
    let read = |at: usize, width: usize| {
        let mut bytes = [0; 8];
        bytes[..width].copy_from_slice(&object[at..at + width]);
        u64::from_le_bytes(bytes) as usize
    };
    let sections = read(40, 8);
    let count = read(60, 2);
    let mut defined = Vec::new();
    for section in 0..count {
        let header = sections + section * 64;
        if read(header + 4, 4) != 2 {
            continue;
        }
        let start = read(header + 24, 8);
        let len = read(header + 32, 8);
        for symbol in (start..start + len).step_by(24) {
            if object[symbol + 4] >> 4 != 0 && read(symbol + 6, 2) != 0 {
                defined.push(symbol);
            }
        }
    }
    assert!(defined.len() >= 2);
    let mut changed = object.to_vec();
    changed[defined[1]..defined[1] + 4].copy_from_slice(&object[defined[0]..defined[0] + 4]);
    changed
}

fn main() {
    raise_stack_limit();
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(arguments.len(), 2, "NEW_OUTPUT_DIRECTORY PINNED_RUSTC");
    let root = PathBuf::from(&arguments[0]);
    std::fs::create_dir(&root).unwrap();
    let rustc = PathBuf::from(&arguments[1]);
    let identity = run(&root, "oracle-identity", std::process::Command::new(&rustc).arg("-vV"));
    assert!(String::from_utf8(identity.stdout).unwrap().contains("commit-hash: 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494\n"));
    let mut controls = 0;

    let good = "#![no_std]\npub fn transform(value: u64) -> u64 { helper(value) ^ 85u64 }\nfn helper(value: u64) -> u64 { value & 255u64 }\npub fn combine(left: u64, right: u64) -> u64 { left | right }\n";
    let source = root.join("original.rs");
    put(&source, good.as_bytes());
    let output = root.join("native.o");
    let prepared = native_object::prepare(&args(&source, &output)).unwrap();
    assert_eq!(prepared.source(), good);
    assert_eq!(prepared.program().fns.len(), 3);
    assert_eq!(prepared.accepted_bodies().len(), 3);
    assert!(!output.exists());
    let complete = native_consumer::complete(prepared).unwrap();
    assert_eq!(complete.functions().len(), 3);
    assert_eq!(complete.functions().iter().filter(|f| f.visibility() == native_consumer::Visibility::Public).count(), 2);
    let helper = complete.functions().iter().find(|f| f.name() == "helper").unwrap();
    let transform = complete.functions().iter().find(|f| f.name() == "transform").unwrap();
    assert_eq!(transform.calls().iter().copied().collect::<Vec<_>>(), [helper.definition()]);
    assert_eq!(helper.visibility(), native_consumer::Visibility::Private);
    for function in complete.functions() {
        for parameter in function.parameters() {
            let span = parameter.span();
            assert_eq!(&good[span.lo as usize..span.hi as usize], parameter.name());
        }
    }
    controls += 1;
    println!("native-consumer-control=inventory-and-visibility: pass");

    // Metadata/emission continues from this owned source and body generation,
    // even if a caller edits the path after native preparation has finished.
    std::fs::write(&source, "invalid changed source bytes").unwrap();
    let emitted = complete.emit().unwrap();
    assert_eq!(emitted.native().prepared().source(), good);
    assert_eq!(emitted.functions().len(), emitted.native().functions().len());
    put(&output, emitted.native().object());
    controls += 1;
    println!("native-consumer-control=owned-source-and-emission: pass");
    let duplicate = duplicate_definition(emitted.native().object());
    let error = match unit_object::symbols(&duplicate) {
        Ok(_) => panic!("duplicated emitted symbol ownership was accepted"),
        Err(error) => error,
    };
    assert_eq!(error, "duplicate nonlocal ELF symbol definition");
    put(&root.join("duplicate-symbol.o"), &duplicate);
    put(&root.join("duplicate-symbol.error"), error.as_bytes());
    controls += 1;
    println!("native-consumer-control=duplicate-symbol-rejected: pass");

    let symbol = |name: &str| {
        let definition = emitted.functions().iter().find(|f| f.name() == name).unwrap().definition();
        String::from_utf8(emitted.native().functions().iter().find(|f| f.definition() == definition).unwrap().symbol().to_vec()).unwrap()
    };
    let caller = root.join("caller.rs");
    let program = root.join("caller");
    let caller_text = format!(
        "unsafe extern \"C\" {{ #[link_name={:?}] fn transform(value:u64)->u64; #[link_name={:?}] fn combine(left:u64,right:u64)->u64; }}\nfn main() {{ for value in (0u64..65536).chain([u64::MAX, 1u64<<63]) {{ assert_eq!(unsafe{{transform(value)}}, (value & 255)^85); assert_eq!(unsafe{{combine(value, 0x101)}}, value|0x101); }} println!(\"native-body-results=131076\"); }}\n",
        symbol("transform"),
        symbol("combine")
    );
    put(&caller, caller_text.as_bytes());
    run(
        &root,
        "caller-compile",
        std::process::Command::new(&rustc)
            .args(["--edition=2024", "-Cpanic=abort"])
            .arg(&caller)
            .arg("-C")
            .arg(format!("link-arg={}", output.display()))
            .arg("-o")
            .arg(&program),
    );
    let result = run(&root, "caller-run", &mut std::process::Command::new(&program));
    assert_eq!(result.stdout, b"native-body-results=131076\n");
    assert!(result.stderr.is_empty());
    controls += 1;
    println!("native-consumer-control=exact-program-results: pass");

    let abandon_source = root.join("abandoned.rs");
    let abandon_output = root.join("abandoned.o");
    put(&abandon_source, good.as_bytes());
    let before = std::fs::read_dir(&root).unwrap().count();
    drop(native_consumer::complete(native_object::prepare(&args(&abandon_source, &abandon_output)).unwrap()).unwrap());
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), before);
    assert!(!abandon_output.exists());
    controls += 1;
    println!("native-consumer-control=abandon-no-emission: pass");

    for (name, text) in [
        ("invalid-private", "#![no_std]\npub fn visible(value:u64)->u64 { value }\nfn hidden(value:u64)->u64 { true }\n"),
        ("duplicate-declaration", "#![no_std]\npub fn value(value:u64)->u64 { value }\npub fn value(value:u64)->u64 { value }\n"),
        ("unknown-type", "#![no_std]\npub fn value(value:Missing)->u64 { 1 }\n"),
    ] {
        let file = root.join(format!("{name}.rs"));
        let out = root.join(format!("{name}.o"));
        put(&file, text.as_bytes());
        let error = match native_object::prepare(&args(&file, &out)) {
            Ok(_) => panic!("{name} was accepted without its complete native checks"),
            Err(error) => error,
        };
        put(&root.join(format!("{name}.native-error")), error.as_bytes());
        assert!(!out.exists());
        controls += 1;
        println!("native-consumer-control={name}: pass");
    }
    for (name, text) in [
        ("unused-private", "#![no_std]\npub fn visible(value:u64)->u64 { value }\nfn hidden(value:u64)->u64 { value }\n"),
        ("unused-parameter", "#![no_std]\npub fn value(unused:u64)->u64 { 1u64 }\n"),
        ("naming-lint", "#![no_std]\npub fn BadName(value:u64)->u64 { value }\n"),
        ("recursive-call", "#![no_std]\npub fn value(input:u64)->u64 { value(input) }\n"),
        ("non-u64-signature", "#![no_std]\npub fn value(input:u32)->u32 { input }\n"),
        ("const-definition", "#![no_std]\nconst VALUE:u64=1;\npub fn value(input:u64)->u64 { input }\n"),
        ("statement-lints", "#![no_std]\npub fn value(input:u64)->u64 { let other=input; other }\n"),
        ("parenthesis-lint", "#![no_std]\npub fn value(input:u64)->u64 { (input) }\n"),
    ] {
        let file = root.join(format!("{name}.rs"));
        let out = root.join(format!("{name}.o"));
        put(&file, text.as_bytes());
        let prepared = native_object::prepare(&args(&file, &out)).unwrap();
        let error = match native_consumer::complete(prepared) {
            Err(error @ native_consumer::Error::Unintegrated(_)) => error,
            _ => panic!("{name} hid an unrepresented metadata obligation"),
        };
        put(&root.join(format!("{name}.integration-error")), format!("{error:?}\n").as_bytes());
        assert!(!out.exists());
        controls += 1;
        println!("native-consumer-control={name}: pass");
    }
    // An existing artifact is never removed on preparation failure, and a
    // fresh valid recovery goes through the native checker again.
    assert!(native_object::prepare(&args(&source, &output)).is_err());
    assert_eq!(std::fs::read(&output).unwrap(), emitted.native().object());
    std::fs::write(&source, good).unwrap();
    let recovery = root.join("recovered.o");
    let recovered = native_consumer::complete(native_object::prepare(&args(&source, &recovery)).unwrap()).unwrap().emit().unwrap();
    assert_eq!(recovered.functions().len(), 3);
    assert!(!recovery.exists(), "object emission cannot publish its caller's final artifact");
    controls += 1;
    println!("native-consumer-control=preserved-output-and-recovery: pass");
    put(&root.join("summary.json"), format!("{{\"passed\":true,\"controls\":{controls},\"native_functions\":3,\"checked_runtime_results\":131076,\"scope\":\"native completed-record and real object boundary; C ABI caller, genuine Rust metadata not yet qualified\"}}\n").as_bytes());
    println!("native consumer record controls: {controls} passed; genuine Rust metadata remains separately unqualified");
}
