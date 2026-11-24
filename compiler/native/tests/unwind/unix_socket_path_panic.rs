use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

struct PanicPath;

impl AsRef<Path> for PanicPath {
    fn as_ref(&self) -> &Path {
        println!("pathname evaluated");
        panic!("pathname panic")
    }
}

#[cfg(target_os = "linux")]
fn descriptor_count() -> usize {
    std::fs::read_dir("/proc/self/fd").unwrap().count()
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    #[cfg(target_os = "linux")]
    let before = descriptor_count();
    for listener in [false, true] {
        let result = std::panic::catch_unwind(|| {
            if listener {
                UnixListener::bind(PanicPath).map(|_| ())
            } else {
                UnixStream::connect(PanicPath).map(|_| ())
            }
        });
        match result {
            Ok(result) => println!("socket result: {:?}", result),
            Err(_) => println!("pathname panic caught"),
        }
        #[cfg(target_os = "linux")]
        assert_eq!(descriptor_count(), before, "socket descriptor leaked");
    }
}
