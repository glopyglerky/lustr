//! Experimental, fail-closed constraint elaboration through native phase APIs.
//! The source stays unchanged. A complete native collect/check/borrowck/codegen
//! follows elaboration; no checked result or diagnostic is treated as acceptance.
use crate::{ast, check, def, diag::Span, driver, intern::Sym, ty};
use std::sync::Arc;
use ty::TyIntern;

mod arena;
mod assert_text;

/// Reuse the existing original-AST display derivation in a fresh native unit.
/// Only assertion text changes; the caller must run complete native checks.
pub(crate) fn original_assertion_text(ast: &ast::Ast, map: &crate::diag::SourceMap, text: &str, interner: &crate::intern::Interner, known: &crate::intern::Known) -> Option<ast::Ast> {
    assert_text::proposal(ast,map,text,interner,known)
}
mod assertions;
mod host_inputs;
mod reuse;
mod scratch;
mod shared;
mod phase;

pub(super) fn session_compile(req: &driver::Request) -> driver::Response {
    shared::compile(req)
}

struct IntoSite {
    expr: ast::ExprId,
    source: Source,
    target: Sym,
}

#[derive(Clone, Copy)]
enum Source {
    Str,
    String,
    OsString(Sym),
    PathBuf(Sym),
    Path(Sym),
    OsStr(Sym),
}

fn platform_adt(p: &check::collect::Program, tys: &impl TyIntern, t: ty::Ty) -> Option<Sym> {
    let (def, args) = tys.adt_parts(t)?;
    if !tys.args(args).is_empty() {
        return None;
    }
    let adt = p.adt(tys.local_adt_of(def)?);
    let tables = p.resolved.as_ref()?;
    let (platform, _) = tables.std_platform?;
    let module = tables.def_info.get(&def)?.module;
    if tables.root_of(module) != platform {
        return None;
    }
    Some(adt.name)
}

fn platform_owned_type(p: &check::collect::Program, tys: &impl TyIntern, t: ty::Ty) -> Option<(Sym, Source)> {
    let name = platform_adt(p, tys, t)?;
    let source = match p.display_path(name).as_str() {
        "std::ffi::OsString" => Source::OsString(name),
        "std::path::PathBuf" => Source::PathBuf(name),
        _ => return None,
    };
    Some((name, source))
}

fn owned_conversion_target(p: &check::collect::Program, tys: &impl TyIntern, t: ty::Ty) -> Option<Sym> {
    platform_owned_type(p, tys, t).map(|(name, _)| name)
}

fn source(p: &check::collect::Program, ck: &check::Checked<'_>, e: ast::ExprId) -> Option<Source> {
    match ck.tys.kind(ck.ty(e)) {
        ty::TyKind::Ref(_, ty::Mut::Shared, t) => {
            if ck.tys.kind(t) == ty::TyKind::Str {
                return Some(Source::Str);
            }
            if let Some(name) = platform_adt(p, &ck.tys, t) {
                match p.display_path(name).as_str() {
                    "std::path::Path" => return Some(Source::Path(name)),
                    "std::ffi::OsStr" => return Some(Source::OsStr(name)),
                    _ => (),
                }
            }
        }
        _ if ck.tys.is_string_ty(ck.ty(e)) => return Some(Source::String),
        _ => (),
    }
    // Only a checked, nongeneric ADT in the actual platform root supplies
    // these owned source types. Parsed spellings and user lookalikes refuse.
    if let Some((_, source)) = platform_owned_type(p, &ck.tys, ck.ty(e)) {
        return Some(source);
    }
    // A rolled-back argument trial leaves no side-table type. These two
    // actual parsed native nodes have fixed types; all other forms refuse.
    match &p.ast.exprs[e.0 as usize] {
        ast::Expr::Str(_) => Some(Source::Str),
        ast::Expr::Macro { kind: ast::MacroKind::Format, .. } => Some(Source::String),
        _ => None,
    }
}

