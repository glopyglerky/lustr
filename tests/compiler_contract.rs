use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    fn native_expansion_rejects(&self, label: &str, source: &Path, artifact: &Path) {
        let result = self.invoke(
            &format!("expanded-native-{label}"),
            self.command("lust").arg("build").arg(source).arg("-o").arg(artifact).env("LUST_LIBRARY", self.package.join("lib/lustr")),
        );
        assert_eq!(result.status.code(), Some(1), "expanded native accepted {label}");
        assert!(!artifact.exists(), "expanded native published invalid {label}");
    }
}

#[test]
#[ignore = "requires the complete frozen native/full runtime; set LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn official_sdk_clippy_and_rustfmt_execute_from_the_package() {
    let f = Fixture::new("official-sdk-tools");
    for tool in ["rustc-full", "cargo", "clippy-driver", "cargo-clippy", "rustfmt", "cargo-fmt"] {
        let result = f.succeeds(&format!("{tool}-version"), f.command(tool).arg("--version"));
        assert!(!result.stdout.is_empty(), "{tool} returned no version");
    }
    f.source("Cargo.toml", "[package]\nname=\"sdk-tools\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    let source = f.source("src/lib.rs", "pub fn comparison(a:bool,b:bool)->bool{(a&&b)||(!a&&b)}\n");
    let result = f.invoke("rustfmt-reject", f.command("rustfmt").arg("--check").arg(&source));
    assert_eq!(result.status.code(), Some(1));
    f.succeeds("rustfmt-format", f.command("rustfmt").arg(&source));
    f.succeeds("cargo-fmt-check", f.command("lustr").args(["fmt", "--", "--check"]));
    let direct = f.invoke(
        "clippy-driver-reject",
        f.command("clippy-driver")
            .args(["--edition=2024", "--crate-type=lib", "--emit=metadata", "-Dclippy::overly_complex_bool_expr"])
            .arg(&source)
            .arg("-o")
            .arg(f.root.join("rejected.rmeta")),
    );
    assert_eq!(direct.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&direct.stderr).contains("overly_complex_bool_expr"));
    let result = f.invoke("clippy-reject", f.command("lustr").args(["clippy", "--offline", "--", "-Dclippy::overly_complex_bool_expr"]));
    assert_eq!(result.status.code(), Some(101));
    assert!(String::from_utf8_lossy(&result.stderr).contains("overly_complex_bool_expr"));
    f.source("src/lib.rs", "pub fn answer() -> u32 { 42 }\n");
    f.succeeds("clippy-valid", f.command("lustr").args(["clippy", "--offline", "--", "-Dwarnings"]));
}

