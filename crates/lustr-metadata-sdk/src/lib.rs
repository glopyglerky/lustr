//! Host-only loading of the construction-qualified SDK receipt.
//! The caller binds the receipt to the original construction output. This
//! adapter verifies both original metadata and link artifacts before returning
//! an immutable context to the core+alloc serializer. It invokes no compiler.
use lustr_metadata_encoder::{Sdk, SdkDependency};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path};

pub fn open(receipt: &Path, sysroot: &Path) -> Result<Sdk, String> {
    let bytes = fs::read(receipt).map_err(|e| format!("read metadata SDK receipt: {e}"))?;
    open_bytes(&bytes, sysroot)
}

/// Bind the same receipt bytes that supply the immutable SDK facts. A caller
/// must not verify a path and then let the loader read a different generation.
pub fn open_bound(receipt: &Path, sysroot: &Path, expected_sha256: &str) -> Result<Sdk, String> {
    let expected = hex::<32>(expected_sha256)?;
    let bytes = fs::read(receipt).map_err(|e| format!("read metadata SDK receipt: {e}"))?;
    if <[u8; 32]>::from(Sha256::digest(&bytes)) != expected {
        return Err("SDK construction receipt differs from its reviewed exact identity".into());
    }
    open_bytes(&bytes, sysroot)
}

pub const STANDARD_RECEIPT_PATH: &str = "lib/lustr-metadata/standard-sdk-receipt.json";

/// The constructor authenticates the SDK archives before installing these
/// passive facts. A scalar std-root archive needs no SDK code to be emitted
/// by the native compiler.
pub fn open_standard(receipt: &Path) -> Result<Sdk, String> {
    let bytes = fs::read(receipt).map_err(|error| format!("read standard metadata SDK receipt: {error}"))?;
    if bytes != include_bytes!("../../../docs/compiler-set/evidence/metadata-sdk-receipt-20261120/standard-receipt.json") {
        return Err("standard metadata SDK receipt differs from its pinned bytes".into());
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| format!("decode standard metadata SDK receipt: {error}"))?;
    if string(&value, "schema")? != "lustr-native-standard-metadata-sdk-v1"
        || string(&value, "rust_revision")? != lustr_metadata_encoder::RUST_REVISION
        || string(&value, "target")? != lustr_metadata_encoder::TARGET
        || value.get("metadata_version").and_then(Value::as_u64) != Some(12)
    {
        return Err("standard metadata SDK identity differs from the pinned compiler".into());
    }
    let rows = value["dependencies"].as_array().ok_or("standard metadata SDK dependencies missing")?;
    let dependencies = rows.iter().enumerate().map(|(index, row)| {
        if row["crate_num"].as_u64() != Some((index + 1) as u64) || !row["host_hash_le_hex"].is_null() {
            return Err("unexpected standard dependency ordinal or host hash".into());
        }
        let kind = match (string(row, "kind")?, row["kind_ordinal"].as_u64()) {
            ("Unconditional", Some(2)) => 2,
            ("Conditional", Some(1)) => 1,
            _ => return Err("unexpected standard dependency kind".into()),
        };
        Ok(SdkDependency {
            name: string(row, "name")?.to_owned(),
            hash: hex::<16>(string(row, "svh_le_hex")?)?,
            host_hash: None,
            kind,
            is_private: row["is_private"].as_bool().ok_or("standard dependency privacy missing")?,
            extra_filename: string(row, "extra_filename")?.to_owned(),
        })
    }).collect::<Result<Vec<_>, String>>()?;
    Sdk::from_standard_facts(string(&value, "cfg_version")?.into(), dependencies, Sha256::digest(&bytes).into())
}

