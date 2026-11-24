//! Compare native synchronization layouts with the selected Rust SDK.
//! Compile with rustc, then pass LUST RUSTC NEW_OUTPUT_DIR TARGET...
//! This qualifies frontend constants, not foreign linking or execution.

use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const TYPES: &[&str] = &[
    "std::sync::Mutex<()>",
    "std::sync::Mutex<u8>",
    "std::sync::Mutex<u64>",
    "std::sync::Mutex<u128>",
    "Option<std::sync::Mutex<bool>>",
    "std::sync::RwLock<()>",
    "std::sync::RwLock<u8>",
    "std::sync::RwLock<u128>",
    "std::sync::OnceLock<core::convert::Infallible>",
    "std::sync::OnceLock<u8>",
    "std::sync::Condvar",
    "Option<std::sync::Condvar>",
    "std::sync::Barrier",
];

fn run(command: &mut Command, directory: &Path, name: &str) -> Result<ExitStatus> {
    let stdout = File::create(directory.join(format!("{name}.stdout")))?;
    let stderr = File::create(directory.join(format!("{name}.stderr")))?;
    let mut child = command.stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr)).spawn()?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            fs::write(directory.join(format!("{name}.status")), format!("{status}\n"))?;
            return Ok(status);
        }
        if started.elapsed() > Duration::from_secs(60) {
            let _ = child.kill();
            let _ = child.wait();
            fs::write(directory.join(format!("{name}.status")), "timeout\n")?;
            return Err(format!("{name}: compiler timed out after 60 seconds").into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn rust_command(rustc: &Path, source: &Path, target: &str, output: &Path) -> Command {
    let mut command = Command::new(rustc);
    command.args(["--edition=2024", "--target", target, "--emit=metadata"]).arg(source).arg("-o").arg(output);
    command
}

fn native_command(lust: &Path, source: &Path, target: &str, directory: &Path) -> Command {
    let mut command = Command::new(lust);
    command
        .arg("check")
        .arg(source)
        .args(["--target", target, "--json"])
        .env("LUST_NO_DAEMON", "1")
        .env("LUST_CACHE_DIR", directory.join("native-cache"));
    command
}

// The oracle emits unoptimized exported functions, each returning one concrete
// size/alignment. Reject changed LLVM forms rather than infer or invent values.
fn constant_return(ir: &str, name: &str) -> Result<u64> {
    let marker = format!("@{name}()");
    let mut bodies = ir.split("\n}").filter(|body| body.lines().any(|line| line.starts_with("define ") && line.contains(&marker)));
    let body = bodies.next().ok_or_else(|| format!("missing oracle function {name}"))?;
    if bodies.next().is_some() {
        return Err(format!("duplicate oracle function {name}").into());
    }
    let mut returns = body.lines().filter_map(|line| line.trim().strip_prefix("ret i64 "));
    let value = returns.next().ok_or_else(|| format!("nonconstant oracle {name}"))?;
    if returns.next().is_some() {
        return Err(format!("multiple oracle returns for {name}").into());
    }
    Ok(value.parse()?)
}

fn qualify(lust: &Path, rustc: &Path, root: &Path, target: &str) -> Result<()> {
    if !target.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
        return Err("target must be a named Rust target triple".into());
    }
    let directory = root.join(target);
    fs::create_dir(&directory)?;
    let oracle = directory.join("oracle.rs");
    let llvm = directory.join("oracle.ll");
    let mut source = String::new();
    for (index, ty) in TYPES.iter().enumerate() {
        for kind in ["size", "align"] {
            source.push_str(&format!("#[unsafe(no_mangle)] pub fn {kind}_{index}() -> u64 {{ std::mem::{kind}_of::<{ty}>() as u64 }}\n"));
        }
    }
    fs::write(&oracle, source)?;
    let status = run(
        Command::new(rustc)
            .args(["--edition=2024", "--crate-type=lib", "--target", target])
            .args(["--emit=llvm-ir", "-C", "opt-level=0"])
            .arg(&oracle)
            .arg("-o")
            .arg(&llvm),
        &directory,
        "rust-oracle",
    )?;
    if !status.success() {
        return Err(format!("{target}: Rust oracle failed; target support must be installed").into());
    }
    let ir = fs::read_to_string(llvm)?;
    let mut fixture = String::from("#![allow(dead_code)]\n");
    let mut values = String::from("type\tsize\talign\n");
    let mut first_size = None;
    for (index, ty) in TYPES.iter().enumerate() {
        let size = constant_return(&ir, &format!("size_{index}"))?;
        let align = constant_return(&ir, &format!("align_{index}"))?;
        first_size.get_or_insert(size);
        values.push_str(&format!("{ty}\t{size}\t{align}\n"));
        fixture.push_str(&format!(
            "const _: () = assert!(std::mem::size_of::<{ty}>() == {size});\n\
             const _: () = assert!(std::mem::align_of::<{ty}>() == {align});\n"
        ));
    }
    fixture.push_str("fn main() {}\n");
    fs::write(directory.join("oracle-values.tsv"), values)?;
    let positive = directory.join("matches.rs");
    fs::write(&positive, fixture)?;
    if !run(&mut rust_command(rustc, &positive, target, &directory.join("matches.rmeta")), &directory, "rust-matches")?.success() {
        return Err(format!("{target}: Rust rejected its extracted layout assertions").into());
    }
    let native_ok = run(&mut native_command(lust, &positive, target, &directory), &directory, "native-matches")?.success();

    // Require a semantic rejection from both engines for a deliberately false
    // assertion. A crash, missing SDK, unsupported type, or setup failure is not
    // a passing negative case.
    let negative = directory.join("mismatch.rs");
    fs::write(
        &negative,
        format!(
            "const _: () = assert!(std::mem::size_of::<{}>() == {});\nfn main() {{}}\n",
            TYPES[0],
            first_size.ok_or("no oracle values")? + 1
        ),
    )?;
    let rust_rejection = run(&mut rust_command(rustc, &negative, target, &directory.join("mismatch.rmeta")), &directory, "rust-mismatch")?;
    let native_rejection = run(&mut native_command(lust, &negative, target, &directory), &directory, "native-mismatch")?;
    let rust_diagnostics = fs::read_to_string(directory.join("rust-mismatch.stderr"))?;
    let native_diagnostics =
        fs::read_to_string(directory.join("native-mismatch.stdout"))? + &fs::read_to_string(directory.join("native-mismatch.stderr"))?;
    if !native_ok
        || rust_rejection.code() != Some(1)
        || !rust_diagnostics.contains("E0080")
        || !rust_diagnostics.contains("assertion failed")
        || native_rejection.code() != Some(1)
        || !native_diagnostics.contains("\"code\":\"L0630\"")
        || !native_diagnostics.contains("const assertion failed")
    {
        return Err(format!("{target}: layout acceptance or semantic rejection failed; see retained diagnostics").into());
    }
    println!("PASS {target}: {} size/alignment assertions and semantic rejection", TYPES.len() * 2);
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 4 {
        return Err("usage: native-target-layout-qualification LUST RUSTC NEW_OUTPUT_DIR TARGET...".into());
    }
    let lust = fs::canonicalize(&args[0])?;
    let rustc = fs::canonicalize(&args[1])?;
    let output = PathBuf::from(&args[2]);
    fs::create_dir(&output)?;
    let output = fs::canonicalize(output)?;
    let compiler_version = Command::new(&rustc).arg("-vV").output()?;
    if !compiler_version.status.success() {
        return Err("could not identify the Rust oracle".into());
    }
    fs::write(output.join("rustc-version.txt"), compiler_version.stdout)?;
    let native_version = Command::new(&lust).arg("--version").output()?;
    if !native_version.status.success() {
        return Err("could not identify the native compiler".into());
    }
    fs::write(output.join("native-version.txt"), native_version.stdout)?;
    let mut failures = Vec::new();
    for target in &args[3..] {
        let target = target.to_str().ok_or("target is not UTF-8")?;
        if let Err(error) = qualify(&lust, &rustc, &output, target) {
            eprintln!("FAIL {error}");
            failures.push(target.to_owned());
        }
    }
    if !failures.is_empty() {
        return Err(format!("frontend layout qualification failed: {}", failures.join(", ")).into());
    }
    println!("Qualified {} selected targets; foreign linking and execution remain separate.", args.len() - 3);
    Ok(())
}
