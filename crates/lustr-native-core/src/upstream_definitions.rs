//! First semantic import from genuine pinned Rust metadata.
//!
//! This bounded service admits public, nongeneric, non-Copy transparent integer
//! structs and safe Rust functions with represented integer/nominal signatures.
//! Every definition retains Rust's full stable crate/path identity. Native
//! bodies are checked once against these facts; Rust never checks those bodies.
//! Standalone metadata supplies declarations without an executable plan.
//! Wider types, obligations, modules, linked unwind and retained generations
//! still require their shared records.

use crate::{check, def, diag, driver, intern, resolve, ty};
use rustc_abi::{CanonAbi, ExternAbi};
use rustc_hir::def::{DefKind, Res};
use rustc_middle::ty::{self as upstream, TyCtxt, TypingEnv};
use rustc_infer::infer::TyCtxtInferExt;
use rustc_span::def_id::{DefId, CRATE_DEF_INDEX};
use rustc_target::callconv::{ArgAbi, ArgAttribute, ArgAttributes, FnAbi, PassMode};
use rustc_trait_selection::infer::InferCtxtExt;
use std::{cell::RefCell, sync::Arc};
use ty::TyIntern;

#[path = "upstream_abi.rs"]
pub(super) mod abi;

/// One immutable import generation, read from the same snapshot used by
/// the original Rust metadata loader.
pub(super) struct BoundFile {
    path: std::path::PathBuf,
    contents: Arc<[u8]>,
}

impl BoundFile {
    pub(super) fn snapshot(input: &std::path::Path, path: &std::path::Path) -> Result<Self, String> {
        let contents: Arc<[u8]> = std::fs::read(input).map_err(|e| format!("read imported artifact: {e}"))?.into();
        Self::snapshot_bytes(contents, path)
    }

    pub(super) fn snapshot_bytes(contents: Arc<[u8]>, path: &std::path::Path) -> Result<Self, String> {
        use std::io::Write;
        let mut output = std::fs::OpenOptions::new().write(true).create_new(true).open(path)
            .map_err(|e| format!("create imported snapshot: {e}"))?;
        output.write_all(&contents).map_err(|e| format!("write imported snapshot: {e}"))?;
        drop(output);
        let path = path.canonicalize().map_err(|e| format!("resolve imported snapshot: {e}"))?;
        Ok(Self { path, contents })
    }

    pub(super) fn path(&self) -> &std::path::Path { &self.path }
    pub(super) fn contents(&self) -> &[u8] { &self.contents }

    fn verify(&self) -> Result<(), String> {
        if std::fs::read(&self.path).map_err(|e| format!("verify imported snapshot: {e}"))?.as_slice() != self.contents.as_ref() {
            return Err("imported artifact changed during the shared request".into());
        }
        Ok(())
    }
}

/// An archive binds its objects to the exact bytes Rust reads for metadata.
pub(super) struct Archive {
    file: BoundFile,
    objects: Vec<Arc<[u8]>>,
}

impl Archive {
    pub(super) fn snapshot(input: &std::path::Path, path: &std::path::Path) -> Result<Self, String> {
        let file = BoundFile::snapshot(input, path)?;
        let extract = |operation: &str, member: Option<&str>| -> Result<Vec<u8>, String> {
            let mut command = std::process::Command::new("ar");
            command.arg(operation).arg(file.path());
            if let Some(member) = member { command.arg(member); }
            let result = command.output().map_err(|e| format!("start imported archive reader: {e}"))?;
            if !result.status.success() {
                return Err(format!("read imported archive: {}", String::from_utf8_lossy(&result.stderr)));
            }
            Ok(result.stdout)
        };
        let members = String::from_utf8(extract("t", None)?).map_err(|e| format!("imported archive member name: {e}"))?;
        let mut seen = std::collections::BTreeSet::new();
        let mut objects = Vec::new();
        for name in members.lines().filter(|name| name.ends_with(".o")) {
            if name.starts_with('-') || !name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
                || !seen.insert(name) {
                return Err("unsupported or duplicate imported object member".into());
            }
            objects.push(extract("p", Some(name))?.into());
        }
        if objects.is_empty() { return Err("imported archive contains no object code".into()); }
        file.verify()?;
        Ok(Self { file, objects })
    }

    pub(super) fn path(&self) -> &std::path::Path { self.file.path() }
    pub(super) fn contents(&self) -> &[u8] { self.file.contents() }
}

