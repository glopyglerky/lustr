//! Current public compiler controls. Historical full-compiler fallback and
//! diagnostic-retry expectations remain only in their original Git history.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "../compiler/native/src/protocol.rs"]
#[allow(dead_code)]
mod transport_protocol;

struct Fixture {
    root: PathBuf,
    package: PathBuf,
}
impl Fixture {
    fn new(scope: &str) -> Self {
        let f = Self {
            package: PathBuf::from(std::env::var_os("LUSTR_TEST_PACKAGE").expect("LUSTR_TEST_PACKAGE")),
            root: PathBuf::from(std::env::var_os("LUSTR_TEST_OUTPUT").expect("LUSTR_TEST_OUTPUT")).join(scope),
        };
        std::fs::create_dir_all(f.root.parent().unwrap()).unwrap();
        std::fs::create_dir(&f.root).unwrap();
        f
    }
    fn command(&self, tool: &str) -> Command {
        let mut c = Command::new(self.package.join("bin").join(tool));
        let mut paths = vec![self.package.join("bin")];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
        c.current_dir(&self.root)
            .env("PATH", std::env::join_paths(paths).unwrap())
            // Cargo injects the build SDK's rustc-dev libraries into test
            // processes. Exercise the runtime's own relocated loader paths.
            .env_remove("LD_LIBRARY_PATH")
            .env_remove("CARGO_BUILD_BUILD_DIR")
            .env("CARGO_HOME", self.root.join("cargo-home"))
            .env("CARGO_TARGET_DIR", self.root.join("target"))
            .env("RUSTC", self.package.join("bin/rustc"))
            .env("RUSTDOC", self.package.join("bin/rustdoc"))
            .env("LUST_CACHE_DIR", self.root.join("native-cache"))
            .env("LUST_DAEMON_SOCKET", self.root.join("daemon.sock"))
            .env("LUST_DAEMON_IDLE", "2")
            .env("LUSTR_NATIVE_DIAGNOSTICS", self.root.join("native-diagnostics.log"))
            .env("LUSTR_TRACE_PATH", self.root.join("trace.jsonl"));
        c
    }
    fn invoke(&self, label: &str, c: &mut Command) -> Output {
        let out = c.output().unwrap();
        std::fs::write(self.root.join(format!("{label}.stdout")), &out.stdout).unwrap();
        std::fs::write(self.root.join(format!("{label}.stderr")), &out.stderr).unwrap();
        out
    }
    fn succeeds(&self, label: &str, c: &mut Command) -> Output {
        let out = self.invoke(label, c);
        assert!(out.status.success(), "{label}: {}", String::from_utf8_lossy(&out.stderr));
        out
    }
    fn source(&self, name: &str, text: &str) -> PathBuf {
        let path = self.root.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, text).unwrap();
        path
    }
    fn engine(&self) -> String {
        let trace = std::fs::read_to_string(self.root.join("trace.jsonl")).unwrap();
        serde_json::from_str::<serde_json::Value>(trace.lines().last().unwrap()).unwrap()["engine"].as_str().unwrap().to_owned()
    }
    fn entry(&self) -> String {
        let trace = std::fs::read_to_string(self.root.join("trace.jsonl")).unwrap();
        serde_json::from_str::<serde_json::Value>(trace.lines().last().unwrap()).unwrap()["entry"].as_str().unwrap().to_owned()
    }

}

fn native_package_wire_identity(package: &Path) -> String {
    use std::os::unix::fs::MetadataExt;
    // The test executable is not installed beside the compiler. Its own
    // protocol::build_id() therefore cannot identify the packaged daemon.
    let core = std::fs::metadata(package.join("bin/lust-core")).expect("packaged native compiler identity");
    format!("{}.{:x}.{:x}.{:x}.{:x}.{:x}", env!("LUST_BUILD_HASH"),
            core.dev(), core.ino(), core.size(), core.mtime(), core.mtime_nsec())
}

