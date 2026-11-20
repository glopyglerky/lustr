//! Observe diagnostics through Rust's original query-tracking callback.
//! Until version-bound diagnostic replay is connected, only quiet successful
//! requests certify retained Rust work. The normal emitter is never replaced.
use rustc_errors::{DiagInner, ErrorGuaranteed};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};

type Tracker = fn(DiagInner, &mut dyn FnMut(DiagInner) -> Option<ErrorGuaranteed>) -> Option<ErrorGuaranteed>;
static ORIGINAL: OnceLock<Tracker> = OnceLock::new();
static OBSERVED: AtomicBool = AtomicBool::new(false);

pub fn install() {
    // rustc_interface::run_compiler installs its callback before creating the
    // ParseSess. Preserve that callback, including incremental side effects.
    ORIGINAL.set(*rustc_errors::TRACK_DIAGNOSTIC.swap(&(track as Tracker))).expect("one compiler context");
}

fn track(diagnostic: DiagInner, emit: &mut dyn FnMut(DiagInner) -> Option<ErrorGuaranteed>) -> Option<ErrorGuaranteed> {
    OBSERVED.store(true, Ordering::Release);
    ORIGINAL.get().unwrap()(diagnostic, emit)
}

pub fn quiet() -> bool {
    !OBSERVED.load(Ordering::Acquire)
}