#[derive(Debug)]
pub enum Error {
    Unintegrated(String),
    Failed(String),
}

fn missing(reason: &str) -> Error { Error::Unintegrated(reason.into()) }

struct Nominal {
    native: def::DefId,
    disk: def::DiskDefId,
    name: String,
    field: ty::IntTy,
    layout: ty::layout::Layout,
}

#[derive(Clone, Copy)]
enum ValueKind {
    Integer(ty::IntTy),
    Nominal(usize),
}

struct Value {
    kind: ValueKind,
    size: u64,
    align: u64,
}

struct Function {
    native: def::DefId,
    disk: def::DiskDefId,
    name: String,
    symbol: String,
    inputs: Vec<Value>,
    output: Value,
}

/// Checked declarations and immutable inputs owned independently of the
/// upstream compiler session.
pub struct Provider {
    alias: String,
    root: def::DefId,
    stable: def::StableCrateHash,
    nominals: Vec<Nominal>,
    functions: Vec<Function>,
    dependencies: Vec<(String, Arc<[u8]>)>,
    emission: RefCell<Option<OwnedEmission>>,
    metadata_only: bool,
    adapter_count: usize,
}

/// No TyCtxt, arena reference or dependency archive pathname escapes into the
/// native continuation. Taking this plan moves it; it cannot be cloned/reused.
struct OwnedEmission {
    objects: Vec<Arc<[u8]>>,
    adapters: Option<abi::Module>,
}

impl driver::imports::Emission for OwnedEmission {
    fn emit(self: Box<Self>) -> Result<Vec<Arc<[u8]>>, String> {
        let Self { mut objects, adapters } = *self;
        if let Some(adapters) = adapters { objects.push(adapters.emit()?); }
        Ok(objects)
    }
}

fn native_type(program: &check::collect::Program, id: def::DefId) -> Result<ty::Ty, String> {
    program.tys.get(ty::TyKind::Adt(id, ty::EMPTY_ARGS)).ok_or_else(|| "missing imported nominal type".into())
}

pub(super) fn integer(actual: upstream::Ty<'_>) -> Option<ty::IntTy> {
    use ty::IntTy as N;
    Some(match actual.kind() {
        upstream::Int(upstream::IntTy::I8) => N::I8,
        upstream::Int(upstream::IntTy::I16) => N::I16,
        upstream::Int(upstream::IntTy::I32) => N::I32,
        upstream::Int(upstream::IntTy::I64) => N::I64,
        upstream::Int(upstream::IntTy::Isize) => N::Isize,
        upstream::Uint(upstream::UintTy::U8) => N::U8,
        upstream::Uint(upstream::UintTy::U16) => N::U16,
        upstream::Uint(upstream::UintTy::U32) => N::U32,
        upstream::Uint(upstream::UintTy::U64) => N::U64,
        upstream::Uint(upstream::UintTy::Usize) => N::Usize,
        _ => return None,
    })
}

