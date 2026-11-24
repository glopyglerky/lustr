//! Compose Lust objects with the pinned Rust compiler for host artifacts.
//! The original Rust driver validates and emits the rest of the original crate.
use crate::host_retained_link;
use rustc_abi::CanonAbi;
use rustc_ast::{
    AttrStyle, BinOpKind, BindingMode, BlockCheckMode, Defaultness, Expr, ExprKind, FnRetTy,
    ForeignItemKind, ForeignMod, Item, ItemKind, PatKind, Safety, Stmt, StmtKind, StrLit,
    StrStyle, Ty, TyKind, UseTree, UseTreeKind, Visibility, VisibilityKind, token::LitKind,
};
use rustc_codegen_ssa::{
    CompiledModule, CompiledModules, CrateInfo, ModuleKind, TargetConfig, traits::CodegenBackend,
};
use rustc_driver::{Callbacks, Compilation};
use rustc_hir::PrimTy;
use rustc_interface::interface::{Compiler, Config};
use rustc_metadata::EncodedMetadata;
use rustc_middle::{dep_graph::WorkProductMap, ty::{self, Instance, TyCtxt, TypingEnv}, util::Providers};
use rustc_session::{
    CodegenBackendInit, EarlyDiagCtxt, EarlySession, IncrCompSession, Session,
    config::{DebugInfo, ErrorOutputType, Input, OptLevel, OutFileName, OutputFilenames, OutputType},
};
use rustc_span::{Ident, SourceFileHash, SourceFileHashAlgorithm, Span, Symbol, edition::Edition, kw, sym};
use rustc_structures::CrateType;
use rustc_target::callconv::{ArgAbi, ArgExtension, PassMode};
use std::{
    any::Any,
    collections::{BTreeSet, HashMap},
    fs,
    io,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{Arc, OnceLock},
};

const HOST: &str = "x86_64-unknown-linux-gnu";

struct Scratch(PathBuf);

