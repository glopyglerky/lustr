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

    fn local_def(&mut self) -> Option<usize> {
        (self.leb()? == 0).then_some(())?;
        self.leb()
    }

    // Pinned rustc_middle::ty::codec reserves tags >= 128 for prior
    // type/predicate records. Read one record from the original cursor or
    // the original, un-abbreviated start of the referenced record.
    fn decode_abbreviated<T>(&mut self, decode: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        let start = self.at;
        if *self.bytes.get(start)? < 128 {
            return decode(self);
        }
        let target = self.leb()?.checked_sub(128)?;
        (target < start && *self.bytes.get(target)? < 128).then_some(())?;
        decode(&mut Self::new(self.bytes, target))
    }

    fn early_region(&mut self, lifetime: (usize, &str)) -> Option<()> {
        (self.byte()? == 0 && self.leb()? == lifetime.0 && self.symbol()? == lifetime.1).then_some(())
    }

    fn self_type(&mut self) -> Option<()> {
        self.decode_abbreviated(|ty| {
            (ty.byte()? == 24 && ty.leb()? == 0 && ty.symbol()? == "Self").then_some(())
        })
    }

    fn skip_span(&mut self) -> Option<()> {
        let tag = self.byte()?;
        let width = usize::from(tag >> 3);
        if tag & 3 == 3 {
            (width <= 8).then_some(())?;
            self.take(width)?;
        } else {
            if tag & 4 == 0 { self.leb()?; } // SyntaxContext
            if tag & 3 != 2 {
                self.leb()?; // BytePos
                if width == 31 { self.leb()?; }
                self.leb()?; // File
                if tag & 3 == 1 { self.leb()?; } // Foreign crate
            }
        }
        Some(())
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
    pub requires_abort: bool,
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

struct IndexedCrate<'a> {
    identity: ScalarCrate<'a>,
    data: &'a [u8],
    tables: BTreeMap<&'static str, Table>,
    traits: (usize, usize),
    trait_impls: (usize, usize),
}

impl IndexedCrate<'_> {
    fn value<const N: usize>(&self, name: &str, index: usize) -> Option<[u8; N]> {
        self.tables.get(name)?.value(self.data, index)
    }

    fn position(&self, name: &str, index: usize) -> Option<usize> {
        native_size(&self.value::<16>(name, index)?[..8])
    }

    fn array(&self, name: &str, index: usize) -> Option<(usize, usize)> {
        read_interleaved(self.value(name, index)?)
    }
}

pub(crate) fn read_core_size_traits(
    bytes: &[u8], expected_version: &str, expected_hash: &[u8; 16],
) -> Option<[usize; 2]> {
    let bytes = bytes.strip_suffix(b"rust-end-file")?;
    (bytes.starts_with(schema::METADATA_HEADER)).then_some(())?;
    let mut prefix = Read::new(bytes, 8);
    let root = native_size(prefix.take(8)?)?;
    let unhashed = native_size(prefix.take(8)?)?;
    (prefix.take(16)? == expected_hash && prefix.text()?.strip_prefix("rustc ")? == expected_version
        && prefix.at < root && root < unhashed).then_some(())?;
    let mut r = Read::new(bytes.get(..unhashed)?, root);
    (r.byte()? == 0 && r.text()? == TARGET && r.symbol()? == "core" && r.take(2)? == [0, 0]).then_some(())?;
    r.take(8)?; // StableCrateId
    (r.byte()? == 0).then_some(())?; // No required panic strategy.
    r.take(6)?;
    // Skip external items, dependencies, dylib formats, library features,
    // and stability implications in the pinned CrateRoot field order.
    for _ in 0..5 {
        r.array()?;
    }
    let (at, count) = r.array()?;
    (count <= 256).then_some(())?;
    let mut result = [None; 2];
    let mut lang_items = Read::new(r.bytes, at);
    for _ in 0..count {
        let def = lang_items.leb()?;
        let tag = usize::from(lang_items.byte()?);
        if let Some(slot) = result.get_mut(tag) {
            (def != 0 && slot.replace(def).is_none()).then_some(())?;
        }
    }
    Some([result[0]?, result[1]?]) // Pinned LangItem::{Sized, MetaSized}.
}

