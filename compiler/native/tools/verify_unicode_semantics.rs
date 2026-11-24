// Independent finite-domain comparison with the exact imported Rust SDK.
// This tool is not linked into the compiler and is never built by Cargo.
#![feature(rustc_private)]
#![allow(dead_code)]
extern crate rustc_driver;
extern crate unicode_ident;
extern crate unicode_normalization;
#[path = "../src/unicode.rs"]
mod native;
#[path = "../src/xid_table.rs"]
mod xid;

fn contains(table: &[(u32, u32)], cp: u32) -> bool {
    table.binary_search_by(|&(lo, hi)| {
        if hi < cp { std::cmp::Ordering::Less }
        else if lo > cp { std::cmp::Ordering::Greater }
        else { std::cmp::Ordering::Equal }
    }).is_ok()
}

fn ccc(cp: u32) -> u8 {
    xid::CCC_RANGES.binary_search_by(|&(lo, hi, _)| {
        if hi < cp { std::cmp::Ordering::Less }
        else if lo > cp { std::cmp::Ordering::Greater }
        else { std::cmp::Ordering::Equal }
    }).map_or(0, |i| xid::CCC_RANGES[i].2 as u8)
}

fn decompose(cp: u32, out: &mut Vec<char>) {
    if (0xac00..0xac00 + 19 * 21 * 28).contains(&cp) {
        let s = cp - 0xac00;
        out.push(char::from_u32(0x1100 + s / (21 * 28)).unwrap());
        out.push(char::from_u32(0x1161 + s % (21 * 28) / 28).unwrap());
        if s % 28 != 0 { out.push(char::from_u32(0x11a7 + s % 28).unwrap()); }
    } else if let Ok(i) = xid::CANON_DECOMP.binary_search_by_key(&cp, |e| e.0) {
        let (_, first, second) = xid::CANON_DECOMP[i];
        decompose(first, out);
        if second != 0 { decompose(second, out); }
    } else {
        out.push(char::from_u32(cp).unwrap());
    }
}

fn main() {
    assert_eq!(native::UNICODE_VERSION, char::UNICODE_VERSION);
    let mut count = 0usize;
    for cp in 0..=0x10ffff {
        let Some(c) = char::from_u32(cp) else { continue };
        count += 1;
        assert_eq!([
            native::is_alphabetic(c), native::is_numeric(c), native::is_alphanumeric(c),
            native::is_lowercase(c), native::is_uppercase(c), native::is_whitespace(c), native::is_control(c),
        ], [c.is_alphabetic(), c.is_numeric(), c.is_alphanumeric(), c.is_lowercase(), c.is_uppercase(), c.is_whitespace(), c.is_control()], "predicates U+{cp:04X}");
        assert_eq!(native::escape_debug(&c.to_string()), c.escape_debug().to_string(), "escape U+{cp:04X}");
        assert_eq!(native::debug_char(c), format!("{c:?}"), "char Debug U+{cp:04X}");
        assert_eq!(contains(xid::XID_START_RANGES, cp), unicode_ident::is_xid_start(c), "XID start U+{cp:04X}");
        assert_eq!(contains(xid::XID_CONTINUE_RANGES, cp), unicode_ident::is_xid_continue(c), "XID continue U+{cp:04X}");
        assert_eq!(ccc(cp), unicode_normalization::char::canonical_combining_class(c), "combining class U+{cp:04X}");
        let mut actual = Vec::new();
        decompose(cp, &mut actual);
        let mut expected = Vec::new();
        unicode_normalization::char::decompose_canonical(c, |part| expected.push(part));
        assert_eq!(actual, expected, "canonical decomposition U+{cp:04X}");
    }
    for &(a, b, cp) in xid::COMPOSE_PAIRS {
        assert_eq!(unicode_normalization::char::compose(char::from_u32(a).unwrap(), char::from_u32(b).unwrap()), char::from_u32(cp), "composition U+{a:04X}, U+{b:04X}");
    }
    for &(_, a, b) in xid::CANON_DECOMP {
        if b == 0 { continue; }
        let actual = xid::COMPOSE_PAIRS.binary_search_by_key(&(a, b), |e| (e.0, e.1)).ok().and_then(|i| char::from_u32(xid::COMPOSE_PAIRS[i].2));
        assert_eq!(actual, unicode_normalization::char::compose(char::from_u32(a).unwrap(), char::from_u32(b).unwrap()), "composition completeness U+{a:04X}, U+{b:04X}");
    }
    assert_eq!(count, 1_112_064);
    println!("PASS {count} scalar values: 7 predicates, debug escaping, XID start/continue, combining classes, canonical decomposition; {} composition pairs and all decomposition-pair exclusions", xid::COMPOSE_PAIRS.len());
}
