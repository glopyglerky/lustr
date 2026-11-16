//! Reuse the installed, pinned Rust LLVM backend. Its codegen-unit query is the
//! connection: successful native definitions become external references, while
//! Rust still emits every other object and the actual Rust metadata.
use rustc_codegen_ssa::{CompiledModules, CrateInfo, TargetConfig, traits::CodegenBackend};
use rustc_middle::{
    dep_graph::WorkProductMap,
    mono::{CodegenUnit, MonoItem},
    ty::TyCtxt,
    util::Providers,
};
use rustc_session::{
    CodegenBackendInit, EarlySession, IncrCompSession, Session,
    config::{OutputFilenames, PrintRequest},
};
use rustc_span::Symbol;
use std::{
    any::Any,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

type UnitProvider = for<'tcx> fn(TyCtxt<'tcx>, Symbol) -> &'tcx CodegenUnit<'tcx>;
static ORIGINAL: OnceLock<UnitProvider> = OnceLock::new();
type AnalysisProvider = for<'tcx> fn(TyCtxt<'tcx>, ());
static ANALYSIS: OnceLock<AnalysisProvider> = OnceLock::new();
static REUSED: AtomicBool = AtomicBool::new(false);
pub fn reuse_checked_context() {
    REUSED.store(true, Ordering::Release);
}
fn analysis(tcx: TyCtxt<'_>, key: ()) {
    if !REUSED.load(Ordering::Acquire) {
        ANALYSIS.get().unwrap()(tcx, key);
    } else {
        // The pinned run_required_analyses establishes this definition-table
        // transition after lowering. Rust's original metadata encoder needs
        // it even when unchanged semantic checks are retained. Keep the table
        // available for lazy queries rather than freezing before they run.
        tcx.ensure_done().hir_crate_items(());
        tcx.untracked().definitions.write().commit_end_of_determinism();
    }
}
pub fn providers(_: &Session, providers: &mut Providers) {
    ORIGINAL.set(providers.queries.codegen_unit).expect("one compiler context");
    providers.queries.codegen_unit = unit;
    ANALYSIS.set(providers.queries.analysis).expect("one compiler context");
    providers.queries.analysis = analysis;
}
fn unit<'tcx>(tcx: TyCtxt<'tcx>, name: Symbol) -> &'tcx CodegenUnit<'tcx> {
    let old = ORIGINAL.get().unwrap()(tcx, name);
    if !old
        .items()
        .keys()
        .any(|item| matches!(item,MonoItem::Fn(instance) if crate::mixed_objects::contains(tcx.symbol_name(*instance).name)))
    {
        return old;
    }
    let mut unit = CodegenUnit::new(old.name());
    unit.set_symbol_name(old.symbol_name());
    if old.is_primary() {
        unit.make_primary();
    }
    if old.is_code_coverage_dead_code_cgu() {
        unit.make_code_coverage_dead_code_cgu();
    }
    for (item, data) in old.items() {
        if let MonoItem::Fn(instance) = item
            && crate::mixed_objects::contains(tcx.symbol_name(*instance).name)
        {
            crate::trace(&format!("native-definition-omitted={}", tcx.symbol_name(*instance).name));
            continue;
        }
        unit.items_mut().insert(*item, *data);
    }
    unit.compute_size_estimate();
    tcx.arena.alloc(unit)
}

pub struct Backend {
    inner: Box<dyn CodegenBackend>,
}
impl Backend {
    pub fn new() -> Self {
        Self { inner: rustc_codegen_llvm::LlvmCodegenBackend::new() }
    }
}
impl CodegenBackend for Backend {
    fn name(&self) -> &'static str {
        self.inner.name()
    }
    fn init(&mut self, s: &EarlySession) -> CodegenBackendInit {
        self.inner.init(s)
    }
    fn print(&self, r: &PrintRequest, o: &mut String, s: &Session) {
        self.inner.print(r, o, s)
    }
    fn target_config(&self, s: &EarlySession) -> TargetConfig {
        self.inner.target_config(s)
    }
    fn supported_crate_types(&self, s: &Session) -> Vec<rustc_structures::CrateType> {
        self.inner.supported_crate_types(s)
    }
    fn print_passes(&self) {
        self.inner.print_passes()
    }
    fn print_version(&self) {
        self.inner.print_version()
    }
    fn has_zstd(&self) -> bool {
        self.inner.has_zstd()
    }
    fn has_mnemonic(&self, s: &Session, m: &str) -> bool {
        self.inner.has_mnemonic(s, m)
    }
    fn metadata_loader(&self) -> Box<rustc_metadata::creader::MetadataLoaderDyn> {
        self.inner.metadata_loader()
    }
    fn provide(&self, p: &mut Providers) {
        self.inner.provide(p)
    }
    fn target_cpu(&self, s: &Session) -> String {
        self.inner.target_cpu(s)
    }
    fn codegen_crate<'tcx>(&self, t: TyCtxt<'tcx>) -> Box<dyn Any> {
        self.inner.codegen_crate(t)
    }
    fn join_codegen(
        &self,
        a: Box<dyn Any>,
        s: &Session,
        i: Option<&IncrCompSession>,
        o: &OutputFilenames,
        c: &CrateInfo,
    ) -> (CompiledModules, WorkProductMap) {
        let (mut modules, work) = self.inner.join_codegen(a, s, i, o, c);
        crate::mixed_objects::append_objects(&mut modules);
        (modules, work)
    }
    fn link(&self, s: &Session, modules: CompiledModules, c: CrateInfo, m: rustc_metadata::EncodedMetadata, o: &OutputFilenames) {
        let mut modules = modules;
        if let Err(e) = crate::mixed_objects::retain_link(s, &mut modules, &c, &m, o) {
            s.dcx().fatal(format!("cannot retain the checked Rust link: {e}"));
        }
        self.inner.link(s, modules, c, m, o)
    }
}