impl<'a> IndexedCrate<'a> {
    fn record(&self, name: &str, index: usize) -> Option<Read<'a>> {
        let position = self.position(name, index)?;
        (0 < position && position < self.data.len()).then(|| Read::new(self.data, position))
    }

    fn type_name(&self, index: usize, parent: usize) -> Option<&'a str> {
        let mut key = self.record("def_keys", index)?;
        (key.byte()? == 1 && key.leb()? == parent && key.byte()? == 5).then_some(())?;
        let name = key.symbol()?;
        (valid_identifier(name) && key.leb()? == 0).then_some(name)
    }

    fn single_associated_item(&self, index: usize) -> Option<usize> {
        let (position, count) = self.array("associated_item_or_field_def_ids", index)?;
        (count == 1).then_some(())?;
        let child = Read::new(self.data, position).leb()?;
        (self.value::<1>("def_kind", child)? == [10]).then_some(child)
    }

    fn own_lifetime(&self, index: usize, parent: usize, parent_count: usize) -> Option<(usize, &'a str)> {
        let mut generics = self.record("generics_of", index)?;
        (generics.byte()? == 1 && generics.local_def()? == parent
            && generics.leb()? == parent_count && generics.leb()? == 1).then_some(())?;
        let name = generics.symbol()?;
        let parameter = generics.local_def()?;
        let slot = generics.leb()?;
        (slot == parent_count && generics.take(2)? == [0, 0]
            && generics.leb()? == 1 && generics.local_def()? == parameter
            && generics.leb()? == slot && generics.take(3)? == [u8::from(parent_count == 1), 0, 0]
            && self.value::<1>("def_kind", parameter)? == [23]).then_some(())?;
        Some((slot, name))
    }

    fn has_self_outlives(&self, index: usize, trait_index: usize, lifetime: (usize, &str)) -> Option<()> {
        let mut clauses = self.record("explicit_clauses_of", index)?;
        (clauses.byte()? == 1 && clauses.local_def()? == trait_index && clauses.leb()? == 1
            && clauses.leb()? == 0).then_some(())?;
        clauses.decode_abbreviated(|clause| {
            (clause.take(2)? == [0, 2]).then_some(())?; // Clause, TypeOutlives
            clause.self_type()?;
            clause.early_region(lifetime)
        })
    }

    fn shared_integer(&self, item: usize, lifetime: (usize, &str)) -> Option<crate::IntegerType> {
        self.record("type_of", item)?.decode_abbreviated(|ty| {
            // Ref, ReEarlyParam, actual associated assignment binder slot.
            (ty.byte()? == 12).then_some(())?;
            ty.early_region(lifetime)?;
            let referent = ty.integer_type()?;
            (ty.byte()? == 0).then_some(referent) // Mutability::Not
        })
    }

    fn size_bound(
        &self, clause: &mut Read<'_>, expected: usize, projection: Option<(usize, (usize, &str))>,
    ) -> Option<()> {
        (clause.leb()? == 0).then_some(())?; // No late-bound variables in this clause.
        clause.decode_abbreviated(|clause| {
            // A plain positive Trait clause from authenticated core, with
            // exactly one type argument for Self.
            (clause.take(3)? == [0, 0, 1] && clause.leb()? == expected && clause.take(2)? == [1, 1]).then_some(())?;
            if let Some((item, lifetime)) = projection {
                clause.decode_abbreviated(|ty| {
                    // Alias, IsRigid::No, Projection. The arguments are the
                    // trait's Self and the declaration's early lifetime.
                    (ty.take(3)? == [23, 1, 0] && ty.local_def()? == item && ty.take(2)? == [2, 1]).then_some(())?;
                    ty.self_type()?;
                    (ty.byte()? == 0).then_some(())?; // GenericArg::Lifetime
                    ty.early_region(lifetime)
                })?;
            } else {
                clause.self_type()?;
            }
            (clause.byte()? == 0).then_some(())?; // ClausePolarity::Positive
            Some(())
        })?;
        clause.skip_span()
    }

    fn unit_type(
        &self, r: &mut Read<'_>, receiver: usize, ctor: usize, name: &str,
    ) -> Option<()> {
        let repr = self.record("repr_options", receiver)?.take(13)?;
        (repr[..5] == [0; 5]).then_some(())?;
        r.decode_abbreviated(|ty| {
            (ty.byte()? == 5 && ty.local_def()? == receiver
                && ty.leb()? == 1 && ty.local_def()? == receiver
                && ty.take(2)? == [1, 1] && ty.local_def()? == ctor && ty.symbol()? == name
                && ty.take(7)? == [1, 0, 0, 0, 0, 12, 0] && ty.take(13)? == repr
                && ty.leb()? == 0).then_some(())
        })
    }
}

