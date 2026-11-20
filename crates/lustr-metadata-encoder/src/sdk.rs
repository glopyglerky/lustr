use alloc::{string::String, vec::Vec};

/// Facts supplied by the original-reader-qualified host SDK context. This is a
/// data record, not a certificate derived from a user-provided success flag.
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
pub struct Sdk {
    pub(crate) cfg_version: String,
    pub(crate) dependencies: Vec<SdkDependency>,
    pub(crate) receipt_hash: [u8; 32],
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
        Ok(Self { cfg_version, dependencies, receipt_hash })
    }
}