fn read_public_transport_request(
    reader: &mut impl std::io::BufRead,
    identity: &str,
    source: &Path,
    output: &Path,
) -> Result<(), String> {
    use std::io::{BufRead, Read};
    use std::os::unix::ffi::OsStrExt;
    let mut header = String::new();
    reader.take(256).read_line(&mut header).map_err(|error| error.to_string())?;
    let words: Vec<_> = header.split_whitespace().collect();
    if !header.ends_with('\n') || words.len() != 16 || words[0] != "build" || words[15] != identity {
        return Err("public request does not name the expected compiler build".into());
    }
    let features: usize = words[11].parse().map_err(|_| "invalid feature count")?;
    let environment: usize = if words[12] == "-" { 0 } else { words[12].parse().map_err(|_| "invalid environment count")? };
    if features > 4096 || environment > 65536 { return Err("request field count exceeds bound".into()); }
    // Consume the actual complete frame: nine fixed fields, feature names,
    // the displayed source path, then environment key/value pairs. Validate
    // identity and paths without rewriting the incoming header or overriding
    // the product's build identity.
    for index in 0..10 + features + 2 * environment {
        let mut size = String::new();
        reader.take(32).read_line(&mut size).map_err(|error| error.to_string())?;
        if !size.ends_with('\n') { return Err("truncated request field size".into()); }
        let length: usize = size.trim().parse().map_err(|_| "invalid request field size")?;
        if length > 1 << 20 { return Err("fixture request field exceeds bound".into()); }
        let mut bytes = vec![0; length];
        reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
        if index == 0 && bytes != source.as_os_str().as_bytes()
            || index == 1 && bytes != output.as_os_str().as_bytes() {
            return Err("public request names a different source or output".into());
        }
    }
    Ok(())
}

