//! Host services shared by the thin client and native daemon. Compiler sessions,
//! semantic state and warm caches do not depend on these process/IPC details.
#![allow(dead_code)]

use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
pub type SigSet = [u64; 16];
#[cfg(target_os = "macos")]
pub type SigSet = u32;
#[cfg(target_os = "linux")]
pub const EMPTY_SIGSET: SigSet = [0; 16];
#[cfg(target_os = "macos")]
pub const EMPTY_SIGSET: SigSet = 0;
pub const SIG_BLOCK: i32 = if cfg!(target_os = "macos") { 1 } else { 0 };
pub const SIG_SETMASK: i32 = if cfg!(target_os = "macos") { 3 } else { 2 };
pub const WNOWAIT: i32 = if cfg!(target_os = "macos") { 0x20 } else { 0x0100_0000 };
pub const RTLD_NOW_LOCAL: i32 = if cfg!(target_os = "macos") { 2 | 4 } else { 2 };
pub const NATIVE_LIBRARY: &str = if cfg!(target_os = "macos") { "liblustr_native_core.dylib" } else { "liblustr_native_core.so" };
pub const INFERENCE_LIBRARY: &str = if cfg!(target_os = "macos") { "liblustr_inference_core.dylib" } else { "liblustr_inference_core.so" };
pub const SOCKET_PATH_CAPACITY: usize = if cfg!(target_os = "macos") { 104 } else { 108 };
#[cfg(target_os = "linux")]
pub type PollCount = u64;
#[cfg(target_os = "macos")]
pub type PollCount = u32;

pub fn sha256_command() -> std::process::Command {
    if cfg!(target_os = "macos") {
        let mut command = std::process::Command::new("shasum");
        command.args(["-a", "256"]);
        command
    } else {
        std::process::Command::new("sha256sum")
    }
}

unsafe extern "C" {
    fn geteuid() -> u32;
    fn kill(pid: i32, signal: i32) -> i32;
}

/// Only ESRCH proves a positive process ID gone. EPERM and unknown errors keep
/// an owner's files; a cache sweep must never delete a live daemon's library.
pub fn process_exists(pid: u32) -> bool {
    let Ok(pid) = i32::try_from(pid) else { return false };
    if pid <= 0 {
        return false;
    }
    // SAFETY: signal zero does not deliver a signal; pid is positive.
    unsafe { kill(pid, 0) == 0 || io::Error::last_os_error().raw_os_error() != Some(3) }
}

/// Owns any directory descriptor needed by a short socket address. Keeping the
/// handle alive is part of the address contract; startup passes it across exec.
pub struct DaemonSocket<'a> {
    path: Cow<'a, Path>,
    directory: Option<File>,
}

impl<'a> DaemonSocket<'a> {
    pub fn resolve(requested: &'a Path) -> io::Result<Self> {
        if requested.as_os_str().as_bytes().len() < SOCKET_PATH_CAPACITY {
            return Ok(Self { path: Cow::Borrowed(requested), directory: None });
        }
        #[cfg(target_os = "linux")]
        {
            // Preserve Linux's exact requested namespace, including external
            // removal as a stop request, without changing cwd on any thread.
            let parent = requested.parent().ok_or_else(|| io::Error::other("socket needs a parent directory"))?;
            let name = requested.file_name().ok_or_else(|| io::Error::other("socket needs a file name"))?;
            let directory = File::open(parent)?;
            if !directory.metadata()?.is_dir() {
                return Err(io::Error::other("socket parent is not a directory"));
            }
            let path = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd())).join(name);
            if path.as_os_str().as_bytes().len() >= SOCKET_PATH_CAPACITY {
                return Err(io::Error::other("socket file name exceeds the host address limit"));
            }
            Ok(Self { path: Cow::Owned(path), directory: Some(directory) })
        }
        #[cfg(target_os = "macos")]
        {
            // Darwin has no /proc descriptor path. Keep a deterministic short
            // address under a private per-user directory. Hash the canonical
            // requested namespace, so unrelated workspaces cannot share state.
            let root = PathBuf::from(format!("/tmp/lustr-daemons-{}", unsafe { geteuid() }));
            Ok(Self { path: Cow::Owned(private_socket_path(requested, &root)?), directory: None })
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn inherited_fd(&self) -> Option<RawFd> {
        self.directory.as_ref().map(AsRawFd::as_raw_fd)
    }
}

