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

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn rustc_reads_line_based_argument_files_before_handling_requests() {
    let f = Fixture::new("rustc-argument-files");
    f.source("dir with spaces/main.rs", "fn main() { println!(\"{}\", 42); }\n");
    let request = |name: &str, output: &str| {
        f.source(name, &format!("--edition=2024\r\ndir with spaces/main.rs\r\n-o\r\n{output}\r\n"));
        format!("@{name}")
    };
    let native = request("native args", "dir with spaces/native");
    let original = request("original args", "dir with spaces/original");
    f.succeeds("native-build", f.command("rustc").arg(native));
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let original_command = || {
        let mut command = Command::new(&oracle);
        command.current_dir(&f.root).env_remove("LD_LIBRARY_PATH");
        command
    };
    f.succeeds("original-build", original_command().arg(original));
    for binary in ["native", "original"] {
        let out = f.succeeds(binary, &mut Command::new(f.root.join("dir with spaces").join(binary)));
        assert_eq!(out.stdout, b"42\n");
    }

    f.source("query", "-vV\n");
    let query = f.succeeds("query", f.command("rustc").arg("@query"));
    assert_eq!(query.stdout, f.succeeds("direct-query", f.command("rustc").arg("-vV")).stdout);
    std::fs::write(f.root.join("bad encoding"), [0xff]).unwrap();
    let failures = ["@not present", "@bad encoding"];
    for (label, mut command) in [("native-bad-argfiles", f.command("rustc")), ("original-bad-argfiles", original_command())] {
        let failure = f.invoke(label, command.args(failures));
        assert_eq!(failure.status.code(), Some(1));
        let message = String::from_utf8_lossy(&failure.stderr);
        assert!(message.contains("not present"));
        assert!(message.contains("UTF-8 error in bad encoding"));
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn rustc_preserves_an_explicit_host_target_and_uncolored_diagnostics() {
    let f = Fixture::new("rustc-host-options");
    let source = f.source("main.rs", "fn main() { println!(\"{}\", cfg!(target_arch = \"x86_64\") && cfg!(target_os = \"linux\")); }\n");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let original = || {
        let mut command = Command::new(&oracle);
        command.current_dir(&f.root).env_remove("LD_LIBRARY_PATH");
        command
    };
    let options = ["--edition=2024", "--target", "x86_64-unknown-linux-gnu", "--color", "never"];
    for (name, mut command) in [("native", f.command("rustc")), ("original", original())] {
        let output = f.root.join(name);
        f.succeeds(&format!("{name}-build"), command.args(options).arg(&source).arg("-o").arg(&output));
        assert_eq!(f.succeeds(name, &mut Command::new(output)).stdout, b"true\n");
    }
    f.source("main.rs", "fn main() { let _value: u32 = \"invalid\"; }\n");
    for (name, mut command) in [("native", f.command("rustc")), ("original", original())] {
        let output = f.root.join(format!("{name}-invalid"));
        let response = f.invoke(&format!("{name}-invalid"), command.args(options).arg(&source).arg("-o").arg(&output));
        assert_eq!(response.status.code(), Some(1));
        assert!(!response.stderr.contains(&27), "unexpected escape sequence");
        assert!(!output.exists());
    }
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn rustc_compiles_anonymous_stdin_with_its_original_source_behavior() {
    use std::process::Stdio;

    let f = Fixture::new("rustc-stdin");
    let text = f.source("present.txt", "actual input\n");
    f.source("side.rs", "pub fn number() -> u32 { 42 }\n");
    let source = f.source("stdin.txt", r#"
mod side;
fn main() {
    println!("{}:{}:{}", file!(), module_path!(), side::number());
    print!("{}", include_str!(env!("LUSTR_STDIN_TEST_INCLUDE")));
}
"#);
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let original = || {
        let mut command = Command::new(&oracle);
        command.current_dir(&f.root).env_remove("LD_LIBRARY_PATH");
        command
    };
    let input = |command: &mut Command, output: &Path| {
        command.args(["--edition=2024", "-"]).arg("-o").arg(output)
            .env("LUSTR_STDIN_TEST_INCLUDE", &text).stdin(Stdio::from(std::fs::File::open(&source).unwrap()));
    };
    for (name, mut command) in [("native", f.command("rustc")), ("original", original())] {
        let output = f.root.join(name);
        input(&mut command, &output);
        f.succeeds(&format!("{name}-build"), &mut command);
        assert_eq!(f.succeeds(name, &mut Command::new(output)).stdout, b"<anon>:rust_out:42\nactual input\n");
    }
    f.source("stdin.txt", "fn main() { println!(\"{}:{}\", file!(), module_path!()); }\n");
    for (name, mut command) in [("native", f.command("rustc")), ("original", original())] {
        let cwd = f.root.join(format!("{name}-default-output"));
        std::fs::create_dir(&cwd).unwrap();
        command.current_dir(&cwd).args(["--edition=2024", "-"])
            .stdin(Stdio::from(std::fs::File::open(&source).unwrap()));
        f.succeeds(&format!("{name}-default-build"), &mut command);
        assert_eq!(f.succeeds(&format!("{name}-default-run"), &mut Command::new(cwd.join("rust_out"))).stdout,
            b"<anon>:rust_out\n");
    }
    f.source("stdin.txt", "fn main() { println!(\"{}\", include_str!(\"present.txt\")); }\n");
    for (name, mut command) in [("native", f.command("rustc")), ("original", original())] {
        let output = f.root.join(format!("{name}-invalid"));
        input(&mut command, &output);
        let response = f.invoke(&format!("{name}-invalid"), &mut command);
        assert_eq!(response.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&response.stderr).contains("non-file source `<anon>`"));
        assert!(!output.exists());
    }
    std::fs::write(&source, [0xff]).unwrap();
    let mut command = f.command("rustc");
    input(&mut command, &f.root.join("invalid-utf8"));
    let response = f.invoke("invalid-utf8", &mut command);
    assert_eq!(response.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&response.stderr).contains("did not contain valid UTF-8"));
    assert!(!f.root.join("invalid-utf8").exists());
    assert!(!f.root.join("<anon>").exists());
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn rustc_preserves_doctest_stdin_paths_and_diagnostic_lines() {
    use std::process::Stdio;

    let f = Fixture::new("rustdoc-stdin-origin");
    let path = f.source("docs/source.rs", "// The generated input arrives on stdin.\n");
    f.source("docs/value.txt", "absolute document directory\n");
    let input = f.source("stdin.txt", r#"fn main() {
    println!("{}:{}", file!(), line!());
    print!("{}", include_str!("value.txt"));
}
"#);
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let commands = || [("native", f.command("rustc")), ("original", Command::new(&oracle))];
    let request = |command: &mut Command, output: &Path, offset: &str| {
        command.current_dir(&f.root).env_remove("LD_LIBRARY_PATH")
            .env("UNSTABLE_RUSTDOC_TEST_PATH", &path)
            .env("UNSTABLE_RUSTDOC_TEST_LINE", offset)
            .args(["--edition", "2024", "--crate-type", "bin", "-"]).arg("-o").arg(output)
            .stdin(Stdio::from(std::fs::File::open(&input).unwrap()));
    };
    for (name, mut command) in commands() {
        let output = f.root.join(name);
        request(&mut command, &output, "24");
        f.succeeds(&format!("{name}-build"), &mut command);
        let expected = format!("{}:2\nabsolute document directory\n", path.display());
        assert_eq!(f.succeeds(name, &mut Command::new(output)).stdout, expected.as_bytes());
    }
    f.source("stdin.txt", "fn main() {\n    let _: i32 = \"wrong\";\n}\n");
    for (offset, line) in [("24", 26), ("-1", 1)] {
        for (compiler, mut command) in commands() {
            let name = format!("{compiler}-invalid-offset{offset}");
            let output = f.root.join(&name);
            request(&mut command, &output, offset);
            let response = f.invoke(&name, &mut command);
            assert_eq!(response.status.code(), Some(1));
            let diagnostic = String::from_utf8_lossy(&response.stderr);
            assert!(diagnostic.contains(&format!("{}:{line}:", path.display())), "{diagnostic}");
            assert!(!output.exists());
        }
    }
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

fn read_public_transport_field(reader: &mut impl std::io::BufRead) -> Result<Vec<u8>, String> {
    use std::io::{BufRead, Read};
    let mut size = String::new();
    reader.take(32).read_line(&mut size).map_err(|error| error.to_string())?;
    if !size.ends_with('\n') { return Err("truncated request field size".into()); }
    let length: usize = size.trim().parse().map_err(|_| "invalid request field size")?;
    if length > 1 << 20 { return Err("fixture request field exceeds bound".into()); }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn read_public_transport_count(reader: &mut impl std::io::BufRead, name: &str, limit: usize) -> Result<usize, String> {
    let bytes = read_public_transport_field(reader)?;
    let count = std::str::from_utf8(&bytes).ok().and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| format!("invalid {name} count"))?;
    if count > limit { return Err(format!("{name} count exceeds bound")); }
    Ok(count)
}

fn read_public_transport_request(
    reader: &mut impl std::io::BufRead,
    identity: &str,
    source: &Path,
    output: &Path,
    target_explicit: bool,
    expected_library: Option<(&transport_protocol::LibraryKey, transport_protocol::library_contract::LibraryOptions)>,
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
    // Ten common fields precede the library discriminator. A library frame
    // carries six values/paths, two counted vectors, optional dependency
    // records, then the request-only options, features and client context.
    // neither trailing fields nor a substituted compiler identity are ignored.
    let mut fixed = Vec::with_capacity(10);
    for _ in 0..10 { fixed.push(read_public_transport_field(reader)?); }
    if fixed[0].as_slice() != source.as_os_str().as_bytes()
        || fixed[1].as_slice() != output.as_os_str().as_bytes() {
        return Err("public request names a different source or output".into());
    }
    if fixed[7].as_slice() != if target_explicit { b"1" } else { b"0" } {
        return Err("public request has a different explicit-target selection".into());
    }
    let present = read_public_transport_field(reader)?;
    match (present.as_slice(), expected_library) {
        (b"0", None) => {}
        (mode @ (b"1" | b"2" | b"3" | b"4" | b"5"), Some((library, options))) => {
            if (mode == b"2") != library.external.is_some() || matches!(mode, b"3" | b"4" | b"5") != library.is_executable()
                || (mode == b"4") != library.custom_test || (mode == b"5") != library.standalone {
                return Err("public library dependency presence changed".into());
            }
            if !fixed[2].is_empty() || fixed[5].as_slice() != library.target_kind().as_bytes() || fixed[6].as_slice() != library.crate_name.as_bytes()
                || fixed[8].as_slice() != source.as_os_str().as_bytes()
                || fixed[9].as_slice() != transport_protocol::library_contract::TARGET.as_bytes()
                || words[8] != "1" {
                return Err("public library request changed its crate, target or profile".into());
            }
            for (name, expected) in [
                ("crate name", library.crate_name.as_bytes()),
                ("metadata", library.metadata.as_bytes()),
                ("extra filename", library.extra_filename.as_bytes()),
                ("SDK receipt", library.sdk_receipt.as_bytes()),
                ("source path", library.source_path.as_os_str().as_bytes()),
                ("working directory", library.working_directory.as_os_str().as_bytes()),
            ] {
                if read_public_transport_field(reader)?.as_slice() != expected {
                    return Err(format!("public library request changed {name}"));
                }
            }
            let cfg_count = read_public_transport_count(reader, "cfg", 4096)?;
            if cfg_count != library.cfg.len() { return Err("public library cfg inventory changed".into()); }
            for expected in &library.cfg {
                if read_public_transport_field(reader)?.as_slice() != expected.as_bytes() {
                    return Err("public library cfg value changed".into());
                }
            }
            let cfg_count = read_public_transport_count(reader, "check-cfg", 4096)?;
            if cfg_count != library.check_cfg.len() { return Err("public library check-cfg inventory changed".into()); }
            for expected in &library.check_cfg {
                if read_public_transport_field(reader)?.as_slice() != expected.as_bytes() {
                    return Err("public library check-cfg value changed".into());
                }
            }
            let search_count = read_public_transport_count(reader, "search path", 4096)?;
            if search_count != library.search_paths.len() { return Err("public library search-path inventory changed".into()); }
            for expected in &library.search_paths {
                if read_public_transport_field(reader)?.as_slice() != expected.as_os_str().as_bytes() {
                    return Err("public library search path changed".into());
                }
            }
            if let Some(dependency) = &library.external {
                if read_public_transport_field(reader)?.as_slice() != dependency.alias.as_bytes()
                    || read_public_transport_field(reader)?.as_slice() != dependency.artifact.as_os_str().as_bytes() {
                    return Err("public library dependency alias or artifact changed".into());
                }
            }
            if let Some(dependencies) = &library.executable_dependencies {
                if read_public_transport_count(reader, "executable dependency", 4096)? != dependencies.len() {
                    return Err("executable dependency inventory changed".into());
                }
                for input in dependencies {
                    let archive = input.archive.as_deref().unwrap_or_else(|| Path::new(""));
                    if read_public_transport_field(reader)?.as_slice() != input.metadata.alias.as_bytes()
                        || read_public_transport_field(reader)?.as_slice() != input.metadata.artifact.as_os_str().as_bytes()
                        || read_public_transport_field(reader)?.as_slice() != archive.as_os_str().as_bytes() {
                        return Err("executable dependency pair changed".into());
                    }
                }
            }
            let embedding = match read_public_transport_field(reader)?.as_slice() {
                b"-" => None, b"1" => Some(true), b"0" => Some(false),
                _ => return Err("invalid public library embedding option".into()),
            };
            let force_warn = match read_public_transport_field(reader)?.as_slice() {
                b"0" => false, b"1" => true,
                _ => return Err("invalid public library forced-lint option".into()),
            };
            if embedding != options.embed_metadata || force_warn != options.force_warn_unused_crate_dependencies {
                return Err("public library request changed its metadata or lint options".into());
            }
            if read_public_transport_field(reader)?.as_slice() != options.debug.encode() {
                return Err("public library request changed its debug output options".into());
            }
            if read_public_transport_field(reader)?.as_slice() != library.optimization.value().as_bytes() {
                return Err("public library request changed its optimization level".into());
            }
            if read_public_transport_field(reader)?.as_slice() != if library.single_codegen_unit { b"1" } else { b"0" } {
                return Err("public library request changed its codegen unit choice".into());
            }
        }
        (b"0" | b"1" | b"2" | b"3" | b"4" | b"5", _) => return Err("public request changed its artifact context presence".into()),
        _ => return Err("invalid public library discriminator".into()),
    }
    for _ in 0..features { read_public_transport_field(reader)?; }
    read_public_transport_field(reader)?; // displayed source
    for _ in 0..environment {
        read_public_transport_field(reader)?; // environment name
        read_public_transport_field(reader)?; // environment value
    }
    Ok(())
}

#[test]
#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
fn public_debug_options_round_trip_without_enabling_linked_debug() {
    use transport_protocol::library_contract::{DebugInfo, DebugOptions, SplitDebug, Strip};
    let args: Vec<String> = [
        "check", "/native-transport/source.rs", "--lib", "--target", "x86_64-unknown-linux-gnu",
        "-o", "/native-transport/libpeer-wire.rmeta",
        "--lustr-library-name", "peer", "--lustr-library-cwd", "/native-transport",
        "--lustr-library-metadata", "wire", "--lustr-library-extra-filename", "-wire",
        "--lustr-library-debug-options", "422",
    ].into_iter().map(str::to_owned).collect();
    let Some(transport_protocol::Invocation::Compile { mut req, .. }) =
        transport_protocol::parse_args(&args)
    else {
        panic!("metadata-only request with full debug options");
    };
    let expected = DebugOptions { info: DebugInfo::Full, split: SplitDebug::Unpacked, strip: Strip::Symbols };
    let frame = transport_protocol::encode(&req);
    let observed = transport_protocol::read_request(&mut std::io::Cursor::new(frame)).unwrap();
    assert_eq!(observed.library_options.debug, expected);
    assert_eq!(observed.session, req.session);
    assert!(!observed.debug);

    req.cmd = transport_protocol::Cmd::Build;
    req.out.set_file_name("libpeer-wire.rlib");
    let frame = transport_protocol::encode(&req);
    assert!(transport_protocol::read_request(&mut std::io::Cursor::new(frame)).is_err());
}

#[test]
fn transport_peer_consumes_and_validates_the_actual_encoded_request() {
    use std::io::Cursor;
    let args: Vec<String> = ["build", "/native-transport/source.rs", "-o", "/native-transport/output"]
        .into_iter().map(str::to_owned).collect();
    let Some(transport_protocol::Invocation::Compile { mut req, .. }) = transport_protocol::parse_args(&args) else { panic!("build request"); };
    req.session.features = vec!["one".into(), "two".into()];
    req.client.env = Some(vec![("KEY".into(), "value λ".into()), ("EMPTY".into(), String::new())]);
    let identity = transport_protocol::build_id();
    for target_explicit in [false, true] {
        req.session.krate.target_explicit = target_explicit;
        let encoded = transport_protocol::encode(&req);
        let mut reader = Cursor::new(&encoded);
        read_public_transport_request(&mut reader, identity, &req.file, &req.out, target_explicit, None).unwrap();
        assert_eq!(reader.position() as usize, encoded.len(), "peer must consume every environment and source field");
        assert!(read_public_transport_request(&mut Cursor::new(&encoded), identity, &req.file, &req.out, !target_explicit, None).is_err());
        assert!(read_public_transport_request(&mut Cursor::new(&encoded), "another-build", &req.file, &req.out, target_explicit, None).is_err());
        assert!(read_public_transport_request(&mut Cursor::new(&encoded), identity, Path::new("/wrong-source.rs"), &req.out, target_explicit, None).is_err());
        assert!(read_public_transport_request(&mut Cursor::new(&encoded), identity, &req.file, Path::new("/wrong-output"), target_explicit, None).is_err());
        assert!(read_public_transport_request(&mut Cursor::new(&encoded[..encoded.len() - 1]), identity, &req.file, &req.out, target_explicit, None).is_err());
    }
}

#[test]
fn transport_peer_validates_library_values_and_counted_vectors() {
    use std::io::{BufRead, Cursor};
    let args: Vec<String> = ["build", "/native-transport/source.rs", "-o", "/native-transport/libtransport_peer-transport.rlib"]
        .into_iter().map(str::to_owned).collect();
    let Some(transport_protocol::Invocation::Compile { mut req, .. }) = transport_protocol::parse_args(&args) else { panic!("build request"); };
    let library = transport_protocol::LibraryKey {
        crate_name: "transport_peer".into(),
        metadata: "wire_identity".into(),
        extra_filename: "-transport".into(),
        sdk_receipt: transport_protocol::library_contract::SDK_RECEIPT.into(),
        source_path: req.file.clone(),
        working_directory: PathBuf::from("/native-transport"),
        cfg: vec!["build_mode=\"safe\"".into()],
        check_cfg: vec!["cfg(docsrs,test)".into(), "cfg(feature, values())".into()],
        search_paths: vec![PathBuf::from("/native-transport/deps-one"), PathBuf::from("/native-transport/deps-two")],
        external: None,
        executable_dependencies: None,
        standalone: false,
        optimization: transport_protocol::OptLevel::None,
        single_codegen_unit: false,
        custom_test: false,
    };
    req.session.krate.target_kind = "lib".into();
    req.session.krate.target_name = library.crate_name.clone();
    req.session.krate.target_explicit = true;
    req.session.target_triple = transport_protocol::library_contract::TARGET.into();
    req.session.profile = transport_protocol::ProfileFlags::DEV;
    req.session.library = Some(library.clone());
    req.library_options = transport_protocol::library_contract::LibraryOptions {
        embed_metadata: Some(false), force_warn_unused_crate_dependencies: true,
        ..Default::default()
    };
    req.session.features = vec!["one".into(), "two".into()];
    req.client.env = Some(vec![("KEY".into(), "value λ".into()), ("EMPTY".into(), String::new())]);
    let encoded = transport_protocol::encode(&req);
    let identity = transport_protocol::build_id();
    let mut reader = Cursor::new(&encoded);
    read_public_transport_request(&mut reader, identity, &req.file, &req.out, true, Some((&library, req.library_options))).unwrap();
    assert_eq!(reader.position() as usize, encoded.len(), "library peer must consume every vector and environment field");
    assert!(read_public_transport_request(&mut Cursor::new(&encoded), identity, &req.file, &req.out, true, None).is_err());
    assert!(read_public_transport_request(&mut Cursor::new(&encoded), identity, &req.file, &req.out, false, Some((&library, req.library_options))).is_err());

    // Keep mutations bound to actual length-prefixed fields from the real
    // encoder. This changes one field, without dropping the remaining frame.
    let mut fields = Vec::new();
    let mut cursor = Cursor::new(&encoded);
    cursor.read_line(&mut String::new()).unwrap();
    while (cursor.position() as usize) < encoded.len() {
        let start = cursor.position() as usize;
        read_public_transport_field(&mut cursor).unwrap();
        fields.push((start, cursor.position() as usize));
    }
    assert_eq!(fields.len(), 37, "library schema/options plus two features and two environment pairs");
    let replace_field = |index: usize, bytes: &[u8]| {
        let (start, end) = fields[index];
        let mut changed = encoded[..start].to_vec();
        changed.extend_from_slice(format!("{}\n", bytes.len()).as_bytes());
        changed.extend_from_slice(bytes);
        changed.extend_from_slice(&encoded[end..]);
        changed
    };
    let wrong_fields: &[(usize, &[u8])] = &[
        (10, b"2"), (10, b"0"), // invalid discriminator and lost library context
        (11, b"wrong_crate"), (12, b"wrong_metadata"), (13, b"-wrong_suffix"),
        (14, b"wrong_sdk"), (15, b"/wrong/source.rs"), (16, b"/wrong/cwd"),
        (17, b"invalid"), (17, b"4097"), (17, b"0"), // malformed, oversized or incomplete cfg inventory
        (18, b"build_mode=\"wrong\""),
        (19, b"4097"), (19, b"1"), (21, b"cfg(wrong)"),
        (22, b"4097"), (22, b"1"), // oversized or incomplete search inventory
        (24, b"/wrong/dependency"),
        (25, b"invalid"), (25, b"-"), (25, b"1"), // malformed, absent or different embedding choice
        (26, b"2"), (26, b"0"), // malformed or different forced-lint choice
        (27, b"102"), (27, b"00"), (27, b"999"), // changed, truncated or unknown debug output choices
        (28, b"3"), (28, b"9"), (28, b"00"), // different, unknown or malformed optimization level
        (29, b"1"), (29, b"2"), // different or invalid codegen unit selection
    ];
    for &(index, bytes) in wrong_fields {
        let changed = replace_field(index, bytes);
        assert!(read_public_transport_request(&mut Cursor::new(&changed), identity, &req.file, &req.out, true, Some((&library, req.library_options))).is_err(),
                "changed library field {index} was accepted");
    }
    for index in [18, 21, 24, 25, 26, 27, 28, 29, 36] {
        let truncated = &encoded[..fields[index].1 - 1];
        assert!(read_public_transport_request(&mut Cursor::new(truncated), identity, &req.file, &req.out, true, Some((&library, req.library_options))).is_err(),
                "truncated vector, request option or final environment field {index} was accepted");
    }
    for (embed_metadata, force_warn_unused_crate_dependencies) in [(None, false), (Some(true), true), (Some(false), true)] {
        req.library_options = transport_protocol::library_contract::LibraryOptions {
            embed_metadata, force_warn_unused_crate_dependencies,
            ..Default::default()
        };
        let frame = transport_protocol::encode(&req);
        let mut reader = Cursor::new(&frame);
        read_public_transport_request(&mut reader, identity, &req.file, &req.out, true, Some((&library, req.library_options))).unwrap();
        assert_eq!(reader.position() as usize, frame.len(), "every actual metadata/lint choice must consume its complete frame");
    }
    let mut dependency_library = library.clone();
    dependency_library.external = Some(transport_protocol::ExternMetadata {
        alias: "wire_dep".into(), artifact: PathBuf::from("/native-transport/deps-one/libwire_dep.rmeta"),
    });
    req.session.library = Some(dependency_library.clone());
    let frame = transport_protocol::encode(&req);
    let mut reader = Cursor::new(&frame);
    read_public_transport_request(&mut reader, identity, &req.file, &req.out, true, Some((&dependency_library, req.library_options))).unwrap();
    assert_eq!(reader.position() as usize, frame.len());
    dependency_library.external.as_mut().unwrap().artifact.set_file_name("another.rmeta");
    assert!(read_public_transport_request(&mut Cursor::new(&frame), identity, &req.file, &req.out, true, Some((&dependency_library, req.library_options))).is_err());

    let mut executable = library;
    executable.executable_dependencies = Some(vec![transport_protocol::ExecutableExtern {
        metadata: transport_protocol::ExternMetadata {
            alias: "wire_dep".into(),
            artifact: PathBuf::from("/native-transport/deps-one/libwire_dep.rmeta"),
        },
        archive: Some(PathBuf::from("/native-transport/deps-one/libwire_dep.rlib")),
    }]);
    req.session.krate.target_kind = "bin".into();
    req.session.library = Some(executable.clone());
    req.library_options.embed_metadata = None;
    let frame = transport_protocol::encode(&req);
    let mut reader = Cursor::new(&frame);
    read_public_transport_request(&mut reader, identity, &req.file, &req.out, true, Some((&executable, req.library_options))).unwrap();
    assert_eq!(reader.position() as usize, frame.len());
    executable.executable_dependencies.as_mut().unwrap()[0].archive.as_mut().unwrap().set_file_name("another.rlib");
    assert!(read_public_transport_request(&mut Cursor::new(&frame), identity, &req.file, &req.out, true, Some((&executable, req.library_options))).is_err());
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
                read_public_transport_request(&mut BufReader::new(&stream), &identity, &expected_source, &expected_output, false, None)
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

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, Clang and ELF tools"]
fn ordinary_cargo_links_rust_debug_information() {
    let f = Fixture::new("rust-debug");
    f.source("Cargo.toml", "[package]\nname = \"debug-leaf\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("src/main.rs", "fn total(input: u32) -> u32 {\n    let sum = input + 5;\n    sum + 1\n}\nfn main() { println!(\"{}\", total(7)); }\n");
    f.succeeds("check", f.command("cargo").args(["check", "--offline"]).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    f.succeeds("build", f.command("cargo").args(["build", "--offline"]).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    let ran = f.succeeds("run", f.command("cargo").args(["run", "--offline", "--quiet"]).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    assert_eq!(ran.stdout, b"13\n");
    let executable = f.root.join("target/debug/debug-leaf");
    let dwarf = f.succeeds("dwarf", Command::new("readelf").arg("--debug-dump=info").arg(&executable));
    let dwarf = String::from_utf8_lossy(&dwarf.stdout);
    for required in ["(Rust)", "): input", "): sum", "): u32", "DW_TAG_lexical_block"] {
        assert!(dwarf.contains(required), "missing Rust DWARF evidence {required}: {dwarf}");
    }
    let good = std::fs::read(&executable).unwrap();
    f.source("src/main.rs", "fn main() { let _: u32 = \"bad\"; }\n");
    let invalid = f.invoke("invalid", f.command("cargo").args(["build", "--offline"]).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    assert_eq!(invalid.status.code(), Some(101));
    assert_eq!(std::fs::read(executable).unwrap(), good, "a rejected edit replaced the last good executable");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE, Clang and ELF tools"]
fn default_cargo_links_debug_from_a_native_no_std_provider() {
    assert_linked_provider_workspace("debug-workspace", "#![no_std]\nfn shift(input: u32) -> u32 { input.wrapping_mul(1_073_741_824) ^ 15 }\n#[inline(never)]\npub fn evaluate(input: u32) -> u32 { shift(input) ^ 10 }\n");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE, Clang and ELF tools"]
fn default_cargo_links_debug_from_a_native_standard_provider() {
    assert_linked_provider_workspace("debug-std-workspace", "fn shift(input: u32) -> u32 { let changed = input ^ 3; changed }\npub fn evaluate(input: u32) -> u32 { let result = shift(input) ^ 10; result }\n");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn cargo_tests_a_no_std_library_with_the_core_prelude() {
    let f = Fixture::new("no-std-library-tests");
    f.source("Cargo.toml", "[package]\nname = \"no-std-tests\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    let source = r#"#![no_std]

pub fn transform(value: u32) -> u32 { value ^ 3 }

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::transform;

    #[test]
    fn exercises_core_prelude() {
        let value: Option<u32> = Some(transform(17));
        assert_eq!(value, Some(18));
        std::println!("core prelude result");
    }
}
"#;
    f.source("src/lib.rs", source);
    let build = ["test", "--offline", "--workspace", "--no-run"];
    let native = || f.command("cargo");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let original = || {
        let mut command = f.command("cargo");
        command.env("RUSTC", &oracle).env("CARGO_TARGET_DIR", f.root.join("original-target"));
        command
    };
    f.succeeds("native-compile", native().args(build));
    f.succeeds("original-compile", original().args(build));
    let run = ["test", "--offline", "--workspace", "--lib", "--", "--nocapture"];
    for (label, mut command) in [("native-runtime", native()), ("original-runtime", original())] {
        let out = f.succeeds(label, command.args(run));
        let output = String::from_utf8_lossy(&out.stdout);
        assert!(output.contains("core prelude result"));
        assert!(output.contains("1 passed; 0 failed"));
    }
    let cases = [
        ("disabled-std", source.replace("#[cfg(test)]\nextern crate std;", "#[cfg(any())]\npub extern crate std;"), "std"),
        ("unqualified-macro", source.replace("std::println!", "println!"), "println"),
        ("unqualified-type", source.replace("let value: Option<u32>", "let value: Vec<u32>"), "Vec"),
    ];
    for (name, invalid, missing) in cases {
        f.source("src/lib.rs", &invalid);
        for (compiler, mut command) in [("native", native()), ("original", original())] {
            let out = f.invoke(&format!("{compiler}-{name}"), command.args(build));
            assert_eq!(out.status.code(), Some(101));
            assert!(String::from_utf8_lossy(&out.stderr).contains(missing));
        }
    }
    f.source("src/lib.rs", source);
    f.succeeds("native-repair", native().args(build));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn ordinary_cargo_checks_and_links_associated_lifetimes_across_native_crates() {
    assert_associated_lifetime_workspace("associated-lifetime-workspace", "", "*input", b"17\n");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn ordinary_cargo_links_code_from_the_same_crate_as_an_associated_type() {
    let code = "\nfn mask(value: u32) -> u32 { value ^ 3 }\npub fn transform(value: u32) -> u32 { mask(value) ^ 8 }\n";
    assert_associated_lifetime_workspace("associated-lifetime-code-workspace", code, "api::transform(*input)", b"26\n");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn ordinary_cargo_preserves_unsized_referents_of_associated_types() {
    let f = Fixture::new("unsized-association");
    f.source("Cargo.toml", "[package]\nname = \"unsized-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nschema = { path = \"schema\" }\n");
    f.source("schema/Cargo.toml", "[package]\nname = \"schema\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    let provider = |pointee| format!("#![no_std]\npub struct Token;\npub trait Access {{\n    type Borrowed<'a> where Self: 'a;\n}}\nimpl Access for Token {{\n    type Borrowed<'r> = &'r {pointee};\n}}\n");
    let app = |argument| format!("use schema::{{Access, Token}};\nfn measure(input: <Token as Access>::Borrowed<'_>) -> usize {{ input.len() }}\nfn main() {{ println!(\"{{}}\", measure({argument})); }}\n");
    let run = ["run", "--offline", "--quiet"];
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let original = || {
        let mut command = f.command("cargo");
        command.env("RUSTC", &oracle).env("CARGO_TARGET_DIR", f.root.join("original-target"));
        command
    };

    f.source("schema/src/lib.rs", &provider("[u8]"));
    f.source("src/main.rs", &app("&[1u8, 2, 3]"));
    assert_eq!(f.succeeds("slice", f.command("cargo").args(run)).stdout, b"3\n");
    assert_eq!(f.succeeds("original-slice", original().args(run)).stdout, b"3\n");
    let binary = f.root.join("target/debug/unsized-app");
    let valid = std::fs::read(&binary).unwrap();

    f.source("schema/src/lib.rs", &provider("[u16]"));
    let failure = f.invoke("changed-slice-element", f.command("cargo").args(run));
    assert_eq!(failure.status.code(), Some(101));
    assert!(failure.stdout.is_empty());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("L0110"));
    assert_eq!(std::fs::read(&binary).unwrap(), valid);
    let failure = f.invoke("original-changed-slice-element", original().args(run));
    assert_eq!(failure.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&failure.stderr).contains("E0308"));

    f.source("schema/src/lib.rs", &provider("str"));
    f.source("src/main.rs", &app("\"hello\""));
    assert_eq!(f.succeeds("string", f.command("cargo").args(run)).stdout, b"5\n");
    assert_eq!(f.succeeds("original-string", original().args(run)).stdout, b"5\n");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
fn assert_associated_lifetime_workspace(scope: &str, code: &str, body: &str, expected: &[u8]) {
    let f = Fixture::new(scope);
    f.source("Cargo.toml", "[workspace]\nmembers = [\"provider\", \"application\"]\nresolver = \"3\"\n");
    f.source("provider/Cargo.toml", "[package]\nname = \"typed-schema\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    let provider = ["#![no_std]\npub struct Token;\npub trait Access {\n    type Borrowed<'a> where Self: 'a;\n}\nimpl Access for Token {\n    type Borrowed<'r> = &'r u32;\n}\n", code].concat();
    f.source("provider/src/lib.rs", &provider);
    f.source("application/Cargo.toml", "[package]\nname = \"projection-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\napi = { package = \"typed-schema\", path = \"../provider\" }\n");
    let source = f.source("application/src/main.rs", &format!(
        "use api::{{Access, Token}};\nfn value(input: <Token as Access>::Borrowed<'_>) -> u32 {{ {body} }}\nfn main() {{ println!(\"{{}}\", value(&17)); }}\n"
    ));
    let request = ["check", "--offline", "-p", "projection-app", "--message-format=json"];
    let check = |label| f.succeeds(label, f.command("cargo").args(request));
    let first = check("check");
    let metadata = first.stdout.split(|&byte| byte == b'\n').find_map(|line| {
        let record = serde_json::from_slice::<serde_json::Value>(line).ok()?;
        if record["reason"] != "compiler-artifact" || record["target"]["name"] != "typed_schema" {
            return None;
        }
        record["filenames"].as_array()?.iter().filter_map(|path| path.as_str())
            .find(|path| path.ends_with(".rmeta")).map(PathBuf::from)
    }).expect("native provider metadata artifact");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let original = |label| f.invoke(label, Command::new(&oracle).env_remove("LD_LIBRARY_PATH").current_dir(&f.root)
        .args(["--edition=2024", "--crate-name", "projection_reader", "--emit=metadata"])
        .arg(&source).arg("--extern").arg(format!("api={}", metadata.display()))
        .arg("-o").arg(f.root.join("independent-reader.rmeta")));
    assert!(original("original-reader").status.success());
    let changed_provider = provider.replace("&'r u32", "&'r u64");
    f.source("provider/src/lib.rs", &changed_provider);
    let changed = f.invoke("changed-assignment", f.command("cargo").args(request));
    assert_eq!(changed.status.code(), Some(101));
    assert!(changed.stdout.split(|&byte| byte == b'\n')
        .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
        .any(|record| record["reason"] == "compiler-message" && record["message"]["code"]["code"] == "L0110"));
    let changed = original("original-reader-changed");
    assert_eq!(changed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&changed.stderr).contains("E0308"));
    f.source("provider/src/lib.rs", &provider);
    check("repair");
    assert!(original("original-reader-repair").status.success());

    let build = f.succeeds("build", f.command("cargo").args([
        "build", "--offline", "-p", "projection-app", "--message-format=json",
    ]));
    let artifacts = build.stdout.split(|&byte| byte == b'\n').find_map(|line| {
        let record = serde_json::from_slice::<serde_json::Value>(line).ok()?;
        if record["reason"] != "compiler-artifact" || record["target"]["name"] != "typed_schema" {
            return None;
        }
        Some(record["filenames"].as_array()?.iter().filter_map(|path| path.as_str())
            .map(PathBuf::from).collect::<Vec<_>>())
    }).expect("native provider link artifacts");
    let independent = f.root.join("independent-linked-reader");
    let mut link = Command::new(&oracle);
    link.env_remove("LD_LIBRARY_PATH").current_dir(&f.root)
        .args(["--edition=2024", "--crate-name", "projection_reader"])
        .arg(&source).arg("-o").arg(&independent);
    for extension in ["rlib", "rmeta"] {
        let artifact = artifacts.iter().find(|path| path.extension().is_some_and(|ext| ext == extension))
            .expect("matching native provider artifact");
        link.arg("--extern").arg(format!("api={}", artifact.display()));
    }
    f.succeeds("original-reader-link", &mut link);
    let ran = f.succeeds("original-reader-run", &mut Command::new(independent));
    assert_eq!(ran.stdout, expected);

    let run = ["run", "--offline", "--quiet", "-p", "projection-app"];
    assert_eq!(f.succeeds("run", f.command("cargo").args(run)).stdout, ran.stdout);
    let binary = f.root.join("target/debug/projection-app");
    let valid = std::fs::read(&binary).unwrap();
    f.source("provider/src/lib.rs", &changed_provider);
    let rejected = f.invoke("changed-assignment-link", f.command("cargo").args(run));
    assert_eq!(rejected.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("L0110"));
    assert!(rejected.stdout.is_empty());
    assert_eq!(std::fs::read(&binary).unwrap(), valid);
    f.source("provider/src/lib.rs", &provider);
    assert_eq!(f.succeeds("repaired-link", f.command("cargo").args(run)).stdout, ran.stdout);
}


#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
fn assert_linked_provider_workspace(scope: &str, library_source: &str) {
    let f = Fixture::new(scope);
    f.source("Cargo.toml", "[workspace]\nmembers = [\"provider\", \"application\"]\nresolver = \"3\"\n");
    f.source("provider/Cargo.toml", "[package]\nname = \"debug-provider\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("provider/src/lib.rs", library_source);
    f.source("application/Cargo.toml", "[package]\nname = \"debug-application\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\ndebug-provider = { path = \"../provider\" }\n");
    let source = f.source("application/src/main.rs", "fn main() { let answer = debug_provider::evaluate(12); println!(\"{answer}\"); }\n");
    f.succeeds("check", f.command("cargo").args(["check", "--offline", "-p", "debug-application"]).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    let build = f.succeeds("build", f.command("cargo").args(["build", "--offline", "-p", "debug-application", "--message-format=json"]).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    let mut artifacts = std::collections::BTreeMap::new();
    for line in build.stdout.split(|&byte| byte == b'\n') {
        let Ok(record) = serde_json::from_slice::<serde_json::Value>(line) else { continue };
        if record["reason"] == "compiler-artifact" && record["target"]["name"] == "debug_provider" {
            for path in record["filenames"].as_array().unwrap().iter().filter_map(|value| value.as_str()) {
                let path = PathBuf::from(path);
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    artifacts.insert(extension.to_owned(), path);
                }
            }
        }
    }
    let native = f.root.join("target/debug/debug-application");
    let run = ["run", "--offline", "--quiet", "-p", "debug-application"];
    let ran = f.succeeds("run", f.command("cargo").args(run).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    assert_eq!(ran.stdout, b"5\n");
    let dwarf = f.succeeds("dwarf", Command::new("readelf").arg("--debug-dump=info").arg(&native));
    let dwarf = String::from_utf8_lossy(&dwarf.stdout);
    for required in ["provider/src/lib.rs", "): evaluate", "): shift", "): input"] {
        assert!(dwarf.contains(required), "missing Rust library DWARF evidence {required}");
    }
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let independently_linked = f.root.join("independent-rust-consumer");
    let mut command = Command::new(oracle);
    command.env_remove("LD_LIBRARY_PATH").current_dir(&f.root).args(["--edition=2024", "--crate-name", "consumer"])
        .arg(source).arg("-o").arg(&independently_linked);
    for kind in ["rlib", "rmeta"] {
        command.arg("--extern").arg(format!("debug_provider={}", artifacts[kind].display()));
    }
    f.succeeds("independent-link", &mut command);
    assert_eq!(f.succeeds("independent-run", &mut Command::new(independently_linked)).stdout, ran.stdout);

    let edited = "#![no_std]\nfn shift(input: u32) -> u32 { input ^ 3 }\npub fn evaluate(input: u32) -> u32 { shift(input) ^ 11 }\n";
    f.source("provider/src/lib.rs", edited);
    let changed = f.succeeds("dependency-body", f.command("cargo").args(run).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    assert_eq!(changed.stdout, b"4\n");
    let valid = std::fs::read(&native).unwrap();
    f.source("provider/src/lib.rs", "#![no_std]\npub fn evaluate(input: u32, mask: u32) -> u32 { input ^ mask }\n");
    let rejected = f.invoke("dependency-signature", f.command("cargo").args(run).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    assert_eq!(rejected.status.code(), Some(101));
    assert!(rejected.stdout.is_empty(), "an invalid executable ran");
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("L0105"));
    assert_eq!(std::fs::read(&native).unwrap(), valid, "an incompatible dependency replaced the last valid executable");
    f.source("provider/src/lib.rs", edited);
    let repaired = f.succeeds("dependency-repair", f.command("cargo").args(run).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    assert_eq!(repaired.stdout, changed.stdout);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, Clang and ELF tools"]
fn default_cargo_debug_records_native_enum_variants() {
    let f = Fixture::new("debug-enum");
    f.source("Cargo.toml", "[package]\nname = \"debug-enum\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("src/main.rs", r#"
#[repr(i8)]
enum Event {
    Idle = -7,
    Value { code: u32, delta: i16 } = 3,
    Tuple(i64) = 11,
}
enum Link<'a> {
    End,
    Next(&'a Link<'a>),
}
fn observe(event: &Event, link: &Link<'_>) -> i64 {
    let selected = match *event {
        Event::Idle => -7,
        Event::Value { code, delta } => i64::from(code) + i64::from(delta),
        Event::Tuple(value) => value,
    };
    selected + match link { Link::End => 0, Link::Next(_) => 1 }
}
fn main() {
    let end = Link::End;
    let next = Link::Next(&end);
    println!("{}", observe(&Event::Idle, &end));
    println!("{}", observe(&Event::Value { code: 43, delta: -2 }, &next));
    println!("{}", observe(&Event::Tuple(11), &end));
}
"#);
    let ran = f.succeeds("run", f.command("cargo").args(["run", "--offline", "--quiet"]).env_remove("CARGO_PROFILE_DEV_DEBUG"));
    assert_eq!(ran.stdout, b"-7\n42\n11\n");
    let executable = f.root.join("target/debug/debug-enum");
    let dwarf = f.succeeds("dwarf", Command::new("readelf").arg("--debug-dump=info").arg(executable));
    let dwarf = String::from_utf8_lossy(&dwarf.stdout);
    for required in ["DW_TAG_variant_part", "): Idle", "): Value", "): Tuple", "): code", "): delta", "): Next"] {
        assert!(dwarf.contains(required), "missing Rust DWARF evidence {required}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn ordinary_cargo_links_a_transitive_native_library() {
    let f = Fixture::new("transitive-native-link");
    f.source("Cargo.toml", "[workspace]\nmembers = [\"provider\", \"middle\", \"application\"]\nresolver = \"3\"\n");
    f.source("provider/Cargo.toml", "[package]\nname = \"value-provider\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("provider/src/lib.rs", "#![no_std]\npub fn encode(value: u32) -> u32 {\n    let offset = value ^ 3;\n    offset ^ 10\n}\n");
    f.source("middle/Cargo.toml", "[package]\nname = \"value-mapping\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nupstream_value = { package = \"value-provider\", path = \"../provider\" }\n");
    f.source("middle/src/lib.rs", "#![no_std]\npub fn map(value: u32) -> u32 { upstream_value::encode(value) ^ 6 }\n");
    f.source("application/Cargo.toml", "[package]\nname = \"value-display\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nvalue-mapping = { path = \"../middle\" }\n");
    let application = "fn main() { let mapped = value_mapping::map(12); println!(\"{mapped}\"); }\n";
    f.source("application/src/main.rs", application);

    f.succeeds("check", f.command("cargo").args(["check", "--offline", "--workspace"]));
    let run = ["run", "--offline", "--quiet", "-p", "value-display"];
    let cold = f.succeeds("run", f.command("cargo").args(run));
    assert_eq!(cold.stdout, b"3\n");
    let binary = f.root.join("target/debug/value-display");
    let initial = std::fs::read(&binary).unwrap();

    f.source("application/src/main.rs", "fn main() { let mapped = value_provider::encode(12); println!(\"{mapped}\"); }\n");
    let rejected = f.invoke("hidden-extern", f.command("cargo").args(run));
    assert_eq!(rejected.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("value_provider"));
    assert!(rejected.stdout.is_empty());
    assert_eq!(std::fs::read(&binary).unwrap(), initial);

    f.source("application/src/main.rs", application);
    f.source("provider/src/lib.rs", "#![no_std]\npub fn encode(value: u32) -> u32 {\n    let offset = value ^ 3;\n    offset ^ 11\n}\n");
    let edited = f.succeeds("transitive-edit", f.command("cargo").args(run));
    assert_eq!(edited.stdout, b"2\n");

    // BitOr binds less tightly than BitXor, so removing the parentheses
    // changes the result and would make the application print 11.
    f.source("provider/src/lib.rs", "#![no_std]\npub fn encode(value: u32) -> u32 { (value | 3) ^ 10 }\n");
    let grouped = f.succeeds("transitive-grouped-operand", f.command("cargo").args(run));
    assert_eq!(grouped.stdout, b"3\n");
    let optimized = f.succeeds("transitive-optimized", f.command("cargo").args(run).arg("--release"));
    assert_eq!(optimized.stdout, grouped.stdout);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn ordinary_cargo_uses_the_outer_lock_for_path_dependencies() {
    let f = Fixture::new("cargo-outer-lock");
    f.source("Cargo.toml", r#"
[package]
name = "outer-lock"
version = "0.1.0"
edition = "2024"

[dependencies]
locked-provider = { path = "provider" }
"#);
    f.source("provider/Cargo.toml", r#"
[package]
name = "locked-provider"
version = "0.1.0"
edition = "2024"
"#);
    f.source("provider/src/lib.rs", "#![no_std]\npub fn value(input: u32) -> u32 { input ^ 15 }\n");
    f.source("src/main.rs", "fn main() { let value = locked_provider::value(12); println!(\"{value}\"); }\n");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let lock = f.root.join("Cargo.lock");
    let initial = std::fs::read(&lock).unwrap();
    assert!(!f.root.join("provider/Cargo.lock").exists());

    f.succeeds("frozen-check", f.command("cargo").args(["check", "--frozen"]));
    let built = f.succeeds("frozen-run", f.command("cargo").args(["run", "--frozen", "--quiet"]));
    assert_eq!(built.stdout, b"3\n");
    assert_eq!(std::fs::read(lock).unwrap(), initial);
    assert!(!f.root.join("provider/Cargo.lock").exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn ordinary_cargo_uses_generated_build_script_inputs() {
    let f = Fixture::new("cargo-generated-input");
    f.source("Cargo.toml", "[package]\nname = \"generated-value\"\nedition = \"2024\"\nversion = \"0.1.0\"\n");
    f.source("build.rs", r#"
fn main() {
    let output = std::env::var("OUT_DIR").unwrap();
    let source = std::fs::read_to_string("value.txt").unwrap();
    let destination = format!("{output}/generated.rs");
    std::fs::write(destination, source).unwrap();
    println!("cargo::rerun-if-changed=value.txt");
    println!("cargo::rustc-env=BUILD_LABEL=generated");
}
"#);
    f.source("src/main.rs", r#"
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

fn main() {
    let label = env!("BUILD_LABEL");
    let value = generated::value();
    println!("{label}:{value}");
}
"#);
    let initial = "pub fn value() -> u32 { 5 }\n";
    f.source("value.txt", initial);
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    f.succeeds("check", f.command("cargo").args(["check", "--frozen"]));
    let run = ["run", "--frozen", "--quiet"];
    let cold = f.succeeds("run", f.command("cargo").args(run));
    assert_eq!(cold.stdout, b"generated:5\n");
    let unchanged = f.succeeds("unchanged", f.command("cargo").args(run));
    assert_eq!(unchanged.stdout, cold.stdout);

    f.source("value.txt", "pub fn value() -> u32 { 7 }\n");
    let edited = f.succeeds("edit", f.command("cargo").args(run));
    assert_eq!(edited.stdout, b"generated:7\n");
    let output = f.root.join("target/debug/generated-value");
    let linked = std::fs::read(&output).unwrap();
    f.source("value.txt", "pub fn value() -> u32 { \"invalid\" }\n");
    let invalid = f.invoke("invalid", f.command("cargo").args(run));
    assert_eq!(invalid.status.code(), Some(101));
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("generated.rs"));
    assert_eq!(std::fs::read(output).unwrap(), linked);

    f.source("value.txt", initial);
    let repaired = f.succeeds("repair", f.command("cargo").args(run));
    assert_eq!(repaired.stdout, cold.stdout);
    let dep_info = std::fs::read_to_string(f.root.join("target/debug/generated-value.d")).unwrap();
    assert!(dep_info.contains("value.txt") && dep_info.contains("generated.rs"));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and Clang"]
fn ordinary_cargo_preserves_build_script_cfg_across_edits() {
    let f = Fixture::new("cargo-build-cfg");
    f.source("Cargo.toml", "[package]\nname=\"build-cfg\"\nedition=\"2024\"\nversion=\"0.1.0\"\n");
    f.source("build.rs", r#"
fn main() {
    println!("cargo::rerun-if-changed=configuration.txt");
    println!("cargo::rustc-check-cfg=cfg(build_enabled)");
    println!("cargo::rustc-check-cfg=cfg(declared_but_unset)");
    println!("cargo::rustc-check-cfg=cfg(build_mode, values(\"fast\", \"safe\", \"invalid\"))");
    let flags = std::fs::read_to_string("configuration.txt").unwrap();
    print!("{flags}");
}
"#);
    f.source("src/main.rs", r#"
#[cfg(all(build_enabled, build_mode = "fast", build_mode = "safe"))]
fn selected() -> u32 { 41 }
#[cfg(not(all(build_enabled, build_mode = "fast", build_mode = "safe")))]
fn selected() -> u32 { 42 }
#[cfg(build_mode = "invalid")]
fn invalid() -> u32 { "invalid body must be rejected" }
#[cfg(declared_but_unset)]
fn not_enabled_by_declaration() -> u32 { "must remain disabled" }
fn main() {
    let mode = cfg!(build_mode = "fast");
    let unset = cfg!(declared_but_unset);
    let value = selected();
    println!("{value}:{mode}:{unset}");
}
"#);
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let target = |original: bool| f.root.join(if original { "original-target" } else { "target" });
    let command = |original: bool| {
        let mut command = f.command("cargo");
        command.env("CARGO_TARGET_DIR", target(original));
        if original { command.env("RUSTC", &oracle); }
        command
    };
    f.succeeds("lockfile", command(false).args(["generate-lockfile", "--offline"]));
    let initial = "cargo::rustc-cfg=build_enabled\ncargo::rustc-cfg=build_mode=\"fast\"\ncargo::rustc-cfg=build_mode=\"safe\"\n";
    for (state, flags, expected) in [
        ("cold", initial, Some(b"41:true:false\n".as_slice())),
        ("edited", "cargo::rustc-cfg=build_mode=\"safe\"\n", Some(b"42:false:false\n".as_slice())),
        ("invalid", "cargo::rustc-cfg=build_mode=\"invalid\"\n", None),
        ("repaired", initial, Some(b"41:true:false\n".as_slice())),
    ] {
        f.source("configuration.txt", flags);
        for (compiler, original) in [("native", false), ("original", true)] {
            let name = format!("{compiler}-{state}");
            let mut command = command(original);
            command.args(["run", "--quiet", "--frozen"]);
            if let Some(expected) = expected {
                assert_eq!(f.succeeds(&name, &mut command).stdout, expected);
            } else {
                let executable = target(original).join("debug/build-cfg");
                let before = std::fs::read(&executable).unwrap();
                let result = f.invoke(&name, &mut command);
                assert_eq!(result.status.code(), Some(101));
                assert!(result.stdout.is_empty());
                assert!(String::from_utf8_lossy(&result.stderr).contains("invalid body must be rejected"));
                assert_eq!(std::fs::read(executable).unwrap(), before);
            }
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and Clang"]
fn ordinary_cargo_uses_the_same_cfg_flags_for_discovery_and_compilation() {
    let f = Fixture::new("cargo-cfg-discovery");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let flags = ["--cfg", "from_cargo_flags", "--cfg=chosen_mode=\"fast\"",
        "--check-cfg", "cfg(from_cargo_flags)", "--check-cfg=cfg(chosen_mode, values(\"fast\"))"];
    for (name, mut compiler) in [("native", f.command("rustc")), ("original", Command::new(&oracle))] {
        let facts = f.succeeds(&format!("{name}-query"), compiler.current_dir(&f.root)
            .env_remove("LD_LIBRARY_PATH").args(["--print", "cfg"]).args(flags)).stdout;
        let facts = std::str::from_utf8(&facts).unwrap();
        assert!(facts.lines().any(|line| line == "from_cargo_flags"));
        assert!(facts.lines().any(|line| line == "chosen_mode=\"fast\""));
    }
    for (name, mut compiler) in [("native", f.command("rustc")), ("original", Command::new(&oracle))] {
        let output = f.invoke(&format!("{name}-invalid-query"), compiler.current_dir(&f.root)
            .env_remove("LD_LIBRARY_PATH").args(["--print", "sysroot", "--print", "cfg", "--cfg", "chosen_mode=bare"]));
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    f.source("Cargo.toml", "[package]\nname=\"cfg-discovery\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    f.source("src/main.rs", r#"
#[cfg(all(from_cargo_flags, chosen_mode = "fast"))]
fn main() { println!("effective cfg"); }
#[cfg(not(all(from_cargo_flags, chosen_mode = "fast")))]
fn main() { let _: i32 = "active cfg flags were lost"; }
"#);
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    for (name, original) in [("native", false), ("original", true)] {
        let mut cargo = f.command("cargo");
        cargo.args(["run", "--quiet", "--frozen"])
            .env("CARGO_ENCODED_RUSTFLAGS", flags.join("\u{1f}"));
        if original {
            cargo.env("RUSTC", &oracle).env("CARGO_TARGET_DIR", f.root.join("original-target"));
        }
        assert_eq!(f.succeeds(&format!("{name}-run"), &mut cargo).stdout, b"effective cfg\n");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn cargo_discovery_accepts_freestanding_link_settings() {
    let f = Fixture::new("cargo-freestanding-discovery");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let query = ["-", "--crate-name", "discovery", "--print=file-names", "--print=crate-name", "--print=cfg"];
    let link = ["-C", "link-arg=-nostdlib", "-Clink-arg=-static", "--codegen=relocation-model=static"];
    for (name, original) in [("native", false), ("original", true)] {
        let command = || {
            let mut compiler = if original { Command::new(&oracle) } else { f.command("rustc") };
            compiler.current_dir(&f.root).env_remove("LD_LIBRARY_PATH").args(query);
            compiler
        };
        let plain = f.succeeds(&format!("{name}-query"), &mut command());
        let configured = f.succeeds(&format!("{name}-with-link-settings"), command().args(link));
        assert_eq!(configured.stdout, plain.stdout);
        let invalid = f.invoke(&format!("{name}-invalid-relocation"), command().arg("-Crelocation-model=invalid"));
        assert!(!invalid.status.success());
        assert!(invalid.stdout.is_empty());
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE, Clang and binutils"]
fn cargo_freestanding_executable_combines_native_objects_and_rust() {
    let f = Fixture::new("cargo-freestanding-mixed");
    f.source("Cargo.toml", "[package]\nname=\"freestanding\"\nversion=\"0.1.0\"\nedition=\"2024\"\n\n[profile.dev]\npanic=\"abort\"\n");
    let source = r#"#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

fn apply_mask(n: u64) -> u64 { n ^ 19 }
fn select_bits(n: u64) -> u64 { n & 63 }

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let transform = core::hint::black_box(apply_mask as fn(u64) -> u64);
    let status = select_bits(transform(57));
    unsafe {
        asm!("syscall", in("rax") 60_u64, in("rdi") status, options(noreturn));
    }
}


#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop { core::hint::spin_loop(); }
}
"#;
    let original_source = f.source("src/main.rs", source);
    let rust = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let flags = ["-Clink-arg=-nostdlib", "-Clink-arg=-static", "-Crelocation-model=static"];
    let cargo = |target: &str, original| {
        let mut command = f.command("cargo");
        command.env("CARGO_ENCODED_RUSTFLAGS", flags.join("\u{1f}"))
            .env("CARGO_PROFILE_DEV_DEBUG", "0")
            .env("CARGO_TARGET_DIR", f.root.join(target));
        if original {
            command.env("RUSTC", &rust);
        }
        command
    };
    f.succeeds("lockfile", cargo("mixed", false).args(["generate-lockfile", "--offline"]));
    f.succeeds("check", cargo("mixed", false).args(["check", "--frozen"]));
    for (name, original) in [("mixed", false), ("original", true)] {
        f.succeeds(&format!("{name}-build"), cargo(name, original).args(["build", "--frozen"]));
        let binary = f.root.join(name).join("debug/freestanding");
        let out = f.invoke(&format!("{name}-run"), &mut Command::new(&binary));
        assert_eq!(out.status.code(), Some(42));
        let elf = f.succeeds(&format!("{name}-elf"), Command::new("readelf").args(["-l", "-d"]).arg(&binary));
        let elf = String::from_utf8(elf.stdout).unwrap();
        assert!(!elf.contains("INTERP") && !elf.contains("(NEEDED)"), "{elf}");
        let symbols = f.succeeds(&format!("{name}-symbols"), Command::new("nm").arg(&binary));
        let symbols = String::from_utf8(symbols.stdout).unwrap();
        let native = symbols.lines().filter(|line| line.split_whitespace().last()
            .and_then(|name| name.strip_prefix("f_")).is_some_and(|suffix| {
                suffix.len() == 16 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
            })).count();
        assert_eq!(native, if original { 0 } else { 2 }, "{symbols}");
    }
    assert_eq!(std::fs::read_to_string(&original_source).unwrap(), source);
    for (edit, invalid, diagnostic) in [
        ("native-error", source.replace("n ^ 19", "n ^ false"), "src/main.rs:6:"),
        ("rust-error", source.replace("let status =", "let status: u32 ="), "E0308"),
    ] {
        f.source("src/main.rs", &invalid);
        let response = f.invoke(edit, cargo(edit, false).args(["build", "--frozen"]));
        assert_eq!(response.status.code(), Some(101));
        let stderr = String::from_utf8(response.stderr).unwrap();
        assert!(stderr.contains(diagnostic), "{stderr}");
        assert!(!f.root.join(edit).join("debug/freestanding").exists());
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and binutils"]
fn cargo_static_library_combines_native_objects_with_rust_async_code() {
    let f = Fixture::new("cargo-mixed-static-library");
    f.source("Cargo.toml", "[package]\nname=\"entrypoints\"\nedition=\"2024\"\nversion=\"0.1.0\"\n\n[lib]\ncrate-type=[\"staticlib\"]\n");
    let source = r#"
use std::{future::Future, pin::pin, task::{Context, Poll, Waker}};

fn apply_mask(input: u64) -> u64 { input ^ 19 }

async fn compute(input: u64) -> u64 {
    let apply = std::hint::black_box(apply_mask as fn(u64) -> u64);
    let value = std::future::ready(input).await;
    apply(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn evaluate(input: u64) -> u64 {
    let mut task = pin!(compute(input));
    let mut context = Context::from_waker(Waker::noop());
    match task.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => unreachable!(),
    }
}
"#;
    let lib = f.source("src/lib.rs", source);
    let consumer = f.source("consumer.rs", r#"
#[link(name = "entrypoints", kind = "static")]
unsafe extern "C" {
    safe fn evaluate(input: u64) -> u64;
}
fn main() { assert_eq!(evaluate(57), 42); }
"#);
    let rust = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let cargo = |name: &str, original: bool, debug: &str| {
        let mut command = f.command("cargo");
        command.env("CARGO_TARGET_DIR", f.root.join(name)).env("CARGO_PROFILE_DEV_DEBUG", debug);
        if original {
            command.env("RUSTC", &rust);
        }
        command
    };
    f.succeeds("lockfile", cargo("mixed", false, "0").args(["generate-lockfile", "--offline"]));
    f.succeeds("default-check", cargo("default", false, "2").args(["check", "--frozen"]));
    for (name, original, debug, expected) in [
        ("mixed", false, "0", 1), ("original", true, "0", 0), ("default", false, "2", 0),
    ] {
        f.succeeds(&format!("{name}-build"), cargo(name, original, debug).args(["build", "--frozen"]));
        let dir = f.root.join(name).join("debug");
        let lib = dir.join("libentrypoints.a");
        let symbols = f.succeeds(&format!("{name}-symbols"), Command::new("nm").arg("--defined-only").arg(&lib));
        let symbols = String::from_utf8(symbols.stdout).unwrap();
        let native = symbols.lines().filter(|line| line.split_whitespace().last()
            .and_then(|name| name.strip_prefix("f_")).is_some_and(|suffix| {
                suffix.len() == 16 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
            })).count();
        assert_eq!(native, expected);
        let exe = f.root.join(format!("{name}-consumer"));
        f.succeeds(&format!("{name}-consumer"), Command::new(&rust)
            .current_dir(&f.root).env_remove("LD_LIBRARY_PATH").arg("--edition=2024")
            .arg(&consumer).arg("-L").arg(format!("native={}", dir.display())).arg("-o").arg(&exe));
        f.succeeds(&format!("{name}-run"), &mut Command::new(exe));
    }
    assert_eq!(std::fs::read_to_string(&lib).unwrap(), source);
    for (name, invalid, diagnostic) in [
        ("native-error", source.replace("input ^ 19", "input ^ false"), "src/lib.rs:4:"),
        ("rust-error", source.replace("-> u64 {\n    let apply", "-> u32 {\n    let apply"), "E0308"),
    ] {
        f.source("src/lib.rs", &invalid);
        let result = f.invoke(name, cargo(name, false, "0").args(["build", "--frozen"]));
        assert_eq!(result.status.code(), Some(101));
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(stderr.contains(diagnostic), "{stderr}");
        assert!(!f.root.join(name).join("debug/libentrypoints.a").exists());
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and Clang"]
fn rustc_consumes_a_rust_archive_with_embedded_metadata() {
    let f = Fixture::new("embedded-extern");
    let manifest = f.source("Cargo.toml", "[package]\nname=\"application\"\nedition=\"2024\"\nversion=\"0.1.0\"\n");
    f.source("provider.rs", "#![no_std]\npub fn doubled(n: u64) -> u64 { n.wrapping_mul(2) }\n");
    f.source("src/main.rs", r#"
fn main() {
    let answer = provider_alias::doubled(21);
    println!("{answer}");
}
"#);
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let oracle_command = || {
        let mut command = Command::new(&oracle);
        command.current_dir(&f.root).env_remove("LD_LIBRARY_PATH");
        command
    };
    let archive = f.root.join("libprovider.rlib");
    let provider = |embedding: &str| {
        let mut command = oracle_command();
        command.args(["--crate-name", "provider", "--crate-type=rlib", "--edition=2024", "-Cembed-bitcode=no"])
            .arg(format!("-Zembed-metadata={embedding}")).args(["--emit=link", "provider.rs", "-o"]).arg(&archive);
        command
    };
    f.succeeds("original-provider", &mut provider("yes"));
    assert!(!f.root.join("libprovider.rmeta").exists());

    let compiler = |original: bool, out: &Path| {
        let mut command = if original { oracle_command() } else { f.command("rustc") };
        command.env("CARGO_MANIFEST_PATH", &manifest).env("CARGO_CRATE_NAME", "application").env("CARGO_BIN_NAME", "application");
        command.args(["--crate-name", "application", "--crate-type=bin", "--edition=2024",
            "-Cmetadata=embedded", "-Cembed-bitcode=no", "--emit=dep-info,metadata", "--out-dir"]).arg(out)
            .arg("--extern").arg(format!("provider_alias={}", archive.display())).arg("src/main.rs");
        command
    };
    let target = |original| f.root.join(if original { "original-target" } else { "target" });
    let cargo = |original| {
        let mut command = f.command("cargo");
        command.env("CARGO_TARGET_DIR", target(original));
        if original { command.env("RUSTC", &oracle); }
        command.args(["rustc", "--quiet", "--frozen", "--", "--extern"])
            .arg(format!("provider_alias={}", archive.display()));
        command
    };
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    for (name, original) in [("native", false), ("original", true)] {
        let out = f.root.join(name);
        std::fs::create_dir(&out).unwrap();
        f.succeeds(&format!("{name}-check"), &mut compiler(original, &out));
        assert!(out.join("libapplication.rmeta").is_file());
        f.succeeds(&format!("{name}-link"), &mut cargo(original));
        let ran = f.succeeds(&format!("{name}-run"), &mut Command::new(target(original).join("debug/application")));
        assert_eq!(ran.stdout, b"42\n");
        assert!(!f.root.join("libprovider.rmeta").exists());
    }

    let plain = f.source("src/plain-tool.rs", r#"
fn main() {
    let answer = provider_alias::doubled(21);
    println!("{}:{answer}", module_path!());
}
"#);
    for (label, mut command) in [("native", f.command("rustc")), ("original", oracle_command())] {
        let directory = f.root.join(format!("{label} direct"));
        std::fs::create_dir(&directory).unwrap();
        let out = directory.join("tool with spaces");
        command.env("CARGO_MANIFEST_PATH", &manifest)
            .args(["--edition=2024", "--color=never", "src/plain-tool.rs", "--extern"])
            .arg(format!("provider_alias={}", archive.display())).arg("-o").arg(&out);
        f.succeeds(&format!("{label}-direct"), &mut command);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        assert_eq!(f.succeeds(&format!("{label}-direct-run"), &mut Command::new(out)).stdout, b"plain_tool:42\n");
    }
    let plain_bytes = std::fs::read(&plain).unwrap();
    let collision = f.invoke("native-direct-output-alias", f.command("rustc")
        .args(["--edition=2024", "src/plain-tool.rs", "--extern"])
        .arg(format!("provider_alias={}", archive.display())).args(["-o", "src/../src/plain-tool.rs"]));
    assert!(!collision.status.success());
    assert_eq!(std::fs::read(&plain).unwrap(), plain_bytes);

    let stdin = f.source("stdin.txt", r#"
fn main() {
    let answer = provider_alias::doubled(21);
    println!("{}:{}:{answer}", file!(), module_path!());
}
"#);
    let document = f.source("src/doc-source.rs", "// Rustdoc supplies the generated bytes over stdin.\n");
    for (label, original) in [("native", false), ("original", true)] {
        for (source, path) in [("anonymous", None), ("document", Some(&document))] {
            let out = f.root.join(format!("{label}-{source}-stdin"));
            let mut command = if original { oracle_command() } else { f.command("rustc") };
            command.args(["--edition=2024", "--color=never", "-"])
                .arg("--extern").arg(format!("provider_alias={}", archive.display())).arg("-o").arg(&out)
                .stdin(std::process::Stdio::from(std::fs::File::open(&stdin).unwrap()));
            if let Some(path) = path {
                command.env("UNSTABLE_RUSTDOC_TEST_PATH", path).env("UNSTABLE_RUSTDOC_TEST_LINE", "24");
            }
            f.succeeds(&format!("{label}-{source}-stdin-link"), &mut command);
            let display = path.map_or_else(|| "<anon>".into(), |path| path.to_string_lossy());
            let expected = format!("{display}:rust_out:42\n");
            assert_eq!(f.succeeds(&format!("{label}-{source}-stdin-run"), &mut Command::new(out)).stdout, expected.as_bytes());
        }
    }
    assert!(!f.root.join("<anon>").exists());

    let doctest = f.source("doc-example.rs", r#"
#![doc(test(no_crate_inject))]
/// An example linked with the independently compiled provider.
///
/// ```standalone_crate
/// assert_eq!(provider_alias::doubled(21), 42);
/// ```
///
/// ```no_run,standalone_crate
/// let _ = provider_alias::doubled(21);
/// panic!("a no_run example must not execute");
/// ```
pub fn example() {}
"#);
    let rustdoc = PathBuf::from(&oracle).with_file_name("rustdoc");
    let trace = f.root.join("rustdoc-builder-trace.jsonl");
    let result = f.succeeds("pinned-rustdoc-native-builder", Command::new(rustdoc).current_dir(&f.root)
        .env_remove("LD_LIBRARY_PATH").env_remove("RUSTC_BOOTSTRAP").env("LUSTR_TRACE_PATH", &trace)
        .args(["--test", "--edition=2024", "-Zunstable-options", "--test-builder"])
        .arg(f.package.join("bin/rustc")).arg("--extern")
        .arg(format!("provider_alias={}", archive.display())).arg(&doctest));
    assert!(String::from_utf8_lossy(&result.stdout).contains("2 passed"));
    let trace = std::fs::read_to_string(trace).unwrap();
    assert_eq!(trace.lines().filter(|line| *line == r#"{"engine":"lust","entry":"stdin"}"#).count(), 2);
    assert!(trace.lines().all(|line| !line.contains("integration-unimplemented")));

    let executable = target(false).join("debug/application");
    let before = std::fs::read(&executable).unwrap();
    std::fs::rename(&archive, f.root.join("original-provider.rlib")).unwrap();
    f.succeeds("original-provider-stub", &mut provider("no"));
    assert!(!f.root.join("libprovider.rmeta").exists());
    // Force a fresh compiler request through Cargo for the negative archives.
    let app = f.root.join("src/main.rs");
    let text = std::fs::read_to_string(&app).unwrap();
    std::fs::write(&app, format!("{text}\n// Stub archive check\n")).unwrap();
    for (label, original) in [("native", false), ("original", true)] {
        let output = f.invoke(&format!("{label}-missing-embedded-metadata"),
            &mut cargo(original));
        assert!(!output.status.success());
    }
    assert_eq!(std::fs::read(&executable).unwrap(), before);
    std::fs::rename(&archive, f.root.join("stub-provider.rlib")).unwrap();
    std::fs::write(&archive, b"invalid Rust archive").unwrap();
    let output = f.invoke("native-corrupt-archive", &mut cargo(false));
    assert!(!output.status.success());
    assert_eq!(std::fs::read(&executable).unwrap(), before);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and Clang"]
fn rustc_resolves_transitive_rust_archives_without_metadata_sidecars() {
    let f = Fixture::new("transitive-embedded-extern");
    let dependencies = f.root.join("deps");
    std::fs::create_dir(&dependencies).unwrap();
    f.source("leaf.rs", "#![no_std]\npub fn doubled(x: u64) -> u64 { x.wrapping_mul(2) }\n");
    f.source("provider.rs", "#![no_std]\npub fn run(x: u64) -> u64 { leaf::doubled(x) }\n");
    f.source("main.rs", "fn main() { let value = provider::run(21); println!(\"{value}\"); }\n");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let original = || {
        let mut command = Command::new(&oracle);
        command.current_dir(&f.root).env_remove("LD_LIBRARY_PATH");
        command
    };
    let library = |name: &str| {
        let mut command = original();
        command.args(["--crate-name", name, "--crate-type=rlib", "--edition=2024",
            "-Cembed-bitcode=no", "-Zembed-metadata=yes", "--emit=link", "-o"])
            .arg(dependencies.join(format!("lib{name}.rlib"))).arg(format!("{name}.rs"));
        command
    };
    let leaf = format!("leaf={}", dependencies.join("libleaf.rlib").display());
    let provider = format!("provider={}", dependencies.join("libprovider.rlib").display());
    f.succeeds("original-leaf", &mut library("leaf"));
    f.succeeds("original-provider", library("provider").args(["--extern", &leaf]));
    assert!(!dependencies.join("libprovider.rmeta").exists());
    assert!(!dependencies.join("libleaf.rmeta").exists());

    let executable = |upstream: bool, output: &Path| {
        let mut command = if upstream { original() } else { f.command("rustc") };
        command.args(["--edition=2024", "main.rs", "--extern", &provider, "-L"])
            .arg(format!("dependency={}", dependencies.display())).arg("-o").arg(output);
        command
    };
    for (label, upstream) in [("native", false), ("original", true)] {
        let output = f.root.join(label);
        f.succeeds(&format!("{label}-link"), &mut executable(upstream, &output));
        assert_eq!(f.succeeds(&format!("{label}-app"), &mut Command::new(output)).stdout, b"42\n");
    }

    f.source("leaf.rs", "#![no_std]\npub fn doubled(x: u64) -> u64 { x.wrapping_mul(3) }\n");
    f.succeeds("original-changed-leaf", &mut library("leaf"));
    for (label, upstream) in [("native", false), ("original", true)] {
        let output = f.root.join(label);
        let before = std::fs::read(&output).unwrap();
        let response = f.invoke(&format!("{label}-stale-leaf"), &mut executable(upstream, &output));
        assert!(!response.status.success());
        assert_eq!(std::fs::read(&output).unwrap(), before);
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and Clang"]
fn cargo_preserves_original_function_documentation_in_rust_metadata() {
    let f = Fixture::new("documented-library");
    f.source("Cargo.toml", "[package]\nname=\"documented\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    f.source("src/lib.rs", r#"
#![no_std]
/// Line documentation from original Rust contains π and 雪.
/** Block documentation from original Rust contains 🦀. */
pub fn flip(value: u64) -> u64 { value ^ 1 }
"#);
    f.source("src/main.rs", "fn main() { let value = documented::flip(42); println!(\"{value}\"); }\n");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let build = f.succeeds("native-build", f.command("cargo").args([
        "build", "--frozen", "--message-format=json",
    ]));
    let metadata_path = build.stdout.split(|&byte| byte == b'\n').find_map(|line| {
        let record = serde_json::from_slice::<serde_json::Value>(line).ok()?;
        if record["reason"] != "compiler-artifact" || record["target"]["name"] != "documented" {
            return None;
        }
        record["filenames"].as_array()?.iter().filter_map(|path| path.as_str())
            .find(|path| path.ends_with(".rmeta")).map(PathBuf::from)
    }).expect("actual documented library metadata");
    let app = f.succeeds("native-app", f.command("cargo").args(["run", "--frozen", "--quiet"]));
    assert_eq!(app.stdout, b"43\n");

    let exported = f.source("reexport.rs", "pub use documented::flip;\n");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let docs = f.root.join("rendered");
    f.succeeds("original-rustdoc-render", Command::new(PathBuf::from(oracle).with_file_name("rustdoc"))
        .current_dir(&f.root).env_remove("LD_LIBRARY_PATH")
        .args(["--edition=2024", "--crate-name", "reexporter", "--extern"])
        .arg(format!("documented={}", metadata_path.display())).arg("--out-dir").arg(&docs).arg(exported));
    let html = std::fs::read_to_string(docs.join("reexporter/fn.flip.html")).unwrap();
    assert!(html.contains("Line documentation from original Rust contains π and 雪."));
    assert!(html.contains("Block documentation from original Rust contains 🦀."));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and Clang"]
fn cargo_documentation_tests_use_native_link_and_check_requests() {
    let f = Fixture::new("cargo-standalone-doctests");
    f.source("Cargo.toml", "[package]\nname=\"doc_consumer\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    f.source("src/lib.rs", r#"
#![no_std]

/// Test the linked library.
///
/// ```standalone_crate
/// assert_eq!(doc_consumer::flip(42), 43);
/// ```
///
/// ```no_run,standalone_crate
/// let value = doc_consumer::flip(42);
/// panic!("no_run executed: {value}");
/// ```
pub fn flip(value: u64) -> u64 { value ^ 1 }
"#);
    let oracle = PathBuf::from(std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE"));
    let mut flags = std::ffi::OsString::from("-Zunstable-options\x1f--test-builder\x1f");
    flags.push(f.package.join("bin/rustc"));
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let installed = f.package.join("bin/rustdoc").is_file();
    let test_doc = || {
        let mut command = f.command("cargo");
        command.env_remove("RUSTDOCFLAGS").env_remove("CARGO_ENCODED_RUSTDOCFLAGS")
            .args(["test", "--doc", "--frozen"]);
        if !installed {
            command.env("RUSTDOC", oracle.with_file_name("rustdoc"))
                .env("CARGO_ENCODED_RUSTDOCFLAGS", &flags);
        }
        command
    };
    let run = f.succeeds("cargo-test-doc", &mut test_doc());
    let output = String::from_utf8(run.stdout).unwrap();
    assert!(output.contains("2 passed; 0 failed"), "{output}");
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert_eq!(trace.lines().filter(|line| {
        serde_json::from_str::<serde_json::Value>(line).is_ok_and(|record|
            record["engine"] == "lust" && record["entry"] == "stdin")
    }).count(), 2, "{trace}");

    let build = f.succeeds("cargo-artifacts", f.command("cargo").args([
        "build", "--frozen", "--message-format=json",
    ]));
    let metadata = build.stdout.split(|&byte| byte == b'\n').find_map(|line| {
        let record = serde_json::from_slice::<serde_json::Value>(line).ok()?;
        if record["reason"] != "compiler-artifact" || record["target"]["name"] != "doc_consumer" {
            return None;
        }
        record["filenames"].as_array()?.iter().filter_map(|path| path.as_str())
            .find(|path| path.ends_with(".rmeta")).map(PathBuf::from)
    }).expect("Cargo documentation library metadata");
    let archive = f.root.join("unused").join(metadata.file_name().unwrap()).with_extension("rlib");
    std::fs::create_dir(archive.parent().unwrap()).unwrap();
    std::fs::write(&archive, b"metadata-only requests do not read this archive").unwrap();
    let artifact = f.source("probe.rs", "fn main() { assert_eq!(doc_consumer::flip(42), 43); }\n");
    let request = |upstream: bool, metadata_only: bool, output: &Path| {
        let mut command = if upstream {
            let mut command = Command::new(&oracle);
            command.current_dir(&f.root).env_remove("LD_LIBRARY_PATH");
            command
        } else {
            f.command("rustc")
        };
        command.arg("--edition=2024");
        if metadata_only {
            command.arg("--emit=metadata");
        }
        command.arg("--extern").arg(format!("doc_consumer={}", metadata.display())).arg("--extern")
            .arg(format!("doc_consumer={}", archive.display())).arg(&artifact).arg("-o").arg(output);
        command
    };
    for (name, upstream) in [("native", false), ("original", true)] {
        let output = f.root.join(format!("{name}-checked"));
        f.succeeds(&format!("{name}-unused-archive"), &mut request(upstream, true, &output));
        assert_eq!(std::fs::metadata(output).unwrap().len(), 0);
    }
    std::fs::remove_file(&archive).unwrap();
    for (name, upstream) in [("native", false), ("original", true)] {
        let output = f.root.join(format!("{name}-missing"));
        let observed = f.invoke(&format!("{name}-missing-archive"), &mut request(upstream, true, &output));
        assert!(!observed.status.success());
        assert!(!output.exists());
    }

    // Use the same compiler session and paths when moving from a metadata-only
    // request to a linked executable, and then back to checking.
    std::fs::copy(metadata.with_extension("rlib"), &archive).unwrap();
    let metadata_only = f.root.join("checked");
    f.succeeds("native-check-before-link", &mut request(false, true, &metadata_only));
    let binary = f.root.join("linked");
    f.succeeds("native-link-after-check", &mut request(false, false, &binary));
    f.succeeds("native-run-after-check", &mut Command::new(binary));
    std::fs::write(&archive, b"metadata-only requests do not read this archive").unwrap();
    f.succeeds("native-check-after-link", &mut request(false, true, &metadata_only));

    if installed {
        let source = r#"
#![no_std]
/// ```compile_fail,standalone_crate
/// let _: u8 = doc_consumer::flip(42);
/// ```
pub fn flip(value: u64) -> u64 { value ^ 1 }
"#;
        f.source("src/lib.rs", source);
        let invalid = f.succeeds("installed-rustdoc-invalid-example", &mut test_doc());
        assert!(String::from_utf8_lossy(&invalid.stdout).contains("1 passed; 0 failed"));

        // This is valid Rust outside the initial native subset. Its tag must
        // fail the documentation test, even if Lust would reject the body.
        f.source("src/lib.rs", &source.replace("let _: u8 = doc_consumer::flip(42);",
            "let value = unsafe { std::ptr::read(&42_u64) }; assert_eq!(doc_consumer::flip(value), 43);"));
        let valid = f.invoke("installed-rustdoc-valid-marked-invalid", &mut test_doc());
        assert_eq!(valid.status.code(), Some(101));
        assert!(String::from_utf8_lossy(&valid.stdout).contains("Test compiled successfully"));
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn cargo_builds_a_host_proc_macro_with_the_pinned_compiler() {
    let f = Fixture::new("cargo-host-proc-macro");
    f.source("Cargo.toml", "[workspace]\nresolver=\"3\"\nmembers=[\"provider\"]\n");
    f.source("provider/Cargo.toml", "[package]\nname=\"host_macro\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[lib]\nproc-macro=true\n");
    f.source("provider/src/lib.rs", r#"
use proc_macro::TokenStream;
#[proc_macro]
pub fn add_one(mut input: TokenStream) -> TokenStream {
    input.extend("+ 1".parse::<TokenStream>().unwrap());
    input
}
"#);
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let build = f.succeeds("cargo-host-macro", f.command("cargo").args([
        "build", "--frozen", "-p", "host_macro", "--message-format=json",
    ]));
    let dylib = build.stdout.split(|&byte| byte == b'\n').find_map(|line| {
        let record = serde_json::from_slice::<serde_json::Value>(line).ok()?;
        if record["reason"] != "compiler-artifact" || record["target"]["name"] != "host_macro" {
            return None;
        }
        record["filenames"].as_array()?.iter().filter_map(|path| path.as_str())
            .find(|path| path.ends_with(".so")).map(PathBuf::from)
    }).expect("actual Cargo proc-macro artifact");
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(trace.lines().any(|line| serde_json::from_str::<serde_json::Value>(line).is_ok_and(|record|
        record["engine"] == "pinned-rust" && record["entry"] == "host-proc-macro")), "{trace}");

    let consumer = f.source("consumer.rs", "fn main() { println!(\"{}\", host_macro::add_one!(41)); }\n");
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let executable = f.root.join("original-consumer");
    f.succeeds("original-macro-consumer", Command::new(oracle).current_dir(&f.root)
        .env_remove("LD_LIBRARY_PATH").args(["--edition=2024", "--extern"])
        .arg(format!("host_macro={}", dylib.display())).arg(&consumer).arg("-o").arg(&executable));
    assert_eq!(f.succeeds("original-consumer-run", &mut Command::new(executable)).stdout, b"42\n");

    let compiler = f.package.join("libexec/lustr-host-proc-macro/bin/lustr-rustc-host-proc-macro");
    let regular = f.source("regular.rs", "fn main() {}\n");
    let output = f.root.join("non-macro");
    let refused = f.invoke("host-role-refuses-binary", Command::new(compiler).env_remove("LD_LIBRARY_PATH")
        .args(["--edition=2024", "--crate-type=bin"]).arg(regular).arg("-o").arg(&output));
    assert_eq!(refused.status.code(), Some(1));
    assert!(!output.exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires a package built with --proc-macro-source"]
fn cargo_native_consumer_tracks_a_host_proc_macro() {
    let f = Fixture::new("cargo-native-proc-macro");
    if !f.package.join("libexec/lustr-macro-expansion/bin/lustr-expand-proc-macro").is_file() {
        return;
    }
    f.source("Cargo.toml", r#"
[package]
name = "macro_consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
host_macro = { path = "provider" }
"#);
    f.source("provider/Cargo.toml", r#"
[package]
name = "host_macro"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true
"#);
    f.source("provider/src/lib.rs", r##"
#![feature(proc_macro_diagnostic, proc_macro_tracked_env, proc_macro_tracked_path)]

use proc_macro::{Diagnostic, Ident, Level, Span, TokenStream, TokenTree, tracked};
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT: AtomicU32 = AtomicU32::new(1);

#[proc_macro]
pub fn step(_: TokenStream) -> TokenStream {
    NEXT.fetch_add(1, Ordering::Relaxed).to_string().parse().unwrap()
}

#[proc_macro]
pub fn call_site(_: TokenStream) -> TokenStream {
    "answer + 1".parse().unwrap()
}

#[proc_macro]
pub fn mixed_site(_: TokenStream) -> TokenStream {
    std::iter::once(TokenTree::Ident(Ident::new("answer", Span::mixed_site()))).collect()
}

#[proc_macro]
pub fn explicit_error(_: TokenStream) -> TokenStream {
    Diagnostic::new(Level::Error, "explicit compiler macro diagnostic").emit();
    "42".parse().unwrap()
}

#[proc_macro]
pub fn increment(mut input: TokenStream) -> TokenStream {
    input.extend("+ 1".parse::<TokenStream>().unwrap());
    input
}

#[proc_macro]
pub fn structured(_: TokenStream) -> TokenStream {
    r#"((168u32 >> 2), { [0, 42][1] }, "macro output")"#.parse().unwrap()
}

#[proc_macro]
pub fn value(input: TokenStream) -> TokenStream {
    let Some(TokenTree::Group(group)) = input.into_iter().next() else { panic!("expected a group") };
    assert_eq!(group.span_open().source_text().as_deref(), Some("("));
    assert_eq!(group.span_close().source_text().as_deref(), Some(")"));
    let token = group.stream().into_iter().next().unwrap();
    assert_eq!(token.span().source_text().as_deref(), Some("0x2a"));
    assert!(token.span().file().ends_with("src/main.rs"));
    assert!(std::env::var_os("RUST_ANALYZER_USE_POSTCARD").is_none());
    assert!(std::env::var_os("RUST_ANALYZER_INTERNALS_DO_NOT_USE").is_none());
    let offset = tracked::env_var("LUSTR_TEST_MACRO_OFFSET").unwrap().parse::<u32>().unwrap();
    tracked::path("asset.txt");
    let asset = std::fs::read_to_string("asset.txt").unwrap().trim().parse::<u32>().unwrap();
    (42 + offset + asset).to_string().parse().unwrap()
}
"##);
    f.source("src/main.rs", r#"
fn independent() -> u32 {
    let answer = 16u32;
    assert_eq!(host_macro::call_site!(), 17);
    assert_eq!(host_macro::step!(), 1);
    assert_eq!(host_macro::step!(), 2);
    let (shifted, indexed, text) = host_macro::structured!();
    assert_eq!(shifted, 42);
    assert_eq!(indexed, 42);
    assert_eq!(text, "macro output");
    host_macro::increment!(16)
}
fn expanded() -> u32 { host_macro::value!((0x2a)) }
fn main() { println!("{} {}", independent(), expanded()); }
"#);
    f.source("asset.txt", "1\n");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let cargo = |operation: &str, offset: &str| {
        let mut command = f.command("cargo");
        command.env("LUSTR_TEST_MACRO_OFFSET", offset)
            .env_remove("RUST_ANALYZER_USE_POSTCARD")
            .env_remove("RUST_ANALYZER_INTERNALS_DO_NOT_USE")
            .args([operation, "--frozen", "--jobs", "2"]);
        command
    };
    f.succeeds("check", &mut cargo("check", "0"));
    assert_eq!(f.succeeds("initial", &mut cargo("run", "0")).stdout, b"17 43\n");
    f.source("asset.txt", "2\n");
    assert_eq!(f.succeeds("asset-change", &mut cargo("run", "0")).stdout, b"17 44\n");
    assert_eq!(f.succeeds("environment-change", &mut cargo("run", "3")).stdout, b"17 47\n");
    let binary = f.root.join("target/debug/macro_consumer");
    let original = std::fs::read(&binary).unwrap();
    f.source("asset.txt", "invalid\n");
    let refused = f.invoke("invalid", &mut cargo("build", "3"));
    assert_eq!(refused.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("cannot expand procedural macro"));
    assert_eq!(std::fs::read(&binary).unwrap(), original);
    f.source("asset.txt", "4\n");
    assert_eq!(f.succeeds("repair", &mut cargo("run", "3")).stdout, b"17 49\n");
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(trace.contains("\"entry\":\"host-proc-macro\""), "{trace}");
    assert!(trace.contains("\"engine\":\"lust\""), "{trace}");

    let reports = f.root.join("unused-macro-reports");
    std::fs::create_dir(&reports).unwrap();
    f.source("src/main.rs", "fn main() {}\n");
    f.succeeds("unused-host-macro", cargo("check", "3").env("LUSTR_LIBRARY_REPORT_DIR", &reports));
    let report = std::fs::read_dir(&reports).unwrap().next().unwrap().unwrap().path().join("report.json");
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
    assert_eq!(report["unused_extern_names"], serde_json::json!(["host_macro"]));

    let repaired = std::fs::read(&binary).unwrap();
    f.source("src/main.rs", "fn main() { let _ = host_macro::explicit_error!(); }\n");
    let diagnosed = f.invoke("emitted-macro-error", &mut cargo("build", "3"));
    assert_eq!(diagnosed.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&diagnosed.stderr).contains("explicit compiler macro diagnostic"));
    assert_eq!(std::fs::read(&binary).unwrap(), repaired);
    f.source("src/main.rs", "fn main() { let answer = 42; let _ = host_macro::mixed_site!(); }\n");
    let refused = f.invoke("non-call-site-macro-output", &mut cargo("build", "3"));
    assert_eq!(refused.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("unrepresented returned span"));
    assert_eq!(std::fs::read(&binary).unwrap(), repaired);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn ordinary_cargo_resolves_a_renamed_dependency_feature() {
    let f = Fixture::new("cargo-dependency-feature");
    f.source("Cargo.toml", "[workspace]\nmembers = [\"provider\", \"app\"]\nresolver = \"3\"\n");
    f.source("provider/Cargo.toml", r#"
[package]
name = "profile-codec"
version = "0.1.0"
edition = "2024"

[features]
shifted = []
"#);
    f.source("provider/src/lib.rs", r#"
#![no_std]
#[cfg(feature = "shifted")]
pub fn transform(value: u32) -> u32 { value ^ 15 }
#[cfg(not(feature = "shifted"))]
pub fn transform(value: u32) -> u32 { value ^ 9 }
"#);
    f.source("app/Cargo.toml", r#"
[package]
name = "profile-reader"
version = "0.1.0"
edition = "2024"

[dependencies]
codec = { package = "profile-codec", path = "../provider" }

[features]
shifted = ["codec/shifted"]
"#);
    f.source("app/src/main.rs", "fn main() { let value = codec::transform(12); println!(\"{value}\"); }\n");
    let run = ["run", "--offline", "--quiet", "-p", "profile-reader"];
    let base = f.succeeds("default", f.command("cargo").args(run));
    assert_eq!(base.stdout, b"5\n");
    let shifted = f.succeeds("shifted", f.command("cargo").args(run).args(["--features", "shifted"]));
    assert_eq!(shifted.stdout, b"3\n");
    let reverted = f.succeeds("reverted", f.command("cargo").args(run));
    assert_eq!(reverted.stdout, base.stdout);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn ordinary_cargo_preserves_resolved_features() {
    let f = Fixture::new("resolved-cargo-features");
    f.source("Cargo.toml", r#"
[package]
name = "resolved-cargo-features"
version = "0.1.0"
edition = "2024"

[features]
default = ["shifted"]
shifted = []
"#);
    let source = r#"
#[cfg(feature = "shifted")]
fn result() -> u32 { 9 }
#[cfg(not(feature = "shifted"))]
fn result() -> u32 { 7 }
fn main() { println!("{}", result()); }
"#;
    f.source("src/main.rs", source);
    let run = ["run", "--offline", "--quiet"];
    let defaults = f.succeeds("default", f.command("cargo").args(run));
    assert_eq!(defaults.stdout, b"9\n");
    let disabled = f.succeeds("no-default", f.command("cargo").args(run).arg("--no-default-features"));
    assert_eq!(disabled.stdout, b"7\n");
    let selected = f.succeeds("explicit", f.command("cargo").args(run).args(["--no-default-features", "--features", "shifted"]));
    assert_eq!(selected.stdout, b"9\n");

    f.source("src/main.rs", &source.replace("{ 9 }", "{ \"invalid\" }"));
    let rejected = f.invoke("invalid-feature", f.command("cargo").args(run));
    assert_eq!(rejected.status.code(), Some(101));
    assert!(rejected.stdout.is_empty());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("u32"));
    let unaffected = f.succeeds("disabled-feature", f.command("cargo").args(run).arg("--no-default-features"));
    assert_eq!(unaffected.stdout, b"7\n");

    f.source("src/main.rs", source);
    let repaired = f.succeeds("feature-repair", f.command("cargo").args(run));
    assert_eq!(repaired.stdout, b"9\n");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn ordinary_cargo_checks_release_profile_cfg() {
    let f = Fixture::new("cargo-release-check");
    f.source("Cargo.toml", "[package]\nname = \"release-check\"\nedition = \"2024\"\nversion = \"0.1.0\"\n");
    let source = r#"
#[cfg(debug_assertions)]
fn value() -> u32 { "debug-only invalid type" }
#[cfg(not(debug_assertions))]
fn value() -> u32 { 7 }
fn main() { println!("{}", value()); }
"#;
    f.source("src/main.rs", source);
    let release = ["check", "--offline", "--release"];
    f.succeeds("release-check", f.command("cargo").args(release));
    assert!(!f.root.join("target/release/release-check").exists());

    for (label, args) in [
        ("dev", &["check", "--offline"][..]),
        ("forced-debug", &["check", "--offline", "--release", "--config", "profile.release.debug-assertions=true"][..]),
    ] {
        let rejected = f.invoke(label, f.command("cargo").args(args));
        assert_eq!(rejected.status.code(), Some(101));
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("mismatched types"));
    }
    f.source("src/main.rs", &source.replace("{ 7 }", "{ \"release-only invalid type\" }"));
    let rejected = f.invoke("invalid-release", f.command("cargo").args(release));
    assert_eq!(rejected.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("mismatched types"));
    f.source("src/main.rs", source);
    f.succeeds("release-repair", f.command("cargo").args(release));

    let run = ["run", "--offline", "--release", "--quiet"];
    let optimized = f.succeeds("optimized-run", f.command("cargo").args(run));
    assert_eq!(optimized.stdout, b"7\n");
    let no_op = f.succeeds("optimized-noop", f.command("cargo").args(run));
    assert_eq!(no_op.stdout, optimized.stdout);
    let binary = f.root.join("target/release/release-check");
    let prior = std::fs::read(&binary).unwrap();
    f.source("src/main.rs", &source.replace("{ 7 }", "{ \"release-only invalid type\" }"));
    let invalid = f.invoke("invalid-optimized", f.command("cargo").args(run));
    assert_eq!(invalid.status.code(), Some(101));
    assert!(invalid.stdout.is_empty());
    assert_eq!(std::fs::read(binary).unwrap(), prior);
    f.source("src/main.rs", source);
    let smaller = f.succeeds(
        "size-optimized-run",
        f.command("cargo").args(run).args(["--config", "profile.release.opt-level=\"s\""]),
    );
    assert_eq!(smaller.stdout, optimized.stdout);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn default_debug_cargo_preserves_unwind_behavior() {
    let f = Fixture::new("debug-panic");
    f.source("Cargo.toml", "[package]\nname = \"debug-panic\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("src/main.rs", r#"
struct Mark(&'static str);
impl Drop for Mark {
    fn drop(&mut self) { println!("drop:{}", self.0); }
}
fn leaf() {
    let _frame = Mark("leaf");
    panic!("original payload");
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let _outer = Mark("outer");
    let recovered = std::panic::catch_unwind(|| {
        let _middle = Mark("middle");
        leaf();
    });
    let payload = recovered.unwrap_err();
    let resumed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| std::panic::resume_unwind(payload)));
    assert_eq!(resumed.unwrap_err().downcast_ref::<&'static str>(), Some(&"original payload"));
    let joined = std::thread::spawn(|| panic!("thread payload")).join();
    assert_eq!(joined.unwrap_err().downcast_ref::<&'static str>(), Some(&"thread payload"));
    println!("caught, resumed, joined");
}
"#);
    let ran = f.succeeds("runtime", f.command("cargo").args(["run", "--offline", "--quiet"]));
    assert_eq!(ran.stdout, b"drop:leaf\ndrop:middle\ncaught, resumed, joined\ndrop:outer\n");
    let optimized = f.succeeds("runtime-release", f.command("cargo").args(["run", "--offline", "--release", "--quiet"]));
    assert_eq!(optimized.stdout, ran.stdout);
    f.source("src/main.rs", "fn main() { panic!(\"uncaught native default debug\"); }\n");
    let failed = f.invoke("uncaught", f.command("cargo").args(["run", "--offline", "--quiet"]));
    assert_eq!(failed.status.code(), Some(101));
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("panicked at") && stderr.contains("uncaught native default debug"), "{stderr}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn ordinary_cargo_executes_native_tests_with_libtest() {
    let f = Fixture::new("rust-test");
    f.source("Cargo.toml", "[package]\nname = \"test-leaf\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("src/main.rs", r#"
fn main() { panic!("ordinary entry was called"); }

#[cfg(test)]
mod tests {
    #[test]
    fn arithmetic() {
        println!("native print is captured");
        assert_eq!(21_u32 * 2, 42);
    }
    #[test]
    #[should_panic(expected = "native panic text")]
    fn expected() { panic!("native panic text"); }
    #[test]
    #[ignore = "negative control"]
    #[should_panic(expected = "wrong text")]
    fn wrong() { panic!("native panic text"); }
}
"#);
    let no_run = f.succeeds("compile", f.command("cargo").args(["test", "--offline", "--no-run"]));
    assert!(no_run.stdout.is_empty());
    let listed = f.succeeds("list", f.command("cargo").args(["test", "--offline", "--", "--list"]));
    for name in ["tests::arithmetic: test", "tests::expected: test", "tests::wrong: test"] {
        assert!(String::from_utf8_lossy(&listed.stdout).contains(name));
    }
    let ran = f.succeeds("run", f.command("cargo").args(["test", "--offline", "--", "--test-threads=2"]));
    let stdout = String::from_utf8_lossy(&ran.stdout);
    assert!(stdout.contains("2 passed; 0 failed; 1 ignored"));
    assert!(stdout.contains("ignored, negative control"));
    assert!(!stdout.contains("native print is captured"));
    assert!(!String::from_utf8_lossy(&ran.stderr).contains("native panic text"));
    let shown = f.succeeds("show-output", f.command("cargo").args(["test", "--offline", "--", "--show-output", "tests::arithmetic"]));
    assert!(String::from_utf8_lossy(&shown.stdout).contains("native print is captured"));

    let wrong = f.invoke("wrong", f.command("cargo").args(["test", "--offline", "--", "--ignored", "tests::wrong"]));
    assert_eq!(wrong.status.code(), Some(101));
    let stdout = String::from_utf8_lossy(&wrong.stdout);
    assert!(stdout.contains("panic did not contain expected string"));
    assert!(stdout.contains("thread 'tests::wrong'"));
    assert!(!stdout.contains("ordinary entry was called"));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn default_cargo_test_reports_native_termination() {
    let f = Fixture::new("test-result");
    f.source("Cargo.toml", "[package]\nname = \"test-result\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("src/main.rs", r#"
#[test]
fn result_passes() -> Result<(), &'static str> { Ok(()) }

#[test]
fn exit_code_passes() -> std::process::ExitCode { std::process::ExitCode::SUCCESS }

#[test]
#[ignore]
fn result_fails() -> Result<(), &'static str> { Err("native returned error") }

#[test]
#[ignore]
fn exit_code() -> std::process::ExitCode { std::process::ExitCode::from(17) }
"#);
    let pass = f.succeeds("pass", f.command("cargo").args(["test", "--offline"]));
    let stdout = String::from_utf8_lossy(&pass.stdout);
    assert!(stdout.contains("result_passes ... ok"), "{stdout}");
    assert!(stdout.contains("exit_code_passes ... ok"), "{stdout}");
    let failure = f.invoke("failed-result", f.command("cargo").args(["test", "--offline", "--", "--ignored", "result_fails"]));
    assert_eq!(failure.status.code(), Some(101));
    let stdout = String::from_utf8_lossy(&failure.stdout);
    assert!(stdout.contains("Error: \"native returned error\""), "{stdout}");
    assert!(stdout.contains("result_fails ... FAILED"), "{stdout}");
    let exit = f.invoke("failed-exit-code", f.command("cargo").args(["test", "--offline", "--", "--ignored", "exit_code"]));
    assert_eq!(exit.status.code(), Some(101));
    let stdout = String::from_utf8_lossy(&exit.stdout);
    assert!(stdout.contains("exit_code ... FAILED"), "{stdout}");
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn cargo_selects_library_unit_test_source() {
    let f = Fixture::new("library-unit-test");
    f.source("Cargo.toml", "[package]\nname = \"library-unit-test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"selected_library\"\npath = \"lib/entry.rs\"\n");
    f.source("lib/entry.rs", r#"
fn calculate(input: u32) -> u32 { input + 9 }

#[cfg(test)]
mod tests {
    #[test]
    fn calls_library_code() { assert_eq!(super::calculate(4), 13); }
}
"#);
    let compiled = f.succeeds("compile", f.command("cargo").args(["test", "--offline", "--lib", "--no-run"]));
    assert!(compiled.stdout.is_empty());
    let ran = f.succeeds("run", f.command("cargo").args(["test", "--offline", "--lib"]));
    let stdout = String::from_utf8_lossy(&ran.stdout);
    assert!(stdout.contains("tests::calls_library_code ... ok"), "{stdout}");
    assert!(stdout.contains("1 passed; 0 failed"), "{stdout}");
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn cargo_runs_a_selected_integration_test_target() {
    let f = Fixture::new("integration-test-target");
    f.source("Cargo.toml", "[package]\nname = \"integration-test-target\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("src/main.rs", "fn main() { println!(\"original application\"); }\n");
    f.source("tests/behavior.rs", r#"
#[test]
fn checks_application() {
    let application = env!("CARGO_BIN_EXE_integration-test-target");
    assert!(!application.is_empty());
    assert_eq!(4_u32 * 9, 36);
}
"#);
    let compiled = f.succeeds("compile", f.command("cargo").args(["test", "--offline", "--test", "behavior", "--no-run"]));
    assert!(compiled.stdout.is_empty());
    let tested = f.succeeds("run", f.command("cargo").args(["test", "--offline", "--test", "behavior"]));
    let stdout = String::from_utf8_lossy(&tested.stdout);
    assert!(stdout.contains("checks_application ... ok"), "{stdout}");
    assert!(stdout.contains("1 passed; 0 failed"), "{stdout}");
    let binary = f.root.join("target/debug/integration-test-target");
    assert_eq!(f.succeeds("application", &mut Command::new(binary)).stdout, b"original application\n");
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn cargo_runs_the_crates_custom_test_main() {
    let f = Fixture::new("custom-test-harness");
    f.source("Cargo.toml", "[package]\nname = \"custom-harness\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[[test]]\nname = \"behavior\"\nharness = false\n");
    f.source("src/main.rs", "fn main() { println!(\"ordinary application\"); }\n");
    f.source("tests/behavior.rs", r#"
#[cfg(test)]
fn main() {
    assert!(!env!("CARGO_BIN_EXE_custom-harness").is_empty());
    assert!(!env!("CARGO_TARGET_TMPDIR").is_empty());
    println!("custom harness uses cfg(test)");
}

#[cfg(not(test))]
fn main() { panic!("test entry compiled with the normal cfg"); }
"#);
    let args = ["test", "--offline", "--test", "behavior"];
    let compiled = f.succeeds("compile", f.command("cargo").args(args).arg("--no-run"));
    assert!(compiled.stdout.is_empty());
    let ran = f.succeeds("run", f.command("cargo").args(args));
    assert_eq!(ran.stdout, b"custom harness uses cfg(test)\n");
    let binary = f.root.join("target/debug/custom-harness");
    assert_eq!(f.succeeds("application", &mut Command::new(binary)).stdout, b"ordinary application\n");
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and Clang"]
fn cargo_test_links_a_native_dev_dependency() {
    let f = Fixture::new("integration-dev-dep");
    f.source("Cargo.toml", "[workspace]\nmembers = [\"provider\", \"application\"]\nresolver = \"3\"\n");
    f.source("provider/Cargo.toml", "[package]\nname = \"test-provider\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    f.source("provider/src/lib.rs", "#![no_std]\npub fn evaluate(input: u32) -> u32 { input ^ 9 }\n");
    f.source("application/Cargo.toml", "[package]\nname = \"with-dev-dep\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dev-dependencies]\ntest-provider = { path = \"../provider\" }\n");
    f.source("application/src/main.rs", "fn main() { println!(\"ordinary executable\"); }\n");
    f.source("application/tests/behavior.rs", r#"
#[test]
fn crosses_a_native_crate_boundary() {
    assert_eq!(test_provider::evaluate(12), 5);
    assert!(!env!("CARGO_BIN_EXE_with-dev-dep").is_empty());
    assert!(!env!("CARGO_TARGET_TMPDIR").is_empty());
}
"#);
    let args = ["test", "--offline", "-p", "with-dev-dep", "--test", "behavior"];
    let compiled = f.succeeds("compile", f.command("cargo").args(args).arg("--no-run"));
    assert!(compiled.stdout.is_empty());
    let tested = f.succeeds("run", f.command("cargo").args(args));
    let stdout = String::from_utf8_lossy(&tested.stdout);
    assert!(stdout.contains("crosses_a_native_crate_boundary ... ok"), "{stdout}");
    assert!(stdout.contains("1 passed; 0 failed"), "{stdout}");
    let binary = f.root.join("target/debug/with-dev-dep");
    assert_eq!(f.succeeds("application", &mut Command::new(binary)).stdout, b"ordinary executable\n");
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

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and binutils"]
fn ordinary_cargo_compiles_a_native_function_with_rust_async_code() {
    let f = Fixture::new("cargo-async-composition");
    f.source("Cargo.toml", "[package]\nname=\"async_program\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    let source = r#"
use std::{future::Future, pin::pin, task::{Context, Poll, Waker}};

fn apply_mask(input: u64) -> u64 { input ^ 19 }

async fn compute(input: u64) -> u64 {
    let apply = std::hint::black_box(apply_mask as fn(u64) -> u64);
    apply(std::future::ready(input).await)
}

fn main() {
    let mut task = pin!(compute(57));
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(value) = task.as_mut().poll(&mut context) else { unreachable!() };
    println!("{value}");
}
"#;
    let path = f.source("src/main.rs", source);
    let rust = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let cargo = |name: &str, original: bool, mixed: bool| {
        let mut command = f.command("cargo");
        command.env("CARGO_TARGET_DIR", f.root.join(name));
        if mixed {
            command.env("CARGO_PROFILE_DEV_DEBUG", "0");
        }
        if original {
            command.env("RUSTC", &rust);
        }
        command
    };
    f.succeeds("lockfile", cargo("default", false, false).args(["generate-lockfile", "--offline"]));
    f.succeeds("default-check", cargo("default", false, false).args(["check", "--frozen"]));
    for (name, original, mixed) in [("default", false, false), ("mixed", false, true), ("original", true, true)] {
        let result = f.succeeds(&format!("{name}-run"), cargo(name, original, mixed).args(["run", "--frozen"]));
        assert_eq!(result.stdout, b"42\n");
        let exe = f.root.join(name).join("debug/async_program");
        let symbols = f.succeeds(&format!("{name}-symbols"), Command::new("nm").arg(exe));
        let symbols = String::from_utf8(symbols.stdout).unwrap();
        let native = symbols.lines().filter(|line| line.split_whitespace().last()
            .and_then(|name| name.strip_prefix("f_")).is_some_and(|suffix| {
                suffix.len() == 16 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
            })).count();
        assert_eq!(native, usize::from(mixed && !original), "{name}");
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(trace.contains("\"entry\":\"rust-complement\""), "{trace}");

    for (name, edit, diagnostic) in [
        ("bad-native", source.replace("input ^ 19", "input ^ false"), "L0113"),
        ("bad-rust", source.replace("async fn compute(input: u64) -> u64", "async fn compute(input: u64) -> u32"), "E0308"),
    ] {
        f.source("src/main.rs", &edit);
        let result = f.invoke(name, cargo(name, false, true).args(["build", "--frozen"]));
        assert_eq!(result.status.code(), Some(101));
        assert!(String::from_utf8_lossy(&result.stderr).contains(diagnostic),
            "{}", String::from_utf8_lossy(&result.stderr));
        assert!(!f.root.join(name).join("debug/async_program").exists());
    }
    let traces = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    f.source("src/main.rs", "fn main() { let value: u64 = false; println!(\"{value}\"); }\n");
    let result = f.invoke("ordinary-invalid", cargo("ordinary-invalid", false, true).args(["check", "--frozen"]));
    assert_eq!(result.status.code(), Some(101));
    let later = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(!later[traces.len()..].contains("\"entry\":\"rust-complement\""), "{later}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and binutils"]
fn ordinary_cargo_uses_the_rust_context_for_unrepresented_dependencies() {
    let f = Fixture::new("cargo-rust-dependency-context");
    f.source("Cargo.toml", "[workspace]\nmembers=[\"producer\",\"app\"]\nresolver=\"3\"\n");
    f.source("producer/Cargo.toml", "[package]\nname=\"producer\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    f.source("app/Cargo.toml", "[package]\nname=\"app\"\nversion=\"0.1.0\"\nedition=\"2024\"\n\n[dependencies]\nproducer={path=\"../producer\"}\n");
    f.source("producer/src/lib.rs", r#"
pub struct Counter(pub u64);
pub trait Value { fn value(&self) -> u64; }
impl Value for Counter { fn value(&self) -> u64 { self.0 } }
pub async fn preserve(input: Counter) -> u64 { input.value() }
"#);
    let program = r#"
use producer::{Counter, Value};

fn mask(input: u64) -> u64 { input ^ 19 }

fn main() { println!("{}", mask(Counter(57).value())); }
"#;
    let source = f.source("app/src/main.rs", program);
    let rust = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    for (name, original) in [("mixed", false), ("original", true)] {
        let cargo = || {
            let mut command = f.command("cargo");
            command.env("CARGO_TARGET_DIR", f.root.join(name)).env("CARGO_PROFILE_DEV_DEBUG", "0");
            if original { command.env("RUSTC", &rust); }
            command
        };
        if !original {
            f.succeeds("lockfile", cargo().args(["generate-lockfile", "--offline"]));
        }
        f.succeeds(&format!("{name}-check"), cargo().args(["check", "--workspace", "--frozen"]));
        let run = f.succeeds(&format!("{name}-run"), cargo().args(["run", "-p", "app", "--frozen"]));
        assert_eq!(run.stdout, b"42\n");
        let exe = f.root.join(name).join("debug/app");
        let symbols = f.succeeds(&format!("{name}-symbols"), Command::new("nm").arg(exe));
        let symbols = String::from_utf8(symbols.stdout).unwrap();
        let native = symbols.lines().filter(|line| line.split_whitespace().last()
            .and_then(|name| name.strip_prefix("f_")).is_some_and(|suffix| {
                suffix.len() == 16 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
            })).count();
        assert_eq!(native, usize::from(!original));
    }
    assert_eq!(std::fs::read_to_string(&source).unwrap(), program);
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(trace.matches("\"entry\":\"rust-complement\"").count() >= 4, "{trace}");

    let invalid = f.source("libbroken.rmeta", "invalid Rust metadata\n");
    let simple = f.source("invalid-import.rs", "extern crate broken; fn main() {}\n");
    let response = f.invoke("malformed-metadata", f.command("rustc").args([
        "--crate-name", "invalid_import", "--edition=2024", "--emit=metadata", "--extern",
    ]).arg(format!("broken={}", invalid.display())).arg(simple).arg("-o").arg(f.root.join("failed.rmeta")));
    assert!(!response.status.success());
    assert!(!f.root.join("failed.rmeta").exists());
    let after = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(!after[trace.len()..].contains("\"entry\":\"rust-complement\""));
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn installed_rust_driver_handles_ordinary_cargo_options_outside_the_native_adapter() {
    let f = Fixture::new("cargo-ordinary-rust-options");
    f.source("Cargo.toml", "[package]\nname=\"edition_fixture\"\nversion=\"0.1.0\"\nedition=\"2021\"\n");
    let source = f.source("src/main.rs", r#"
fn main() {
    let answer = [1, 2, 3].into_iter().map(|n| n * 7).sum::<u32>();
    println!("{answer}");
}
"#);
    let original = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    for (name, oracle) in [("lustr", false), ("original", true)] {
        let cargo = || {
            let mut command = f.command("cargo");
            command.env("CARGO_TARGET_DIR", f.root.join(name));
            if oracle { command.env("RUSTC", &original); }
            command
        };
        f.succeeds(&format!("{name}-check"), cargo().args(["check", "--frozen"]));
        let response = f.succeeds(&format!("{name}-run"), cargo().args(["run", "--frozen"]));
        assert_eq!(response.stdout, b"42\n");
    }
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(trace.contains("\"entry\":\"rust-request\""), "{trace}");

    let object = f.root.join("result.o");
    f.succeeds("object", f.command("rustc").args(["--edition=2021", "--emit=obj"])
        .arg(&source).arg("-o").arg(&object));
    assert_eq!(&std::fs::read(object).unwrap()[..4], b"\x7fELF");
    f.source("src/main.rs", "fn main() { let _: u64 = false; }\n");
    let response = f.invoke("invalid", f.command("cargo").args(["check", "--frozen"])
        .env("CARGO_TARGET_DIR", f.root.join("invalid")));
    assert_eq!(response.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&response.stderr).contains("E0308"));
    assert!(!f.root.join("invalid/debug/edition_fixture").exists());
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn ordinary_cargo_resolves_transitive_rust_metadata() {
    let f = Fixture::new("cargo-rust-transitive-context");
    f.source("Cargo.toml", "[workspace]\nmembers=[\"app\",\"bridge\",\"provider\"]\nresolver=\"3\"\n");
    for (name, versioned) in [
        ("app", "[dependencies]\nbridge={path=\"../bridge\"}\n"),
        ("bridge", "[dependencies]\nprovider={path=\"../provider\"}\n"),
        ("provider", ""),
    ] {
        f.source(&format!("{name}/Cargo.toml"), &format!(
            "[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2024\"\n{versioned}"));
    }
    f.source("provider/src/lib.rs", r#"
pub struct Counter(pub u64);
pub trait Value { fn value(self) -> u64; }
impl Value for Counter { fn value(self) -> u64 { self.0 } }
pub async fn value(counter: Counter) -> u64 { counter.value() }
"#);
    f.source("bridge/src/lib.rs", r#"
pub use provider::Value;
pub fn convert(value: u64) -> provider::Counter { provider::Counter(value) }
"#);
    f.source("app/src/main.rs", r#"
use bridge::Value;
fn mask(value: u64) -> u64 { value ^ 19 }
fn main() { println!("{}", mask(bridge::convert(57).value())); }
"#);
    let oracle = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let cargo = |name: &str, original: bool| {
        let mut command = f.command("cargo");
        command.env("CARGO_TARGET_DIR", f.root.join(name)).env("CARGO_PROFILE_DEV_DEBUG", "0");
        if original { command.env("RUSTC", &oracle); }
        command
    };
    for (name, original) in [("lustr", false), ("original", true)] {
        f.succeeds(&format!("{name}-check"), cargo(name, original).args(["check", "--workspace", "--frozen"]));
        let result = f.succeeds(&format!("{name}-run"), cargo(name, original).args(["run", "-p", "app", "--frozen"]));
        assert_eq!(result.stdout, b"42\n");
    }
    f.source("bridge/src/lib.rs", "pub fn convert(value: u64) -> provider::Counter { provider::Counter(value == 0) }\n");
    for (name, original) in [("lustr-invalid", false), ("original-invalid", true)] {
        let result = f.invoke(name, cargo(name, original).args(["check", "--workspace", "--frozen"]));
        assert_eq!(result.status.code(), Some(101));
        assert!(String::from_utf8_lossy(&result.stderr).contains("E0308"));
        assert!(!f.root.join(name).join("debug/app").exists());
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn installed_rust_reports_the_requested_non_host_target() {
    let f = Fixture::new("cargo-cross-target-discovery");
    let original = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    for target in [vec!["--target", "wasm32-unknown-unknown"], vec!["--target=wasm32-unknown-unknown"]] {
        let query = [
            "-", "--crate-name", "probe", "--crate-type=cdylib",
            "--print=file-names", "--print=cfg", "--print=split-debuginfo",
        ];
        let expected = f.succeeds("original-wasm-discovery",
            Command::new(&original).current_dir(&f.root).env_remove("LD_LIBRARY_PATH").args(query).args(&target));
        let answer = f.succeeds("lustr-wasm-discovery", f.command("rustc").args(query).args(&target));
        assert_eq!(answer.stdout, expected.stdout);
        assert!(String::from_utf8_lossy(&answer.stdout).contains("target_arch=\"wasm32\""));
    }
    let original = f.succeeds("original-target-list",
        Command::new(&original).current_dir(&f.root).env_remove("LD_LIBRARY_PATH").arg("--print=target-list"));
    let answer = f.succeeds("lustr-target-list", f.command("rustc").args(["--print", "target-list"]));
    assert_eq!(answer.stdout, original.stdout);
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT, LUSTR_TEST_ORACLE and binutils"]
fn ordinary_cargo_checks_unsafe_in_the_pinned_rust_compiler() {
    let f = Fixture::new("cargo-unsafe-composition");
    f.source("Cargo.toml", "[package]\nname=\"unsafe_composition\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    let source = r#"
fn mask(value: u64) -> u64 { value ^ 19 }
unsafe fn load(pointer: *const u64) -> u64 { unsafe { *pointer } }
fn main() {
    let value = unsafe { load(&57) };
    println!("{}", mask(value));
}
"#;
    let path = f.source("src/main.rs", source);
    let original = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let cargo = |lane: &str, oracle: bool| {
        let mut command = f.command("cargo");
        command.env("CARGO_PROFILE_DEV_DEBUG", "0").env("CARGO_TARGET_DIR", f.root.join(lane));
        if oracle { command.env("RUSTC", &original); }
        command
    };
    for (lane, oracle) in [("lustr", false), ("original", true)] {
        f.succeeds(&format!("{lane}-check"), cargo(lane, oracle).args(["check", "--frozen"]));
        let run = f.succeeds(&format!("{lane}-run"), cargo(lane, oracle).args(["run", "--frozen"]));
        assert_eq!(run.stdout, b"42\n");
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    let binary = f.root.join("lustr/debug/unsafe_composition");
    let symbols = f.succeeds("native-symbols", Command::new("nm").arg(binary));
    assert!(String::from_utf8_lossy(&symbols.stdout).lines().any(|line|
        line.split_whitespace().last().and_then(|name| name.strip_prefix("f_"))
            .is_some_and(|id| id.len() == 16 && id.bytes().all(|byte| byte.is_ascii_hexdigit()))));

    f.source("src/main.rs", &source.replace("unsafe { load(&57) }", "load(&57)"));
    for (lane, oracle) in [("lustr-invalid", false), ("original-invalid", true)] {
        let response = f.invoke(lane, cargo(lane, oracle).args(["build", "--frozen"]));
        assert_eq!(response.status.code(), Some(101));
        assert!(String::from_utf8_lossy(&response.stderr).contains("E0133"));
        assert!(!f.root.join(lane).join("debug/unsafe_composition").exists());
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn cargo_runs_proc_macro_unit_tests_with_the_matching_rust_runtime() {
    let f = Fixture::new("cargo-proc-macro-test-runtime");
    f.source("Cargo.toml", "[package]\nname=\"macro_unit_test\"\nversion=\"0.1.0\"\nedition=\"2024\"\n\n[lib]\nproc-macro=true\n");
    f.source("src/lib.rs", r#"
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn preserve(input: TokenStream) -> TokenStream { input }

#[test]
fn observed_test_body() { assert_eq!(6 * 7, 42); }
"#);
    let original = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    for (lane, oracle) in [("lustr", false), ("original", true)] {
        let mut cargo = f.command("cargo");
        cargo.args(["test", "--frozen", "--lib"]).env("CARGO_TARGET_DIR", f.root.join(lane));
        if oracle { cargo.env("RUSTC", &original); }
        let result = f.succeeds(lane, &mut cargo);
        assert!(String::from_utf8_lossy(&result.stdout).contains("test observed_test_body ... ok"));
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn cargo_library_accepts_an_explicit_rust_archive() {
    let f = Fixture::new("cargo-explicit-rust-archive");
    let original = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let provider = f.source("provider.rs", "#![no_std]\npub fn twice(value: u64) -> u64 { value.wrapping_mul(2) }\n");
    let archive = f.root.join("libprovider.rlib");
    f.succeeds("lustr-provider", f.command("rustc").args([
        "--crate-name", "provider", "--crate-type=rlib", "--edition=2024",
        "-Zembed-metadata=yes", "-Cembed-bitcode=no", "--emit=link",
    ]).arg(&provider).arg("-o").arg(&archive));
    let published = std::fs::read(&archive).unwrap();
    assert!(!f.root.join("libprovider.rmeta").exists());

    f.source("Cargo.toml", "[package]\nname=\"archive_consumer\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    f.source("src/lib.rs", "#![no_std]\npub fn run(value: u64) -> u64 { external::twice(value) }\n");
    let main = f.source("main.rs", "fn main() { println!(\"{}\", archive_consumer::run(21)); }\n");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let dependency = format!("external={}", archive.display());
    for (lane, oracle) in [("lustr", false), ("original", true)] {
        let output = f.root.join(lane);
        let mut cargo = f.command("cargo");
        cargo.args(["rustc", "--lib", "--frozen", "--message-format=json", "--", "--extern", &dependency, "-Zembed-metadata=yes"])
            .env("CARGO_TARGET_DIR", &output);
        if oracle { cargo.env("RUSTC", &original); }
        let compiled = f.succeeds(&format!("{lane}-consumer"), &mut cargo);
        let library = compiled.stdout.split(|&byte| byte == b'\n').find_map(|line| {
            let record: serde_json::Value = serde_json::from_slice(line).ok()?;
            if record["reason"] != "compiler-artifact" || record["target"]["name"] != "archive_consumer" {
                return None;
            }
            record["filenames"].as_array()?.iter().filter_map(|filename| filename.as_str())
                .find(|path| path.ends_with(".rlib")).map(PathBuf::from)
        }).expect("Cargo reported the archive consumer's actual library");
        assert!(library.is_file());
        let application = output.join("application");
        f.succeeds(&format!("{lane}-app-link"), Command::new(&original)
            .current_dir(&f.root).env_remove("LD_LIBRARY_PATH")
            .arg("--edition=2024").arg(&main).arg("--extern")
            .arg(format!("archive_consumer={}", library.display()))
            .arg("-L").arg(format!("dependency={}", f.root.display()))
            .arg("-o").arg(&application));
        assert_eq!(f.succeeds(&format!("{lane}-app"), &mut Command::new(application)).stdout, b"42\n");
        assert_eq!(std::fs::read(&archive).unwrap(), published);
    }
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(trace.contains("\"entry\":\"rust-request\""), "{trace}");
}

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE, LUSTR_TEST_OUTPUT and independent LUSTR_TEST_ORACLE"]
fn cargo_std_library_retains_a_native_no_std_dependency() {
    let f = Fixture::new("cargo-std-library-dependency");
    f.source("Cargo.toml", "[workspace]\nmembers=[\"app\",\"relay\",\"provider\"]\nresolver=\"3\"\n");
    for (name, dependencies) in [
        ("app", "[dependencies]\nrelay={path=\"../relay\"}\n"),
        ("relay", "[dependencies]\nprovider={path=\"../provider\"}\n"),
        ("provider", ""),
    ] {
        f.source(&format!("{name}/Cargo.toml"), &format!(
            "[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2024\"\n{dependencies}"));
    }
    let provider = "#![no_std]\npub fn mask(value: u64) -> u64 { value ^ 19 }\n";
    f.source("provider/src/lib.rs", provider);
    f.source("relay/src/lib.rs", "pub fn evaluate(value: u64) -> u64 { provider::mask(value) }\n");
    f.source("app/src/main.rs", "fn main() { println!(\"{}\", relay::evaluate(57)); }\n");
    f.succeeds("lockfile", f.command("cargo").args(["generate-lockfile", "--offline"]));
    let original = std::env::var_os("LUSTR_TEST_ORACLE").expect("independent LUSTR_TEST_ORACLE");
    let mut previous = None;
    for (step, source, expected) in [
        ("initial", provider.to_owned(), b"42\n".as_slice()),
        ("provider-body-edit", provider.replace("19", "18"), b"43\n".as_slice()),
    ] {
        f.source("provider/src/lib.rs", &source);
        let check = f.succeeds(&format!("{step}-check"), f.command("cargo").args([
            "check", "--workspace", "--frozen", "--message-format=json",
        ]));
        let metadata = check.stdout.split(|&byte| byte == b'\n').find_map(|line| {
            let record: serde_json::Value = serde_json::from_slice(line).ok()?;
            if record["reason"] != "compiler-artifact" || record["target"]["name"] != "relay" {
                return None;
            }
            record["filenames"].as_array()?.iter().filter_map(|filename| filename.as_str())
                .find(|path| path.ends_with(".rmeta")).map(PathBuf::from)
        }).expect("Cargo reported relay's metadata");
        let facts = f.succeeds(&format!("{step}-original-reader"), Command::new(&original)
            .env_remove("LD_LIBRARY_PATH").arg("-Zls=root").arg(&metadata));
        let text = String::from_utf8(facts.stdout).unwrap();
        assert!(text.lines().any(|line| line.starts_with("20 provider-")), "{text}");
        let bytes = std::fs::read(metadata).unwrap();
        let hash = <[u8; 16]>::try_from(&bytes[24..40]).unwrap();
        if let Some(prior) = previous { assert_ne!(hash, prior); }
        previous = Some(hash);
        let run = f.succeeds(&format!("{step}-run"), f.command("cargo").args(["run", "-p", "app", "--frozen"]));
        assert_eq!(run.stdout, expected);
        let oracle = f.succeeds(&format!("{step}-original-run"), f.command("cargo")
            .args(["run", "-p", "app", "--frozen"]).env("RUSTC", &original)
            .env("CARGO_TARGET_DIR", f.root.join("original-target")));
        assert_eq!(oracle.stdout, expected);
    }
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(!trace.contains("\"entry\":\"rust-complement\""), "{trace}");
}
