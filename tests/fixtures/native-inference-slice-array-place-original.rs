use std::cell::Cell;
use std::ffi::{OsStr as BorrowedOs, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path as BorrowedPath, PathBuf as OwnedPath};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

struct Guard(Rc<Cell<usize>>);
impl Drop for Guard {
    fn drop(&mut self) { self.0.set(self.0.get() + 1); }
}
fn before() -> usize { let value = || 7usize; value() }
fn copy(source: &BorrowedOs) -> OwnedPath {
    let inferred = source.into();
    inferred
}
fn paths(n: usize) -> Vec<OwnedPath> {
    let first = OsString::from("borrowed-base");
    let second = OsString::from(format!("borrowed-{n}"));
    let a = first.as_os_str();
    let b = second.as_os_str();
    let mut result = Vec::new();
    result.extend([a.into(), b.into()]);
    result
}
fn after(n: usize) -> usize { let add = move |x: usize| x + n; add(2) }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let drops = Rc::new(Cell::new(0));
    {
        let _outer = Guard(drops.clone());
        let mut owner = OsString::from("borrowed-owned");
        let source = owner.as_os_str();
        let inferred = source.into();
        let copied: OwnedPath = inferred;
        owner.push("after-copy");
        assert_eq!(copied, OwnedPath::from("borrowed-owned"));
        assert_eq!(copy(copied.as_os_str()), copied);
        let convert = |source: &BorrowedOs| -> OwnedPath { let inferred = source.into(); inferred };
        assert_eq!(convert(copied.as_os_str()), copied);
        assert_eq!(paths(before())[1], OwnedPath::from("borrowed-7"));
        let raw = OsString::from_vec(vec![b'r', 0xff, b'w']);
        let source = raw.as_os_str();
        let inferred = source.into();
        let bytes: OwnedPath = inferred;
        let expected_raw = &[b'r', 0xff, b'w'];
        assert_eq!(bytes.as_os_str().as_bytes(), expected_raw);
        let expected_copy = &[b'x', 0xfe];
        assert_eq!(copy(BorrowedOs::from_bytes(&[b'x', 0xfe])).as_os_str().as_bytes(), expected_copy);
        let prior = |source: &BorrowedPath| -> OwnedPath { let inferred = source.into(); inferred };
        assert_eq!(prior(copied.as_path()), copied);
        let os = OsString::from("old-owned");
        let inferred_owned = os.into();
        let old_owned: OwnedPath = inferred_owned;
        assert_eq!(old_owned, OwnedPath::from("old-owned"));
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _inner = Guard(drops.clone());
            assert_eq!(copy(copied.as_os_str()), copied);
            panic!("borrowed path drop witness");
        })).is_err(), "must unwind");
        assert_eq!(drops.get(), 1);
        assert_eq!(after(before()), 9);
    }
    assert_eq!(drops.get(), 2);
    println!("borrowed OsStr and drop: {}", drops.get());
}
