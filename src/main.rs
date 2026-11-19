use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Command, ExitCode};
mod native_link;
mod native_queries;
mod owned_child;

#[allow(dead_code)]
mod lust {
    include!("../compiler/native/src/client.rs");

    mod selection {
        include!("cargo_selection.rs");
    }

    pub struct Call(protocol::Request, Option<selection::Selection>, Option<Vec<String>>);

    pub fn host_triple() -> &'static str {
        protocol::host_triple()
    }

    pub fn prime_selection_tools(parent: &Path) -> std::io::Result<()> {
        selection::prime_selection_tools(parent)
    }

    impl Call {
        pub fn needs_rust_artifact(&self) -> bool {
            matches!(self.0.cmd, protocol::Cmd::Build) && self.0.session.krate.target_kind == "lib"
                || !(self.0.session.target_triple.is_empty() || self.0.session.target_triple == protocol::host_triple())
        }
        pub fn library_compatible(&self) -> bool {
            self.0.session.krate.manifest_path.is_none() && self.expansion_compatible()
        }
        pub fn expansion_compatible(&self) -> bool {
            protocol::source_override().is_none()
                && !std::env::vars_os()
                    .any(|(name, _)| name.to_str().is_some_and(|s| s.starts_with("LUST_FRONT_") || s == "LUST_SOURCES_READY"))
        }
        pub fn validate_selection(&mut self, parent: &Path, args: &[String]) -> std::io::Result<()> {
            if self.0.session.krate.manifest_path.is_some() {
                let selection = selection::select(&self.0, parent, args)?;
                // The unchanged native resolver selects within the supplied
                // manifest. Bind it to the validated member and target too.
                self.0.session.krate.manifest_path = Some(selection.manifest.clone());
                self.0.session.krate.target_name = selection.name.clone();
                self.1 = Some(selection);
            }
            Ok(())
        }
        pub fn bind_native_arguments(&self, args: &mut Vec<String>) {
            if self.2.is_some() {
                // Build once through the owned request; only after success may
                // the public entry execute the unchanged caller's program.
                args[0] = "build".into();
                if let Some(at) = args.iter().position(|arg| arg == "--") {
                    args.truncate(at);
                }
            }
            if let Some(selection) = &self.1 {
                // The daemon receives the bound Request; the original
                // process reparses argv and needs the identical selection.
                let at = args.iter().position(|arg| arg == "--").unwrap_or(args.len());
                let mut bound = vec!["--manifest-path".to_owned(), selection.manifest.to_str().unwrap().to_owned()];
                if self.0.session.krate.target_kind == "bin" {
                    bound.extend(["--bin".to_owned(), selection.name.clone()]);
                }
                args.splice(at..at, bound);
            }
        }
        pub fn run_program(&self) -> Option<std::io::Error> {
            self.2.as_ref().map(|args| std::process::Command::new(&self.0.out).args(args).exec())
        }
    }

    pub fn prepare(args: &[String]) -> Option<Call> {
        match protocol::parse_args(args) {
            Some(protocol::Invocation::Compile { mut req, run, args }) => {
                req.session.krate.package_id.push_str("#lustr-exclusive-v1");
                Some(Call(req, None, run.then_some(args)))
            }
            _ => None,
        }
    }

    pub fn native_release(args: &[String]) -> bool {
        matches!(protocol::parse_args(args), Some(protocol::Invocation::ReleaseCc { reqs: Ok(reqs) })
            if reqs.iter().all(|request| request.session.krate.manifest_path.is_none()))
    }

    pub fn answer(call: &Call, core: &Path) -> Result<Option<(i32, Vec<u8>, Vec<u8>)>, String> {
        if env_on("LUST_NO_DAEMON") || protocol::source_override().is_some() {
            return Ok(None);
        }
        let socket = protocol::socket_path();
        answer_at(call, core, &socket)
    }

    fn answer_at(call: &Call, core: &Path, socket: &Path) -> Result<Option<(i32, Vec<u8>, Vec<u8>)>, String> {
        let address = protocol::host_runtime::DaemonSocket::resolve(socket).map_err(|e| format!("native daemon address: {e}"))?;
        let socket = address.path();
        if let Some(r) = protocol::request(&socket, &call.0)? {
            return Ok(Some((r.code, r.stdout.into_bytes(), r.stderr.into_bytes())));
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut started = false;
        while std::time::Instant::now() < deadline {
            if !started {
                if let Some(lock) = claim_start(&protocol::daemon_lock_path(&socket)) {
                    let fd = lock.as_raw_fd();
                    let dir = log_dir(&call.0.file);
                    std::fs::create_dir_all(&dir).map_err(|e| format!("native daemon log directory: {e}"))?;
                    let log = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(dir.join("daemon.log"))
                        .map_err(|e| format!("native daemon log: {e}"))?;
                    let mut command = std::process::Command::new(core);
                    command
                        .arg("daemon")
                        .envs(daemon_alloc_env())
                        .env("LUST_DAEMON_LOCK_FD", fd.to_string())
                        .env("LUST_DAEMON_SOCKET", socket)
                        .env_remove("LUST_DAEMON_PREWARM")
                        .env_remove("LUST_DAEMON_PREWARM_AFTER")
                        .stdin(std::process::Stdio::null())
                        .stdout(log.try_clone().map_err(|e| format!("native daemon log handle: {e}"))?)
                        .stderr(log);
                    // Start the unchanged daemon once. The first request compiles in that
                    // retained session instead of compiling cold and prewarming it again.
                    let directory_fd = address.inherited_fd();
                    if let Some(fd) = directory_fd {
                        command.env("LUST_DAEMON_DIRECTORY_FD", fd.to_string());
                    } else {
                        command.env_remove("LUST_DAEMON_DIRECTORY_FD");
                    }
                    unsafe {
                        command.pre_exec(move || {
                            if setsid() < 0 || fcntl(fd, F_SETFD, 0) < 0 || directory_fd.is_some_and(|fd| fcntl(fd, F_SETFD, 0) < 0) {
                                return Err(std::io::Error::last_os_error());
                            }
                            Ok(())
                        });
                    }
                    command.spawn().map_err(|e| format!("cannot start the native daemon: {e}"))?;
                    started = true;
                }
            }
            if let Some(r) = protocol::request(&socket, &call.0)? {
                return Ok(Some((r.code, r.stdout.into_bytes(), r.stderr.into_bytes())));
            }
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        Err("native daemon did not become ready before the startup deadline".into())
    }
}

fn trace(engine: &str) {
    trace_entry(engine, "exec");
}

fn trace_entry(engine: &str, entry: &str) {
    if let Some(path) = std::env::var_os("LUSTR_TRACE_PATH") {
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(format!("{{\"engine\":\"{engine}\",\"entry\":\"{entry}\"}}\n").as_bytes());
        }
    }
}