impl Scratch {
    fn create(output: &Path) -> io::Result<Self> {
        fs::create_dir_all(output)?;
        let root = output.join(format!(".lustr-freestanding-{}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&root)?;
        Ok(Self(root))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Artifact {
    Freestanding,
    CargoUnit,
}

struct CompletedItems {
    object: Option<PathBuf>,
    functions: Vec<(String, String)>,
    pending_native: Option<host_retained_link::NativeObjectPending>,
}

struct Composition<'a> {
    artifact: Artifact,
    args: &'a [String],
    native_codegen: bool,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    object: Arc<OnceLock<PathBuf>>,
    retained_link: Arc<OnceLock<host_retained_link::Cache>>,
    pending_link: Arc<OnceLock<host_retained_link::Pending>>,
    pending_native: Option<host_retained_link::NativeObjectPending>,
    reused_link: bool,
    bindings: Vec<(Symbol, String, IntegerSignature)>,
    scratch: Option<Scratch>,
}

fn type_name(ty: &Ty) -> Option<Symbol> {
    let TyKind::Path(None, path) = &ty.kind else { return None };
    let [segment] = path.segments.as_slice() else { return None };
    segment.args.is_none().then_some(segment.ident.name)
}

fn primitive_integer(name: Symbol) -> Option<PrimTy> {
    match PrimTy::from_name(name)? {
        PrimTy::Int(ty::IntTy::I128) | PrimTy::Uint(ty::UintTy::U128) => None,
        primitive @ (PrimTy::Int(_) | PrimTy::Uint(_)) => Some(primitive),
        _ => None,
    }
}

struct IntegerSignature {
    args: Vec<PrimTy>,
    result: PrimTy,
    declarations: BTreeSet<usize>,
}

impl IntegerSignature {
    fn types(&self) -> impl Iterator<Item = PrimTy> + '_ {
        self.args.iter().copied().chain(std::iter::once(self.result))
    }
}

fn integer_bits(ty: PrimTy) -> u64 {
    match ty {
        PrimTy::Int(ty) => ty.bit_width().unwrap_or(64),
        PrimTy::Uint(ty) => ty.bit_width().unwrap_or(64),
        _ => unreachable!("a native integer signature contains only integers"),
    }
}

fn written_alias(item: &Item) -> Option<(Symbol, PrimTy)> {
    let ItemKind::TyAlias(alias) = &item.kind else { return None };
    if !item.attrs.is_empty() || !matches!(alias.defaultness, Defaultness::Implicit)
        || !alias.generics.params.is_empty() || !alias.generics.where_clause.is_empty()
        || !alias.after_where_clause.is_empty() || !alias.bounds.is_empty()
        || PrimTy::from_name(alias.ident.name).is_some()
    {
        return None;
    }
    Some((alias.ident.name, primitive_integer(type_name(alias.ty.as_deref()?)?)?))
}

fn imports_name(tree: &UseTree, parent: Option<Symbol>, expected: Symbol) -> bool {
    match &tree.kind {
        UseTreeKind::Glob(_) => true,
        UseTreeKind::Simple(_) => match tree.ident().name {
            kw::SelfLower => parent == Some(expected),
            name => name == expected,
        },
        UseTreeKind::Nested { items, .. } => {
            let parent = tree.prefix.segments.last().map(|segment| segment.ident.name).or(parent);
            items.iter().any(|item| imports_name(&item.inner, parent, expected))
        }
    }
}

fn may_expand(attr: &rustc_ast::Attribute, prelude_test: bool) -> bool {
    if prelude_test && attr.is_word() && attr.has_name(sym::test) {
        return false;
    }
    !attr.is_doc_comment() && attr.name().is_none_or(|name| {
        !rustc_feature::is_builtin_attr_name(name)
            || matches!(name, sym::cfg_attr | sym::derive | sym::prelude_import)
    })
}

fn may_shadow_primitive(item: &Item, primitive: Symbol, prelude_test: bool) -> bool {
    if item.attrs.iter().any(|attr| may_expand(attr, prelude_test)) {
        return true;
    }
    match &item.kind {
        ItemKind::Use(tree) => imports_name(tree, None, primitive),
        ItemKind::MacCall(_) | ItemKind::DelegationMac(_) => true,
        ItemKind::ForeignMod(module) => module.items.iter().any(|item| {
            matches!(item.kind, ForeignItemKind::MacCall(_))
                || item.kind.ident().is_some_and(|ident| ident.name == primitive)
                || item.attrs.iter().any(|attr| may_expand(attr, false))
        }),
        kind => kind.ident().is_some_and(|ident| ident.name == primitive),
    }
}

/// Select only a closed expression in the currently shared value domain.
/// A path must be the parameter. Operations that need names, macro
/// expansion, implicit dereferencing or a runtime remain with pinned Rust.
fn closed_expression(expression: &Expr, parameters: &[Ident]) -> Option<bool> {
    if !expression.attrs.is_empty() {
        return None;
    }
    match &expression.kind {
        ExprKind::Path(None, path)
            if matches!(path.segments.as_slice(), [segment]
                if segment.args.is_none() && parameters.iter().any(|ident| ident.name == segment.ident.name)) => Some(true),
        ExprKind::Lit(literal) if matches!(literal.kind, LitKind::Integer | LitKind::Bool) => Some(false),
        ExprKind::Paren(expression) => closed_expression(expression, parameters),
        ExprKind::Binary(op, lhs, rhs)
            if matches!(op.node, BinOpKind::BitXor | BinOpKind::BitAnd | BinOpKind::BitOr) =>
        {
            Some(closed_expression(lhs, parameters)? | closed_expression(rhs, parameters)?)
        }
        _ => None,
    }
}

fn native_function(item: &Item, aliases: &HashMap<Symbol, Option<(usize, PrimTy)>>) -> Option<IntegerSignature> {
    let ItemKind::Fn(function) = &item.kind else { return None };
    if !item.attrs.is_empty() || function.sig.header.has_qualifiers()
        || !matches!(item.vis.kind, VisibilityKind::Public | VisibilityKind::Inherited)
        || !function.generics.params.is_empty() || !function.generics.where_clause.predicates.is_empty()
        || function.contract.is_some() || function.define_opaque.is_some() || function.eii_impl.is_some()
    {
        return None;
    }
    let FnRetTy::Ty(result) = &function.sig.decl.output else { return None };
    let mut declarations = BTreeSet::new();
    let mut resolved_type = |ty: &Ty| {
        let name = type_name(ty)?;
        if let Some(primitive) = primitive_integer(name) {
            return Some(primitive);
        }
        let (index, primitive) = aliases.get(&name).copied().flatten()?;
        declarations.insert(index);
        Some(primitive)
    };
    let mut parameters = Vec::with_capacity(function.sig.decl.inputs.len());
    let mut args = Vec::with_capacity(function.sig.decl.inputs.len());
    for parameter in &function.sig.decl.inputs {
        let PatKind::Ident(BindingMode::NONE, name, None) = parameter.pat.kind else { return None };
        if !parameter.attrs.is_empty() { return None; }
        parameters.push(name);
        args.push(resolved_type(&parameter.ty)?);
    }
    let result = resolved_type(result)?;
    let signature = IntegerSignature { args, result, declarations };
    let body = function.body.as_ref()?;
    let [statement] = body.stmts.as_slice() else { return None };
    let StmtKind::Expr(expression) = &statement.kind else { return None };
    (body.rules == BlockCheckMode::Default && closed_expression(expression, &parameters) == Some(true))
        .then_some(signature)
}

impl Composition<'_> {
    fn complete_items(
        &self, sess: &Session, input: &Path, items: &[&Item], directory: &Path, no_std: Option<Span>,
        codegen: crate::native_object::ItemCodegen,
    ) -> Result<CompletedItems, String> {
        let file = sess.source_map().lookup_source_file(items[0].span.lo());
        let source = file.src.as_ref().ok_or("missing original source snapshot")?;
        let verify_original = |text: &str| {
            text.len() == file.unnormalized_source_len as usize && file.src_hash.matches(text)
        };
        let original_matches = (!file.normalized_pos.is_empty())
            .then_some(&verify_original as &dyn Fn(&str) -> bool);
        let object = directory.join("native-items.o");
        let setting = |value| if value { "on" } else { "off" };
        let link = sess.opts.output_types.contains_key(&OutputType::Exe);
        let name = sess.opts.crate_name.as_deref().ok_or("missing original native crate name")?;
        let metadata = format!("{name}/{:?}/{:?}", sess.opts.crate_types, sess.opts.cg.metadata);
        let digest = SourceFileHash::new_in_memory(SourceFileHashAlgorithm::Sha256, &metadata).to_string();
        let metadata = digest.strip_prefix("sha256=").ok_or("unexpected native crate digest encoding")?;
        let extent = |span: Span| {
            sess.source_map().span_to_snippet(span)
                .map_err(|error| format!("cannot select the original item: {error:?}"))?;
            Ok::<_, String>(file.relative_position(span.lo()).0..file.relative_position(span.hi()).0)
        };
        use crate::native_object::{SelectedItem, SelectedKind};
        let selected = items.iter().map(|item| {
            let kind = match item.kind {
                ItemKind::Fn(_) => SelectedKind::Function,
                ItemKind::TyAlias(_) => SelectedKind::TypeAlias,
                _ => return Err("unsupported native item selection kind".into()),
            };
            Ok::<_, String>(SelectedItem { span: extent(item.span)?, kind })
        }).collect::<Result<Vec<_>, _>>()?;
        let policy_span = no_std.map(extent).transpose()?;
        let args = [
            "--crate-name".to_owned(), name.to_owned(), "--edition=2024".into(), "--crate-type=lib".into(),
            "--emit=obj".into(), "--target=x86_64-unknown-linux-gnu".into(), "-Cpanic=abort".into(),
            "-Copt-level=0".into(), "-Cdebuginfo=0".into(), format!("-Cmetadata={metadata}"),
            format!("-Coverflow-checks={}", setting(sess.overflow_checks())),
            format!("-Cdebug-assertions={}", setting(sess.opts.debug_assertions)),
            input.to_str().ok_or("native item source path must be UTF-8")?.to_owned(), "-o".into(),
            object.to_str().ok_or("native item output path must be UTF-8")?.to_owned(),
        ];
        let cache_directory = self.output.as_deref().and_then(host_retained_link::cargo_unit_dir);
        let mut pending_native = None;
        let functions = sess.time("lustr_native_items", || {
            crate::native_object::complete_items(
                &args, source, &selected, link, policy_span, codegen, original_matches,
                |output, text, options| host_retained_link::emit_native_object(
                    output, text, options, cache_directory, &mut pending_native,
                ),
            )
        })?;
        if object.is_file() != link {
            return Err("native compiler produced the wrong artifact for this request".into());
        }
        Ok(CompletedItems { object: link.then_some(object), functions, pending_native })
    }

}

// Preserve Rust's function-item identity. Rust lowers only the forwarding
// call and keeps its ordinary function values, coercions and symbol rules.
fn bind(sess: &Session, item: &mut Item, symbol: &str) -> Symbol {
    let span = item.span;
    let ItemKind::Fn(function) = &mut item.kind else { unreachable!() };
    let parameters: Vec<_> = function.sig.decl.inputs.iter().map(|parameter| {
        let PatKind::Ident(_, name, None) = parameter.pat.kind else { unreachable!() };
        name
    }).collect();
    let mut local = Symbol::intern(symbol);
    while parameters.iter().any(|parameter| local == parameter.name) {
        local = Symbol::intern(&format!("_{local}"));
    }
    let id = Ident::new(local, span);
    let private = Visibility { kind: VisibilityKind::Inherited, span };
    let mut declaration = function.clone();
    declaration.ident = id;
    declaration.body = None;
    declaration.sig.header.safety = Safety::Safe(span);
    let link_name = rustc_ast::attr::mk_attr_name_value_str(
        &sess.psess.attr_id_generator, AttrStyle::Outer, sym::link_name, Symbol::intern(symbol), span,
    );
    let foreign = Box::new(Item {
        attrs: [link_name].into_iter().collect(), id: rustc_ast::DUMMY_NODE_ID,
        span, vis: private.clone(), kind: ForeignItemKind::Fn(declaration), tokens: None,
    });
    let abi = Symbol::intern("Rust");
    let foreign = ItemKind::ForeignMod(ForeignMod {
        extern_span: span, safety: Safety::Unsafe(span),
        abi: Some(StrLit { symbol: abi, suffix: None, symbol_unescaped: abi, style: StrStyle::Cooked, span }),
        items: [foreign].into_iter().collect(),
    });
    let import = Box::new(Item {
        attrs: Default::default(), id: rustc_ast::DUMMY_NODE_ID,
        span, vis: private, kind: foreign, tokens: None,
    });
    let expr = |kind| Box::new(Expr {
        id: rustc_ast::DUMMY_NODE_ID, kind, span, attrs: Default::default(), tokens: None,
    });
    let path = |ident| expr(ExprKind::Path(None, rustc_ast::Path::from_ident(ident)));
    let call = expr(ExprKind::Call(path(id), parameters.into_iter().map(path).collect()));
    function.body.as_mut().unwrap().stmts = [
        Stmt { id: rustc_ast::DUMMY_NODE_ID, kind: StmtKind::Item(import), span },
        Stmt { id: rustc_ast::DUMMY_NODE_ID, kind: StmtKind::Expr(call), span },
    ].into_iter().collect();
    item.tokens = None;
    local
}

impl Callbacks for Composition<'_> {
    fn config(&mut self, config: &mut Config) {
        let early = EarlyDiagCtxt::new(ErrorOutputType::default());
        let host = config.opts.target_triple.tuple() == HOST;
        if matches!(self.artifact, Artifact::Freestanding)
            && (!host || config.opts.crate_types != [CrateType::Executable])
        {
            early.early_fatal("the parsed request does not match the selected host artifact");
        }
        let native_artifact = matches!(config.opts.crate_types.as_slice(),
            [] | [CrateType::Executable | CrateType::Rlib | CrateType::StaticLib]);
        if !host || !native_artifact || !self.native_codegen || config.opts.cg.embed_bitcode
            || !crate::embedding_environment_supported()
            || config.opts.test && matches!(self.artifact, Artifact::Freestanding)
            || config.opts.output_types.keys().any(|kind|
            !matches!(kind, OutputType::Exe | OutputType::Metadata | OutputType::DepInfo))
        {
            return;
        }
        let Input::File(input) = &config.input else { return };
        let output = match (&config.output_dir, &config.output_file) {
            (Some(output), _) => output.clone(),
            (None, Some(OutFileName::Real(file))) => file.parent()
                .filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_owned(),
            (None, None) => PathBuf::from("."),
            (None, Some(OutFileName::Stdout)) => return,
        };
        self.input = Some(input.clone());
        self.output = Some(output);
        let object = Arc::clone(&self.object);
        let retained_link = Arc::clone(&self.retained_link);
        let pending_link = Arc::clone(&self.pending_link);
        config.make_codegen_backend = Some(Box::new(move |_| Box::new(Backend {
            original: rustc_codegen_llvm::LlvmCodegenBackend::new(),
            object,
            retained_link,
            pending_link,
        })));
    }