impl Value {
    fn capture<'tcx>(tcx: TyCtxt<'tcx>, actual: upstream::Ty<'tcx>, nominal_types: &[upstream::Ty<'tcx>]) -> Result<Self, Error> {
        let kind = if let Some(value) = integer(actual) { ValueKind::Integer(value) }
            else { ValueKind::Nominal(nominal_types.iter().position(|&ty| ty == actual)
                .ok_or_else(|| missing("external signature type requires additional shared semantic records"))?) };
        let layout = tcx.layout_of(TypingEnv::fully_monomorphized().as_query_input(actual))
            .map_err(|e| Error::Failed(format!("upstream signature layout failed: {e:?}")))?;
        if layout.uninhabited || layout.largest_niche.is_some() {
            return Err(missing("external signature validity requires additional shared records"));
        }
        Ok(Self { kind, size: layout.size.bytes(), align: layout.align.bytes() })
    }

    fn install(&self, program: &mut check::collect::Program, nominals: &[Nominal]) -> Result<ty::Ty, String> {
        match self.kind {
            ValueKind::Integer(value) => Ok(program.tys.intern(ty::TyKind::Int(value))),
            ValueKind::Nominal(index) => native_type(program, nominals[index].native),
        }
    }

    fn verify(&self, program: &check::collect::Program, nominals: &[Nominal]) -> Result<ty::Ty, String> {
        let native = match self.kind {
            ValueKind::Integer(value) => program.tys.get(ty::TyKind::Int(value)).ok_or("missing imported integer type")?,
            ValueKind::Nominal(index) => native_type(program, nominals[index].native)?,
        };
        let abi = crate::cgen::native_type_abi(program, native).ok_or("native signature ABI is unavailable")?;
        if abi.size != self.size || abi.align != self.align || abi.param_dropped || abi.param_by_ptr || abi.ret_by_out {
            return Err("native signature representation differs from the admitted upstream ABI".into());
        }
        Ok(native)
    }

    fn matches_backend(&self, repr: rustc_abi::BackendRepr, nominals: &[Nominal]) -> bool {
        let field = match self.kind { ValueKind::Integer(field) => field, ValueKind::Nominal(index) => nominals[index].field };
        matches!(repr, rustc_abi::BackendRepr::Scalar(scalar)
            if matches!(scalar.primitive(), rustc_abi::Primitive::Int(integer, signed)
                if integer.size().bytes() == self.size && signed == field.signed()))
    }

    fn native_c_layout<'tcx>(&self, tcx: TyCtxt<'tcx>, actual: upstream::Ty<'tcx>) -> Result<rustc_abi::TyAndLayout<'tcx, upstream::Ty<'tcx>>, Error> {
        let mut layout = tcx.layout_of(TypingEnv::fully_monomorphized().as_query_input(actual))
            .map_err(|e| Error::Failed(format!("native C representation layout failed: {e:?}")))?;
        if matches!(self.kind, ValueKind::Nominal(_)) {
            // Native C emits a real one-field C struct, not Rust's scalar
            // repr(transparent) calling representation. Keep the authoritative
            // field offsets/size/alignment but classify this C aggregate with
            // the imported target ABI implementation. This is a separate view;
            // the original Rust type and its FnAbi remain unchanged.
            let mut aggregate = (**layout.layout).clone();
            aggregate.backend_repr = rustc_abi::BackendRepr::Memory { sized: true };
            layout.layout = tcx.mk_layout(aggregate);
        }
        Ok(layout)
    }
}

pub(super) fn ids(tcx: TyCtxt<'_>, original: DefId) -> (def::DefId, def::DiskDefId) {
    let hash = tcx.def_path_hash(original);
    let disk = def::DiskDefId {
        krate: def::StableCrateHash(hash.stable_crate_id().as_u64()),
        hash: def::DefPathHash(hash.local_hash().as_u64()),
    };
    (def::DefId { krate: def::CrateId(1), hash: disk.hash }, disk)
}

impl Provider {
    /// Consume only a privately constructed, complete archive generation.
    pub(super) fn capture(tcx: TyCtxt<'_>, alias: &str, archive: &Archive, adapter_control: abi::Control) -> Result<Self, Error> {
        Self::capture_from(tcx, alias, &archive.file, Some((archive, adapter_control)))
    }

    pub(super) fn capture_metadata(tcx: TyCtxt<'_>, alias: &str, file: &BoundFile) -> Result<Self, Error> {
        Self::capture_from(tcx, alias, file, None)
    }

