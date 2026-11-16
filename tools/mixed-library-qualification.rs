//! Real mixed rlib emission, source-hidden Rust consumers and fresh metadata.
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

fn retain(source: &Path, target: &Path) {
    if target.exists() {
        assert_eq!(fs::read(source).unwrap(), fs::read(target).unwrap(), "ambiguous artifact {}", target.display());
    } else {
        fs::copy(source, target).unwrap();
        fs::set_permissions(target, fs::Permissions::from_mode(0o644)).unwrap();
    }
}

fn artifacts(root: &Path, target: &Path) {
    if !root.is_dir() {
        return;
    }
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            artifacts(&path, target);
        } else if path.extension().is_some_and(|extension| matches!(extension.to_str(), Some("rlib" | "rmeta" | "so"))) {
            retain(&path, &target.join(path.file_name().unwrap()));
        }
    }
}

fn find(directory: &Path, prefix: &str, extension: &str) -> PathBuf {
    let paths: Vec<_> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name().unwrap().to_str().unwrap().starts_with(prefix) && path.extension().is_some_and(|ext| ext == extension)
        })
        .collect();
    assert_eq!(paths.len(), 1, "expected one {prefix}*.{extension}: {paths:?}");
    paths[0].clone()
}

struct HiddenSources(Vec<(PathBuf, PathBuf)>);
impl HiddenSources {
    fn new(work: &Path) -> Self {
        let mut paths = Vec::new();
        for name in ["mixed-lib", "macro-provider", "portable"] {
            let source = work.join(name);
            let hidden = work.join(format!("hidden-{name}"));
            fs::rename(&source, &hidden).unwrap();
            paths.push((source, hidden));
        }
        Self(paths)
    }
}
impl Drop for HiddenSources {
    fn drop(&mut self) {
        for (source, hidden) in &self.0 {
            fs::rename(hidden, source).unwrap();
        }
    }
}

struct Qualification {
    work: PathBuf,
    compiler: PathBuf,
    rustc: PathBuf,
    trace: PathBuf,
}
impl Qualification {
    fn run(&self, label: &str, command: &mut Command, success: bool) -> Output {
        command.env_remove("LD_LIBRARY_PATH");
        fs::write(self.work.join(format!("{label}.command")), format!("{command:?}\n")).unwrap();
        let output = command.output().unwrap();
        fs::write(self.work.join(format!("{label}.stdout")), &output.stdout).unwrap();
        fs::write(self.work.join(format!("{label}.stderr")), &output.stderr).unwrap();
        fs::write(self.work.join(format!("{label}.status")), format!("{}\n", output.status)).unwrap();
        assert_eq!(output.status.success(), success, "{label}: {}", String::from_utf8_lossy(&output.stderr));
        output
    }

    fn mixed_environment(&self, command: &mut Command) {
        command
            .env("LUSTR_MIXED_TRACE", &self.trace)
            .env("LUSTR_MIXED_CACHE_DIR", self.work.join("mixed-state"))
            .env("LUST_CACHE_DIR", self.work.join("native-cache"))
            .env("LUST_DAEMON_SOCKET", self.work.join("native.sock"))
            .env("LUST_DAEMON_IDLE", "2s")
            .env_remove("LUST_LIBRARY")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS");
    }

    fn build(&self, label: &str, success: bool) -> String {
        fs::write(&self.trace, "").unwrap();
        let source = self.work.join("mixed-lib/src/lib.rs");
        let before = fs::read(&source).unwrap();
        let mut command = Command::new(&self.compiler);
        command
            .current_dir(&self.work)
            .args(["build", "--offline", "--jobs", "2", "-vv", "-p", "library-app", "--target-dir"])
            .arg(self.work.join("target"))
            .arg("--config")
            .arg(format!("build.build-dir={:?}", self.work.join("intermediates")));
        self.mixed_environment(&mut command);
        self.run(label, &mut command, success);
        assert_eq!(fs::read(source).unwrap(), before, "compiler changed library source");
        fs::copy(&self.trace, self.work.join(format!("{label}.trace"))).unwrap();
        fs::read_to_string(&self.trace).unwrap()
    }

    fn execute(&self, label: &str, executable: &Path, expected: &[u8]) {
        retain(executable, &self.work.join(format!("actual-app-{label}")));
        let output = self.run(label, &mut Command::new(executable), true);
        assert_eq!(output.stdout, expected, "{label}");
    }

