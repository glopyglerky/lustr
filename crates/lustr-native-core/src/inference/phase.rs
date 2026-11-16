//! Consume actual call-borrowed checking results, never a preferred old program.
//! Only owned AST proposals and weak input identities survive the callbacks.
//! The V59 fresh-collect proof remains required; warm-only elaboration is held.
use super::{SeedSites, apply_seed, seed_blocked, seed_custom_extend};
use crate::{ast, check, def, diag, driver};
use std::sync::{Arc, Mutex, Weak};

#[derive(Default)]
struct Observed {
    id: Option<driver::PhaseId>,
    input: Weak<ast::Ast>,
    fresh: bool,
    complete: bool,
    blocked: bool,
    custom_extend: bool,
    sites: SeedSites,
    plan: Option<(ast::Ast, usize, usize)>,
}

#[derive(Default)]
pub(super) struct Observer(Mutex<Observed>);

impl driver::PhaseObserver for Observer {
    fn prepared(&self, phase: driver::Prepared<'_>) {
        let mut state = self.0.lock().unwrap();
        assert!(state.id.is_none(), "one observer belongs to one native invocation");
        state.id = Some(phase.id);
        state.input = Arc::downgrade(phase.input);
        state.fresh = check::collect::provenance::current(phase.input, phase.program);
        // Preserve the conservative boundary until warm checked generations
        // and every required input/reuse transition independently qualify.
        if state.fresh {
            state.blocked = seed_blocked(phase.program);
            state.custom_extend = seed_custom_extend(phase.program);
        }
    }

    fn checked(&self, phase: driver::Prepared<'_>, function: def::LocalDefId, checked: &check::Checked<'_>, diagnostics: &[diag::Diag]) {
        if diagnostics.is_empty() { return; }
        let mut state = self.0.lock().unwrap();
        assert_eq!(state.id, Some(phase.id));
        if !state.fresh || state.blocked { return; }
        let custom_extend = state.custom_extend;
        state.sites.record(phase.program, function, checked, diagnostics, custom_extend);
    }

    fn diagnostics(&self, phase: driver::Prepared<'_>, diagnostics: &[diag::Diag]) {
        let mut state = self.0.lock().unwrap();
        assert_eq!(state.id, Some(phase.id));
        state.complete = true;
        if state.fresh && !diagnostics.is_empty() {
            // Build an owned proposal while this exact Program is alive.
            // keep_err may later retain an entirely different Program.
            state.plan = std::mem::take(&mut state.sites).finish(phase.program, phase.input);
        }
    }
}

impl Observer {
    pub(super) fn fresh_for(&self, session: &driver::Session) -> bool {
        let state = self.0.lock().unwrap();
        state.complete && state.fresh
            && state.input.upgrade().is_some_and(|input| Arc::ptr_eq(&input, &session.front.ast))
    }

    pub(super) fn apply(&self, session: &mut driver::Session) -> bool {
        if !self.fresh_for(session) { return false; }
        let Some(plan) = self.0.lock().unwrap().plan.take() else { return false; };
        apply_seed(session, plan);
        true
    }
}