fn native_diagnostic(stderr: &[u8]) {
    if let Some(path) = std::env::var_os("LUSTR_NATIVE_DIAGNOSTICS") {
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(stderr);
        }
    }
}

fn main() -> ExitCode {
    let mut arguments = std::env::args_os();
    let program = arguments.next();
    let args: Vec<OsString> = arguments.collect();
    if std::env::var_os("LUST_PROBE_FAKE_DIR").is_some() {
        return incomplete_integration("an externally selected internal compiler-probe mode");
    }
    if args == ["--lustr-version"] {
        println!(
            "lustr-compiler {} (declared Rust source {}; complementary integration incomplete)",
            env!("CARGO_PKG_VERSION"),
            env!("LUSTR_CORE_SOURCE")
        );
        return ExitCode::SUCCESS;
    }
    for name in ["RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"] {
        if std::env::var_os(name).is_some_and(|value| !value.is_empty()) {
            return incomplete_integration("a configured external compiler wrapper");
        }
    }
    let parent = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    if let Some(result) = native_queries::answer(&args, &parent, lust::host_triple()) {
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => incomplete_integration(&error),
        };
    }
    let compiler_mode = program.is_some_and(|a| std::path::Path::new(&a).file_name().is_some_and(|n| n == "rustc"));
    if !compiler_mode && args == ["--lustr-prime-selection-tools"] {
        return match lust::prime_selection_tools(&parent) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("cannot preverify Cargo selection tools: {error}");
                ExitCode::from(2)
            }
        };
    }
    let native_cli = !compiler_mode && lustr_compiler::is_native_command(&args);
    if !compiler_mode && !native_cli && lustr_compiler::cargo_command(&args) {
        return incomplete_integration("Cargo compilation and alternate compiler selection");
    }
    let native = if native_cli {
        args.iter().map(|a| a.to_str().map(str::to_owned)).collect::<Option<Vec<_>>>()
    } else {
        lustr_compiler::native_request(&args).map(|r| r.args)
    };
    let prepared = native.and_then(|args| lust::prepare(&args).map(|call| (args, call)));
    if native_cli && prepared.is_none() {
        // The upstream parser distinguishes real native release compilation
        // from its Cargo/rustc delegation. Never run the latter through Lustr.
        let native = args.iter().map(|a| a.to_str().map(str::to_owned)).collect::<Option<Vec<_>>>();
        if args.first().is_some_and(|arg| arg == "test") || native.as_ref().is_some_and(|args| lust::native_release(args)) {
            trace("lust-native-command");
            let error = Command::new(parent.join("lust-core")).args(&args).exec();
            eprintln!("cannot execute the native release compiler: {error}");
            return ExitCode::from(2);
        }
        return incomplete_integration("this native CLI profile, oracle or argument contract");
    }
    if let Some((mut native, mut call)) = prepared {
        if call.needs_rust_artifact() {
            return incomplete_integration("the requested Rust artifact, target or Cargo layout");
        }
        if let Err(error) = call.validate_selection(&parent, &native) {
            eprintln!("invalid Cargo target selection: {error}");
            return ExitCode::from(2);
        }
        call.bind_native_arguments(&mut native);
        let entry;
        let result = if let Some(response) = match lust::answer(&call, &parent.join("lust-core")) {
            Ok(response) => response,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(3);
            }
        } {
            entry = "daemon";
            Some(response)
        } else if let Some(result) = call
            .library_compatible()
            .then(|| native_link::compile(&parent.join("../lib").join(native_link::NATIVE_LIBRARY), &native))
            .flatten()
        {
            entry = "native-library";
            match result {
                Ok(response) => Some(response),
                Err(error) => {
                    eprintln!("{error}");
                    return ExitCode::from(2);
                }
            }
        } else {
            entry = "upstream-process";
            match owned_child::output(Command::new(parent.join("lust")).args(&native)) {
                Ok(output) if output.status.signal().is_none() => Some((output.status.code().unwrap_or(2), output.stdout, output.stderr)),
                Ok(output) => {
                    // Compiler crashes are failures. Do not turn them into successful fallback builds.
                    let _ = io::stderr().write_all(&output.stderr);
                    propagate_signal(output.status.signal().unwrap());
                }
                Err(error) => {
                    eprintln!("cannot execute native Lust: {error}");
                    return ExitCode::from(2);
                }
            }
        };
        if let Some((code, stdout, stderr)) = result {
            trace_entry(if code == 0 { "lust" } else { "lust-failed" }, entry);
            native_diagnostic(&stderr);
            io::stdout().write_all(&stdout).expect("cannot write compiler stdout");
            io::stderr().write_all(&stderr).expect("cannot write compiler stderr");
            if code == 0 {
                if let Some(error) = call.run_program() {
                    eprintln!("lustr: cannot execute the compiled program: {error}");
                    return ExitCode::from(2);
                }
            }
            // A native diagnostic or exit status is not a capability result.
            // Preserve it once. No second implementation may retry this work.
            std::process::exit(code);
        }
        eprintln!("lustr: the native compiler returned no result");
        return ExitCode::from(2);
    }
    incomplete_integration("this rustc request or compiler command")
}

fn incomplete_integration(request: &str) -> ExitCode {
    trace("integration-unimplemented");
    eprintln!("lustr: integration is incomplete for {request}; no stock compiler was invoked");
    ExitCode::from(2)
}

fn propagate_signal(signal_number: i32) -> ! {
    unsafe extern "C" {
        fn signal(number: i32, handler: usize) -> usize;
        fn raise(number: i32) -> i32;
    }
    // Linux x86_64 is enforced by the build. Restore SIG_DFL, including for
    // SIGPIPE which Rust normally ignores, then preserve the child's signal.
    unsafe {
        signal(signal_number, 0);
        raise(signal_number);
    }
    std::process::exit(128 + signal_number);
}