    fn snapshot(&self, label: &str, expected: &[u8]) -> Vec<u8> {
        let directory = self.work.join(format!("artifacts-{label}"));
        fs::create_dir(&directory).unwrap();
        artifacts(&self.work.join("target"), &directory);
        artifacts(&self.work.join("intermediates"), &directory);
        let library = find(&directory, "libmixed_lib-", "rlib");
        let metadata = find(&directory, "libmixed_lib-", "rmeta");
        let members = self.run(&format!("{label}-archive"), Command::new("ar").arg("t").arg(&library), true);
        let members = String::from_utf8(members.stdout).unwrap();
        let native: Vec<_> = members.lines().filter(|name| name.starts_with("lustr-native-")).collect();
        assert_eq!(native.len(), 2, "expected both real native object members: {members}");
        assert_ne!(native[0], native[1], "native archive member names must be distinct");

        // The independent process receives only genuine compiler artifacts;
        // the producer, proc-macro and no_std dependency sources are absent.
        let _hidden = HiddenSources::new(&self.work);
        let executable = self.work.join(format!("independent-{label}"));
        self.run(
            &format!("{label}-independent-build"),
            Command::new(&self.rustc)
                .arg(self.work.join("independent.rs"))
                .args(["--edition=2024", "--extern"])
                .arg(format!("mixed_lib={}", library.display()))
                .arg("--extern")
                .arg(format!("mixed_lib={}", metadata.display()))
                .arg("-L")
                .arg(format!("dependency={}", directory.display()))
                .arg("-o")
                .arg(&executable),
            true,
        );
        self.execute(&format!("{label}-independent-run"), &executable, expected);
        self.run(
            &format!("{label}-independent-check"),
            Command::new(&self.rustc)
                .arg(self.work.join("independent.rs"))
                .args(["--edition=2024", "--emit=metadata", "--extern"])
                .arg(format!("mixed_lib={}", metadata.display()))
                .arg("-L")
                .arg(format!("dependency={}", directory.display()))
                .arg("-o")
                .arg(self.work.join(format!("independent-{label}.rmeta"))),
            true,
        );
        fs::read(metadata).unwrap()
    }

    fn bitcode_policy(&self) {
        let source = self.work.join("bitcode.rs");
        put(&source, "pub fn kernel(x:i64)->i64{x ^ 17}\n");
        let library = self.work.join("libbitcode_boundary.rlib");
        fs::write(&self.trace, "").unwrap();
        let mut command = Command::new(self.compiler.with_file_name("rustc"));
        command
            .arg(&source)
            .args([
                "--crate-name=bitcode_boundary",
                "--crate-type=rlib",
                "--edition=2024",
                "-Copt-level=0",
                "-Cdebuginfo=0",
                "-Cembed-bitcode=yes",
                "-o",
            ])
            .arg(&library);
        self.mixed_environment(&mut command);
        self.run("bitcode-request", &mut command, true);
        assert!(
            !fs::read_to_string(&self.trace).unwrap().contains("native-definitions="),
            "bitcode request used the native object boundary"
        );
        let caller = self.work.join("bitcode-caller.rs");
        put(&caller, "fn main(){println!(\"{}\",bitcode_boundary::kernel(0));}\n");
        let executable = self.work.join("bitcode-caller");
        self.run(
            "bitcode-lto-consumer",
            Command::new(&self.rustc)
                .arg(&caller)
                .args(["--edition=2024", "-Clto=fat", "--extern"])
                .arg(format!("bitcode_boundary={}", library.display()))
                .arg("-o")
                .arg(&executable),
            true,
        );
        self.execute("bitcode-lto-run", &executable, b"17\n");
    }
}

