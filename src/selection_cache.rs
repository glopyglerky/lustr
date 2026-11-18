// A deliberately bounded cache of successful public-entry Cargo selection.
// Cargo remains the parser and package-spec resolver. This admission grammar
// only excludes manifests whose metadata can depend on unobserved input.
use super::{Request, Value};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const LIMIT: u64 = 1024 * 1024;
const SCHEMA: &str = "lustr-cargo-selection-v2";

unsafe extern "C" {
    fn geteuid() -> u32;
}

fn private_directory(root: &Path) -> Option<()> {
    match fs::DirBuilder::new().mode(0o700).create(root) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return None,
    }
    let metadata = root.symlink_metadata().ok()?;
    // SAFETY: geteuid has no arguments or preconditions and does not mutate
    // process state. The directory itself must not be a symlink.
    (metadata.is_dir() && metadata.uid() == unsafe { geteuid() } && metadata.mode() & 0o077 == 0).then_some(())
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn read(path: &Path) -> Option<Vec<u8>> {
    let file = File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() as u64 <= LIMIT).then_some(bytes)
}

fn text(path: &Path) -> Option<String> {
    String::from_utf8(read(path)?).ok()
}

fn path_text(path: &Path) -> Option<&str> {
    path.to_str()
}

fn absent(path: &Path) -> bool {
    matches!(path.symlink_metadata(), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
}

fn plain_string(value: &str) -> Option<&str> {
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    (!value.contains(['"', '\\', '\n', '\r']) && !value.chars().any(char::is_control)).then_some(value)
}

fn scalar(value: &str) -> bool {
    plain_string(value).is_some() || matches!(value, "true" | "false") || !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

// This is not a TOML parser. Any syntax outside this small literal grammar
// takes the original Cargo path, including otherwise valid TOML syntax.
fn admitted_manifest(manifest: &str) -> Option<Vec<PathBuf>> {
    let mut section = "";
    let mut workspace = false;
    let mut package = false;
    let mut paths = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            section = match line {
                "[package]" => {
                    package = true;
                    "package"
                }
                "[workspace]" => {
                    workspace = true;
                    "workspace"
                }
                "[[bin]]" | "[lib]" | "[[example]]" | "[[test]]" | "[[bench]]" => "target",
                "[profile.dev]" | "[profile.release]" | "[profile.test]" | "[profile.bench]" | "[profile.doc]" => "profile",
                _ => return None,
            };
            continue;
        }
        let (key, value) = line.split_once('=')?;
        let key = key.trim();
        let value = value.trim();
        if !scalar(value) {
            return None;
        }
        let allowed = match section {
            "package" => matches!(
                key,
                "name"
                    | "version"
                    | "edition"
                    | "rust-version"
                    | "description"
                    | "license"
                    | "repository"
                    | "homepage"
                    | "documentation"
                    | "readme"
                    | "default-run"
                    | "autobins"
                    | "autolib"
                    | "autoexamples"
                    | "autotests"
                    | "autobenches"
                    | "publish"
            ),
            "target" => matches!(key, "name" | "path" | "edition" | "test" | "bench" | "doc" | "doctest" | "harness"),
            "profile" => matches!(
                key,
                "opt-level"
                    | "debug"
                    | "strip"
                    | "debug-assertions"
                    | "overflow-checks"
                    | "lto"
                    | "panic"
                    | "incremental"
                    | "codegen-units"
                    | "rpath"
                    | "split-debuginfo"
            ),
            _ => false,
        };
        if !allowed {
            return None;
        }
        if section == "target" && key == "path" {
            let path = PathBuf::from(plain_string(value)?);
            if path.as_os_str().is_empty() || !path.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir)) {
                return None;
            }
            paths.push(path);
        }
    }
    (workspace && package).then_some(paths)
}

// Body contents are not inputs to Cargo target selection. Record path type,
// symlink binding and canonical destination without invalidating on body edits.
fn binding(path: &Path) -> Option<Value> {
    let shown = path_text(path)?;
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(json!([shown, "absent"])),
        Err(_) => return None,
    };
    let kind = if metadata.file_type().is_symlink() {
        "symlink"
    } else if metadata.is_file() {
        "file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        return None;
    };
    let destination = match path.canonicalize() {
        Ok(path) => Some(path_text(&path)?.to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => return None,
    };
    Some(json!([shown, kind, destination]))
}

