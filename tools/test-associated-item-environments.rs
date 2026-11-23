//! Differential provider-scope and retained-edit qualification. Expectations
//! are fixed in the fixture before either compiler runs. The pinned reference
//! runs first; a candidate failure is terminal, never a compiler fallback.
use std::{fs, path::{Path, PathBuf}, process::{Child, Command, Stdio}, time::{Duration, Instant}};
use serde_json::{Value, json};

fn write(path: impl AsRef<Path>, data: impl AsRef<[u8]>) { fs::write(path, data).unwrap(); }
fn sha(path: &Path) -> String {
    let output = Command::new("sha256sum").arg(path).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().split_whitespace().next().unwrap().into()
}
struct Daemon(Child);
impl Drop for Daemon { fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); } }
struct Run { sdk: PathBuf, package: Option<PathBuf>, root: PathBuf, operations: Vec<Value> }
impl Run {
    fn command(&self, program: &Path) -> Command {
        let mut command = Command::new(program);
        command.env_clear().current_dir(&self.root).env("PATH", "/usr/bin:/bin")
            .env("HOME", self.root.join("home")).env("CARGO_HOME", self.root.join("cargo-home"))
            .env("TMPDIR", self.root.join("tmp")).env("LANG", "C").env("LC_ALL", "C")
            .env("LUST_CACHE_DIR", self.root.join("cache")).env("LUST_DAEMON_SOCKET", self.root.join("s"))
            .env("LUST_DAEMON_IDLE", "300").env("LUST_STATS", "1");
        command
    }
    fn run(&mut self, label: &str, command: &mut Command, expected: i32) -> Vec<u8> {
        write(self.root.join(format!("{label}.command")), format!("{command:?}\n"));
        let start = Instant::now();
        let output = command.output().unwrap();
        write(self.root.join(format!("{label}.stdout")), &output.stdout);
        write(self.root.join(format!("{label}.stderr")), &output.stderr);
        self.operations.push(json!({"case":label,"code":output.status.code(),"expected":expected,"elapsed_ns":start.elapsed().as_nanos(),
            "stdout_sha256":sha(&self.root.join(format!("{label}.stdout"))),"stderr_sha256":sha(&self.root.join(format!("{label}.stderr")))}));
        write(self.root.join("operations.json"), serde_json::to_vec_pretty(&self.operations).unwrap());
        assert_eq!(output.status.code(), Some(expected), "{label}: {}", String::from_utf8_lossy(&output.stderr));
        [output.stdout, output.stderr].concat()
    }
    fn reference(&mut self, label: &str, source: &Path, expected: i32) {
        let mut command = self.command(&self.sdk.join("bin/rustc"));
        command.args(["--edition=2024", "--crate-name", "item_environment", "--emit=metadata"])
            .arg(source).arg("-o").arg(self.root.join("reference.rmeta")).env("LD_LIBRARY_PATH", self.sdk.join("lib"));
        let output = self.run(label, &mut command, expected);
        if expected != 0 { assert!(String::from_utf8_lossy(&output).contains("error[E")); }
    }
    fn native(&mut self, label: &str, source: &Path, expected: i32, retained: bool) -> String {
        let mut command = self.command(&self.package.as_ref().unwrap().join("bin/lustr"));
        command.arg("check").arg(source).args(["--timings", "-o"]).arg(self.root.join("native-output"));
        if !retained { command.env("LUST_NO_DAEMON", "1"); }
        let output = self.run(label, &mut command, expected);
        let output = String::from_utf8(output).unwrap();
        if expected != 0 { assert!(output.contains("error[L"), "not a native semantic rejection: {output}"); }
        output
    }
}