#[derive(Debug)]
pub struct ProjectionDefinitions<'a> {
    pub receiver: (&'a str, u64),
    pub family: (&'a str, u64),
    pub item: (&'a str, u64),
    pub referent: Option<crate::IntegerType>,
    pub code_free: bool,
}

// Read the declaration, implementation and its result from the pinned
// metadata schema. These records have not yet been installed in a session.
fn read_projection_definitions<'a>(krate: &mut IndexedCrate<'a>, sdk: &Sdk) -> Option<ProjectionDefinitions<'a>> {
    let [sized, meta_sized] = sdk.core_size_traits?;
    let (traits, trait_count) = krate.traits;
    let (children, child_count) = krate.array("module_children_non_reexports", 0)?;
    (trait_count == 1 && (3..=65_538).contains(&child_count) && krate.trait_impls.1 <= 1
        && only_implicit_prelude_reexport(krate.data, krate.value("module_children_reexports", 0)?, "core")?
        && krate.array("ambig_module_children", 0)?.1 == 0
        && krate.array("attributes", 0)?.1 == 0).then_some(())?;
    let trait_index = Read::new(krate.data, traits).leb()?;
    let mut children = Read::new(krate.data, children);
    let (mut receiver, mut ctor, mut family) = (None, None, false);
    let mut indices = BTreeSet::new();
    let mut functions = Vec::new();
    let mut names = BTreeSet::new();
    for _ in 0..child_count {
        let index = children.leb()?;
        (index != 0 && indices.insert(index)).then_some(())?;
        match krate.value::<1>("def_kind", index)? {
            [12] => {
                if let Some(function) = read_scalar_function(krate, index)? {
                    names.insert(function.name).then_some(())?;
                    functions.push(function);
                }
                continue;
            }
            [2] if receiver.is_none() => receiver = Some(index),
            [37] if ctor.is_none() => ctor = Some(index),
            [6] if index == trait_index && !family => family = true,
            _ => return None,
        }
        (krate.record("visibility", index)?.byte()? == 0
            && krate.array("attributes", index)?.1 == 0).then_some(())?;
    }
    let (receiver, ctor) = (receiver?, ctor?);
    family.then_some(())?;
    let receiver_name = krate.type_name(receiver, 0)?;
    let mut variant = krate.record("variant_data", receiver)?;
    (variant.take(5)? == [0, 1, 0, 1, 1] && variant.leb()? == ctor && variant.byte()? == 0
        && krate.array("associated_item_or_field_def_ids", receiver)?.1 == 0
        && krate.record("generics_of", receiver)?.take(7)? == [0; 7]).then_some(())?;
    krate.unit_type(&mut krate.record("type_of", receiver)?, receiver, ctor, receiver_name)?;
    let mut definition = krate.record("trait_def", trait_index)?;
    // No marker/auto traits, specialization, implementation restrictions,
    // extra required items, forced dyn behavior or const/unsafe trait flags.
    (definition.local_def()? == trait_index && definition.take(15)? == [0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        .then_some(())?;
    let mut clauses = krate.record("explicit_clauses_of", trait_index)?;
    (clauses.take(2)? == [0, 1]).then_some(())?;
    krate.size_bound(&mut clauses, meta_sized, None)?;
    for table in ["explicit_super_clauses_of", "explicit_implied_clauses_of"] {
        let (at, count) = krate.array(table, trait_index)?;
        (count == 1).then_some(())?;
        krate.size_bound(&mut Read::new(krate.data, at), meta_sized, None)?;
    }
    let item = krate.single_associated_item(trait_index)?;
    (krate.record("assoc_container", item)?.byte()? == 0
        && krate.value::<1>("defaultness", item)? == [1]
        && krate.position("type_of", item)? == 0).then_some(())?;
    let lifetime = krate.own_lifetime(item, trait_index, 1)?;
    krate.has_self_outlives(item, trait_index, lifetime)?;
    for table in ["explicit_item_bounds", "explicit_item_self_bounds"] {
        let (at, count) = krate.array(table, item)?;
        (count == 1).then_some(())?;
        krate.size_bound(&mut Read::new(krate.data, at), sized, Some((item, lifetime)))?;
    }

    let referent = if krate.trait_impls.1 == 0 {
        None
    } else {
        let mut impls = Read::new(krate.data, krate.trait_impls.0);
        (impls.local_def()? == trait_index).then_some(())?;
        let (position, count) = impls.array()?;
        (count == 1).then_some(())?;
        let implementation = Read::new(krate.data, position).leb()?;
        (krate.value::<1>("def_kind", implementation)? == [26]
            && krate.record("generics_of", implementation)?.take(7)? == [0; 7]
            && krate.record("explicit_clauses_of", implementation)?.take(2)? == [0; 2]).then_some(())?;
        let mut header = krate.record("impl_trait_header", implementation)?;
        (header.local_def()? == trait_index && header.take(2)? == [1, 1]).then_some(())?;
        krate.unit_type(&mut header, receiver, ctor, receiver_name)?;
        (header.take(3)? == [0, 1, 1]).then_some(())?; // Positive, safe, non-const impl.
        let assignment = krate.single_associated_item(implementation)?;
        let mut container = krate.record("assoc_container", assignment)?;
        (container.take(2)? == [2, 0] && container.local_def()? == item).then_some(())?;
        let lifetime = krate.own_lifetime(assignment, implementation, 0)?;
        let mut clauses = krate.record("explicit_clauses_of", assignment)?;
        (clauses.byte()? == 1 && clauses.local_def()? == implementation && clauses.leb()? == 0).then_some(())?;
        Some(krate.shared_integer(assignment, lifetime)?)
    };
    let identity = |index, parent| Some((
        krate.type_name(index, parent)?,
        u64::from_le_bytes(krate.value("def_path_hashes", index)?),
    ));
    let projection = ProjectionDefinitions {
        receiver: identity(receiver, 0)?,
        family: identity(trait_index, 0)?,
        item: identity(item, trait_index)?,
        referent,
        code_free: child_count == 3,
    };
    krate.identity.functions = functions;
    Some(projection)
}

fn read_index<'a>(bytes: &'a [u8], sdk: &Sdk, link: bool) -> Option<IndexedCrate<'a>> {
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
    let requires_abort = match r.byte()? {
        0 => false,
        1 if r.byte()? == 1 => true,
        _ => return None,
    };
    // Unwind-in-drop default, edition 2024, no allocator or panic handler.
    (r.take(6)? == [0, 3, 0, 0, 0, 0] && r.array()?.1 == 0).then_some(())?;
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
    let mut traits = (0, 0);
    let mut trait_impls = (0, 0);
    for field in &schema::ROOT_FIELDS[11..26] {
        let (at, len) = r.array()?;
        match *field {
            "lang_items_missing" if !sdk.standard => (len == 1 && data.get(at) == Some(&151)).then_some(())?,
            "traits" => traits = (at, len),
            "impls" => trait_impls = (at, len),
            _ => (len == 0).then_some(())?,
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
    let identity = ScalarCrate { name, stable_id, svh, filename_suffix, requires_abort, functions: Vec::new(), dependencies };
    Some(IndexedCrate { identity, data, tables, traits, trait_impls })
}

fn read_crate<'a>(bytes: &'a [u8], sdk: &Sdk, link: bool) -> Option<ScalarCrate<'a>> {
    read_scalar_index(read_index(bytes, sdk, link)?, sdk)
}

