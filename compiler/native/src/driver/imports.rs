//! Imported semantic definitions for one owned native request.
//!
//! The provider installs resolved definitions before collection checks any
//! initializer or body. It supplies actual compiler facts, never declarations
//! generated as source. The first boundary is deliberately one request in a
//! fresh session: retained imports need a joined dependency generation before
//! they may authorize another request or a disk-cache answer.

use crate::{check::collect::Program, diag::Diag, intern::Interner, resolve::imports::ResolvedTables};

/// An immutable input to the imported semantic service. The provider records
/// the real metadata/compiler/object identity it consumed; native checking's
/// definition reads remain attached to the imported definitions themselves.
pub struct Dependency {
    pub identity: String,
    pub contents: std::sync::Arc<[u8]>,
}

/// Native-supported work whose joined context/profile is not implemented by
/// this first import seam. It is not invalid Rust or an emission failure.
#[derive(Debug)]
pub struct Unintegrated(pub &'static str);

/// Owned code generation for one already checked imported request. The plan
/// contains no borrowed compiler arenas and is consumed exactly once by Build.
/// Checking or abandoning a prepared request drops it without executing it.
pub trait Emission: Send {
    fn emit(self: Box<Self>) -> Result<Vec<std::sync::Arc<[u8]>>, String>;
}

/// Facts and code belonging to an authoritative imported compiler context.
/// The provider must finish semantic/ABI admission before publishing a def.
/// A provider error is a compiler failure, never a complementary fallback.
pub trait Definitions {
    /// Name decoded from an artifact supplied for an actual Cargo unit.
    fn cargo_crate_name(&self) -> Option<&str> { None }
    fn cargo_extern_matches(&self, _alias: &str, _name: &str) -> bool { false }
    fn mount(&self, tables: &mut ResolvedTables, interner: &Interner) -> Result<(), Diag>;
    fn install(&self, program: &mut Program) -> Result<(), Vec<Diag>>;
    /// Validate the installed semantic facts without executing an emitter.
    fn validate_checked(&self, program: &Program) -> Result<(), String>;
    fn dependencies(&self) -> Vec<Dependency>;
    /// Transfer the immutable, single-use plan after semantic validation.
    /// Constructing or taking the plan must not emit code or materialize objects.
    fn take_emission(&self) -> Result<Box<dyn Emission + 'static>, String>;
}
