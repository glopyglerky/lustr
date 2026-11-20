// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Adapted from rustc_span/src/def_id.rs, rustc_hir_id/src/definitions.rs,
// rustc_hir_id/src/def_path_hash_map.rs and rustc_symbol_mangling/src/v0.rs
// at 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494. See source-slices.json.
use alloc::string::String;
use core::hash::{Hash, Hasher};
use crate::upstream::stable_hash::{FromStableHash, SipHasher128Hash, StableSipHasher128};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Hash64(u64);
impl FromStableHash for Hash64 {
    type Hash = SipHasher128Hash;
    fn from(SipHasher128Hash([first, _]): Self::Hash) -> Self { Self(first) }
}

pub(crate) fn stable_crate_id(name: &str, metadata: &str, cfg_version: &str) -> u64 {
    let mut hasher = StableSipHasher128::new();
    name.hash(&mut hasher);
    // The admitted request has one explicit metadata value. There is no order
    // or duplicate to erase; this is StableCrateId::new's sorted unique input.
    hasher.write(b"metadata");
    hasher.write_usize(metadata.len());
    hasher.write(metadata.as_bytes());
    hasher.write(b"lib");
    // Compiler identity is an explicit retained SDK input, never a daemon's
    // mutable RUSTC_FORCE_RUSTC_VERSION environment value.
    hasher.write(cfg_version.as_bytes());
    hasher.finish::<Hash64>().0
}

// Preserve every original discriminant. Removing unused variants would change
// both the original DefKey encoding and its stable hash.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum DefPathData<'a> {
    CrateRoot, Impl, ForeignMod, Use, GlobalAsm, TypeNs(&'a str), ValueNs(&'a str),
    MacroNs(&'a str), LifetimeNs(&'a str), Closure, TestBinderConstraints, Ctor,
    AnonConst, OpaqueTy, OpaqueLifetime(&'a str), AnonAssocTy(&'a str),
    SyntheticCoroutineBody, NestedStatic,
}

pub(crate) fn function_path_hash(root_hash: u64, name: &str) -> u64 {
    let mut hasher = StableSipHasher128::new();
    Hash64(root_hash).hash(&mut hasher);
    core::mem::discriminant(&DefPathData::ValueNs(name)).hash(&mut hasher);
    name.hash(&mut hasher);
    0u32.hash(&mut hasher); // No duplicate declaration survived completion.
    hasher.finish::<Hash64>().0
}

/// Flat ordinary item specialization of the original V0SymbolMangler.
/// The parent is the real crate identity; the namespace is ValueNs; there are
/// no erased generic arguments, shims or invented export_name attributes.
pub(crate) fn function_symbol(name: &str, crate_name: &str, stable_id: u64) -> String {
    let mut out = String::from("_RNvC");
    // v0::push_disambiguator_fixed_len. Its fixed width is part of this pin.
    if let Some(x) = stable_id.checked_sub(1) {
        out.push('s');
        if let Some(x) = x.checked_sub(1) { out.push_str(&base62_fixed(x)); }
        out.push('_');
    }
    push_ident(crate_name, &mut out);
    // ValueNs disambiguator zero emits no bytes.
    push_ident(name, &mut out);
    out
}

fn base62_fixed(mut value: u64) -> String {
    // rustc_data_structures::base_n::ToBaseN specialized to u64/base 62.
    const ALPHABET: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut output = [b'0'; 11];
    let mut index = output.len();
    loop {
        index -= 1;
        output[index] = ALPHABET[(value % 62) as usize];
        value /= 62;
        if value == 0 { break; }
    }
    String::from_utf8(output.to_vec()).unwrap()
}

fn push_ident(ident: &str, output: &mut String) {
    use core::fmt::Write;
    // ASCII identifiers were established by native completion. The upstream
    // punycode branch is deliberately not represented by this first adapter.
    assert!(ident.bytes().all(|b| b == b'_' || b.is_ascii_alphanumeric()));
    let _ = write!(output, "{}", ident.len());
    if let Some('_' | '0'..='9') = ident.chars().next() { output.push('_'); }
    output.push_str(ident);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_pinned_reference_crate_identity_is_preserved() {
        assert_eq!(stable_crate_id("consumer", "native_consumer_v1", "1.102.0-nightly (5b7b37aef 2026-11-08)"), 5585881185830709363);
    }

    #[test]
    fn original_stable_hash_matches_core_namespace_extraction() {
        struct Original([u64; 2]);
        impl rustc_stable_hash::FromStableHash for Original {
            type Hash = rustc_stable_hash::SipHasher128Hash;
            fn from(rustc_stable_hash::SipHasher128Hash(words): Self::Hash) -> Self { Self(words) }
        }
        struct Adapted([u64; 2]);
        impl FromStableHash for Adapted {
            type Hash = SipHasher128Hash;
            fn from(SipHasher128Hash(words): Self::Hash) -> Self { Self(words) }
        }
        for len in [0, 1, 7, 8, 9, 15, 16, 17, 31, 32, 63, 64, 129, 1024, 8193] {
            let bytes: alloc::vec::Vec<_> = (0..len).map(|n| (n * 113) as u8).collect();
            let mut original = rustc_stable_hash::StableSipHasher128::new();
            let mut adapted = StableSipHasher128::new();
            for piece in bytes.chunks(7) { original.write(piece); adapted.write(piece); }
            original.write_usize(len); adapted.write_usize(len);
            original.write_u64(u64::MAX); adapted.write_u64(u64::MAX);
            "definition".hash(&mut original); "definition".hash(&mut adapted);
            assert_eq!(original.finish::<Original>().0, adapted.finish::<Adapted>().0);
        }
    }
}
