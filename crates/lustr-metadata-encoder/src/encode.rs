// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Original metadata v12 serialization algorithms, adapted to completed native
// facts. The exact upstream functions and every semantic cut are recorded in
// source-slices.json. There is no TyCtxt, HIR synthesis, or analysis callback.
use alloc::{borrow::{Cow, ToOwned}, collections::{BTreeMap, BTreeSet}, string::String, vec::Vec};
use core::hash::{Hash, Hasher};

use md5::{Digest, Md5};
use twox_hash::XxHash3_128;

use crate::identity::{function_path_hash, function_symbol, stable_crate_id};
use crate::table::{Table, TableBuilder};
use crate::upstream::codec::Encoder;
use crate::upstream::mem_encoder::MemEncoder;
use crate::upstream::schema::{DOC_COMMENT, METADATA_HEADER, ROOT_FIELDS, TABLE_FIELDS};
use crate::{Encoded, Input, Sdk, Span, SymbolMapping};
#[path = "encode_projection.rs"]
mod projection;

// Ordinal from the pinned rustc_attr_ir language_item_table.
const EH_PERSONALITY_LANG_ITEM: u8 = 151;

#[derive(Clone, Copy)]
struct Array { position: usize, len: usize }
impl Array {
    const EMPTY: Self = Self { position: 1, len: 0 };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LazyState { NoNode, NodeStart(usize), Previous(usize) }

struct Context {
    opaque: MemEncoder,
    lazy_state: LazyState,
    symbols: BTreeMap<String, usize>,
    spans: BTreeMap<Span, usize>,
    tables: Vec<TableBuilder>,
    normalized_positions: Vec<(u32, u32)>,
}

impl Context {
    fn empty() -> Self {
        Self { opaque: MemEncoder::new(), lazy_state: LazyState::NoNode,
            symbols: BTreeMap::new(), spans: BTreeMap::new(), tables: Vec::new(), normalized_positions: Vec::new() }
    }
    fn new(normalized_positions: Vec<(u32, u32)>) -> Self {
        Self { tables: TABLE_FIELDS.iter().map(|_| TableBuilder::default()).collect(),
            normalized_positions, ..Self::empty() }
    }
    fn metadata_header(&mut self, sdk: &Sdk) {
        self.opaque.emit_raw_bytes(METADATA_HEADER);
        self.opaque.emit_raw_bytes(&[0; 32]); // root, unhashed root, SVH; filled after encoding.
        self.opaque.emit_str(&format!("rustc {}", sdk.cfg_version));
    }
    fn crate_header(&mut self, name: &str, is_stub: bool) {
        self.opaque.emit_u8(0); self.opaque.emit_str(crate::TARGET); // TargetTuple
        self.symbol(name); self.opaque.emit_bool(false); self.opaque.emit_bool(is_stub);
    }
    fn table(&mut self, field: &str) -> &mut TableBuilder {
        let index = TABLE_FIELDS.iter().position(|&name| name == field).expect("original table field");
        &mut self.tables[index]
    }
    fn node(&mut self, encode: impl FnOnce(&mut Self)) -> usize {
        let pos = self.opaque.position();
        assert_ne!(pos, 0);
        assert_eq!(self.lazy_state, LazyState::NoNode);
        self.lazy_state = LazyState::NodeStart(pos);
        encode(self);
        self.lazy_state = LazyState::NoNode;
        assert!(pos <= self.opaque.position());
        pos
    }
    fn array<T>(&mut self, values: &[T], mut encode: impl FnMut(&mut Self, &T)) -> Array {
        let position = self.node(|cx| for value in values { encode(cx, value); });
        Array { position, len: values.len() }
    }
    // rustc_metadata::EncodeContext::emit_lazy_distance, with positions already
    // checked nonzero at node construction. The original ordering is enforced.
    fn distance(&mut self, pos: usize) {
        let distance = match self.lazy_state {
            LazyState::NoNode => panic!("metadata lazy reference outside a node"),
            LazyState::NodeStart(start) => { assert!(pos <= start); start - pos }
            LazyState::Previous(last) => { assert!(last <= pos); pos - last }
        };
        self.lazy_state = LazyState::Previous(pos);
        self.opaque.emit_usize(distance);
    }
    fn array_ref(&mut self, value: Array) {
        self.opaque.emit_usize(value.len);
        if value.len != 0 { self.distance(value.position); }
    }
    fn table_ref(&mut self, value: Table) {
        self.opaque.emit_usize(value.width_and_sparse);
        self.opaque.emit_usize(value.len);
        self.distance(value.position);
    }
    fn symbol(&mut self, value: &str) {
        // The native symbol adapter has no predefined Rust interner IDs. It
        // uses the original string/offset arms, preserving symbol contents.
        if let Some(&position) = self.symbols.get(value) {
            self.opaque.emit_u8(1); // SYMBOL_OFFSET
            self.opaque.emit_usize(position);
        } else {
            self.opaque.emit_u8(0); // SYMBOL_STR
            let position = self.opaque.position();
            self.symbols.insert(value.to_owned(), position);
            self.opaque.emit_str(value);
        }
    }
    fn span(&mut self, span: Span) {
        // SourceFile::normalized_byte_pos maps the native offsets in the
        // original file to the coordinates serialized in Rust metadata.
        let offset = |original| {
            let count = self.normalized_positions.partition_point(|&(pos, diff)| pos + diff <= original);
            original - count.checked_sub(1).map_or(0, |index| self.normalized_positions[index].1)
        };
        let span = Span { lo: offset(span.lo), hi: offset(span.hi) };
        if let Some(&last) = self.spans.get(&span) {
            let offset = self.opaque.position() - last;
            let relative = offset < last;
            let value = if relative { offset } else { last };
            let bytes = (usize::BITS - value.leading_zeros()).div_ceil(8) as usize;
            self.opaque.emit_u8(3 | (if relative { 4 } else { 0 }) | ((bytes as u8) << 3));
            self.opaque.write_with(|out| { *out = value.to_le_bytes(); bytes });
        } else {
            self.spans.insert(span, self.opaque.position());
            if span.lo == 0 && span.hi == 0 {
                self.opaque.emit_u8(2 | 4); // Partial, root context, length zero.
                return;
            }
            let len = span.hi - span.lo;
            self.opaque.emit_u8(4 | ((len.min(31) as u8) << 3)); // Local/root.
            self.opaque.emit_u32(span.lo);
            if len >= 31 { self.opaque.emit_u32(len); }
            self.opaque.emit_u32(0); // Exact single source file.
        }
    }
    fn local_def(&mut self, index: u32) { self.opaque.emit_u32(index); }
    fn def(&mut self, index: u32) { self.opaque.emit_u32(0); self.local_def(index); }
    fn fact(&mut self, field: &str, index: usize, encode: impl FnOnce(&mut Self)) {
        let position = self.node(encode);
        self.table(field).set_position(index, position);
    }
    fn array_fact<T>(&mut self, field: &str, index: usize, values: &[T], encode: impl FnMut(&mut Self, &T)) {
        let array = self.array(values, encode);
        self.table(field).set_array(index, array.position, array.len);
    }
}

pub(crate) fn encode(
    input: &Input<'_>, sdk: &Sdk, link: bool, family: Option<&crate::ProjectionFamily>,
) -> Result<Encoded, String> {
    let stable_id = stable_crate_id(input.crate_name, input.metadata, &sdk.cfg_version);
    let mut path_map = crate::upstream::odht_writer::HashTableOwned::default();
    path_map.insert(&stable_id, &0);
    let emitted: BTreeMap<_, _> = input.emitted.iter().map(|i| (i.definition, &i.symbol)).collect();
    let mut definitions = Vec::with_capacity(input.functions.len());
    let mut mappings = Vec::with_capacity(input.emitted.len());
    let first_function = if family.is_some() { projection::FIRST_FUNCTION } else { 1 };
    for (n, function) in input.functions.iter().enumerate() {
        let index = first_function + n as u32;
        let hash = function_path_hash(stable_id, &function.name);
        if path_map.insert(&hash, &index).is_some() {
            return Err("metadata DefPathHash collision; the original definition allocation cannot continue".into());
        }
        definitions.push((index, hash));
        if link {
            mappings.push(SymbolMapping { definition: function.definition, rust_def_index: index,
                rust_def_path_hash: hash, native_symbol: emitted[&function.definition].clone(),
                rust_symbol: function_symbol(&function.name, input.crate_name, stable_id), public: function.public });
        }
    }

    let (normalized_source, normalized_positions) = normalize_crlf(input.source);
    let mut cx = Context::new(normalized_positions);
    cx.metadata_header(sdk);

    let dependencies = cx.array(&sdk.dependencies, |cx, dep| {
        cx.symbol(&dep.name);
        cx.opaque.emit_raw_bytes(&dep.hash);
        cx.opaque.emit_bool(dep.host_hash.is_some());
        if let Some(hash) = dep.host_hash { cx.opaque.emit_raw_bytes(&hash); }
        cx.opaque.emit_u8(dep.kind);
        cx.opaque.emit_bool(dep.is_private);
    });
    // The pinned weak-lang-item check records EhPersonality when neither
    // the completed crate nor its dependencies define it. The admitted leaf
    // has no lang attributes and its pinned core/builtins pair has no provider.
    // This metadata fact is independent of whether the request emits code.
    let missing_lang_items = if sdk.standard { Array::EMPTY } else {
        cx.array(&[EH_PERSONALITY_LANG_ITEM], |cx, &item| cx.opaque.emit_u8(item))
    };

    // Deterministic original definitions: crate root plus every completed local
    // declaration, including private functions. The same mapping drives the
    // object symbol rewrite and the export query specialization below.
    cx.table("def_path_hashes").set(0, &stable_id.to_le_bytes());
    cx.fact("def_keys", 0, |cx| {
        cx.opaque.emit_u8(0); // parent: None
        cx.opaque.emit_u8(0); // DefPathData::CrateRoot
        cx.opaque.emit_u32(0);
    });
    for (function, &(index, hash)) in input.functions.iter().zip(&definitions) {
        let index = index as usize;
        cx.table("def_path_hashes").set(index, &hash.to_le_bytes());
        cx.fact("def_keys", index, |cx| {
            cx.opaque.emit_u8(1); cx.local_def(0);
            cx.opaque.emit_u8(6); // Original DefPathData::ValueNs
            cx.symbol(&function.name);
            cx.opaque.emit_u32(0);
        });
    }
    let mut children: Vec<u32> = definitions.iter().map(|&(index, _)| index).collect();
    let (traits, impls) = if let Some(family) = family {
        projection::encode_family(&mut cx, sdk, stable_id, &mut path_map, &mut children, family)?
    } else {
        (Array::EMPTY, Array::EMPTY)
    };
    cx.array_fact("module_children_non_reexports", 0, &children, |cx, &id| cx.local_def(id));
    cx.table("def_kind").set(0, &[1]); // FixedSizeEncoding: Mod + 1.
    cx.fact("def_span", 0, |cx| cx.span(input.root_span));
    cx.array_fact::<()>("attributes", 0, &[], |_, _| unreachable!());
    cx.fact("visibility", 0, |cx| cx.opaque.emit_u8(0));
    cx.fact("expn_that_defined", 0, |cx| { cx.opaque.emit_u32(0); cx.opaque.emit_u32(0); });

    for (function, &(def_index, _)) in input.functions.iter().zip(&definitions) {
        let index = def_index as usize;
        cx.table("def_kind").set(index, &[12]); // FixedSizeEncoding: Fn + 1.
        cx.fact("def_span", index, |cx| cx.span(function.span));
        cx.fact("def_ident_span", index, |cx| cx.span(function.name_span));
        // Rust retains source doc comments for downstream exported definitions.
        let docs = if function.public { function.docs.as_slice() } else { &[] };
        cx.array_fact("attributes", index, docs, |cx, doc| {
            cx.opaque.emit_raw_bytes(&[0, DOC_COMMENT, 0, 0, u8::from(doc.block)]);
            cx.span(doc.span);
            cx.symbol(&doc.text);
        });
        cx.fact("visibility", index, |cx| {
            cx.opaque.emit_u8(if function.public { 0 } else { 1 });
            if !function.public { cx.local_def(0); }
        });
        cx.fact("codegen_fn_attrs", index, |cx| codegen_attrs(cx, function.inline_never));
        cx.array_fact::<()>("variances_of", index, &[], |_, _| unreachable!());
        cx.fact("fn_sig", index, |cx| {
            cx.opaque.emit_usize(0); // Binder bound vars, original order.
            cx.opaque.emit_usize(function.parameters.len() + 1);
            for ty in function.parameters.iter().map(|param| param.ty).chain([function.result]) {
                cx.opaque.emit_raw_bytes(&ty.metadata_tags());
            }
            cx.opaque.emit_u8(2 | (1 << 6)); // Rust packed ABI + safe.
            cx.opaque.emit_u8(u8::MAX); // No splatted argument.
        });
        cx.fact("generics_of", index, |cx| {
            cx.opaque.emit_u8(0); // parent None
            cx.opaque.emit_usize(0); // parent_count
            cx.opaque.emit_usize(0); // own_params
            cx.opaque.emit_usize(0); // param_def_id_to_index
            cx.opaque.emit_bool(false); // has_self
            cx.opaque.emit_bool(false); // has_arg_pos_consts
            cx.opaque.emit_u8(0); // has_late_bound_regions None
        });
        cx.fact("explicit_clauses_of", index, |cx| {
            cx.opaque.emit_u8(0); // parent None
            cx.opaque.emit_usize(0); // actually no clauses
        });
        cx.fact("type_of", index, |cx| {
            cx.opaque.emit_u8(13); // Original TyKind::FnDef
            cx.def(def_index);
            cx.opaque.emit_usize(0); // Binder bound vars
            cx.opaque.emit_usize(0); // GenericArgs
        });
        cx.table("constness").set(index, &[1]); // Original table NotConst.
        cx.array_fact("fn_arg_idents", index, &function.parameters, |cx, param| {
            cx.opaque.emit_u8(1); // Some(Ident)
            cx.symbol(&param.name); cx.span(param.span);
        });
    }

    let tables: Vec<Table> = cx.tables.iter().map(|t| t.encode(&mut cx.opaque)).collect();

    // The pinned Rust export providers stop before querying emission for
    // metadata-only checks. Only a link has mappings from actual instances.
    let mut non_generic: Vec<_> = mappings.iter().filter(|m| m.public).collect();
    non_generic.sort_by_key(|m| m.rust_def_path_hash);
    let exports = cx.array(&non_generic, |cx, mapping| {
        cx.opaque.emit_u8(0); // ExportedSymbol::NonGeneric
        cx.def(mapping.rust_def_index);
        cx.opaque.emit_u8(1); // SymbolExportLevel::Rust
        cx.opaque.emit_u8(0); // SymbolExportKind::Text
        cx.opaque.emit_bool(false); cx.opaque.emit_bool(false);
    });
    // The original generic-export operation must see the real nonempty graph.
    // Each joined emitted instance is an Item with no generic arguments; none
    // qualifies for its `has_generics` branch. Missing instances were rejected
    // before this operation, rather than replaced by an empty mono graph.
    let completed_by_id: BTreeMap<_, _> = input.functions.iter().map(|f| (f.definition, f)).collect();
    let mut visited_instances = BTreeSet::new();
    for instance in input.emitted {
        let function = completed_by_id.get(&instance.definition).expect("validated instance");
        assert!(visited_instances.insert(function.definition));
        // Closed Function records have ordinary item kind and no generic
        // parameters/arguments, which is precisely this upstream exclusion.
    }
    assert_eq!(visited_instances.len(), mappings.len());

    // Root expansion data is real context, not an omitted hygiene obligation.
    // No expanded identifiers exist in the completed scope. The crate module
    // still records ExpnId::root, so serialize its original root ExpnData/hash.
    let root_expansion = cx.node(|cx| {
        cx.opaque.emit_u8(0); // ExpnKind::Root
        cx.opaque.emit_u32(0); cx.opaque.emit_u32(0); // parent root
        cx.span(Span { lo: 0, hi: 0 });
        cx.opaque.emit_u32(0); // disambiguator
        cx.span(Span { lo: 0, hi: 0 });
        cx.opaque.emit_u8(0); // allow_internal_unstable
        cx.opaque.emit_u8(3); // Edition2024
        cx.opaque.emit_u8(1); cx.def(0); // macro_def_id = crate root
        cx.opaque.emit_u8(0); // parent_module
        for _ in 0..4 { cx.opaque.emit_bool(false); }
    });
    let root_expansion_hash = cx.node(|cx| cx.opaque.emit_raw_bytes(&[0; 16]));
    let syntax_contexts = TableBuilder::default().encode(&mut cx.opaque);
    let mut expansion_data = TableBuilder::default(); expansion_data.set_position(0, root_expansion);
    let expansion_data = expansion_data.encode(&mut cx.opaque);
    let mut expansion_hashes = TableBuilder::default(); expansion_hashes.set_position(0, root_expansion_hash);
    let expansion_hashes = expansion_hashes.encode(&mut cx.opaque);

    let path_hash_map = cx.node(|cx| {
        let bytes = path_map.raw_bytes();
        cx.opaque.emit_usize(bytes.len()); cx.opaque.emit_raw_bytes(bytes);
        cx.opaque.emit_usize(0); // No nondeterministically allocated definitions.
    });
    let source_file = cx.node(|cx| encode_source_file(cx, input, &normalized_source, stable_id));
    let mut source_map = TableBuilder::default(); source_map.set_position(0, source_file);
    let source_map = source_map.encode(&mut cx.opaque);
    // Original add_target_modifiers always records TargetFeatures, including
    // an empty value. The admitted x86_64 Linux hard-float context has no CPU
    // or feature override; soft-float is its only modifier feature and is
    // incompatible with this ABI. An absent modifier record is not equivalent
    // to this actual empty modifier value, and breaks original crate checks.
    let target_modifiers = cx.array(&[()], |cx, _| {
        cx.opaque.emit_u8(2); // OptionsTargetModifiers::TargetFeatures
        cx.opaque.emit_str("");
    });
    // Original gather_enabled_denied_partial_mitigations includes disabled
    // mitigations too, sorted by the original kind. In this closed request,
    // StackProtector is None and control-flow guard is not enabled. Omitting
    // StackProtector is different: creader substitutes Enabled(false), whose
    // enum order is lower than StackProtector(None), and warns on a real caller.
    let denied_partial_mitigations = cx.array(&[0u8, 1u8], |cx, &kind| {
        cx.opaque.emit_u8(kind); // StackProtector, then ControlFlowGuard.
        if kind == 0 {
            cx.opaque.emit_u8(1); // DeniedPartialMitigationLevel::StackProtector
            cx.opaque.emit_u8(0); // StackProtector::None
        } else {
            cx.opaque.emit_u8(0); // DeniedPartialMitigationLevel::Enabled
            cx.opaque.emit_bool(false);
        }
    });

    let root = cx.node(|cx| {
        let mut field_index = 0;
        let mut field = |name: &str, f: &mut dyn FnMut(&mut Context)| {
            assert_eq!(ROOT_FIELDS[field_index], name); field_index += 1; f(cx);
        };
        field("header", &mut |cx| cx.crate_header(input.crate_name, false));
        field("stable_crate_id", &mut |cx| cx.opaque.emit_raw_bytes(&stable_id.to_le_bytes()));
        field("required_panic_strategy", &mut |cx| {
            cx.opaque.emit_bool(input.profile.panic_abort);
            if input.profile.panic_abort {
                cx.opaque.emit_u8(1); // Original PanicStrategy::Abort.
            }
        });
        field("panic_in_drop_strategy", &mut |cx| cx.opaque.emit_u8(0));
        field("edition", &mut |cx| cx.opaque.emit_u8(3));
        for name in ["has_global_allocator", "has_alloc_error_handler", "has_panic_handler", "has_default_lib_allocator"] {
            field(name, &mut |cx| cx.opaque.emit_bool(false));
        }
        field("externally_implementable_items", &mut |cx| cx.array_ref(Array::EMPTY));
        field("crate_deps", &mut |cx| cx.array_ref(dependencies));
        for name in ["dylib_dependency_formats", "lib_features", "stability_implications", "lang_items"] {
            field(name, &mut |cx| cx.array_ref(Array::EMPTY));
        }
        field("lang_items_missing", &mut |cx| cx.array_ref(missing_lang_items));
        for name in [
            "stripped_cfg_items", "diagnostic_items", "canonical_symbols", "fake_doc_items", "native_libraries", "foreign_modules",
            ] {
            field(name, &mut |cx| cx.array_ref(Array::EMPTY));
        }
        field("traits", &mut |cx| cx.array_ref(traits));
        field("impls", &mut |cx| cx.array_ref(impls));
        field("incoherent_impls", &mut |cx| cx.array_ref(Array::EMPTY));
        field("interpret_alloc_index", &mut |cx| cx.array_ref(Array::EMPTY));
        field("proc_macro_data", &mut |cx| cx.opaque.emit_u8(0));
        field("tables", &mut |cx| for table in &tables { cx.table_ref(*table); });
        for name in ["debugger_visualizers", "exportable_items", "stable_order_of_exportable_impls"] {
            field(name, &mut |cx| cx.array_ref(Array::EMPTY));
        }
        field("exported_non_generic_symbols", &mut |cx| cx.array_ref(exports));
        field("exported_generic_symbols", &mut |cx| cx.array_ref(Array::EMPTY));
        field("syntax_contexts", &mut |cx| cx.table_ref(syntax_contexts));
        field("expn_data", &mut |cx| cx.table_ref(expansion_data));
        field("expn_hashes", &mut |cx| cx.table_ref(expansion_hashes));
        field("def_path_hash_map", &mut |cx| cx.distance(path_hash_map));
        field("source_map", &mut |cx| cx.table_ref(source_map));
        field("target_modifiers", &mut |cx| cx.array_ref(target_modifiers));
        field("denied_partial_mitigations", &mut |cx| cx.array_ref(denied_partial_mitigations));
        for name in ["compiler_builtins", "needs_allocator", "needs_panic_runtime", "no_builtins", "panic_runtime", "profiler_runtime"] {
            field(name, &mut |cx| cx.opaque.emit_bool(false));
        }
        field("symbol_mangling_version", &mut |cx| cx.opaque.emit_u8(1));
        field("specialization_enabled_in", &mut |cx| cx.opaque.emit_bool(false));
        assert_eq!(field_index, ROOT_FIELDS.len());
    });

    // Original metadata hashing uses XXH3-128 over a supplemental semantic
    // fingerprint followed by the bytes through CrateRoot, with header slots
    // zero during hashing. Native checked source/request/object identities take
    // the place of the Rust HIR/query-state supplement. This is a conservative
    // native change detector, not a claim of byte-identical Rust HIR hashing.
    let supplement = semantic_fingerprint(input, sdk, link);
    let mut hasher = XxHash3_128::new();
    hasher.write(&supplement[..16]); hasher.write(&cx.opaque.data);
    let crate_hash = hasher.finish_128().to_le_bytes();
    let unhashed = cx.node(|cx| {
        cx.opaque.emit_str(input.extra_filename); // Actual original request, outside the semantic hash.
        cx.opaque.emit_usize(sdk.dependencies.len() + 1);
        cx.opaque.emit_str(""); // LOCAL_CRATE filler.
        for dep in &sdk.dependencies { cx.opaque.emit_str(&dep.extra_filename); }
    });
    cx.opaque.emit_raw_bytes(b"rust-end-file");
    let mut metadata = cx.opaque.finish();
    metadata[8..16].copy_from_slice(&(root as u64).to_le_bytes());
    metadata[16..24].copy_from_slice(&(unhashed as u64).to_le_bytes());
    metadata[24..40].copy_from_slice(&crate_hash);
    let metadata_stub = (input.embed_metadata == Some(false)).then(|| {
        // Original encode_metadata stub: only CrateHeader, unhashed position 0,
        // and the already computed full metadata SVH. No second analysis/hash.
        let mut stub = Context::empty();
        stub.metadata_header(sdk);
        let root = stub.node(|cx| cx.crate_header(input.crate_name, true));
        stub.opaque.emit_raw_bytes(b"rust-end-file");
        let mut bytes = stub.opaque.finish();
        bytes[8..16].copy_from_slice(&(root as u64).to_le_bytes());
        bytes[24..40].copy_from_slice(&crate_hash);
        bytes
    });
    Ok(Encoded { metadata, metadata_stub, stable_crate_id: stable_id, crate_hash, symbols: mappings })
}

fn codegen_attrs(cx: &mut Context, inline_never: bool) {
    // Original CodegenFnAttrs::new for the admitted ordinary function/profile:
    // every field follows the original struct's derived serialization order.
    cx.opaque.emit_u32(0); // flags
    cx.opaque.emit_u8(if inline_never { 3 } else { 0 }); // InlineAttr::Never or None.
    cx.opaque.emit_u8(0); // optimize Default
    cx.opaque.emit_u8(0); // symbol_name None: preserve original v0 mangling
    cx.opaque.emit_usize(0); // foreign_item_symbol_aliases
    cx.opaque.emit_u8(0); // link_ordinal
    cx.opaque.emit_usize(0); // target_features
    cx.opaque.emit_bool(false); // safe_target_features
    cx.opaque.emit_u8(0); // linkage
    cx.opaque.emit_u8(0); // import_linkage
    cx.opaque.emit_u8(0); // link_section
    cx.opaque.emit_u16(0); // Original SanitizerSet empty (fixed-width u16).
    cx.opaque.emit_u8(2); // RtsanSetting::Caller
    for _ in 0..6 { cx.opaque.emit_u8(0); } // Remaining six Option fields.
}

/// rustc_span::normalize_newlines records the offset after each retained LF
/// and the cumulative number of removed CR bytes. Borrow unchanged source.
fn normalize_crlf(source: &str) -> (Cow<'_, str>, Vec<(u32, u32)>) {
    let mut lines = source.match_indices("\r\n").peekable();
    if lines.peek().is_none() {
        return (Cow::Borrowed(source), Vec::new());
    }
    let mut normalized = String::with_capacity(source.len());
    let mut positions = Vec::new();
    let mut previous = 0;
    for (start, _) in lines {
        normalized.push_str(&source[previous..start]);
        normalized.push('\n');
        previous = start + 2;
        positions.push((normalized.len() as u32, (previous - normalized.len()) as u32));
    }
    normalized.push_str(&source[previous..]);
    (Cow::Owned(normalized), positions)
}

fn encode_source_file(cx: &mut Context, input: &Input<'_>, normalized_source: &str, stable_id: u64) {
    let source_path = input.source_path;
    let working_directory = input.working_directory;
    cx.opaque.emit_u8(0); // FileName::Real
    cx.opaque.emit_u8(0); // RealFileName.local removed by update_for_crate_metadata
    cx.opaque.emit_str(source_path);
    cx.opaque.emit_str(working_directory);
    cx.opaque.emit_str(source_path); // Already absolute embeddable_name.
    cx.opaque.emit_u8(0); // No remap scopes in this original request.
    cx.opaque.emit_u8(0); // SourceFileHashAlgorithm::Md5
    cx.opaque.emit_usize(32);
    cx.opaque.emit_raw_bytes(&Md5::digest(input.source.as_bytes()));
    cx.opaque.emit_raw_bytes(&[0; 16]);
    cx.opaque.emit_u8(0); // checksum_hash None
    cx.opaque.emit_u32(normalized_source.len() as u32);
    cx.opaque.emit_u32(input.source.len() as u32); // Original unnormalized_source_len.

    // Original SourceFile difference-list encoding and normalized UTF-8 positions.
    let mut lines = vec![0u32];
    let mut multibyte = Vec::new();
    for (index, ch) in normalized_source.char_indices() {
        if ch == '\n' && index + 1 < normalized_source.len() {
            lines.push((index + 1) as u32);
        } else if !ch.is_ascii() {
            multibyte.push((index as u32, ch.len_utf8() as u8));
        }
    }
    cx.opaque.emit_u32(lines.len() as u32);
    let max_line_length = lines.windows(2).map(|pair| pair[1] - pair[0]).max().unwrap_or(0);
    let bytes_per_diff = match max_line_length { 0..=0xff => 1, 0x100..=0xffff => 2, _ => 4 };
    cx.opaque.emit_u8(bytes_per_diff);
    for pair in lines.windows(2) {
        cx.opaque.emit_raw_bytes(&(pair[1] - pair[0]).to_le_bytes()[..bytes_per_diff as usize]);
    }
    cx.opaque.emit_usize(multibyte.len());
    for (position, bytes) in multibyte {
        cx.opaque.emit_u32(position);
        cx.opaque.emit_u8(bytes);
    }
    cx.opaque.emit_raw_bytes(&source_file_id(input, stable_id));
    cx.opaque.emit_usize(cx.normalized_positions.len());
    for &(position, difference) in &cx.normalized_positions {
        cx.opaque.emit_u32(position);
        cx.opaque.emit_u32(difference);
    }
    cx.opaque.emit_u32(0); // local crate
}

fn source_file_id(input: &Input<'_>, stable_id: u64) -> [u8; 16] {
    use crate::upstream::stable_hash::{FromStableHash, SipHasher128Hash, StableSipHasher128};
    struct Hash128([u64; 2]);
    impl FromStableHash for Hash128 {
        type Hash = SipHasher128Hash;
        fn from(SipHasher128Hash(parts): Self::Hash) -> Self { Self(parts) }
    }
    // StableSourceFileId::from_filename_for_export over the exact exported
    // RealFileName. The trivial enum variants use upstream StableHash's u8
    // discriminants; path components use its original explicit boundary tags.
    let mut hash = StableSipHasher128::new();
    hash.write_u8(0); // FileName::Real
    hash.write_u8(0); // local None
    stable_path(input.source_path, &mut hash);
    stable_path(input.working_directory, &mut hash);
    stable_path(input.source_path, &mut hash);
    hash.write_u8(0); // scopes
    hash.write_u8(1); hash.write_u64(stable_id); // Some(StableCrateId)
    let Hash128([a, b]) = hash.finish::<Hash128>();
    let mut bytes = [0; 16]; bytes[..8].copy_from_slice(&a.to_le_bytes()); bytes[8..].copy_from_slice(&b.to_le_bytes()); bytes
}

fn stable_path(path: &str, hash: &mut crate::upstream::stable_hash::StableSipHasher128) {
    // This source context is explicitly POSIX absolute Unicode. The original
    // Path::components removes repeated separators and inner CurDir entries,
    // keeps ParentDir, and emits one RootDir. Hash original encoded component
    // bytes, exactly as Unix OsStr::hash, with original stable-path tags.
    hash.write_u8(0); // StartOfPath
    hash.write_usize(0); hash.write_u8(7); // RootDir
    let components = path.split('/').filter(|part| !part.is_empty() && *part != ".");
    for (index, component) in components.enumerate() {
        hash.write_usize(index + 1);
        if component == ".." { hash.write_u8(9); }
        else { hash.write_u8(10); component.as_bytes().hash(hash); }
    }
    hash.write_u8(11); // EndOfPath
}

fn semantic_fingerprint(input: &Input<'_>, sdk: &Sdk, link: bool) -> [u8; 16] {
    use crate::upstream::stable_hash::{FromStableHash, SipHasher128Hash, StableSipHasher128};
    struct Fingerprint([u64; 2]);
    impl FromStableHash for Fingerprint {
        type Hash = SipHasher128Hash;
        fn from(SipHasher128Hash(parts): Self::Hash) -> Self { Self(parts) }
    }
    let mut digest = StableSipHasher128::new();
    for part in [b"lustr-completed-scalar-metadata-v2".as_slice(), input.source.as_bytes(),
        input.crate_name.as_bytes(), input.metadata.as_bytes(), crate::TARGET.as_bytes(),
        sdk.receipt_hash.as_slice()] {
        digest.write_u64(part.len() as u64); digest.write(part);
    }
    if let Some(dependency) = sdk.direct_dependency {
        digest.write(b"lustr-direct-dependency");
        digest.write(&dependency);
    }
    digest.write_u64(input.objects.len() as u64);
    for object in input.objects {
        digest.write_u64(object.member.len() as u64); digest.write(object.member.as_bytes());
        digest.write(&object.producer_object_sha256); digest.write_u64(object.definitions.len() as u64);
        for definition in &object.definitions { digest.write_u64(definition.krate); digest.write_u64(definition.path); }
    }
    digest.write(&[input.profile.overflow_checks as u8, input.profile.debug_assertions as u8]);
    if !link {
        digest.write(b"lustr-metadata-only-check");
    }
    // Upstream tracks the unstable Option<bool> itself, not just its effective
    // default. Keep the qualified absent-option encoding unchanged; explicit
    // true/false are distinct native supplemental facts. Forced lint/JSON are
    // excluded from the upstream crate hash and do not enter this supplement.
    if let Some(embed) = input.embed_metadata {
        digest.write(b"lustr-tracked-unstable-embed-metadata");
        digest.write_u8(embed as u8);
    }
    let Fingerprint([a, b]) = digest.finish::<Fingerprint>();
    let mut bytes = [0; 16]; bytes[..8].copy_from_slice(&a.to_le_bytes()); bytes[8..].copy_from_slice(&b.to_le_bytes()); bytes
}

#[cfg(test)]
mod path_tests {
    use super::*;
    use crate::upstream::stable_hash::{FromStableHash, SipHasher128Hash, StableSipHasher128};
    struct Words([u64; 2]);
    impl FromStableHash for Words {
        type Hash = SipHasher128Hash;
        fn from(SipHasher128Hash(words): Self::Hash) -> Self { Self(words) }
    }
    #[test]
    fn posix_adapter_preserves_original_path_and_os_string_hashes() {
        for path in ["/", "//", "/a/b", "/a//./b/../c/", "/./a/../../x", "/α/字.rs", "/a/b\\c"] {
            let mut original = StableSipHasher128::new();
            original.write_u8(0);
            for (index, component) in std::path::Path::new(path).components().enumerate() {
                original.write_usize(index);
                match component {
                    std::path::Component::RootDir => original.write_u8(7),
                    std::path::Component::CurDir => original.write_u8(8),
                    std::path::Component::ParentDir => original.write_u8(9),
                    std::path::Component::Normal(name) => { original.write_u8(10); name.hash(&mut original); }
                    std::path::Component::Prefix(_) => unreachable!(),
                }
            }
            original.write_u8(11);
            let mut adapted = StableSipHasher128::new();
            stable_path(path, &mut adapted);
            assert_eq!(original.finish::<Words>().0, adapted.finish::<Words>().0, "{path}");
        }
    }
}
