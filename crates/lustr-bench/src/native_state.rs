//! Explicitly owned retained state for complete-project Lustr campaigns.
//! Nothing here is enabled for historical generic-runner specifications.
use super::{R, daemons::Daemons, hash};
use nix::sys::prctl::set_child_subreaper;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};

pub const OWNED_KEYS: &[&str] = &[
    "TMPDIR",
    "XDG_CACHE_HOME",
    "LUSTR_MIXED_CACHE_DIR",
    "LUST_CACHE_DIR",
    "LUST_DAEMON_SOCKET",
    "LUST_DAEMON_IDLE",
    "LUST_NO_DAEMON",
    "LUST_NO_CACHE",
];

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub kind: String,
    pub toolchain: PathBuf,
    pub component_manifest: PathBuf,
    pub build_receipt: PathBuf,
}

impl Policy {
    /// Bind the constructor's actual optimized component receipt and every
    /// listed compiler binary. External-input guards rehash these after work.
    pub fn validate(&self, external_inputs: &[PathBuf]) -> R<()> {
        if self.kind != "owned-lustr-mixed-v1" || !self.toolchain.is_absolute() {
            return Err("unknown native-state policy or relative toolchain".into());
        }
        for path in [&self.component_manifest, &self.build_receipt] {
            if !path.is_absolute() || !external_inputs.contains(path) || !path.is_file() {
                return Err("native component receipts must be existing, hash-guarded external inputs".into());
            }
        }
        let manifest = fs::read_to_string(&self.component_manifest)?;
        let receipt = fs::read_to_string(&self.build_receipt)?;
        let values = |text: &str, prefix: &str| -> Vec<String> {
            text.lines().filter_map(|line| line.strip_prefix(prefix).map(str::to_owned)).collect()
        };
        let declared = values(&manifest, "Native build=");
        let built = values(&receipt, "cargo:rustc-env=LUST_BUILD_HASH=");
        if declared.len() != 1
            || declared != built
            || declared[0].len() != 16
            || !declared[0].bytes().all(|b| b.is_ascii_hexdigit())
            || values(&manifest, "Cargo artifact profile=") != ["release"]
        {
            return Err("missing or inconsistent optimized native build receipt".into());
        }
        for name in ["lustr", "rustc-full", "lust", "lust-core", "cargo-lustr"] {
            let path = self.toolchain.join("bin").join(name);
            if !external_inputs.contains(&path) {
                return Err(format!("native component is not a guarded input: {name}").into());
            }
            let expected = format!("{}  {}", hash(&path)?, path.to_str().ok_or("component path is not UTF-8")?);
            if manifest.lines().filter(|line| *line == expected).count() != 1 {
                return Err(format!("native component differs from its constructor receipt: {name}").into());
            }
        }
        Ok(())
    }
}

pub struct Runtime {
    owner: Option<Daemons>,
    targets: PathBuf,
    epoch: usize,
    pub receipts: Vec<Value>,
}

impl Runtime {
    pub fn new(targets: &Path) -> R<Self> {
        fs::create_dir(targets)?;
        let targets = fs::canonicalize(targets)?;
        set_child_subreaper(true)?;
        Ok(Self { owner: Some(Daemons::new()?), targets, epoch: 0, receipts: Vec::new() })
    }

    fn owned_target(&self, target: &Path) -> R<PathBuf> {
        let metadata = fs::symlink_metadata(target)?;
        let canonical = fs::canonicalize(target)?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || canonical != target
            || canonical.parent() != Some(self.targets.as_path())
        {
            return Err("native state target escaped its owned campaign directory".into());
        }
        Ok(canonical)
    }

    pub fn prepare(&mut self, lane: &str, target: &Path, command: usize) -> R<BTreeMap<OsString, OsString>> {
        let owner = self.owner.as_ref().ok_or("native state has already shut down")?;
        let root = self.owned_target(target)?;
        let mut env = BTreeMap::new();
        for (key, name) in
            [("TMPDIR", "tmp"), ("XDG_CACHE_HOME", "xdg"), ("LUSTR_MIXED_CACHE_DIR", "mixed-cache"), ("LUST_CACHE_DIR", "native-cache")]
        {
            let path = root.join(name);
            match fs::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(error.into()),
            }
            let metadata = fs::symlink_metadata(&path)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() || fs::canonicalize(&path)?.parent() != Some(root.as_path()) {
                return Err("native state path escaped its owned target".into());
            }
            env.insert(OsString::from(key), path.into_os_string());
        }
        let socket = owner.sockets.join(format!("{lane}.sock"));
        if socket.as_os_str().as_encoded_bytes().len() >= 104 {
            return Err("native daemon socket path is too long".into());
        }
        env.insert("LUST_DAEMON_SOCKET".into(), socket.clone().into_os_string());
        env.insert("LUST_DAEMON_IDLE".into(), "120s".into());
        self.receipts.push(json!({"event":"prepared","epoch":self.epoch,"command":command,
            "lane":lane,"target":root,"socket":socket,"cache_scope":"owned-lane-target",
            "environment":env.iter().map(|(key,value)|(key.to_string_lossy(),value.to_string_lossy())).collect::<BTreeMap<_,_>>()}));
        Ok(env)
    }

    fn stop(&mut self, reason: &str) -> R<()> {
        let Some(mut owner) = self.owner.take() else { return Err("duplicate native-state shutdown".into()) };
        match owner.stop() {
            Ok(count) => {
                self.receipts.push(json!({"event":"shutdown","epoch":self.epoch,"reason":reason,
                    "observed_successful_exits":count,"remaining_children":0,"status":"passed"}));
                Ok(())
            }
            Err(error) => {
                self.receipts.push(json!({"event":"shutdown","epoch":self.epoch,"reason":reason,
                    "status":"failed","error":error.to_string()}));
                Err(error)
            }
        }
    }

    /// Stop and reap owned processes BEFORE removing Cargo, native, mixed and
    /// temporary state. OS page caches and dependency source caches stay warm.
    pub fn reset(&mut self, target: &Path, reason: &str) -> R<()> {
        self.owned_target(target)?;
        self.stop(reason)?;
        fs::remove_dir_all(target)?;
        fs::create_dir(target)?;
        if fs::read_dir(target)?.next().is_some() {
            return Err("fresh lane target is not empty".into());
        }
        self.epoch += 1;
        self.owner = Some(Daemons::new()?);
        self.receipts.push(json!({"event":"cleared","epoch":self.epoch,"reason":reason,
            "target":target,"entries_after_clear":0,"cache_scope":"cargo+mixed+native+tmp+xdg"}));
        Ok(())
    }

    pub fn finish(&mut self) -> R<()> {
        self.stop("final")
    }
}
