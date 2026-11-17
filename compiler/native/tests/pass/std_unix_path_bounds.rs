// A pathname reserves one byte of sockaddr_un.sun_path for its terminator.
// Linux has 108 path bytes; Darwin has 104. Count bytes, not Unicode chars.
use std::io::ErrorKind;
#[cfg(target_os = "linux")]
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
use std::path::Path;

fn round_trip(path: &str) {
    let addr = SocketAddr::from_pathname(path).unwrap();
    assert_eq!(addr.as_pathname(), Some(Path::new(path)));
    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind_addr(&addr).unwrap();
    let client = UnixStream::connect_addr(&addr).unwrap();
    let (server, peer) = listener.accept().unwrap();
    assert!(peer.is_unnamed());
    assert_eq!(listener.local_addr().unwrap().as_pathname(), Some(Path::new(path)));
    assert_eq!(client.peer_addr().unwrap().as_pathname(), Some(Path::new(path)));
    drop(server);
    drop(client);
    drop(listener);
    std::fs::remove_file(path).unwrap();
}

fn main() {
    let capacity = if cfg!(target_os = "macos") { 104usize } else { 108usize };
    let maximum = capacity - 1;
    let ascii = "p".repeat(maximum);
    round_trip(&ascii);
    let unicode = format!("{}x", "é".repeat(maximum / 2));
    assert_eq!(unicode.len(), maximum);
    round_trip(&unicode);

    // Include every byte count that the previous Linux bound wrongly admitted
    // on Darwin, plus both targets' immediately adjacent bounds.
    for n in [103usize, 104, 105, 106, 107, 108, 109] {
        let path = "p".repeat(n);
        let result = SocketAddr::from_pathname(&path);
        assert_eq!(result.is_ok(), n < capacity);
        if n >= capacity {
            assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
            assert_eq!(UnixListener::bind(&path).unwrap_err().kind(), ErrorKind::InvalidInput);
            assert_eq!(UnixStream::connect(&path).unwrap_err().kind(), ErrorKind::InvalidInput);
        }
    }
    let too_many_bytes = "é".repeat(capacity / 2);
    assert_eq!(too_many_bytes.len(), capacity);
    assert_eq!(SocketAddr::from_pathname(&too_many_bytes).unwrap_err().kind(), ErrorKind::InvalidInput);

    for path in ["\0", "p\0q"] {
        println!("{:?}", SocketAddr::from_pathname(path).unwrap_err());
        assert_eq!(UnixListener::bind(path).unwrap_err().kind(), ErrorKind::InvalidInput);
        assert_eq!(UnixStream::connect(path).unwrap_err().kind(), ErrorKind::InvalidInput);
    }
    let empty = SocketAddr::from_pathname("").unwrap();
    assert!(empty.is_unnamed());
    assert_eq!(empty.as_pathname(), None);
    println!("{:?}", empty);
    #[cfg(target_os = "linux")]
    {
        // An empty abstract name is named, unlike an empty pathname or an
        // unbound endpoint. Check the address after the kernel round trip.
        let addr = SocketAddr::from_abstract_name("").unwrap();
        let listener = UnixListener::bind_addr(&addr).unwrap();
        let local = listener.local_addr().unwrap();
        assert!(!local.is_unnamed());
        assert_eq!(local.as_abstract_name(), Some(&b""[..]));
        let client = UnixStream::connect_addr(&addr).unwrap();
        let (server, peer) = listener.accept().unwrap();
        assert!(peer.is_unnamed());
        assert_eq!(client.peer_addr().unwrap().as_abstract_name(), Some(&b""[..]));
        drop(server);
        drop(client);
        drop(listener);
        println!("empty abstract socket round trip passed");
    }
    println!("pathname bounds and real maximum-length socket round trips passed");
}