    fn after_crate_root_parsing(&mut self, compiler: &Compiler, krate: &mut rustc_ast::Crate) -> Compilation {
        let sess = &compiler.sess;
        let no_std_attr = krate.attrs.iter().find(|attr| attr.is_word() && attr.has_name(sym::no_std));
        let no_std = no_std_attr.is_some();
        if matches!(self.artifact, Artifact::Freestanding)
            && (sess.panic_strategy().unwinds() || !no_std
                || !krate.attrs.iter().any(|attr| attr.is_word() && attr.has_name(sym::no_main)))
        {
            sess.dcx().fatal("the freestanding compiler requires no_std, no_main and panic=abort");
        }
        // Source-level lint directives and other crate policies can affect a
        // body independently of its value. Keep those bodies with Rust.
        let plain_policy = match self.artifact {
            Artifact::Freestanding => matches!(krate.attrs.as_slice(), [first, second]
                if first.has_name(sym::no_std) && second.has_name(sym::no_main)),
            Artifact::CargoUnit => krate.attrs.is_empty()
                || matches!(krate.attrs.as_slice(), [attr] if no_std && attr.has_name(sym::no_std)),
        };
        let rust_debug = match sess.opts.debuginfo {
            DebugInfo::None => false,
            DebugInfo::Full => true,
            _ => return Compilation::Continue,
        };
        if self.input.is_none() || !plain_policy
            || matches!(self.artifact, Artifact::Freestanding) && (sess.opts.optimize != OptLevel::No || rust_debug)
            || sess.opts.edition != Edition::Edition2024 || sess.opts.crate_name.is_none()
            || sess.opts.lint_opts.iter().any(|(lint, _)| lint != "unused_crate_dependencies")
        {
            return Compilation::Continue;
        }
        let optimization = match sess.opts.optimize {
            OptLevel::No => crate::protocol::OptLevel::None,
            OptLevel::Less => crate::protocol::OptLevel::Less,
            OptLevel::More => crate::protocol::OptLevel::More,
            OptLevel::Aggressive => crate::protocol::OptLevel::Aggressive,
            OptLevel::Size => crate::protocol::OptLevel::Size,
            OptLevel::SizeMin => crate::protocol::OptLevel::MinSize,
        };
        // The native body uses the primitive ABI. Keep the original Rust
        // body if expansion or resolution could rebind the unqualified type.
        // The pinned prelude's test attribute removes the function outside
        // test mode and adds its harness descriptor in test mode.
        // Its name can also denote an imported attribute macro.
        let prelude_test = [sym::std, sym::core].into_iter().all(|name| sess.opts.externs.get(name.as_str()).is_none())
            && !krate.items.iter().any(|item| {
                item.attrs.iter().any(|attr| attr.has_name(sym::macro_use))
                    || match &item.kind {
                        ItemKind::Use(tree) => imports_name(tree, None, sym::test),
                        kind => kind.ident().is_some_and(|ident| matches!(ident.name, sym::test | sym::std | sym::core)),
                    }
            });
        let mut primitive_bindings = HashMap::new();
        let mut aliases = HashMap::new();
        if matches!(self.artifact, Artifact::CargoUnit) {
            for (index, item) in krate.items.iter().enumerate() {
                if let Some((name, primitive)) = written_alias(item) {
                    aliases.entry(name).and_modify(|prior| *prior = None).or_insert(Some((index, primitive)));
                }
            }
        }
        let selected: Vec<_> = krate.items.iter().enumerate().filter_map(|(index, item)| {
            let signature = native_function(item, &aliases)?;
            if matches!(self.artifact, Artifact::Freestanding) {
                let word = PrimTy::Uint(ty::UintTy::U64);
                if signature.args != [word] || signature.result != word { return None; }
            }
            let resolved_primitives = signature.types().all(|ty| {
                let name = ty.name();
                *primitive_bindings.entry(name).or_insert_with(|| {
                    sess.opts.externs.get(name.as_str()).is_none_or(|external| !external.add_prelude)
                        && !krate.items.iter().any(|item| may_shadow_primitive(item, name, prelude_test))
                })
            });
            resolved_primitives.then_some((index, item.as_ref(), signature))
        }).collect();
        if let Some((_, first, _)) = selected.first() {
            let codegen = crate::native_object::ItemCodegen {
                optimization, rust_debug, unwind_tables: sess.must_emit_unwind_tables(),
                // Like native Rust libraries, narrow integer objects use the
                // export path that leaves argument extension to the Rust ABI.
                narrow_rust_abi: selected.iter().any(|(_, _, signature)| signature.types().any(|ty| integer_bits(ty) < 32)),
            };
            let directory = Scratch::create(self.output.as_deref().unwrap())
                .unwrap_or_else(|error| sess.dcx().fatal(format!("cannot create native item work directory: {error}")));
            let input = self.input.as_deref().unwrap();
            let context: BTreeSet<_> = selected.iter().flat_map(|(index, _, signature)|
                std::iter::once(*index).chain(signature.declarations.iter().copied())).collect();
            let items: Vec<_> = context.into_iter().map(|index| krate.items[index].as_ref()).collect();
            let completed = self.complete_items(sess, input, &items, &directory.0, no_std_attr.map(|attr| attr.span), codegen)
                .unwrap_or_else(|error| sess.dcx().span_fatal(first.span, error));
            if completed.functions.len() != selected.len() {
                sess.dcx().span_fatal(first.span, "native object does not cover every selected definition");
            }
            let mut symbols = Vec::with_capacity(selected.len());
            for ((_, item, _), (name, symbol)) in selected.iter().zip(completed.functions) {
                let ItemKind::Fn(function) = &item.kind else { unreachable!() };
                if name != function.ident.name.as_str()
                    || !symbol.starts_with("f_")
                    || !symbol.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    || symbols.contains(&symbol)
                {
                    sess.dcx().span_fatal(item.span, "invalid native object symbol binding");
                }
                symbols.push(symbol);
            }
            self.pending_native = completed.pending_native;
            if let Some(object) = completed.object {
                self.object.set(object).expect("one native object per invocation");
                let file = sess.source_map().lookup_source_file(first.span.lo());
                let bodies = selected.iter().map(|(_, item, _)| {
                    let ItemKind::Fn(function) = &item.kind else { unreachable!() };
                    let span = function.body.as_ref().unwrap().span;
                    file.relative_position(span.lo()).0..file.relative_position(span.hi()).0
                }).collect::<Vec<_>>();
                if let Some(cache) = host_retained_link::Cache::open(
                    sess, self.artifact, self.output.as_deref().unwrap(), self.args, &file, &bodies, &symbols,
                ) {
                    let outputs = rustc_interface::util::build_output_filenames(&krate.attrs, sess);
                    if let OutFileName::Real(deps) = outputs.path(OutputType::DepInfo)
                        && let Some(saved) = cache.load(sess, &outputs, &directory.0)
                    {
                        if sess.opts.json_artifact_notifications {
                            sess.dcx().emit_artifact_notification(&deps, "dep-info");
                        }
                        compiler.codegen_backend.link(sess, saved.modules, saved.info, saved.metadata, &outputs);
                        self.reused_link = true;
                        self.scratch = Some(directory);
                        return Compilation::Stop;
                    }
                    cache.track_diagnostics();
                    let _ = self.retained_link.set(cache);
                }
            }
            let signatures: Vec<_> = selected.into_iter().map(|(index, _, signature)| (index, signature)).collect();
            for ((index, signature), symbol) in signatures.into_iter().zip(symbols) {
                let name = bind(sess, &mut krate.items[index], &symbol);
                self.bindings.push((name, symbol, signature));
            }
            self.scratch = Some(directory);
        }
        Compilation::Continue
    }

