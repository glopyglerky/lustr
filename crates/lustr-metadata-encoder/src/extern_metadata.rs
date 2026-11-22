// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Bounded reads from rustc_metadata's v12 decoder and table formats.
// Source slices and exact version are recorded in source-slices.json.
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};

use crate::{Sdk, TARGET, upstream::{predefined_symbols::PREDEFINED_SYMBOLS, schema}};

struct Read<'a> {
    bytes: &'a [u8],
    at: usize,
    node: usize,
    previous: Option<usize>,
}

impl<'a> Read<'a> {
    fn new(bytes: &'a [u8], at: usize) -> Self {
        Self { bytes, at, node: at, previous: None }
    }

    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(len)?;
        let part = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(part)
    }

    fn byte(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn leb(&mut self) -> Option<usize> {
        let mut result = 0usize;
        for shift in (0..usize::BITS).step_by(7) {
            let byte = self.byte()?;
            let low = usize::from(byte & 127);
            if low > usize::MAX >> shift {
                return None;
            }
            result |= low << shift;
            if byte < 128 {
                return Some(result);
            }
        }
        None
    }

    fn text(&mut self) -> Option<&'a str> {
        let len = self.leb()?;
        let text = core::str::from_utf8(self.take(len)?).ok()?;
        (self.byte()? == 0xc1).then_some(text)
    }

    fn symbol(&mut self) -> Option<&'a str> {
        match self.byte()? {
            0 => self.text(),
            1 => {
                let target = self.leb()?;
                (target < self.at).then_some(())?;
                Self::new(self.bytes, target).text()
            }
            2 => PREDEFINED_SYMBOLS.get(self.leb()?).copied(),
            _ => None,
        }
    }

    fn lazy(&mut self) -> Option<usize> {
        let distance = self.leb()?;
        let at = match self.previous {
            None => self.node.checked_sub(distance)?,
            Some(previous) => previous.checked_add(distance)?,
        };
        (0 < at && at <= self.node).then_some(())?;
        self.previous = Some(at);
        Some(at)
    }

    fn array(&mut self) -> Option<(usize, usize)> {
        let len = self.leb()?;
        if len == 0 {
            return Some((0, 0));
        }
        Some((self.lazy()?, len))
    }

    fn integer_type(&mut self) -> Option<crate::IntegerType> {
        // rustc_middle::ty::codec reserves high-bit tags for abbreviations.
        // An abbreviation must point to a prior direct primitive tag pair.
        let start = self.at;
        if *self.bytes.get(start)? < 128 {
            return crate::IntegerType::from_metadata_tags(self.take(2)?);
        }
        let target = self.leb()?.checked_sub(128)?;
        let end = target.checked_add(2)?;
        (end <= start).then_some(())?;
        crate::IntegerType::from_metadata_tags(self.bytes.get(target..end)?)
    }
}

struct Table {
    at: usize,
    packed: bool,
    width: usize,
    len: usize,
}

impl Table {
    fn read(reader: &mut Read<'_>) -> Option<Self> {
        let width = reader.leb()?;
        let len = reader.leb()?;
        Some(Self { at: reader.lazy()?, packed: width & 1 != 0, width: width >> 1, len })
    }

