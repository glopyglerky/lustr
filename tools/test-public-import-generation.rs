//! Exercise imported-generation edits through installed Lustr and ordinary
//! default-profile Cargo check. Compile this development tool with serde_json.
//! The wrapper records actual Cargo argv for independent compiler-request replay;
//! it forwards each request exactly once and never selects another compiler.
use std::{collections::BTreeSet, fs, path::{Path, PathBuf}, process::{Child, Command, Stdio}, time::{Duration, Instant}};
use serde_json::{Value, json};

const PROVIDER: &str = "#![no_std]\npub fn project(x: u64) -> u64 { x & 63u64 }\n";
const CONSUMER: &str = "#![no_std]\npub fn read(x: u64) -> u64 { renamed::project(x) }\npub fn spare(x: u64) -> u64 { x ^ 7u64 }\n";

fn write(path: impl AsRef<Path>, data: impl AsRef<[u8]>) { fs::write(path, data).unwrap(); }

fn wrapper(args: &[String], captures: &Path) -> ! {
    if let Some(index) = args.iter().position(|arg| arg == "--crate-name")
        && args.iter().any(|arg| arg == "--emit=dep-info,metadata") {
        let env: Vec<_> = ["CARGO_CRATE_NAME", "CARGO_MANIFEST_DIR", "CARGO_MANIFEST_PATH", "CARGO_PKG_NAME", "CARGO_BIN_NAME"]
            .into_iter().filter_map(|name| std::env::var(name).ok().map(|value| (name, value))).collect();
        write(captures.join(format!("{}.json", args[index + 1])),
            serde_json::to_vec_pretty(&json!({"program":args[0], "args":&args[1..], "cwd":std::env::current_dir().unwrap(), "env":env})).unwrap());
    }
    // The recorder is an evaluator around Cargo, not a requested compiler
    // wrapper. The candidate keeps its existing external-wrapper refusal.
    let status = Command::new(&args[0]).args(&args[1..]).env_remove("RUSTC_WRAPPER").status().unwrap();
    std::process::exit(status.code().unwrap_or(125));
}

struct Daemon(Child);
impl Drop for Daemon { fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); } }

