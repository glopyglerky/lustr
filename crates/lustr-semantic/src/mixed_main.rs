#![feature(rustc_private)]
extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_codegen_llvm;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_structures;
extern crate rustc_target;
extern crate serde_json;

use rustc_ast::{
    ast,
    visit::{self, Visitor},
};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::{
    mono::MonoItem,
    ty::{self, TyCtxt, TypingEnv},
};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    hash::{Hash, Hasher},
    path::PathBuf,
    process::{Command, ExitCode},
};

rustc_driver::override_c_allocator_in_binary!();
mod mixed_backend;
mod mixed_cache;
mod mixed_diagnostics;
mod mixed_entry;
mod mixed_objects;
mod mixed_profile;
mod mixed_symbol_rename;

#[allow(dead_code)]
#[rustfmt::skip]
mod native {
    include!("../../../compiler/native/src/client.rs");

    pub fn compile(
        file: &Path,
        output: &Path,
        name: &str,
        identity: &str,
        overflow: bool,
        assertions: bool,
    ) -> Option<(i32, String, String)> {
        request(file, output, name, identity, overflow, assertions, false)
    }
    pub fn check(
        file: &Path,
        name: &str,
        identity: &str,
        overflow: bool,
        assertions: bool,
    ) -> Option<(i32, String, String)> {
        request(file, file, name, identity, overflow, assertions, true)
    }
    fn request(
        file: &Path,
        output: &Path,
        name: &str,
        identity: &str,
        overflow: bool,
        assertions: bool,
        check: bool,
    ) -> Option<(i32, String, String)> {
        let _profile = super::mixed_profile::span(if check { "native-check" } else { "native-compile" });
        let setup = super::mixed_profile::span("native-request-setup");
        if env_on("LUST_NO_DAEMON") || protocol::source_override().is_some() {return None;}
        let args = vec![if check {"check".into()} else {"build".into()}, file.to_str()?.into(), "-o".into(), output.to_str()?.into()];
        let Some(Invocation::Compile { mut req, .. }) = protocol::parse_args(&args) else { return None };
        req.session.krate.target_kind = "lustr-mixed-native".into();
        req.session.krate.target_name = name.into();
        req.session.krate.package_id = format!("lustr-mixed-{identity}");
        req.session.profile.overflow_checks = overflow;
        req.session.profile.debug_assertions = assertions;
        let core = std::env::current_exe().ok()?.with_file_name("lust-core");
        // A packaged toolchain also has the pristine Lust daemon. Keep its
        // original session and this compiler revision in distinct namespaces.
        let original_socket = protocol::socket_path();
        let socket =
            original_socket.with_file_name(format!("mixed-{}.sock", super::hash(&(original_socket.as_os_str(), env!("LUST_BUILD_HASH")))));
        let address = protocol::host_runtime::DaemonSocket::resolve(&socket).ok()?;
        let socket = address.path();
        drop(setup);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let roundtrip = super::mixed_profile::span("native-daemon-roundtrip");
            let response = protocol::request(&socket, &req);
            let bounds = roundtrip.close();
            if let Some(r) = response {
                super::mixed_profile::native_response(&_profile, bounds, identity, r.code, &r.stderr);
                return Some((r.code, r.stdout, r.stderr));
            }
            if std::time::Instant::now() >= deadline {
                return None;
            }
            if let Some(lock) = claim_start(&protocol::daemon_lock_path(&socket)) {
                let fd = lock.as_raw_fd();
                let dir = log_dir(file);
                std::fs::create_dir_all(&dir).ok()?;
                let log = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("daemon.log")).ok()?;
                let mut command = std::process::Command::new(&core);
                command
                    .arg("daemon")
                    .envs(daemon_alloc_env())
                    .env("LUST_DAEMON_LOCK_FD", fd.to_string())
                    .env("LUST_DAEMON_SOCKET", &socket)
                    .env_remove("LUST_DAEMON_PREWARM")
                    .env_remove("LUST_DAEMON_PREWARM_AFTER")
                    .stdin(std::process::Stdio::null())
                    .stdout(log.try_clone().ok()?)
                    .stderr(log);
                let directory_fd = address.inherited_fd();
                if let Some(fd) = directory_fd {
                    command.env("LUST_DAEMON_DIRECTORY_FD", fd.to_string());
                } else {
                    command.env_remove("LUST_DAEMON_DIRECTORY_FD");
                }
                unsafe {
                    command.pre_exec(move || {
                        if setsid() < 0 || fcntl(fd, F_SETFD, 0) < 0
                            || directory_fd.is_some_and(|fd|fcntl(fd,F_SETFD,0)<0) {
                            return Err(std::io::Error::last_os_error());
                        }
                        Ok(())
                    });
                }
                command.spawn().ok()?;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

fn hash(value: &impl Hash) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut h);
    format!("{:016x}", h.finish())
}
fn trace(value: &str) {
    if let Some(path) = std::env::var_os("LUSTR_MIXED_TRACE") {
        use std::io::Write;
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
            // Cargo can run several compiler processes at once. Keep the
            // complete diagnostic and its final newline together, including
            // multi-line emitter details and any write_all short-write retry.
            let record = format!("{value}\n");
            if f.lock().is_ok() {
                let _ = f.write_all(record.as_bytes());
            }
        }
    }
}