fn site(p: &check::collect::Program, ck: &check::Checked<'_>, e: ast::ExprId, target: Sym) -> Option<IntoSite> {
    let ast::Expr::MethodCall { recv, name, generic_args, args, .. } = &p.ast.exprs[e.0 as usize] else {
        return None;
    };
    if p.name(*name) != "into" || !generic_args.is_empty() || !args.is_empty() {
        return None;
    }
    Some(IntoSite { expr: e, source: source(p, ck, *recv)?, target })
}

fn type_node(ast: &mut ast::Ast, node: ast::TyExpr, span: Span) -> ast::TyId {
    let id = ast::TyId(ast.tys.len().try_into().unwrap());
    ast.tys.push(node);
    ast.ty_spans.push(span);
    id
}

fn path(ast: &mut ast::Ast, names: &[Sym], span: Span) -> ast::TyId {
    type_node(ast, ast::TyExpr::Path(names.iter().copied().map(ast::PathSeg::plain).collect()), span)
}

fn elaborate(ast: &mut ast::Ast, symbols: &crate::intern::Interner, sites: &[IntoSite], assertions: &[assertions::Site]) {
    let mut additions = Vec::new();
    for site in sites {
        let span = ast.expr_spans[site.expr.0 as usize];
        let target = path(ast, &[site.target], span);
        let src = match site.source {
            Source::Str => {
                let inner = path(ast, &[symbols.intern("str")], span);
                type_node(ast, ast::TyExpr::Ref { lifetime: None, mutable: false, inner }, span)
            }
            Source::String => path(ast, &[symbols.intern("std"), symbols.intern("string"), symbols.intern("String")], span),
            Source::OsString(name) | Source::PathBuf(name) => path(ast, &[name], span),
            Source::Path(name) | Source::OsStr(name) => {
                let inner = path(ast, &[name], span);
                type_node(ast, ast::TyExpr::Ref { lifetime: None, mutable: false, inner }, span)
            }
        };
        additions.push((
            site.expr,
            arena::Addition::Call(ast::Expr::QPath {
                self_ty: src,
                trait_path: vec![
                    ast::PathSeg::plain(symbols.intern("std")),
                    ast::PathSeg::plain(symbols.intern("convert")),
                    ast::PathSeg { name: symbols.intern("Into"), args: ast::GenericArgs::Angle(vec![ast::GenericArg::Type(target)]) },
                ],
                segs: vec![ast::PathSeg::plain(symbols.intern("into"))],
            }),
        ));
    }
    assertions::elaborate(ast, symbols, assertions, &mut additions);
    additions.sort_by_key(|(e, _)| e.0);
    arena::elaborate(ast, additions);
}

pub(super) fn compile(req: &driver::Request) -> driver::Response {
    compile_with_cache(req, None)
}

pub(super) fn retained_compile(req: &driver::Request) -> driver::Response {
    // This separate entry is bounded and serialized. The original one-shot
    // ABI and the product's existing native success path keep their contract.
    static CACHE: std::sync::Mutex<reuse::Cache> = std::sync::Mutex::new(reuse::Cache::new());
    let mut cache = CACHE.lock().unwrap();
    if !cache.runtime_valid() {
        return driver::Response {
            code: 2,
            stderr: "error: native retained runtime is unsealed or changed; use a fresh compiler process\n".into(),
            ..Default::default()
        };
    }
    if let Some(response) = cache.lookup(req) {
        return response;
    }
    compile_with_cache(req, Some(&mut cache))
}

