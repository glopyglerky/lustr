//! Reuse the pinned Rust link inputs for a freestanding executable whose only
//! edits are in already assigned, closed Lust bodies.
use rustc_codegen_ssa::{CompiledModule, CompiledModules, CrateInfo};
use rustc_errors::{DiagInner, ErrorGuaranteed, TRACK_DIAGNOSTIC};
use rustc_metadata::EncodedMetadata;
use rustc_session::{Session, config::{OutFileName, OutputFilenames, OutputType}};
use rustc_span::{SourceFileHash, SourceFileHashAlgorithm};
use std::{
    fs::{self, File, OpenOptions},
    io,
    ops::Range,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::{OnceLock, atomic::{AtomicBool, Ordering}},
};

type Tracker = fn(DiagInner, &mut dyn FnMut(DiagInner) -> Option<ErrorGuaranteed>) -> Option<ErrorGuaranteed>;
static ORIGINAL_TRACKER: OnceLock<Tracker> = OnceLock::new();
static DIAGNOSTIC_SEEN: AtomicBool = AtomicBool::new(false);
static TRACKER: Tracker = |diagnostic, emit| {
    DIAGNOSTIC_SEEN.store(true, Ordering::Relaxed);
    ORIGINAL_TRACKER.get().unwrap()(diagnostic, emit)
};

const RECORD: &str = "lustr-retained-rust-link-v1";

pub(super) struct Cache {
    directory: PathBuf,
    context: String,
    _lock: File,
}

pub(super) struct Saved {
    pub modules: CompiledModules,
    pub info: CrateInfo,
    pub metadata: EncodedMetadata,
    pub outputs: OutputFilenames,
    pub dep_info: PathBuf,
}

pub(super) struct Pending {
    path: PathBuf,
    destination: PathBuf,
}

