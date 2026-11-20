//! Source-owned metadata serialization for a completed native scalar library.
//!
//! This crate does not parse or accept Rust programs. Its separate research
//! caller owns a private native completion token and emitted object, and
//! converts their immutable facts into `Input`. The authored public retained scalar-library caller
//! shares that producer; qualification remains source-bound.
//! The encoder checks the supplied inventory and
//! preserves the pinned Rust metadata schema. It does not contain a Rust
//! frontend, type checker, MIR builder, compiler driver, or fallback route.

#![no_std]
#[macro_use]
extern crate alloc;
#[cfg(test)]
extern crate std;

mod encode;
mod identity;
mod sdk;
mod table;
mod upstream {
    #![allow(dead_code)]
    pub(crate) mod codec;
    pub(crate) mod int_overflow;
    pub(crate) mod leb128;
    pub(crate) mod mem_encoder;
    pub(crate) mod schema;
    pub(crate) mod stable_hash;
    pub(crate) mod odht_writer;
    pub(crate) mod odht_groups;
}

use alloc::{collections::{BTreeMap, BTreeSet}, string::String, vec::Vec};

pub use sdk::{Sdk, SdkDependency};

pub const RUST_REVISION: &str = "5b7b37aefcb5b2fba3c2392bc9065b9e4602d494";
pub const TARGET: &str = "x86_64-unknown-linux-gnu";

/// Native identities are retained verbatim. They are never cast into Rust IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativeId {
    pub krate: u64,
    pub path: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub lo: u32,
    pub hi: u32,
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub span: Span,
}

/// A completed ordinary, safe, nongeneric `(u64, ...) -> u64` declaration.
/// Absence of other declarations, attributes and body obligations is proved by
/// the native completion producer; a success exit code cannot create that proof.
#[derive(Clone, Debug)]
pub struct Function {
    pub definition: NativeId,
    pub name: String,
    pub name_span: Span,
    pub span: Span,
    pub public: bool,
    pub parameters: Vec<Parameter>,
    pub calls: Vec<NativeId>,
}

#[derive(Clone, Debug)]
pub struct EmittedFunction {
    pub definition: NativeId,
    pub symbol: String,
}

/// The profile represented by this first source-owned seam. These flags come
/// from the original request; this interface does not rewrite its optimization,
/// panic, debug, MIR-sharing or inlining settings to avoid an obligation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub overflow_checks: bool,
    pub debug_assertions: bool,
}

/// An actual native producer object, its planned archive member name, and the
/// source definitions its symbol table owns. The digest precedes verified Rust
/// symbol renaming; the host separately binds final renamed member bytes.
#[derive(Clone, Debug)]
pub struct ObjectMember {
    pub member: String,
    pub producer_object_sha256: [u8; 32],
    pub definitions: Vec<NativeId>,
}

pub struct Input<'a> {
    pub crate_name: &'a str,
    pub metadata: &'a str,
    pub extra_filename: &'a str,
    /// Actual unstable -Z request; absent and explicit true remain distinct tracked facts.
    pub embed_metadata: Option<bool>,
    pub source_path: &'a str,
    pub working_directory: &'a str,
    pub source: &'a str,
    pub root_span: Span,
    pub profile: Profile,
    pub functions: &'a [Function],
    pub emitted: &'a [EmittedFunction],
    pub backend_helpers: &'a [String],
    /// Canonically ordered producer objects; definitions partition `emitted`.
    pub objects: &'a [ObjectMember],
}

#[derive(Clone, Debug)]
pub struct SymbolMapping {
    pub definition: NativeId,
    pub rust_def_index: u32,
    pub rust_def_path_hash: u64,
    pub native_symbol: String,
    pub rust_symbol: String,
    pub public: bool,
}

pub struct Encoded {
    pub metadata: Vec<u8>,
    /// Split rlibs contain this header-only stub, bound to the full metadata SVH.
    pub metadata_stub: Option<Vec<u8>>,
    pub stable_crate_id: u64,
    pub crate_hash: [u8; 16],
    pub symbols: Vec<SymbolMapping>,
}

pub fn encode(input: &Input<'_>, sdk: &Sdk) -> Result<Encoded, String> {
    validate(input)?;
    encode::encode(input, sdk)
}

/// Original `RmetaLink::encode` for the actual Rust object members of the
/// admitted library. The completed scope has no native library declarations;
/// an empty member list cannot stand in for the emitted object inventory.
pub fn encode_rlib_link(object_members: &[&str]) -> Result<Vec<u8>, String> {
    use upstream::codec::Encoder;
    let mut seen = BTreeSet::new();
    if object_members.is_empty() || object_members.len() > 65_535 {
        return Err("rlib link metadata requires the actual nonempty object member inventory".into());
    }
    for name in object_members {
        if name.is_empty() || name.len() > 255 || !name.ends_with(".o")
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            || !seen.insert(*name)
        {
            return Err("rlib object member name is malformed or duplicated".into());
        }
    }
    let mut encoder = upstream::mem_encoder::MemEncoder::new();
    encoder.emit_usize(object_members.len());
    for name in object_members { encoder.emit_str(name); }
    encoder.emit_usize(0); // Native libraries are absent in the completed scope.
    let mut bytes = encoder.finish();
    bytes.extend_from_slice(b"rust-end-file");
    Ok(bytes)
}

fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().enumerate().all(|(i, c)| {
            c == b'_' || c.is_ascii_alphabetic() || (i != 0 && c.is_ascii_digit())
        })
}

fn validate(input: &Input<'_>) -> Result<(), String> {
    let failure = |reason: &str| format!("native metadata completion: {reason}");
    if !identifier(input.crate_name)
        || input.metadata.is_empty()
        || input.metadata.len() > 128
        || !input.metadata.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(failure("invalid original crate identity"));
    }
    if input.extra_filename.len() > 128
        || !(input.extra_filename.is_empty() || input.extra_filename.starts_with('-'))
        || !input.extra_filename.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c)) {
        return Err(failure("invalid original extra-filename"));
    }
    if !input.source_path.starts_with('/') || input.source_path.contains('\0')
        || !input.working_directory.starts_with('/') || input.working_directory.contains('\0')
    {
        return Err(failure("absolute Unicode source path is required"));
    }
    if input.source.len() > 2 * 1024 * 1024
        || !input.source.is_ascii()
        || input.source.as_bytes().contains(&b'\r')
        || input.root_span != (Span { lo: 0, hi: input.source.len() as u32 })
    {
        return Err(failure("source normalization/span context is unrepresented"));
    }
    if input.functions.is_empty() || input.functions.len() > 65_535 {
        return Err(failure("a nonempty bounded completed function inventory is required"));
    }
    let mut definitions = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut last_span_end = 0;
    for function in input.functions {
        let text = |span: Span| input.source.get(span.lo as usize..span.hi as usize);
        if !identifier(&function.name)
            || !names.insert(&function.name)
            || definitions.insert(function.definition, function).is_some()
            || function.span.lo < last_span_end
            || function.span.lo >= function.span.hi
            || text(function.span).is_none()
            || function.name_span.lo < function.span.lo
            || function.name_span.hi > function.span.hi
            || text(function.name_span) != Some(function.name.as_str())
            || function.parameters.len() > 254
        {
            return Err(failure("definition identity, order, name or span differs from completed source"));
        }
        last_span_end = function.span.hi;
        let mut parameters = BTreeSet::new();
        for parameter in &function.parameters {
            if !identifier(&parameter.name)
                || !parameters.insert(&parameter.name)
                || parameter.span.lo < function.name_span.hi
                || parameter.span.hi > function.span.hi
                || text(parameter.span) != Some(parameter.name.as_str())
            {
                return Err(failure("parameter identity/span differs from completed source"));
            }
        }
    }
    let mut emitted = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for instance in input.emitted {
        if !definitions.contains_key(&instance.definition)
            || !emitted.insert(instance.definition)
            || !identifier(&instance.symbol)
            || !symbols.insert(&instance.symbol)
        {
            return Err(failure("emitted instance is missing, duplicated, foreign or malformed"));
        }
    }
    if emitted.len() != definitions.len() {
        return Err(failure("an accepted definition has no actual emitted instance"));
    }
    let member_names: Vec<_> = input.objects.iter().map(|object| object.member.as_str()).collect();
    encode_rlib_link(&member_names)?;
    let mut object_definitions = BTreeSet::new();
    let mut previous = None;
    for object in input.objects {
        if object.definitions.is_empty() || object.definitions.windows(2).any(|pair|pair[0]>=pair[1])
            || previous.is_some_and(|name: &str| name>=object.member.as_str()) {
            return Err(failure("object member/definition inventory is not canonical"));
        }
        previous=Some(object.member.as_str());
        for definition in &object.definitions {
            if !emitted.contains(definition) || !object_definitions.insert(*definition) {
                return Err(failure("object definition is duplicated or has no emitted instance"));
            }
        }
    }
    if object_definitions!=emitted { return Err(failure("emitted instance has no exact object owner")); }
    for helper in input.backend_helpers {
        if !identifier(helper) || !symbols.insert(helper) {
            return Err(failure("backend helper overlaps a source definition"));
        }
    }
    // This is a graph-integrity check on actual checked call records, not body
    // checking or an alternative source parser. All records must name members
    // of the same completed/emitted generation.
    for function in input.functions {
        let mut calls = BTreeSet::new();
        for called in &function.calls {
            if !definitions.contains_key(called) || !calls.insert(called) {
                return Err(failure("call edge is foreign or duplicated"));
            }
        }
    }
    Ok(())
}