    fn after_expansion(&mut self, _: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        for (name, symbol, signature) in &self.bindings {
            let foreign = tcx.hir_crate_items(()).foreign_items().find_map(|id| {
                let item = tcx.hir_foreign_item(id);
                if item.ident.name != *name {
                    return None;
                }
                let instance = Instance::mono(tcx, item.owner_id.def_id.to_def_id());
                (tcx.symbol_name(instance).name == *symbol).then_some(instance)
            }).unwrap_or_else(|| tcx.dcx().fatal("the native declaration did not resolve to its object symbol"));
            let abi = tcx.fn_abi_of_instance(TypingEnv::fully_monomorphized().as_query_input((foreign, ty::List::empty())))
                .unwrap_or_else(|error| tcx.dcx().fatal(format!("cannot read the original call ABI: {error:?}")));
            let scalar = |arg: &ArgAbi<'_, ty::Ty<'_>>, expected: PrimTy| {
                let actual = match *arg.layout.ty.kind() {
                    ty::Int(int) => PrimTy::Int(int),
                    ty::Uint(uint) => PrimTy::Uint(uint),
                    _ => return false,
                };
                let width = integer_bits(expected);
                actual == expected && arg.layout.size.bits() == width && arg.layout.align.bytes() == width / 8
                    && matches!(arg.mode, PassMode::Direct(attrs) if attrs.arg_ext == ArgExtension::None)
            };
            if abi.conv != CanonAbi::Rust
                || abi.args.len() != signature.args.len() || !scalar(&abi.ret, signature.result)
                || !abi.args.iter().zip(&signature.args).all(|(arg, &expected)| scalar(arg, expected))
            {
                tcx.dcx().fatal("the original call ABI differs from the native object");
            }
        }
        Compilation::Continue
    }
}