    fn value<const N: usize>(&self, bytes: &[u8], mut index: usize) -> Option<[u8; N]> {
        let mut value = [0; N];
        if index >= self.len || self.width == 0 {
            return Some(value);
        }
        (self.width <= N).then_some(())?;
        let mut start = self.at;
        if self.packed {
            let entry = start.checked_add((index / 64).checked_mul(12)?)?;
            let mut cursor = Read::new(bytes, entry);
            let bitmap = u64::from_le_bytes(cursor.take(8)?.try_into().ok()?);
            let base = u32::from_le_bytes(cursor.take(4)?.try_into().ok()?) as usize;
            let bit = 1u64 << (index % 64);
            if bitmap & bit == 0 {
                return Some(value);
            }
            index = base.checked_add((bitmap & (bit - 1)).count_ones() as usize)?;
            start = start.checked_add(self.len.div_ceil(64).checked_mul(12)?)?;
        }
        start = start.checked_add(index.checked_mul(self.width)?)?;
        value[..self.width].copy_from_slice(bytes.get(start..start.checked_add(self.width)?)?);
        Some(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ScalarFunction<'a> {
    pub name: &'a str,
    pub parameters: Vec<crate::IntegerType>,
    pub result: crate::IntegerType,
    pub path_hash: u64,
}

#[derive(Debug)]
pub struct ScalarCrate<'a> {
    pub name: &'a str,
    pub stable_id: u64,
    pub svh: [u8; 16],
    pub filename_suffix: &'a str,
    pub functions: Vec<ScalarFunction<'a>>,
    pub dependencies: Vec<ScalarDependency<'a>>,
}

#[derive(Debug)]
pub struct ScalarDependency<'a> {
    pub name: &'a str,
    pub svh: [u8; 16],
    pub filename_suffix: &'a str,
}

impl ScalarCrate<'_> {
    pub fn symbol(&self, name: &str) -> String {
        crate::identity::function_symbol(name, self.name, self.stable_id)
    }
}

/// Read the supported scalar interface from full metadata on the pinned host.
/// The caller joins the crate name, filename and actual Cargo graph before exposing it.
pub fn read_scalar_crate<'a>(bytes: &'a [u8], sdk: &Sdk) -> Option<ScalarCrate<'a>> {
    read_crate(bytes, sdk, false)
}

/// Preserve non-SDK dependency identities so the executable loader can bind
/// them to actual Cargo artifacts before publishing any definitions.
pub fn read_link_crate<'a>(bytes: &'a [u8], sdk: &Sdk) -> Option<ScalarCrate<'a>> {
    read_crate(bytes, sdk, true)
}

