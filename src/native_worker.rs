//! One-shot matched native ABI child for the experimental frontend extension.
//! This executable never selects or invokes a compatibility compiler.
mod native_link;

fn main() {
    use std::io::Write;
    let Some(parent) = std::env::current_exe().ok().and_then(|p| p.parent().map(std::path::Path::to_owned)) else {
        std::process::exit(2);
    };
    let Some(args) = std::env::args_os().skip(1).map(|a| a.into_string().ok()).collect::<Option<Vec<_>>>() else {
        std::process::exit(2);
    };
    let response = native_link::compile(&parent.join("../lib").join(native_link::INFERENCE_LIBRARY), &args);
    match response {
        Some(Ok((code, stdout, stderr))) => {
            std::io::stdout().write_all(&stdout).expect("native stdout");
            std::io::stderr().write_all(&stderr).expect("native stderr");
            std::process::exit(code);
        }
        Some(Err(error)) => eprintln!("{error}"),
        None => eprintln!("matched native inference entry unavailable"),
    }
    std::process::exit(2);
}
