// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Checked type-family records in the pinned rustc_metadata v12 schema.
use super::{Array, Context};
use crate::identity::{lifetime_path_hash, sole_impl_hash, type_path_hash, unit_constructor_hash};
use crate::upstream::{codec::Encoder, odht_writer::HashTableOwned};
use crate::{NamedDefinition, ProjectionFamily, Sdk, Span};
use alloc::{string::String, vec::Vec};

const RECEIVER: u32 = 1;
const CONSTRUCTOR: u32 = 2;
const FAMILY: u32 = 3;
const ITEM: u32 = 4;
const DECLARATION_LIFETIME: u32 = 5;
const IMPLEMENTATION: u32 = 6;
const ASSIGNMENT: u32 = 7;
const ASSIGNMENT_LIFETIME: u32 = 8;
pub(super) const FIRST_FUNCTION: u32 = ASSIGNMENT_LIFETIME + 1;

struct Definition<'a> {
    index: u32,
    parent: u32,
    key: u8,
    kind: u8,
    hash: u64,
    name: Option<&'a str>,
    span: Span,
    ident_span: Option<Span>,
}

impl Definition<'_> {
    fn write(&self, cx: &mut Context) {
        let index = self.index as usize;
        cx.table("def_path_hashes").set(index, &self.hash.to_le_bytes());
        cx.table("def_kind").set(index, &[self.kind]);
        cx.fact("def_keys", index, |cx| {
            cx.opaque.emit_u8(1);
            cx.local_def(self.parent);
            cx.opaque.emit_u8(self.key);
            if let Some(name) = self.name {
                cx.symbol(name);
            }
            cx.opaque.emit_u32(0);
        });
        cx.fact("def_span", index, |cx| cx.span(self.span));
        if let Some(span) = self.ident_span {
            cx.fact("def_ident_span", index, |cx| cx.span(span));
        }
        cx.array_fact::<()>("attributes", index, &[], |_, _| unreachable!());
    }
}

fn named(index: u32, parent: u32, kind: u8, hash: u64, data: &NamedDefinition) -> Definition<'_> {
    Definition {
        index, parent, kind, hash, key: 5, name: Some(&data.name),
        span: data.span, ident_span: Some(data.name_span),
    }
}

fn lifetime(index: u32, parent: u32, name: &str, span: Span, hash: u64) -> Definition<'_> {
    Definition { index, parent, key: 8, kind: 23, hash, name: Some(name), span, ident_span: Some(span) }
}

fn public(cx: &mut Context, index: u32) {
    cx.fact("visibility", index as usize, |cx| cx.opaque.emit_u8(0));
}

fn empty_generics(cx: &mut Context) {
    cx.opaque.emit_raw_bytes(&[0; 7]);
}

fn early_region(cx: &mut Context, slot: u32, name: &str) {
    cx.opaque.emit_u8(0);
    cx.opaque.emit_u32(slot);
    cx.symbol(name);
}

fn self_type(cx: &mut Context) {
    cx.opaque.emit_u8(24);
    cx.opaque.emit_u32(0);
    cx.symbol("Self");
}

fn lifetime_generics(cx: &mut Context, parent: u32, parameter: u32, slot: u32, name: &str) {
    cx.opaque.emit_u8(1);
    cx.def(parent);
    cx.opaque.emit_u32(slot);
    cx.opaque.emit_usize(1);
    cx.symbol(name);
    cx.def(parameter);
    cx.opaque.emit_u32(slot);
    cx.opaque.emit_raw_bytes(&[0, 0]); // Pure-wrt-drop is false, kind Lifetime.
    cx.opaque.emit_usize(1);
    cx.def(parameter);
    cx.opaque.emit_u32(slot);
    cx.opaque.emit_raw_bytes(&[u8::from(slot == 1), 0, 0]);
}

fn size_bound(cx: &mut Context, target: usize, family: &ProjectionFamily, alias: bool) {
    cx.opaque.emit_raw_bytes(&[0, 0, 0, 1]); // Binder, Predicate::Clause, Trait, core CrateNum.
    cx.opaque.emit_usize(target);
    cx.opaque.emit_raw_bytes(&[1, 1]); // One argument, GenericArg::Type.
    if alias {
        cx.opaque.emit_raw_bytes(&[23, 1, 0]); // TyKind::Alias, IsRigid::No, Projection.
        cx.def(ITEM);
        cx.opaque.emit_raw_bytes(&[2, 1]);
        self_type(cx);
        cx.opaque.emit_u8(0); // GenericArg::Lifetime
        early_region(cx, 1, &family.lifetime);
    } else {
        self_type(cx);
    }
    cx.opaque.emit_u8(0); // ClausePolarity::Positive
    cx.span(if alias { family.item.name_span } else { family.family.name_span });
}

