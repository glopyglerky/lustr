//! Current-call collection evidence for the experimental shared session.
//! Weak identities neither keep an old program alive nor certify an incremental
//! mutation. Only an actual completed collect in this native phase records one.
use super::Program;
use crate::{ast::Ast, def::{CrateId, StableCrateHash}, resolve::imports::ResolvedTables};
use std::sync::{Arc, Weak, Mutex, atomic::{AtomicBool, Ordering}};

struct Proof {
    input: Weak<Ast>,
    collected: Weak<Ast>,
    resolved: Option<Weak<ResolvedTables>>,
    krate: CrateId,
    stable: StableCrateHash,
}
struct State { owner: Option<std::thread::ThreadId>, proof: Option<Proof> }
// The native entry lock serializes phase lifetimes. Bind observations to the
// actual collecting thread as well. No extra initial-exec TLS budget is needed.
static STATE: Mutex<State> = Mutex::new(State { owner: None, proof: None });
static ENABLED: AtomicBool = AtomicBool::new(false);

pub(crate) struct Phase;
impl Drop for Phase {
    fn drop(&mut self) {
        ENABLED.store(false,Ordering::Release);
        let mut s=STATE.lock().unwrap_or_else(|e|e.into_inner());s.owner=None;s.proof=None;
    }
}
pub(crate) fn begin() -> Phase {
    let mut s=STATE.lock().unwrap_or_else(|e|e.into_inner());assert!(s.owner.is_none());s.owner=Some(std::thread::current().id());s.proof=None;
    ENABLED.store(true,Ordering::Release);
    Phase
}
pub(crate) fn clear() { STATE.lock().unwrap_or_else(|e|e.into_inner()).proof=None; }
pub(super) fn input(ast:&Arc<Ast>) -> Option<Weak<Ast>> {
    if !ENABLED.load(Ordering::Acquire) { return None; }
    let s=STATE.lock().unwrap_or_else(|e|e.into_inner());
    (s.owner==Some(std::thread::current().id())).then(||Arc::downgrade(ast))
}
pub(super) fn collected(input:Option<Weak<Ast>>,p:&Program) {
    if let Some(input)=input { let mut s=STATE.lock().unwrap_or_else(|e|e.into_inner());
        if s.owner!=Some(std::thread::current().id()) { return; }
        s.proof=Some(Proof {
        input,collected:Arc::downgrade(&p.ast),resolved:p.resolved.as_ref().map(Arc::downgrade),krate:p.krate,stable:p.stable_crate,
    }); }
}
pub(crate) fn current(ast:&Arc<Ast>,p:&Program) -> bool {
    let s=STATE.lock().unwrap_or_else(|e|e.into_inner());
    s.owner==Some(std::thread::current().id()) && s.proof.as_ref().is_some_and(|proof| {
        proof.krate==p.krate && proof.stable==p.stable_crate
            && proof.input.upgrade().is_some_and(|input|Arc::ptr_eq(&input,ast))
            && proof.collected.upgrade().is_some_and(|collected|Arc::ptr_eq(&collected,&p.ast))
            && match (&proof.resolved,&p.resolved) {
                (None,None)=>true,
                (Some(old),Some(now))=>old.upgrade().is_some_and(|old|Arc::ptr_eq(&old,now)),
                _=>false,
            }
    })
}