fn compile_with_cache(req: &driver::Request, mut cache: Option<&mut reuse::Cache>) -> driver::Response {
    let before = cache.as_ref().and_then(|_| reuse::Before::read(req));
    // Start with an unchanged native Check that retains its parsed/collected
    // state. A rejected check supplies inference evidence directly; a passing
    // check still needs the requested full Build before acceptance. The primary
    // product ABI remains separate and tries ordinary Lust before this worker.
    // Own the directory before Session construction, including panic paths
    // caught by the outer ABI. The one-shot session leak policy is separate.
    let scratch = match scratch::Scratch::new(&std::env::temp_dir()) {
        Ok(scratch) => scratch,
        Err(error) => {
            return driver::Response {
                code: 2,
                stderr: format!("error: cannot create native inference scratch directory: {error}\n"),
                ..Default::default()
            };
        }
    };
    let mut session = driver::Session::new(scratch.path().to_owned(), true, req.session.clone());
    if cache.is_some() {
        // A retained artifact proof must observe this request's native C
        // compilation, including the runtime unit. The inherited disk unit
        // cache has no complete host-header/link-input key. Do not seed its
        // slot/definition manifests or accept its objects before our Check.
        session.cache_tried = true;
        session.disk_cache = None;
    }
    let host_observer = before.as_ref().and_then(|_| host_inputs::Observer::begin());
    // Build has original refusal guards that precede reading/checking source.
    // A Check must never replace those answers. The ABI already refuses cc;
    // foreign emit-C requests remain the original compiler's supported route.
    let triple = &req.session.target_triple;
    let original_build_guard = req.cmd == driver::Cmd::Build
        && ((req.session.krate.target_kind == "lib" && !req.harness)
            || (!triple.is_empty() && triple != crate::protocol::host_triple() && req.emit_c.is_none()));
    if original_build_guard {
        session.caches.persistent = false;
        let response = driver::compile(req, &mut session);
        if cache.is_none() {
            std::mem::forget(session);
        }
        return response;
    }
    let mut checking = req.clone();
    checking.cmd = driver::Cmd::Check;
    let first = driver::compile(&checking, &mut session);
    let retry = matches!(first.code, 0 | 1).then(|| retry_request(req, &session)).flatten();
    let before = before.and_then(|mut b| retry.as_ref().filter(|r| b.observed(&session, r)).map(|_| b));
    let mut seeded = false;
    let mut result = if first.code == 0 && req.cmd != driver::Cmd::Check {
        if retry.is_some() {
            seeded = assert_text::seed(&mut session);
        }
        session.caches.persistent = false;
        driver::compile(req, &mut session)
    } else if let Some(retry) = retry {
        let mut response = first;
        // A rejected conversion can hide later checked constraints in the
        // same function. Recheck after elaboration before inspecting those
        // constraints; never guess their types from syntax. Two rounds bound
        // this owned expansion, and unresolved programs remain failures.
        for _ in 0..2 {
            let inferred = seed(&mut session);
            let text = assert_text::seed(&mut session);
            if !inferred && !text {
                break;
            }
            seeded = true;
            // Force standalone native linking, with no transient daemon base.
            session.caches.persistent = false;
            response = driver::compile(&retry, &mut session);
            if response.code != 1 {
                break;
            }
        }
        response
    } else {
        first
    };
    // The one-shot entry exits after the request; avoid detached cache drops.
    let host = if seeded && result.code == 0 {
        match host_observer.map(|observer| observer.finish(&session, req)) {
            Some(Ok(proof)) => proof,
            Some(Err(failure)) => {
                result = failure;
                None
            }
            None => None,
        }
    } else {
        None
    };
    if seeded && let (Some(cache), Some(before)) = (cache.as_mut(), before) {
        cache.store(req, before, &result, host);
    }
    if cache.is_none() {
        std::mem::forget(session);
    }
    result
}

fn retry_request(req: &driver::Request, session: &driver::Session) -> Option<driver::Request> {
    if session.front.patch.is_none() || session.front.env_hash.is_none() {
        return None;
    }
    let deps: Vec<_> = session.front.chunks.iter().flat_map(|c| c.expand_deps.iter().cloned()).collect();
    validated_retry(req, &session.front.interner, session.front.client.clone(), &deps)
}

