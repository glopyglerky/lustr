//! A versioned source-library component has its own pristine daemon. Explicit
//! user source overrides retain upstream's always-cold behavior.
use std::hash::{Hash, Hasher};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn inputs(library: &Path) -> std::io::Result<Vec<String>> {
    fn visit(base: &Path, directory: &Path, paths: &mut Vec<String>, prefix: &str) -> std::io::Result<()> {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                visit(base, &entry.path(), paths, prefix)?;
            } else if kind.is_file() {
                let path = entry.path();
                let relative =
                    path.strip_prefix(base).unwrap().to_str().ok_or_else(|| std::io::Error::other("non-UTF8 installed source"))?;
                paths.push(format!("{prefix}{relative}"));
            } else {
                return Err(std::io::Error::other("nonregular installed source"));
            }
        }
        Ok(())
    }
    let mut paths = vec!["../lustr-native.id".to_owned()];
    visit(library, &library.join("std"), &mut paths, "")?;
    let baseline = library.join("../lust/library");
    visit(&baseline, &baseline, &mut paths, "../lust/library/")?;
    if std::fs::read_link(library.join("deps"))? != Path::new("../lust/library/deps") {
        return Err(std::io::Error::other("unexpected adapter source link"));
    }
    paths.sort();
    Ok(paths)
}

pub struct Sealed {
    pub core: PathBuf,
    pub socket: PathBuf,
    pub identity: String,
}

pub fn sealed(parent: &Path, socket: &Path) -> Option<Sealed> {
    let library = parent.join("../lib/lustr");
    let support = parent.join("../lib/lustr-expanded/lib/lust");
    if std::fs::read_link(support.join("library")).ok()? != Path::new("../../../lustr")
        || std::fs::read_link(support.join("tcc")).ok()? != Path::new("../../../lust/tcc")
    {
        return None;
    }
    let manifest = library.join("sources.sha256");
    let bytes = std::fs::read(&manifest).ok()?;
    let declared: Option<Vec<_>> = std::str::from_utf8(&bytes)
        .ok()?
        .lines()
        .map(|line| {
            let (hash, name) = line.split_once("  ")?;
            (hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())).then(|| name.to_owned())
        })
        .collect();
    let inputs = inputs(&library).ok()?;
    if declared? != inputs {
        return None;
    }
    // Verify every source and derive the manifest identity in one child. The
    // emitted source records must exactly equal the sealed manifest, including
    // inventory order; the final record hashes that manifest itself.
    let checked =
        crate::owned_child::output(Command::new("sha256sum").current_dir(&library).arg("--").args(&inputs).arg("sources.sha256")).ok()?;
    if !checked.status.success() {
        return None;
    }
    let record = checked.stdout.strip_prefix(bytes.as_slice())?;
    let identity = std::str::from_utf8(record).ok()?.strip_suffix("  sources.sha256\n")?.to_owned();
    if identity.len() != 64 || !identity.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    if std::fs::read(&manifest).ok()? != bytes {
        return None;
    }
    let core = parent.join("../lib/lustr-expanded/bin/lust-core");
    let component = std::fs::symlink_metadata(&core).ok()?;
    let original = std::fs::metadata(parent.join("lust-core")).ok()?;
    // Upstream's wire identity includes the real core inode. The component
    // must be a hard link to that same immutable binary, not a copied peer
    // with a forged identity. Its executable path supplies its support prefix.
    if !component.file_type().is_file()
        || (component.dev(), component.ino(), component.size(), component.mtime(), component.mtime_nsec())
            != (original.dev(), original.ino(), original.size(), original.mtime(), original.mtime_nsec())
    {
        return None;
    }
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    socket.hash(&mut hash);
    identity.hash(&mut hash);
    let socket = socket.with_file_name(format!("lx-{:016x}.sock", hash.finish()));
    Some(Sealed { core, socket, identity })
}
