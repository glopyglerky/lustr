// Target Rust semantics, independent of the Rust compiler building Lust.
// Both this module's data and the C runtime tables come from one pinned SDK.
#[path = "unicode_data.rs"]
mod data;

pub use data::UNICODE_VERSION;

fn contains(table: &[(u32, u32)], c: char) -> bool {
    let cp = c as u32;
    table.binary_search_by(|&(lo, hi)| {
        if hi < cp { std::cmp::Ordering::Less }
        else if lo > cp { std::cmp::Ordering::Greater }
        else { std::cmp::Ordering::Equal }
    }).is_ok()
}

pub fn is_alphabetic(c: char) -> bool { contains(data::ALPHABETIC, c) }
pub fn is_numeric(c: char) -> bool { contains(data::NUMERIC, c) }
pub fn is_alphanumeric(c: char) -> bool { is_alphabetic(c) || is_numeric(c) }
pub fn is_lowercase(c: char) -> bool { contains(data::LOWERCASE, c) }
pub fn is_uppercase(c: char) -> bool { contains(data::UPPERCASE, c) }
pub fn is_whitespace(c: char) -> bool { contains(data::WHITESPACE, c) }
pub fn is_control(c: char) -> bool { contains(data::CONTROL, c) }

fn push_escape(out: &mut String, c: char, single_quote: bool, double_quote: bool) {
    match c {
        '\t' => out.push_str("\\t"),
        '\r' => out.push_str("\\r"),
        '\n' => out.push_str("\\n"),
        '\0' => out.push_str("\\0"),
        '\\' => out.push_str("\\\\"),
        '\'' if single_quote => out.push_str("\\'"),
        '"' if double_quote => out.push_str("\\\""),
        _ if contains(data::ESCAPE, c) => {
            use std::fmt::Write as _;
            write!(out, "\\u{{{:x}}}", c as u32).unwrap();
        }
        _ => out.push(c),
    }
}

/// Rust's char/str escape_debug, including both quote escapes.
pub fn escape_debug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() { push_escape(&mut out, c, true, true); }
    out
}

/// Rust's Debug representation for one character, including its quotes.
pub fn debug_char(c: char) -> String {
    let mut out = String::from("'");
    push_escape(&mut out, c, true, false);
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn unicode_semantics_survive_retained_body_and_constant_edits() {
        std::thread::Builder::new().stack_size(crate::pool::STACK_SIZE).spawn(|| {
            use crate::{driver, protocol};
            use std::{fs, path::PathBuf, process::Command};
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            struct Owned(PathBuf);
            impl Drop for Owned { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
            let owner = loop {
                let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!("lust-unicode-retained-{}-{id}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => break Owned(path),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(e) => panic!("create test directory: {e}"),
                }
            };
            let file = owner.0.join("main.rs");
            let key = protocol::SessionKey::synthetic(&file, false);
            let mut session = driver::Session::new(owner.0.join("objects"), true, key.clone());
            let req = protocol::Request {
                cmd: protocol::Cmd::Build, session: key, file: file.clone(), out: owner.0.join("program"),
                json: false, timings: true, debug: false, emit_c: None, max_errors: 20,
                run_args: None, harness: false, cc: false, library_options: Default::default(), client: protocol::Client::default(),
            };
            for (phase, cp, tag, expected) in [
                ("cold", 0x558u32, 0, true), ("unchanged", 0x558, 0, true),
                ("body edit", 0x558, 1, true), ("constant edit", 0x557, 2, false),
                ("revert", 0x558, 3, true),
            ] {
                if phase != "unchanged" {
                    let source = format!(r#"const VALUE: bool = '\u{{{cp:x}}}'.is_lowercase();
fn main() {{
    let c = std::env::args().nth(1).unwrap().chars().next().unwrap();
    println!("{tag} {{}} {{}}", VALUE, c.is_lowercase());
}}
"#);
                    fs::write(&file, source).unwrap();
                }
                let response = driver::compile(&req, &mut session);
                assert_eq!(response.code, 0, "{phase}: {}", response.stderr);
                let output = Command::new(&req.out).arg(char::from_u32(cp).unwrap().to_string()).output().unwrap();
                assert!(output.status.success(), "{phase}: {:?}", output);
                assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("{tag} {expected} {expected}\n"), "{phase}");
                assert!(output.stderr.is_empty(), "{phase}: {:?}", output.stderr);
            }
        }).unwrap().join().unwrap();
    }
}