fn main() {
    let arguments: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    assert!((2..=3).contains(&arguments.len()), "usage: PINNED_SDK NEW_OUTPUT [CONSTRUCTED_PACKAGE]");
    let sdk = arguments[0].canonicalize().unwrap();
    let package = arguments.get(2).map(|path|path.canonicalize().unwrap());
    let root = &arguments[1];
    fs::create_dir(root).unwrap();
    let root = root.canonicalize().unwrap();
    for name in ["home", "cargo-home", "tmp"] { fs::create_dir(root.join(name)).unwrap(); }
    let mut test = Run { sdk, package, root, operations:Vec::new() };
    let pin = include_str!("../upstream/bluefe-core.pin");
    let expected = pin.lines().find_map(|line|line.strip_prefix("rustc_sha256 = \"").and_then(|line|line.strip_suffix('"'))).unwrap();
    assert_eq!(sha(&test.sdk.join("bin/rustc")), expected);
    let cases: Value = serde_json::from_str(include_str!("fixtures/associated-item-environments.json")).unwrap();
    let source = test.root.join("input.rs");
    // The independent expected outcomes must hold before the candidate runs.
    for case in cases.as_array().unwrap() {
        write(&source, case["source"].as_str().unwrap());
        test.reference(&format!("original-{}", case["case"].as_str().unwrap()), &source, case["status"].as_i64().unwrap() as i32);
    }
    if test.package.is_none() {
        write(test.root.join("reference-passed.json"), serde_json::to_vec_pretty(&json!({"passed":true,"cases":cases,"operations":test.operations})).unwrap());
        println!("{} pinned reference expectations passed", cases.as_array().unwrap().len());
        return;
    }
    for case in cases.as_array().unwrap() {
        write(&source, case["source"].as_str().unwrap());
        test.native(&format!("native-{}", case["case"].as_str().unwrap()), &source, case["status"].as_i64().unwrap() as i32, false);
    }

    let mut daemon_command = test.command(&test.package.as_ref().unwrap().join("bin/lust-core"));
    daemon_command.arg("daemon").stdout(Stdio::from(fs::File::create(test.root.join("daemon.stdout")).unwrap()))
        .stderr(Stdio::from(fs::File::create(test.root.join("daemon.stderr")).unwrap()));
    let mut daemon = Daemon(daemon_command.spawn().unwrap());
    let start = Instant::now();
    while !test.root.join("s").exists() {
        assert!(daemon.0.try_wait().unwrap().is_none());
        assert!(start.elapsed() < Duration::from_secs(15));
        std::thread::sleep(Duration::from_millis(10));
    }
    let base = "trait Family { type View: Copy where Self: Copy; type Other: Copy; }\nimpl<T> Family for T { type View where T: Copy = T; type Other = u32; }\nfn read(value: <u32 as Family>::View) -> u32 { value }\nfn spare(value: u32) -> u32 { value ^ 7 }\nfn main() {}\n";
    let body_edit = base.replace("value ^ 7", "value ^ 15");
    let mut history = Vec::new();
    for (label, content, status, max_checked) in [
        ("cold", base.to_owned(), 0, 3),
        ("noop", base.to_owned(), 0, 0),
        ("referent-invalid", base.replace("where T: Copy = T", "where T: Copy = u64"), 1, 3),
        ("referent-repair", base.to_owned(), 0, 1),
        ("condition-invalid", base.replace("where T: Copy = T", "where T: Clone = T"), 1, 3),
        ("condition-repair", base.to_owned(), 0, 1),
        ("sibling-invalid", base.replace("type Other = u32", "type Other = T"), 1, 3),
        ("sibling-repair", base.to_owned(), 0, 1),
        ("body-edit", body_edit.clone(), 0, 1),
        ("body-noop", body_edit, 0, 0),
    ] {
        write(&source, content);
        test.reference(&format!("history-{label}-original"), &source, status);
        let native = test.native(&format!("history-{label}-native"), &source, status, true);
        let checked = native.lines().find(|line|line.starts_with("lust-visits:")).and_then(|line|line.split_whitespace().find_map(|field|field.strip_prefix("checked=")))
            .map(|count|count.parse::<usize>().unwrap());
        if status == 0 { assert!(checked.is_some_and(|count|count <= max_checked), "{label}: lost retained work: {native}"); }
        history.push(json!({"case":label,"status":status,"checked":checked}));
        write(test.root.join("history.json"), serde_json::to_vec_pretty(&history).unwrap());
    }
    let mut binaries = serde_json::Map::new();
    for name in ["lustr", "lust", "lust-core"] { binaries.insert(name.into(),json!(sha(&test.package.as_ref().unwrap().join("bin").join(name)))); }
    write(test.root.join("verification.json"), serde_json::to_vec_pretty(&json!({
        "schema":"lustr-associated-item-environments-v1","passed":true,"rust_revision":"5b7b37aefcb5b2fba3c2392bc9065b9e4602d494",
        "full_compiler_qualification":false,"performance_qualified":false,"fresh_process_trace_qualified":false,
        "candidate_sha256":binaries,"qualifier_sha256":sha(Path::new(file!())),"cases":cases,"history":history,"operations":test.operations
    })).unwrap());
    println!("associated-item scope differential and retained histories passed");
}
