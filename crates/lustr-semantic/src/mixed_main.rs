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
    collections::{BTreeMap, BTreeSet},
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

    pub enum Outcome {
        Disabled,
        Complementary,
        Checked,
        Emitted(String),
    }

    pub fn compile(
        file: &Path,
        output: &Path,
        name: &str,
        identity: &str,
        overflow: bool,
        assertions: bool,
    ) -> Result<Outcome, String> {
        request(file, output, name, identity, overflow, assertions, false)
    }
    pub fn check(
        file: &Path,
        name: &str,
        identity: &str,
        overflow: bool,
        assertions: bool,
    ) -> Result<Outcome, String> {
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
    ) -> Result<Outcome, String> {
        let _profile = super::mixed_profile::span(if check { "native-check" } else { "native-compile" });
        let setup = super::mixed_profile::span("native-request-setup");
        if env_on("LUST_NO_DAEMON") || protocol::source_override().is_some() {
            super::trace("native-boundary=disabled-by-configuration");
            return Ok(Outcome::Disabled);
        }
        let input = file.to_str().ok_or("mixed native input path is not UTF-8")?;
        let output = output.to_str().ok_or("mixed native output path is not UTF-8")?;
        let args = vec![if check {"check".into()} else {"build".into()}, input.into(), "-o".into(), output.into()];
        let Some(Invocation::Compile { mut req, .. }) = protocol::parse_args(&args) else {
            return Err("mixed native request could not be constructed".into());
        };
        req.session.krate.target_kind = "lustr-mixed-native".into();
        req.session.krate.target_name = name.into();
        req.session.krate.package_id = format!("lustr-mixed-{identity}");
        req.session.profile.overflow_checks = overflow;
        req.session.profile.debug_assertions = assertions;
        let core = std::env::current_exe().map_err(|e| format!("locate mixed native compiler: {e}"))?.with_file_name("lust-core");
        // A packaged toolchain also has the pristine Lust daemon. Keep its
        // original session and this compiler revision in distinct namespaces.
        let original_socket = protocol::socket_path();
        let socket =
            original_socket.with_file_name(format!("mixed-{}.sock", super::hash(&(original_socket.as_os_str(), env!("LUST_BUILD_HASH")))));
        let address = protocol::host_runtime::DaemonSocket::resolve(&socket).map_err(|e| format!("resolve mixed native socket: {e}"))?;
        let socket = address.path();
        drop(setup);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let roundtrip = super::mixed_profile::span("native-daemon-roundtrip");
            let response = protocol::request_mixed(&socket, &req)?;
            let bounds = roundtrip.close();
            if let Some(r) = response {
                let admission = r.mixed.map(|outcome| match outcome {
                    protocol::MixedOutcome::Checked => "checked",
                    protocol::MixedOutcome::Emitted => "emitted",
                    protocol::MixedOutcome::Complementary => "complementary",
                    protocol::MixedOutcome::Failed => "failed",
                });
                super::mixed_profile::native_response(&_profile, bounds, identity, r.code, admission, &r.stderr);
                super::trace(&r.stderr);
                return outcome(r, check, identity);
            }
            if std::time::Instant::now() >= deadline {
                return Err("mixed native daemon did not become available before the startup deadline".into());
            }
            if let Some(lock) = claim_start(&protocol::daemon_lock_path(&socket)) {
                let fd = lock.as_raw_fd();
                let dir = log_dir(file);
                std::fs::create_dir_all(&dir).map_err(|e| format!("create native log directory: {e}"))?;
                let log = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("daemon.log"))
                    .map_err(|e| format!("open native daemon log: {e}"))?;
                let mut command = std::process::Command::new(&core);
                command
                    .arg("daemon")
                    .envs(daemon_alloc_env())
                    .env("LUST_DAEMON_LOCK_FD", fd.to_string())
                    .env("LUST_DAEMON_SOCKET", &socket)
                    .env_remove("LUST_DAEMON_PREWARM")
                    .env_remove("LUST_DAEMON_PREWARM_AFTER")
                    .stdin(std::process::Stdio::null())
                    .stdout(log.try_clone().map_err(|e| format!("duplicate native daemon log: {e}"))?)
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
                command.spawn().map_err(|e| format!("start mixed native compiler: {e}"))?;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn outcome(response: protocol::Response, check: bool, identity: &str) -> Result<Outcome, String> {
        use protocol::MixedOutcome;
        // Only an explicit pre-emission complementary result changes which
        // implementation owns the definition. Exit codes and diagnostic text
        // never establish capability, including a successful untyped reply.
        match (response.mixed, response.code) {
            (Some(MixedOutcome::Checked), 0) if check && response.stdout.is_empty() => Ok(Outcome::Checked),
            (Some(MixedOutcome::Emitted), 0) if !check && response.stdout == format!("lustr_mixed_{identity}") =>
                Ok(Outcome::Emitted(response.stdout)),
            (Some(MixedOutcome::Complementary), 0) if response.stdout.is_empty() => {
                super::trace("native-boundary=complementary-before-emission");
                Ok(Outcome::Complementary)
            },
            (Some(MixedOutcome::Failed), code) if code != 0 =>
                Err(format!("native compilation failed (status {code}): {}", response.stderr.trim_end())),
            _ => Err(format!("invalid mixed native outcome {:?} (status {}): {}",
                response.mixed, response.code, response.stderr.trim_end())),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{outcome, protocol::{MixedOutcome, Response}, Outcome};

        #[test]
        fn only_explicit_admission_or_complement_can_succeed() {
            let reply = |mixed, code, stdout: &str| Response { mixed, code, stdout: stdout.into(), ..Default::default() };
            assert!(matches!(outcome(reply(Some(MixedOutcome::Checked), 0, ""), true, "1234"), Ok(Outcome::Checked)));
            assert!(matches!(outcome(reply(Some(MixedOutcome::Emitted), 0, "lustr_mixed_1234"), false, "1234"), Ok(Outcome::Emitted(_))));
            assert!(matches!(outcome(reply(Some(MixedOutcome::Complementary), 0, ""), false, "1234"), Ok(Outcome::Complementary)));
            for (mixed, code, symbol, check) in [
                (None, 0, "", true),
                (None, 1, "", false),
                (Some(MixedOutcome::Failed), 1, "", false),
                (Some(MixedOutcome::Failed), 0, "", false),
                (Some(MixedOutcome::Checked), 0, "", false),
                (Some(MixedOutcome::Emitted), 0, "lustr_mixed_1234", true),
                (Some(MixedOutcome::Emitted), 0, "lustr_mixed_5678", false),
                (Some(MixedOutcome::Emitted), 1, "lustr_mixed_1234", false),
                (Some(MixedOutcome::Complementary), 1, "", false),
                (Some(MixedOutcome::Complementary), 0, "unexpected object", false),
            ] {
                assert!(outcome(reply(mixed, code, symbol), check, "1234").is_err());
            }
        }
    }
}

fn required<T, E: std::fmt::Display>(tcx: TyCtxt<'_>, result: Result<T, E>, operation: &str) -> T {
    result.unwrap_or_else(|error| tcx.sess.dcx().fatal(format!("lustr {operation}: {error}")))
}

fn native_boundary(tcx: TyCtxt<'_>, result: Result<native::Outcome, String>) -> Option<String> {
    match required(tcx, result, "mixed native boundary") {
        native::Outcome::Disabled | native::Outcome::Complementary => None,
        native::Outcome::Checked => Some(String::new()),
        native::Outcome::Emitted(symbol) => Some(symbol),
    }
}

fn native_generation(original: &std::path::Path) -> std::io::Result<PathBuf> {
    fs::create_dir_all(original.parent().unwrap())?;
    for n in 0u64.. {
        let directory = original.with_file_name(format!("{}-{n}", original.file_name().unwrap().to_string_lossy()));
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::other("mixed generation identities exhausted"))
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
fn canonical_identifier(map: &rustc_span::source_map::SourceMap, ident: rustc_span::Ident) -> bool {
    let Ok(spelling) = map.span_to_snippet(ident.span) else { return false };
    fn unquoted(s: &str) -> &str {
        let s = s.strip_prefix("r#").unwrap_or(s);
        s.strip_prefix('\'').unwrap_or(s)
    }
    // Preserve this scalar projection's qualified source-spelling contract.
    // The native lexer now normalizes identifiers, but this bridge's identity
    // handoff for decomposed spellings still needs its own qualification.
    // Unicode comments and canonical names remain eligible.
    unquoted(&spelling) == unquoted(ident.name.as_str())
}

struct NativeSource<'a> {
    map: &'a rustc_span::source_map::SourceMap,
    represented: bool,
}
impl<'ast> Visitor<'ast> for NativeSource<'_> {
    fn visit_mac_call(&mut self, _: &'ast ast::MacCall) {
        self.represented = false;
    }
    fn visit_attribute(&mut self, _: &'ast ast::Attribute) {
        self.represented = false;
    }
    fn visit_ident(&mut self, ident: &'ast rustc_span::Ident) {
        self.represented &= canonical_identifier(self.map, *ident);
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
    if tcx.def_kind(id) != DefKind::Fn {
        return false;
    }
    let Some(parent) = tcx.opt_local_parent(id) else { return false };
    if parent != rustc_span::def_id::CRATE_DEF_ID && tcx.def_kind(parent) != DefKind::Mod {
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

fn represented_signature<'tcx>(tcx: TyCtxt<'tcx>, instance: ty::Instance<'tcx>) -> Option<ty::FnSig<'tcx>> {
    let id = instance.def_id();
    let signature = tcx.fn_sig(id).instantiate_identity().skip_binder();
    let primitive = |t: ty::Ty<'tcx>| {
        matches!(
            t.kind(),
            ty::TyKind::Int(ty::IntTy::I8 | ty::IntTy::I16 | ty::IntTy::I32 | ty::IntTy::I64 | ty::IntTy::Isize)
                | ty::TyKind::Uint(ty::UintTy::U8 | ty::UintTy::U16 | ty::UintTy::U32 | ty::UintTy::U64 | ty::UintTy::Usize)
        )
    };
    // Unsupported signatures can contain late-bound regions. They remain
    // with Rust before constructing a C ABI query; putting an opened bound
    // region in Binder::dummy is an internal compiler error, not a capability.
    if !primitive(signature.output()) || !signature.inputs().iter().all(|t| primitive(*t)) {
        return None;
    }
    let mut c_signature = signature;
    c_signature.fn_sig_kind = c_signature.fn_sig_kind.set_abi(rustc_abi::ExternAbi::C { unwind: false });
    let environment = TypingEnv::fully_monomorphized();
    let Ok(rust_abi) = tcx.fn_abi_of_instance(environment.as_query_input((instance, ty::List::empty()))) else { return None };
    let Ok(c_abi) = tcx.fn_abi_of_fn_ptr(environment.as_query_input((ty::Binder::dummy(c_signature), ty::List::empty()))) else {
        return None;
    };
    let mode_equal = |a: &rustc_target::callconv::PassMode, b: &rustc_target::callconv::PassMode| {
        matches!((a, b), (rustc_target::callconv::PassMode::Direct(a), rustc_target::callconv::PassMode::Direct(b)) if a == b)
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
        return None;
    }
    Some(signature)
}

fn parsed_candidates(compiler: &interface::Compiler, krate: &ast::Crate) -> BTreeMap<(u32, u32), Candidate> {
    struct Collect<'a> {
        compiler: &'a interface::Compiler,
        candidates: BTreeMap<(u32, u32), Candidate>,
    }
    impl<'ast> Visitor<'ast> for Collect<'_> {
        fn visit_item(&mut self, item: &'ast ast::Item) {
            // An attribute on a module can transform all nested source. Only
            // unmodified loaded source items are projected into native units.
            if !item.attrs.is_empty() {
                return;
            }
            if matches!(&item.kind, ast::ItemKind::Mod(..)) {
                visit::walk_item(self, item);
                return;
            }
            let ast::ItemKind::Fn(f) = &item.kind else { return };
            if f.sig.header.has_qualifiers() || !f.generics.params.is_empty() {
                return;
            }
            let Some(body) = &f.body else { return };
            let map = self.compiler.sess.source_map();
            if !canonical_identifier(map, f.ident) {
                return;
            }
            let mut source = NativeSource { map, represented: true };
            visit::walk_block(&mut source, body);
            if !source.represented {
                return;
            }
            let params: Option<Vec<_>> = f
                .sig
                .decl
                .inputs
                .iter()
                .map(|p| match &p.pat.kind {
                    ast::PatKind::Ident(_, ident, None) if p.attrs.is_empty() && canonical_identifier(map, *ident) => {
                        map.span_to_snippet(p.pat.span).ok()
                    }
                    _ => None,
                })
                .collect();
            let Some(params) = params else { return };
            let map = self.compiler.sess.source_map();
            let Ok(header) = map.span_to_snippet(item.span.with_hi(body.span.lo())) else { return };
            let body_span = body.span;
            let Ok(body) = map.span_to_snippet(body_span) else { return };
            // Keep required raw-identifier quoting in the internal source.
            let Ok(name) = map.span_to_snippet(f.ident.span) else { return };
            self.candidates.insert((body_span.lo().0, body_span.hi().0), Candidate { name, params, header, body, body_span });
        }
    }
    let mut collect = Collect { compiler, candidates: BTreeMap::new() };
    visit::walk_crate(&mut collect, krate);
    collect.candidates
}