fn stamp(path: &Path) -> Option<Value> {
    let canonical = path.canonicalize().ok()?;
    let m = canonical.metadata().ok()?;
    file_stamp(path, &canonical, &m)
}

fn file_stamp(path: &Path, canonical: &Path, m: &fs::Metadata) -> Option<Value> {
    if !m.is_file() {
        return None;
    }
    Some(json!([
        path_text(path)?,
        path_text(canonical)?,
        m.dev(),
        m.ino(),
        m.mode(),
        m.uid(),
        m.gid(),
        m.len(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec()
    ]))
}

fn native_executable(path: &Path) -> Option<Value> {
    let canonical = path.canonicalize().ok()?;
    if canonical.file_name()?.to_str()? == "rustup" {
        return None;
    }
    let mut magic = [0; 4];
    let mut file = File::open(&canonical).ok()?;
    file.read_exact(&mut magic).ok()?;
    if magic != *b"\x7fELF"
        && !matches!(
            magic,
            [0xcf, 0xfa, 0xed, 0xfe]
                | [0xfe, 0xed, 0xfa, 0xcf]
                | [0xce, 0xfa, 0xed, 0xfe]
                | [0xfe, 0xed, 0xfa, 0xce]
                | [0xca, 0xfe, 0xba, 0xbe]
        )
    {
        return None;
    }
    file_stamp(path, &canonical, &file.metadata().ok()?)
}

fn pin(key: &str) -> Option<&'static str> {
    let prefix = format!("{key} = \"");
    include_str!("../upstream/bluefe-core.pin").lines().find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('"'))
}

#[derive(PartialEq)]
struct Tools {
    state: Value,
    files: Vec<(PathBuf, String, Value)>,
}

fn toolchain(parent: &Path) -> Option<Tools> {
    // Other hosts retain fresh Cargo selection until their source-built SDK
    // identity has an equally narrow admission rule.
    if super::super::protocol::host_triple() != "x86_64-unknown-linux-gnu" {
        return None;
    }
    let package = parent.parent()?;
    let receipt = text(&package.join("lib/lustr-selection-tools.txt"))?;
    let mut lines = receipt.lines();
    if lines.next()? != "lustr-selection-toolchain-v1"
        || lines.next()? != format!("source={}", pin("source")?)
        || lines.next()? != "host=x86_64-unknown-linux-gnu"
    {
        return None;
    }
    let mut expected = std::collections::BTreeMap::new();
    for line in lines {
        let (hash, relative) = line.split_once("  ")?;
        let relative = PathBuf::from(relative);
        if hash.len() != 64
            || !hash.bytes().all(|b| b.is_ascii_hexdigit())
            || !relative.components().all(|c| matches!(c, Component::Normal(_)))
            || expected.insert(relative, hash.to_owned()).is_some()
        {
            return None;
        }
    }
    if expected.get(Path::new("bin/cargo"))? != pin("cargo_sha256")?
        || expected.get(Path::new("bin/rustc-reference"))? != pin("rustc_sha256")?
    {
        return None;
    }
    let mut files = vec![std::env::current_exe().ok()?, parent.join("cargo"), parent.join("rustc-full"), parent.join("rustdoc")];
    if !absent(&parent.join("rustc-reference")) {
        files.push(parent.join("rustc-reference"));
    }
    let mut identities = files.iter().map(|p| native_executable(p)).collect::<Option<Vec<_>>>()?;
    let mut verified = Vec::new();
    for name in ["cargo", "rustc-full", "rustdoc", "rustc-reference"] {
        let path = parent.join(name);
        let hash = expected.remove(&Path::new("bin").join(name))?;
        verified.push((path.clone(), hash, stamp(&path)?));
    }
    let lib = parent.parent()?.join("lib");
    let mut libraries = Vec::new();
    for entry in fs::read_dir(&lib).ok()? {
        let path = entry.ok()?.path();
        let name = path.file_name()?.to_str()?;
        if name.contains(".so") || name.ends_with(".dylib") {
            libraries.push(path);
        }
    }
    // Every source-built package contains its matching rustc driver. Unknown
    // layouts and externally selected loaders/wrappers do not use this cache.
    if !libraries.iter().any(|p| p.file_name().unwrap().to_string_lossy().starts_with("librustc_driver-")) {
        return None;
    }
    libraries.sort();
    for path in libraries {
        let identity = stamp(&path)?;
        let hash = expected.remove(&Path::new("lib").join(path.file_name()?))?;
        identities.push(identity.clone());
        verified.push((path, hash, identity));
    }
    if !expected.is_empty() {
        return None;
    }
    Some(Tools { state: json!({"receipt":receipt, "identities":identities}), files: verified })
}

