//! Paired measurements of the actual source-built toolchain and matched references.
//! Linux x86-64 is the current mixed implementation's qualification boundary.
//! Successful execution establishes only this corpus; timing never admits a release.
#![forbid(unsafe_code)]

#[path = "../native_evidence.rs"]
mod native_evidence;
#[path = "../order.rs"]
mod order;
#[path = "../stats.rs"]
mod stats;

use nix::{
    errno::Errno,
    sys::{
        prctl::set_child_subreaper,
        signal::{Signal, killpg},
        wait::{WaitPidFlag, WaitStatus, waitpid},
    },
    unistd::Pid,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{DirBuilderExt, FileTypeExt},
        process::{CommandExt, ExitStatusExt},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type R<T> = Result<T, Box<dyn std::error::Error>>;
const PRISTINE: &str = "240e08d91d4aed0299639f6f11fd522912b248f9";
const PRISTINE_TREE: &str = "0ab1b09815802894e6078de0af59273cbdf46cb0";
const RUST_SOURCE: &str = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494";
const TIMEOUT: Duration = Duration::from_secs(180);
const OUTPUT_LIMIT: u64 = 64 * 1024 * 1024;

fn require(condition: bool, message: impl Into<String>) -> R<()> {
    if condition { Ok(()) } else { Err(message.into().into()) }
}
fn digest(path: &Path) -> R<String> {
    let mut file = File::open(path)?;
    let mut h = Sha256::new();
    let mut bytes = [0u8; 65536];
    loop {
        let count = file.read(&mut bytes)?;
        if count == 0 {
            return Ok(h.finalize().iter().map(|byte| format!("{byte:02x}")).collect());
        }
        h.update(&bytes[..count]);
    }
}
fn put(path: &Path, bytes: impl AsRef<[u8]>) -> R<()> {
    fs::create_dir_all(path.parent().ok_or("missing output parent")?)?;
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(bytes.as_ref())?;
    Ok(())
}
fn source_files(directory: &Path, out: &mut BTreeMap<PathBuf, String>) -> R<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        if ty.is_dir() {
            source_files(&path, out)?;
        } else if ty.is_file() {
            out.insert(path, digest(&entry.path())?);
        } else {
            return Err("fixture contains an unexpected link or special file".into());
        }
    }
    Ok(())
}

struct Foreground(Option<Child>);
impl Drop for Foreground {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = killpg(Pid::from_raw(child.id() as i32), Signal::SIGKILL);
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Daemons are started by the real compiler. A private socket directory and
/// child-subreaper ownership let their normal shutdown remain observable.
struct Daemons {
    sockets: PathBuf,
    done: bool,
}
impl Daemons {
    fn new() -> R<Self> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let sockets = std::env::temp_dir().join(format!("lmb-{}-{stamp:x}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&sockets)?;
        require(sockets.as_os_str().as_encoded_bytes().len() < 65, "temporary socket directory is too long")?;
        Ok(Self { sockets, done: false })
    }
    fn stop(&mut self) -> R<usize> {
        for entry in fs::read_dir(&self.sockets)? {
            let entry = entry?;
            if entry.file_type()?.is_socket() {
                // Upstream's documented, controlled stop request. Never remove
                // lock files while the corresponding process could be alive.
                fs::remove_file(entry.path())?;
            }
        }
        let start = Instant::now();
        let mut count = 0;
        let mut failure = None;
        loop {
            match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(_, 0)) => count += 1,
                Ok(WaitStatus::StillAlive) => {
                    require(start.elapsed() < Duration::from_secs(35), "owned daemon did not shut down within 35 seconds")?;
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(status) => {
                    count += 1;
                    failure = Some(format!("owned daemon ended abnormally: {status:?}"));
                }
                Err(Errno::ECHILD) => break,
                Err(Errno::EINTR) => (),
                Err(error) => return Err(error.into()),
            }
        }
        self.done = true;
        fs::remove_dir_all(&self.sockets)?;
        if let Some(failure) = failure {
            return Err(failure.into());
        }
        Ok(count)
    }
}
impl Drop for Daemons {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.stop();
        }
    }
}