#[derive(Clone)]
struct Candidate {
    name: String,
    params: Vec<String>,
    header: String,
    body: String,
    body_span: rustc_span::Span,
}
#[derive(Default)]
struct Macros(bool);
impl<'a> Visitor<'a> for Macros {
    fn visit_mac_call(&mut self, _: &'a ast::MacCall) {
        self.0 = true;
    }
    fn visit_attribute(&mut self, _: &'a ast::Attribute) {
        self.0 = true;
    }
}

/// The isolated native unit must see the same bindings as this Rust body.
/// Carrying arbitrary Rust item/trait/const meanings into it is a later bridge.
#[derive(Clone, Copy)]
enum BodyBindings {
    Current,
    // Only after the unchanged semantic context and a previously checked
    // body, or its certified literal edit, establish the same resolution.
    Retained,
}
fn represented_body(tcx: TyCtxt<'_>, id: rustc_span::def_id::LocalDefId, candidate: &Candidate, bindings: BodyBindings) -> bool {
    use rustc_hir::{
        self as hir,
        def::{DefKind, Res},
        intravisit::{self, Visitor},
    };
    if tcx.def_kind(id) != DefKind::Fn || tcx.opt_local_parent(id) != Some(rustc_span::def_id::CRATE_DEF_ID) {
        return false;
    }
    let body = tcx.hir_body_owned_by(id);
    let span = body.value.span;
    if (span.lo(), span.hi(), span.ctxt()) != (candidate.body_span.lo(), candidate.body_span.hi(), candidate.body_span.ctxt()) {
        return false;
    }
    struct Scope<'tcx> {
        valid: bool,
        tcx: TyCtxt<'tcx>,
        owner: rustc_span::def_id::LocalDefId,
        bindings: BodyBindings,
    }
    impl<'tcx> Visitor<'tcx> for Scope<'tcx> {
        fn visit_path(&mut self, path: &hir::Path<'tcx>, _: hir::HirId) {
            if !matches!(path.res, Res::Local(_) | Res::PrimTy(_)) {
                self.valid = false;
            }
            intravisit::walk_path(self, path);
        }
        fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
            match expr.kind {
                hir::ExprKind::MethodCall(segment, receiver, _, _) => {
                    let named = matches!(
                        segment.ident.name.as_str(),
                        "wrapping_add"
                            | "wrapping_sub"
                            | "wrapping_mul"
                            | "wrapping_neg"
                            | "wrapping_shl"
                            | "wrapping_shr"
                            | "rotate_left"
                            | "rotate_right"
                    );
                    let resolved = matches!(self.bindings, BodyBindings::Retained)
                        || (|| {
                            // Reuse the actual Rust method identity, not its spelling.
                            // User traits and lookalike modules cannot establish it.
                            let checked = self.tcx.typeck(self.owner);
                            let method = checked.type_dependent_def_id(expr.hir_id)?;
                            let implementation = self.tcx.inherent_impl_of_assoc(method)?;
                            let core = self.tcx.lang_items().sized_trait()?.krate;
                            let receiver = checked.expr_ty_adjusted(receiver);
                            Some(
                                method.krate == core
                                    && matches!(receiver.kind(), ty::TyKind::Int(_) | ty::TyKind::Uint(_))
                                    && self.tcx.type_of(implementation).instantiate_identity().skip_norm_wip() == receiver,
                            )
                        })()
                        .unwrap_or(false);
                    self.valid &= named && resolved;
                }
                hir::ExprKind::Call(..)
                | hir::ExprKind::ConstBlock(..)
                | hir::ExprKind::Path(hir::QPath::TypeRelative(..))
                | hir::ExprKind::Closure { .. }
                | hir::ExprKind::InlineAsm(..) => self.valid = false,
                hir::ExprKind::Lit(lit) if !matches!(lit.node, ast::LitKind::Int(..) | ast::LitKind::Bool(..)) => self.valid = false,
                _ => {}
            }
            intravisit::walk_expr(self, expr);
        }
        fn visit_nested_body(&mut self, _: hir::BodyId) {
            self.valid = false;
        }
        fn visit_nested_item(&mut self, _: hir::ItemId) {
            self.valid = false;
        }
    }
    let mut scope = Scope { valid: true, tcx, owner: id, bindings };
    scope.visit_body(body);
    scope.valid
}