fn read_crate<'a>(bytes: &'a [u8], sdk: &Sdk, link: bool) -> Option<ScalarCrate<'a>> {
    let sdk_dependencies = &sdk.dependencies;
    let file = bytes.strip_suffix(b"rust-end-file")?;
    (file.starts_with(schema::METADATA_HEADER)).then_some(())?;
    let mut prefix = Read::new(file, 8);
    let root = native_size(prefix.take(8)?)?;
    let unhashed = native_size(prefix.take(8)?)?;
    let svh = prefix.take(16)?.try_into().ok()?;
    (prefix.text()?.strip_prefix("rustc ")? == sdk.cfg_version && prefix.at < root && root < unhashed && unhashed < file.len())
        .then_some(())?;

    let mut unhashed_tail = Read::new(file, unhashed);
    let filename_suffix = unhashed_tail.text()?;
    let suffix_count = unhashed_tail.leb()?.checked_sub(1)?;
    (suffix_count <= 4096 && unhashed_tail.text()?.is_empty()).then_some(())?;
    let suffixes = (0..suffix_count).map(|_| unhashed_tail.text()).collect::<Option<Vec<_>>>()?;
    (unhashed_tail.at == file.len()).then_some(())?;

    let data = file.get(..unhashed)?;
    let mut r = Read::new(data, root);
    (r.byte()? == 0 && r.text()? == TARGET).then_some(())?;
    let name = r.symbol()?;
    valid_identifier(name).then_some(())?;
    (r.take(2)? == [0, 0]).then_some(())?;
    let stable_id = u64::from_le_bytes(r.take(8)?.try_into().ok()?);
    // Panic abort, unwind-in-drop default, edition 2024, no allocator or panic handler.
    (r.take(8)? == [1, 1, 0, 3, 0, 0, 0, 0] && r.array()?.1 == 0).then_some(())?;
    let (deps_position, dep_count) = r.array()?;
    (dep_count == suffix_count && (link || dep_count == sdk_dependencies.len())).then_some(())?;
    let mut deps = Read::new(data, deps_position);
    let mut dependencies = Vec::new();
    let mut seen = BTreeSet::new();
    let mut matched = 0;
    for (index, filename_suffix) in suffixes.into_iter().enumerate() {
        let name = deps.symbol()?;
        let svh = deps.take(16)?.try_into().ok()?;
        (deps.byte()? == 0 && seen.insert(name)).then_some(())?;
        let (kind, is_private) = (deps.byte()?, deps.byte()?);
        let expected = if link { sdk_dependencies.iter().find(|dep| dep.name == name) }
            else { sdk_dependencies.get(index) };
        if let Some(expected) = expected {
            (expected.name == name && expected.hash == svh && expected.kind == kind
                && u8::from(expected.is_private) == is_private && expected.extra_filename == filename_suffix).then_some(())?;
            matched += 1;
        } else {
            (link && valid_identifier(name) && kind == 2 && is_private == 0).then_some(())?;
            dependencies.push(ScalarDependency { name, svh, filename_suffix });
        }
    }
    (matched == sdk_dependencies.len()).then_some(())?;
    for field in &schema::ROOT_FIELDS[11..26] {
        let (at, len) = r.array()?;
        if *field == "lang_items_missing" {
            (len == 1 && data.get(at) == Some(&151)).then_some(())?;
        } else {
            (len == 0).then_some(())?;
        }
    }
    (r.byte()? == 0).then_some(())?;
    let tables = schema::TABLE_FIELDS.iter().map(|name| Some((*name, Table::read(&mut r)?))).collect::<Option<BTreeMap<_, _>>>()?;
    // Empty visualizers, export overrides and generic export inventory.
    (r.array()?.1 == 0 && r.array()?.1 == 0 && r.array()?.1 == 0).then_some(())?;
    r.array()?; // A full link may have non-generic exports. Check reads declarations.
    (r.array()?.1 == 0).then_some(())?;
    for _ in 0..3 {
        Table::read(&mut r)?;
    } // Hygiene tables.
    r.lazy()?; // DefPathHash lookup map.
    Table::read(&mut r)?; // Source map.
    let (modifiers, count) = r.array()?;
    (count == 1 && Read::new(data, modifiers).take(3)? == [2, 0, 0xc1]).then_some(())?;
    let (mitigations, count) = r.array()?;
    (count == 2 && Read::new(data, mitigations).take(6)? == [0, 1, 0, 1, 0, 0]).then_some(())?;
    // No special runtime roles or specialization, symbol ABI is original V0.
    (r.take(8)? == [0, 0, 0, 0, 0, 0, 1, 0] && r.at == unhashed).then_some(())?;
    let table = |name| tables.get(name);
    let value = |name, index| table(name)?.value::<16>(data, index);
    let position = |name, index| native_size(&value(name, index)?[..8]);
    let array_len = |name, index| read_interleaved(value(name, index)?).map(|(_, len)| len);
    let (children_at, child_count) = read_interleaved(value("module_children_non_reexports", 0)?)?;
    (child_count > 0
        && child_count <= 65_535
        && only_implicit_core_reexport(data, value("module_children_reexports", 0)?)?
        && array_len("ambig_module_children", 0)? == 0
        && array_len("attributes", 0)? == 0)
        .then_some(())?;
    let mut children = Read::new(data, children_at);
    let mut indices = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut functions = Vec::new();
    for _ in 0..child_count {
        let index = children.leb()?;
        (index != 0 && indices.insert(index) && table("def_kind")?.value::<1>(data, index)? == [12]).then_some(())?;
        let visibility_offset = position("visibility", index)?;
        match *data.get(visibility_offset)? {
            0 => {}
            1 => continue,
            _ => return None,
        }
        let mut key = Read::new(data, position("def_keys", index)?);
        (key.take(3)? == [1, 0, 6]).then_some(())?;
        let function_name = key.symbol()?;
        (valid_identifier(function_name) && names.insert(function_name) && key.leb()? == 0).then_some(())?;

        (array_len("attributes", index)? == 0
            && table("constness")?.value::<1>(data, index)? == [1]
            && Read::new(data, position("generics_of", index)?).take(7)? == [0; 7]
            && Read::new(data, position("explicit_clauses_of", index)?).take(2)? == [0; 2])
            .then_some(())?;
        let mut codegen = Read::new(data, position("codegen_fn_attrs", index)?);
        (codegen.take(13)? == [0; 13] && codegen.byte()? == 2 && codegen.take(6)? == [0; 6]).then_some(())?;
        let mut signature = Read::new(data, position("fn_sig", index)?);
        (signature.leb()? == 0).then_some(())?;
        let count = signature.leb()?;
        (0 < count && count <= 65).then_some(())?;
        let parameters = (1..count).map(|_| signature.integer_type()).collect::<Option<Vec<_>>>()?;
        let result = signature.integer_type()?;
        (signature.take(2)? == [66, 255]).then_some(())?;
        let path_hash = u64::from_le_bytes(table("def_path_hashes")?.value(data, index)?);
        (path_hash == crate::identity::function_path_hash(stable_id, function_name)).then_some(())?;
        functions.push(ScalarFunction { name: function_name, parameters, result, path_hash });
    }
    Some(ScalarCrate { name, stable_id, svh, filename_suffix, functions, dependencies })
}

