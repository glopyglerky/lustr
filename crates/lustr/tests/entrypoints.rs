//! The retired coordinator must never become another compiler entry.
//! Exercise real executables and Cargo extension lookup with effect-recording
//! tools. The integrated compiler has its own native acceptance suite.
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SERIAL: AtomicU64 = AtomicU64::new(0);
const REFUSAL: &[u8] = b"lustr: the independent compiler coordinator has been retired; no alternate compiler was invoked\n";

struct Fixture {
    root: PathBuf,
    marker: PathBuf,
    spy: PathBuf,
    cargo: OsString,
}

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("lustr retired entries {} {nonce} {serial}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("bin")).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname='retired-entry-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n")
            .unwrap();
        // A real valid input must be refused just as invalid/unknown work is.
        fs::write(root.join("src/main.rs"), "fn main() { std::process::exit(23); }\n").unwrap();
        let marker = root.join("tool-called");
        let spy = root.join(format!("effect-spy{}", std::env::consts::EXE_SUFFIX));
        let source = root.join("effect-spy.rs");
        fs::write(
            &source,
            r#"
fn main() {
    std::fs::write(std::env::var_os("LUSTR_REFUSAL_SPY").unwrap(), b"unexpected tool invocation").unwrap();
    std::process::exit(79);
}
"#,
        )
        .unwrap();
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let built = Command::new(rustc).arg(&source).arg("-o").arg(&spy).output().unwrap();
        assert!(built.status.success(), "cannot build refusal witness: {}", String::from_utf8_lossy(&built.stderr));
        // Verify the witness itself before asking it to detect a delegation.
        let control = Command::new(&spy).env("LUSTR_REFUSAL_SPY", &marker).output().unwrap();
        assert_eq!(control.status.code(), Some(79));
        assert_eq!(fs::read(&marker).unwrap(), b"unexpected tool invocation");
        fs::remove_file(&marker).unwrap();
        for name in ["cargo", "rustc", "rustdoc", "lust", "rustc-full", "rustc-reference", "clippy-driver"] {
            fs::copy(&spy, root.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))).unwrap();
        }
        for (name, binary) in [("cargo-lust", env!("CARGO_BIN_EXE_cargo-lust")), ("cargo-lustr", env!("CARGO_BIN_EXE_cargo-lustr"))] {
            fs::copy(binary, root.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))).unwrap();
        }
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| env!("CARGO").into());
        Self { root, marker, spy, cargo }
    }

    fn command(&self, program: &OsStr) -> Command {
        let mut paths = vec![self.root.join("bin")];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
        let mut command = Command::new(program);
        command.current_dir(&self.root).env("PATH", std::env::join_paths(paths).unwrap()).env("LUSTR_REFUSAL_SPY", &self.marker);
        for key in ["LUST_CARGO", "LUST_BIN", "RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"] {
            command.env(key, &self.spy);
        }
        command.env("LUST_CRANELIFT_BACKEND", &self.spy);
        command
    }

    fn refusal(&self, command: &mut Command) {
        let result = command.output().unwrap();
        assert_eq!(result.status.code(), Some(2), "{command:?}: {result:?}");
        assert!(result.stdout.is_empty(), "retired entry emitted successful output: {result:?}");
        assert_eq!(result.stderr, REFUSAL, "{command:?}");
        self.no_effects();
    }

    fn no_effects(&self) {
        assert!(!self.marker.exists(), "retired entry executed a configured compiler, Cargo, wrapper or native helper");
        assert!(!self.root.join("target").exists(), "retired entry built a Cargo artifact");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn every_retired_cli_refuses_compilation_without_running_tools() {
    let fixture = Fixture::new();
    for binary in [env!("CARGO_BIN_EXE_lustr"), env!("CARGO_BIN_EXE_cargo-lustr"), env!("CARGO_BIN_EXE_cargo-lust")] {
        for args in [
            vec!["build", "--workspace"],
            vec!["check", "--workspace"],
            vec!["test", "--workspace"],
            vec!["run", "--", "--release", "argument with spaces"],
            vec!["doc"],
            vec!["doctor"],
            vec!["plan", "build"],
            vec!["--engine", "auto", "build"],
            vec!["--engine", "rustc", "build"],
            vec!["--engine", "cranelift", "build"],
            vec!["--engine", "lust", "build", "src/main.rs"],
            vec!["+nightly", "build"],
            vec!["--explain", "build"],
            vec!["unknown-extension"],
            vec![],
        ] {
            fixture.refusal(fixture.command(OsStr::new(binary)).args(args));
        }
        fixture.refusal(fixture.command(OsStr::new(binary)).arg("--bluefe-root").arg(&fixture.root).args(["build", "--release"]));
        fixture.refusal(fixture.command(OsStr::new(binary)).arg("--bluefe-root").arg(&fixture.root).args(["--frontend-jobs=4", "rustc"]));
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            fixture.refusal(fixture.command(OsStr::new(binary)).arg("build").arg(OsString::from_vec(vec![b'x', 0xff])));
        }
    }
    // Real Cargo extension discovery must reach the same refusal. The Cargo
    // executable is explicit; all child compiler/helper candidates are spies.
    for extension in ["lust", "lustr"] {
        fixture.refusal(fixture.command(&fixture.cargo).args([extension, "run", "--offline", "--", "argument with spaces"]));
    }
}

#[test]
fn only_retirement_information_is_successful() {
    let fixture = Fixture::new();
    for binary in [env!("CARGO_BIN_EXE_lustr"), env!("CARGO_BIN_EXE_cargo-lustr"), env!("CARGO_BIN_EXE_cargo-lust")] {
        for option in ["--help", "-h", "--version", "-V"] {
            let result = fixture.command(OsStr::new(binary)).arg(option).output().unwrap();
            assert!(result.status.success(), "{binary} {option}: {result:?}");
            assert!(result.stderr.is_empty());
            assert!(String::from_utf8_lossy(&result.stdout).contains("retired"));
            fixture.no_effects();
        }
        fixture.refusal(fixture.command(OsStr::new(binary)).args(["--help", "build"]));
        fixture.refusal(fixture.command(OsStr::new(binary)).args(["--version", "build"]));
    }
}