// Unrepresented code-generation policies stay with the complete backend.
// This checks compiler request options, not source-language keywords.
fn represented_request(args: &[String]) -> bool {
    let mut args = args.iter().skip(1);
    while let Some(arg) = args.next() {
        if arg.starts_with('@') || arg.starts_with("--codegen") || arg == "--test" {
            return false;
        }
        if let Some(option) = arg.strip_prefix("-Z") {
            let option = if option.is_empty() {
                let Some(next) = args.next() else { return false };
                next.as_str()
            } else {
                option
            };
            // The matching Cargo requests split metadata for its rlibs.
            // Rust's original encoder and archive writer retain this policy;
            // no other unstable code-generation request is represented.
            if !matches!(option, "embed-metadata=no" | "embed-metadata=yes") {
                return false;
            }
            continue;
        }
        if let Some(option) = arg.strip_prefix("-C") {
            let option = if option.is_empty() {
                let Some(next) = args.next() else { return false };
                next.as_str()
            } else {
                option
            };
            let name = option.split('=').next().unwrap();
            if !matches!(
                name,
                "embed-bitcode"
                    | "metadata"
                    | "extra-filename"
                    | "incremental"
                    | "debuginfo"
                    | "opt-level"
                    | "panic"
                    | "overflow-checks"
                    | "debug-assertions"
                    | "codegen-units"
                    | "linker"
                    | "link-arg"
                    | "link-args"
                    | "strip"
            ) {
                return false;
            }
        }
    }
    true
}

