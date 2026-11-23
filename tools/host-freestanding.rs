//! Compose Lust objects with the pinned Rust compiler for a freestanding unit.
//! The original Rust driver validates and emits the rest of the original crate.
use rustc_ast::{
    AttrStyle, BinOpKind, BindingMode, BlockCheckMode, Expr, ExprKind, FnRetTy,
    ForeignItemKind, ForeignMod, Item, ItemKind, PatKind, Safety, StmtKind, StrLit,
    StrStyle, Ty, TyKind, VisibilityKind, token::LitKind,
};
use rustc_codegen_ssa::{
    CompiledModule, CompiledModules, CrateInfo, ModuleKind, TargetConfig, traits::CodegenBackend,
};
use rustc_driver::{Callbacks, Compilation};
use rustc_hir::{self as hir, def::Res, def_id::DefId, intravisit::{self, Visitor}};
use rustc_interface::interface::{Compiler, Config};
use rustc_metadata::EncodedMetadata;
use rustc_middle::{dep_graph::WorkProductMap, ty::{self, Instance, TyCtxt, TypingEnv}, util::Providers};
use rustc_session::{
    CodegenBackendInit, EarlyDiagCtxt, EarlySession, IncrCompSession, Session,
    config::{DebugInfo, ErrorOutputType, Input, OptLevel, OutputFilenames, OutputType},
};
use rustc_span::{Symbol, edition::Edition, sym};
use rustc_structures::CrateType;
use rustc_target::callconv::{ArgAbi, PassMode};
use std::{
    any::Any,
    fs,
    io::{self, Write},
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    sync::{Arc, Mutex},
};

const HOST: &str = "x86_64-unknown-linux-gnu";

struct Scratch(PathBuf);