/// Read an actual Cargo extern for an executable. The caller verifies any
/// archive against the returned interface and its required object inventory.
pub fn read_executable_crate<'a>(
    bytes: &'a [u8], sdk: &Sdk,
) -> Option<(ScalarCrate<'a>, Option<ProjectionDefinitions<'a>>)> {
    let mut index = read_index(bytes, sdk, true)?;
    if index.traits.1 == 0 && index.trait_impls.1 == 0 {
        return Some((read_scalar_index(index, sdk)?, None));
    }
    (!sdk.standard && index.identity.dependencies.is_empty()).then_some(())?;
    let projection = read_projection_definitions(&mut index, sdk)?;
    Some((index.identity, Some(projection)))
}

fn read_scalar_index<'a>(mut krate: IndexedCrate<'a>, sdk: &Sdk) -> Option<ScalarCrate<'a>> {
    (krate.traits.1 == 0 && krate.trait_impls.1 == 0).then_some(())?;
    let data = krate.data;
    let (children_at, child_count) = krate.array("module_children_non_reexports", 0)?;
    (child_count > 0
        && child_count <= 65_535
        && only_implicit_prelude_reexport(data, krate.value("module_children_reexports", 0)?, if sdk.standard { "std" } else { "core" })?
        && krate.array("ambig_module_children", 0)?.1 == 0
        && krate.array("attributes", 0)?.1 == 0)
        .then_some(())?;
    let mut children = Read::new(data, children_at);
    let mut indices = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut functions = Vec::new();
    for _ in 0..child_count {
        let index = children.leb()?;
        (index != 0 && indices.insert(index) && krate.value::<1>("def_kind", index)? == [12]).then_some(())?;
        if let Some(function) = read_scalar_function(&krate, index)? {
            names.insert(function.name).then_some(())?;
            functions.push(function);
        }
    }
    krate.identity.functions = functions;
    Some(krate.identity)
}