fn private_directory(root: &Path) -> io::Result<()> {
    match std::fs::DirBuilder::new().mode(0o700).create(root) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let metadata = root.symlink_metadata()?;
    if !metadata.is_dir() || metadata.uid() != unsafe { geteuid() } || metadata.mode() & 0o077 != 0 {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "daemon runtime directory must be private and owned by this user"));
    }
    Ok(())
}

fn private_socket_path(requested: &Path, root: &Path) -> io::Result<PathBuf> {
    let parent = requested.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let name = requested.file_name().ok_or_else(|| io::Error::other("socket needs a file name"))?;
    let canonical = parent.canonicalize()?.join(name);
    // Two independently seeded stable lanes; this is a namespace key, not an
    // authentication boundary. The private directory enforces that boundary.
    let mut a = 0xcbf2_9ce4_8422_2325u64;
    let mut b = 0x8422_2325_cbf2_9ce4u64;
    for byte in canonical.as_os_str().as_bytes() {
        a = (a ^ *byte as u64).wrapping_mul(0x100_0000_01b3);
        b = (b ^ *byte as u64).rotate_left(7).wrapping_mul(0x9e37_79b1_85eb_ca87);
    }
    let path = root.join(format!("{a:016x}{b:016x}.sock"));
    if path.as_os_str().as_bytes().len() >= 104 {
        return Err(io::Error::other("daemon runtime directory exceeds the host socket address limit"));
    }
    private_directory(root)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    struct Temporary(PathBuf);
    impl Temporary {
        fn new(name: &str) -> Self {
            // macOS's per-user TMPDIR can itself exceed sockaddr_un's limit.
            let path = Path::new("/tmp").join(format!("lustr-host-{name}-{}-{:?}", std::process::id(), std::thread::current().id()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn short_socket_keeps_its_namespace_without_a_descriptor() {
        let requested = Path::new(".lust-daemon-test.sock");
        let socket = DaemonSocket::resolve(requested).unwrap();
        assert_eq!(socket.path(), requested);
        assert!(socket.inherited_fd().is_none());
    }

    #[test]
    fn private_socket_is_stable_and_isolates_workspaces() {
        let temp = Temporary::new("namespace");
        let root = temp.0.join("run");
        let first = temp.0.join("one");
        let second = temp.0.join("two");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        let path = private_socket_path(&first.join("daemon.sock"), &root).unwrap();
        assert_eq!(path, private_socket_path(&first.join("./daemon.sock"), &root).unwrap());
        assert_ne!(path, private_socket_path(&second.join("daemon.sock"), &root).unwrap());
        assert_ne!(path, private_socket_path(&first.join("other.sock"), &root).unwrap());
        assert!(path.as_os_str().as_bytes().len() < 104);
        assert_eq!(root.symlink_metadata().unwrap().mode() & 0o777, 0o700);
    }

    #[test]
    fn private_socket_rejects_shared_and_symlinked_directories() {
        let temp = Temporary::new("permissions");
        let root = temp.0.join("shared");
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o777)).unwrap();
        assert_eq!(private_socket_path(&temp.0.join("daemon.sock"), &root).unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        let link = temp.0.join("link");
        symlink(&root, &link).unwrap();
        assert_eq!(private_socket_path(&temp.0.join("daemon.sock"), &link).unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn liveness_never_probes_a_process_group() {
        assert!(process_exists(std::process::id()));
        assert!(!process_exists(0));
        assert!(!process_exists(u32::MAX));
    }
}
