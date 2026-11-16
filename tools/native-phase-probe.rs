//! Direct native ABI probe: no compatibility fallback and no application retry.
#[path = "../src/native_link.rs"]
mod native_link;

fn main() {
    use std::io::Write;
    let mut args = std::env::args_os().skip(1);
    let library = std::path::PathBuf::from(args.next().expect("native-phase-probe LIBRARY LUST_ARGS..."));
    let args: Vec<String> = args.map(|a| a.into_string().expect("native protocol requires UTF-8 arguments")).collect();
    let before: Vec<_> = std::env::vars_os().collect();
    let (code, stdout, stderr) = native_link::compile(&library, &args).expect("native entry unavailable").expect("native entry failed");
    assert_eq!(std::env::vars_os().collect::<Vec<_>>(), before, "native ABI mutated the process environment");
    std::io::stdout().write_all(&stdout).unwrap();
    std::io::stderr().write_all(&stderr).unwrap();
    std::process::exit(code);
}
