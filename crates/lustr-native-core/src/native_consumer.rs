//! Standalone caller of the common native completion boundary.
use crate::{diag, native_object};
use std::collections::BTreeSet;
pub(crate) use crate::driver::library::completion::{Error, Function, Visibility};
/// This token is not constructible from a success flag or an emitter's exit
/// status. It owns the actual native program and a complete joined inventory.
/// All bodies have already passed native checking, lowering and borrowck.
pub(crate) struct CompletedUnit {
    prepared: native_object::PreparedUnit,
    functions: Vec<Function>,
}

impl CompletedUnit {
    pub(crate) fn prepared(&self) -> &native_object::PreparedUnit {
        &self.prepared
    }
    pub(crate) fn functions(&self) -> &[Function] {
        &self.functions
    }
    pub(crate) fn root_span(&self) -> diag::Span {
        diag::Span::new(diag::FileId(0), 0, self.prepared.source().len() as u32)
    }
    pub(crate) fn emit(self) -> Result<EmittedUnit, String> {
        let Self { prepared, functions } = self;
        let object = native_object::emit(prepared)?;
        let declared: BTreeSet<_> = functions.iter().map(Function::definition).collect();
        let emitted: BTreeSet<_> = object.functions().iter().map(native_object::EmittedFunction::definition).collect();
        if declared != emitted || object.functions().len() != functions.len() {
            return Err("native metadata declaration and emitted-instance inventories differ".into());
        }
        Ok(EmittedUnit { object, functions })
    }
}

pub(crate) struct EmittedUnit {
    object: native_object::EmittedUnit,
    functions: Vec<Function>,
}

impl EmittedUnit {
    pub(crate) fn native(&self) -> &native_object::EmittedUnit {
        &self.object
    }
    pub(crate) fn functions(&self) -> &[Function] {
        &self.functions
    }
}

pub(crate) fn complete(prepared: native_object::PreparedUnit) -> Result<CompletedUnit, Error> {
    if prepared.working_directory().is_none() { return Err(Error::Failed("native request working directory was not retained")); }
    let functions = crate::driver::library::completion::complete(crate::driver::library::completion::View {
        program: prepared.program(), source: prepared.source(), map: prepared.source_map(),
        bodies: prepared.accepted_bodies(), identifier: &|name, span| prepared.identifier_span(name, span),
    }, true)?;
    Ok(CompletedUnit { prepared, functions })
}