    fn capture_from(tcx: TyCtxt<'_>, alias: &str, file: &BoundFile, link: Option<(&Archive, abi::Control)>) -> Result<Self, Error> {
        if tcx.sess.target.llvm_target.as_ref() != "x86_64-unknown-linux-gnu"
            || link.is_some() && tcx.sess.panic_strategy().unwinds() {
            return Err(missing("this semantic import requires x86_64 Linux; executable emission requires an explicit abort profile"));
        }
        let candidates: Vec<_> = tcx.crates(()).iter().copied().filter(|&c| tcx.crate_name(c).as_str() == alias).collect();
        let [krate] = candidates.as_slice() else { return Err(missing("one unaliased external crate is required")) };
        file.verify().map_err(Error::Failed)?;
        let source = tcx.used_crate_source(*krate);
        let actual = if link.is_some() { &source.rlib } else { &source.rmeta };
        if actual.as_deref() != Some(file.path()) {
            return Err(Error::Failed("upstream metadata came from a different artifact generation".into()));
        }
        if link.is_some() && tcx.required_panic_strategy(*krate) != Some(rustc_target::spec::PanicStrategy::Abort) {
            return Err(missing("the imported object must declare its abort strategy in genuine Rust metadata"));
        }
        let root = DefId { krate: *krate, index: CRATE_DEF_INDEX };
        let environment = TypingEnv::fully_monomorphized();
        let mut nominals = Vec::new();
        let mut nominal_types = Vec::new();
        let children = tcx.module_children(root);
        for child in children {
            if !child.vis.is_public() { continue; }
            if !child.reexport_chain.is_empty() { return Err(missing("reexport ownership is not integrated by this import slice")); }
            let Res::Def(kind, id) = child.res else { return Err(missing("external nondefinition binding")) };
            if !matches!(kind, DefKind::Struct | DefKind::Fn | DefKind::Ctor(..)) {
                return Err(missing("external modules, constants and traits need their shared records"));
            }
            if kind != DefKind::Struct { continue; }
            let actual = tcx.normalize_erasing_regions(environment, tcx.type_of(id).instantiate_identity());
            let upstream::Adt(adt, args) = actual.kind() else { return Err(Error::Failed("struct metadata has a non-ADT type".into())) };
            let variant = adt.non_enum_variant();
            let field = variant.fields.iter().next().ok_or_else(|| missing("external aggregate needs a nonzero field"))?;
            let field_type = tcx.normalize_erasing_regions(environment, field.ty(tcx, *args));
            let field_integer = integer(field_type).ok_or_else(|| missing("external aggregate field requires additional shared type records"))?;
            if !args.is_empty() || tcx.generics_of(id).count() != 0 || !tcx.param_env(id).is_empty()
                || !adt.is_struct() || !adt.repr().transparent() || adt.repr().pack.is_some() || adt.repr().align.is_some()
                || adt.is_variant_list_non_exhaustive() || variant.is_field_list_non_exhaustive()
                || variant.fields.len() != 1 || !field.vis.is_public()
                || variant.ctor_kind() != Some(rustc_hir::def::CtorKind::Fn)
                || actual.needs_drop(tcx, environment) || tcx.type_is_copy_modulo_regions(environment, actual) {
                return Err(missing("external aggregate needs additional type, obligation, Copy or drop records"));
            }
            if !tcx.inherent_impls(id).is_empty() {
                return Err(missing("external inherent methods require their shared definition records"));
            }
            let layout = tcx.layout_of(environment.as_query_input(actual)).map_err(|e| Error::Failed(format!("upstream layout failed: {e:?}")))?;
            let field_layout = tcx.layout_of(environment.as_query_input(field_type)).map_err(|e| Error::Failed(format!("upstream field layout failed: {e:?}")))?;
            if layout.size != field_layout.size || layout.align != field_layout.align || layout.fields.offset(0).bytes() != 0
                || layout.largest_niche.is_some() || layout.uninhabited {
                return Err(missing("transparent aggregate layout is outside this call boundary"));
            }
            // Native structural auto traits are sound for this shape only
            // after the actual upstream solver establishes the same facts.
            // A negative impl is not visible in field layout or Copy/drop.
            let inference = tcx.infer_ctxt().build(upstream::TypingMode::non_body_analysis());
            for definition in [tcx.get_diagnostic_item(rustc_span::sym::Send), tcx.lang_items().sync_trait(), tcx.lang_items().unpin_trait()] {
                let definition = definition.ok_or_else(|| Error::Failed("missing upstream auto-trait identity".into()))?;
                if !inference.type_implements_trait(definition, [actual], upstream::ParamEnv::empty()).must_apply_modulo_regions() {
                    return Err(missing("negative or unresolved imported Send/Sync/Unpin facts require their shared trait record"));
                }
            }
            let (native, disk) = ids(tcx, id);
            nominal_types.push(actual);
            nominals.push(Nominal {
                native, disk, name: child.ident.name.to_string(), field: field_integer,
                layout: ty::layout::Layout { size: layout.size.bytes(), align: layout.align.bytes(), field_offsets: vec![layout.fields.offset(0).bytes()], variant_offsets: Vec::new(),
                    niche: None, enum_tag: None, uninhabited: false },
            });
        }
        if !tcx.trait_impls_in_crate(*krate).is_empty() {
            return Err(missing("external trait implementations require their shared obligation records"));
        }
        // Metadata may retain a body for downstream monomorphization without
        // emitting an object definition, for example #[inline(always)]. Such
        // a function needs the complementary instance-materialization service;
        // a later unresolved-symbol linker error must not discover capability.
        let mut available_functions = std::collections::BTreeSet::new();
        let mut available_definitions = std::collections::BTreeSet::new();
        let mut required_definitions = std::collections::BTreeSet::new();
        for object in link.iter().flat_map(|(archive, _)| &archive.objects) {
            let symbols = crate::unit_object::symbols(object)
                .map_err(|e| Error::Failed(format!("invalid imported definition object: {e}")))?;
            available_functions.extend(symbols.functions);
            available_definitions.extend(symbols.defined);
            required_definitions.extend(symbols.undefined);
        }
        if required_definitions.iter().any(|name| !available_definitions.contains(name)) {
            return Err(missing("imported object closure requires additional shared runtime or dependency records"));
        }
        let mut functions = Vec::new();
        let mut adapters = Vec::new();
        for child in children {
            if !child.vis.is_public() { continue; }
            let Res::Def(DefKind::Fn, id) = child.res else { continue };
            if tcx.generics_of(id).count() != 0 || !tcx.param_env(id).is_empty() || tcx.is_const_fn(id) {
                return Err(missing("external generic or const function requires a shared obligation/body record"));
            }
            let instance = upstream::Instance::mono(tcx, id);
            let signature = tcx.normalize_erasing_regions(environment, tcx.fn_sig(id).instantiate_identity());
            if !signature.bound_vars().is_empty() {
                return Err(missing("external bound regions require their shared region signature"));
            }
            let signature = signature.skip_binder();
            if !signature.fn_sig_kind.is_safe() || signature.abi() != ExternAbi::Rust
                || signature.c_variadic() {
                return Err(missing("external function qualifier/signature is outside this call boundary"));
            }
            let inputs: Vec<_> = signature.inputs().iter().map(|&actual| Value::capture(tcx, actual, &nominal_types)).collect::<Result<_, _>>()?;
            let output = Value::capture(tcx, signature.output(), &nominal_types)?;
            let (native, disk) = ids(tcx, id);
            let symbol = tcx.symbol_name(instance).name;
            if link.is_none() {
                functions.push(Function { native, disk, name: child.ident.name.to_string(), symbol: symbol.to_owned(), inputs, output });
                continue;
            }
            let rust_abi = tcx.fn_abi_of_instance(environment.as_query_input((instance, upstream::List::empty())))
                .map_err(|e| Error::Failed(format!("upstream Rust FnAbi failed: {e:?}")))?;
            let argument = |value: &Value, actual| -> Result<_, Error> {
                Ok(ArgAbi::new(value.native_c_layout(tcx, actual)?, |_, _| {
                    let mut attributes = ArgAttributes::new();
                    attributes.regular |= ArgAttribute::NoUndef;
                    attributes
                }))
            };
            let mut c_abi = FnAbi {
                args: inputs.iter().zip(signature.inputs()).map(|(value, &actual)| argument(value, actual)).collect::<Result<Vec<_>, _>>()?.into_boxed_slice(),
                ret: argument(&output, signature.output())?, c_variadic: false,
                fixed_count: inputs.len().try_into().map_err(|_| Error::Failed("native C signature has too many arguments".into()))?,
                conv: CanonAbi::C, can_unwind: false,
            };
            c_abi.adjust_for_foreign_abi(&upstream::layout::LayoutCx::new(tcx, environment), ExternAbi::C { unwind: false });
            let direct = |a: &PassMode, b: &PassMode| matches!((a, b), (PassMode::Direct(a), PassMode::Direct(b)) if a == b);
            if !matches!((rust_abi.conv, c_abi.conv), (CanonAbi::Rust, CanonAbi::C))
                || rust_abi.can_unwind || c_abi.can_unwind || rust_abi.c_variadic || c_abi.c_variadic
                || rust_abi.fixed_count != c_abi.fixed_count || rust_abi.args.len() != inputs.len() || c_abi.args.len() != inputs.len() {
                return Err(missing("actual upstream Rust and C FnAbi conventions are outside this call boundary"));
            }
            if !inputs.iter().zip(&rust_abi.args).all(|(value, rust)| value.matches_backend(rust.layout.backend_repr, &nominals))
                || !output.matches_backend(rust_abi.ret.layout.backend_repr, &nominals) {
                return Err(missing("integer call lowering differs from the authoritative signature representation"));
            }
            if symbol.is_empty() || !symbol.bytes().enumerate().all(|(index, byte)| byte == b'_' || byte.is_ascii_alphabetic() || index != 0 && byte.is_ascii_digit()) {
                return Err(missing("the actual imported symbol needs a quoted linkage representation"));
            }
            if !available_functions.contains(symbol.as_bytes()) {
                return Err(Error::Unintegrated(format!("imported instance {} requires complementary body materialization", tcx.def_path_str(id))));
            }
            let symbol = if rust_abi.args.iter().zip(&c_abi.args).all(|(a, b)| direct(&a.mode, &b.mode))
                && direct(&rust_abi.ret.mode, &c_abi.ret.mode) { symbol.to_owned() }
            else {
                let adapter = abi::Function::capture(format!("lustr_import_abi_{:016x}_{:016x}", disk.krate.0, disk.hash.0),
                    symbol, &inputs, &output, &rust_abi.args.iter().map(|arg| arg.mode.clone()).collect::<Vec<_>>(),
                    &rust_abi.ret.mode, &c_abi.args.iter().map(|arg| arg.mode.clone()).collect::<Vec<_>>(), &c_abi.ret.mode)?;
                if available_definitions.contains(adapter.symbol.as_bytes()) {
                    return Err(Error::Failed("integer adapter symbol is already owned by the imported archive".into()));
                }
                let symbol = adapter.symbol.clone();
                adapters.push(adapter);
                symbol
            };
            functions.push(Function { native, disk, name: child.ident.name.to_string(),
                symbol, inputs, output });
        }
        if functions.is_empty() { return Err(missing("this shared call slice needs a real function")); }
        let mut identities = std::collections::BTreeSet::new();
        for id in nominals.iter().map(|n| n.disk).chain(functions.iter().map(|f| f.disk)) {
            if !identities.insert(id) { return Err(Error::Failed("duplicate upstream definition identity".into())); }
        }
        let mut dependencies = vec![(format!("rust:{}:{:?}", env!("LUSTR_IMPORTED_COMMIT"), tcx.crate_hash(*krate)), file.contents.clone())];
        let (emission, adapter_count) = if let Some((archive, control)) = link {
            let adapters = abi::Module::capture(&adapters, control)?;
            dependencies.extend(adapters.iter().flat_map(|module| module.dependencies()));
            let count = adapters.as_ref().map_or(0, |module| module.count());
            (Some(OwnedEmission { objects: archive.objects.clone(), adapters }), count)
        } else {
            (None, 0)
        };
        Ok(Self { alias: alias.into(), root: ids(tcx, root).0, stable: ids(tcx, root).1.krate, nominals, functions,
            dependencies, adapter_count, emission: RefCell::new(emission), metadata_only: link.is_none() })
    }

