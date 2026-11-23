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
        (mode @ (b"1" | b"2" | b"3"), Some((library, options))) => {
            if (mode == b"2") != library.external.is_some() || (mode == b"3") != library.is_executable() {
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
            let cfg_count = read_public_transport_count(reader, "check-cfg", 2)?;
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
        }
        (b"0" | b"1" | b"2" | b"3", _) => return Err("public request changed its artifact context presence".into()),
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
        check_cfg: vec!["cfg(docsrs,test)".into(), "cfg(feature, values())".into()],
        search_paths: vec![PathBuf::from("/native-transport/deps-one"), PathBuf::from("/native-transport/deps-two")],
        external: None,
        executable_dependencies: None,
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
    assert_eq!(fields.len(), 33, "library schema/options plus two features and two environment pairs");
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
        (17, b"invalid"), (17, b"3"), (17, b"1"), // malformed, oversized or incomplete cfg inventory
        (19, b"cfg(wrong)"),
        (20, b"4097"), (20, b"1"), // oversized or incomplete search inventory
        (22, b"/wrong/dependency"),
        (23, b"invalid"), (23, b"-"), (23, b"1"), // malformed, absent or different embedding choice
        (24, b"2"), (24, b"0"), // malformed or different forced-lint choice
        (25, b"102"), (25, b"00"), (25, b"999"), // changed, truncated or unknown debug output choices
    ];
    for &(index, bytes) in wrong_fields {
        let changed = replace_field(index, bytes);
        assert!(read_public_transport_request(&mut Cursor::new(&changed), identity, &req.file, &req.out, true, Some((&library, req.library_options))).is_err(),
                "changed library field {index} was accepted");
    }
    for index in [19, 22, 23, 24, 25, 32] {
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
