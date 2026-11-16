//! A native driver session shared by Lust and the owned complement.
//!
//! This experimental entry retains the original resolver, AST and semantic
//! state, rather than retaining an executable as the answer to another build.
//! Every request enters the actual requested native phase first. Elaboration
//! uses that same session and still requires the complete native recheck.
use super::{assert_text, phase::Observer, retry_request, scratch::Scratch};
use crate::{driver, protocol};
use std::{cell::RefCell, ffi::OsString, path::PathBuf};

#[derive(PartialEq, Eq)]
struct Context {
    request: Vec<u8>,
    environment: Vec<(OsString, OsString)>,
    directory: PathBuf,
    graph: Option<String>,
    bindings: Vec<PathBuf>,
}

impl Context {
    fn read(req: &driver::Request) -> Option<Self> {
        let mut environment: Vec<_> = std::env::vars_os().collect();
        environment.sort();
        let graph = if let Some(manifest) = &req.session.krate.manifest_path {
            let key = &req.session;
            let graph_request = crate::resolve::cargo::GraphRequest {
                manifest_path: manifest.clone(), package: None,
                features: key.features.clone(), default_features: key.default_features,
                all_features: key.all_features, target_triple: key.target_triple.clone(),
                cargo_net: key.cargo_net,
                env: req.client.env.as_deref().map(crate::resolve::cargo::cargo_env_of),
            };
            Some(format!("{:?}", crate::resolve::cargo::load_graph(&graph_request).ok()?))
        } else { None };
        let mut bindings = vec![req.file.canonicalize().ok()?, req.session.krate.root.canonicalize().ok()?];
        if let Some(manifest) = &req.session.krate.manifest_path { bindings.push(manifest.canonicalize().ok()?); }
        Some(Self {
            request: protocol::encode(req),
            environment,
            directory: std::env::current_dir().ok()?,
            graph,
            bindings,
        })
    }
}

struct Active {
    // Drop/settle the native state before removing its generated files.
    session: driver::Session,
    _scratch: Scratch,
    context: Context,
}

impl Drop for Active {
    fn drop(&mut self) {
        self.session.settle();
    }
}

#[derive(Default)]
struct State {
    runtime: super::reuse::Cache,
    active: Option<Active>,
    retired: bool,
}

thread_local! {
    // Session contains native thread-owned state. It is never moved through a
    // raw handle, declared Send, or shared between compiler calling threads.
    // Keep the large native Session on the heap. Inline TLS would require
    // 36 KiB of initial-exec TLS and prevent the DSO loading normally.
    static STATE: RefCell<Box<State>> = RefCell::new(Box::default());
}

fn failure(message: &str) -> driver::Response {
    driver::Response { code: 2, stderr: format!("error: {message}\n"), ..Default::default() }
}

pub(super) fn compile(req: &driver::Request) -> driver::Response {
    if req.cc || req.emit_c.is_some() || req.harness {
        return failure("native shared session does not yet implement release, emitted-C or test-harness requests");
    }
    if crate::driver::release_c::on(false, false) {
        return failure("native shared session has not qualified the requested release-C configuration");
    }
    STATE.with(|slot| {
        let Ok(mut state) = slot.try_borrow_mut() else {
            return failure("native session entry is not reentrant");
        };
        if state.retired || !state.runtime.runtime_valid() {
            state.active = None;
            state.retired = true;
            return failure("native session runtime changed; use a fresh compiler process");
        }
        let Some(context) = Context::read(req) else {
            state.active = None;
            return failure("cannot observe native session context");
        };
        if state.active.as_ref().is_some_and(|active| active.context != context) {
            state.active = None;
        }
        if state.active.is_none() {
            let scratch = match Scratch::new(&std::env::temp_dir()) {
                Ok(scratch) => scratch,
                Err(_) => return failure("cannot own native session scratch directory"),
            };
            let mut session = driver::Session::new(scratch.path().to_owned(), true, req.session.clone());
            // No unproved disk objects or manifests enter the session. The
            // existing resolver checks actual source/macro reads on edits.
            session.cache_tried = true;
            session.disk_cache = None;
            session.set_output(driver::OutputKind::Standalone);
            state.active = Some(Active { session, _scratch: scratch, context });
        }
        // A panic retires this thread's state even though the outer ABI catches
        // it. No partly mutated native session may answer a later request.
        state.retired = true;
        let active = state.active.as_mut().unwrap();
        let session = &mut active.session;
        // Retain semantic read sets while the explicit native output plan
        // emits a standalone executable directly. No intermediate image link
        // or second final link is needed.
        // Regenerate every C unit's object so host-header/link changes cannot
        // be hidden by in-memory or disk object reuse. Generated C and native
        // semantic state remain governed by the driver's edit/read checks.
        session.caches.persistent = true;
        session.caches.objects = None;
        let _phase = crate::check::collect::provenance::begin();
        let mut observed = Observer::default();
        let mut response = driver::compile_observed(req, session, Some(&observed));
        let mut elaborated = false;
        if response.code == 1 && let Some(retry) = retry_request(req, session) {
            // Rejection is retained as a rejection until the native complement
            // supplies a justified AST elaboration and the full driver passes.
            for _ in 0..2 {
                // The native callback consumed the CURRENT checked results
                // before keep_err could retain an older preferred program.
                // Keep V59 fresh-collect/input/resolver proof and warm refusal.
                if !observed.fresh_for(session) { break; }
                let inferred = observed.apply(session);
                let text = assert_text::seed(session);
                if !inferred && !text { break; }
                elaborated = true;
                session.caches.objects = None;
                crate::check::collect::provenance::clear();
                observed = Observer::default();
                response = driver::compile_observed(&retry, session, Some(&observed));
                if response.code != 1 { break; }
            }
        }
        // An elaborated AST has no cross-request equivalence proof. Keep
        // original native sessions across edits, but retire transformed state
        // after this request, on success as well as failure.
        if elaborated || response.code > 1 {
            state.active = None;
        }
        state.retired = false;
        response
    })
}
