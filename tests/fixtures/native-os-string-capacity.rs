use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::panic::{AssertUnwindSafe, catch_unwind};

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut value = OsString::from_vec(vec![b'a', 0xff, b'z']);
    value.reserve(19);
    println!("{:?} / {}", value.as_encoded_bytes(), value.capacity());
    let address = value.as_encoded_bytes().as_ptr();
    value.reserve(0);
    assert_eq!(address, value.as_encoded_bytes().as_ptr());
    let capacity = value.capacity();
    assert!(catch_unwind(AssertUnwindSafe(|| value.reserve(usize::MAX))).is_err());
    assert_eq!(value.as_encoded_bytes().to_vec(), vec![b'a', 0xff, b'z']);
    assert_eq!(value.capacity(), capacity);
    assert_eq!(address, value.as_encoded_bytes().as_ptr());
    assert!(catch_unwind(AssertUnwindSafe(|| value.reserve_exact(usize::MAX))).is_err());
    assert_eq!(value.as_encoded_bytes().to_vec(), vec![b'a', 0xff, b'z']);
    assert_eq!(value.capacity(), capacity);
    let mut exact = OsString::from("abc");
    exact.reserve_exact(19);
    println!("{} / {} / {:?}", exact.len(), exact.capacity(), value.as_encoded_bytes());
    value.push("after-panic");
    println!("{:?}", value.as_encoded_bytes());
}
