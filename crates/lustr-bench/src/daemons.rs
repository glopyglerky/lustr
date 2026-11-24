//! Ownership and observed shutdown of the original Lust daemons.
//! Shared by the fixed mixed corpus and explicitly opted-in project campaigns.
use nix::{
    errno::Errno,
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, FileTypeExt},
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type R<T> = Result<T, Box<dyn std::error::Error>>;
fn require(condition: bool, message: impl Into<String>) -> R<()> {
    if condition { Ok(()) } else { Err(message.into().into()) }
}

/// Daemons are started by the real compiler. A private socket directory and
/// child-subreaper ownership let their normal shutdown remain observable.
pub struct Daemons {
    pub sockets: PathBuf,
    done: bool,
}
impl Daemons {
    pub fn new() -> R<Self> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let sockets = std::env::temp_dir().join(format!("lmb-{}-{stamp:x}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&sockets)?;
        require(sockets.as_os_str().as_encoded_bytes().len() < 65, "temporary socket directory is too long")?;
        Ok(Self { sockets, done: false })
    }
    pub fn stop(&mut self) -> R<usize> {
        for entry in fs::read_dir(&self.sockets)? {
            let entry = entry?;
            if entry.file_type()?.is_socket() {
                // Upstream's documented, controlled stop request. Never remove
                // lock files while the corresponding process could be alive.
                fs::remove_file(entry.path())?;
            }
        }
        let start = Instant::now();
        let mut count = 0;
        let mut failure = None;
        loop {
            match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(_, 0)) => count += 1,
                Ok(WaitStatus::StillAlive) => {
                    require(start.elapsed() < Duration::from_secs(35), "owned daemon did not shut down within 35 seconds")?;
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(status) => {
                    count += 1;
                    failure = Some(format!("owned daemon ended abnormally: {status:?}"));
                }
                Err(Errno::ECHILD) => break,
                Err(Errno::EINTR) => (),
                Err(error) => return Err(error.into()),
            }
        }
        self.done = true;
        fs::remove_dir_all(&self.sockets)?;
        if let Some(failure) = failure {
            return Err(failure.into());
        }
        Ok(count)
    }
}
impl Drop for Daemons {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.stop();
        }
    }
}
