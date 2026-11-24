//! Independent stock-compiler controls for preserved compiler fixtures.
//! They establish expected Rust behavior, not current Lustr coverage. Full
//! Lustr qualification must repeat these outcomes through its integrated entry.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED: &str = "derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1\n";
const EXPECTED_PANIC_TRACE: &str =
    "fixture: entering catch_unwind\nfixture: panic hook: qualification panic\nfixture: catch_unwind returned; caught=true drops=1\n";
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("cargo-lust-{name}-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn fixture(&self, name: &str) -> PathBuf {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(name);
        // Spaces deliberately exercise argument preservation.
        let destination = self.0.join(format!("{name} with spaces"));
        copy_tree(&source, &destination);
        destination
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        // Direct Cargo runs may leave build output in the source fixture.
        // Qualification always compiles into its own fresh target directory.
        if entry.file_name() == OsStr::new("target") {
            continue;
        }
        let destination = destination.join(entry.file_name());
        let kind = entry.file_type().unwrap();
        assert!(!kind.is_symlink(), "fixture must not contain symlinks");
        if kind.is_dir() {
            copy_tree(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn cargo() -> OsString {
    std::env::var_os("LUST_TEST_CARGO").unwrap_or_else(|| OsString::from(env!("CARGO")))
}

fn command(scratch: &Scratch, fixture: &Path, target: &str) -> Command {
    let mut command = Command::new(cargo());
    command.current_dir(fixture);
    command.env("CARGO_TARGET_DIR", scratch.0.join(target));
    // The fixture is an independent stock-compiler reference. Do not inherit
    // an experimental backend from the test runner.
    command.env_remove("CARGO_PROFILE_DEV_CODEGEN_BACKEND");
    command.env_remove("CARGO_PROFILE_TEST_CODEGEN_BACKEND");
    command
}

fn execute(command: &mut Command) -> Output {
    command.output().unwrap_or_else(|error| panic!("could not execute {command:?}: {error}"))
}

fn success(output: &Output) {
    success_at("command", output);
}

fn success_at(stage: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{stage} failed: status={}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn independent_stock_reference_exercises_full_rust_fixture() {
    let scratch = Scratch::new("full-rust");
    let fixture = scratch.fixture("full-rust");
    let arguments =
        ["run", "--offline", "--locked", "--quiet", "-p", "fixture-app", "--", "--release", "two words", "--profile=application-argument"];
    let baseline = execute(command(&scratch, &fixture, "baseline").args(arguments));
    success(&baseline);
    let expected = format!("{EXPECTED}argument=--release\nargument=two words\nargument=--profile=application-argument\n");
    assert_eq!(baseline.stdout, expected.as_bytes());

    assert_eq!(baseline.stderr, EXPECTED_PANIC_TRACE.as_bytes());
}

#[test]
fn independent_stock_reference_checks_tests_and_doctests() {
    let scratch = Scratch::new("commands");
    let fixture = scratch.fixture("full-rust");
    let checked =
        execute(command(&scratch, &fixture, "reference").args(["check", "--offline", "--locked", "--workspace", "--message-format=json"]));
    success(&checked);
    let stdout = String::from_utf8(checked.stdout).unwrap();
    assert!(stdout.contains("build-finished"));
    assert!(stdout.lines().all(|line| line.starts_with('{')), "stock reference polluted Cargo JSON: {stdout}");

    let tested = execute(command(&scratch, &fixture, "reference").args(["test", "--offline", "--locked", "--workspace"]));
    success(&tested);
    let stdout = String::from_utf8_lossy(&tested.stdout);
    assert!(stdout.contains("features_keep_their_rust_semantics"), "library test was not run: {stdout}");
    assert!(stdout.contains("body_probe"), "doctest was not run: {stdout}");
}

#[test]
fn independent_stock_reference_rejects_invalid_borrows() {
    let scratch = Scratch::new("invalid");
    let fixture = scratch.fixture("invalid-borrow");
    let baseline = execute(command(&scratch, &fixture, "baseline").args(["check", "--offline", "--locked"]));
    assert!(!baseline.status.success());
    assert!(String::from_utf8_lossy(&baseline.stderr).contains("E0502"));
}

#[test]
fn command_scoped_profile_defaults_survive_later_cargo_config_overrides() {
    let scratch = Scratch::new("config-ordering");
    let fixture = scratch.fixture("full-rust");
    let request = cargo_lust::Request {
        engine: cargo_lust::Engine::Cranelift,
        explain: false,
        toolchain: None,
        bluefe_root: None,
        frontend_jobs: None,
        args: [
            "build",
            "--offline",
            "--locked",
            "--workspace",
            "-vv",
            "--color",
            "never",
            "--config",
            "profile.dev.package.fixture-core.opt-level=2",
        ]
        .map(OsString::from)
        .to_vec(),
    };
    let plan = cargo_lust::plan(
        &request,
        cargo_lust::Programs {
            cargo: cargo(),
            lust: "unused-lust".into(),
            explicit_cargo: true,
            cranelift_backend: Some(std::env::current_exe().unwrap().into_os_string()),
        },
    )
    .unwrap();

    // Exercise the retained plan argument ordering with real stable Cargo. Replace
    // only the unstable backend values with stable profile settings and omit
    // their feature gate; no Cranelift installation is needed for this regression.
    let arguments: Vec<_> = plan
        .args
        .into_iter()
        .filter(|arg| arg != "-Zcodegen-backend")
        .map(|arg| match arg.to_str() {
            Some(value) if value.starts_with("profile.dev.codegen-backend=") => OsString::from("profile.dev.opt-level=1"),
            Some(value) if value.starts_with("profile.test.codegen-backend=") => OsString::from("profile.test.opt-level=1"),
            _ => arg,
        })
        .collect();
    let built = execute(command(&scratch, &fixture, "config-ordering").args(arguments));
    success_at("real Cargo configuration ordering regression", &built);
    for (crate_name, level) in [("fixture_app", "1"), ("fixture_core", "2")] {
        let commands = verbose_rustc_commands(&built.stderr, crate_name);
        assert_eq!(commands.len(), 1);
        assert!(
            commands[0].windows(2).any(|pair| pair[0] == "-C" && pair[1] == format!("opt-level={level}")),
            "Cargo lost the requested profile value for {crate_name}; compiler command: {:?}",
            commands[0]
        );
    }
}

fn verbose_command_words(command: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = command.chars();
    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (Some('\''), '\'') | (Some('"'), '"') => quote = None,
            (Some('\''), _) => word.push(ch),
            (_, '\\') => {
                word.push(chars.next().expect("incomplete escape in Cargo's verbose command"));
                started = true;
            }
            (Some('"'), _) => word.push(ch),
            (None, '\'' | '"') => {
                quote = Some(ch);
                started = true;
            }
            (None, ch) if ch.is_whitespace() => {
                if started {
                    result.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            _ => {
                word.push(ch);
                started = true;
            }
        }
    }
    assert!(quote.is_none(), "unterminated quote in Cargo's verbose command: {command}");
    if started {
        result.push(word);
    }
    result
}

fn verbose_rustc_commands(stderr: &[u8], crate_name: &str) -> Vec<Vec<String>> {
    let log = std::str::from_utf8(stderr).expect("verbose qualification requires UTF-8 Cargo diagnostics");
    let commands: Vec<_> = log
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("Running `")?.strip_suffix('`'))
        .map(verbose_command_words)
        .filter(|words| words.windows(2).any(|pair| pair[0] == "--crate-name" && pair[1] == crate_name))
        // Cargo's rustc invocations specify emitted artifacts. Exclude rustdoc
        // commands that also have --crate-name when qualifying `cargo test`.
        .filter(|words| words.iter().any(|word| word.starts_with("--emit=")))
        .collect();
    assert!(!commands.is_empty(), "expected an actual rustc invocation for {crate_name}; Cargo log:\n{log}");
    commands
}

#[cfg(unix)]
const FCNTL_EXPECTED: &[u8] = b"fcntl variadic passed: getfd=2-args setfd=3-args std-owned-fd-drops=2\n";

#[test]
#[cfg(unix)]
fn independent_stock_reference_preserves_variadic_and_owned_file_abi() {
    let scratch = Scratch::new("fcntl-variadic");
    let fixture = scratch.fixture("fcntl-variadic");
    {
        let target = "reference";
        let output = execute(command(&scratch, &fixture, target).args(["run", "--offline", "--locked", "--quiet"]));
        success_at("variadic fcntl baseline execution", &output);
        assert_eq!(output.stdout, FCNTL_EXPECTED);
        assert!(output.stderr.is_empty(), "unexpected fcntl fixture stderr: {}", String::from_utf8_lossy(&output.stderr));
        let tested = execute(command(&scratch, &fixture, target).args(["test", "--offline", "--locked"]));
        success_at("variadic fcntl baseline Cargo test", &tested);
        assert!(String::from_utf8_lossy(&tested.stdout).contains("test tests::variadic_fcntl_preserves_owned_file_lifecycle ... ok"));
    }
}