impl Tools {
    fn verify(&self, root: &Path) -> Option<()> {
        private_directory(root)?;
        let key = digest(&serde_json::to_vec(&self.state).ok()?);
        let certificate = root.join(format!("tools-{key}.json"));
        if let Some(value) = private_record(root, &certificate) {
            if value["schema"] == SCHEMA && value["tools"] == self.state {
                return Some(());
            }
        }
        for (path, expected, identity) in &self.files {
            let mut file = File::open(path).ok()?;
            let mut hash = Sha256::new();
            let mut block = [0; 65536];
            loop {
                let length = file.read(&mut block).ok()?;
                if length == 0 {
                    break;
                }
                hash.update(&block[..length]);
            }
            let actual: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
            if &actual != expected || &stamp(path)? != identity {
                return None;
            }
        }
        write_record(root, &certificate, &json!({"schema":SCHEMA,"tools":self.state}))
    }
}

fn private_record(root: &Path, path: &Path) -> Option<Value> {
    private_directory(root)?;
    let owner = root.metadata().ok()?.uid();
    let metadata = path.symlink_metadata().ok()?;
    if !metadata.is_file() || metadata.uid() != owner || metadata.mode() & 0o077 != 0 || metadata.nlink() != 1 {
        return None;
    }
    let record: Value = serde_json::from_slice(&read(path)?).ok()?;
    let payload = &record["payload"];
    if record["sha256"].as_str()? != digest(&serde_json::to_vec(payload).ok()?) {
        return None;
    }
    Some(payload.clone())
}

