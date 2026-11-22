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
        Ok(Self { cfg_version, dependencies, receipt_hash, dependency_functions: BTreeSet::new() })
    }

    /// Join an actual direct leaf dependency to the observed pinned Cargo check closure.
    pub fn with_scalar_leaf(&self, leaf: &ScalarCrate<'_>) -> Option<Self> {
        if self.dependencies.len() != 2 || self.dependencies.iter().any(|dep| dep.name == leaf.name) {
            return None;
        }
        let mut context = self.clone();
        // The explicit dependency is registered after core and before the
        // compiler_builtins load is completed for this closed graph.
        context.dependencies.insert(1, SdkDependency {
            name: leaf.name.into(), hash: leaf.svh, host_hash: None,
            kind: 2, is_private: false, extra_filename: leaf.filename_suffix.into(),
        });
        context.dependency_functions = leaf.functions.iter().map(|function| NativeId {
            krate: leaf.stable_id, path: function.path_hash,
        }).collect();
        Some(context)
    }
}