fn unit_repr(cx: &mut Context, seed: u64) {
    cx.opaque.emit_raw_bytes(&[0; 5]);
    cx.opaque.emit_raw_bytes(&seed.to_le_bytes());
}

fn unit_type(cx: &mut Context, name: &str, seed: u64) {
    cx.opaque.emit_u8(5); // TyKind::Adt and its AdtDefData.
    cx.def(RECEIVER);
    cx.opaque.emit_usize(1);
    cx.def(RECEIVER);
    cx.opaque.emit_raw_bytes(&[1, 1]);
    cx.def(CONSTRUCTOR);
    cx.symbol(name);
    cx.opaque.emit_raw_bytes(&[1, 0, 0, 0, 0, 12, 0]); // Unit variant/discriminant and AdtFlags.
    unit_repr(cx, seed);
    cx.opaque.emit_usize(0);
}

pub(super) fn encode_family(
    cx: &mut Context, sdk: &Sdk, root: u64, path_map: &mut HashTableOwned,
    children: &mut Vec<u32>, family: &ProjectionFamily,
) -> Result<(Array, Array), String> {
    let [sized, meta_sized] = sdk.core_size_traits.ok_or("authenticated core size trait identities are missing")?;
    let receiver = type_path_hash(root, &family.receiver.name);
    let trait_hash = type_path_hash(root, &family.family.name);
    let item_hash = type_path_hash(trait_hash, &family.item.name);
    let mut definitions = Vec::from([
        named(RECEIVER, 0, 2, receiver, &family.receiver),
        Definition { index: CONSTRUCTOR, parent: RECEIVER, key: 11, kind: 37,
            hash: unit_constructor_hash(receiver), name: None, span: family.receiver.span,
            ident_span: Some(family.receiver.name_span) },
        named(FAMILY, 0, 6, trait_hash, &family.family),
        named(ITEM, FAMILY, 10, item_hash, &family.item),
        lifetime(DECLARATION_LIFETIME, ITEM, &family.lifetime, family.lifetime_span, lifetime_path_hash(item_hash, &family.lifetime)),
    ]);
    if let Some(implementation) = &family.implementation {
        let impl_hash = sole_impl_hash(root);
        let assignment = type_path_hash(impl_hash, &implementation.item.name);
        definitions.extend([
            Definition { index: IMPLEMENTATION, parent: 0, key: 1, kind: 26,
                hash: impl_hash, name: None, span: implementation.span, ident_span: None },
            named(ASSIGNMENT, IMPLEMENTATION, 10, assignment, &implementation.item),
            lifetime(ASSIGNMENT_LIFETIME, ASSIGNMENT, &implementation.lifetime, implementation.lifetime_span,
                lifetime_path_hash(assignment, &implementation.lifetime)),
        ]);
    }
    for definition in &definitions {
        if path_map.insert(&definition.hash, &definition.index).is_some() {
            return Err("metadata projection definition path hash collision".into());
        }
        definition.write(cx);
    }
    for index in [RECEIVER, FAMILY].into_iter().chain(family.implementation.iter().map(|_| IMPLEMENTATION)) {
        cx.fact("expn_that_defined", index as usize, |cx| {
            cx.opaque.emit_u32(0);
            cx.opaque.emit_u32(0);
        });
    }
    children.extend([RECEIVER, CONSTRUCTOR, FAMILY]);
    for index in [RECEIVER, CONSTRUCTOR, FAMILY, ITEM] { public(cx, index); }
    // Fingerprint::to_smaller_hash used by TyCtxt::repr_options_of, with
    // the default Cargo request's absent layout randomization override.
    let seed = root.wrapping_mul(3).wrapping_add(receiver);
    for index in [RECEIVER, CONSTRUCTOR] {
        cx.fact("generics_of", index as usize, empty_generics);
        cx.fact("explicit_clauses_of", index as usize, |cx| cx.opaque.emit_raw_bytes(&[0, 0]));
        cx.fact("type_of", index as usize, |cx| unit_type(cx, &family.receiver.name, seed));
        cx.array_fact::<()>("variances_of", index as usize, &[], |_, _| unreachable!());
    }
    cx.fact("repr_options", RECEIVER as usize, |cx| unit_repr(cx, seed));
    cx.fact("params_in_repr", RECEIVER as usize, |cx| cx.opaque.emit_raw_bytes(&[0, 0]));
    cx.fact("variant_data", RECEIVER as usize, |cx| {
        cx.opaque.emit_raw_bytes(&[0, 1, 0, 1, 1]);
        cx.local_def(CONSTRUCTOR);
        cx.opaque.emit_u8(0);
    });
    cx.array_fact::<()>("associated_item_or_field_def_ids", RECEIVER as usize, &[], |_, _| unreachable!());
    cx.array_fact("associated_item_or_field_def_ids", FAMILY as usize, &[ITEM], |cx, id| cx.local_def(*id));
    cx.array_fact("module_children_non_reexports", FAMILY as usize, &[ITEM], |cx, id| cx.local_def(*id));
    cx.fact("generics_of", FAMILY as usize, |cx| {
        cx.opaque.emit_raw_bytes(&[0, 0, 1]);
        cx.symbol("Self");
        cx.def(FAMILY);
        cx.opaque.emit_raw_bytes(&[0, 0, 1, 0, 0, 1]);
        cx.def(FAMILY);
        cx.opaque.emit_raw_bytes(&[0, 1, 0, 0]);
    });
    cx.fact("trait_def", FAMILY as usize, |cx| {
        cx.def(FAMILY);
        cx.opaque.emit_raw_bytes(&[0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    });
    // The trait's Self type has no own lifetime bound. The associated
    // item's separate Self outlives clause does not define this default.
    cx.fact("object_lifetime_default", FAMILY as usize, |cx| cx.opaque.emit_u8(0));
    cx.fact("explicit_clauses_of", FAMILY as usize, |cx| {
        cx.opaque.emit_raw_bytes(&[0, 1]);
        size_bound(cx, meta_sized, family, false);
    });
    for table in ["explicit_super_clauses_of", "explicit_implied_clauses_of"] {
        cx.array_fact(table, FAMILY as usize, &[meta_sized], |cx, &bound| size_bound(cx, bound, family, false));
    }

    cx.fact("assoc_container", ITEM as usize, |cx| cx.opaque.emit_u8(0));
    cx.table("defaultness").set(ITEM as usize, &[1]);
    cx.fact("generics_of", ITEM as usize, |cx| lifetime_generics(cx, FAMILY, DECLARATION_LIFETIME, 1, &family.lifetime));
    cx.fact("explicit_clauses_of", ITEM as usize, |cx| {
        cx.opaque.emit_u8(1);
        cx.def(FAMILY);
        cx.opaque.emit_raw_bytes(&[1, 0, 0, 2]);
        self_type(cx);
        early_region(cx, 1, &family.lifetime);
        cx.span(family.item.span);
    });
    for table in ["explicit_item_bounds", "explicit_item_self_bounds"] {
        cx.array_fact(table, ITEM as usize, &[sized], |cx, &bound| size_bound(cx, bound, family, true));
    }

    let traits = cx.array(&[FAMILY], |cx, id| cx.local_def(*id));
    let Some(implementation) = &family.implementation else { return Ok((traits, Array::EMPTY)); };
    public(cx, ASSIGNMENT);
    cx.fact("generics_of", IMPLEMENTATION as usize, empty_generics);
    cx.fact("explicit_clauses_of", IMPLEMENTATION as usize, |cx| cx.opaque.emit_raw_bytes(&[0, 0]));
    cx.fact("type_of", IMPLEMENTATION as usize, |cx| unit_type(cx, &family.receiver.name, seed));
    cx.fact("impl_trait_header", IMPLEMENTATION as usize, |cx| {
        cx.def(FAMILY);
        cx.opaque.emit_raw_bytes(&[1, 1]);
        unit_type(cx, &family.receiver.name, seed);
        cx.opaque.emit_raw_bytes(&[0, 1, 1]);
    });
    cx.array_fact("associated_item_or_field_def_ids", IMPLEMENTATION as usize, &[ASSIGNMENT], |cx, id| cx.local_def(*id));
    cx.fact("assoc_container", ASSIGNMENT as usize, |cx| {
        cx.opaque.emit_raw_bytes(&[2, 0]); // TraitImpl(Ok(parent trait item))
        cx.def(ITEM);
    });
    cx.fact("generics_of", ASSIGNMENT as usize, |cx| {
        lifetime_generics(cx, IMPLEMENTATION, ASSIGNMENT_LIFETIME, 0, &implementation.lifetime);
    });
    cx.fact("explicit_clauses_of", ASSIGNMENT as usize, |cx| {
        cx.opaque.emit_u8(1);
        cx.def(IMPLEMENTATION);
        cx.opaque.emit_usize(0);
    });
    cx.fact("type_of", ASSIGNMENT as usize, |cx| {
        cx.opaque.emit_u8(12);
        early_region(cx, 0, &implementation.lifetime);
        cx.opaque.emit_raw_bytes(&implementation.referent.metadata_tags());
        cx.opaque.emit_u8(0);
    });
    let candidates = cx.array(&[IMPLEMENTATION], |cx, id| {
        cx.local_def(*id);
        cx.opaque.emit_raw_bytes(&[1, 5]); // Some(SimplifiedType::Adt)
        cx.def(RECEIVER);
    });
    let impls = cx.array(&[FAMILY], |cx, id| {
        cx.def(*id);
        cx.array_ref(candidates);
    });
    Ok((traits, impls))
}
