// Every combination of read/write/append/truncate/create/create_new on an
// existing and a missing file. This includes the order of invalid-flag errors.
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::PathBuf;

struct OwnedDirectory(PathBuf);
impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn main() {
    let mut attempt = 0u64;
    let directory = loop {
        let path = std::env::temp_dir().join(format!("lust-open-options-{}-{}", std::process::id(), attempt));
        match fs::create_dir(&path) {
            Ok(()) => break OwnedDirectory(path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => attempt += 1,
            Err(error) => panic!("create owned directory: {}", error),
        }
    };
    let path = directory.0.join("case");
    for flags in 0u32..64 {
        for existing in [false, true] {
            if existing {
                fs::write(&path, b"original").unwrap();
            }
            let result = OpenOptions::new()
                .read(flags & 1 != 0)
                .write(flags & 2 != 0)
                .append(flags & 4 != 0)
                .truncate(flags & 8 != 0)
                .create(flags & 16 != 0)
                .create_new(flags & 32 != 0)
                .open(&path);
            match result {
                Ok(file) => println!("{} {} ok {}", flags, existing, file.metadata().unwrap().len()),
                Err(error) => println!("{} {} {:?}: {}", flags, existing, error.kind(), error),
            }
            if path.exists() {
                fs::remove_file(&path).unwrap();
            }
        }
    }
}