impl Scratch {
    fn create(output: &Path) -> io::Result<Self> {
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

struct NativeItem {
    name: Symbol,
    symbol: String,
    object: PathBuf,
}

type NativeItems = Arc<Mutex<Vec<NativeItem>>>;

struct Freestanding {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    producer: PathBuf,
    items: NativeItems,
    scratch: Option<Scratch>,
}

fn builtin_u64(ty: &Ty) -> bool {
    matches!(&ty.kind, TyKind::Path(None, path)
        if matches!(path.segments.as_slice(), [segment]
            if segment.ident.name == sym::u64 && segment.args.is_none()))
}

/// Select only a closed expression in the currently shared value domain.
/// A path must be the parameter. Operations that need names, macro
/// expansion, implicit dereferencing or a runtime remain with pinned Rust.
fn closed_expression(expression: &Expr, parameter: Symbol) -> Option<bool> {
    if !expression.attrs.is_empty() {
        return None;
    }
    match &expression.kind {
        ExprKind::Path(None, path)
            if matches!(path.segments.as_slice(), [segment]
                if segment.ident.name == parameter && segment.args.is_none()) => Some(true),
        ExprKind::Lit(literal) if matches!(literal.kind, LitKind::Integer | LitKind::Bool) => Some(false),
        ExprKind::Paren(expression) => closed_expression(expression, parameter),
        ExprKind::Binary(op, lhs, rhs)
            if matches!(op.node, BinOpKind::BitXor | BinOpKind::BitAnd | BinOpKind::BitOr) =>
        {
            Some(closed_expression(lhs, parameter)? | closed_expression(rhs, parameter)?)
        }
        _ => None,
    }
}

fn native_function(item: &Item) -> Option<&rustc_ast::Fn> {
    let ItemKind::Fn(function) = &item.kind else { return None };
    if !item.attrs.is_empty() || function.sig.header.has_qualifiers()
        || !matches!(item.vis.kind, VisibilityKind::Public | VisibilityKind::Inherited)
        || !function.generics.params.is_empty() || !function.generics.where_clause.predicates.is_empty()
        || function.contract.is_some() || function.define_opaque.is_some() || function.eii_impl.is_some()
    {
        return None;
    }
    let [parameter] = function.sig.decl.inputs.as_slice() else { return None };
    let FnRetTy::Ty(result) = &function.sig.decl.output else { return None };
    let PatKind::Ident(BindingMode::NONE, name, None) = parameter.pat.kind else { return None };
    if !parameter.attrs.is_empty() || !builtin_u64(&parameter.ty) || !builtin_u64(result) {
        return None;
    }
    let body = function.body.as_ref()?;
    let [statement] = body.stmts.as_slice() else { return None };
    let StmtKind::Expr(expression) = &statement.kind else { return None };
    (body.rules == BlockCheckMode::Default && closed_expression(expression, name.name) == Some(true))
        .then_some(function)
}

impl Freestanding {
    fn emit(&self, sess: &Session, input: &Path, item: &Item, directory: &Path) -> Result<NativeItem, String> {
        let ItemKind::Fn(function) = &item.kind else { unreachable!() };
        let file = sess.source_map().lookup_source_file(item.span.lo());
        let source = file.src.as_ref().ok_or("missing original source snapshot")?;
        sess.source_map().span_to_snippet(item.span)
            .map_err(|error| format!("cannot select the original item: {error:?}"))?;
        let start = file.relative_position(item.span.lo()).0;
        let end = file.relative_position(item.span.hi()).0;
        let object = directory.join(format!("{start}.o"));
        let setting = |value| if value { "on" } else { "off" };
        let mut child = Command::new(&self.producer)
            .args([start.to_string(), end.to_string()])
            .args(["--crate-name", "lustr_freestanding", "--edition=2024", "--crate-type=lib", "--emit=obj"])
            .args(["--target=x86_64-unknown-linux-gnu", "-Cpanic=abort", "-Copt-level=0", "-Cdebuginfo=0"])
            .args([
                format!("-Cmetadata=item_{start}"),
                format!("-Coverflow-checks={}", setting(sess.overflow_checks())),
                format!("-Cdebug-assertions={}", setting(sess.opts.debug_assertions)),
            ])
            .arg(input).arg("-o").arg(&object)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|error| format!("cannot start native item compiler: {error}"))?;
        let sent = child.stdin.take().unwrap().write_all(source.as_bytes());
        let result = child.wait_with_output().map_err(|error| format!("cannot read native item result: {error}"))?;
        if !result.status.success() {
            return Err(String::from_utf8_lossy(&result.stderr).into_owned());
        }
        sent.map_err(|error| format!("cannot send original source to native compiler: {error}"))?;
        let symbol = std::str::from_utf8(&result.stdout).map_err(|error| error.to_string())?
            .strip_suffix('\n').filter(|symbol| {
                symbol.starts_with("f_") && symbol.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            }).ok_or("invalid native object symbol")?;
        if !object.is_file() {
            return Err("native compiler did not publish an object".into());
        }
        Ok(NativeItem { name: function.ident.name, symbol: symbol.to_owned(), object })
    }
}

// Use Rust's own foreign-item lowering, resolution and calling convention.
// The signature and visibility come from the original function.
fn bind(sess: &Session, item: &mut Item, native: &NativeItem) {
    let span = item.span;
    let ItemKind::Fn(function) = &item.kind else { unreachable!() };
    let mut declaration = function.clone();
    declaration.body = None;
    declaration.sig.header.safety = Safety::Safe(span);
    let link_name = rustc_ast::attr::mk_attr_name_value_str(
        &sess.psess.attr_id_generator, AttrStyle::Outer, sym::link_name, Symbol::intern(&native.symbol), span,
    );
    let foreign = Box::new(Item {
        attrs: [link_name].into_iter().collect(), id: rustc_ast::DUMMY_NODE_ID,
        span, vis: item.vis.clone(), kind: ForeignItemKind::Fn(declaration), tokens: None,
    });
    let abi = Symbol::intern("Rust");
    item.kind = ItemKind::ForeignMod(ForeignMod {
        extern_span: span, safety: Safety::Unsafe(span),
        abi: Some(StrLit { symbol: abi, suffix: None, symbol_unescaped: abi, style: StrStyle::Cooked, span }),
        items: [foreign].into_iter().collect(),
    });
    item.vis.kind = VisibilityKind::Inherited;
    item.tokens = None;
}

impl Callbacks for Freestanding {
    fn config(&mut self, config: &mut Config) {
        let early = EarlyDiagCtxt::new(ErrorOutputType::default());
        if config.opts.target_triple.tuple() != HOST
            || config.opts.crate_types != [CrateType::Executable]
            || config.opts.test || config.opts.edition != Edition::Edition2024
            || !(config.opts.output_types.contains_key(&OutputType::Exe)
                || config.opts.output_types.contains_key(&OutputType::Metadata))
            || !config.opts.output_types.contains_key(&OutputType::DepInfo)
            || config.opts.output_types.len() != 2
        {
            early.early_fatal("expected a host freestanding Cargo executable request");
        }
        let Input::File(input) = &config.input else {
            early.early_fatal("expected an original source file");
        };
        let Some(output) = &config.output_dir else {
            early.early_fatal("expected a Cargo output directory");
        };
        self.input = Some(input.clone());
        self.output = Some(output.clone());
        let items = Arc::clone(&self.items);
        config.make_codegen_backend = Some(Box::new(move |_| Box::new(Backend {
            original: rustc_codegen_llvm::LlvmCodegenBackend::new(),
            items,
        })));
    }

