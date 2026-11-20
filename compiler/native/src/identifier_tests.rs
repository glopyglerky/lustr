use super::*;

#[test]
fn normalized_identifier_tokens_preserve_spans_rawness_and_packing() {
    let interner = Interner::default();
    let cases = [
        ("café", "café", false, false),
        ("cafe\u{301}", "café", false, false),
        ("r#café", "café", true, false),
        ("r#cafe\u{301}", "café", true, false),
        ("'é", "é", false, true),
        ("'e\u{301}", "é", false, true),
        ("'r#é", "é", true, true),
        ("'r#e\u{301}", "é", true, true),
        ("a\u{315}\u{300}", "à\u{315}", false, false),
        ("à\u{315}", "à\u{315}", false, false),
        ("\u{1100}\u{1161}", "가", false, false),
        ("가", "가", false, false),
        ("Å", "Å", false, false),
        ("Å", "Å", false, false),
        ("x\u{301}", "x\u{301}", false, false),
    ];
    let source = cases.iter().map(|c| c.0).collect::<Vec<_>>().join(" ");
    let file = FileId(7);
    let base = 19;
    let lexed = lex(&source, file, base, &interner).unwrap();
    assert_eq!(lexed.tokens.len(), cases.len() + 1);
    let mut offset = base;
    for (token, (written, canonical, raw, lifetime)) in lexed.tokens.iter().zip(cases) {
        let expected = if lifetime { Tok::Lifetime(interner.intern(canonical), raw) } else { Tok::Ident(interner.intern(canonical), raw) };
        assert_eq!(token.tok, expected, "{written}");
        assert_eq!(token.span, Span::new(file, offset, offset + written.len() as u32), "{written}");
        offset += written.len() as u32 + 1;
    }
    let restored = PackedLexed::pack(&lexed).unpack();
    assert_eq!(restored.strings, lexed.strings);
    assert_eq!(restored.tokens.len(), lexed.tokens.len());
    for (actual, expected) in restored.tokens.iter().zip(&lexed.tokens) {
        assert_eq!((actual.tok, actual.span), (expected.tok, expected.span));
    }
}

#[test]
fn nfc_identifiers_survive_retained_edits_and_rejected_duplicates() {
    std::thread::Builder::new().stack_size(crate::pool::STACK_SIZE).spawn(|| {
        use crate::{driver, protocol};
        use std::{fs, path::PathBuf, process::Command};
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        struct Owned(PathBuf);
        impl Drop for Owned { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let owner = loop {
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("lust-identifier-nfc-{}-{id}", std::process::id()));
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
        // The two middle edits have the same source byte length but different
        // rawness. Name, span length or normalized spelling alone cannot key
        // the macro result correctly.
        assert_eq!("cafe\u{301}e\u{301}".len(), "r#caféé".len());
        for (phase, module, function, argument, arm, text) in [
            ("cold", "résumé", "café", "caféé", 1, "caféé"),
            ("unchanged", "résumé", "café", "caféé", 1, "caféé"),
            ("decomposed body", "résumé", "café", "cafe\u{301}e\u{301}", 1, "caféé"),
            ("raw body at equal byte length", "résumé", "café", "r#caféé", 2, "r#caféé"),
            ("decomposed declarations", "re\u{301}sume\u{301}", "cafe\u{301}", "r#cafe\u{301}e\u{301}", 2, "r#caféé"),
            ("revert", "résumé", "café", "caféé", 1, "caféé"),
        ] {
            let source = format!(r#"const VALEUR_é: u32 = 7;
mod {module} {{ pub const fn {function}(v: u32) -> u32 {{ v + crate::VALEUR_é }} }}
const VALUE: u32 = résumé::café(9);
macro_rules! choose {{ (caféé) => {{ 1u32 }}; (r#caféé) => {{ 2u32 }}; }}
fn main() {{
    println!("{{}} {{}} {{}} {{}} {{}}", VALUE, résumé::café(9), choose!({argument}), stringify!({argument}), "é");
}}
"#);
            if phase != "unchanged" { fs::write(&file, &source).unwrap(); }
            let response = driver::compile(&req, &mut session);
            assert_eq!(response.code, 0, "{phase}: {}", response.stderr);
            let output = Command::new(&req.out).output().unwrap();
            assert!(output.status.success(), "{phase}: {output:?}");
            assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("16 16 {arm} {text} e\u{301}\n"), "{phase}");
            assert!(output.stderr.is_empty(), "{phase}: {:?}", output.stderr);
            if phase == "decomposed declarations" {
                let last_success = fs::read(&req.out).unwrap();
                fs::write(&file, format!("{source}\nfn é() {{}}\nfn é() {{}}\n")).unwrap();
                let rejected = driver::compile(&req, &mut session);
                assert_ne!(rejected.code, 0, "canonically duplicate functions must reject");
                assert!(rejected.stderr.contains("L0106"), "{}", rejected.stderr);
                assert_eq!(fs::read(&req.out).unwrap(), last_success, "a rejected edit must preserve the last successful artifact");
            }
        }
    }).unwrap().join().unwrap();
}
