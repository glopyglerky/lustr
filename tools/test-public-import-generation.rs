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
    let status = Command::new(&args[0]).args(&args[1..]).status().unwrap();
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
        let request: Value = serde_json::from_slice(&fs::read(self.root.join("captures/reader.json")).unwrap()).unwrap();
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

    write(&provider, PROVIDER.replace("63u64", "127u64"));
    test.run("provider-body", &mut test.cargo(), 0);
    test.observe("provider-body", Some(&[]));
    test.oracle("original-reads-updated-provider", 0);

    write(&provider, PROVIDER.replace("u64", "u32"));
    let diagnostics = test.run("provider-type", &mut test.cargo(), 101);
    assert!(String::from_utf8_lossy(&diagnostics).contains("L0110"));
    test.observe("provider-type", Some(&["read"]));
    test.oracle("original-type-mismatch", 1);

    write(&provider, PROVIDER);
    test.run("type-repair", &mut test.cargo(), 0);
    test.observe("type-repair", Some(&["read"]));
    test.run("repaired-noop", &mut test.replay(), 0);
    test.observe("repaired-noop", Some(&[]));

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
    write(test.root.join("passed.json"), serde_json::to_vec_pretty(&json!({"passed":true, "scope":"scalar retained metadata generation; GAT service and wider artifacts remain open", "history":test.history})).unwrap());
    println!("default Cargo check and retained imported-generation history passed");
}