fn read_scalar_function<'a>(krate: &IndexedCrate<'a>, index: usize) -> Option<Option<ScalarFunction<'a>>> {
    match krate.record("visibility", index)?.byte()? {
        0 => {}
        1 => return Some(None),
        _ => return None,
    }
    let mut key = krate.record("def_keys", index)?;
    (key.take(3)? == [1, 0, 6]).then_some(())?;
    let function_name = key.symbol()?;
    (valid_identifier(function_name) && key.leb()? == 0).then_some(())?;

    (krate.array("attributes", index)?.1 == 0
        && krate.value::<1>("constness", index)? == [1]
        && krate.record("generics_of", index)?.take(7)? == [0; 7]
        && krate.record("explicit_clauses_of", index)?.take(2)? == [0; 2]).then_some(())?;
    let mut codegen = krate.record("codegen_fn_attrs", index)?;
    (codegen.take(13)? == [0; 13] && codegen.byte()? == 2 && codegen.take(6)? == [0; 6]).then_some(())?;
    let mut signature = krate.record("fn_sig", index)?;
    (signature.leb()? == 0).then_some(())?;
    let count = signature.leb()?;
    (0 < count && count <= 65).then_some(())?;
    let parameters = (1..count).map(|_| signature.integer_type()).collect::<Option<Vec<_>>>()?;
    let result = signature.integer_type()?;
    (signature.take(2)? == [66, 255]).then_some(())?;
    let path_hash = u64::from_le_bytes(krate.value("def_path_hashes", index)?);
    (path_hash == crate::identity::function_path_hash(krate.identity.stable_id, function_name)).then_some(())?;
    Some(Some(ScalarFunction { name: function_name, parameters, result, path_hash }))
}

fn only_implicit_prelude_reexport(data: &[u8], value: [u8; 16], dependency: &str) -> Option<bool> {
    let (offset, count) = read_interleaved(value)?;
    match count {
        0 => Some(true),
        1 => {
            let mut child = Read::new(data, offset);
            (child.symbol()? == dependency).then_some(())?;
            // Discard the observed indirect span without using it as semantic authority.
            let tag = child.byte()?;
            let width = usize::from(tag >> 3);
            (tag & 3 == 3 && (1..=8).contains(&width)).then_some(())?;
            child.take(width)?;
            // Res::Def(Mod, dependency:root), restricted to current root, one ExternCrate def.
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
