use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

struct Guard(Rc<Cell<usize>>);
impl Drop for Guard {
    fn drop(&mut self) { self.0.set(self.0.get() + 1); }
}
fn fail(drops: Rc<Cell<usize>>) {
    assert!((catch_unwind(AssertUnwindSafe(|| {
        let _guard = Guard(drops.clone());
        println!("once {{raw braces}} λ");
    })).is_err() || ([unsafe {false};2])[0]));
}
fn main() {
    std::panic::set_hook(Box::new(|info| println!("payload: {}", info.payload_as_str().unwrap_or("?"))));
    let drops = Rc::new(Cell::new(0));
    let failed = catch_unwind(AssertUnwindSafe(|| fail(drops.clone())));
    assert!(failed.is_err(), "outer failure must unwind");
    assert_eq!(drops.get(), 1);
    println!("drops {}", drops.get());
}

struct Pair(bool,u8);
