// Socket creation precedes user AsRef and pathname validation at the
// declared Rust pin. This remains observable when socket creation fails.
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

struct LoggedPath<'a>(&'a str);

impl AsRef<Path> for LoggedPath<'_> {
    fn as_ref(&self) -> &Path {
        println!("pathname evaluated");
        Path::new(self.0)
    }
}

fn main() {
    let long = "/".repeat(200);
    for path in [long.as_str(), "bad\0path"] {
        println!("connect {:?}", UnixStream::connect(LoggedPath(path)).map(|_| ()));
        println!("bind {:?}", UnixListener::bind(LoggedPath(path)).map(|_| ()));
    }
}