struct Backend {
    original: Box<dyn CodegenBackend>,
    object: Arc<OnceLock<PathBuf>>,
    retained_link: Arc<OnceLock<host_retained_link::Cache>>,
    pending_link: Arc<OnceLock<host_retained_link::Pending>>,
}

impl CodegenBackend for Backend {
    fn name(&self) -> &'static str { self.original.name() }
    fn init(&mut self, sess: &EarlySession) -> CodegenBackendInit { self.original.init(sess) }
    fn target_config(&self, sess: &EarlySession) -> TargetConfig { self.original.target_config(sess) }
    fn provide(&self, providers: &mut Providers) { self.original.provide(providers); }
    fn target_cpu(&self, sess: &Session) -> String { self.original.target_cpu(sess) }
    fn codegen_crate<'tcx>(&self, tcx: TyCtxt<'tcx>) -> Box<dyn Any> { self.original.codegen_crate(tcx) }

    fn join_codegen(
        &self, ongoing: Box<dyn Any>, sess: &Session, incr: Option<&IncrCompSession>,
        outputs: &OutputFilenames, crate_info: &CrateInfo,
    ) -> (CompiledModules, WorkProductMap) {
        self.original.join_codegen(ongoing, sess, incr, outputs, crate_info)
    }

    fn link(
        &self, sess: &Session, mut modules: CompiledModules, info: CrateInfo,
        metadata: EncodedMetadata, outputs: &OutputFilenames,
    ) {
        let pending = self.retained_link.get().and_then(|cache|
            cache.capture(sess, &modules, &info, &metadata, outputs));
        if let Some(object) = self.object.get() {
            modules.modules.push(CompiledModule {
                name: "lustr-native-items".into(), kind: ModuleKind::Regular, object: Some(object.clone()),
                global_asm_object: None, dwarf_object: None, bytecode: None, assembly: None, llvm_ir: None,
            });
        }
        self.original.link(sess, modules, info, metadata, outputs);
        if let Some(pending) = pending {
            let _ = self.pending_link.set(pending);
        }
    }
}