fn candidate_for<'a>(
    candidates: &'a BTreeMap<(u32, u32), Candidate>,
    tcx: TyCtxt<'_>,
    id: rustc_span::def_id::LocalDefId,
) -> Option<&'a Candidate> {
    if tcx.def_kind(id) != rustc_hir::def::DefKind::Fn {
        return None;
    }
    let body = tcx.hir_body_owned_by(id);
    let span = body.value.span;
    // Same spellings in distinct modules have distinct source and definition
    // identities. Never use a short name as cross-invocation authority.
    let candidate = candidates.get(&(span.lo().0, span.hi().0))?;
    (candidate.body_span.ctxt() == span.ctxt()).then_some(candidate)
}

struct Compiler {
    directory: PathBuf,
    index: PathBuf,
    cached: Option<serde_json::Value>,
    candidates: BTreeMap<(u32, u32), Candidate>,
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
        self.candidates = parsed_candidates(compiler, krate);
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
        let mut prepared = false;
        let mut entries = BTreeMap::new();
        let mut considered = BTreeSet::new();
        let instances: Vec<_> = if self.check_only {
            // A metadata request has no monomorphization/codegen units.
            // Use actual nongeneric definitions from the checked HIR.
            // The candidate/owner checks retain the module and source identity.
            tcx.hir_crate_items(())
                .owners()
                .filter_map(|owner| {
                    let id = owner.def_id.to_def_id();
                    (tcx.def_kind(id) == rustc_hir::def::DefKind::Fn && tcx.generics_of(id).count() == 0)
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
            let Some(candidate) = candidate_for(&self.candidates, tcx, id.expect_local()) else { continue };
            if !represented_body(tcx, id.expect_local(), candidate, BodyBindings::Current) {
                continue;
            }
            let Some(signature) = represented_signature(tcx, instance) else { continue };
            if candidate.params.len() != signature.inputs().len() {
                continue;
            }
            let symbol = tcx.symbol_name(instance).name.to_owned();
            if !considered.insert(symbol.clone()) {
                continue;
            }
            let identity = hash(&symbol);
            if !prepared {
                self.directory = required(tcx, native_generation(&self.directory), "create mixed generation");
                prepared = true;
            }
            let path = self.directory.join(&identity);
            required(tcx, fs::create_dir(&path), "create native unit directory");
            let source = path.join("kernel.rs");
            let object = path.join("native.o");
            let params = candidate.params.iter().zip(signature.inputs()).map(|(p, t)| format!("{p}:{t}")).collect::<Vec<_>>().join(",");
            let zeros = candidate.params.iter().map(|_| "0").collect::<Vec<_>>().join(",");
            let prefix = format!("fn {}({params})->{}", candidate.name, signature.output());
            let suffix = format!("\nfn main(){{let _={}({zeros});}}\n", candidate.name);
            let text = format!("{prefix}{}{suffix}", candidate.body);
            required(tcx, fs::write(&source, &text), "write checked native projection");
            if self.check_only {
                if native_boundary(
                    tcx,
                    native::check(&source, &name, &identity, tcx.sess.overflow_checks(), tcx.sess.opts.debug_assertions),
                )
                .is_none()
                {
                    continue;
                }
            } else {
                let Some(native_symbol) = native_boundary(
                    tcx,
                    native::compile(&source, &object, &name, &identity, tcx.sess.overflow_checks(), tcx.sess.opts.debug_assertions),
                ) else {
                    continue;
                };
                let linked = path.join("native-link.o");
                let result = {
                    let _profile = mixed_profile::span("native-symbol-rename");
                    mixed_symbol_rename::rename(&object, &native_symbol, &symbol, &linked)
                };
                required(tcx, result, "rename admitted native symbol");
                entries.insert(symbol.clone(), linked);
            }
            self.records.push(serde_json::json!({"name":name,"symbol":symbol,"identity":identity,
                    "definition":format!("{:?}",tcx.def_path_hash(id)),"header":candidate.header,
                    "body":candidate.body,"prefix":prefix,"suffix":suffix}));
            self.selected.push(symbol.clone());
            trace(&format!("native-admitted-definition={}", tcx.def_path_str(id)));
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

#[cfg(test)]
mod capability_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Default)]
    struct Probe {
        candidates: BTreeMap<(u32, u32), Candidate>,
        admitted: BTreeSet<String>,
    }
    impl Callbacks for Probe {
        fn after_crate_root_parsing(&mut self, compiler: &interface::Compiler, krate: &mut ast::Crate) -> Compilation {
            self.candidates = parsed_candidates(compiler, krate);
            Compilation::Continue
        }
        fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
            for owner in tcx.hir_crate_items(()).owners() {
                let id = owner.def_id;
                if tcx.def_kind(id) != rustc_hir::def::DefKind::Fn || tcx.generics_of(id).count() != 0 {
                    continue;
                }
                let Some(candidate) = candidate_for(&self.candidates, tcx, id) else { continue };
                if tcx.opt_item_name(id).is_some_and(|name| name.as_str() == "type") {
                    assert_eq!(candidate.name, "r#type", "projected source must retain raw keyword quoting");
                }
                if represented_body(tcx, id, candidate, BodyBindings::Current)
                    && represented_signature(tcx, ty::Instance::mono(tcx, id.to_def_id())).is_some()
                {
                    self.admitted.insert(tcx.def_path_str(id.to_def_id()));
                }
            }
            Compilation::Stop
        }
    }