#[test]
#[ignore = "requires the complete frozen native/full runtime; set LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_library_identity_controls_retained_source_state() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new("library-identity");
    let runtime = f.root.join("runtime with spaces");
    std::fs::create_dir_all(runtime.join("bin")).unwrap();
    std::fs::create_dir_all(runtime.join("lib/lustr/std/src")).unwrap();
    std::fs::create_dir_all(runtime.join("lib/lustr-expanded/bin")).unwrap();
    std::fs::create_dir_all(runtime.join("lib/lustr-expanded/lib/lust")).unwrap();
    std::fs::copy(f.package.join("bin/lustr"), runtime.join("bin/lustr")).unwrap();
    symlink(f.package.join("bin/lust"), runtime.join("bin/lust")).unwrap();
    std::fs::copy(f.package.join("bin/lust-core"), runtime.join("bin/lust-core")).unwrap();
    // Both the actual upstream inode identity and the component executable
    // path must survive installation; a symlink or independent copy cannot
    // supply this combination.
    std::fs::hard_link(runtime.join("bin/lust-core"), runtime.join("lib/lustr-expanded/bin/lust-core")).unwrap();
    symlink(f.package.join("lib/lust"), runtime.join("lib/lust")).unwrap();
    symlink(f.package.join("lib/liblustr_native_core.so"), runtime.join("lib/liblustr_native_core.so")).unwrap();
    std::fs::copy(f.package.join("lib/lustr-native.id"), runtime.join("lib/lustr-native.id")).unwrap();
    for item in std::fs::read_dir(f.package.join("lib/lustr/std/src")).unwrap() {
        let item = item.unwrap();
        std::fs::copy(item.path(), runtime.join("lib/lustr/std/src").join(item.file_name())).unwrap();
    }
    symlink("../lust/library/deps", runtime.join("lib/lustr/deps")).unwrap();
    symlink("../../../lustr", runtime.join("lib/lustr-expanded/lib/lust/library")).unwrap();
    symlink("../../../lust/tcc", runtime.join("lib/lustr-expanded/lib/lust/tcc")).unwrap();
    let library = runtime.join("lib/lustr");
    let manifest = std::fs::read_to_string(f.package.join("lib/lustr/sources.sha256")).unwrap();
    std::fs::write(library.join("sources.sha256"), &manifest).unwrap();
    let reseal = || {
        let paths: Vec<_> = manifest.lines().map(|line| line.split_once("  ").unwrap().1).collect();
        let output = Command::new("sha256sum").current_dir(&library).args(paths).output().unwrap();
        assert!(output.status.success());
        std::fs::write(library.join("sources.sha256"), output.stdout).unwrap();
    };
    let source = f.source("main.rs", "fn main(){println!(\"{}\",std::env::consts::DLL_PREFIX);}\n");
    let output = f.root.join("application");
    let build = || {
        let mut command = f.command(runtime.join("bin/lustr").to_str().unwrap());
        command.arg("build").arg(&source).arg("-o").arg(&output);
        command
    };
    f.succeeds("sealed-first", &mut build());
    assert_eq!(f.entry(), "source-library-daemon");
    assert_eq!(f.succeeds("sealed-first-run", &mut Command::new(&output)).stdout, b"lib\n");
    let env_path = library.join("std/src/env.rs");
    let original = std::fs::read_to_string(&env_path).unwrap();
    std::fs::write(&env_path, original.replace("= \"lib\";", "= \"changed-for-test\";")).unwrap();
    f.succeeds("changed-unsealed", &mut build());
    assert_eq!(f.entry(), "source-library-extension");
    assert_eq!(f.succeeds("changed-unsealed-run", &mut Command::new(&output)).stdout, b"changed-for-test\n");
    reseal();
    f.succeeds("changed-resealed", &mut build());
    assert_eq!(f.entry(), "source-library-daemon");
    assert_eq!(f.succeeds("changed-resealed-run", &mut Command::new(&output)).stdout, b"changed-for-test\n");
    std::fs::write(&env_path, original).unwrap();
    reseal();
    f.succeeds("restored", &mut build());
    assert_eq!(f.entry(), "source-library-daemon");
    assert_eq!(f.succeeds("restored-run", &mut Command::new(&output)).stdout, b"lib\n");
    std::fs::write(library.join("std/src/new-untracked.rs"), "// not in the sealed inventory\n").unwrap();
    f.succeeds("extra-input", &mut build());
    assert_eq!(f.entry(), "source-library-extension");
    assert_eq!(f.succeeds("extra-input-run", &mut Command::new(&output)).stdout, b"lib\n");
    // Requests must retain the client's macro environment, even though the
    // component daemon has its own socket environment.
    f.source("main.rs", "fn main(){println!(\"{}:{}\",std::env::consts::DLL_PREFIX,env!(\"LUST_DAEMON_SOCKET\"));}\n");
    std::fs::remove_file(library.join("std/src/new-untracked.rs")).unwrap();
    f.succeeds("client-environment", &mut build());
    assert_eq!(f.entry(), "source-library-daemon");
    assert_eq!(
        f.succeeds("client-environment-run", &mut Command::new(&output)).stdout,
        format!("lib:{}\n", f.root.join("daemon.sock").display()).as_bytes()
    );
    let component = runtime.join("lib/lustr-expanded/bin/lust-core");
    std::fs::remove_file(&component).unwrap();
    std::fs::copy(runtime.join("bin/lust-core"), &component).unwrap();
    f.succeeds("copied-core", &mut build());
    assert_eq!(f.entry(), "source-library-extension");
    std::fs::remove_file(&component).unwrap();
    std::fs::hard_link(runtime.join("bin/lust-core"), &component).unwrap();
    f.succeeds("restored-core-link", &mut build());
    assert_eq!(f.entry(), "source-library-daemon");
    let support = runtime.join("lib/lustr-expanded/lib/lust");
    for (name, expected, wrong) in [("library", "../../../lustr", "../../../lust/library"), ("tcc", "../../../lust/tcc", "../../../lustr")]
    {
        std::fs::remove_file(support.join(name)).unwrap();
        symlink(wrong, support.join(name)).unwrap();
        f.succeeds(&format!("wrong-{name}-link"), &mut build());
        assert_eq!(f.entry(), "source-library-extension");
        std::fs::remove_file(support.join(name)).unwrap();
        symlink(expected, support.join(name)).unwrap();
        f.succeeds(&format!("restored-{name}-link"), &mut build());
        assert_eq!(f.entry(), "source-library-daemon");
    }
}

#[test]
#[ignore = "requires the complete frozen native/full runtime; set LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_os_string_reserve_preserves_unwind_and_buffer() {
    let f = Fixture::new("native-os-string-capacity");
    let source = f.source("main.rs", include_str!("fixtures/native-os-string-capacity.rs"));
    let expanded = f.root.join("expanded");
    let oracle = f.root.join("oracle");
    let first = f.invoke(
        "pristine-first-rejection",
        f.command("lust").arg("build").arg(&source).arg("-o").arg(&expanded).env("LUST_NO_DAEMON", "1"),
    );
    assert_eq!(first.status.code(), Some(1));
    f.succeeds("expanded-build", f.command("lustr").arg("build").arg(&source).arg("-o").arg(&expanded).env("LUST_NO_DAEMON", "1"));
    assert_eq!(f.engine(), "lustr-native");
    f.succeeds("oracle-build", f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&oracle));
    let result = f.succeeds("expanded-run", &mut Command::new(&expanded));
    let reference = f.succeeds("oracle-run", &mut Command::new(&oracle));
    assert_eq!(result.stdout, reference.stdout);
    assert_eq!(result.stderr, reference.stderr);
}

