//! Historical paired measurements of the retired mixed package and references.
//! This explicit Linux x86-64 experiment requires that historical package layout.
//! It is not a product compiler route or current-package qualification. The
//! independent Rust oracle adapter and evidence controls remain useful separately.
#![forbid(unsafe_code)]

#[path = "../daemons.rs"]
mod daemons;
use daemons::Daemons;

#[path = "../native_evidence.rs"]
mod native_evidence;
#[path = "../order.rs"]
mod order;
#[path = "../stats.rs"]
mod stats;

use nix::{
    sys::{
        prctl::set_child_subreaper,
        signal::{Signal, killpg},
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
    time::{Duration, Instant},
};

type R<T> = Result<T, Box<dyn std::error::Error>>;
const PRISTINE: &str = "240e08d91d4aed0299639f6f11fd522912b248f9";
const PRISTINE_TREE: &str = "0ab1b09815802894e6078de0af59273cbdf46cb0";
const RUST_SOURCE: &str = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494";
const TIMEOUT: Duration = Duration::from_secs(180);
const OUTPUT_LIMIT: u64 = 64 * 1024 * 1024;
const RUSTC_FLAGS: &[&str] = &[
    "--edition=2024",
    "--target=x86_64-unknown-linux-gnu",
    "-C",
    "opt-level=0",
    "-C",
    "debuginfo=0",
    "-C",
    "panic=unwind",
    "-C",
    "overflow-checks=on",
    "-C",
    "debug-assertions=on",
];
const TEST_RECEIPT: &str = "lustr benchmark rustc-test: ";

fn rustc_arguments(command: &mut Command, source: &Path, output: &Path, operation: &str) {
    command.arg(source).args(RUSTC_FLAGS);
    command
        .arg(match operation {
            "build" => "--emit=link",
            "check" => "--emit=metadata",
            "test" => "--test",
            _ => unreachable!("fixed native reference operation"),
        })
        .arg("-o")
        .arg(output);
}

fn child_status(command: &mut Command, phase: &str) -> R<i32> {
    let argv: Vec<_> = std::iter::once(command.get_program()).chain(command.get_args()).map(OsString::from).collect();
    let status = command.status()?;
    eprintln!("{TEST_RECEIPT}{}", json!({"phase":phase,"argv":argv,"exit":status.code(),"signal":status.signal()}));
    if let Some(signal) = status.signal() {
        // Preserve a failed child as a failed/signal result, never execute the
        // test after compiler failure and never retry through another engine.
        nix::sys::signal::raise(Signal::try_from(signal)?)?;
        return Err("signalled child unexpectedly returned".into());
    }
    Ok(status.code().ok_or("missing child exit status")?)
}

/// The outer existing recorder times this complete compile-and-test command.
/// This adapter adds no clock or estimator; both actual child commands and
/// outcomes remain in its raw stderr alongside the compiler/test diagnostics.
fn rustc_test(arguments: &[OsString]) -> R<i32> {
    require(arguments.len() == 3, "internal rustc test requires RUSTC SOURCE NEW_TEST_BINARY")?;
    let source = Path::new(&arguments[1]);
    let output = Path::new(&arguments[2]);
    require(!output.exists(), "test output must be fresh")?;
    let mut compile = Command::new(&arguments[0]);
    rustc_arguments(&mut compile, source, output, "test");
    let code = child_status(&mut compile, "compile")?;
    if code != 0 {
        return Ok(code);
    }
    require(output.is_file(), "rustc did not publish the test binary")?;
    child_status(Command::new(output).args(["--test-threads=1", "--color", "never"]), "execute")
}

fn verify_test_output(lane: usize, stdout: &[u8], passed: bool) -> R<()> {
    if lane == 3 {
        let text = std::str::from_utf8(stdout)?;
        let expected = if passed {
            "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;"
        } else {
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;"
        };
        let summaries: Vec<_> = text.lines().filter(|line| line.starts_with("test result:")).collect();
        require(summaries.len() == 1 && summaries[0].starts_with(expected), "matched rustc did not execute exactly the required test")
    } else {
        let summaries: Vec<Value> = stdout
            .split(|b| *b == b'\n')
            .filter_map(|line| serde_json::from_slice(line).ok())
            .filter(|v: &Value| v["kind"] == "summary")
            .collect();
        require(
            summaries.len() == 1
                && summaries[0]["build"] == "ok"
                && summaries[0]["total"] == 1
                && summaries[0]["passed"] == usize::from(passed)
                && summaries[0]["failed"] == usize::from(!passed)
                && summaries[0]["timed_out"] == 0
                && summaries[0]["ignored"] == 0
                && summaries[0]["filtered_out"] == 0,
            "native compile-plus-test did not execute exactly the required test",
        )
    }
}

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
    fn native_executable(&self, lane: usize) -> R<PathBuf> {
        match lane {
            0 => Ok(self.pristine.clone()),
            1 => Ok(self.toolchain.join("bin/lust")),
            2 => Ok(self.toolchain.join("bin/lustr")),
            3 => Ok(self.toolchain.join("bin/rustc-reference")),
            _ => Err("unknown native lane".into()),
        }
    }
    fn native(&self, lane: usize, source: &Path, state: &Path, sockets: &Daemons, operation: &str, cold: bool) -> R<Command> {
        let exe = self.native_executable(lane)?;
        if lane == 3 {
            let socket = sockets.sockets.join("unused-rustc.sock");
            if operation == "test" {
                let mut command = self.command(&std::env::current_exe()?, source.parent().unwrap(), state, &socket)?;
                command.arg("--native-rustc-test").arg(&exe).arg(source).arg(state.join(format!("test-program-{}", self.serial)));
                return Ok(command);
            }
            let mut command = self.command(&exe, source.parent().unwrap(), state, &socket)?;
            let output = state.join(if operation == "check" { "metadata.rmeta" } else { "program" });
            rustc_arguments(&mut command, source, &output, operation);
            return Ok(command);
        }
        let mut command = self.command(&exe, source.parent().unwrap(), state, &sockets.sockets.join(format!("n{lane}.sock")))?;
        command.arg(operation).arg(source);
        if operation == "build" {
            command.arg("-o").arg(state.join("program"));
        } else if operation == "test" {
            command.arg("--json");
        }
        if operation != "test" {
            command.args([
                "--target",
                "x86_64-unknown-linux-gnu",
                "-C",
                "panic=unwind",
                "-C",
                "overflow-checks=on",
                "-C",
                "debug-assertions=on",
            ]);
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
    fn native_reference_probes(&mut self) -> R<()> {
        let root = self.root.join("native-reference-probes");
        let source = root.join("source/main.rs");
        put(&source, native_source("17", false))?;
        let mut sockets = Daemons::new()?;
        for (phase, operation, text, expected) in [
            ("reference-probe-check", "check", native_source("17", false), 0),
            ("reference-probe-build", "build", native_source("17", false), 0),
            ("reference-probe-test", "test", native_source("17", false), 0),
            ("reference-probe-invalid", "check", native_source("true", false), 1),
            ("reference-probe-failed-test", "test", native_source("17", false).replace("kernel(42),59", "kernel(42),60"), 101),
            ("reference-probe-recovery", "build", native_source("17", false), 0),
        ] {
            fs::write(&source, text)?;
            for lane in [2, 3] {
                let state = root.join(format!("lane{lane}"));
                let mut command = self.native(lane, &source, &state, &sockets, operation, false)?;
                let result = self.run("native", native_lane(lane), phase, None, &source, expected, &mut command)?;
                if operation == "test" {
                    verify_test_output(lane, &result.stdout, expected == 0)?;
                } else if operation == "build" {
                    self.runtime("native", native_lane(lane), phase, &source, &state, &state.join("program"), b"59\n")?;
                }
            }
        }
        let count = sockets.stop()?;
        require(count > 0, "native/Rust reference probes lost native daemon execution")?;
        self.event(json!({"kind":"daemon-exits","scope":"native-reference-probes","successful_exits":count}))
    }
    fn native_samples(&mut self) -> R<()> {
        let schedules = order::rounds(4, self.pairs, &mut self.rng);
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
                            if prime == 0 && lane < 3 {
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
                        verify_test_output(lane, &result.stdout, true)?;
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
            let expected = if family == "native" { vec!["pristine", "integrated", "entry", "rustc"] } else { vec!["reference", "mixed"] };
            require(lanes.len() == expected.len(), "missing comparison lane")?;
            for name in &expected {
                let samples = lanes.get_mut(*name).ok_or("missing required lane")?;
                samples.sort_by_key(|p| p.0);
                require(
                    samples.len() == self.pairs && samples.iter().enumerate().all(|(i, p)| p.0 == i),
                    "incomplete or duplicate paired samples",
                )?;
            }
            let comparisons = if family == "native" {
                vec![("pristine", "integrated"), ("pristine", "entry"), ("rustc", "entry")]
            } else {
                vec![("reference", "mixed")]
            };
            for (baseline, candidate) in comparisons {
                let pairs: Vec<_> = lanes[baseline].iter().zip(&lanes[candidate]).map(|(a, b)| (a.1, b.1)).collect();
                let summary = stats::summarize(&pairs, 10_000, 0x4c55535452)?;
                let observation = if summary.bootstrap_95[1] < 1.0 {
                    "slower-on-this-corpus"
                } else if summary.bootstrap_95[0] > 1.0 {
                    "faster-on-this-corpus"
                } else {
                    "inconclusive"
                };
                rows.push(json!({"family":family,"phase":phase,"baseline":baseline,"candidate":candidate,"ratio_direction":"baseline time / candidate time; greater than 1 is faster","observation":observation,"analysis":summary}));
            }
        }
        Ok(json!(rows))
    }
}

fn native_lane(lane: usize) -> &'static str {
    ["pristine", "integrated", "entry", "rustc"][lane]
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
    let all_arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    if all_arguments.first().is_some_and(|value| value == "--native-rustc-test") {
        std::process::exit(rustc_test(&all_arguments[1..])?);
    }
    require(cfg!(all(target_os = "linux", target_arch = "x86_64")), "historical mixed-package measurements require Linux x86-64")?;
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
    require(args.next().is_none() && pairs >= 12 && pairs <= 96 && pairs % 4 == 0, "pairs must be a multiple of four between 12 and 96")?;
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
    // Collected before timing. Preserve exactly what the collector observed;
    // old campaigns without this input must not acquire invented host facts.
    let host = if let Some(path) = std::env::var_os("LUSTR_BENCH_HOST_RECORD") {
        let path = PathBuf::from(path);
        require(fs::metadata(&path)?.len() <= 65536, "host record is too large")?;
        let bytes = fs::read(&path)?;
        let record: Value = serde_json::from_slice(&bytes)?;
        require(record["schema"] == "lustr-benchmark-host-v1", "unsupported host record")?;
        let output = campaign.root.join("host.json");
        put(&output, bytes)?;
        Some(json!({"sha256":digest(&output)?,"record":record}))
    } else {
        None
    };
    let mut native_lane_executables = BTreeMap::new();
    for lane in 0..4 {
        native_lane_executables.insert(native_lane(lane), campaign.native_executable(lane)?);
    }
    let mut report = json!({
        "schema":2,"status":"failed","performance_admission":"not granted by this runner","pairs":pairs,
        "pristine_revision":head,"pristine_tree":tree,"pristine_source":pristine_source,"compiler_sha256":identities,"rust_reference_version":rust,
        "order_method":order::METHOD,"seed":0x4c55535452u64,"summaries":null,
        "scope":"one native scalar corpus and one real Cargo/proc-macro/async/no_std+alloc graph; not full language, all workloads, all profiles or all platforms",
        "profiles":"owned compiler components: manifest release profiles; fixture applications: dev opt0/debug0, normal unwind/checks; full Rust reference is the matching imported SDK",
        "cold_scope":"fresh per-pair Cargo target, intermediate, mixed, native and XDG runtime cache directories; native cold disables daemon/disk compiler caches; OS page caches are not flushed",
        "warm_scope":"same-lane retained caches and original native daemons; matched standalone rustc is nonincremental and starts a process on every request; its no-op is not Cargo Fresh; untimed probes require retained checks/body edits",
        "timing_scope":"command wall time through observed child exit, 100us polling; tracing/stats disabled for all measured commands; runtime assertions after builds are outside compile-only timing; compile-plus-test executes actual tests",
        "inference_limits":"shared-host paired observations; bootstrap assumes independent pairs; exploratory per-mode comparisons; pristine binary build recipe must accompany this source/binary identity record",
        "native_lane_executables":native_lane_executables,
        "native_rustc_reference":{
            "compiler":campaign.toolchain.join("bin/rustc-reference"),"test_driver":std::env::current_exe()?,
            "flags":RUSTC_FLAGS,"incremental":"disabled; standalone rustc default",
            "build":"--emit=link","check":"--emit=metadata",
            "test":"--test then execute --test-threads=1 --color never; one outer wall-time measurement",
            "source":"identical native_source bytes across all four native lanes",
            "native_profile":"edition 2024, TinyCC dev/no debug; DEV_UNWIND with overflow checks and debug assertions; test uses the native test harness defaults"
        }
    });
    if let Some(host) = host {
        report["host"] = host;
    }
    campaign.event(json!({"kind":"identities","report":report}))?;
    let result = (|| -> R<Value> {
        set_child_subreaper(true)?;
        campaign.native_probes()?;
        campaign.native_reference_probes()?;
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
        "Completed historical package measurements, without current compiler qualification; inspect per-mode results in {}",
        campaign.root.join("report.json").display()
    );
    Ok(())
}

#[cfg(test)]
mod reference_tests {
    use super::*;

    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn scratch() -> Scratch {
        let path = std::env::temp_dir().join(format!(
            "lustr-rustc-reference-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Scratch(path)
    }
    fn rustc() -> OsString {
        std::env::var_os("LUSTR_BENCH_TEST_RUSTC").or_else(|| std::env::var_os("RUSTC")).unwrap_or_else(|| "rustc".into())
    }

    #[test]
    fn reference_build_check_edits_and_rejection_execute_real_rustc() {
        let directory = scratch();
        let source = directory.0.join("main.rs");
        for (literal, signature, expected) in [("17", false, b"59\n"), ("19", false, b"57\n"), ("17", true, b"59\n")] {
            fs::write(&source, native_source(literal, signature)).unwrap();
            let binary = directory.0.join("program");
            let mut compile = Command::new(rustc());
            rustc_arguments(&mut compile, &source, &binary, "build");
            let output = compile.output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            let result = Command::new(&binary).output().unwrap();
            assert!(result.status.success());
            assert_eq!(result.stdout, expected);
            assert!(result.stderr.is_empty());
            let mut check = Command::new(rustc());
            rustc_arguments(&mut check, &source, &directory.0.join("metadata.rmeta"), "check");
            assert!(check.output().unwrap().status.success());
        }
        fs::write(&source, native_source("true", false)).unwrap();
        let metadata = directory.0.join("invalid.rmeta");
        let mut check = Command::new(rustc());
        rustc_arguments(&mut check, &source, &metadata, "check");
        let result = check.output().unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(!metadata.exists());
        assert!(String::from_utf8_lossy(&result.stderr).contains("E0277"));
    }

    #[test]
    fn reference_test_adapter_executes_once_and_preserves_failure() {
        let directory = scratch();
        let source = directory.0.join("tests.rs");
        let marker = directory.0.join("executed");
        for (name, passes, expected) in [("pass", true, 0), ("fail", false, 101)] {
            fs::write(&source, format!(
                "#[test] fn scalar_contract() {{ use std::io::Write; let mut f=std::fs::OpenOptions::new().create(true).append(true).open({marker:?}).unwrap(); f.write_all(b\"x\").unwrap(); assert!({passes}); }}\n"
            )).unwrap();
            assert_eq!(rustc_test(&[rustc(), source.clone().into_os_string(), directory.0.join(name).into_os_string()]).unwrap(), expected);
        }
        assert_eq!(fs::read(&marker).unwrap(), b"xx", "each accepted test binary must execute exactly once");
        fs::write(&source, native_source("true", false)).unwrap();
        let invalid = directory.0.join("invalid-test");
        assert_eq!(rustc_test(&[rustc(), source.into_os_string(), invalid.clone().into_os_string()]).unwrap(), 1);
        assert!(!invalid.exists());
        assert_eq!(fs::read(&marker).unwrap(), b"xx", "a failed compile must not execute or retry a test");
    }
}