fn validated_retry(
    req: &driver::Request,
    interner: &crate::intern::Interner,
    mut client: crate::expand::ClientView,
    deps: &[crate::expand::Dep],
) -> Option<driver::Request> {
    use crate::expand::{ClientView, Dep};
    let mut retry = req.clone();
    if deps.iter().any(|d| matches!(d, Dep::Position)) {
        return None;
    }
    // The upstream macro API maps a non-Unicode value to absence, whereas
    // rustc rejects a read of it. Never elaborate that mismatching input.
    if req.client.env.is_none()
        && deps.iter().any(|d| {
            matches!(d, Dep::Env { name, .. }
            if matches!(std::env::var(name), Err(std::env::VarError::NotUnicode(_))))
        })
    {
        return None;
    }
    if let Some(manifest) = &req.session.krate.manifest_path {
        use crate::resolve::{cargo, cfg::CfgEnv};
        let key = &req.session;
        let graph_req = cargo::GraphRequest {
            manifest_path: manifest.clone(),
            package: None,
            features: key.features.clone(),
            default_features: key.default_features,
            all_features: key.all_features,
            target_triple: key.target_triple.clone(),
            cargo_net: key.cargo_net,
            env: req.client.env.as_deref().map(cargo::cargo_env_of),
        };
        let graph = cargo::load_graph(&graph_req).ok()?;
        let cfg = CfgEnv::new(&key.target_triple, &key.features, key.krate.test, key.profile);
        let unit =
            cargo::select_unit(&graph, &graph_req, &key.krate.target_kind, &key.krate.target_name, &cfg, interner, key.krate.test).ok()?;
        // The current bridge admits one package's bin/lib closure, with no
        // build script. Cross-package and build-script env need scoped proof.
        if unit.package.targets.iter().any(|t| t.kind.iter().any(|k| k == "custom-build")) {
            return None;
        }
        fn same_package<'a>(deps: &'a [cargo::DepUnit], package: &str, envs: &mut Vec<&'a [(String, String)]>) -> bool {
            for d in deps {
                if d.package_id != package || d.source.is_some() {
                    return false;
                }
                envs.push(&d.cargo_env);
                if !same_package(&d.deps, package, envs) {
                    return false;
                }
            }
            true
        }
        let mut envs = Vec::new();
        if !same_package(&unit.deps, &unit.package_id, &mut envs) {
            return None;
        }
        // Keep the caller's complete Unicode env view; non-Unicode macro
        // reads were refused above. No process env mutates.
        let mut env = req
            .client
            .env
            .clone()
            .unwrap_or_else(|| std::env::vars_os().filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?))).collect());
        for (name, value) in &unit.cargo_env {
            if !envs.iter().all(|e| e.iter().any(|(n, v)| n == name && v == value)) {
                continue;
            }
            let reads: Vec<_> = deps
                .iter()
                .filter_map(|d| match d {
                    Dep::Env { name: n, value } if n == name => Some(value),
                    _ => None,
                })
                .collect();
            if reads.is_empty() || !reads.iter().all(|v| v.as_deref() == Some(value.as_str())) {
                continue;
            }
            // Only facts actually read under the same Cargo value in every
            // observed scope can populate the validation view.
            env.retain(|(n, _)| n != name);
            env.push((name.clone(), value.clone()));
        }
        client = ClientView { env: Some(Arc::new(env.clone())), shown_root: client.shown_root };
        retry.client.env = Some(env);
    }
    // Run the real native checks against every recorded file and env value.
    // A stale input refuses elaboration; no read or failure is discarded.
    (!crate::expand::deps_stale(deps, &client)).then_some(retry)
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::expand::{ClientView, Dep};
    use std::hash::{Hash, Hasher};

    #[test]
    fn real_dependency_validator_refuses_stale_files_env_and_positions() {
        let scratch = scratch::Scratch::new(&std::env::temp_dir()).unwrap();
        let source = scratch.path().join("main.rs");
        std::fs::write(&source, "fn main() {}\n").unwrap();
        let args = vec!["build".to_owned(), source.to_str().unwrap().to_owned()];
        let Some(crate::protocol::Invocation::Compile { mut req, .. }) = crate::protocol::parse_args(&args) else { panic!("request") };
        req.client.env = Some(vec![("LUSTR_RECORDED".into(), "old".into())]);
        let mut client = ClientView { env: Some(Arc::new(req.client.env.clone().unwrap())), shown_root: None };
        let input = scratch.path().join("include.txt");
        let bytes = b"original included bytes";
        std::fs::write(&input, bytes).unwrap();
        let mut hash = crate::intern::FxHasher::default();
        bytes.as_slice().hash(&mut hash);
        let deps = vec![
            Dep::File { path: input.clone(), hash: hash.finish() },
            Dep::Env { name: "LUSTR_RECORDED".into(), value: Some("old".into()) },
            Dep::Env { name: "LUSTR_UNSET".into(), value: None },
            Dep::Shown { path: source.to_str().unwrap().into(), shown: source.to_str().unwrap().into() },
        ];
        let interner = crate::intern::Interner::default();
        let validate = |client: ClientView, deps: &[Dep]| validated_retry(&req, &interner, client, deps).is_some();
        assert!(validate(client.clone(), &deps));
        std::fs::write(&input, b"changed included bytes").unwrap();
        assert!(!validate(client.clone(), &deps));
        std::fs::write(&input, bytes).unwrap();
        assert!(validate(client.clone(), &deps));
        client.env = Some(Arc::new(vec![("LUSTR_RECORDED".into(), "new".into())]));
        assert!(!validate(client.clone(), &deps));
        client.env = Some(Arc::new(vec![("LUSTR_RECORDED".into(), "old".into()), ("LUSTR_UNSET".into(), "now-set".into())]));
        assert!(!validate(client.clone(), &deps));
        client.env = Some(Arc::new(req.client.env.clone().unwrap()));
        let mut positioned = deps.clone();
        positioned.push(Dep::Position);
        assert!(!validate(client.clone(), &positioned));
        client.shown_root = Some((scratch.path().to_owned(), "different-root".into()));
        assert!(!validate(client, &deps));
        assert_eq!(req.client.env.as_ref().unwrap(), &[("LUSTR_RECORDED".to_owned(), "old".to_owned())]);
        std::fs::remove_file(input).unwrap();
        assert!(!validate(ClientView { env: Some(Arc::new(req.client.env.clone().unwrap())), shown_root: None }, &deps));
    }
}