fn open_bytes(bytes: &[u8], sysroot: &Path) -> Result<Sdk, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| format!("decode metadata SDK receipt: {e}"))?;
    if string(&value, "schema")? != "lustr-native-metadata-sdk-v1"
        || string(&value, "rust_revision")? != lustr_metadata_encoder::RUST_REVISION
        || string(&value, "target")? != lustr_metadata_encoder::TARGET
        || value.get("metadata_version").and_then(Value::as_u64) != Some(12)
    {
        return Err("metadata SDK receipt identity differs from the declared source/target/schema".into());
    }
    let cfg_version = string(&value, "cfg_version")?.to_owned();
    if cfg_version != "1.102.0-nightly (5b7b37aef 2026-11-08)" {
        return Err("invalid metadata SDK version".into());
    }
    let sysroot = fs::canonicalize(sysroot).map_err(|e| format!("resolve metadata SDK root: {e}"))?;
    let records = value.get("dependencies").and_then(Value::as_array).ok_or("metadata SDK dependencies are absent")?;
    // This is the original fixed no_std construction reference's actual
    // closure and order. A missing builtins dependency is not an empty
    // environment, and a wider receipt is not silently flattened.
    let expected = [("core", false), ("compiler_builtins", true)];
    if records.len() != expected.len() {
        return Err("metadata SDK dependency closure is not the qualified core/builtins pair".into());
    }
    let mut dependencies = Vec::new();
    let mut core_metadata = None;
    for (index, dep) in records.iter().enumerate() {
        if dep.get("crate_num").and_then(Value::as_u64) != Some((index + 1) as u64) {
            return Err("metadata SDK dependency numbering is not complete and ordered".into());
        }
        let name = string(dep, "name")?.to_owned();
        if name != expected[index].0 {
            return Err("metadata SDK dependency order/name differs from the original construction context".into());
        }
        let kind = dep.get("kind_ordinal").and_then(Value::as_u64).ok_or("metadata SDK dependency kind is absent")?;
        if kind != 2 || string(dep, "kind")? != "Unconditional" {
            return Err("metadata SDK dependency is not the qualified unconditional dependency".into());
        }
        let hash = hex::<16>(string(dep, "svh_le_hex")?)?;
        let host_hash = match dep.get("host_hash_le_hex") {
            Some(Value::Null) => None,
            _ => return Err("metadata SDK host hash differs from the qualified context".into()),
        };
        let is_private = dep.get("is_private").and_then(Value::as_bool).ok_or("metadata SDK privacy fact is absent")?;
        if is_private != expected[index].1 {
            return Err("metadata SDK dependency privacy differs from the original construction context".into());
        }
        let extra_filename = string(dep, "extra_filename")?.to_owned();
        if extra_filename.len() != 17
            || !extra_filename.starts_with('-')
            || !extra_filename.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
        {
            return Err("metadata SDK extra filename is malformed".into());
        }
        let artifact = read_artifact(dep, "artifact", &sysroot, &name, &extra_filename, "rmeta")?;
        let link_artifact = read_artifact(dep, "link_artifact", &sysroot, &name, &extra_filename, "rlib")?;
        if !link_artifact.starts_with(b"!<arch>\n") {
            return Err("metadata SDK link artifact is not its original archive".into());
        }
        // Original metadata v12 places the semantic hash after its two
        // fixed-width root positions. Bind the supplied hash to actual
        // metadata bytes, rather than trusting an edited JSON hash field.
        if artifact.len() < 40 || &artifact[..8] != b"rust\0\0\0\x0c" || artifact[24..40] != hash {
            return Err("metadata SDK metadata header/version/hash differs from its receipt".into());
        }
        let root = u64::from_le_bytes(artifact[8..16].try_into().unwrap());
        let unhashed = u64::from_le_bytes(artifact[16..24].try_into().unwrap());
        if root < 40 || root >= unhashed || unhashed >= artifact.len() as u64 {
            return Err("metadata SDK metadata root positions are invalid".into());
        }
        if name == "core" {
            core_metadata = Some(artifact);
        }
        dependencies.push(SdkDependency { name, hash, host_hash, kind: kind as u8, is_private, extra_filename });
    }
    let sdk = Sdk::from_facts(cfg_version, dependencies, Sha256::digest(bytes).into())?;
    Ok(sdk.with_core_metadata(core_metadata.as_deref().ok_or("verified core metadata is missing")?))
}
fn read_artifact(dep: &Value, field: &str, sysroot: &Path, name: &str, suffix: &str, extension: &str) -> Result<Vec<u8>, String> {
    let artifact = dep.get(field).ok_or_else(|| format!("metadata SDK dependency {field} is absent"))?;
    let relative = Path::new(string(artifact, "path")?);
    let expected = format!("lib/rustlib/{}/lib/lib{name}{suffix}.{extension}", lustr_metadata_encoder::TARGET);
    if relative != Path::new(&expected) || relative.is_absolute() || relative.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err("metadata SDK artifact path does not identify the declared dependency below its root".into());
    }
    let path = fs::canonicalize(sysroot.join(relative)).map_err(|e| format!("resolve metadata SDK artifact: {e}"))?;
    if !path.starts_with(sysroot) {
        return Err("metadata SDK artifact escapes its root".into());
    }
    let contents = fs::read(&path).map_err(|e| format!("read metadata SDK artifact: {e}"))?;
    if artifact.get("size").and_then(Value::as_u64) != Some(contents.len() as u64)
        || hex::<32>(string(artifact, "sha256")?)? != <[u8; 32]>::from(Sha256::digest(&contents))
    {
        return Err("metadata SDK artifact differs from its construction receipt".into());
    }
    Ok(contents)
}

fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("metadata SDK field {field} is absent"))
}

fn hex<const N: usize>(value: &str) -> Result<[u8; N], String> {
    if value.len() != 2 * N || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("metadata SDK hash has invalid length or bytes".into());
    }
    let mut result = [0; N];
    for (out, pair) in result.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        let decode = |v: u8| if v.is_ascii_digit() { v - b'0' } else { v.to_ascii_lowercase() - b'a' + 10 };
        *out = decode(pair[0]) * 16 + decode(pair[1]);
    }
    Ok(result)
}
