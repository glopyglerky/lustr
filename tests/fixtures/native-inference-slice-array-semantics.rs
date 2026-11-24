use std::cell::Cell;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

struct Guard(Rc<Cell<usize>>);
impl Drop for Guard {
    fn drop(&mut self) { self.0.set(self.0.get() + 1); }
}
struct Value(u8);
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        println!("eq {} {}", self.0, other.0);
        self.0 % 2 == other.0 % 2
    }
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "V({})", self.0) }
}
impl Drop for Value {
    fn drop(&mut self) { println!("drop {}", self.0); }
}
fn before() -> usize { let f = || 3usize; f() }
fn slice<'a>(s: &'a [u8], order: &Cell<u32>, step: u32) -> &'a [u8] {
    order.set(order.get() * 10 + step);
    s
}
fn array<'a>(a: &'a [u8; 3], order: &Cell<u32>, step: u32) -> &'a [u8; 3] {
    order.set(order.get() * 10 + step);
    a
}
fn after(n: usize) -> usize { let f = move |x: usize| x + n; f(2) }
fn main() {
    std::panic::set_hook(Box::new(|info| println!("panic: {}", info.payload_as_str().unwrap_or("?"))));
    let a = [1u8, 2, 3];
    let s: &[u8] = &a;
    let order = Cell::new(0);
    assert_eq!(slice(s, &order, 1), array(&a, &order, 2));
    assert_eq!(array(&a, &order, 3), slice(s, &order, 4));
    assert_eq!(order.get(), 1234);
    assert_eq!(s, &[1u8, 2, 3], "unused message {}", { order.set(999); 0 });
    assert_eq!(order.get(), 1234);
    assert_eq!(vec![1u8, 2].as_slice(), &[1u8, 2]);
    assert_eq!(&[1u8, 2], vec![1u8, 2].as_slice());
    assert_eq!(&[] as &[u8], &[] as &[u8; 0]);
    assert_ne!(s, &[1u8, 2]);
    assert_ne!(&[1u8, 2], s);
    assert_eq!(&[true, false][..], &[true, false]);
    assert_eq!(&['x', 'y'], &['x', 'y'][..]);
    assert_eq!(&[1.0f64][..], &[1.0f64]);
    let closure = |source: &[u8]| { assert_eq!(source, &[1u8, 2, 3]); };
    closure(s);
    let drops = Rc::new(Cell::new(0));
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = Guard(drops.clone());
        assert_eq!(&[1u8, 2][..], &[1u8, 3]);
    })).is_err(), "equality must unwind");
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = Guard(drops.clone());
        assert_ne!(&[1u8, 2], &[1u8, 2][..]);
    })).is_err(), "inequality must unwind");
    assert_eq!(&[Value(1)][..], &[Value(3)]);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = Guard(drops.clone());
        assert_eq!(&[Value(1)], &[Value(2)][..], "comparison {}", 7u8);
    })).is_err(), "custom equality must unwind");
    assert_eq!(drops.get(), 3);
    assert_eq!(after(before()), 5);
    println!("slice array order {} drops {}", order.get(), drops.get());
}
