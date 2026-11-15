use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

fn main() {
    let mut value = OsString::from_vec(vec![b'd', b'/', 0xff, b'z']);
    value.reserve(64);
    let first = PathBuf::from(&value);
    let second = PathBuf::from(value.as_os_str());
    let third = PathBuf::from(&first);
    assert_eq!(first, second);
    assert_eq!(second, third);
    assert_eq!(first.as_os_str(), value.as_os_str());
    println!("{:?}", first.as_os_str().as_encoded_bytes());
    println!("{} / {} / {}", first.capacity(), second.capacity(), third.capacity());
    value.push("later");
    println!("{:?} / {:?}", first.as_os_str().as_encoded_bytes(), value.as_encoded_bytes());
    let text = String::from("unchanged");
    println!("{} / {}", PathBuf::from(text.clone()).display(), PathBuf::from(&text).display());
}
