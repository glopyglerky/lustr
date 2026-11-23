//! Imported semantic definitions for one owned native request.
//!
//! The provider installs resolved definitions before collection checks any
//! initializer or body. It supplies actual compiler facts, never declarations
//! generated as source. Retained owners bind validated declaration inputs to
//! native read replay and keep the exact dependency bytes for artifact use.
//! Disk-cache restoration of imported generations remains unavailable.

use std::sync::Arc;

use crate::{check::collect::Program, def, diag::Diag, intern::Interner, resolve::imports::ResolvedTables, ty};

/// An immutable metadata, compiler, or object input consumed by the provider.
#[derive(PartialEq, Eq)]
pub struct Dependency {
    pub identity: String,
    pub contents: Arc<[u8]>,
}

/// The checked declarations the Cargo metadata reader mounts for an executable.
/// It does not expose implementation bodies, inlining, or const evaluation.
#[derive(PartialEq, Eq)]
pub struct CargoDefinitions {
    pub alias: String,
    pub crate_name: String,
    pub stable: def::StableCrateHash,
    pub index: def::CrateId,
    pub functions: Vec<ScalarFunction>,
    pub projection: Option<ProjectionDefinition>,
}

/// A validated nongeneric receiver and an associated lifetime mapped to a
/// shared integer reference. A missing implementation is itself a fact.
#[derive(PartialEq, Eq)]
pub struct ProjectionDefinition {
    pub receiver: (String, def::DefPathHash),
    pub family: (String, def::DefPathHash),
    pub item: (String, def::DefPathHash),
    pub referent: Option<ty::IntTy>,
}

#[derive(PartialEq, Eq)]
pub struct ScalarFunction {
    pub name: String,
    pub hash: def::DefPathHash,
    pub parameters: Vec<ty::IntTy>,
    pub result: ty::IntTy,
    pub symbol: String,
}

/// Inputs that affect the current checked declarations. Providers without a
/// complete representation of their facts conservatively use exact bytes.
#[derive(PartialEq, Eq)]
pub(crate) enum Generation {
    Inputs(Vec<Dependency>),
    Cargo(Vec<Arc<CargoDefinitions>>),
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
    /// Validated definitions actually exposed to this executable's checker.
    /// Exact metadata and archive inputs remain bound to artifact publication.
    fn cargo_definitions(&self) -> Option<Vec<Arc<CargoDefinitions>>> { None }
    /// Transfer the immutable, single-use plan after semantic validation.
    /// Constructing or taking the plan must not emit code or materialize objects.
    fn take_emission(&self) -> Result<Box<dyn Emission + 'static>, String>;
}
