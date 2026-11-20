use std::cell::Cell;
use std::ffi::OsString;
use std::path::{Path as BorrowedPath, PathBuf as OwnedPath};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

struct Guard(Rc<Cell<usize>>);
impl Drop for Guard {
    fn drop(&mut self) { self.0.set(self.0.get() + 1); }
}
fn before() -> usize { let value = || 7usize; value() }
fn copy(source: &BorrowedPath) -> OwnedPath {
    let inferred = source.into();
    inferred
}
fn paths(n: usize) -> Vec<OwnedPath> {
    let first = OwnedPath::from("borrowed-base");
    let second = OwnedPath::from(format!("borrowed-{n}"));
    let a = first.as_path();
    let b = second.as_path();
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
        let mut owner = OwnedPath::from("borrowed-owned");
        let source = owner.as_path();
        let inferred = source.into();
        let copied: OwnedPath = inferred;
        owner.push("after-copy");
        assert_eq!(copied, OwnedPath::from("borrowed-owned"));
        assert_eq!(copy(copied.as_path()), copied);
        let convert = |source: &BorrowedPath| -> OwnedPath { let inferred = source.into(); inferred };
        assert_eq!(convert(copied.as_path()), copied);
        assert_eq!(paths(before())[1], OwnedPath::from("borrowed-7"));
        let os = OsString::from("old-owned");
        let inferred_owned = os.into();
        let old_owned: OwnedPath = inferred_owned;
        assert_eq!(old_owned, OwnedPath::from("old-owned"));
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _inner = Guard(drops.clone());
            assert_eq!(copy(copied.as_path()), copied);
            panic!("borrowed path drop witness");
        })).is_err(), "must unwind");
        assert_eq!(drops.get(), 1);
        assert_eq!(after(before()), 9);
    }
    assert_eq!(drops.get(), 2);
    println!("borrowed Path and drop: {}", drops.get());
}