#[test]
#[ignore = "requires the complete frozen native/full runtime; set LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_path_reserve_preserves_unwind_and_buffer() {
    let f = Fixture::new("native-path-capacity");
    let source = f.source("main.rs", include_str!("fixtures/native-path-capacity.rs"));
    let expanded = f.root.join("expanded");
    let oracle = f.root.join("oracle");
    let first = f.invoke(
        "pristine-first-rejection",
        f.command("lust").arg("build").arg(&source).arg("-o").arg(&expanded).env("LUST_NO_DAEMON", "1"),
    );
    assert_eq!(first.status.code(), Some(1));
    assert!(!expanded.exists());
    f.succeeds("oracle-build", f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&oracle));
    let reference = f.succeeds("oracle-run", &mut Command::new(&oracle));
    for mode in ["cold", "daemon", "repeat"] {
        let mut command = f.command("lustr");
        command.arg("build").arg(&source).arg("-o").arg(&expanded);
        if mode == "cold" {
            command.env("LUST_NO_DAEMON", "1");
        }
        f.succeeds(&format!("expanded-{mode}"), &mut command);
        assert_eq!(f.engine(), "lustr-native");
        let actual = f.succeeds(&format!("expanded-{mode}-run"), &mut Command::new(&expanded));
        assert_eq!(f.entry(), if mode == "cold" { "source-library-extension" } else { "source-library-daemon" });
        assert_eq!(actual.stdout, reference.stdout);
        assert_eq!(actual.stderr, reference.stderr);
    }
    for (label, invalid) in [
        ("wrong-amount", "fn main(){let mut p=std::path::PathBuf::new();p.reserve(\"invalid\");}\n"),
        ("immutable", "fn main(){let p=std::path::PathBuf::new();p.reserve_exact(4);}\n"),
        (
            "borrow-live",
            "fn main(){let mut p=std::path::PathBuf::from(\"x\");let r=p.as_path();p.reserve(4);println!(\"{}\",r.display());}\n",
        ),
    ] {
        f.source("main.rs", invalid);
        std::fs::remove_file(&expanded).unwrap();
        f.native_expansion_rejects(label, &source, &expanded);
        let actual = f.invoke(label, f.command("lustr").arg("build").arg(&source).arg("-o").arg(&expanded));
        assert_eq!(actual.status.code(), Some(1));
        assert!(!expanded.exists());
        let reference =
            f.invoke(&format!("oracle-{label}"), f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&oracle));
        assert_eq!(reference.status.code(), Some(1));
        std::fs::write(&expanded, b"not an accepted artifact").unwrap();
    }
}

#[test]
#[ignore = "requires the complete frozen native/full runtime; set LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_borrowed_path_conversions_match_rust() {
    let f = Fixture::new("native-path-from");
    let source = f.source("main.rs", include_str!("fixtures/native-path-from.rs"));
    let expanded = f.root.join("expanded");
    let oracle = f.root.join("oracle");
    let first = f.invoke(
        "pristine-first-rejection",
        f.command("lust").arg("build").arg(&source).arg("-o").arg(&expanded).env("LUST_NO_DAEMON", "1"),
    );
    assert_eq!(first.status.code(), Some(1));
    f.succeeds("oracle-build", f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&oracle));
    let reference = f.succeeds("oracle-run", &mut Command::new(&oracle));
    for mode in ["cold", "daemon"] {
        let mut command = f.command("lustr");
        command.arg("build").arg(&source).arg("-o").arg(&expanded);
        if mode == "cold" {
            command.env("LUST_NO_DAEMON", "1");
        }
        f.succeeds(&format!("expanded-{mode}"), &mut command);
        assert_eq!(f.engine(), "lustr-native");
        let result = f.succeeds(&format!("expanded-{mode}-run"), &mut Command::new(&expanded));
        assert_eq!(result.stdout, reference.stdout);
        assert_eq!(result.stderr, reference.stderr);
    }
    for (label, invalid) in [
        ("not-path", "fn main(){let _=std::path::PathBuf::from(&42u32);}\n"),
        (
            "no-suppression",
            "fn main(){let p=std::path::PathBuf::from(&std::ffi::OsString::from(\"x\"));let r=&p;drop(p);println!(\"{}\",r.display());}\n",
        ),
    ] {
        f.source("main.rs", invalid);
        std::fs::remove_file(&expanded).unwrap();
        f.native_expansion_rejects(label, &source, &expanded);
        let result = f.invoke(label, f.command("lustr").arg("build").arg(&source).arg("-o").arg(&expanded));
        assert_eq!(result.status.code(), Some(1));
        assert!(!expanded.exists());
        let reference =
            f.invoke(&format!("oracle-{label}"), f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&oracle));
        assert_eq!(reference.status.code(), Some(1));
        std::fs::write(&expanded, b"not an accepted artifact").unwrap();
    }
}