    fn verify(&self, program: &check::collect::Program) -> Result<(), String> {
        if program.imported_crates.get(&self.root.krate) != Some(&self.stable) {
            return Err("imported stable crate identity changed".into());
        }
        for nominal in &self.nominals {
            let native = native_type(program, nominal.native)?;
            let layout = ty::layout::layout_of_ty(&program.tys, program, native).map_err(|_| "native imported layout failed")?;
            if layout != nominal.layout || check::collect::is_copy(program, &program.tys, native)
                || check::collect::needs_drop(program, &program.tys, native) {
                return Err("native representation differs from the authoritative imported nominal".into());
            }
            let abi = crate::cgen::native_type_abi(program, native).ok_or("native ABI is unavailable")?;
            if abi.size != layout.size || abi.align != layout.align || abi.param_dropped || abi.param_by_ptr || abi.ret_by_out {
                return Err("native emitted C passing mode differs from the admitted upstream ABI".into());
            }
            if driver::graph::def_crate_hash(program, nominal.native) != nominal.disk.krate.0 {
                return Err("imported definition identity changed".into());
            }
        }
        for function in &self.functions {
            let (params, result, linkage) = program.native_fn(function.native).ok_or("missing imported signature")?;
            let expected_params: Vec<_> = function.inputs.iter().map(|value| value.verify(program, &self.nominals)).collect::<Result<_, _>>()?;
            if params != expected_params
                || result != function.output.verify(program, &self.nominals)?
                || linkage.id != function.disk || linkage.symbol != function.symbol {
                return Err("imported function identity/signature changed".into());
            }
            let regions = program.regions.native_fns.get(&function.native).ok_or("missing imported region signature")?;
            if regions.params != params || regions.ret != result || regions.n_univ != 0
                || !regions.facts.is_empty() || !regions.names.is_empty() || !regions.early.is_empty()
                || !regions.fn_bounds.is_empty() || !regions.trait_bounds.is_empty() || !regions.item_bounds.is_empty()
                || regions.opaque_caps.is_some() || !regions.header.is_empty() || !regions.bounds.is_empty() {
                return Err("imported region signature differs from the admitted upstream signature".into());
            }
        }
        Ok(())
    }
}

