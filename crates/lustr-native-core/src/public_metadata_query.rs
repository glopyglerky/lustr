//! Read exact imported declarations through the installed Rust service.
use crate::{def, driver::imports::ScalarFunction, ty::IntTy};
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Path, PathBuf}, process::Command};

use super::{ImportFailure, Stamp, hash, stamp};

pub(super) struct Interface {
    pub name: String,
    pub suffix: String,
    pub stable: def::StableCrateHash,
    pub svh: [u8; 16],
    pub requires_abort: bool,
    pub functions: Vec<ScalarFunction>,
    pub inputs: Vec<(PathBuf, Stamp)>,
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, size: usize) -> Option<&'a [u8]> {
        let (value, rest) = self.0.split_at_checked(size)?;
        self.0 = rest;
        Some(value)
    }
    fn bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }
    fn count(&mut self, limit: usize) -> Option<usize> {
        let count = u32::from_le_bytes(self.bytes()?) as usize;
        (count <= limit).then_some(count)
    }
    fn text(&mut self) -> Option<&'a str> {
        let length = self.count(1 << 20)?;
        std::str::from_utf8(self.take(length)?).ok()
    }
    fn integer(&mut self) -> Option<IntTy> {
        const TYPES: [IntTy; 10] = [IntTy::I8, IntTy::I16, IntTy::I32, IntTy::I64, IntTy::Isize,
            IntTy::U8, IntTy::U16, IntTy::U32, IntTy::U64, IntTy::Usize];
        TYPES.get(self.take(1)?[0] as usize).copied()
    }
}

fn identifier(name: &str) -> bool {
    !name.is_empty() && name.bytes().enumerate().all(|(index, byte)| {
        byte == b'_' || byte.is_ascii_alphabetic() || index != 0 && byte.is_ascii_digit()
    })
}

fn decode(output: &[u8], artifact: &[u8], mut sdk_inputs: BTreeMap<PathBuf, Stamp>) -> Option<Interface> {
    let mut reader = Reader(output.strip_prefix(b"lustr-external-scalar-v1\0")?);
    if reader.text()?.strip_prefix("sha256=")? != hash(artifact) {
        return None;
    }
    let name = reader.text()?;
    let suffix = reader.text()?;
    if !identifier(name) || !suffix.is_empty() && (!suffix.starts_with('-')
        || !suffix.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'))
    {
        return None;
    }
    let stable = def::StableCrateHash(u64::from_le_bytes(reader.bytes()?));
    let svh = reader.bytes()?;
    let requires_abort = match reader.take(1)?[0] { 0 => false, 1 => true, _ => return None };
    let mut inputs = Vec::new();
    for _ in 0..reader.count(512)? {
        let path = PathBuf::from(reader.text()?);
        let before = sdk_inputs.remove(&path)?;
        if stamp(&path).ok()? != before {
            return None;
        }
        inputs.push((path, before));
    }
    let mut names = BTreeSet::new();
    let mut definitions = BTreeSet::new();
    let mut functions = Vec::new();
    for _ in 0..reader.count(65_535)? {
        let function = reader.text()?;
        let hash = def::DefPathHash(u64::from_le_bytes(reader.bytes()?));
        let parameters = (0..reader.count(64)?).map(|_| reader.integer()).collect::<Option<Vec<_>>>()?;
        let result = reader.integer()?;
        let symbol = reader.text()?;
        if !identifier(function) || !identifier(symbol) || !names.insert(function) || !definitions.insert(hash) {
            return None;
        }
        functions.push(ScalarFunction { name: function.into(), hash, parameters, result, symbol: symbol.into() });
    }
    if functions.is_empty() || !reader.0.is_empty() {
        return None;
    }
    Some(Interface { name: name.into(), suffix: suffix.into(), stable, svh, requires_abort, functions, inputs })
}

pub(super) fn read(
    installation: &Path, alias: &str, path: &Path, bytes: &[u8], search_paths: &[PathBuf],
) -> Result<Interface, ImportFailure> {
    let root = installation.join("libexec/lustr-host-proc-macro");
    let compiler = root.join("bin/lustr-rustc-host-proc-macro");
    let receipt = root.join("metadata-interface.id");
    let owned = [&compiler, &receipt];
    let before = owned.iter().map(|file| stamp(file)).collect::<Result<Vec<_>, _>>()?;
    let expected = fs::read_to_string(&receipt).map_err(|error| format!("read installed metadata reader identity: {error}"))?;
    let compiler_bytes = fs::read(&compiler).map_err(|error| format!("read installed metadata reader: {error}"))?;
    if expected.trim_end() != hash(&compiler_bytes) {
        return Err("installed metadata reader identity changed".into());
    }
    let sdk = root.join("lib").canonicalize().map_err(|error| format!("resolve installed metadata SDK: {error}"))?;
    let library = sdk.join("rustlib").join(crate::protocol::host_triple()).join("lib");
    let mut sdk_inputs = BTreeMap::new();
    for entry in fs::read_dir(library).map_err(|error| format!("read installed target metadata: {error}"))? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().is_some_and(|ext| matches!(ext.to_str(), Some("rmeta" | "rlib" | "so"))) {
            let path = path.canonicalize().map_err(|error| error.to_string())?;
            if !path.starts_with(&sdk) {
                return Err("installed metadata target escapes its SDK".into());
            }
            sdk_inputs.insert(path.clone(), stamp(&path)?);
        }
    }
    let answer = Command::new(&compiler).arg("--lustr-read-crate-interface").arg(alias).arg(path)
        .args(search_paths)
        .env_remove("LD_LIBRARY_PATH").output().map_err(|error| format!("start Rust metadata reader: {error}"))?;
    if answer.status.code() == Some(crate::protocol::RUST_COMPLEMENT_REQUIRED) {
        let expected = format!("lustr-extern-requires-rust-v1\0sha256={}", hash(bytes));
        if answer.stdout != expected.as_bytes() || !answer.stderr.is_empty() {
            return Err("invalid Rust metadata capability response".into());
        }
        for (file, prior) in owned.into_iter().zip(&before) {
            if stamp(file)? != *prior {
                return Err("installed metadata reader changed during import".into());
            }
        }
        return Err(ImportFailure::RequiresRust("the original Rust dependency requires its compiler context"));
    }
    if !answer.status.success() {
        return Err(format!("Rust metadata reader did not represent the extern: {}", String::from_utf8_lossy(&answer.stderr).trim_end()).into());
    }
    let mut interface = decode(&answer.stdout, bytes, sdk_inputs).ok_or("invalid installed Rust metadata response")?;
    for (file, prior) in owned.into_iter().zip(before) {
        if stamp(file)? != prior {
            return Err("installed metadata reader changed during import".into());
        }
        interface.inputs.push((file.clone(), prior));
    }
    Ok(interface)
}