#[test]
#[ignore = "requires the complete frozen native/full runtime; set LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_library_expansion_keeps_program_checks_and_runtime() {
    use std::os::unix::ffi::OsStringExt;
    let f = Fixture::new("native-library-expansion");
    let source = f.source("main.rs", include_str!("fixtures/native-env-consts.rs"));
    let expanded = f.root.join("expanded");
    let oracle = f.root.join("oracle");
    let pristine = f.invoke(
        "pristine-first-rejection",
        f.command("lust").arg("build").arg(&source).arg("-o").arg(&expanded).env("LUST_NO_DAEMON", "1"),
    );
    assert_eq!(pristine.status.code(), Some(1));
    assert!(!expanded.exists());
    f.succeeds("oracle-build", f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&oracle));
    let run = |program: &Path| {
        let mut command = Command::new(program);
        command
            .current_dir(&f.root)
            .env_remove("LUSTR_ABSENT_VARIABLE")
            .env("LUSTR_NON_UNICODE", std::ffi::OsString::from_vec(vec![b'a', 0xff, b'z']));
        command
    };
    let reference = f.succeeds("oracle-run", &mut run(&oracle));
    for mode in ["cold", "daemon", "repeat"] {
        let mut command = f.command("lustr");
        command.arg("build").arg(&source).arg("-o").arg(&expanded);
        if mode == "cold" {
            command.env("LUST_NO_DAEMON", "1");
        }
        f.succeeds(&format!("expanded-{mode}"), &mut command);
        assert_eq!(f.engine(), "lustr-native");
        let result = f.succeeds(&format!("expanded-{mode}-run"), &mut run(&expanded));
        assert_eq!(result.stdout, reference.stdout);
        assert_eq!(result.stderr, reference.stderr);
    }
    // Keep the original single-file and Cargo fallback fixtures as native
    // expansion regressions now that the missing constant is implemented.
    f.source("main.rs", "fn main(){println!(\"full:{}\",std::env::consts::EXE_EXTENSION);}\n");
    f.succeeds("original-single-file", f.command("lustr").arg("build").arg(&source).arg("-o").arg(&expanded));
    assert_eq!(f.engine(), "lustr-native");
    assert_eq!(f.succeeds("original-single-file-run", &mut Command::new(&expanded)).stdout, b"full:\n");
    f.succeeds(
        "explicit-source-override",
        f.command("lustr").arg("build").arg(&source).arg("-o").arg(&expanded).env("LUST_LIBRARY", f.package.join("lib/lust/library")),
    );
    assert_eq!(f.engine(), "full-rust-core");
    // The private composition path is not a proof of canonical type-name
    // coverage. This valid combination still needs the compatibility core.
    f.source("main.rs", "fn main(){let _=std::env::consts::EXE_SUFFIX;println!(\"{}\",std::any::type_name::<std::env::VarError>());}\n");
    f.succeeds("canonical-type-name", f.command("lustr").arg("build").arg(&source).arg("-o").arg(&expanded));
    assert_eq!(f.engine(), "full-rust-core");
    assert_eq!(f.succeeds("canonical-type-name-run", &mut Command::new(&expanded)).stdout, b"std::env::VarError\n");
    let manifest =
        f.source("project/Cargo.toml", "[package]\nname=\"native_expansion\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n");
    let main = f.source("project/src/main.rs", "fn main(){println!(\"{}:{}\",env!(\"BUILD_VALUE\"),std::env::consts::EXE_EXTENSION);}\n");
    f.source("project/build.rs", "fn main(){println!(\"cargo:rustc-env=BUILD_VALUE=observed\");}\n");
    f.succeeds(
        "original-cargo",
        f.command("lustr").arg("build").arg(&main).arg("--manifest-path").arg(&manifest).args(["--offline", "-o"]).arg(&expanded),
    );
    assert_eq!(f.engine(), "lustr-native");
    assert_eq!(f.succeeds("original-cargo-run", &mut Command::new(&expanded)).stdout, b"observed:\n");
    for (label, invalid) in [
        ("wrong-type", "fn main(){let _:u32=std::env::consts::EXE_SUFFIX;}\n"),
        ("missing-name", "fn main(){let _=std::env::consts::EXE_PREFIX;}\n"),
        ("borrow", "fn main(){let s=String::from(std::env::consts::EXE_SUFFIX);let r=&s;drop(s);println!(\"{}\",r);}\n"),
    ] {
        f.source("main.rs", invalid);
        std::fs::remove_file(&expanded).unwrap();
        f.native_expansion_rejects(label, &source, &expanded);
        let result = f.invoke(label, f.command("lustr").arg("build").arg(&source).arg("-o").arg(&expanded));
        assert_eq!(result.status.code(), Some(1), "{label} accepted invalid Rust");
        assert!(!expanded.exists());
        let reference =
            f.invoke(&format!("oracle-{label}"), f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&oracle));
        assert_eq!(reference.status.code(), Some(1));
        // The next rejection needs an existing artifact for the same removal.
        std::fs::write(&expanded, b"not an accepted artifact").unwrap();
    }
}