    fn after_crate_root_parsing(&mut self, compiler: &Compiler, krate: &mut rustc_ast::Crate) -> Compilation {
        let sess = &compiler.sess;
        if sess.panic_strategy().unwinds()
            || !krate.attrs.iter().any(|attr| attr.is_word() && attr.has_name(sym::no_std))
            || !krate.attrs.iter().any(|attr| attr.is_word() && attr.has_name(sym::no_main))
        {
            sess.dcx().fatal("the freestanding compiler requires no_std, no_main and panic=abort");
        }
        // Source-level lint directives and other crate policies can affect a
        // body independently of its value. Keep those bodies with Rust.
        if sess.opts.optimize != OptLevel::No || sess.opts.debuginfo != DebugInfo::None
            || !matches!(krate.attrs.as_slice(), [first, second] if first.has_name(sym::no_std) && second.has_name(sym::no_main))
            || sess.opts.lint_opts.iter().any(|(lint, _)| lint != "unused_crate_dependencies")
        {
            return Compilation::Continue;
        }
        for item in &mut krate.items {
            if native_function(item).is_none() {
                continue;
            }
            let source = sess.source_map().lookup_source_file(item.span.lo());
            if !source.normalized_pos.is_empty() {
                continue;
            }
            if self.scratch.is_none() {
                self.scratch = Some(Scratch::create(self.output.as_deref().unwrap())
                    .unwrap_or_else(|error| sess.dcx().fatal(format!("cannot create native item work directory: {error}"))));
            }
            let input = self.input.as_deref().unwrap();
            let native = self.emit(sess, input, item, &self.scratch.as_ref().unwrap().0)
                .unwrap_or_else(|error| sess.dcx().span_fatal(item.span, error));
            bind(sess, item, &native);
            self.items.lock().unwrap().push(native);
        }
        Compilation::Continue
    }

    fn after_expansion(&mut self, _: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        let items = self.items.lock().unwrap();
        if items.is_empty() {
            return Compilation::Continue;
        }
        let mut bindings = Vec::with_capacity(items.len());
        for native in &*items {
            let foreign = tcx.hir_crate_items(()).foreign_items().find_map(|id| {
                let item = tcx.hir_foreign_item(id);
                if item.ident.name != native.name {
                    return None;
                }
                let instance = Instance::mono(tcx, item.owner_id.def_id.to_def_id());
                (tcx.symbol_name(instance).name == native.symbol).then_some(instance)
            }).unwrap_or_else(|| tcx.dcx().fatal("the native declaration did not resolve to its object symbol"));
            let abi = tcx.fn_abi_of_instance(TypingEnv::fully_monomorphized().as_query_input((foreign, ty::List::empty())))
                .unwrap_or_else(|error| tcx.dcx().fatal(format!("cannot read the original call ABI: {error:?}")));
            let scalar = |arg: &ArgAbi<'_, ty::Ty<'_>>| {
                matches!(arg.layout.ty.kind(), ty::Uint(ty::UintTy::U64))
                    && arg.layout.size.bits() == 64 && arg.layout.align.bytes() == 8
                    && matches!(arg.mode, PassMode::Direct(_))
            };
            if abi.can_unwind || abi.args.len() != 1 || !scalar(&abi.args[0]) || !scalar(&abi.ret) {
                tcx.dcx().fatal("the original call ABI differs from the native object");
            }
            bindings.push(foreign.def_id());
        }
        let mut calls = DirectCalls { tcx, bindings: &bindings };
        for owner in tcx.hir_body_owners() {
            calls.visit_body(tcx.hir_body_owned_by(owner));
        }
        Compilation::Continue
    }
}

/// A foreign declaration may have a different function-item identity. Use
/// this bridge only for direct calls until native function values are shared.
struct DirectCalls<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    bindings: &'a [DefId],
}

impl<'tcx> Visitor<'tcx> for DirectCalls<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if let hir::ExprKind::Path(hir::QPath::Resolved(_, path)) = expr.kind
            && let Res::Def(_, definition) = path.res
            && self.bindings.contains(&definition)
            && !matches!(self.tcx.hir_node(self.tcx.parent_hir_id(expr.hir_id)),
                hir::Node::Expr(parent) if matches!(parent.kind, hir::ExprKind::Call(callee, _)
                    if callee.hir_id == expr.hir_id))
        {
            self.tcx.dcx().span_fatal(expr.span, "native function values are not yet supported by this link interface");
        }
        intravisit::walk_expr(self, expr);
    }
}

struct Backend {
    original: Box<dyn CodegenBackend>,
    items: NativeItems,
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
        modules.modules.extend(self.items.lock().unwrap().iter().map(|native| CompiledModule {
            name: native.symbol.clone(), kind: ModuleKind::Regular, object: Some(native.object.clone()),
            global_asm_object: None, dwarf_object: None, bytecode: None, assembly: None, llvm_ir: None,
        }));
        self.original.link(sess, modules, info, metadata, outputs);
    }
}

pub(crate) fn run(program: &str, args: &[String]) -> ExitCode {
    rustc_driver::catch_with_exit_code(|| {
        let early = EarlyDiagCtxt::new(ErrorOutputType::default());
        let executable = std::env::current_exe().unwrap_or_else(|error| early.early_fatal(error.to_string()));
        let bin = executable.parent().unwrap();
        let Some(sysroot) = bin.parent().and_then(Path::to_str) else {
            early.early_fatal("cannot locate the pinned Rust SDK for freestanding linking");
        };
        let mut driver = vec![program.to_owned()];
        driver.extend_from_slice(args);
        driver.extend(["--sysroot".to_owned(), sysroot.to_owned()]);
        let mut callbacks = Freestanding {
            input: None, output: None, producer: bin.join("native-freestanding-item"),
            items: Arc::default(), scratch: None,
        };
        rustc_driver::compiler_entrypoint(&driver, &mut callbacks);
    })
}
