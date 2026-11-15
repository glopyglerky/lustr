use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut path = PathBuf::from(OsString::from_vec(vec![b'a', 0xff, b'z']));
    path.reserve(31);
    let capacity = path.capacity();
    assert!(capacity >= 34);
    let address = path.as_os_str().as_encoded_bytes().as_ptr();
    path.reserve(0);
    path.reserve_exact(0);
    assert_eq!(address, path.as_os_str().as_encoded_bytes().as_ptr());
    assert!(catch_unwind(AssertUnwindSafe(|| path.reserve(usize::MAX))).is_err());
    assert_eq!(path.capacity(), capacity);
    assert_eq!(path.as_os_str().as_encoded_bytes().as_ptr(), address);
    assert_eq!(path.as_os_str().as_encoded_bytes().to_vec(), vec![b'a', 0xff, b'z']);
    assert!(catch_unwind(AssertUnwindSafe(|| path.reserve_exact(usize::MAX))).is_err());
    assert_eq!(path.capacity(), capacity);
    assert_eq!(path.as_os_str().as_encoded_bytes().as_ptr(), address);
    path.push("after-panic");
    println!("{:?}", path.as_os_str().as_encoded_bytes());
    let mut exact = PathBuf::from("abc");
    exact.reserve_exact(19);
    assert!(exact.capacity() >= 22);
    exact.push("child");
    assert_eq!(exact, PathBuf::from("abc/child"));
    println!("{}", exact.display());
}