impl driver::imports::Definitions for Provider {
    fn metadata_only(&self) -> bool { self.metadata_only }

    fn cargo_definitions(&self) -> Option<Vec<Arc<driver::imports::CargoDefinitions>>> {
        if !self.metadata_only || !self.nominals.is_empty() {
            return None;
        }
        let integer = |value: &Value| match value.kind {
            ValueKind::Integer(kind) => Some(kind),
            ValueKind::Nominal(_) => None,
        };
        let functions = self.functions.iter().map(|function| Some(driver::imports::ScalarFunction {
            name: function.name.clone(),
            hash: function.disk.hash,
            parameters: function.inputs.iter().map(integer).collect::<Option<Vec<_>>>()?,
            result: integer(&function.output)?,
            symbol: function.symbol.clone(),
        })).collect::<Option<Vec<_>>>()?;
        Some(vec![Arc::new(driver::imports::CargoDefinitions {
            alias: self.alias.clone(),
            crate_name: self.alias.clone(),
            stable: self.stable,
            index: self.root.krate,
            functions,
            projection: None,
        })])
    }

    fn mount(&self, tables: &mut resolve::imports::ResolvedTables, interner: &intern::Interner) -> Result<(), diag::Diag> {
        use resolve::imports::{Bind, DefInfo, ExternInfo, InsertOut, ResKind, Target, VisKind};
        if tables.module_defs.iter().chain(tables.def_info.keys()).any(|definition| definition.krate == self.root.krate) {
            return Err(diag::Diag::new("L0508", diag::Span::default(), "imported crate index is already owned", "Allocate a distinct shared crate identity."));
        }
        let root = tables.add_crate_root(interner.intern(&self.alias));
        tables.module_defs[root.0 as usize] = self.root;
        tables.externs.push(ExternInfo { name: self.alias.clone(), def: self.root, is_std_lib: false,
            owner: ty::defs::ROOT_MODULE, root_module: Some(root), unsupported: None });
        let failure = || diag::Diag::new("L0508", diag::Span::default(), "conflicting imported definition binding", "Use one authoritative dependency generation.");
        for nominal in &self.nominals {
            let flat = interner.intern(&format!("$lustr$upstream${:016x}${:016x}", nominal.disk.krate.0, nominal.disk.hash.0));
            tables.def_info.insert(nominal.native, DefInfo { kind: ResKind::Adt, flat, module: root, vis: VisKind::Pub });
            tables.display_names.insert(flat, interner.intern(&format!("{}::{}", self.alias, nominal.name)));
            tables.tuple_field_vis.insert(nominal.native, (root, vec![VisKind::Pub]));
            for (index, namespace) in [driver::graph::Namespace::Type, driver::graph::Namespace::Value].into_iter().enumerate() {
                if tables.flat[index].insert(flat, nominal.native).is_some()
                    || !matches!(tables.insert(root, namespace, interner.intern(&nominal.name), Bind {
                        target: Target::Def(nominal.native), vis: VisKind::Pub, rank: 2, def_module: root,
                    }), InsertOut::Changed) { return Err(failure()); }
            }
        }
        for function in &self.functions {
            let flat = interner.intern(&format!("$lustr$upstream${:016x}${:016x}", function.disk.krate.0, function.disk.hash.0));
            tables.def_info.insert(function.native, DefInfo { kind: ResKind::Fn, flat, module: root, vis: VisKind::Pub });
            tables.display_names.insert(flat, interner.intern(&format!("{}::{}", self.alias, function.name)));
            if tables.flat[1].insert(flat, function.native).is_some()
                || !matches!(tables.insert(root, driver::graph::Namespace::Value, interner.intern(&function.name), Bind {
                    target: Target::Def(function.native), vis: VisKind::Pub, rank: 2, def_module: root,
                }), InsertOut::Changed) { return Err(failure()); }
        }
        Ok(())
    }