fn main() {
    unsafe extern "C" {
        fn prctl(option: i32, ...) -> i32;
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }
    assert_eq!(unsafe { prctl(36, 1usize, 0usize, 0usize, 0usize) }, 0);
    let mut args = std::env::args_os().skip(1);
    let work = PathBuf::from(args.next().expect("new work directory"));
    let compiler = PathBuf::from(args.next().expect("lustr entry"));
    let rustc = PathBuf::from(args.next().expect("matching original rustc"));
    assert!(args.next().is_none());
    fs::create_dir(&work).unwrap();
    let work = work.canonicalize().unwrap();
    retain(&compiler, &work.join("actual-entry-bytes"));
    let driver = compiler.with_file_name("rustc-full").canonicalize().unwrap();
    retain(&driver, &work.join("actual-mixed-driver-bytes"));
    retain(&driver.with_file_name("lust-core"), &work.join("actual-native-core-bytes"));
    let q = Qualification { trace: work.join("mixed.trace"), work, compiler, rustc };
    put(
        &q.work.join("Cargo.toml"),
        "[workspace]\nmembers=['app','mixed-lib','macro-provider','portable']\nresolver='3'\n[profile.dev]\ndebug=0\n",
    );
    put(&q.work.join("portable/Cargo.toml"), "[package]\nname='portable'\nversion='0.1.0'\nedition='2024'\n");
    put(
        &q.work.join("portable/src/lib.rs"),
        "#![no_std]\nextern crate alloc;\npub fn adjustment()->i64{alloc::vec![2i64,3].iter().sum()}\n",
    );
    put(
        &q.work.join("macro-provider/Cargo.toml"),
        "[package]\nname='macro-provider'\nversion='0.1.0'\nedition='2024'\n[lib]\nproc-macro=true\n",
    );
    put(
        &q.work.join("macro-provider/src/lib.rs"),
        "use proc_macro::TokenStream;\n#[proc_macro]\npub fn general(_:TokenStream)->TokenStream{\"pub async fn general(x:i64)->i64{native(x)+portable::adjustment()}\".parse().unwrap()}\n",
    );
    put(
        &q.work.join("mixed-lib/Cargo.toml"),
        "[package]\nname='mixed-lib'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nmacro-provider={path='../macro-provider'}\nportable={path='../portable'}\n",
    );
    let library = "#![no_std]\nuse macro_provider::general;\ngeneral!();\npub fn native(x:i64)->i64{x ^ 17}\npub fn arithmetic(x:u64)->u64{x.wrapping_add(17)}\npub struct Pair{pub left:i64,pub right:i64}\npub fn pair(x:i64)->Pair{Pair{left:native(x),right:portable::adjustment()}}\npub fn generic<T:Copy>(x:T)->T{x}\n";
    let source = q.work.join("mixed-lib/src/lib.rs");
    put(&source, library);
    put(
        &q.work.join("app/Cargo.toml"),
        "[package]\nname='library-app'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nmixed-lib={path='../mixed-lib'}\n",
    );
    let consumer = "fn main(){assert_eq!(mixed_lib::generic(7u8),7);let pair=mixed_lib::pair(42);let w=std::task::Waker::noop();let mut future=std::pin::pin!(mixed_lib::general(42));let mut context=std::task::Context::from_waker(w);let value=match std::future::Future::poll(future.as_mut(),&mut context){std::task::Poll::Ready(value)=>value,std::task::Poll::Pending=>panic!(\"unexpected pending\")};println!(\"{} {} {} {}\",pair.left,pair.right,value,mixed_lib::arithmetic(u64::MAX));}\n";
    put(&q.work.join("app/src/main.rs"), consumer);
    put(&q.work.join("independent.rs"), consumer);
    let cold = q.build("cold", true);
    assert!(cold.contains("native-definitions=2"), "mixed library definitions were not produced: {cold}");
    assert_eq!(cold.matches("native-definition-omitted=").count(), 2, "both library definitions must be emitted by Lust: {cold}");
    q.execute("cargo-cold-run", &q.work.join("target/debug/library-app"), b"59 5 64 16\n");
    let original_metadata = q.snapshot("cold", b"59 5 64 16\n");
    fs::write(&source, library.replace("x ^ 17", "x ^ 19")).unwrap();
    let edit = q.build("body-edit", true);
    assert!(edit.contains("native-definitions=2"), "library edit did not generate native definitions: {edit}");
    assert!(!edit.contains("reused-rust-objects=1"), "linked library reused an old metadata recipe");
    q.execute("cargo-edited-run", &q.work.join("target/debug/library-app"), b"57 5 62 16\n");
    let edited_metadata = q.snapshot("edited", b"57 5 62 16\n");
    assert_ne!(original_metadata, edited_metadata, "edited library retained stale Rust metadata");
    fs::write(&source, library.replace("x ^ 17", "x ^ true")).unwrap();
    q.build("invalid-library", false);
    fs::write(&source, library).unwrap();
    let recovery = q.build("recovery", true);
    assert!(recovery.contains("native-definitions=2"), "recovery lost mixed library compilation: {recovery}");
    q.execute("cargo-recovery-run", &q.work.join("target/debug/library-app"), b"59 5 64 16\n");
    q.snapshot("recovery", b"59 5 64 16\n");
    q.bitcode_policy();
    let mut daemons = 0;
    loop {
        let mut status = 0;
        let pid = unsafe { waitpid(-1, &mut status, 0) };
        if pid < 0 {
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(10));
            break;
        }
        assert_eq!(status, 0, "owned native daemon {pid}");
        daemons += 1;
    }
    assert!(daemons > 0, "no native daemon was observed");
    fs::write(q.work.join("owned-daemons.status"), format!("{daemons} actual idle exit0; no remaining children\n")).unwrap();
    println!(
        "mixed rlib objects + real Rust metadata + source-hidden original-compiler consumers + proc macros/core/alloc + fresh edited metadata + rejection/recovery + preserved bitcode/LTO request: actual0"
    );
}