fn seed_blocked(p: &check::collect::Program) -> bool {
    // A competing source trait named `into` makes method lookup a separate
    // obligation. This experiment does not resolve that case.
    let competing_into = p.ast.items.iter().chain(&p.ast.stmt_items).any(|item| {
        !p.std_source_span(item.span)
            && matches!(&item.kind, ast::ItemKind::Trait(t) if t.fns.iter().any(|f| matches!(p.name(f.name), "into" | "extend")))
    });
    competing_into
}

fn seed_custom_extend(p: &check::collect::Program) -> bool {
    // Source-written Extend implementations could add another item solution.
    // Refuse the entire rule in their presence, even for unrelated receivers.
    p.gx.index.impls_of_trait(p.gx.lang.extend).iter().any(|&id| {
        let r = p.gx.index.get(id);
        r.local && r.synthetic == crate::traits::index::Synthetic::None && !p.std_source_span(r.span)
    })
}

fn seed_eligible(p: &check::collect::Program, f: def::LocalDefId) -> bool {
    let i = f.0 as usize;
    !(p.gx.fn_class.get(i) != Some(&check::generics::FnClass::Plain)
            || p.check_skips(f)
            || p.fn_shipped_crate(f).is_some()
            || p.std_source_span(p.fn_def(f).name_span)
            || p.fn_def(f).body.is_none())
}