struct Outcome {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}
struct Campaign {
    root: PathBuf,
    raw: File,
    events: Vec<Value>,
    serial: usize,
    pairs: usize,
    pristine: PathBuf,
    built: PathBuf,
    toolchain: PathBuf,
    path: OsString,
    rng: u64,
}
impl Campaign {
    fn event(&mut self, value: Value) -> R<()> {
        writeln!(self.raw, "{value}")?;
        self.raw.flush()?;
        self.events.push(value);
        Ok(())
    }
    fn command(&self, executable: &Path, cwd: &Path, state: &Path, socket: &Path) -> R<Command> {
        for directory in [state, &state.join("tmp"), &state.join("xdg")] {
            fs::create_dir_all(directory)?;
        }
        let mut command = Command::new(executable);
        command
            .current_dir(cwd)
            .env_clear()
            .env("PATH", &self.path)
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .env("TZ", "UTC")
            .env("TMPDIR", state.join("tmp"))
            .env("XDG_CACHE_HOME", state.join("xdg"))
            .env("LUST_CACHE_DIR", state.join("native-cache"))
            .env("LUST_DAEMON_SOCKET", socket)
            .env("LUST_DAEMON_IDLE", "120s")
            .env("LUSTR_MIXED_CACHE_DIR", state.join("mixed-cache"))
            .env("CARGO_BUILD_JOBS", "2")
            .env("CARGO_NET_OFFLINE", "true")
            .env("CARGO_TERM_COLOR", "never")
            .env("RUSTDOC", self.toolchain.join("bin/rustdoc"))
            .env("RUSTC", self.toolchain.join("bin/rustc-reference"))
            .env("CARGO", self.toolchain.join("bin/cargo"));
        // Rustup is not the compiler under test. The fixture's entire dependency
        // graph is local, but Cargo may still need its ordinary home directory.
        for name in ["HOME", "CARGO_HOME", "RUSTUP_HOME"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        Ok(command)
    }
    fn run(
        &mut self,
        family: &str,
        lane: &str,
        phase: &str,
        sample: Option<usize>,
        source: &Path,
        expected: i32,
        command: &mut Command,
    ) -> R<Outcome> {
        let serial = self.serial;
        self.serial += 1;
        let stem = format!("{serial:05}-{family}-{lane}-{phase}");
        let stdout = self.root.join(format!("{stem}.stdout"));
        let stderr = self.root.join(format!("{stem}.stderr"));
        let mut inputs = BTreeMap::new();
        if source.is_dir() {
            source_files(source, &mut inputs)?;
        } else {
            inputs.insert(source.to_path_buf(), digest(source)?);
        }
        let argv: Vec<_> = std::iter::once(command.get_program()).chain(command.get_args()).map(|s| s.to_owned()).collect();
        let start = Instant::now();
        let child = command
            .stdin(Stdio::null())
            .stdout(OpenOptions::new().create_new(true).write(true).open(&stdout)?)
            .stderr(OpenOptions::new().create_new(true).write(true).open(&stderr)?)
            .process_group(0)
            .spawn()?;
        let mut foreground = Foreground(Some(child));
        let mut timed_out = false;
        let status = loop {
            let child = foreground.0.as_mut().unwrap();
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if start.elapsed() >= TIMEOUT {
                timed_out = true;
                let _ = killpg(Pid::from_raw(child.id() as i32), Signal::SIGKILL);
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_micros(100));
        };
        let seconds = start.elapsed().as_secs_f64();
        foreground.0.take();
        let mut after = BTreeMap::new();
        if source.is_dir() {
            source_files(source, &mut after)?;
        } else {
            after.insert(source.to_path_buf(), digest(source)?);
        }
        let stdout_bytes = fs::metadata(&stdout)?.len();
        let stderr_bytes = fs::metadata(&stderr)?.len();
        self.event(json!({
            "kind":"command", "family":family,"lane":lane,"phase":phase,"sample":sample,
            "argv":argv,"cwd":command.get_current_dir(),"source_sha256":inputs,"source_after_sha256":after,
            "seconds":seconds,"expected_exit":expected,"exit":status.code(),"signal":status.signal(),"timed_out":timed_out,
            "stdout":stdout.file_name(),"stderr":stderr.file_name(),"stdout_sha256":digest(&stdout)?,"stderr_sha256":digest(&stderr)?,
            "stdout_bytes":stdout_bytes,"stderr_bytes":stderr_bytes
        }))?;
        require(phase == "generate-lockfile" || inputs == after, "command changed the measured application's source")?;
        require(
            !timed_out && status.code() == Some(expected),
            format!("{family}/{lane}/{phase}: wrong exit {status}; raw output retained"),
        )?;
        require(stdout_bytes + stderr_bytes <= OUTPUT_LIMIT, "command output exceeds measurement limit")?;
        Ok(Outcome { stdout: fs::read(stdout)?, stderr: fs::read(stderr)? })
    }
    fn native(&self, lane: usize, source: &Path, state: &Path, sockets: &Daemons, operation: &str, cold: bool) -> R<Command> {
        let exe = match lane {
            0 => self.pristine.clone(),
            1 => self.toolchain.join("bin/lust"),
            2 => self.toolchain.join("bin/lustr"),
            _ => return Err("unknown native lane".into()),
        };
        let mut command = self.command(&exe, source.parent().unwrap(), state, &sockets.sockets.join(format!("n{lane}.sock")))?;
        command.arg(operation).arg(source);
        if operation == "build" {
            command.arg("-o").arg(state.join("program"));
        } else if operation == "test" {
            command.arg("--json");
        }
        if cold {
            command.env("LUST_NO_DAEMON", "1").env("LUST_NO_CACHE", "1");
        }
        Ok(command)
    }
    fn cargo(&self, lane: usize, source: &Path, state: &Path, sockets: &Daemons, operation: &str) -> R<Command> {
        let exe = self.toolchain.join(if lane == 0 { "bin/cargo" } else { "bin/lustr" });
        let mut command = self.command(&exe, source, state, &sockets.sockets.join(format!("c{lane}.sock")))?;
        command
            .args([operation, "--locked", "--offline", "--jobs", "2", "-p", "mixed-app", "--bin", "mixed-app", "--target-dir"])
            .arg(state.join("target"))
            .arg("--config")
            .arg(format!("build.build-dir={:?}", state.join("intermediates")));
        if operation == "test" {
            command.args(["--", "--test-threads=1"]);
        }
        Ok(command)
    }
    fn runtime(&mut self, family: &str, lane: &str, phase: &str, source: &Path, state: &Path, program: &Path, expected: &[u8]) -> R<()> {
        let mut command = self.command(program, source.parent().unwrap(), state, &state.join("unused.sock"))?;
        let result = self.run(family, lane, &format!("{phase}-runtime"), None, source, 0, &mut command)?;
        require(
            result.stdout == expected && result.stderr.is_empty(),
            format!("{family}/{lane}/{phase}: runtime differs from the fixed oracle"),
        )
    }
    fn fixture(&mut self, root: &Path, sockets: &Daemons) -> R<PathBuf> {
        let source = root.join("source");
        put(&source.join("Cargo.toml"), "[workspace]\nmembers=['app','fixture-macro','portable']\nresolver='3'\n[profile.dev]\ndebug=0\n")?;
        put(
            &source.join("app/Cargo.toml"),
            "[package]\nname='mixed-app'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nfixture-macro={path='../fixture-macro'}\nportable={path='../portable'}\n",
        )?;
        put(&source.join("app/src/main.rs"), mixed_source("17", false))?;
        put(
            &source.join("fixture-macro/Cargo.toml"),
            "[package]\nname='fixture-macro'\nversion='0.1.0'\nedition='2024'\n[lib]\nproc-macro=true\n",
        )?;
        put(
            &source.join("fixture-macro/src/lib.rs"),
            "use proc_macro::TokenStream;\n#[proc_macro]\npub fn general(_:TokenStream)->TokenStream{\"async fn general(x:u64)->u64{kernel(x) as u64}\".parse().unwrap()}\n",
        )?;
        put(&source.join("portable/Cargo.toml"), "[package]\nname='portable'\nversion='0.1.0'\nedition='2024'\n")?;
        put(
            &source.join("portable/src/lib.rs"),
            "#![no_std]\nextern crate alloc;\npub fn adjustment()->u64{alloc::vec![0u64].iter().sum()}\n",
        )?;
        let mut command =
            self.command(&self.toolchain.join("bin/cargo"), &source, &root.join("lockfile-state"), &sockets.sockets.join("unused.sock"))?;
        command.args(["generate-lockfile", "--offline"]);
        self.run("cargo", "reference", "generate-lockfile", None, &source, 0, &mut command)?;
        Ok(source)
    }
    fn native_probes(&mut self) -> R<()> {
        let root = self.root.join("native-probes");
        let source = root.join("source/main.rs");
        put(&source, native_source("17", false))?;
        let mut sockets = Daemons::new()?;
        for lane in 0..3 {
            let state = root.join(format!("lane{lane}"));
            fs::write(&source, native_source("17", false))?;
            for phase in ["prime", "warm", "reuse"] {
                let mut command = self.native(lane, &source, &state, &sockets, "check", false)?;
                command.env("LUST_STATS", "1").arg("--timings");
                if lane == 2 {
                    command.env("LUSTR_TRACE_PATH", root.join("entry.trace"));
                }
                let result = self.run("native", native_lane(lane), phase, None, &source, 0, &mut command)?;
                if phase == "prime" {
                    wait_socket(&sockets.sockets.join(format!("n{lane}.sock")))?;
                }
                if phase == "reuse" {
                    let text = String::from_utf8_lossy(&result.stderr);
                    let route = native_evidence::retained_check(&text)
                        .map_err(|reason| format!("{}: invalid native retained check evidence ({reason}): {text}", native_lane(lane)))?;
                    self.event(json!({"kind":"native-retained-check","lane":native_lane(lane),"route":route}))?;
                }
            }
            fs::write(&source, native_source("19", false))?;
            let mut command = self.native(lane, &source, &state, &sockets, "check", false)?;
            command.env("LUST_STATS", "1").arg("--timings");
            let result = self.run("native", native_lane(lane), "body-probe", None, &source, 0, &mut command)?;
            let text = String::from_utf8_lossy(&result.stderr);
            require(
                !text.contains("mode=full") && (text.contains("mode=bodies") || text.contains("path=patch") || text.contains("why=bodies")),
                format!("{}: native body edit did not use a retained frontend route: {text}", native_lane(lane)),
            )?;
        }
        let trace = fs::read_to_string(root.join("entry.trace"))?;
        require(
            trace.lines().any(|line| serde_json::from_str::<Value>(line).is_ok_and(|v| v["engine"] == "lust" && v["entry"] == "daemon")),
            "lustr supported-work probe did not use the native daemon",
        )?;
        let count = sockets.stop()?;
        require(count >= 3, "missing owned native daemons in retained probes")?;
        self.event(json!({"kind":"daemon-exits","scope":"native-probes","successful_exits":count}))
    }
    fn mixed_probes(&mut self) -> R<()> {
        let root = self.root.join("mixed-probes");
        let mut sockets = Daemons::new()?;
        let source = self.fixture(&root, &sockets)?;
        let state = root.join("lane1");
        let trace = root.join("mixed.trace");
        let app = source.join("app/src/main.rs");
        for (phase, operation, literal, expected, evidence) in [
            ("cold", "build", "17", 0, "native-definitions="),
            ("body", "build", "19", 0, "reused-rust-objects=1 full-analysis=0"),
            ("check", "check", "17", 0, "native-checked-definitions="),
            ("check-body", "check", "19", 0, "reused-check-context=1 full-analysis=0"),
            ("rejection", "build", "true", 101, ""),
        ] {
            fs::write(&app, mixed_source(literal, false))?;
            fs::write(&trace, "")?;
            let mut command = self.cargo(1, &source, &state, &sockets, operation)?;
            command.env("LUSTR_MIXED_TRACE", &trace).arg("-vv");
            self.run("cargo", "mixed", &format!("probe-{phase}"), None, &source, expected, &mut command)?;
            let text = fs::read_to_string(&trace)?;
            fs::copy(&trace, root.join(format!("{phase}.trace")))?;
            require(
                text.contains(evidence),
                format!("mixed {phase} did not execute its required native/retained route; inspect {}", trace.display()),
            )?;
            if phase == "cold" || phase == "body" {
                let expected = if phase == "cold" { b"59\n" } else { b"57\n" };
                self.runtime("cargo", "mixed", phase, &app, &state, &state.join("target/debug/mixed-app"), expected)?;
            }
        }
        let count = sockets.stop()?;
        require(count > 0, "mixed probe did not start an observable native daemon")?;
        self.event(json!({"kind":"daemon-exits","scope":"mixed-probes","successful_exits":count}))
    }
    fn native_samples(&mut self) -> R<()> {
        let schedules = order::rounds(3, self.pairs, &mut self.rng);
        for (sample, lanes) in schedules.into_iter().enumerate() {
            let root = self.root.join(format!("native-{sample:03}"));
            let source = root.join("source/main.rs");
            put(&source, native_source("17", false))?;
            let mut sockets = Daemons::new()?;
            self.event(json!({"kind":"order","family":"native","sample":sample,"lanes":lanes}))?;
            for (phase, operation, literal, signature, expected, cold) in [
                ("cold", "build", "17", false, 0, true),
                ("no-op", "build", "17", false, 0, false),
                ("body-edit", "build", "19", false, 0, false),
                ("signature-edit", "build", "17", true, 0, false),
                ("check", "check", "17", false, 0, false),
                ("check-no-op", "check", "17", false, 0, false),
                ("check-body-edit", "check", "19", false, 0, false),
                ("rejection", "check", "true", false, 1, false),
                ("compile-plus-test", "test", "17", false, 0, false),
            ] {
                if phase != "no-op" && phase != "check-no-op" {
                    fs::write(&source, native_source(literal, signature))?;
                }
                for &lane in &lanes {
                    let state = root.join(format!("lane{lane}"));
                    if phase == "no-op" {
                        for prime in 0..2 {
                            let mut command = self.native(lane, &source, &state, &sockets, "build", false)?;
                            self.run("native", native_lane(lane), "excluded-prime", None, &source, 0, &mut command)?;
                            if prime == 0 {
                                wait_socket(&sockets.sockets.join(format!("n{lane}.sock")))?;
                            }
                        }
                    }
                    let mut command = self.native(lane, &source, &state, &sockets, operation, cold)?;
                    let result = self.run("native", native_lane(lane), phase, Some(sample), &source, expected, &mut command)?;
                    if operation == "build" {
                        let expected = if literal == "19" { b"57\n" } else { b"59\n" };
                        self.runtime("native", native_lane(lane), phase, &source, &state, &state.join("program"), expected)?;
                    } else if operation == "test" {
                        let summaries: Vec<Value> = result
                            .stdout
                            .split(|b| *b == b'\n')
                            .filter_map(|line| serde_json::from_slice(line).ok())
                            .filter(|v: &Value| v["kind"] == "summary")
                            .collect();
                        require(
                            summaries.len() == 1
                                && summaries[0]["build"] == "ok"
                                && summaries[0]["passed"] == 1
                                && summaries[0]["failed"] == 0
                                && summaries[0]["timed_out"] == 0,
                            "native compile-plus-test did not execute the required test",
                        )?;
                    }
                }
            }
            let count = sockets.stop()?;
            require(count >= 3, "native sample lost its retained daemon coverage")?;
            self.event(json!({"kind":"daemon-exits","scope":format!("native-{sample}"),"successful_exits":count}))?;
        }
        Ok(())
    }
    fn mixed_samples(&mut self) -> R<()> {
        let schedules = order::rounds(2, self.pairs, &mut self.rng);
        for (sample, lanes) in schedules.into_iter().enumerate() {
            let root = self.root.join(format!("cargo-{sample:03}"));
            let mut sockets = Daemons::new()?;
            let source = self.fixture(&root, &sockets)?;
            let app = source.join("app/src/main.rs");
            self.event(json!({"kind":"order","family":"cargo","sample":sample,"lanes":lanes}))?;
            for (phase, operation, literal, signature, expected) in [
                ("cold", "build", "17", false, 0),
                ("no-op", "build", "17", false, 0),
                ("body-edit", "build", "19", false, 0),
                ("signature-edit", "build", "17", true, 0),
                ("check", "check", "17", false, 0),
                ("check-no-op", "check", "17", false, 0),
                ("check-body-edit", "check", "19", false, 0),
                ("rejection", "build", "true", false, 101),
                ("compile-plus-test", "test", "17", false, 0),
            ] {
                if phase != "no-op" && phase != "check-no-op" {
                    fs::write(&app, mixed_source(literal, signature))?;
                }
                for &lane in &lanes {
                    let state = root.join(format!("lane{lane}"));
                    let mut command = self.cargo(lane, &source, &state, &sockets, operation)?;
                    let result = self.run("cargo", cargo_lane(lane), phase, Some(sample), &source, expected, &mut command)?;
                    if operation == "build" && expected == 0 {
                        let expected = if literal == "19" { b"57\n" } else { b"59\n" };
                        self.runtime("cargo", cargo_lane(lane), phase, &app, &state, &state.join("target/debug/mixed-app"), expected)?;
                    } else if operation == "test" {
                        require(
                            String::from_utf8_lossy(&result.stdout).contains("1 passed; 0 failed"),
                            "Cargo compile-plus-test did not execute the required test",
                        )?;
                    }
                }
            }
            let count = sockets.stop()?;
            require(count > 0, "mixed sample did not observe its native daemon")?;
            self.event(json!({"kind":"daemon-exits","scope":format!("cargo-{sample}"),"successful_exits":count}))?;
        }
        Ok(())
    }
    fn summaries(&self) -> R<Value> {
        let mut groups = BTreeMap::<(String, String), BTreeMap<String, Vec<(usize, f64)>>>::new();
        for event in &self.events {
            if event["kind"] == "command"
                && let Some(sample) = event["sample"].as_u64()
            {
                groups
                    .entry((event["family"].as_str().unwrap().into(), event["phase"].as_str().unwrap().into()))
                    .or_default()
                    .entry(event["lane"].as_str().unwrap().into())
                    .or_default()
                    .push((sample as usize, event["seconds"].as_f64().unwrap()));
            }
        }
        require(groups.len() == 18, "missing required native/Cargo measurement modes")?;
        let mut rows = Vec::new();
        for ((family, phase), mut lanes) in groups {
            let expected = if family == "native" { vec!["pristine", "integrated", "entry"] } else { vec!["reference", "mixed"] };
            require(lanes.len() == expected.len(), "missing comparison lane")?;
            for name in &expected {
                let samples = lanes.get_mut(*name).ok_or("missing required lane")?;
                samples.sort_by_key(|p| p.0);
                require(
                    samples.len() == self.pairs && samples.iter().enumerate().all(|(i, p)| p.0 == i),
                    "incomplete or duplicate paired samples",
                )?;
            }
            for name in &expected[1..] {
                let pairs: Vec<_> = lanes[expected[0]].iter().zip(&lanes[*name]).map(|(a, b)| (a.1, b.1)).collect();
                let summary = stats::summarize(&pairs, 10_000, 0x4c55535452)?;
                let observation = if summary.bootstrap_95[1] < 1.0 {
                    "slower-on-this-corpus"
                } else if summary.bootstrap_95[0] > 1.0 {
                    "faster-on-this-corpus"
                } else {
                    "inconclusive"
                };
                rows.push(json!({"family":family,"phase":phase,"baseline":expected[0],"candidate":name,"ratio_direction":"baseline time / candidate time; greater than 1 is faster","observation":observation,"analysis":summary}));
            }
        }
        Ok(json!(rows))
    }
}

fn native_lane(lane: usize) -> &'static str {
    ["pristine", "integrated", "entry"][lane]
}
fn cargo_lane(lane: usize) -> &'static str {
    ["reference", "mixed"][lane]
}
fn wait_socket(path: &Path) -> R<()> {
    let start = Instant::now();
    while !fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_socket()) {
        require(start.elapsed() < Duration::from_secs(5), "native daemon socket did not become ready")?;
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}
fn native_source(literal: &str, signature: bool) -> String {
    let ty = if signature { "u32" } else { "u64" };
    format!(
        "fn kernel(x:{ty})->{ty}{{x ^ {literal}}}\nfn main(){{println!(\"{{}}\",kernel(42));}}\n#[test]\nfn scalar_contract(){{assert_eq!(kernel(42),59);}}\n"
    )
}
fn mixed_source(literal: &str, signature: bool) -> String {
    let (ty, body) = if signature { ("i64", format!("(x ^ {literal}) as i64")) } else { ("u64", format!("x ^ {literal}")) };
    format!(
        "use fixture_macro::general;\ngeneral!();\nfn kernel(x:u64)->{ty}{{{body}}}\nfn main(){{let w=std::task::Waker::noop();let mut f=std::pin::pin!(general(42+portable::adjustment()));let mut cx=std::task::Context::from_waker(w);match std::future::Future::poll(f.as_mut(),&mut cx){{std::task::Poll::Ready(v)=>println!(\"{{v}}\"),std::task::Poll::Pending=>panic!(\"unexpected pending\")}}}}\n#[test]\nfn scalar_contract(){{assert_eq!(kernel(42),59);}}\n"
    )
}
fn query(command: &mut Command) -> R<String> {
    let output = command.output()?;
    require(output.status.success(), format!("identity query failed: {}", String::from_utf8_lossy(&output.stderr)))?;
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

fn main() -> R<()> {
    require(cfg!(all(target_os = "linux", target_arch = "x86_64")), "current mixed qualification requires Linux x86-64")?;
    let mut args = std::env::args_os().skip(1);
    let pristine_source =
        PathBuf::from(args.next().ok_or("PRISTINE_SOURCE PRISTINE_LUST SOURCE_BUILT_OUTPUT NEW_OUTPUT [PAIRS]")?).canonicalize()?;
    let pristine = PathBuf::from(args.next().ok_or("missing pristine compiler")?).canonicalize()?;
    let built = PathBuf::from(args.next().ok_or("missing source-built output")?).canonicalize()?;
    let root = PathBuf::from(args.next().ok_or("missing new output directory")?);
    let pairs = args
        .next()
        .map(|v| v.into_string().map_err(|_| "non-UTF8 sample count").and_then(|v| v.parse::<usize>().map_err(|_| "invalid sample count")))
        .transpose()?
        .unwrap_or(12);
    require(args.next().is_none() && pairs >= 12 && pairs <= 96 && pairs % 6 == 0, "pairs must be a multiple of six between 12 and 96")?;
    require(root.is_absolute() && !root.exists(), "output must be a new absolute path")?;
    let head = query(Command::new("git").arg("-C").arg(&pristine_source).args(["rev-parse", "HEAD"]))?;
    let tree = query(Command::new("git").arg("-C").arg(&pristine_source).args(["rev-parse", "HEAD^{tree}"]))?;
    require(head == PRISTINE && tree == PRISTINE_TREE, "pristine source does not match the pinned Lust reference")?;
    require(
        query(Command::new("git").arg("-C").arg(&pristine_source).args(["status", "--porcelain", "--untracked-files=no"]))?.is_empty(),
        "pristine tracked source is modified",
    )?;
    require(
        fs::read_to_string(built.join("BUILD-PROFILE.txt"))?.contains("Cargo artifact profile: release\n"),
        "timing requires a source-built --release composition",
    )?;
    let toolchain = built.join("toolchain").canonicalize()?;
    let rust = query(Command::new(toolchain.join("bin/rustc-reference")).arg("-vV"))?;
    require(
        rust.lines().any(|line| line == format!("commit-hash: {RUST_SOURCE}")),
        "reference Rust SDK does not match the integration pin",
    )?;
    fs::DirBuilder::new().mode(0o700).create(&root)?;
    let root = root.canonicalize()?;
    let mut path = OsString::from(toolchain.join("bin"));
    path.push(":/usr/local/bin:/usr/bin:/bin");
    let mut campaign = Campaign {
        raw: OpenOptions::new().create_new(true).write(true).open(root.join("raw.jsonl"))?,
        root,
        events: Vec::new(),
        serial: 0,
        pairs,
        pristine,
        built,
        toolchain,
        path,
        rng: 0x4c55535452,
    };
    let mut identities = BTreeMap::new();
    for file in [
        std::env::current_exe()?,
        campaign.pristine.clone(),
        campaign.pristine.with_file_name("lust-core"),
        campaign.toolchain.join("bin/lustr"),
        campaign.toolchain.join("bin/lust"),
        campaign.toolchain.join("bin/lust-core"),
        campaign.toolchain.join("bin/rustc-full"),
        campaign.toolchain.join("bin/rustc-reference"),
        campaign.toolchain.join("bin/cargo"),
        campaign.toolchain.join("bin/cargo-lustr"),
        campaign.toolchain.join("lib/liblustr_native_core.so"),
        campaign.built.join("BUILD-PROFILE.txt"),
        campaign.built.join("COMPONENTS.txt"),
        campaign.built.join("NATIVE-BUILD-RECEIPT.txt"),
    ] {
        identities.insert(file.clone(), digest(&file)?);
    }
    fs::copy(campaign.built.join("BUILD-PROFILE.txt"), campaign.root.join("BUILD-PROFILE.txt"))?;
    fs::copy(campaign.built.join("COMPONENTS.txt"), campaign.root.join("COMPONENTS.txt"))?;
    fs::copy(campaign.built.join("NATIVE-BUILD-RECEIPT.txt"), campaign.root.join("NATIVE-BUILD-RECEIPT.txt"))?;
    let mut report = json!({
        "schema":1,"status":"failed","performance_admission":"not granted by this runner","pairs":pairs,
        "pristine_revision":head,"pristine_tree":tree,"pristine_source":pristine_source,"compiler_sha256":identities,"rust_reference_version":rust,
        "order_method":order::METHOD,"seed":0x4c55535452u64,"summaries":null,
        "scope":"one native scalar corpus and one real Cargo/proc-macro/async/no_std+alloc graph; not full language, all workloads, all profiles or all platforms",
        "profiles":"owned compiler components: manifest release profiles; fixture applications: dev opt0/debug0, normal unwind/checks; full Rust reference is the matching imported SDK",
        "cold_scope":"fresh per-pair Cargo target, intermediate, mixed, native and XDG runtime cache directories; native cold disables daemon/disk compiler caches; OS page caches are not flushed",
        "warm_scope":"same-lane retained caches and original native daemons; untimed probes separately require real retained checks/body edits; Cargo no-op is not a compiler request",
        "timing_scope":"command wall time through observed child exit, 100us polling; tracing/stats disabled for all measured commands; runtime assertions after builds are outside compile-only timing; compile-plus-test executes actual tests",
        "inference_limits":"shared-host paired observations; bootstrap assumes independent pairs; exploratory per-mode comparisons; pristine binary build recipe must accompany this source/binary identity record"
    });
    campaign.event(json!({"kind":"identities","report":report}))?;
    let result = (|| -> R<Value> {
        set_child_subreaper(true)?;
        campaign.native_probes()?;
        campaign.mixed_probes()?;
        campaign.native_samples()?;
        campaign.mixed_samples()?;
        campaign.summaries()
    })();
    match &result {
        Ok(summaries) => {
            report["status"] = json!("completed");
            report["summaries"] = summaries.clone();
        }
        Err(error) => report["error"] = json!(error.to_string()),
    }
    report["records"] = json!(campaign.events);
    report["raw_sha256"] = json!(digest(&campaign.root.join("raw.jsonl"))?);
    fs::write(campaign.root.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    campaign.raw.sync_all()?;
    result?;
    println!(
        "Completed bounded native and mixed measurements; assess per-mode ratios and release requirements in {}",
        campaign.root.join("report.json").display()
    );
    Ok(())
}
