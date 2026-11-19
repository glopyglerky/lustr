//! Execute checked integer ABI and duplicate module identities through Cargo.
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::{Command, Output},
};

const SOURCE: &str = r#"fn kernel(x:u64)->u64{x ^ 17}
mod first {pub fn kernel(x:u64)->u64{x ^ 19}}
mod second {pub mod nested {pub fn kernel(x:u64)->u64{x ^ 23}}}
fn signed(x:i32)->i32{x.wrapping_add(17)}
fn unsigned(x:u32)->u32{x.wrapping_add(17)}
fn narrow(x:u8)->u8{x.wrapping_add(17)}
fn byte(x:u32)->u8{x as u8}
fn half(x:u32)->i16{x as i16}
fn borrowed(x:&u32)->u32{*x}
fn decomposed(é:u64)->u64{é ^ 17}
fn main(){
    let args:Vec<String>=std::env::args().collect();
    let root:u64=args[1].parse().unwrap();
    let first_bias:u64=args[2].parse().unwrap();
    let second_bias:u64=args[3].parse().unwrap();
    let signed_bias:i64=args[4].parse().unwrap();
    let unsigned_bias:u64=args[5].parse().unwrap();
    let mut count=0;
    for input in (0u32..=65535).chain([65536,0x7fff_ffff,0x8000_0000,0xffff_0000,0xffff_ffff]){
        assert_eq!(kernel(u64::from(input)),u64::from(input)^root);
        assert_eq!(first::kernel(u64::from(input)),u64::from(input)^first_bias);
        assert_eq!(second::nested::kernel(u64::from(input)),u64::from(input)^second_bias);
        assert_eq!(signed(input as i32),((input as i32 as i64+signed_bias) as u64&0xffff_ffff) as u32 as i32);
        assert_eq!(unsigned(input),((u64::from(input)+unsigned_bias)&0xffff_ffff) as u32);
        assert_eq!(byte(input),(input&255) as u8);
        assert_eq!(half(input),(input&65535) as u16 as i16);
        assert_eq!(borrowed(&input),input);
        assert_eq!(decomposed(u64::from(input)),u64::from(input)^17);
        let small=input as u8;
        assert_eq!(narrow(small),((u16::from(small)+17)&255) as u8);
        count+=1;
    }
    println!("{count}");
}
"#;

struct Qualification {
    root: PathBuf,
    compiler: PathBuf,
    reference: PathBuf,
    trace: PathBuf,
}

impl Qualification {
    fn run(&self, label: &str, command: &mut Command, success: bool) -> Output {
        command.env_remove("LD_LIBRARY_PATH");
        fs::write(self.root.join(format!("{label}.command")), format!("{command:?}\n")).unwrap();
        let output = command.output().unwrap();
        fs::write(self.root.join(format!("{label}.stdout")), &output.stdout).unwrap();
        fs::write(self.root.join(format!("{label}.stderr")), &output.stderr).unwrap();
        fs::write(self.root.join(format!("{label}.status")), format!("{}\n", output.status)).unwrap();
        assert_eq!(output.status.success(), success, "{label}: {}", String::from_utf8_lossy(&output.stderr));
        output
    }

    fn compile(&self, label: &str, check: bool, success: bool) -> String {
        fs::write(&self.trace, "").unwrap();
        let before = fs::read(self.root.join("src/main.rs")).unwrap();
        let mut command = Command::new(&self.compiler);
        command
            .current_dir(&self.root)
            .args([if check { "check" } else { "build" }, "--offline", "--jobs", "2", "-vv", "--target-dir"])
            .arg(self.root.join("target"))
            .arg("--config")
            .arg(format!("build.build-dir={:?}", self.root.join("intermediates")))
            .env("LUSTR_MIXED_TRACE", &self.trace)
            .env("LUSTR_MIXED_CACHE_DIR", self.root.join("mixed-state"))
            .env("LUST_CACHE_DIR", self.root.join("native-cache"))
            .env("LUST_DAEMON_SOCKET", self.root.join("native.sock"))
            .env("LUST_DAEMON_IDLE", "2s")
            .env_remove("LUST_LIBRARY")
            .env_remove("LUST_NO_DAEMON")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS");
        self.run(label, &mut command, success);
        assert_eq!(fs::read(self.root.join("src/main.rs")).unwrap(), before, "{label}: compiler changed source");
        let trace = fs::read_to_string(&self.trace).unwrap();
        fs::write(self.root.join(format!("{label}.trace")), &trace).unwrap();
        trace
    }

    fn execute(&self, label: &str, biases: &[&str; 5]) {
        let application = self.root.join("target/debug/mixed_scalar");
        let retained = self.root.join(format!("application-{label}"));
        fs::copy(application, &retained).unwrap();
        let native = self.run(label, Command::new(&retained).args(biases), true);
        let reference = self.root.join(format!("reference-{label}"));
        self.run(
            &format!("{label}-reference-build"),
            Command::new(&self.reference)
                .arg(self.root.join("src/main.rs"))
                .args(["--edition=2024", "--crate-name=mixed_scalar", "-Copt-level=0", "-Cdebuginfo=0", "-o"])
                .arg(&reference),
            true,
        );
        let stock = self.run(&format!("{label}-reference-run"), Command::new(reference).args(biases), true);
        assert_eq!(native.stdout, b"65541\n", "{label}");
        assert_eq!(native.stdout, stock.stdout, "{label}: runtime differential");
        assert!(native.stderr.is_empty() && stock.stderr.is_empty());
    }

