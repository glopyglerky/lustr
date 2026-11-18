use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Command, ExitCode};
mod native_expansion;
mod native_link;
mod owned_child;

#[allow(dead_code)]
mod lust {
    include!("../compiler/native/src/client.rs");

    mod full {
        include!("full.rs");
    }

    pub struct Call(protocol::Request, Option<full::Selection>);

    pub fn prime_selection_tools(parent: &Path) -> std::io::Result<()> {
        full::prime_selection_tools(parent)
    }

    impl Call {
        pub fn expanded(&self, identity: &str) -> Self {
            let mut request = self.0.clone();
            request.session.krate.package_id.push_str("#lustr-library-v1:");
            request.session.krate.package_id.push_str(identity);
            Call(request, None)
        }
        pub fn needs_rust_artifact(&self) -> bool {
            matches!(self.0.cmd, protocol::Cmd::Build) && self.0.session.krate.target_kind == "lib"
                || !(self.0.session.target_triple.is_empty() || self.0.session.target_triple == protocol::host_triple())
                || self.0.session.krate.manifest_path.is_some()
                    && std::env::var("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT").is_ok_and(|v| v != "1")
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
                let selection = full::select(&self.0, parent, args)?;
                // The unchanged native resolver selects within the supplied
                // manifest. Bind it to the validated member and target too.
                self.0.session.krate.manifest_path = Some(selection.manifest.clone());
                self.0.session.krate.target_name = selection.name.clone();
                self.1 = Some(selection);
            }
            Ok(())
        }
        pub fn full(&self, parent: &Path) -> std::io::Result<std::process::ExitStatus> {
            full::run(&self.0, parent, self.1.as_ref())
        }
        pub fn bind_native_arguments(&self, args: &mut Vec<String>) {
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
        pub fn full_entry(&self) -> &'static str {
            if self.0.session.krate.manifest_path.is_some() {
                "cargo"
            } else if matches!(self.0.cmd, protocol::Cmd::Check) {
                "owned-rustc-check"
            } else {
                "exec"
            }
        }
    }

    pub fn prepare(args: &[String]) -> Option<Call> {
        match protocol::parse_args(args) {
            Some(protocol::Invocation::Compile { mut req, run: false, .. }) => {
                if req.session.krate.manifest_path.is_some() {
                    if let Some(env) = req.client.env.as_mut() {
                        if !env.iter().any(|(k, _)| k == "__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT") {
                            env.push(("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT".into(), "1".into()));
                        }
                    }
                }
                Some(Call(req, None))
            }
            _ => None,
        }
    }

    pub fn answer(call: &Call, core: &Path) -> Option<(i32, Vec<u8>, Vec<u8>)> {
        if env_on("LUST_NO_DAEMON") || protocol::source_override().is_some() {
            return None;
        }
        let socket = protocol::socket_path();
        answer_at(call, core, &socket)
    }

    pub fn expanded_answer(call: &Call, parent: &Path) -> Option<(i32, Vec<u8>, Vec<u8>)> {
        if env_on("LUST_NO_DAEMON") {
            return None;
        }
        let sealed = crate::native_expansion::sealed(parent, &protocol::socket_path())?;
        answer_at(&call.expanded(&sealed.identity), &sealed.core, &sealed.socket)
    }

    fn answer_at(call: &Call, core: &Path, socket: &Path) -> Option<(i32, Vec<u8>, Vec<u8>)> {
        let address = protocol::host_runtime::DaemonSocket::resolve(socket).ok()?;
        let socket = address.path();
        if let Some(r) = protocol::request(&socket, &call.0) {
            return Some((r.code, r.stdout.into_bytes(), r.stderr.into_bytes()));
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut started = false;
        while std::time::Instant::now() < deadline {
            if !started {
                if let Some(lock) = claim_start(&protocol::daemon_lock_path(&socket)) {
                    let fd = lock.as_raw_fd();
                    let dir = log_dir(&call.0.file);
                    std::fs::create_dir_all(&dir).ok()?;
                    let log = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("daemon.log")).ok()?;
                    let mut command = std::process::Command::new(core);
                    command
                        .arg("daemon")
                        .envs(daemon_alloc_env())
                        .env("LUST_DAEMON_LOCK_FD", fd.to_string())
                        .env("LUST_DAEMON_SOCKET", socket)
                        .env("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "1")
                        .env_remove("LUST_DAEMON_PREWARM")
                        .env_remove("LUST_DAEMON_PREWARM_AFTER")
                        .stdin(std::process::Stdio::null())
                        .stdout(log.try_clone().ok()?)
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
                    command.spawn().ok()?;
                    started = true;
                }
            }
            if let Some(r) = protocol::request(&socket, &call.0) {
                return Some((r.code, r.stdout.into_bytes(), r.stderr.into_bytes()));
            }
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        None
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
    if args == ["--lustr-version"] {
        println!("lustr-compiler {} (full core {})", env!("CARGO_PKG_VERSION"), env!("LUSTR_CORE_SOURCE"));
        return ExitCode::SUCCESS;
    }
    let parent = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
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
        // Use the same coordinator as both Cargo extensions without loading an
        // intermediate executable. Its Unix exec and exit-code handling still
        // own the Cargo invocation, including explicit engines and plan output.
        std::process::exit(cargo_lust::run_cli_with_default_root(&args, parent.parent().unwrap()));
    }
    let native = if native_cli {
        args.iter().map(|a| a.to_str().map(str::to_owned)).collect::<Option<Vec<_>>>()
    } else {
        lustr_compiler::native_request(&args).map(|r| r.args)
    };
    let prepared = native.and_then(|args| lust::prepare(&args).map(|call| (args, call)));
    if native_cli && prepared.is_none() {
        trace("upstream-lust-command");
        let error = Command::new(parent.join("lust")).args(&args).env("RUSTC", parent.join("rustc-full")).exec();
        eprintln!("cannot execute the upstream Lust command: {error}");
        return ExitCode::from(2);
    }
    if let Some((mut native, mut call)) = prepared {
        if let Err(error) = call.validate_selection(&parent, &native) {
            eprintln!("invalid Cargo target selection: {error}");
            return ExitCode::from(2);
        }
        call.bind_native_arguments(&mut native);
        let mut entry = "unattempted";
        let mut result = if call.needs_rust_artifact() {
            None
        } else if let Some(response) = lust::answer(&call, &parent.join("lust-core")) {
            entry = "daemon";
            Some(response)
        } else if let Some(result) =
            call.library_compatible().then(|| native_link::compile(&parent.join("../lib").join(native_link::NATIVE_LIBRARY), &native)).flatten()
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
            match owned_child::output(
                Command::new(parent.join("lust")).args(&native).env("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "1"),
            ) {
                Ok(output) if output.status.signal().is_none() => Some((output.status.code().unwrap_or(2), output.stdout, output.stderr)),
                Ok(output) => {
                    // Compiler crashes are failures. Do not turn them into successful fallback builds.
                    let _ = io::stderr().write_all(&output.stderr);
                    propagate_signal(output.status.signal().unwrap());
                }
                Err(error) => {
                    eprintln!("cannot execute pristine Lust: {error}");
                    return ExitCode::from(2);
                }
            }
        };
        let mut engine = "lust";
        // A diagnostic is only a retry hint. The unchanged compiler must
        // check the complete program with the explicitly composed library.
        // Never alter source or reuse the pristine daemon/cache for this tree.
        let library = parent.join("../lib/lustr");
        if result.as_ref().is_some_and(|(code, _, stderr)| {
            *code == 1
                && [
                    b"std::env::consts::".as_slice(),
                    b"`From` for `std::path::PathBuf`".as_slice(),
                    b"no method `reserve` on type `std::ffi::OsStr`".as_slice(),
                    b"no method `reserve_exact` on type `std::ffi::OsStr`".as_slice(),
                    b"no method `reserve` on type `std::path::Path`".as_slice(),
                    b"no method `reserve_exact` on type `std::path::Path`".as_slice(),
                ]
                .iter()
                .any(|hint| stderr.windows(hint.len()).any(|s| s == *hint))
        }) && call.expansion_compatible()
            && library.join("std/src/platform.rs").is_file()
        {
            let (_, _, stderr) = result.as_ref().unwrap();
            trace_entry("lust-rejected", entry);
            native_diagnostic(stderr);
            entry = "source-library-extension";
            engine = "lustr-native";
            result = if let Some(answer) = lust::expanded_answer(&call, &parent) {
                entry = "source-library-daemon";
                Some(answer)
            } else {
                match owned_child::output(
                    Command::new(parent.join("lust"))
                        .args(&native)
                        .env("LUST_LIBRARY", &library)
                        .env("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "1"),
                ) {
                    Ok(output) if output.status.signal().is_none() => {
                        Some((output.status.code().unwrap_or(2), output.stdout, output.stderr))
                    }
                    Ok(output) => {
                        let _ = io::stderr().write_all(&output.stderr);
                        propagate_signal(output.status.signal().unwrap());
                    }
                    Err(error) => {
                        eprintln!("cannot execute the native library expansion: {error}");
                        return ExitCode::from(2);
                    }
                }
            };
        }
        #[cfg(feature = "native-inference")]
        if result.as_ref().is_some_and(|(code, _, _)| *code == 1)
            && call.expansion_compatible()
            && native_link::process_compatible()
            && library.join("std/src/platform.rs").is_file()
            && parent.join("lustr-native-worker").is_file()
        {
            let (_, _, stderr) = result.as_ref().unwrap();
            trace_entry(if engine == "lust" { "lust-rejected" } else { "lustr-native-rejected" }, entry);
            native_diagnostic(stderr);
            engine = "lustr-native";
            entry = "native-inference-worker";
            result = match owned_child::output(
                Command::new(parent.join("lustr-native-worker"))
                    .args(&native)
                    .env("LUST_LIBRARY", &library)
                    .env("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "1"),
            ) {
                Ok(output) if output.status.signal().is_none() => Some((output.status.code().unwrap_or(2), output.stdout, output.stderr)),
                Ok(output) => {
                    let _ = io::stderr().write_all(&output.stderr);
                    propagate_signal(output.status.signal().unwrap());
                }
                Err(error) => {
                    eprintln!("cannot execute native inference worker: {error}");
                    return ExitCode::from(2);
                }
            };
        }
        if let Some((code, stdout, stderr)) = result {
            if code == 0 {
                trace_entry(engine, entry);
                io::stdout().write_all(&stdout).expect("cannot write compiler stdout");
                io::stderr().write_all(&stderr).expect("cannot write compiler stderr");
                std::process::exit(0);
            }
            if code != 1 {
                let _ = io::stdout().write_all(&stdout);
                let _ = io::stderr().write_all(&stderr);
                std::process::exit(code);
            }
            trace_entry(if engine == "lust" { "lust-rejected" } else { "lustr-native-rejected" }, entry);
            native_diagnostic(&stderr);
        }
        if native_cli {
            trace_entry("full-rust-core", call.full_entry());
            return match call.full(&parent) {
                Ok(status) => {
                    if let Some(signal) = status.signal() {
                        propagate_signal(signal);
                    }
                    ExitCode::from(status.code().unwrap_or(2) as u8)
                }
                Err(error) => {
                    eprintln!("full Rust compilation failed: {error}");
                    ExitCode::from(2)
                }
            };
        }
    }
    trace("full-rust-core");
    let full_args = native_cli.then(|| lustr_compiler::native_command(&args).expect("recognized native command"));
    let compiler_args = full_args.as_deref().unwrap_or(&args);
    let error = Command::new(parent.join("rustc-full")).args(compiler_args).exec();
    eprintln!("cannot execute the matched full Rust core: {error}");
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