struct Test {
    root: PathBuf,
    package: PathBuf,
    sdk: PathBuf,
    report_paths: BTreeSet<PathBuf>,
    history: Vec<Value>,
}
impl Test {
    fn command(&self, program: impl AsRef<Path>) -> Command {
        let mut command = Command::new(program.as_ref());
        command.env_clear().current_dir(self.root.join("workspace"))
            .env("PATH", "/usr/bin:/bin").env("HOME", self.root.join("home"))
            .env("CARGO_HOME", self.root.join("cargo-home")).env("TMPDIR", self.root.join("tmp"))
            .env("CARGO_TERM_COLOR", "never").env("LUST_CACHE_DIR", self.root.join("cache"))
            .env("LUST_DAEMON_SOCKET", self.root.join("s")).env("LUST_DAEMON_IDLE", "300")
            .env("LUSTR_LIBRARY_REPORT_DIR", self.root.join("reports"));
        command
    }
    fn run(&self, label: &str, command: &mut Command, expected: i32) -> Vec<u8> {
        write(self.root.join(format!("{label}.command")), format!("{command:?}\n"));
        let started = Instant::now();
        let output = command.output().unwrap();
        write(self.root.join(format!("{label}.stdout")), &output.stdout);
        write(self.root.join(format!("{label}.stderr")), &output.stderr);
        write(self.root.join(format!("{label}.result.json")), serde_json::to_vec_pretty(&json!({"code":output.status.code(), "elapsed_ns":started.elapsed().as_nanos()})).unwrap());
        assert_eq!(output.status.code(), Some(expected), "{label}: {}", String::from_utf8_lossy(&output.stderr));
        [output.stdout, output.stderr].concat()
    }
    fn cargo(&self) -> Command {
        let mut command = self.command(self.package.join("bin/cargo"));
        command.args(["check", "--workspace", "--offline", "--message-format=json", "--jobs", "1", "--target-dir"])
            .arg(self.root.join("target")).arg("--config")
            .arg(format!("build.build-dir={:?}", self.root.join("intermediates")))
            .env("RUSTC", self.package.join("bin/rustc"))
            .env("RUSTC_WRAPPER", std::env::current_exe().unwrap()).env("LUSTR_CAPTURE", self.root.join("captures"));
        command
    }
    fn replay(&self) -> Command {
        self.replay_named("reader")
    }
    fn replay_named(&self, name: &str) -> Command {
        let request: Value = serde_json::from_slice(&fs::read(self.root.join(format!("captures/{name}.json"))).unwrap()).unwrap();
        let mut command = self.command(self.package.join("bin/rustc"));
        command.current_dir(request["cwd"].as_str().unwrap());
        command.args(request["args"].as_array().unwrap().iter().map(|arg| arg.as_str().unwrap()));
        for pair in request["env"].as_array().unwrap() {
            command.env(pair[0].as_str().unwrap(), pair[1].as_str().unwrap());
        }
        command
    }
    fn reports(&mut self) -> Vec<Value> {
        let paths: BTreeSet<_> = fs::read_dir(self.root.join("reports")).unwrap().map(|entry| entry.unwrap().path().join("report.json")).collect();
        let records = paths.difference(&self.report_paths).map(|path| serde_json::from_slice(&fs::read(path).unwrap()).unwrap()).collect();
        self.report_paths = paths;
        records
    }
    fn observe(&mut self, label: &str, expected: Option<&[&str]>) {
        let records = self.reports();
        let consumers: Vec<_> = records.iter().filter(|record| record["context"]["crate_name"] == "reader").collect();
        if let Some(expected) = expected {
            assert_eq!(consumers.len(), 1, "{label}: no unique actual reader request: {records:?}");
            let names: BTreeSet<_> = consumers[0]["checked_definitions"].as_array().unwrap().iter()
                .map(|definition| definition["name"].as_str().unwrap()).collect();
            assert_eq!(names, expected.iter().copied().collect(), "{label}: unrelated body rechecked");
        }
        self.history.push(json!({"case":label, "records":records}));
        write(self.root.join("history.json"), serde_json::to_vec_pretty(&self.history).unwrap());
    }
    fn oracle(&self, label: &str, expected: i32) {
        let request: Value = serde_json::from_slice(&fs::read(self.root.join("captures/reader.json")).unwrap()).unwrap();
        let args: Vec<_> = request["args"].as_array().unwrap().iter().map(|arg| arg.as_str().unwrap()).collect();
        let external = args.iter().position(|arg| *arg == "--extern").unwrap();
        let dependency = args[external + 1];
        let (_, path) = dependency.split_once('=').unwrap();
        let mut command = self.command(self.sdk.join("bin/rustc"));
        command.args(["--edition=2024", "--crate-type=lib", "--emit=metadata", "consumer/src/lib.rs", "--extern", dependency, "-L"])
            .arg(format!("dependency={}", Path::new(path).parent().unwrap().display()))
            .arg("-o").arg(self.root.join(format!("{label}.rmeta"))).env("LD_LIBRARY_PATH", self.sdk.join("lib"));
        self.run(label, &mut command, expected);
    }
    fn metadata(&self, name: &str) -> PathBuf {
        let request: Value = serde_json::from_slice(&fs::read(self.root.join(format!("captures/{name}.json"))).unwrap()).unwrap();
        let args = request["args"].as_array().unwrap();
        let index = args.iter().position(|arg| arg == "--out-dir").unwrap();
        let directory = Path::new(args[index + 1].as_str().unwrap());
        let paths: Vec<_> = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path())
            .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with(&format!("lib{name}-")) && path.extension().is_some_and(|ext| ext == "rmeta")).collect();
        assert_eq!(paths.len(), 1); paths[0].clone()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(captures) = std::env::var_os("LUSTR_CAPTURE") { wrapper(&args, Path::new(&captures)); }
    let [package, sdk, root] = args.as_slice() else { panic!("usage: PACKAGE PINNED_SDK NEW_OUTPUT"); };
    let root = PathBuf::from(root);
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    assert!(root.join("s").as_os_str().len() <= 107, "socket path too long");
    for path in ["workspace/provider/src", "workspace/consumer/src", "reports", "captures", "home", "cargo-home", "tmp"] { fs::create_dir_all(root.join(path)).unwrap(); }
    write(root.join("workspace/Cargo.toml"), "[workspace]\nmembers = [\"provider\", \"consumer\"]\nresolver = \"3\"\n");
    write(root.join("workspace/provider/Cargo.toml"), "[package]\nname = \"schema\"\nversion = \"0.0.0\"\nedition = \"2024\"\n");
    write(root.join("workspace/consumer/Cargo.toml"), "[package]\nname = \"reader\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nrenamed = { package = \"schema\", path = \"../provider\" }\n");
    let provider = root.join("workspace/provider/src/lib.rs");
    write(&provider, PROVIDER);
    write(root.join("workspace/consumer/src/lib.rs"), CONSUMER);
    let mut test = Test { root, package:PathBuf::from(package).canonicalize().unwrap(), sdk:PathBuf::from(sdk).canonicalize().unwrap(), report_paths:BTreeSet::new(), history:Vec::new() };
    let mut daemon_command = test.command(test.package.join("bin/lust-core"));
    daemon_command.arg("daemon").stdout(Stdio::from(fs::File::create(test.root.join("daemon.stdout")).unwrap()))
        .stderr(Stdio::from(fs::File::create(test.root.join("daemon.stderr")).unwrap()));
    let mut daemon = Daemon(daemon_command.spawn().unwrap());
    let start = Instant::now();
    while !test.root.join("s").exists() {
        assert!(daemon.0.try_wait().unwrap().is_none(), "daemon exited");
        assert!(start.elapsed() < Duration::from_secs(15), "daemon did not bind");
        std::thread::sleep(Duration::from_millis(10));
    }
    test.run("cold", &mut test.cargo(), 0);
    test.observe("cold", Some(&["read", "spare"]));
    test.oracle("original-reads-native-provider", 0);
    test.run("cargo-noop", &mut test.cargo(), 0);
    assert!(test.reports().is_empty(), "Cargo noop unexpectedly compiled");
    test.run("request-noop", &mut test.replay(), 0);
    test.observe("request-noop", Some(&[]));

    let provider_output = test.metadata("schema");
    let provider_bytes = fs::read(&provider_output).unwrap();
    let reader_output = test.metadata("reader");
    let reader_bytes = fs::read(&reader_output).unwrap();
    write(&provider, PROVIDER.replace("x & 63u64", "true"));
    test.run("invalid-provider-request", &mut test.replay_named("schema"), 1);
    assert_eq!(fs::read(&provider_output).unwrap(), provider_bytes, "invalid provider request replaced valid metadata");
    test.observe("invalid-provider-request", None);
    let invalid = test.run("invalid-provider", &mut test.cargo(), 101);
    assert!(String::from_utf8_lossy(&invalid).contains("L0110"));
    // Cargo may remove stale output before invoking its compiler. Any output
    // still present must be the previous valid bytes, never a failed result.
    if let Ok(bytes) = fs::read(&provider_output) { assert_eq!(bytes, provider_bytes, "invalid provider published metadata"); }
    assert_eq!(fs::read(&reader_output).unwrap(), reader_bytes);
    test.observe("invalid-provider", None);
    write(&provider, PROVIDER);
    test.run("provider-repair", &mut test.cargo(), 0);
    test.observe("provider-repair", Some(&[]));

    write(&provider, PROVIDER.replace("63u64", "127u64"));
    test.run("provider-body", &mut test.cargo(), 0);
    test.observe("provider-body", Some(&[]));
    test.oracle("original-reads-updated-provider", 0);
    let reader_bytes = fs::read(&reader_output).unwrap();

    write(&provider, PROVIDER.replace("u64", "u32"));
    let diagnostics = test.run("provider-type", &mut test.cargo(), 101);
    assert!(String::from_utf8_lossy(&diagnostics).contains("L0110"));
    if let Ok(bytes) = fs::read(&reader_output) { assert_eq!(bytes, reader_bytes, "invalid consumer replaced the last valid metadata"); }
    test.observe("provider-type", Some(&["read"]));
    test.oracle("original-type-mismatch", 1);

    write(&provider, PROVIDER);
    test.run("type-repair", &mut test.cargo(), 0);
    test.observe("type-repair", Some(&["read"]));
    test.run("repaired-noop", &mut test.replay(), 0);
    test.observe("repaired-noop", Some(&[]));

    write(&provider, PROVIDER.replace("project", "renamed_definition"));
    test.run("missing-definition", &mut test.cargo(), 101);
    test.observe("missing-definition", None);
    test.oracle("original-missing-definition", 1);
    write(&provider, PROVIDER);
    test.run("definition-repair", &mut test.cargo(), 0);
    test.observe("definition-repair", Some(&[]));
    test.run("definition-repair-noop", &mut test.replay(), 0);
    test.observe("definition-repair-noop", Some(&[]));

    // A malformed replacement must fail before touching retained semantic
    // state or the last successful public output. Restore exact bytes, then
    // require an actual compiler request to reuse both unchanged bodies.
    let request: Value = serde_json::from_slice(&fs::read(test.root.join("captures/reader.json")).unwrap()).unwrap();
    let args = request["args"].as_array().unwrap();
    let external = args.iter().position(|arg| arg == "--extern").unwrap();
    let metadata = PathBuf::from(args[external + 1].as_str().unwrap().split_once('=').unwrap().1);
    let valid = fs::read(&metadata).unwrap();
    write(&metadata, b"malformed metadata");
    let failure = test.run("malformed-input", &mut test.replay(), 3);
    assert!(String::from_utf8_lossy(&failure).contains("extern artifact"));
    assert!(test.reports().is_empty());
    write(&metadata, valid);
    test.run("malformed-repair", &mut test.replay(), 0);
    test.observe("malformed-repair", Some(&[]));

    // Ordinary associated items remain native-owned. Differential checks run
    // through the installed public CLI and the independent pinned Rust reader.
    let copy = "struct Codec; trait Family { type View where Self: Copy; } impl Family for Codec { type View = u32; } fn main() {}";
    let outlives = "trait Family { type View where Self: 'static; } impl<'b> Family for &'b u8 { type View = u32; } fn main() {}";
    let region_trait = "trait Tag<T> {} trait Family { type View where Self: Tag<&'static u8>; } impl<'a> Tag<&'a u8> for &'a u8 {} impl<'b> Family for &'b u8 { type View = u32; } fn main() {}";
    let projection = "trait Family { type View where Self: Copy; } fn take<T: Family>(_: T::View) {} fn main() {}";
    let predicate_source = test.root.join("predicates.rs");
    for (label, source, status) in [
        ("copy-invalid", copy.to_owned(), 1),
        ("copy-valid", copy.replace("struct Codec;", "#[derive(Copy, Clone)] struct Codec;"), 0),
        ("projection-invalid", projection.to_owned(), 1),
        ("projection-valid", projection.replace("T: Family>", "T: Family + Copy>"), 0),
        ("outlives-invalid", outlives.to_owned(), 1),
        ("outlives-valid", outlives.replace("impl<'b> Family for &'b u8", "impl Family for &'static u8"), 0),
        ("region-trait-invalid", region_trait.to_owned(), 1),
        ("region-trait-valid", region_trait.replace("impl<'b> Family for &'b u8", "impl Family for &'static u8"), 0),
    ] {
        write(&predicate_source, source);
        let mut native = test.command(test.package.join("bin/lustr"));
        native.arg("check").arg(&predicate_source).arg("-o").arg(test.root.join("predicate-native"));
        let native_output = test.run(&format!("predicate-{label}-native"), &mut native, status);
        let mut original = test.command(test.sdk.join("bin/rustc"));
        original.args(["--edition=2024", "--emit=metadata"]).arg(&predicate_source)
            .arg("-o").arg(test.root.join("predicate-original.rmeta")).env("LD_LIBRARY_PATH", test.sdk.join("lib"));
        let original_output = test.run(&format!("predicate-{label}-original"), &mut original, status);
        if status != 0 {
            assert!(String::from_utf8_lossy(&native_output).contains(if matches!(label, "copy-invalid" | "projection-invalid") { "L0612" } else { "L0617" }));
            assert!(String::from_utf8_lossy(&original_output).contains("error[E"));
        }
    }

    // Separate linked-artifact regression control, explicitly using the
    // existing abort/debug-disabled link context. This does not qualify an
    // ordinary default-profile build or run.
    fs::create_dir_all(test.root.join("linked/src")).unwrap();
    write(test.root.join("linked/Cargo.toml"), "[package]\nname = \"linked_generation\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nschema = { path = \"../workspace/provider\" }\n[profile.dev]\npanic = \"abort\"\ndebug = 0\nincremental = false\n");
    write(test.root.join("linked/src/main.rs"), "fn main() { println!(\"{}\", schema::project(255u64)); }\n");
    test.reports();
    for (label, mask, expected) in [("cold", "63u64", b"63\n".as_slice()), ("noop", "63u64", b"63\n"), ("provider-edit", "127u64", b"127\n")] {
        if label != "noop" { write(&provider, PROVIDER.replace("63u64", mask)); }
        let mut command = test.command(test.package.join("bin/cargo"));
        command.current_dir(test.root.join("linked")).args(["run", "--offline", "--jobs", "1", "--target-dir"])
            .arg(test.root.join("linked-target")).arg("--config")
            .arg(format!("build.build-dir={:?}", test.root.join("linked-intermediates")))
            .env("RUSTC", test.package.join("bin/rustc"));
        let label = format!("linked-{label}");
        test.run(&label, &mut command, 0);
        assert_eq!(fs::read(test.root.join(format!("{label}.stdout"))).unwrap(), expected);
        let records = test.reports();
        let executables: Vec<_> = records.iter().filter(|record|record["schema"] == "lustr-public-executable-v1").collect();
        if label == "linked-noop" { assert!(records.is_empty()); }
        else {
            assert_eq!(executables.len(), 1);
            assert_eq!(executables[0]["artifact_reused"], false);
            assert_eq!(executables[0]["checked_bodies"], if label == "linked-cold" { 1 } else { 0 });
        }
        test.history.push(json!({"case":label,"records":records}));
        write(test.root.join("history.json"), serde_json::to_vec_pretty(&test.history).unwrap());
    }
    write(test.root.join("passed.json"), serde_json::to_vec_pretty(&json!({"passed":true, "scope":"scalar retained metadata generation; GAT service and wider artifacts remain open", "history":test.history})).unwrap());
    println!("default Cargo check and retained imported-generation history passed");
}