#[derive(Default)]
struct SeedSites { sites: Vec<IntoSite>, assertions: Vec<assertions::Site> }
impl SeedSites {
    fn record(&mut self, p: &check::collect::Program, f: def::LocalDefId, ck: &check::Checked<'_>, errors: &[crate::diag::Diag], custom_extend: bool) {
        if !seed_eligible(p, f) { return; }
        if !errors.iter().any(|e| matches!(e.code, "L0610" | "L0640" | "L0110")) {
            return;
        }
        for j in 0..ck.expr_ty.len() {
            let e = ast::ExprId(ck.expr_base + j as u32);
            if let ast::Expr::Macro { args, .. } = &p.ast.exprs[e.0 as usize]
                && let Some(right) = args.get(1)
                && errors.iter().any(|d| d.code == "L0110" && d.span == p.ast.expr_spans[right.0 as usize])
                && let Some(array) = assertions::site(p, ck, e, p.fn_def(f))
            {
                self.assertions.push(array);
            }
            let ast::Expr::MethodCall { recv, name, name_span, args, generic_args } = &p.ast.exprs[e.0 as usize] else {
                continue;
            };
            if p.name(*name) == "into"
                && errors
                    .iter()
                    .any(|d| d.code == "L0640" && d.span == *name_span && d.message.contains("a target a later statement fixes"))
            {
                if let Some(target) = owned_conversion_target(p, &ck.tys, ck.ty(e)) {
                    if let Some(s) = site(p, ck, e, target) {
                        self.sites.push(s);
                    }
                }
            }
            if custom_extend || p.name(*name) != "extend" || !generic_args.is_empty() {
                continue;
            }
            let [arg] = args.as_slice() else {
                continue;
            };
            if !errors.iter().any(|d| d.code == "L0610" && d.span == p.ast.expr_spans[arg.0 as usize]) {
                continue;
            }
            let Some(elem) = ck.tys.vec_elem(ck.ty(*recv)) else {
                continue;
            };
            let Some(target) = owned_conversion_target(p, &ck.tys, elem) else {
                continue;
            };
            if check::collect::is_copy(p, &ck.tys, elem) {
                continue;
            }
            let ast::Expr::Array(items) = &p.ast.exprs[arg.0 as usize] else {
                continue;
            };
            if items.is_empty() {
                continue;
            }
            let proposed: Option<Vec<_>> = items.iter().map(|&x| site(p, ck, x, target)).collect();
            if let Some(proposed) = proposed {
                self.sites.extend(proposed);
            }
        }
    }

    fn finish(mut self, p: &check::collect::Program, input: &ast::Ast) -> Option<(ast::Ast, usize, usize)> {
        self.sites.sort_by_key(|s| s.expr.0);
        self.sites.dedup_by_key(|s| s.expr.0);
        self.assertions.sort_by_key(|e| e.expr().0);
        self.assertions.dedup_by_key(|e| e.expr().0);
        if self.sites.is_empty() && self.assertions.is_empty() { return None; }
        let mut ast = (*p.ast).clone();
        if !crate::check::collect::restore_original_unsafe(&mut ast, input) { return None; }
        elaborate(&mut ast, &p.interner, &self.sites, &self.assertions);
        Some((ast, self.sites.len(), self.assertions.len()))
    }
}

fn apply_seed(session: &mut driver::Session, plan: (ast::Ast, usize, usize)) {
    let (ast, sites, assertions) = plan;
    eprintln!("native inference: {sites} constraint-derived conversion sites; {assertions} checked assertion unsizings; full native recheck follows");
    session.front.ast = Arc::new(ast);
    session.front.kept = [None, None];
    session.front.last_check = None;
    session.caches.ok.fill(false);
    session.caches.has_out.fill(false);
    session.caches.gen_reuse.clear();
}

fn seed(session: &mut driver::Session) -> bool {
    let Some(kept) = session.front.kept.iter().flatten().next() else { return false; };
    let p = &kept.program;
    if seed_blocked(p) { return false; }
    let custom_extend = seed_custom_extend(p);
    let mut sites = SeedSites::default();
    for i in 0..p.fns.len() {
        let f = def::LocalDefId(i.try_into().unwrap());
        if !seed_eligible(p, f) { continue; }
        let (ck, errors) = check::check_fn(p, f);
        sites.record(p, f, &ck, &errors, custom_extend);
    }
    let Some(plan) = sites.finish(p, &session.front.ast) else { return false; };
    apply_seed(session, plan);
    true
}