struct Compiler {
    directory: PathBuf,
    index: PathBuf,
    cached: Option<serde_json::Value>,
    candidates: BTreeMap<String, Candidate>,
    selected: Vec<String>,
    frontend: bool,
    enabled: bool,
    check_only: bool,
    reuse_allowed: bool,
    reused: bool,
    represented: bool,
    context: Option<serde_json::Value>,
    records: Vec<serde_json::Value>,
    _lock: Option<fs::File>,
}
impl Callbacks for Compiler {
    fn config(&mut self, config: &mut interface::Config) {
        use rustc_session::config::OutputType;
        self.check_only = config.opts.output_types.contains_key(&OutputType::Metadata)
            && config.opts.output_types.keys().all(|kind| matches!(kind, OutputType::Metadata | OutputType::DepInfo));
        let library = config.opts.crate_types == [rustc_structures::CrateType::Rlib];
        self.enabled = self.represented
            && cfg!(all(target_os = "linux", target_arch = "x86_64"))
            && config.opts.optimize == rustc_session::config::OptLevel::No
            && (self.check_only || config.opts.cg.debuginfo == rustc_session::config::DebugInfo::None)
            && config.opts.target_triple.to_string() == env!("LUSTR_BUILD_RUSTC_HOST")
            && config.opts.unstable_opts.offload.is_empty()
            && (config.opts.crate_types == [rustc_structures::CrateType::Executable] || library)
            // Native scalar objects have no embedded LLVM bitcode. An rlib
            // may mix them only when the request explicitly disables it;
            // bitcode/LTO requests keep the complete original backend.
            && (self.check_only || !library || !config.opts.cg.embed_bitcode)
            && (self.check_only
                || config
                    .opts
                    .output_types
                    .keys()
                    .all(|kind| matches!(kind, OutputType::Exe | OutputType::DepInfo) || library && *kind == OutputType::Metadata));
        // An rlib edit must re-encode genuine metadata for its new source.
        // The retained executable link recipe carries prior metadata and is
        // never reused for linked libraries. Metadata-only checks keep their
        // separately qualified fresh-encoding path.
        self.reuse_allowed &= self.check_only || !library;
        self.reuse_allowed &= config.opts.lint_opts.iter().all(|(name, _)| name == "unused_crate_dependencies");
        if self.enabled {
            let previous = config.psess_created.take();
            config.psess_created = Some(Box::new(move |session| {
                if let Some(previous) = previous {
                    previous(session);
                }
                mixed_diagnostics::install();
            }));
            config.make_codegen_backend = Some(Box::new(|_| Box::new(mixed_backend::Backend::new())));
            config.override_queries = Some(mixed_backend::providers);
        }
    }
    fn after_crate_root_parsing(&mut self, compiler: &interface::Compiler, krate: &mut ast::Crate) -> Compilation {
        let _profile = mixed_profile::span("parse-candidates");
        self.frontend = true;
        trace("frontend=1");
        if !self.enabled {
            return Compilation::Continue;
        }
        self.reuse_allowed &= krate.attrs.is_empty();
        for item in &krate.items {
            let ast::ItemKind::Fn(f) = &item.kind else { continue };
            if !item.attrs.is_empty() || f.sig.header.has_qualifiers() || !f.generics.params.is_empty() {
                continue;
            }
            let Some(body) = &f.body else { continue };
            let mut macros = Macros::default();
            visit::walk_block(&mut macros, body);
            if macros.0 {
                continue;
            }
            let params: Option<Vec<_>> = f
                .sig
                .decl
                .inputs
                .iter()
                .map(|p| match &p.pat.kind {
                    ast::PatKind::Ident(_, _, None) if p.attrs.is_empty() => compiler.sess.source_map().span_to_snippet(p.pat.span).ok(),
                    _ => None,
                })
                .collect();
            let Some(params) = params else { continue };
            let Ok(header) = compiler.sess.source_map().span_to_snippet(item.span.with_hi(body.span.lo())) else { continue };
            let body_span = body.span;
            let Ok(body) = compiler.sess.source_map().span_to_snippet(body_span) else { continue };
            let name = f.ident.to_string();
            self.candidates.insert(name.clone(), Candidate { name, params, header, body, body_span });
        }
        Compilation::Continue
    }
    fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        let _profile = mixed_profile::span("after-expansion");
        // Continue through the driver's original dep-info writer. Only its
        // analysis query is skipped after the retained boundary is verified.
        if mixed_cache::attempt(self, tcx).is_some() {
            mixed_backend::reuse_checked_context();
        }
        Compilation::Continue
    }
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        let _profile = mixed_profile::span("select-native-after-analysis");
        if self.reused {
            // Checks continue through Rust's original metadata encoder. Its
            // current source/crate hash and real rmeta are never reused from
            // the previous request. Link requests have already relinked.
            return if self.check_only { Compilation::Continue } else { Compilation::Stop };
        }
        if !self.enabled
            || tcx.sess.dcx().has_errors().is_some()
            || !self.check_only && !tcx.sess.opts.output_types.should_codegen()
            || tcx.sess.opts.optimize != rustc_session::config::OptLevel::No
            || !self.check_only && tcx.sess.opts.cg.debuginfo != rustc_session::config::DebugInfo::None
            || tcx.sess.target.llvm_target.as_ref() != "x86_64-unknown-linux-gnu"
        {
            return Compilation::Continue;
        }
        let original = self.directory.clone();
        if fs::create_dir_all(original.parent().unwrap()).is_err() {
            return Compilation::Continue;
        }
        for n in 0.. {
            self.directory = original.with_file_name(format!("{}-{n}", original.file_name().unwrap().to_string_lossy()));
            match fs::create_dir(&self.directory) {
                Ok(()) => break,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Compilation::Continue,
            }
        }
        let mut entries = BTreeMap::new();
        let instances: Vec<_> = if self.check_only {
            // A metadata request has no monomorphization/codegen units.
            // Use actual nongeneric root definitions from the checked HIR.
            tcx.hir_crate_items(())
                .owners()
                .filter_map(|owner| {
                    let id = owner.def_id.to_def_id();
                    (tcx.def_kind(id) == rustc_hir::def::DefKind::Fn
                        && tcx.opt_local_parent(owner.def_id) == Some(rustc_span::def_id::CRATE_DEF_ID)
                        && tcx.generics_of(id).count() == 0)
                        .then(|| ty::Instance::mono(tcx, id))
                })
                .collect()
        } else {
            tcx.collect_and_partition_mono_items(())
                .codegen_units
                .iter()
                .flat_map(|unit| {
                    unit.items().keys().filter_map(|item| match item {
                        MonoItem::Fn(instance) => Some(*instance),
                        _ => None,
                    })
                })
                .collect()
        };
        for instance in instances {
            let id = instance.def_id();
            if !id.is_local() || tcx.generics_of(id).count() != 0 {
                continue;
            }
            let Some(name) = tcx.opt_item_name(id) else { continue };
            let name = name.to_string();
            let Some(candidate) = self.candidates.get(&name) else { continue };
            if !represented_body(tcx, id.expect_local(), candidate, BodyBindings::Current) {
                continue;
            }
            let signature = tcx.fn_sig(id).instantiate_identity().skip_binder();
            let mut c_signature = signature;
            c_signature.fn_sig_kind = c_signature.fn_sig_kind.set_abi(rustc_abi::ExternAbi::C { unwind: false });
            let environment = TypingEnv::fully_monomorphized();
            let Ok(rust_abi) = tcx.fn_abi_of_instance(environment.as_query_input((instance, ty::List::empty()))) else { continue };
            let Ok(c_abi) = tcx.fn_abi_of_fn_ptr(environment.as_query_input((ty::Binder::dummy(c_signature), ty::List::empty()))) else {
                continue;
            };
            let mode_equal = |a: &rustc_target::callconv::PassMode, b: &rustc_target::callconv::PassMode| {
                matches!((a, b), (rustc_target::callconv::PassMode::Direct(_), rustc_target::callconv::PassMode::Direct(_)))
                    && format!("{a:?}") == format!("{b:?}")
            };
            // The pinned LLVM backend maps canonical Rust and C to the
            // same LLVM C calling convention (upstream abi.rs). Compare
            // the actual argument/return modes too; a canonical ABI name
            // alone neither establishes nor disproves compatibility.
            let same_convention =
                rust_abi.conv == c_abi.conv || matches!((rust_abi.conv, c_abi.conv), (rustc_abi::CanonAbi::Rust, rustc_abi::CanonAbi::C));
            if !same_convention
                || !mode_equal(&rust_abi.ret.mode, &c_abi.ret.mode)
                || rust_abi.args.len() != c_abi.args.len()
                || !rust_abi.args.iter().zip(&c_abi.args).all(|(a, b)| mode_equal(&a.mode, &b.mode))
            {
                continue;
            }
            let primitive = |t: ty::Ty<'tcx>| {
                matches!(
                    t.kind(),
                    ty::TyKind::Int(ty::IntTy::I64 | ty::IntTy::Isize) | ty::TyKind::Uint(ty::UintTy::U64 | ty::UintTy::Usize)
                )
            };
            if !primitive(signature.output())
                || !signature.inputs().iter().all(|t| primitive(*t))
                || candidate.params.len() != signature.inputs().len()
            {
                continue;
            }
            let symbol = tcx.symbol_name(instance).name.to_owned();
            let identity = hash(&symbol);
            let path = self.directory.join(&identity);
            if fs::create_dir(&path).is_err() {
                continue;
            }
            let source = path.join("kernel.rs");
            let object = path.join("native.o");
            let params = candidate.params.iter().zip(signature.inputs()).map(|(p, t)| format!("{p}:{t}")).collect::<Vec<_>>().join(",");
            let zeros = candidate.params.iter().map(|_| "0").collect::<Vec<_>>().join(",");
            let prefix = format!("fn {}({params})->{}", candidate.name, signature.output());
            let suffix = format!("\nfn main(){{let _={}({zeros});}}\n", candidate.name);
            let text = format!("{prefix}{}{suffix}", candidate.body);
            if fs::write(&source, &text).is_err() {
                continue;
            }
            if self.check_only {
                let Some((code, _, diagnostic)) =
                    native::check(&source, &name, &identity, tcx.sess.overflow_checks(), tcx.sess.opts.debug_assertions)
                else {
                    continue;
                };
                trace(&diagnostic);
                if code != 0 {
                    continue;
                }
            } else {
                let Some((code, native_symbol, diagnostic)) =
                    native::compile(&source, &object, &name, &identity, tcx.sess.overflow_checks(), tcx.sess.opts.debug_assertions)
                else {
                    continue;
                };
                trace(&diagnostic);
                if code != 0 {
                    continue;
                }
                let linked = path.join("native-link.o");
                let result = {
                    let _profile = mixed_profile::span("native-symbol-rename");
                    mixed_symbol_rename::rename(&object, &native_symbol, &symbol, &linked)
                };
                if result.is_err() {
                    continue;
                }
                entries.insert(symbol.clone(), linked);
            }
            self.records.push(serde_json::json!({"name":name,"symbol":symbol,"identity":identity,
                    "definition":format!("{:?}",tcx.def_path_hash(id)),"header":candidate.header,
                    "body":candidate.body,"prefix":prefix,"suffix":suffix}));
            self.selected.push(symbol.clone());
        }
        trace(&format!("{}={}", if self.check_only { "native-checked-definitions" } else { "native-definitions" }, self.records.len()));
        if !self.records.is_empty() {
            if self.reuse_allowed {
                let excluded = self.records.iter().filter_map(|r| r["definition"].as_str().map(str::to_owned)).collect();
                self.context = mixed_cache::context(tcx, &excluded);
            }
            if !self.check_only {
                mixed_objects::install(self.directory.clone(), entries);
            }
        }
        Compilation::Continue
    }
}