    fn certificates(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut records = BTreeMap::new();
        for entry in fs::read_dir(self.root.join("mixed-state")).unwrap() {
            let path = entry.unwrap().path().join("current.json");
            if path.is_file() {
                records.insert(path.clone(), fs::read(path).unwrap());
            }
        }
        records
    }

    fn replace(&self, old: &str, new: &str) {
        let path = self.root.join("src/main.rs");
        let source = fs::read_to_string(&path).unwrap();
        assert_eq!(source.matches(old).count(), 1, "unambiguous source edit");
        fs::write(path, source.replacen(old, new, 1)).unwrap();
    }

    fn remove_cargo_artifacts(&self) {
        let certificates = self.certificates();
        assert!(!certificates.is_empty());
        self.run(
            "clean-cargo-artifacts",
            Command::new(&self.compiler)
                .current_dir(&self.root)
                .args(["clean", "--offline", "-p", "mixed_scalar", "--target-dir"])
                .arg(self.root.join("target"))
                .arg("--config")
                .arg(format!("build.build-dir={:?}", self.root.join("intermediates"))),
            true,
        );
        assert!(!self.root.join("target/debug/mixed_scalar").exists());
        assert_eq!(self.certificates(), certificates, "Cargo cleanup changed the compiler's retained certificates");
    }
}

fn exact_count(trace: &str, prefix: &str, expected: usize) {
    let values: Vec<_> = trace.lines().filter_map(|line| line.strip_prefix(prefix)).collect();
    assert_eq!(values, [expected.to_string()], "{trace}");
    let mut definitions: Vec<_> = trace.lines().filter_map(|line| line.strip_prefix("native-admitted-definition=")).collect();
    definitions.sort_unstable();
    assert_eq!(definitions, ["byte", "first::kernel", "half", "kernel", "second::nested::kernel", "signed", "unsigned"], "{trace}");
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().expect("work directory"));
    let compiler = PathBuf::from(args.next().expect("lustr command"));
    let reference = PathBuf::from(args.next().expect("matching rustc reference"));
    assert!(args.next().is_none());
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"mixed_scalar\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n[profile.dev]\ndebug=0\n",
    )
    .unwrap();
    fs::write(root.join("src/main.rs"), SOURCE).unwrap();
    let q = Qualification { trace: root.join("mixed.trace"), root, compiler, reference };
    let trace = q.compile("cold-build", false, true);
    exact_count(&trace, "native-definitions=", 7);
    assert_eq!(trace.lines().filter(|line| line.starts_with("native-definition-omitted=")).count(), 7, "{trace}");
    q.execute("cold-run", &["17", "19", "23", "17", "17"]);

    // Cargo can restore its published executable from the intermediate build
    // without invoking rustc. Ask Cargo to remove this package's artifacts so
    // the next build executes the identical compiler request. The native/Rust
    // retained generation lives outside Cargo's artifact directories.
    q.remove_cargo_artifacts();
    let trace = q.compile("unchanged-build", false, true);
    assert!(trace.contains("reused-rust-objects=1 full-analysis=0"), "{trace}");
    q.execute("unchanged-run", &["17", "19", "23", "17", "17"]);

    q.replace("x ^ 19", "x ^ 29");
    let trace = q.compile("module-edit-build", false, true);
    assert!(trace.contains("reused-rust-objects=1 full-analysis=0"), "{trace}");
    q.execute("module-edit-run", &["17", "29", "23", "17", "17"]);

    q.replace("fn unsigned(x:u32)->u32{x.wrapping_add(17)}", "fn unsigned(x:u32)->u32{x.wrapping_add(19)}");
    let trace = q.compile("width-edit-build", false, true);
    assert!(trace.contains("reused-rust-objects=1 full-analysis=0"), "{trace}");
    q.execute("width-edit-run", &["17", "29", "23", "17", "19"]);

    let trace = q.compile("cold-check", true, true);
    exact_count(&trace, "native-checked-definitions=", 7);
    q.replace("x ^ 29", "x ^ 31");
    let trace = q.compile("module-edit-check", true, true);
    assert!(trace.contains("reused-check-context=1 full-analysis=0 native-check-edits=1"), "{trace}");

    let certificates = q.certificates();
    assert!(!certificates.is_empty());
    q.replace("fn signed(x:i32)->i32{x.wrapping_add(17)}", "fn signed(x:i32)->i32{x.wrapping_add(true)}");
    let trace = q.compile("invalid-width-build", false, false);
    assert!(!trace.contains("reused-rust-objects=1"), "{trace}");
    assert!(!trace.contains("native-definition-omitted="), "{trace}");
    for (path, before) in certificates {
        assert_eq!(fs::read(path).unwrap(), before);
    }
    q.replace("fn signed(x:i32)->i32{x.wrapping_add(true)}", "fn signed(x:i32)->i32{x.wrapping_add(17)}");
    let trace = q.compile("recovered-build", false, true);
    assert!(trace.contains("reused-rust-objects=1 full-analysis=0"), "{trace}");
    q.execute("recovered-run", &["17", "31", "23", "17", "19"]);
    fs::write(q.root.join("summary.txt"), "7 native definitions, including distinct nested module identities and i32/u32 calls\n8 compiler requests, 5 mixed and 5 reference executions\n65541 inputs x 10 checks x 5 mixed executions = 3277050 mixed-application assertions\n8/16-bit Rust argument ABI remains complementary\n").unwrap();
    println!("mixed scalar qualification passed");
}