fn write_record(root: &Path, path: &Path, payload: &Value) -> Option<()> {
    let record = json!({"sha256":digest(&serde_json::to_vec(payload).ok()?), "payload":payload});
    let bytes = serde_json::to_vec(&record).ok()?;
    if bytes.len() as u64 > LIMIT {
        return None;
    }
    private_directory(root)?;
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let temporary = root.join(format!(".{}-{}.tmp", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
    struct Remove(PathBuf);
    impl Drop for Remove {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let remove = Remove(temporary.clone());
    let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary).ok()?;
    file.write_all(&bytes).ok()?;
    drop(file);
    fs::rename(&temporary, path).ok()?;
    drop(remove);
    Some(())
}

fn environment() -> Option<String> {
    let mut vars = std::env::vars_os().collect::<Vec<_>>();
    vars.sort();
    let mut bytes = Vec::new();
    for (key, value) in vars {
        let name = key.to_str()?;
        if name.starts_with("LD_")
            || name.starts_with("DYLD_")
            || name.starts_with("LUSTR_MIXED_")
            || name.starts_with("LUSTR_SEMANTIC_")
            || name.starts_with("RUSTC_LOG")
            || matches!(name, "RUSTFLAGS" | "RUSTC_WRAPPER" | "RUSTC_WORKSPACE_WRAPPER") && !value.is_empty()
            || name.starts_with("CARGO_")
                && !matches!(
                    name,
                    "CARGO_HOME"
                        | "CARGO_NET_OFFLINE"
                        | "CARGO_TERM_COLOR"
                        | "CARGO_BUILD_JOBS"
                        | "CARGO_INCREMENTAL"
                        | "CARGO_TARGET_DIR"
                        | "CARGO_BUILD_BUILD_DIR"
                )
        {
            return None;
        }
        for field in [key.as_bytes(), value.as_bytes()] {
            bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
            bytes.extend_from_slice(field);
        }
    }
    Some(digest(&bytes))
}

fn cargo_home(cwd: &Path) -> Option<PathBuf> {
    let home = std::env::var_os("CARGO_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|v| PathBuf::from(v).join(".cargo")))?;
    Some(if home.is_absolute() { home } else { cwd.join(home) })
}

fn config(cwd: &Path, home: &Path) -> Option<Value> {
    let mut directories = cwd.ancestors().map(|p| p.join(".cargo")).collect::<Vec<_>>();
    directories.push(home.to_owned());
    directories.sort();
    directories.dedup();
    for directory in &directories {
        for file in ["config", "config.toml"] {
            if !absent(&directory.join(file)) {
                return None;
            }
        }
    }
    // Cargo still has to be able to open its package-cache lock. A directory,
    // unreadable path or replacement must not conceal a fresh Cargo failure.
    let lock = home.join(".package-cache");
    let opened = OpenOptions::new().read(true).write(true).open(&lock).ok()?;
    if !opened.metadata().ok()?.is_file() {
        return None;
    }
    Some(json!([directories.iter().map(|p| binding(p)).collect::<Option<Vec<_>>>()?, stamp(&lock)?]))
}

fn discovery(root: &Path, explicit: &[PathBuf]) -> Option<Value> {
    let mut paths = explicit.iter().map(|p| root.join(p)).collect::<Vec<_>>();
    paths.extend(["src/main.rs", "src/lib.rs", "build.rs", "README", "README.md", "README.txt"].map(|p| root.join(p)));
    for relative in ["src/bin", "examples", "tests", "benches"] {
        let directory = root.join(relative);
        paths.push(directory.clone());
        if absent(&directory) {
            continue;
        }
        if !directory.is_dir() {
            return None;
        }
        for entry in fs::read_dir(&directory).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            paths.push(path.clone());
            if path.is_dir() {
                paths.push(path.join("main.rs"));
            }
            if paths.len() > 4096 {
                return None;
            }
        }
    }
    paths.sort();
    paths.dedup();
    Some(json!(paths.iter().map(|p| binding(p)).collect::<Option<Vec<_>>>()?))
}

fn target_bindings(metadata: &Value) -> Option<Value> {
    let mut paths = Vec::new();
    for package in metadata["packages"].as_array()? {
        for target in package["targets"].as_array()? {
            paths.push(PathBuf::from(target["src_path"].as_str()?));
        }
    }
    paths.sort();
    paths.dedup();
    Some(json!(paths.iter().map(|p| binding(p)).collect::<Option<Vec<_>>>()?))
}

#[derive(PartialEq)]
pub struct Snapshot {
    root: PathBuf,
    entry: PathBuf,
    manifest: PathBuf,
    state: Value,
    tools: Tools,
}

impl Snapshot {
    pub fn capture(request: &Request, parent: &Path, args: &[String]) -> Option<Self> {
        if !request.session.cargo_net.offline
            || std::env::var_os("LUSTR_NO_SELECTION_CACHE").is_some()
            || std::env::var_os("LUST_NO_CACHE").is_some_and(|v| v != "0")
        {
            return None;
        }
        let cwd = std::env::current_dir().ok()?;
        let shown = request.session.krate.manifest_path.as_ref()?;
        let manifest = shown.canonicalize().ok()?;
        if manifest.file_name()? != "Cargo.toml" {
            return None;
        }
        let project = manifest.parent()?;
        let manifest_text = text(&manifest)?;
        let explicit = admitted_manifest(&manifest_text)?;
        let lock = text(&project.join("Cargo.lock"))?;
        let home = cargo_home(&cwd)?;
        let environment = environment()?;
        let context = json!([SCHEMA, args, path_text(parent)?, path_text(&cwd)?, environment]);
        let tools = toolchain(parent)?;
        let state = json!({"context":context, "manifest":manifest_text, "lock":lock,
            "manifest_binding":binding(shown)?, "source":binding(&request.file)?,
            "config":config(&cwd,&home)?, "tools":tools.state,
            "discovery":discovery(project,&explicit)?});
        let base = std::env::var_os("LUST_CACHE_DIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| request.file.parent().unwrap_or(Path::new(".")).join(".lust-cache"));
        let root = base.join("cargo-selection-v1");
        let entry = root.join(format!("{}.json", digest(&serde_json::to_vec(&context).ok()?)));
        Some(Self { root, entry, manifest, state, tools })
    }

    fn admitted_metadata(&self, metadata: &Value) -> Option<()> {
        let packages = metadata["packages"].as_array()?;
        if packages.len() != 1
            || !packages[0]["dependencies"].as_array()?.is_empty()
            || metadata["workspace_members"].as_array()?.len() != 1
            || metadata["workspace_root"].as_str()? != self.manifest.parent()?.to_str()?
            || packages[0]["manifest_path"].as_str()? != self.manifest.to_str()?
            || packages[0]["source"] != Value::Null
            || packages[0]["targets"].as_array()?.iter().any(|t| t["kind"].as_array().is_none_or(|k| k.iter().any(|v| v == "custom-build")))
        {
            return None;
        }
        Some(())
    }

    pub fn load(&self) -> Option<(Value, Option<String>)> {
        let payload = private_record(&self.root, &self.entry)?;
        if payload["schema"] != SCHEMA || payload["state"] != self.state {
            return None;
        }
        let metadata = &payload["metadata"];
        self.admitted_metadata(metadata)?;
        if payload["targets"] != target_bindings(metadata)? {
            return None;
        }
        let package_id = match &payload["package_id"] {
            Value::Null => None,
            Value::String(id) => Some(id.clone()),
            _ => return None,
        };
        Some((metadata.clone(), package_id))
    }

    pub fn store(&self, metadata: &Value, package_id: Option<&str>) {
        let _ = self.store_inner(metadata, package_id);
    }

    fn store_inner(&self, metadata: &Value, package_id: Option<&str>) -> Option<()> {
        self.admitted_metadata(metadata)?;
        fs::DirBuilder::new().recursive(true).mode(0o700).create(self.root.parent()?).ok()?;
        self.tools.verify(&self.root)?;
        let payload = json!({"schema":SCHEMA, "state":self.state, "metadata":metadata,
            "package_id":package_id, "targets":target_bindings(metadata)?});
        write_record(&self.root, &self.entry, &payload)
    }
}

pub fn diagnostic(state: &str) {
    if let Some(path) = std::env::var_os("LUSTR_SELECTION_DIAGNOSTICS") {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{{\"selection_cache\":\"{state}\"}}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_requires_explicit_empty_workspace_and_literal_known_syntax() {
        let base = "[package]\nname=\"sample\"\nversion=\"0.1.0\"\n[[bin]]\npath=\"src/main.rs\"\n[workspace]\n";
        assert_eq!(admitted_manifest(base), Some(vec![PathBuf::from("src/main.rs")]));
        for tail in
            ["members=[]", "exclude=[]", "default-members=[]", "[dependencies]\nother=\"1\"", "[patch.crates-io]", "[features]\na=[]"]
        {
            assert!(admitted_manifest(&format!("{base}{tail}\n")).is_none(), "{tail}");
        }
        for head in [
            "cargo-features=[]",
            "[package]\nworkspace=\"..\"",
            "[package]\nbuild=\"other.rs\"",
            "[package]\nname=\"\"\"multi\"\"\"",
            "[package]\nname=\"a\\nb\"",
            "[package]\n\"name\"=\"a\"",
            "[package]\nversion.workspace=true",
        ] {
            assert!(admitted_manifest(&format!("{head}\n{base}")).is_none(), "{head}");
        }
        assert!(admitted_manifest(&base.replace("[workspace]", "")).is_none());
        assert!(admitted_manifest(&base.replace("src/main.rs", "../main.rs")).is_none());
    }
}
