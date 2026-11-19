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

/// Facts and code belonging to an authoritative imported compiler context.
/// The provider must finish semantic/ABI admission before publishing a def.
/// A provider error is a compiler failure, never a complementary fallback.
pub trait Definitions {
    fn mount(&self, tables: &mut ResolvedTables, interner: &Interner) -> Result<(), Diag>;
    fn install(&self, program: &mut Program) -> Result<(), Vec<Diag>>;
    fn validate_checked(&self, program: &Program) -> Result<(), String>;
    fn dependencies(&self) -> Vec<Dependency>;
    fn objects(&self) -> Vec<std::sync::Arc<[u8]>>;
}