#[test]
#[ignore = "requires the complete frozen native/full runtime; set LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_and_full_compiler_contract() {
    let f = Fixture::new("complete-contract");
    let source = f.source("main.rs", "fn main(){println!(\"first\");}\n");
    let executable = f.root.join("application");
    f.succeeds(
        "native-cold",
        f.command("lustr")
            .args([OsStr::new("build"), source.as_os_str(), OsStr::new("-o"), executable.as_os_str()])
            .env("LUST_NO_DAEMON", "1"),
    );
    assert_eq!(f.engine(), "lust");
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(trace.lines().last().unwrap()).unwrap()["entry"], "native-library");
    f.succeeds("native-daemon-first", f.command("lustr").arg("build").arg(&source).arg("-o").arg(&executable));
    assert_eq!(f.entry(), "daemon");
    f.source("main.rs", "fn main(){println!(\"edited\");}\n");
    f.succeeds("native-daemon-edit", f.command("lustr").arg("build").arg(&source).arg("-o").arg(&executable));
    assert_eq!(f.entry(), "daemon");
    assert_eq!(f.engine(), "lust");
    assert_eq!(f.succeeds("native-run", &mut Command::new(&executable)).stdout, b"edited\n");
    f.succeeds("raw-native", f.command("rustc").arg("--edition=2024").arg(&source).arg("-o").arg(&executable));
    assert_eq!(f.engine(), "lust");
    f.source("main.rs", "fn main(){println!(\"full:{}\",std::env::join_paths([\"\"]).unwrap().to_str().unwrap());}\n");
    f.succeeds("full-language", f.command("lustr").arg("build").arg(&source).arg("-o").arg(&executable));
    assert_eq!(f.engine(), "full-rust-core");
    assert_eq!(f.succeeds("full-run", &mut Command::new(&executable)).stdout, b"full:\n");
    let invalid = f.source("E0502.rs", "fn main(){let _:u32=\"wrong type\";}\n");
    let bad = f.invoke(
        "rejection",
        f.command("rustc").arg("--edition=2024").arg("--error-format=json").arg(&invalid).arg("-o").arg(f.root.join("invalid")),
    );
    assert_eq!(bad.status.code(), Some(1));
    assert!(!f.root.join("invalid").exists());
    let codes: Vec<String> = String::from_utf8(bad.stderr)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|v| v["level"] == "error")
        .filter_map(|v| v["code"]["code"].as_str().map(str::to_owned))
        .collect();
    assert!(codes.iter().any(|c| c == "E0308"));
    assert!(!codes.iter().any(|c| c == "E0502"));
    let library = f.source("library.rs", "pub fn answer()->u32{42}\n");
    f.succeeds(
        "real-metadata",
        f.command("rustc")
            .args(["--edition=2024", "--crate-type=rlib", "--emit=metadata"])
            .arg(&library)
            .arg("-o")
            .arg(f.root.join("library.rmeta")),
    );
    assert_eq!(f.engine(), "full-rust-core");
    assert!(std::fs::metadata(f.root.join("library.rmeta")).unwrap().len() > 0);
    let wasm = f.source(
        "wasm.rs",
        "#![no_std]\n#[panic_handler]fn panic(_: &core::panic::PanicInfo)->!{loop{}}\n#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u32{42}\n#[unsafe(no_mangle)]pub extern \"C\" fn bits(value:u32)->u32{value.count_ones()}\n#[unsafe(no_mangle)]pub extern \"C\" fn rotate(value:u32)->u32{value.rotate_left(8)}\n",
    );
    f.succeeds(
        "wasm",
        f.command("rustc")
            .args(["--edition=2024", "--crate-type=cdylib", "--target=wasm32-unknown-unknown", "-Cpanic=abort"])
            .arg(&wasm)
            .arg("-o")
            .arg(f.root.join("module.wasm")),
    );
    assert_eq!(&std::fs::read(f.root.join("module.wasm")).unwrap()[..8], b"\0asm\x01\0\0\0");
    // Execute the emitted no_std module in an independent WebAssembly runtime.
    // A valid header alone does not establish that the linked exports work.
    let node = std::env::var_os("LUSTR_TEST_NODE").unwrap_or_else(|| "node".into());
    let execution = f.succeeds(
        "wasm-execution",
        Command::new(node)
            .current_dir(&f.root)
            .arg("-e")
            .arg(concat!(
                "const assert = require('node:assert/strict');",
                "const bytes = require('node:fs').readFileSync(process.argv[1]);",
                "const module = new WebAssembly.Module(bytes);",
                "assert.deepEqual(WebAssembly.Module.imports(module), []);",
                "const api = new WebAssembly.Instance(module, {}).exports;",
                "assert.equal(api.answer(), 42);",
                "assert.equal(api.bits(0), 0);",
                "assert.equal(api.bits(0xffffffff), 32);",
                "assert.equal(api.bits(0xf0f00f0f), 16);",
                "assert.equal(api.rotate(0x12345678), 0x34567812);",
                "console.log('no_std WebAssembly execution passed');",
            ))
            .arg(f.root.join("module.wasm")),
    );
    assert_eq!(execution.stdout, b"no_std WebAssembly execution passed\n");

    let manifest =
        f.source("project/Cargo.toml", "[package]\nname=\"compiler_contract\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n");
    let main = f.source("project/src/main.rs", "fn main(){println!(\"{}\",env!(\"BUILD_VALUE\"));}\n");
    f.source("project/build.rs", "fn main(){use std::io::Write;let p=std::path::Path::new(&std::env::var(\"CARGO_MANIFEST_DIR\").unwrap()).join(\"runs.log\");std::fs::OpenOptions::new().create(true).append(true).open(p).unwrap().write_all(b\"run\\n\").unwrap();println!(\"cargo:rerun-if-changed=build.rs\");println!(\"cargo:rustc-env=BUILD_VALUE=observed\");}\n");
    f.succeeds(
        "cargo-native",
        f.command("lustr").arg("build").arg(&main).arg("--manifest-path").arg(&manifest).arg("--offline").arg("-o").arg(&executable),
    );
    assert_eq!(f.engine(), "lust");
    f.source(
        "project/src/main.rs",
        "fn main(){println!(\"{}:{}\",env!(\"BUILD_VALUE\"),std::env::join_paths([\"\"]).unwrap().to_str().unwrap());}\n",
    );
    f.succeeds(
        "cargo-full-fallback",
        f.command("lustr").arg("build").arg(&main).arg("--manifest-path").arg(&manifest).arg("--offline").arg("-o").arg(&executable),
    );
    assert_eq!(f.engine(), "full-rust-core");
    assert_eq!(std::fs::read_to_string(f.root.join("project/runs.log")).unwrap(), "run\n");
    assert_eq!(f.succeeds("cargo-full-run", &mut Command::new(&executable)).stdout, b"observed:\n");
    f.succeeds("cargo-full-check", f.command("lustr").arg("check").arg(&main).arg("--manifest-path").arg(&manifest).arg("--offline"));
    // A proc macro, doctest, unwind and Drop exercise the complete Cargo path.
    f.source(
        "project/macros/Cargo.toml",
        "[package]\nname=\"contract_macro\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[lib]\nproc-macro=true\n",
    );
    f.source("project/macros/src/lib.rs", "extern crate proc_macro;#[proc_macro]pub fn number(_:proc_macro::TokenStream)->proc_macro::TokenStream{\"42u32\".parse().unwrap()}\n");
    f.source("project/Cargo.toml", "[package]\nname=\"compiler_contract\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\nmembers=[\"macros\"]\n[dependencies]\ncontract_macro={path=\"macros\"}\n");
    f.source("project/src/lib.rs", "/// ```\n/// assert_eq!(compiler_contract::answer(),42);\n/// ```\npub fn answer()->u32{contract_macro::number!()}\n#[cfg(test)]mod tests{#[test]fn recovery(){use std::sync::atomic::{AtomicUsize,Ordering};static DROPS:AtomicUsize=AtomicUsize::new(0);struct Guard;impl Drop for Guard{fn drop(&mut self){DROPS.fetch_add(1,Ordering::SeqCst);}}assert!(std::panic::catch_unwind(||{let _g=Guard;panic!(\"expected\")}).is_err());assert_eq!(DROPS.load(Ordering::SeqCst),1);assert_eq!(super::answer(),42);}}\n");
    f.succeeds("cargo-proc-macro-tests-and-doctest", f.command("cargo").args(["test", "--offline", "--manifest-path"]).arg(&manifest));
    let version = f.succeeds("version", f.command("rustc").arg("-vV"));
    assert!(String::from_utf8(version.stdout).unwrap().contains("5b7b37aefcb5b2fba3c2392bc9065b9e4602d494"));
    assert!(Path::new(&f.root).join("cargo-proc-macro-tests-and-doctest.stdout").is_file());
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
    assert_eq!(f.engine(), "upstream-lust-command");
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
    assert_eq!(f.engine(), "upstream-lust-command");
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn cargo_native_first_build_preserves_build_script_environment() {
    let f = Fixture::new("cargo-native-first");
    let manifest = f.source("project/Cargo.toml", "[package]\nname=\"native_project\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n");
    let main = f.source("project/src/main.rs", "fn main(){println!(\"{}\",env!(\"BUILD_VALUE\"));}\n");
    f.source(
        "project/build.rs",
        "fn main(){println!(\"cargo:rerun-if-changed=build.rs\");println!(\"cargo:rustc-env=BUILD_VALUE=observed\");}\n",
    );
    let executable = f.root.join("application");
    f.succeeds(
        "native-project-build",
        f.command("lustr").arg("build").arg(&main).arg("--manifest-path").arg(&manifest).arg("--offline").arg("-o").arg(&executable),
    );
    assert_eq!(f.engine(), "lust");
    assert_eq!(f.succeeds("native-project-run", &mut Command::new(executable)).stdout, b"observed\n");
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
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT on Linux"]
fn native_rejection_preserves_full_core_resource_context() {
    let f = Fixture::new("resource-context");
    let source = f.source("resource.rs", "fn main(){let _=std::env::join_paths([\"\"]);let limits=include_str!(\"/proc/self/limits\");println!(\"{}\",limits.lines().find(|s|s.starts_with(\"Max stack size\")).unwrap());}\n");
    let baseline = f.root.join("baseline");
    let candidate = f.root.join("candidate");
    f.succeeds("baseline-build", f.command("rustc-full").arg("--edition=2024").arg(&source).arg("-o").arg(&baseline));
    f.succeeds("candidate-build", f.command("lustr").arg("build").arg(&source).arg("-o").arg(&candidate).env("LUST_NO_DAEMON", "1"));
    assert_eq!(f.engine(), "full-rust-core");
    let trace = std::fs::read_to_string(f.root.join("trace.jsonl")).unwrap();
    assert!(
        trace
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .any(|value| value["engine"] == "lust-rejected" && value["entry"] == "native-library")
    );
    let baseline = f.succeeds("baseline-run", &mut Command::new(baseline));
    let candidate = f.succeeds("candidate-run", &mut Command::new(candidate));
    assert_eq!(candidate.stdout, baseline.stdout);
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
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn cargo_selection_preserves_explicit_package_target_and_source() {
    let f = Fixture::new("cargo-selection");
    let manifest = f.source(
        "project/Cargo.toml",
        "[workspace]\nmembers=[\"one\",\"two\"]\nresolver=\"3\"\n[workspace.package]\nversion=\"0.2.0\"\nedition=\"2024\"\n",
    );
    f.source("project/one/Cargo.toml", "[package]\nname=\"one\"\nversion.workspace=true\nedition.workspace=true\n[[bin]]\nname=\"first\"\npath=\"shared.rs\"\n[[bin]]\nname=\"second\"\npath=\"shared.rs\"\n[[bin]]\nname=\"other\"\npath=\"other.rs\"\n");
    let source =
        f.source("project/one/shared.rs", "fn main(){println!(\"{}\",std::env::join_paths([\"\"]).unwrap().to_str().unwrap());}\n");
    f.source("project/one/other.rs", "fn main(){}\n");
    f.source("project/two/Cargo.toml", "[package]\nname=\"two\"\nversion.workspace=true\nedition.workspace=true\n");
    let other_source = f.source("project/two/src/main.rs", "fn main(){}\n");
    let output = f.root.join("application");
    let unique_bad = f.invoke(
        "unique-missing-bin",
        f.command("lustr")
            .arg("build")
            .arg(&other_source)
            .arg("--manifest-path")
            .arg(&manifest)
            .args([
                "--package",
                "two",
                "--bin",
                "nonexistent",
                "--offline",
                "--target",
                "wasm32-unknown-unknown",
                "-C",
                "panic=abort",
                "-o",
            ])
            .arg(&output),
    );
    assert!(!unique_bad.status.success(), "nonexistent bin was replaced by the only existing target");
    assert!(!output.exists());
    for (label, selectors) in [
        ("missing-bin", vec!["--package", "one", "--bin", "nonexistent"]),
        ("other-source", vec!["--package", "one", "--bin", "other"]),
        ("other-package", vec!["--package", "two", "--bin", "first"]),
        ("missing-package", vec!["--package", "nonexistent", "--bin", "first"]),
    ] {
        // Rejection must not replace an existing caller artifact.
        std::fs::write(&output, b"unchanged").unwrap();
        let result = f.invoke(
            label,
            f.command("lustr")
                .arg("build")
                .arg(&source)
                .arg("--manifest-path")
                .arg(&manifest)
                .args(["--offline", "-o"])
                .arg(&output)
                .args(selectors)
                .env("LUST_NO_DAEMON", "1"),
        );
        assert!(!result.status.success(), "{label} silently selected a different target");
        assert_eq!(std::fs::read(&output).unwrap(), b"unchanged", "{label} replaced output");
    }
    for (label, selectors) in [
        ("second-bin", vec!["--package", "one", "--bin", "second"]),
        ("version-spec", vec!["--package=one@0.2.0", "--bin=first"]),
        ("inferred-member", vec!["--bin", "second"]),
    ] {
        f.succeeds(
            label,
            f.command("lustr")
                .arg("build")
                .arg(&source)
                .arg("--manifest-path")
                .arg(&manifest)
                .args(["--offline", "-o"])
                .arg(&output)
                .args(selectors)
                .env("LUST_NO_DAEMON", "1"),
        );
        assert_eq!(f.engine(), "full-rust-core");
        assert_eq!(f.succeeds(&format!("{label}-run"), &mut Command::new(&output)).stdout, b"\n");
    }
    // Both workspace members now offer 'second'. The native resolver must
    // receive the validated member, rather than picking the first member.
    f.source("project/one/shared.rs", "fn main(){println!(\"one\");}\n");
    f.source(
        "project/two/Cargo.toml",
        "[package]\nname=\"two\"\nversion.workspace=true\nedition.workspace=true\n[[bin]]\nname=\"second\"\npath=\"src/main.rs\"\n",
    );
    f.source("project/two/src/main.rs", "fn main(){println!(\"two\");}\n");
    f.succeeds(
        "native-selected-member",
        f.command("lustr")
            .arg("build")
            .arg(&other_source)
            .arg("--manifest-path")
            .arg(&manifest)
            .args(["--offline", "--package", "two", "--bin", "second", "-o"])
            .arg(&output)
            .env("LUST_NO_DAEMON", "1"),
    );
    assert_eq!(f.engine(), "lust");
    assert_eq!(f.succeeds("native-selected-member-run", &mut Command::new(&output)).stdout, b"two\n");
    std::fs::write(&output, b"unchanged").unwrap();
    let bad = f.invoke(
        "native-missing-package",
        f.command("lustr")
            .arg("build")
            .arg(&other_source)
            .arg("--manifest-path")
            .arg(&manifest)
            .args(["--offline", "--package", "nonexistent", "--bin", "second", "-o"])
            .arg(&output)
            .env("LUST_NO_DAEMON", "1"),
    );
    assert!(!bad.status.success());
    assert_eq!(std::fs::read(&output).unwrap(), b"unchanged");
}

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT"]
fn native_library_selection_survives_process_startup() {
    let f = Fixture::new("native-library-selection");
    for (directory, package, library, kind) in [
        ("hyphen", "hyphen-lib", None, None),
        ("renamed", "app", Some("app_library"), None),
        ("rlib", "library_kind_rlib", None, Some("rlib")),
        ("staticlib", "library_kind_staticlib", None, Some("staticlib")),
        ("cdylib", "library_kind_cdylib", None, Some("cdylib")),
        ("dylib", "library_kind_dylib", None, Some("dylib")),
        ("proc-macro", "library_kind_proc_macro", None, Some("proc-macro")),
    ] {
        let mut library_config = if library.is_some() || kind.is_some() { "\n[lib]\n".to_owned() } else { String::new() };
        if let Some(name) = library {
            library_config.push_str(&format!("name={name:?}\n"));
        }
        if let Some(kind) = kind {
            library_config.push_str(&if kind == "proc-macro" {
                "proc-macro=true\n".to_owned()
            } else {
                format!("crate-type=[{kind:?}]\n")
            });
        }
        // A proc-macro library may not export an ordinary public function.
        // A private helper checks target selection without claiming macro execution.
        let visibility = if kind == Some("proc-macro") { "" } else { "pub " };
        let valid = format!("{visibility}fn answer()->u32{{42}}\n");
        let invalid = format!("{visibility}fn answer()->u32{{\"wrong library\"}}\n");
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
                .env("RUSTC", f.package.join("bin/rustc-reference")),
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
            .env("RUSTC", f.package.join("bin/rustc-reference")),
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

#[test]
#[ignore = "requires LUSTR_TEST_PACKAGE and LUSTR_TEST_OUTPUT on Linux"]
fn cancellation_terminates_owned_cargo_and_build_script_descendants() {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    struct Cleanup(Vec<i32>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            for group in &self.0 {
                unsafe {
                    kill(-*group, 9);
                }
            }
        }
    }
    fn alive(pid: i32) -> bool {
        std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .ok()
            .and_then(|s| s.rsplit_once(')').map(|(_, rest)| !rest.trim_start().starts_with('Z')))
            .unwrap_or(false)
    }
    let f = Fixture::new("cancellation");
    let manifest = f.source("project/Cargo.toml", "[package]\nname=\"cancellation\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n");
    let source = f.source("project/src/main.rs", "fn main(){}\n");
    f.source(
        "project/build.rs",
        r#"fn main(){
        let root=std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        if std::env::args().any(|s|s=="owned-leaf") {
            std::fs::write(root.join("leaf.pid"),std::process::id().to_string()).unwrap();
            loop {std::thread::sleep(std::time::Duration::from_secs(1));}
        }
        std::fs::write(root.join("script.pid"),std::process::id().to_string()).unwrap();
        std::process::Command::new(std::env::current_exe().unwrap()).arg("owned-leaf").spawn().unwrap().wait().unwrap();
    }"#,
    );
    for (label, signal, group) in [("pid-term", 15, false), ("group-term", 15, true), ("pid-int", 2, false), ("group-int", 2, true)] {
        let marker = f.root.join("project/leaf.pid");
        let _ = std::fs::remove_file(&marker);
        let output = f.root.join("application");
        std::fs::write(&output, b"original").unwrap();
        let mut command = f.command("lustr");
        command
            .arg("build")
            .arg(&source)
            .arg("--manifest-path")
            .arg(&manifest)
            .args(["--target", "wasm32-unknown-unknown", "-C", "panic=abort", "--offline", "-o"])
            .arg(&output)
            .process_group(0)
            .stdout(Stdio::from(std::fs::File::create(f.root.join(format!("{label}.stdout"))).unwrap()))
            .stderr(Stdio::from(std::fs::File::create(f.root.join(format!("{label}.stderr"))).unwrap()));
        let mut wrapper = command.spawn().unwrap();
        let mut cleanup = Cleanup(vec![wrapper.id() as i32]);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() && Instant::now() < deadline {
            assert!(wrapper.try_wait().unwrap().is_none(), "wrapper exited before {label}");
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(marker.exists(), "build script did not reach cancellable work");
        let leaf: i32 = std::fs::read_to_string(&marker).unwrap().parse().unwrap();
        let script: i32 = std::fs::read_to_string(f.root.join("project/script.pid")).unwrap().parse().unwrap();
        let status = std::fs::read_to_string(format!("/proc/{script}/status")).unwrap();
        let cargo: i32 = status.lines().find_map(|l| l.strip_prefix("PPid:")).unwrap().trim().parse().unwrap();
        cleanup.0.push(cargo);
        unsafe {
            assert_eq!(kill(if group { -(wrapper.id() as i32) } else { wrapper.id() as i32 }, signal), 0);
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        let status = loop {
            if let Some(status) = wrapper.try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "{label} did not terminate the wrapper");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(status.signal(), Some(signal), "{label} signal was not preserved");
        let deadline = Instant::now() + Duration::from_secs(2);
        while (alive(leaf) || alive(script) || alive(cargo)) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!alive(leaf) && !alive(script) && !alive(cargo), "{label} left compiler/build-script work alive");
        assert_eq!(std::fs::read(&output).unwrap(), b"original");
    }
}