    fn install(&self, program: &mut check::collect::Program) -> Result<(), Vec<diag::Diag>> {
        let report = |reason: String| vec![diag::Diag::new("L0508", diag::Span::default(), reason, "Imported semantic/ABI admission failed.")];
        if self.root.krate == program.krate || self.root.krate == ty::BUILTIN_CRATE
            || program.imported_crates.insert(self.root.krate, self.stable).is_some() {
            return Err(report("imported stable crate identity is already owned".into()));
        }
        for nominal in &self.nominals {
            let word = program.tys.intern(ty::TyKind::Int(nominal.field));
            let info = program.resolved.as_ref().and_then(|tables| tables.def_info.get(&nominal.native)).ok_or_else(|| report("imported type has no original binding".into()))?;
            let (flat, module) = (info.flat, info.module);
            let local = ty::AdtId(program.adts.len() as u32);
            program.tys.register_adt(nominal.native, local);
            program.defs.register_adt_shape(nominal.native, local);
            program.adts.push(check::collect::AdtDef::new(flat, false, vec![check::collect::Variant {
                name: flat, shape: check::collect::VariantShape::Tuple, fields: vec![check::collect::Field {
                    name: program.interner.intern("0"), ty: word, span: diag::Span::default(), derive_bounds: Vec::new(),
                }], disc: 0, non_exhaustive: false,
            }], Default::default(), diag::Span::default(), ty::layout::Repr { kind: ty::layout::ReprKind::Transparent, ..Default::default() }, false));
            program.gx.adt_info.push((nominal.native, 0));
            program.gx.adt_generics.push(Default::default());
            program.gx.adt_envs.push(Vec::new());
            program.gx.adt_scopes.push(Default::default());
            program.gx.adt_instance_of.push(None);
            program.regions.adt_fields.insert(nominal.native, vec![vec![word]]);
            let mut data = ty::defs::DefData::skeleton(ty::defs::DefKind::Struct, None, flat, ty::defs::Vis::Pub, diag::Span::default());
            data.module = module;
            program.defs.insert(nominal.native, data);
            program.type_names.insert(flat, local);
        }
        for function in &self.functions {
            let info = program.resolved.as_ref().and_then(|tables| tables.def_info.get(&function.native)).ok_or_else(|| report("imported function has no original binding".into()))?;
            let mut data = ty::defs::DefData::skeleton(ty::defs::DefKind::ForeignFn, None, info.flat, ty::defs::Vis::Pub, diag::Span::default());
            data.module = info.module;
            let params = function.inputs.iter().map(|value| value.install(program, &self.nominals)).collect::<Result<Vec<_>, _>>().map_err(&report)?;
            let ret = function.output.install(program, &self.nominals).map_err(&report)?;
            data.sig = ty::SigData::Fn { params: params.clone(), ret };
            data.native_linkage = Some(Box::new(ty::defs::NativeLinkage { id: function.disk, symbol: function.symbol.clone() }));
            program.defs.insert(function.native, data);
            // The checked upstream binder is empty. Every admitted argument
            // and result is an integer or a nongeneric transparent integer
            // nominal. Preserve that real zero-region signature for item-to-
            // pointer coercion and call proofs; absence is not such a proof.
            program.regions.native_fns.insert(function.native, crate::borrowck::RegionSig {
                n_univ: 0, params, ret, facts: Vec::new(), names: Vec::new(), early: Vec::new(),
                fn_bounds: Vec::new(), trait_bounds: Vec::new(), item_bounds: Vec::new(),
                opaque_caps: None, header: Vec::new(), bounds: Vec::new(),
            });
        }
        self.verify(program).map_err(report)
    }

    fn validate_checked(&self, program: &check::collect::Program) -> Result<(), String> {
        self.verify(program)
    }
    fn dependencies(&self) -> Vec<driver::imports::Dependency> {
        self.dependencies.iter().cloned()
            .map(|(identity, contents)| driver::imports::Dependency { identity, contents }).collect()
    }
    fn take_emission(&self) -> Result<Box<dyn driver::imports::Emission + 'static>, String> {
        self.emission.borrow_mut().take().map(|emission| Box::new(emission) as Box<dyn driver::imports::Emission>)
            .ok_or_else(|| "imported emission plan was already transferred".into())
    }
}

impl Provider {
    pub(super) fn adapter_count(&self) -> usize { self.adapter_count }
}