fn only_implicit_core_reexport(data: &[u8], value: [u8; 16]) -> Option<bool> {
    let (offset, count) = read_interleaved(value)?;
    match count {
        0 => Some(true),
        1 => {
            let mut child = Read::new(data, offset);
            (child.symbol()? == "core").then_some(())?;
            // Discard the observed indirect span without using it as semantic authority.
            let tag = child.byte()?;
            let width = usize::from(tag >> 3);
            (tag & 3 == 3 && (1..=8).contains(&width)).then_some(())?;
            child.take(width)?;
            // Res::Def(Mod, core:root), restricted to current root, one ExternCrate def.
            Some(child.take(9)? == [0, 0, 1, 0, 1, 0, 0, 1, 2] && child.leb()? == 0 && child.leb()? != 0)
        }
        _ => Some(false),
    }
}

fn read_interleaved(pair: [u8; 16]) -> Option<(usize, usize)> {
    let mut at = [0; 8];
    let mut len = [0; 8];
    for i in 0..8 {
        at[i] = pair[2 * i];
        len[i] = pair[2 * i + 1];
    }
    Some((native_size(&at)?, native_size(&len)?))
}

fn native_size(value: &[u8]) -> Option<usize> {
    usize::try_from(u64::from_le_bytes(value.try_into().ok()?)).ok()
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().enumerate().all(|(index, byte)| byte == b'_' || byte.is_ascii_alphabetic() || index > 0 && byte.is_ascii_digit())
}

/// Match a real full metadata artifact with the metadata carried by its archive.
pub fn archive_metadata_matches(full: &[u8], embedded: &[u8], name: &str) -> bool {
    if full == embedded { return true; }
    let matches = || -> Option<()> {
        let stub = embedded.strip_suffix(b"rust-end-file")?;
        (stub.starts_with(schema::METADATA_HEADER) && stub.get(16..24)? == [0; 8]
            && stub.get(24..40)? == full.get(24..40)?).then_some(())?;
        let root = native_size(stub.get(8..16)?)?;
        let mut prefix = Read::new(stub, 40);
        (prefix.text()? == Read::new(full, 40).text()? && prefix.at == root).then_some(())?;
        let mut header = Read::new(stub, root);
        (header.byte()? == 0 && header.text()? == TARGET && header.symbol()? == name
            && header.take(2)? == [0, 1] && header.at == stub.len()).then_some(())
    };
    matches().is_some()
}

pub fn archive_object_members(data: &[u8]) -> Option<Vec<&str>> {
    let payload = data.strip_suffix(b"rust-end-file")?;
    let mut r = Read::new(payload, 0);
    let count = r.leb()?;
    (count <= payload.len()).then_some(())?;
    let objects = (0..count).map(|_| r.text()).collect::<Option<Vec<_>>>()?;
    // This deliberately shares the admitted original native-lib inventory.
    (r.leb()? == 0 && r.at == payload.len()).then_some(objects)
}