fn main() -> ExitCode {
    let mut os: Vec<OsString> = std::env::args_os().collect();
    if os.iter().any(|a| a.to_str().is_none()) {
        use std::os::unix::process::CommandExt;
        let original = std::env::current_exe().unwrap().with_file_name("rustc-reference");
        let error = Command::new(original).args(os.drain(1..)).exec();
        eprintln!("cannot enter the complete compiler: {error}");
        return ExitCode::from(2);
    }
    let args: Vec<String> = os.into_iter().map(|a| a.into_string().unwrap()).collect();
    let represented = represented_request(&args);
    // Once request flags rule out native acceleration, run the already-pinned
    // rustc driver directly. This avoids mixed cache and backend setup without
    // another process startup or a change to the complete Rust implementation.
    // No source-content heuristic is used.
    // The separate phase-profile campaign needs an in-process receipt for
    // each invocation; keep that diagnostic route complete and labeled.
    if std::env::var_os("LUSTR_MIXED_PROFILE_DIR").is_none()
        && (!represented || mixed_entry::full_rust_only(&args, env!("LUSTR_BUILD_RUSTC_HOST")))
    {
        struct CompleteRust;
        impl Callbacks for CompleteRust {}
        let dcx = rustc_session::EarlyDiagCtxt::new(rustc_session::config::ErrorOutputType::default());
        rustc_driver::init_rustc_env_logger(&dcx);
        rustc_driver::install_ctrlc_handler();
        trace("full-rust-direct=1");
        return rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&args, &mut CompleteRust));
    }
    let request = mixed_profile::span("driver");
    let setup = mixed_profile::span("request-setup");
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    let uid = unsafe { geteuid() };
    let root = std::env::var_os("LUSTR_MIXED_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("lustr-mixed-{uid}")));
    let environment: BTreeMap<_, _> =
        std::env::vars_os().filter(|(k, _)| !["CARGO_MAKEFLAGS", "MAKEFLAGS", "MFLAGS", "NUM_JOBS"].iter().any(|v| k == v)).collect();
    use std::os::unix::fs::MetadataExt;
    let executable = std::env::current_exe().ok();
    let identity = |p: &std::path::Path| fs::metadata(p).ok().map(|m| (m.dev(), m.ino(), m.len(), m.mtime(), m.mtime_nsec()));
    let binaries = executable.as_ref().map(|p| (identity(p), identity(&p.with_file_name("lust-core"))));
    let key = hash(&(&args, std::env::current_dir().ok(), environment, binaries));
    let directory = root.join(&key);
    let index = directory.join("current.json");
    use std::os::unix::fs::DirBuilderExt;
    // Generated projections contain application source. Keep them private and
    // refuse an existing writable/shared or substituted cache directory.
    let private = |path: &std::path::Path, mask: u32| {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(path).ok()?;
        let m = fs::symlink_metadata(path).ok()?;
        (m.is_dir() && m.uid() == uid && m.mode() & mask == 0).then_some(())
    };
    let lock = private(&root, 0o022)
        .and_then(|_| private(&directory, 0o077))
        .and_then(|_| fs::OpenOptions::new().create(true).write(true).truncate(false).open(directory.join("request.lock")).ok())
        .and_then(|f| f.try_lock().ok().map(|_| f));
    let cached = lock.as_ref().and_then(|_| mixed_cache::read(&index));
    trace(&format!("request-key={} retained={}", directory.file_name().unwrap().to_string_lossy(), cached.is_some()));
    let mut callbacks = Compiler {
        directory: directory.join("generation"),
        index,
        cached,
        candidates: BTreeMap::new(),
        selected: Vec::new(),
        frontend: false,
        enabled: false,
        check_only: false,
        reuse_allowed: lock.is_some(),
        reused: false,
        represented: lock.is_some() && represented,
        context: None,
        records: Vec::new(),
        _lock: lock,
    };
    let dcx = rustc_session::EarlyDiagCtxt::new(rustc_session::config::ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&dcx);
    rustc_driver::install_ctrlc_handler();
    drop(setup);
    let driver = mixed_profile::span("rustc-driver");
    let status = rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&args, &mut callbacks));
    drop(driver);
    if status == ExitCode::SUCCESS
        && callbacks.reuse_allowed
        && mixed_diagnostics::quiet()
        && (callbacks.check_only || callbacks.directory.join("checked.rlink").is_file())
        && let Some(context) = callbacks.context.take()
        && let Err(e) = mixed_cache::publish(
            &callbacks.index,
            &callbacks.directory,
            context,
            std::mem::take(&mut callbacks.records),
            callbacks.check_only,
        )
    {
        trace(&format!("cache-not-retained={e}"));
    }
    drop(request);
    mixed_profile::finish(status == ExitCode::SUCCESS, callbacks.reused, callbacks.check_only, &key);
    status
}