impl Drop for Pending {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn hash(data: impl AsRef<[u8]>) -> String {
    SourceFileHash::new_in_memory(SourceFileHashAlgorithm::Sha256, data).to_string()
}

fn add(key: &mut Vec<u8>, bytes: &[u8]) {
    key.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    key.extend_from_slice(bytes);
}

fn add_stamp(key: &mut Vec<u8>, path: &Path) -> io::Result<()> {
    let stat = fs::metadata(path)?;
    add(key, path.as_os_str().as_encoded_bytes());
    for value in [stat.dev(), stat.ino(), stat.len(), stat.mtime() as u64,
        stat.mtime_nsec() as u64, stat.ctime() as u64, stat.ctime_nsec() as u64]
    {
        key.extend_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

fn source_context(source: &str, bodies: &[Range<u32>], key: &mut Vec<u8>) -> Option<()> {
    let mut previous = 0;
    for body in bodies {
        let start = body.start as usize;
        let end = body.end as usize;
        add(key, source.as_bytes().get(previous..start)?);
        let body = source.as_bytes().get(start..end)?.iter().copied();
        // Keeping coordinates fixed also keeps the original meaning of spans
        // and position-sensitive macros in the surrounding Rust code.
        if !body.clone().all(|byte| byte.is_ascii() && byte != b'\r') {
            return None;
        }
        let coordinates: Vec<_> = body.map(|byte| if matches!(byte, b'\n' | b'\t') { byte } else { 0 }).collect();
        add(key, &coordinates);
        previous = end;
    }
    add(key, source.as_bytes().get(previous..)?);
    Some(())
}

fn inputs_are_closed(sess: &Session) -> bool {
    sess.env_depinfo.lock().is_empty() && sess.file_depinfo.lock().is_empty()
        && sess.source_map().files().iter().filter(|file| !file.is_imported()).count() == 1
}

fn dep_info(outputs: &OutputFilenames) -> Option<PathBuf> {
    match outputs.path(OutputType::DepInfo) {
        OutFileName::Real(path) => Some(path),
        OutFileName::Stdout => None,
    }
}

impl Cache {
    pub(super) fn open(
        sess: &Session, output: &Path, args: &[String], source: &str, bodies: &[Range<u32>], symbols: &[String],
    ) -> Option<Self> {
        // The first retained contract covers the existing freestanding
        // request. Extra Rust externs and library search paths need their
        // own resolved-input records before they can share it.
        if sess.opts.externs.iter().next().is_some() || !sess.opts.search_paths.is_empty()
            || sess.opts.cg.link_args != ["-nostdlib", "-static"]
            || sess.opts.output_types.len() != 2
            || !sess.opts.output_types.contains_key(&OutputType::Exe)
            || !sess.opts.output_types.contains_key(&OutputType::DepInfo)
            || ["LD_LIBRARY_PATH", "LD_PRELOAD"].iter().any(|name|
                std::env::var_os(name).is_some_and(|value| !value.is_empty()))
        {
            return None;
        }
        let output = output.canonicalize().ok()?;
        let mut identity = Vec::new();
        add(&mut identity, RECORD.as_bytes());
        add(&mut identity, std::env::current_dir().ok()?.as_os_str().as_encoded_bytes());
        for arg in args {
            add(&mut identity, arg.as_bytes());
        }
        let unit = hash(&identity);
        let suffix = unit.strip_prefix("sha256=")?;
        let directory = output.join(format!(".lustr-link-{suffix}"));
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false)
            .mode(0o600).open(output.join(format!(".lustr-link-{suffix}.lock"))).ok()?;
        lock.try_lock().ok()?;

        add_stamp(&mut identity, &std::env::current_exe().ok()?).ok()?;
        let mut environment: Vec<_> = std::env::vars_os().collect();
        environment.sort_unstable();
        add(&mut identity, b"environment");
        for (name, value) in environment {
            if name != "CARGO_MAKEFLAGS" {
                add(&mut identity, name.as_encoded_bytes());
                add(&mut identity, value.as_encoded_bytes());
            }
        }
        add(&mut identity, b"target libraries");
        for libraries in [sess.target_tlib_path.dir.to_path_buf(), sess.opts.sysroot.path().join("lib")] {
            add_stamp(&mut identity, &libraries).ok()?;
            let mut paths: Vec<_> = fs::read_dir(libraries).ok()?
                .map(|entry| entry.map(|entry| entry.path())).collect::<io::Result<_>>().ok()?;
            paths.sort_unstable();
            for path in paths {
                add_stamp(&mut identity, &path).ok()?;
            }
        }
        add(&mut identity, b"native symbols");
        for symbol in symbols {
            add(&mut identity, symbol.as_bytes());
        }
        add(&mut identity, b"source");
        source_context(source, bodies, &mut identity)?;
        Some(Self { directory, context: hash(identity), _lock: lock })
    }

    pub(super) fn track_diagnostics(&self) {
        if ORIGINAL_TRACKER.set(*TRACK_DIAGNOSTIC).is_ok() {
            TRACK_DIAGNOSTIC.swap(&TRACKER);
        }
    }

    pub(super) fn load(&self, sess: &Session, scratch: &Path) -> Option<Saved> {
        if !inputs_are_closed(sess) || !fs::symlink_metadata(&self.directory).is_ok_and(|meta|
            meta.is_dir() && meta.mode() & 0o077 == 0)
        {
            return None;
        }
        let record = fs::read_to_string(self.directory.join("record")).ok()?;
        let mut lines = record.lines();
        if lines.next()? != RECORD || lines.next()? != self.context {
            return None;
        }
        let data = fs::read(self.directory.join("rust.rlink")).ok()?;
        if lines.next()? != hash(&data) {
            return None;
        }
        let (mut modules, info, metadata, outputs) =
            CompiledModules::deserialize_rlink(sess, data).ok()?;
        let deps = dep_info(&outputs)?;
        if !metadata.full().is_empty() || lines.next()? != hash(fs::read(&deps).ok()?) {
            return None;
        }
        let objects = modules.modules.iter_mut().chain(modules.allocator_module.iter_mut());
        for (index, module) in objects.enumerate() {
            let name = format!("rust-{index}.o");
            let path = self.directory.join(&name);
            let bytes = fs::read(&path).ok()?;
            if module.object.as_deref() != Some(path.as_path()) || lines.next()? != hash(&bytes) {
                return None;
            }
            let input = scratch.join(name);
            fs::write(&input, bytes).ok()?;
            module.object = Some(input);
        }
        if lines.next().is_some() {
            return None;
        }
        Some(Saved { modules, info, metadata, outputs, dep_info: deps })
    }

    pub(super) fn capture(
        &self, sess: &Session, modules: &CompiledModules, info: &CrateInfo,
        metadata: &EncodedMetadata, outputs: &OutputFilenames,
    ) -> Option<Pending> {
        if !inputs_are_closed(sess) || DIAGNOSTIC_SEEN.load(Ordering::Relaxed)
            || !metadata.full().is_empty()
        {
            return None;
        }
        let path = self.directory.with_extension(format!("pending-{}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&path).ok()?;
        let pending = Pending { path, destination: self.directory.clone() };
        let mut digests = Vec::new();
        let mut capture = |module: &CompiledModule| {
            if module.global_asm_object.is_some() || module.dwarf_object.is_some()
                || module.bytecode.is_some() || module.assembly.is_some() || module.llvm_ir.is_some()
            {
                return None;
            }
            let bytes = fs::read(module.object.as_ref()?).ok()?;
            let name = format!("rust-{}.o", digests.len());
            fs::write(pending.path.join(&name), &bytes).ok()?;
            digests.push(hash(bytes));
            Some(CompiledModule {
                name: module.name.clone(), kind: module.kind, object: Some(self.directory.join(name)),
                global_asm_object: None, dwarf_object: None, bytecode: None, assembly: None, llvm_ir: None,
            })
        };
        let modules = CompiledModules {
            modules: modules.modules.iter().map(&mut capture).collect::<Option<_>>()?,
            allocator_module: match &modules.allocator_module {
                Some(module) => Some(capture(module)?),
                None => None,
            },
        };
        let rlink = pending.path.join("rust.rlink");
        CompiledModules::serialize_rlink(sess, &rlink, &modules, info, metadata, outputs).ok()?;
        let record = [RECORD.to_owned(), self.context.clone(), hash(fs::read(rlink).ok()?),
            hash(fs::read(dep_info(outputs)?).ok()?)].into_iter().chain(digests).collect::<Vec<_>>();
        fs::write(pending.path.join("record"), record.join("\n")).ok()?;
        Some(pending)
    }
}

impl Pending {
    pub(super) fn publish(self) {
        if DIAGNOSTIC_SEEN.load(Ordering::Relaxed) {
            return;
        }
        if !self.destination.exists() {
            let _ = fs::rename(&self.path, &self.destination);
            return;
        }
        // The request lock protects readers and writers. Only replace data
        // already created by this cache inside the requested output directory.
        if !fs::symlink_metadata(&self.destination).is_ok_and(|meta| meta.is_dir() && meta.mode() & 0o077 == 0)
            || !fs::read_to_string(self.destination.join("record")).is_ok_and(|text| text.lines().next() == Some(RECORD))
        {
            return;
        }
        let old = self.path.with_extension(format!("previous-{}", std::process::id()));
        if fs::rename(&self.destination, &old).is_ok() {
            if fs::rename(&self.path, &self.destination).is_ok() {
                let _ = fs::remove_dir_all(old);
            } else {
                let _ = fs::rename(old, &self.destination);
            }
        }
    }
}