#[test]
fn transport_peer_consumes_and_validates_the_actual_encoded_request() {
    use std::io::Cursor;
    let args: Vec<String> = ["build", "/native-transport/source.rs", "-o", "/native-transport/output"]
        .into_iter().map(str::to_owned).collect();
    let Some(transport_protocol::Invocation::Compile { mut req, .. }) = transport_protocol::parse_args(&args) else { panic!("build request"); };
    req.session.features = vec!["one".into(), "two".into()];
    req.client.env = Some(vec![("KEY".into(), "value λ".into()), ("EMPTY".into(), String::new())]);
    let encoded = transport_protocol::encode(&req);
    let identity = transport_protocol::build_id();
    let mut reader = Cursor::new(&encoded);
    read_public_transport_request(&mut reader, identity, &req.file, &req.out).unwrap();
    assert_eq!(reader.position() as usize, encoded.len(), "peer must consume every environment and source field");
    assert!(read_public_transport_request(&mut Cursor::new(&encoded), "another-build", &req.file, &req.out).is_err());
    assert!(read_public_transport_request(&mut Cursor::new(&encoded), identity, Path::new("/wrong-source.rs"), &req.out).is_err());
    assert!(read_public_transport_request(&mut Cursor::new(&encoded), identity, &req.file, Path::new("/wrong-output")).is_err());
    assert!(read_public_transport_request(&mut Cursor::new(&encoded[..encoded.len() - 1]), identity, &req.file, &req.out).is_err());
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT on a Unix socket host"]
fn public_entries_keep_ambiguous_daemon_failures_terminal() {
    use std::io::{BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::time::{Duration, Instant};
    let f = Fixture::new("native-transport");
    let source = f.source("valid.rs", "fn main() { println!(\"valid native program\"); }\n");
    let artifact = f.root.join("previous-program");
    let prior = b"previous successful artifact must remain unchanged";
    let socket = f.root.join("daemon.sock");
    let identity = native_package_wire_identity(&f.package);
    let replies = [
        ("lost-reply", Vec::new()),
        ("old-protocol", b"0 0 0\n".to_vec()),
        ("wrong-build", b"0 0 0 another-build\n".to_vec()),
        ("truncated-body", format!("0 7 0 {identity}\n").into_bytes()),
        ("status-wraps-to-success", format!("256 0 0 {identity}\n").into_bytes()),
        ("second-status-wraps-to-success", format!("512 0 0 {identity}\n").into_bytes()),
        ("status-wraps-to-other-failure", format!("257 0 0 {identity}\n").into_bytes()),
        ("reported-failure", format!("3 0 7 {identity}\nfailure").into_bytes()),
    ];
    for tool in ["lust", "lustr"] {
        for (label, reply) in &replies {
            std::fs::write(&artifact, prior).unwrap();
            let listener = UnixListener::bind(&socket).expect("required real daemon socket");
            listener.set_nonblocking(true).unwrap();
            let reply = reply.clone();
            let identity = identity.clone();
            let expected_source = source.clone();
            let expected_output = artifact.clone();
            let server = std::thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(10);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        Err(error) => panic!("public entry did not deliver a request: {error}"),
                    }
                };
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                read_public_transport_request(&mut BufReader::new(&stream), &identity, &expected_source, &expected_output)
                    .expect("peer received the complete request for this installed compiler, source and output");
                stream.write_all(&reply).unwrap();
            });
            let result = f.invoke(
                &format!("{tool}-{label}"),
                f.command(tool).arg("build").arg(&source).arg("-o").arg(&artifact)
                    .env_remove("LUST_NO_DAEMON").env_remove("LUST_LIBRARY").env_remove("LUST_SOURCES_READY"),
            );
            server.join().unwrap();
            assert_eq!(result.status.code(), Some(3), "{tool}-{label}: {}", String::from_utf8_lossy(&result.stderr));
            if *label == "reported-failure" {
                assert_eq!(result.stderr, b"failure", "{tool} must preserve a valid same-build compiler failure");
            }
            assert_eq!(std::fs::read(&artifact).unwrap(), prior, "{tool}-{label} replayed a failed compiler request");
            assert!(result.stdout.is_empty(), "{tool}-{label} produced success output");
            assert!(socket.exists(), "{tool}-{label} removed an ambiguously responding daemon");
            std::fs::remove_file(&socket).unwrap();
        }
    }
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_file_tests_preserve_single_execution() {
    let f = Fixture::new("native-file-tests");
    let source = f.source("tests.rs", "fn main(){}\n#[test]\nfn works(){assert_eq!(21*2,42);}\n");
    let summary = |out: &Output| {
        let values: Vec<serde_json::Value> = out
            .stdout
            .split(|byte| *byte == b'\n')
            .filter_map(|line| serde_json::from_slice(line).ok())
            .filter(|value: &serde_json::Value| value["kind"] == "summary")
            .collect();
        assert_eq!(values.len(), 1, "one native test summary is required");
        assert_eq!(values[0]["build"], "ok");
        values.into_iter().next().unwrap()
    };
    let passed = f.succeeds("passed", f.command("lustr").arg("test").arg(&source).arg("--json"));
    assert_eq!(summary(&passed)["passed"], 1);
    assert_eq!(f.engine(), "lust-native-command");
    f.source(
        "tests.rs",
        "fn main(){}\n#[test]\nfn fails(){use std::io::Write;let mut file=std::fs::OpenOptions::new().create(true).append(true).open(\"attempts\").unwrap();file.write_all(b\"x\").unwrap();panic!(\"expected failure\");}\n",
    );
    let failed = f.invoke("failed", f.command("lustr").arg("test").arg(&source).arg("--json"));
    // The upstream native test runner follows Cargo's failed-test status.
    assert_eq!(failed.status.code(), Some(101));
    let result = summary(&failed);
    assert_eq!(result["passed"], 0);
    assert_eq!(result["failed"], 1);
    assert_eq!(result["timed_out"], 0);
    assert_eq!(std::fs::read(f.root.join("attempts")).unwrap(), b"x", "failed native test must execute exactly once");
    assert_eq!(f.engine(), "lust-native-command");
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_output_failure_does_not_become_success() {
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;
    use std::process::Stdio;
    let f = Fixture::new("native-output-failure");
    let source = f.source("main.rs", "fn main(){}\n");
    let output = f.root.join("application");
    let normal = f.succeeds(
        "timings",
        f.command("lustr").arg("build").arg(&source).arg("--timings").arg("-o").arg(&output).env("LUST_NO_DAEMON", "1"),
    );
    assert!(!normal.stderr.is_empty());
    let (write, read) = UnixStream::pair().unwrap();
    drop(read);
    let broken = f.invoke(
        "broken-stderr",
        f.command("lustr")
            .arg("build")
            .arg(source)
            .arg("--timings")
            .arg("-o")
            .arg(output)
            .env("LUST_NO_DAEMON", "1")
            .stderr(Stdio::from(OwnedFd::from(write))),
    );
    assert_eq!(broken.status.code(), Some(101));
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn large_valid_native_argument_lists_use_the_original_process() {
    let f = Fixture::new("large-native-arguments");
    let source = f.source("main.rs", "fn main(){}\n");
    let mut command = f.command("lustr");
    command.arg("build").arg(source).arg("-o").arg(f.root.join("application")).env("LUST_NO_DAEMON", "1");
    for _ in 0..2050 {
        command.args(["-C", "debug-assertions=on"]);
    }
    f.succeeds("large-argument-build", &mut command);
    assert_eq!(f.engine(), "lust");
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(trace.lines().last().unwrap()).unwrap()["entry"], "upstream-process");
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn native_library_selection_survives_process_startup() {
    let f = Fixture::new("native-library-selection");
    for (directory, package, library, kind) in [
        ("hyphen", "hyphen-lib", None, None),
        ("renamed", "app", Some("app_library"), None),
        ("rlib", "library_kind_rlib", None, Some("rlib")),
        ("staticlib", "library_kind_staticlib", None, Some("staticlib")),
        ("cdylib", "library_kind_cdylib", None, Some("cdylib")),
        ("dylib", "library_kind_dylib", None, Some("dylib")),
    ] {
        let mut library_config = if library.is_some() || kind.is_some() { "\n[lib]\n".to_owned() } else { String::new() };
        if let Some(name) = library {
            library_config.push_str(&format!("name={name:?}\n"));
        }
        if let Some(kind) = kind {
            library_config.push_str(&format!("crate-type=[{kind:?}]\n"));
        }
        let valid = "pub fn answer()->u32{42}\n".to_owned();
        let invalid = "pub fn answer()->u32{\"wrong library\"}\n".to_owned();
        let manifest = f.source(
            &format!("{directory}/Cargo.toml"),
            &format!("[package]\nname={package:?}\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n{library_config}"),
        );
        let source = f.source(&format!("{directory}/src/lib.rs"), &valid);
        if library.is_some() {
            // A mistaken fallback to the package-named binary must fail this
            // positive library check, even though both targets exist.
            f.source(&format!("{directory}/src/main.rs"), "fn main(){let _:u32=\"wrong bin\";}\n");
        }
        f.succeeds(
            &format!("{directory}-cargo-library"),
            f.command("cargo")
                .args(["check", "--offline", "--lib", "--manifest-path"])
                .arg(&manifest)
                .env("RUSTC", std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE")),
        );
        for tool in ["lust", "lustr"] {
            f.succeeds(
                &format!("{directory}-{tool}-library"),
                f.command(tool)
                    .arg("check")
                    .arg(&source)
                    .arg("--manifest-path")
                    .arg(&manifest)
                    .args(["--lib", "--offline"])
                    .env("LUST_NO_DAEMON", "1")
                    .env("LUST_NO_CACHE", "1"),
            );
            if tool == "lustr" {
                assert_eq!(f.engine(), "lust", "valid library must use native checking");
                assert_eq!(f.entry(), "upstream-process", "the process must reparse the original library selection");
            }
        }
        // The same selection must reject an invalid library even when a
        // package-named binary is valid; accepted-to-rejected edits stay real.
        f.source(&format!("{directory}/src/lib.rs"), &invalid);
        if library.is_some() {
            f.source(&format!("{directory}/src/main.rs"), "fn main(){}\n");
        }
        for tool in ["lust", "lustr"] {
            let rejected = f.invoke(
                &format!("{directory}-{tool}-rejected-library"),
                f.command(tool)
                    .arg("check")
                    .arg(&source)
                    .arg("--manifest-path")
                    .arg(&manifest)
                    .args(["--lib", "--offline"])
                    .env("LUST_NO_DAEMON", "1")
                    .env("LUST_NO_CACHE", "1"),
            );
            assert!(!rejected.status.success(), "{tool} accepted the invalid {directory} library");
            assert!(!rejected.stderr.is_empty(), "{tool} lost the library diagnostic");
        }
        f.source(&format!("{directory}/src/lib.rs"), &valid);
        if library.is_some() {
            f.source(&format!("{directory}/src/main.rs"), "fn main(){let _:u32=\"wrong bin\";}\n");
        }
        for tool in ["lust", "lustr"] {
            f.succeeds(
                &format!("{directory}-{tool}-recovered-library"),
                f.command(tool)
                    .arg("check")
                    .arg(&source)
                    .arg("--manifest-path")
                    .arg(&manifest)
                    .args(["--lib", "--offline"])
                    .env("LUST_NO_DAEMON", "1")
                    .env("LUST_NO_CACHE", "1"),
            );
            if tool == "lustr" {
                assert_eq!(f.engine(), "lust", "recovered library must return to native checking");
                assert_eq!(f.entry(), "upstream-process");
            }
        }
    }
    let manifest = f.source("bin-only/Cargo.toml", "[package]\nname=\"binary_only\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n");
    let source = f.source("bin-only/src/main.rs", "fn main(){}\n");
    let oracle = f.invoke(
        "bin-only-cargo-library",
        f.command("cargo")
            .args(["check", "--offline", "--lib", "--manifest-path"])
            .arg(&manifest)
            .env("RUSTC", std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE")),
    );
    assert!(!oracle.status.success(), "Cargo unexpectedly found a library in the bin-only fixture");
    for tool in ["lust", "lustr"] {
        let rejected = f.invoke(
            &format!("bin-only-{tool}-library"),
            f.command(tool)
                .arg("check")
                .arg(&source)
                .arg("--manifest-path")
                .arg(&manifest)
                .args(["--lib", "--offline"])
                .env("LUST_NO_DAEMON", "1")
                .env("LUST_NO_CACHE", "1"),
        );
        assert_eq!(rejected.status.code(), Some(2), "{tool} substituted a binary for the absent library");
        assert!(!rejected.stderr.is_empty(), "{tool} lost the target-selection diagnostic");
    }
}
