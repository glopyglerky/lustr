//! Compose Lust objects with the pinned Rust compiler for a freestanding unit.
//! The original Rust driver validates and emits the rest of the original crate.
use rustc_ast::{
    AttrStyle, BinOpKind, BindingMode, BlockCheckMode, Expr, ExprKind, FnRetTy,
    ForeignItemKind, ForeignMod, Item, ItemKind, PatKind, Safety, Stmt, StmtKind, StrLit,
    StrStyle, Ty, TyKind, Visibility, VisibilityKind, token::LitKind,
};
use rustc_codegen_ssa::{
    CompiledModule, CompiledModules, CrateInfo, ModuleKind, TargetConfig, traits::CodegenBackend,
};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface::{Compiler, Config};
use rustc_metadata::EncodedMetadata;
use rustc_middle::{dep_graph::WorkProductMap, ty::{self, Instance, TyCtxt, TypingEnv}, util::Providers};
use rustc_session::{
    CodegenBackendInit, EarlyDiagCtxt, EarlySession, IncrCompSession, Session,
    config::{DebugInfo, ErrorOutputType, Input, OptLevel, OutputFilenames, OutputType},
};
use rustc_span::{Ident, Symbol, edition::Edition, sym};
use rustc_structures::CrateType;
use rustc_target::callconv::{ArgAbi, PassMode};
use std::{
    any::Any,
    fs,
    io::{self, Write},
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    sync::{Arc, OnceLock},
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

struct Freestanding {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    producer: PathBuf,
    object: Arc<OnceLock<PathBuf>>,
    bindings: Vec<(Symbol, String)>,
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
    fn complete_items(
        &self, sess: &Session, input: &Path, items: &[&Item], directory: &Path,
    ) -> Result<(Option<PathBuf>, Vec<String>), String> {
        let file = sess.source_map().lookup_source_file(items[0].span.lo());
        let source = file.src.as_ref().ok_or("missing original source snapshot")?;
        let object = directory.join("native-items.o");
        let setting = |value| if value { "on" } else { "off" };
        let link = sess.opts.output_types.contains_key(&OutputType::Exe);
        let mut command = Command::new(&self.producer);
        command.arg(if link { "link" } else { "check" });
        for item in items {
            sess.source_map().span_to_snippet(item.span)
                .map_err(|error| format!("cannot select the original item: {error:?}"))?;
            command.args([
                file.relative_position(item.span.lo()).0.to_string(),
                file.relative_position(item.span.hi()).0.to_string(),
            ]);
        }
        let mut child = command
            .args(["--", "--crate-name", "lustr_freestanding", "--edition=2024", "--crate-type=lib", "--emit=obj"])
            .args(["--target=x86_64-unknown-linux-gnu", "-Cpanic=abort", "-Copt-level=0", "-Cdebuginfo=0"])
            .args([
                "-Cmetadata=native_items".to_owned(),
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
        let report = std::str::from_utf8(&result.stdout).map_err(|error| error.to_string())?;
        let mut report = report.lines();
        let mut symbols = Vec::with_capacity(items.len());
        for item in items {
            let ItemKind::Fn(function) = &item.kind else { unreachable!() };
            let (name, symbol) = report.next().and_then(|line| line.split_once(' '))
                .ok_or("missing native object symbol")?;
            if name != function.ident.name.as_str()
                || !symbol.starts_with("f_")
                || !symbol.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || symbols.iter().any(|previous| previous == symbol)
            {
                return Err("invalid native object symbol binding".into());
            }
            symbols.push(symbol.to_owned());
        }
        if report.next().is_some() {
            return Err("unexpected additional native symbol binding".into());
        }
        if object.is_file() != link {
            return Err("native compiler produced the wrong artifact for this request".into());
        }
        Ok((link.then_some(object), symbols))
    }
}

// Preserve Rust's function-item identity. Rust lowers only the forwarding
// call and keeps its ordinary function values, coercions and symbol rules.
fn bind(sess: &Session, item: &mut Item, symbol: &str) -> Symbol {
    let span = item.span;
    let ItemKind::Fn(function) = &mut item.kind else { unreachable!() };
    let PatKind::Ident(_, parameter, None) = function.sig.decl.inputs[0].pat.kind
        else { unreachable!() };
    let mut local = Symbol::intern(symbol);
    if local == parameter.name {
        local = Symbol::intern(&format!("_{symbol}"));
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
    let call = expr(ExprKind::Call(path(id), [path(parameter)].into_iter().collect()));
    function.body.as_mut().unwrap().stmts = [
        Stmt { id: rustc_ast::DUMMY_NODE_ID, kind: StmtKind::Item(import), span },
        Stmt { id: rustc_ast::DUMMY_NODE_ID, kind: StmtKind::Expr(call), span },
    ].into_iter().collect();
    item.tokens = None;
    local
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
        let object = Arc::clone(&self.object);
        config.make_codegen_backend = Some(Box::new(move |_| Box::new(Backend {
            original: rustc_codegen_llvm::LlvmCodegenBackend::new(),
            object,
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
        let selected: Vec<_> = krate.items.iter().enumerate().filter(|(_, item)| {
            native_function(item).is_some()
                && sess.source_map().lookup_source_file(item.span.lo()).normalized_pos.is_empty()
        }).map(|(index, item)| (index, item.as_ref())).collect();
        if let Some((_, first)) = selected.first() {
            let directory = Scratch::create(self.output.as_deref().unwrap())
                .unwrap_or_else(|error| sess.dcx().fatal(format!("cannot create native item work directory: {error}")));
            let input = self.input.as_deref().unwrap();
            let items: Vec<_> = selected.iter().map(|(_, item)| *item).collect();
            let (object, symbols) = self.complete_items(sess, input, &items, &directory.0)
                .unwrap_or_else(|error| sess.dcx().span_fatal(first.span, error));
            let indices: Vec<_> = selected.into_iter().map(|(index, _)| index).collect();
            for (index, symbol) in indices.into_iter().zip(symbols) {
                let name = bind(sess, &mut krate.items[index], &symbol);
                self.bindings.push((name, symbol));
            }
            if let Some(object) = object {
                self.object.set(object).expect("one native object per invocation");
            }
            self.scratch = Some(directory);
        }
        Compilation::Continue
    }

    fn after_expansion(&mut self, _: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        for (name, symbol) in &self.bindings {
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
            let scalar = |arg: &ArgAbi<'_, ty::Ty<'_>>| {
                matches!(arg.layout.ty.kind(), ty::Uint(ty::UintTy::U64))
                    && arg.layout.size.bits() == 64 && arg.layout.align.bytes() == 8
                    && matches!(arg.mode, PassMode::Direct(_))
            };
            if abi.can_unwind || abi.args.len() != 1 || !scalar(&abi.args[0]) || !scalar(&abi.ret) {
                tcx.dcx().fatal("the original call ABI differs from the native object");
            }
        }
        Compilation::Continue
    }
}

struct Backend {
    original: Box<dyn CodegenBackend>,
    object: Arc<OnceLock<PathBuf>>,
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
        if let Some(object) = self.object.get() {
            modules.modules.push(CompiledModule {
                name: "lustr-native-items".into(), kind: ModuleKind::Regular, object: Some(object.clone()),
                global_asm_object: None, dwarf_object: None, bytecode: None, assembly: None, llvm_ir: None,
            });
        }
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
            object: Arc::default(), bindings: Vec::new(), scratch: None,
        };
        rustc_driver::compiler_entrypoint(&driver, &mut callbacks);
    })
}
