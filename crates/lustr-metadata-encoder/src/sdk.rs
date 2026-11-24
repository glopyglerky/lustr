use alloc::{collections::BTreeSet, string::String, vec::Vec};

use crate::{NativeId, ScalarCrate};

/// Facts supplied by the original-reader-qualified host SDK context. This is a
/// data record, not a certificate derived from a user-provided success flag.
#[derive(Clone)]
pub struct SdkDependency {
    pub name: String,
    pub hash: [u8; 16],
    pub host_hash: Option<[u8; 16]>,
    pub kind: u8,
    pub is_private: bool,
    pub extra_filename: String,
}

/// Immutable SDK facts retained by the serializer. The host adapter establishes
/// their artifact and original-reader provenance before calling `from_facts`.
/// This core type performs no file IO, JSON decoding or subprocess invocation.
#[derive(Clone)]
pub struct Sdk {
    pub(crate) cfg_version: String,
    pub(crate) dependencies: Vec<SdkDependency>,
    pub(crate) receipt_hash: [u8; 32],
    pub(crate) dependency_functions: BTreeSet<NativeId>,
    pub(crate) direct_dependency: Option<[u8; 16]>,
    pub(crate) standard: bool,
    pub(crate) core_size_traits: Option<[usize; 2]>,
}

impl Sdk {
    /// Check the represented context shape. Actual SDK artifact qualification
    /// belongs to the original construction receipt and host adapter; arbitrary
    /// values passed here do not establish compiler or SDK correctness.
    pub fn from_facts(cfg_version: String, dependencies: Vec<SdkDependency>, receipt_hash: [u8; 32]) -> Result<Self, String> {
        let expected = [("core", false), ("compiler_builtins", true)];
        if cfg_version != "1.102.0-nightly (5b7b37aef 2026-11-08)" || dependencies.len() != expected.len() {
            return Err("metadata SDK version or dependency closure is not represented".into());
        }
        for (dep, (name, private)) in dependencies.iter().zip(expected) {
            if dep.name != name
                || dep.is_private != private
                || dep.kind != 2
                || dep.host_hash.is_some()
                || dep.extra_filename.len() != 17
                || !dep.extra_filename.starts_with('-')
                || !dep.extra_filename.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
            {
                return Err("metadata SDK dependency facts differ from the qualified scope".into());
            }
        }
        Ok(Self { cfg_version, dependencies, receipt_hash, dependency_functions: BTreeSet::new(),
            direct_dependency: None, standard: false, core_size_traits: None })
    }

    /// Keep the language-item identities recorded by the authenticated core
    /// artifact, whose exact digest the host checked before this call.
    pub fn with_core_metadata(mut self, artifact: &[u8]) -> Self {
        self.core_size_traits = self.dependencies.iter().find(|dependency| dependency.name == "core")
            .and_then(|core| crate::extern_metadata::read_core_size_traits(artifact, &self.cfg_version, &core.hash));
        self
    }

    /// A separate construction-qualified std prelude and runtime dependency
    /// inventory. The host must authenticate the receipt before supplying it.
    pub fn from_standard_facts(cfg_version: String, dependencies: Vec<SdkDependency>, receipt_hash: [u8; 32]) -> Result<Self, String> {
        let names: BTreeSet<_> = dependencies.iter().map(|dependency| dependency.name.as_str()).collect();
        if cfg_version != "1.102.0-nightly (5b7b37aef 2026-11-08)"
            || dependencies.len() != 19 || names.len() != dependencies.len()
            || dependencies[0].name != "std" || dependencies[0].is_private
            || dependencies[1].name != "core" || dependencies[1].is_private
            || dependencies[18].name != "panic_unwind" || !dependencies[18].is_private
            || dependencies.iter().enumerate().any(|(index, dep)| {
                dep.kind != if index == 18 { 1 } else { 2 } || dep.host_hash.is_some()
                    || dep.extra_filename.len() != 17 || !dep.extra_filename.starts_with('-')
                    || !dep.extra_filename.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
            })
        {
            return Err("standard metadata SDK facts differ from the pinned dependency closure".into());
        }
        Ok(Self { cfg_version, dependencies, receipt_hash, dependency_functions: BTreeSet::new(),
            direct_dependency: None, standard: true, core_size_traits: None })
    }

    /// Join an actual direct leaf dependency to the observed pinned Cargo library closure.
    pub fn with_scalar_leaf(&self, leaf: &ScalarCrate<'_>) -> Option<Self> {
        // The pinned compiler registers a no_std leaf after core for a
        // no_std consumer, and after the implicit std closure for a std root.
        let ordinal = match (self.standard, self.dependencies.len()) {
            (false, 2) => 1,
            (true, 19) => 19,
            _ => return None,
        };
        if !leaf.dependencies.is_empty() || self.dependencies.iter().any(|dep| dep.name == leaf.name) {
            return None;
        }
        let mut context = self.clone();
        context.dependencies.insert(ordinal, SdkDependency {
            name: leaf.name.into(), hash: leaf.svh, host_hash: None,
            kind: 2, is_private: false, extra_filename: leaf.filename_suffix.into(),
        });
        context.dependency_functions = leaf.functions.iter().map(|function| NativeId {
            krate: leaf.stable_id, path: function.path_hash,
        }).collect();
        context.direct_dependency = Some(leaf.svh);
        Some(context)
    }
}
