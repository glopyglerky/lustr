//! Deferred, opt-in diagnostic instrumentation. Not part of the public binary.
use std::{cell::{RefCell, Cell}, time::Instant};

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Stage {
    FullSelect, Capture, InputRead, Environment, ContextJson, Toolchain,
    ReceiptRead, ReceiptParse, Executable, ExecType, ExecAccess, ExecCanonical,
    ExecRead, ExecStamp, LibraryDiscovery, LibraryAccess, LibraryStamp,
    ToolStateJson, Binding, Config, Discovery, SnapshotJson, EntryKey,
    Load, RecordPolicy, RecordRead, RecordParse, RecordChecksum, RecordClone,
    StateDigest, SelectionDecode, PackageAdmission, TargetBindings, Diagnostic,
    Stamp,
}

const NAMES: [&str; 35] = [
    "full_select", "capture", "input_read", "environment", "context_json", "toolchain",
    "receipt_read", "receipt_parse", "executable", "exec_type", "exec_access", "exec_canonical",
    "exec_read", "exec_stamp", "library_discovery", "library_access", "library_stamp",
    "tool_state_json", "binding", "config", "discovery", "snapshot_json", "entry_key",
    "load", "record_policy", "record_read", "record_parse", "record_checksum", "record_clone",
    "state_digest", "selection_decode", "package_admission", "target_bindings", "diagnostic",
    "stamp",
];

thread_local! {
    static COUNTERS: RefCell<([u64; 35], [u64; 35])> = const { RefCell::new(([0;35], [0;35])) };
    static STATE: Cell<u8> = const { Cell::new(0) };
}

pub fn event(state: &str) {
    STATE.set(match state { "hit" => 1, "miss" => 2, "ineligible" => 3, _ => 4 });
}

pub fn cache_state() -> &'static str {
    match STATE.get() { 1 => "hit", 2 => "miss", 3 => "ineligible", 4 => "other", _ => "none" }
}

pub struct Span(Stage, Instant);
impl Span {
    pub fn new(stage: Stage) -> Self { Self(stage, Instant::now()) }
}
impl Drop for Span {
    fn drop(&mut self) {
        let elapsed = self.1.elapsed().as_nanos() as u64;
        COUNTERS.with(|counter| {
            let mut counter = counter.borrow_mut();
            counter.0[self.0 as usize] += elapsed;
            counter.1[self.0 as usize] += 1;
        });
    }
}

pub fn time<T>(stage: Stage, operation: impl FnOnce() -> T) -> T {
    let _span = Span::new(stage);
    operation()
}

// All formatting/allocation happens after full::select and its outer timer end.
// Spans are inclusive and nested; they must not be summed as independent costs.
pub fn take() -> serde_json::Value {
    COUNTERS.with(|counter| {
        let counters = counter.replace(([0;35], [0;35]));
        serde_json::Value::Object(NAMES.iter().enumerate().map(|(i, &name)| {
            (name.to_owned(), serde_json::json!({"ns":counters.0[i],"calls":counters.1[i]}))
        }).collect())
    })
}
