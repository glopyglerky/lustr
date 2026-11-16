//! A uniquely created directory owned through normal return and unwinding.
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) struct Scratch(PathBuf);

impl Scratch {
    pub(super) fn new(parent: &Path) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(io::Error::other)?.as_nanos();
        for _ in 0..128 {
            let serial = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("lustr-inference-{}-{epoch}-{serial}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(io::ErrorKind::AlreadyExists, "cannot create unique inference scratch"))
    }

    pub(super) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // This path was atomically created by this guard, never adopted.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_nested_outputs_on_return_and_unwind_without_touching_peers() {
        let parent = Scratch::new(&std::env::temp_dir()).unwrap();
        let unrelated = parent.path().join(format!("lustr-inference-{}", std::process::id()));
        std::fs::create_dir(&unrelated).unwrap();
        std::fs::write(unrelated.join("keep"), b"preexisting").unwrap();
        let normal_path;
        {
            let normal = Scratch::new(parent.path()).unwrap();
            normal_path = normal.path().to_owned();
            std::fs::create_dir(normal.path().join("objects")).unwrap();
            std::fs::write(normal.path().join("objects/unit.o"), b"object").unwrap();
        }
        assert!(!normal_path.exists());
        let unwound_path = std::cell::RefCell::new(PathBuf::new());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let scratch = Scratch::new(parent.path()).unwrap();
            *unwound_path.borrow_mut() = scratch.path().to_owned();
            std::fs::write(scratch.path().join("unit.o"), b"partial").unwrap();
            panic!("compiler work unwound");
        }));
        assert!(result.is_err());
        assert!(!unwound_path.borrow().exists());
        assert_eq!(std::fs::read(unrelated.join("keep")).unwrap(), b"preexisting");
        let a = Scratch::new(parent.path()).unwrap();
        let b = Scratch::new(parent.path()).unwrap();
        assert_ne!(a.path(), b.path());
    }
}
