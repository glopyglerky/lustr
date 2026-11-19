// Compiled unchanged by native and matching Rust. Compare every output byte,
// not a finite hash: predicates, case conversion, Debug and final-sigma contexts.
use std::io::Write;

fn number(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

fn text(out: &mut Vec<u8>, s: &str) {
    number(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}

fn main() {
    let mut out = Vec::new();
    let mut stdout = std::io::stdout();
    for cp in 0u32..=0x10ffff {
        if let Some(c) = char::from_u32(cp) {
            number(&mut out, cp);
            let predicates = [c.is_alphabetic(), c.is_numeric(), c.is_alphanumeric(), c.is_lowercase(), c.is_uppercase(), c.is_whitespace(), c.is_control()];
            for value in predicates { out.push(value as u8); }
            text(&mut out, &c.to_uppercase().collect::<String>());
            text(&mut out, &c.to_lowercase().collect::<String>());
            text(&mut out, &format!("{c:?}"));
            text(&mut out, &format!("{:?}", format!("a{c}b")));
            text(&mut out, &format!("{c}Σ").to_lowercase());
            text(&mut out, &format!("A{c}Σ").to_lowercase());
            text(&mut out, &format!("ΑΣ{c}").to_lowercase());
            text(&mut out, &format!("ΑΣ{c}A").to_lowercase());
            if out.len() >= 65536 {
                stdout.write_all(&out).unwrap();
                out.clear();
            }
        }
    }
    stdout.write_all(&out).unwrap();
}
