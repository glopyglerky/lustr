//! Disposable linker inputs are copies, not new persistent cache generations.
//!
//! The retained source and its certificate have already been published through
//! the durable path. The linker reads these copies before the compiler returns;
//! after a process or machine failure they are abandoned and recreated from the
//! retained generation. They must not mutate that generation or be published as
//! retained objects. Flushing each temporary copy adds no recovery guarantee.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) fn stage(path: &mut Option<PathBuf>, directory: &Path, next: &mut usize) -> io::Result<()> {
    let Some(source) = path else { return Ok(()) };
    let name = source.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "link input has no file name"))?;
    let target = directory.join(format!("object-{}-{}", *next, name.to_string_lossy()));
    *next += 1;
    // Preserve independent inodes: arbitrary configured linkers may modify or
    // delete their inputs. A hard link would expose the committed cache bytes.
    fs::copy(&*source, &target)?;
    // No sync_all here. This path is never a persistent cache publication.
    *source = target;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static SERIAL: AtomicU64 = AtomicU64::new(0);
            loop {
                let path = std::env::temp_dir().join(format!(
                    "lustr-link-input-test-{}-{}",
                    std::process::id(),
                    SERIAL.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(e) => panic!("create test directory: {e}"),
                }
            }
        }
        fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, bytes).unwrap();
            path
        }
        fn scratch(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            fs::create_dir(&path).unwrap();
            path
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn absent_input_does_no_work() {
        let d = Directory::new();
        let mut path = None;
        let mut next = 0;
        stage(&mut path, &d.0.join("does-not-exist"), &mut next).unwrap();
        assert_eq!(path, None);
        assert_eq!(next, 0);
        assert_eq!(fs::read_dir(&d.0).unwrap().count(), 0);
    }

    #[test]
    fn equal_basenames_stay_distinct() {
        let d = Directory::new();
        let left = d.scratch("left").join("input.o");
        let right = d.scratch("right").join("input.o");
        fs::write(&left, b"left object").unwrap();
        fs::write(&right, b"right object").unwrap();
        let target = d.scratch("link");
        let mut a = Some(left.clone());
        let mut b = Some(right.clone());
        let mut next = 0;
        stage(&mut a, &target, &mut next).unwrap();
        stage(&mut b, &target, &mut next).unwrap();
        assert_ne!(a, b);
        assert_eq!(next, 2);
        assert_eq!(fs::read(a.unwrap()).unwrap(), b"left object");
        assert_eq!(fs::read(b.unwrap()).unwrap(), b"right object");
        assert_eq!(fs::read(left).unwrap(), b"left object");
        assert_eq!(fs::read(right).unwrap(), b"right object");
    }

    #[test]
    fn linker_writes_do_not_change_retained_bytes() {
        let d = Directory::new();
        let source = d.file("retained.o", b"committed object");
        let mut path = Some(source.clone());
        stage(&mut path, &d.scratch("link"), &mut 0).unwrap();
        let staged = path.unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_ne!(fs::metadata(&source).unwrap().ino(), fs::metadata(&staged).unwrap().ino());
        }
        fs::write(&staged, b"linker mutated its input").unwrap();
        assert_eq!(fs::read(&source).unwrap(), b"committed object");
    }

    #[test]
    fn linker_cleanup_does_not_remove_retained_bytes() {
        let d = Directory::new();
        let source = d.file("retained.o", b"committed object");
        let mut path = Some(source.clone());
        stage(&mut path, &d.scratch("link"), &mut 0).unwrap();
        fs::remove_file(path.unwrap()).unwrap();
        assert_eq!(fs::read(source).unwrap(), b"committed object");
    }

    #[test]
    fn failed_copy_does_not_repoint_the_recipe() {
        let d = Directory::new();
        let source = d.0.join("missing.o");
        let mut path = Some(source.clone());
        assert!(stage(&mut path, &d.scratch("link"), &mut 0).is_err());
        assert_eq!(path, Some(source));
    }

    #[test]
    fn missing_scratch_directory_does_not_change_the_source() {
        let d = Directory::new();
        let source = d.file("retained.o", b"committed object");
        let mut path = Some(source.clone());
        assert!(stage(&mut path, &d.0.join("missing"), &mut 0).is_err());
        assert_eq!(path, Some(source.clone()));
        assert_eq!(fs::read(source).unwrap(), b"committed object");
    }

    #[test]
    fn empty_object_is_still_copied() {
        let d = Directory::new();
        let source = d.file("empty.o", b"");
        let mut path = Some(source.clone());
        stage(&mut path, &d.scratch("link"), &mut 0).unwrap();
        assert_ne!(path, Some(source));
        assert_eq!(fs::metadata(path.unwrap()).unwrap().len(), 0);
    }

    #[test]
    fn abandoned_inputs_are_rebuilt_from_the_committed_generation() {
        let d = Directory::new();
        let bytes: Vec<u8> = (0..=255).cycle().take(65536).collect();
        let source = d.file("retained.o", &bytes);
        fs::File::open(&source).unwrap().sync_all().unwrap();
        fs::File::open(&d.0).unwrap().sync_all().unwrap();
        let first = d.scratch("abandoned-link");
        let mut path = Some(source.clone());
        stage(&mut path, &first, &mut 0).unwrap();
        fs::write(path.unwrap(), b"partial failed link").unwrap();
        fs::remove_dir_all(first).unwrap();
        let mut retry = Some(source.clone());
        stage(&mut retry, &d.scratch("retry"), &mut 0).unwrap();
        assert_eq!(fs::read(retry.unwrap()).unwrap(), bytes);
        assert_eq!(fs::read(source).unwrap(), bytes);
    }
}