pub(crate) fn run(artifact: Artifact, program: &str, args: &[String]) -> ExitCode {
    rustc_driver::catch_with_exit_code(|| {
        let early = EarlyDiagCtxt::new(ErrorOutputType::default());
        let executable = std::env::current_exe().unwrap_or_else(|error| early.early_fatal(error.to_string()));
        let bin = executable.parent().unwrap();
        let Some(sysroot) = bin.parent().and_then(Path::to_str) else {
            early.early_fatal("cannot locate the pinned Rust SDK for freestanding linking");
        };
        let mut driver = vec![program.to_owned()];
        driver.extend_from_slice(args);
        if !args.iter().any(|arg| arg == "--sysroot" || arg.starts_with("--sysroot=")) {
            driver.extend(["--sysroot".to_owned(), sysroot.to_owned()]);
        }
        let mut callbacks = Composition {
            artifact, args, native_codegen: represents_native_codegen(args),
            input: None, output: None,
            object: Arc::default(), retained_link: Arc::default(), pending_link: Arc::default(),
            pending_native: None,
            reused_link: false, bindings: Vec::new(), scratch: None,
        };
        rustc_driver::compiler_entrypoint(&driver, &mut callbacks);
        if let Some(pending) = callbacks.pending_native.take() {
            pending.publish();
        }
        // Only retain successful Rust analysis and code generation. The
        // original linker runs on every request, including its diagnostics.
        if let Some(pending) = Arc::get_mut(&mut callbacks.pending_link).and_then(OnceLock::take) {
            pending.publish();
        }
        if callbacks.reused_link {
            host_retained_link::trace("retained-rust-link");
        }
    })
}

fn represents_native_codegen(args: &[String]) -> bool {
    let mut rest = args.iter().map(String::as_str);
    while let Some(arg) = rest.next() {
        if let Some(option) = arg.strip_prefix("-Z") {
            let option = if option.is_empty() { rest.next().unwrap_or_default() } else { option };
            // The original Rust driver owns archive metadata and dep-info.
            // These options leave the native item's object unchanged.
            if !matches!(option.split_once('=').map(|(key, _)| key), Some("embed-metadata" | "checksum-hash-algorithm")) {
                return false;
            }
            continue;
        }
        let codegen = if matches!(arg, "-C" | "--codegen") {
            rest.next()
        } else {
            arg.strip_prefix("-C").or_else(|| arg.strip_prefix("--codegen="))
        };
        if let Some(option) = codegen {
            let key = option.split_once('=').map_or(option, |(key, _)| key);
            if !matches!(key, "embed-bitcode" | "embed-metadata" | "debuginfo" | "metadata" | "extra-filename"
                | "incremental" | "panic" | "strip" | "opt-level" | "overflow-checks"
                | "debug-assertions" | "link-arg" | "relocation-model")
            {
                return false;
            }
        }
    }
    true
}
