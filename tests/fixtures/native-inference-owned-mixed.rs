use std::cell::Cell;
use std::ffi::OsString as OwnedOs;
use std::path::PathBuf as OwnedPath;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

struct Guard(Rc<Cell<usize>>);
impl Drop for Guard {
    fn drop(&mut self) { self.0.set(self.0.get() + 1); }
}
fn before() -> usize { let value = || 7usize; value() }
fn roundtrip(n: usize) -> OwnedOs {
    let source = OwnedOs::from(format!("owned-{n}"));
    let inferred = source.into();
    let path: OwnedPath = inferred;
    let back = path.into();
    let result: OwnedOs = back;
    result
}
fn paths(n: usize) -> Vec<OwnedPath> {
    let mut result = Vec::new();
    result.extend([OwnedOs::from("base").into(), OwnedOs::from(format!("item-{n}")).into()]);
    result
}
fn arguments() -> Vec<OwnedOs> {
    let mut result = Vec::new();
    result.extend([OwnedPath::from("path-base").into(), OwnedPath::from("path-next").into()]);
    result
}
fn after(n: usize) -> usize { let add = move |x: usize| x + n; add(2) }
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let drops = Rc::new(Cell::new(0));
    {
        let _outer = Guard(drops.clone());
        assert_eq!(roundtrip(before()), OwnedOs::from("owned-7"));
        assert_eq!(paths(before())[1], OwnedPath::from("item-7"));
        assert_eq!(arguments()[0], OwnedOs::from("path-base"));
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _inner = Guard(drops.clone());
            let during = roundtrip(11);
            assert_eq!(during, OwnedOs::from("owned-11"));
            panic!("owned path drop witness");
        })).is_err(), "must unwind");
        assert_eq!(drops.get(), 1);
        assert_eq!(after(before()), 9);
    }
    assert_eq!(drops.get(), 2);
    println!("owned PathBuf/OsString roundtrip and drop: {}", drops.get());
}
