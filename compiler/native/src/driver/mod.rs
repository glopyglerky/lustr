// The driver: session state (session.rs), the whole-request disk cache (cache.rs),
// and the compile pipeline (compile.rs). Callers use the re-exports below, so the
// public paths (`driver::Session`, `driver::compile`, `driver::cache::Cache`) stay
// stable when items move between these files.
pub(crate) mod ast_hash;
mod ir_cache;
pub mod buildscript;
pub mod cache;
mod compile;
mod const_reach;
pub(crate) mod kept_delta;
mod kept_derive;
pub(crate) mod laps;
mod genreuse_oracle;
mod item_oracle;
mod resolve_oracle;
mod macro_scope;
mod item_outline;
pub(crate) mod gen_reuse;
pub mod graph;
mod patch;
pub(crate) mod sig_nodes;
mod span_shift;
mod session;
#[path = "../../../../crates/lustr-native-core/src/mixed_unit.rs"]
mod mixed;
pub(crate) mod release_c;
pub mod read_cover;

pub use crate::protocol::{Cmd, Request, Response, SessionKey};
pub use compile::{Stamps, defer_cc, drop_parked_cc, finish_parked_cc, parked_cc_result, release_bases, sweep_bases};
pub(crate) use item_oracle::failure as oracle_failure;
pub(crate) use patch::drop_unkept;

/// Compiles one request in `session` (`compile::compile`). Under the
/// patch oracle (`item_oracle.rs`) a daemon check request then compares
/// its answer with a fresh session's.
pub fn compile(req: &Request, session: &mut Session) -> Response {
    compile_observed(req, session, None)
}

/// An optional, call-borrowed view of the actual body-checking phase. Early
/// parse/collect failures and cached check answers produce no phase events.
/// Observations do not grant acceptance or establish cross-request reuse.
pub fn compile_observed(req: &Request, session: &mut Session, observer: Option<&dyn PhaseObserver>) -> Response {
    let resp = compile::compile_observed(req, session, observer);
    if item_oracle::on() {
        item_oracle::compare_response(req, session, &resp);
    }
    resp
}

/// `compile` without the oracle's response compare, for the daemon's
/// background guard checks: their file can change while they run, so a
/// second read of it may see other bytes.
pub fn compile_background(req: &Request, session: &mut Session) -> Response {
    let resp = compile::compile(req, session);
    if item_oracle::on() {
        item_oracle::forget(session);
    }
    resp
}
pub use session::{OutputKind, Session};

/// Process-local identity of one observed driver invocation, not crate metadata
/// or a semantic fingerprint. Allocated only when an observer is requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhaseId(u64);
impl PhaseId {
    fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1)).expect("native phase identity exhausted"))
    }
}

/// All references are borrowed from the current invocation. `program.resolved`
/// is its actual resolver table generation; neither it nor the input AST is a
/// stable on-disk identity. No retained preferred program is consulted here.
#[derive(Clone, Copy)]
pub struct Prepared<'a> {
    pub id: PhaseId,
    pub request: &'a Request,
    pub input: &'a std::sync::Arc<crate::ast::Ast>,
    pub program: &'a crate::check::collect::Program,
    pub map: &'a crate::diag::SourceMap,
    /// Root text used when the single-file path has no SourceMap entry.
    pub source: &'a str,
    /// Functions actually checked now; others may have qualified native reuse.
    pub work: &'a [usize],
}

/// Callbacks receive shared phase borrows. Body callbacks run on native workers,
/// before Checked is consumed by lowering and before IR is discarded. Their
/// order is unspecified; aggregate diagnostics run after all workers settle,
/// before rendering/drop/keep_err. There is no synthesized Checked for reused
/// functions and no change to the driver's worklist or error decisions.
pub trait PhaseObserver: Sync {
    fn prepared(&self, _phase: Prepared<'_>) {}
    fn checked(&self, _phase: Prepared<'_>, _function: crate::def::LocalDefId, _checked: &crate::check::Checked<'_>, _diagnostics: &[crate::diag::Diag]) {}
    fn lowered(&self, _phase: Prepared<'_>, _function: crate::def::LocalDefId, _body: &crate::ir::Body<'_>, _diagnostics: &[crate::diag::Diag]) {}
    fn diagnostics(&self, _phase: Prepared<'_>, _diagnostics: &[crate::diag::Diag]) {}
}