    fn admitted(source: &str) -> BTreeSet<String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory =
            std::env::temp_dir().join(format!("lustr-mixed-capability-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&directory).unwrap();
        let input = directory.join("input.rs");
        fs::write(&input, source).unwrap();
        let args = vec![
            "rustc".into(),
            input.display().to_string(),
            "--edition=2024".into(),
            "--crate-name=capability".into(),
            "--crate-type=rlib".into(),
            "--emit=metadata".into(),
            "-Adead_code".into(),
            "--sysroot".into(),
            env!("LUSTR_BUILD_SYSROOT").into(),
            "--out-dir".into(),
            directory.display().to_string(),
        ];
        let mut probe = Probe::default();
        assert_eq!(rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&args, &mut probe)), ExitCode::SUCCESS);
        fs::remove_dir_all(directory).unwrap();
        probe.admitted
    }

    #[test]
    fn checked_integer_signatures_preserve_actual_extension_contracts() {
        let types = ["i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "isize", "usize"];
        let source = types.iter().map(|ty| format!("pub fn kernel_{ty}(x:{ty})->{ty}{{x.wrapping_add(17)}}\n")).collect::<String>();
        // x86_64 Rust and C use different extension attributes for i8/u8/i16/u16.
        // Those calls need an ABI adapter, even though Lust can emit their bodies.
        let expected = ["i32", "u32", "i64", "u64", "isize", "usize"].iter().map(|ty| format!("kernel_{ty}")).collect();
        assert_eq!(admitted(&source), expected);
    }

    #[test]
    fn narrow_returns_with_wide_arguments_use_their_matching_abi() {
        let types = ["i8", "u8", "i16", "u16"];
        let source = types.iter().map(|ty| format!("pub fn return_{ty}(x:u32)->{ty}{{x as {ty}}}\n")).collect::<String>();
        let expected = types.iter().map(|ty| format!("return_{ty}")).collect();
        assert_eq!(admitted(&source), expected);
    }

    #[test]
    fn unrepresented_source_bindings_do_not_enter_native_units() {
        let source = "pub fn wide(x:u128)->u128{x ^ 17}\n            pub fn borrowed(x:&u32)->u32{*x}\n            pub fn generic<T>(x:T)->T{x}\n            pub fn nested(x:u64)->u64{fn kernel(x:u64)->u64{x ^ 17} kernel(x)}\n            #[allow(dead_code)] pub mod attributed {pub fn kernel(x:u64)->u64{x ^ 17}}\n            pub mod arithmetic {pub fn kernel(x:u64)->u64{x ^ 17}}\n            pub fn calls(x:u64)->u64{arithmetic::kernel(x)}\n";
        assert_eq!(admitted(source), ["arithmetic::kernel"].into_iter().map(str::to_owned).collect());
    }

    #[test]
    fn identifier_normalization_is_an_explicit_native_source_capability() {
        let source = "pub mod names {\n            pub fn decomposed(e\u{301}:u64)->u64{e\u{301} ^ 17}\n            pub fn local(x:u64)->u64{let e\u{301}=x;e\u{301} ^ 17}\n            pub fn canonical(é:u64)->u64{é ^ 17}\n            pub fn comments(x:u64)->u64{/* e\u{301} */x ^ 17}\n            pub fn r#type(r#match:u32)->u32{r#match ^ 17}\n}
";
        assert_eq!(admitted(source), ["names::canonical", "names::comments", "names::r#type"].into_iter().map(str::to_owned).collect());
    }

    #[test]
    fn same_named_module_functions_keep_distinct_checked_identities() {
        let source = "pub fn kernel(x:u64)->u64{x ^ 17}\n            pub mod first { pub fn kernel(x:u64)->u64{x ^ 19} }\n            pub mod second { pub mod nested { pub fn kernel(x:u64)->u64{x ^ 23} } }\n";
        assert_eq!(admitted(source), ["kernel", "first::kernel", "second::nested::kernel"].into_iter().map(str::to_owned).collect());
    }
}
