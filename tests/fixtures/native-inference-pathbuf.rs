use std::cell::Cell;
use std::ffi::OsString;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf as OwnedPath;
use std::rc::Rc;

struct Guard(Rc<Cell<usize>>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn before() -> usize {
    let f = || 7usize;
    f()
}

fn paths(n: usize) -> Vec<OwnedPath> {
    let mut values = Vec::new();
    values.extend(["base".into(), format!("item-{n}").into()]);
    values
}

fn delayed() -> OwnedPath {
    let owned = String::from("delayed-owned");
    let value = owned.into();
    value
}

fn arguments() -> Vec<OsString> {
    let mut values = Vec::new();
    values.extend(["existing".into(), format!("old-rule").into()]);
    values
}

fn after(n: usize) -> usize {
    let add = move |x: usize| x + n;
    add(2)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let drops = Rc::new(Cell::new(0));
    {
        let _outer = Guard(drops.clone());
        let values = paths(before());
        assert_eq!(values[0], OwnedPath::from("base"));
        assert_eq!(values[1], OwnedPath::from("item-7"));
        assert_eq!(delayed(), OwnedPath::from("delayed-owned"));
        assert_eq!(arguments()[0], OsString::from("existing"));
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _inner = Guard(drops.clone());
                let during = paths(11);
                assert_eq!(during[1], OwnedPath::from("item-11"));
                panic!("path drop witness");
            }))
            .is_err(),
            "must unwind"
        );
        assert_eq!(drops.get(), 1);
        assert_eq!(after(before()), 9);
    }
    assert_eq!(drops.get(), 2);
    println!("mixed PathBuf/OsString inference and drop: {}", drops.get());
}
