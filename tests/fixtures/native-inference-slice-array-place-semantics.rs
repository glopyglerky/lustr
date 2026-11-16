use std::cell::Cell;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

const EXPECTED: &[u8; 1] = &[1u8];
fn empty_arena() { assert_eq!(&[1u8][..], EXPECTED); }
fn destructured((a, s): (&[u8; 3], &[u8])) { assert_eq!(a, s); }
fn nested(a: &[u8; 3], s: &[u8]) {
    fn inner(a: &[u8; 3], s: &[u8]) { assert_eq!(s, a); }
    assert_eq!(a, s);
    inner(a, s);
    assert_eq!(s, a);
}
struct Holder<'a> { array: &'a [u8; 3] }
impl Holder<'_> {
    fn check(&self, s: &[u8]) { assert_eq!(self.array, s); }
}
struct Guard(Rc<Cell<usize>>);
impl Drop for Guard { fn drop(&mut self) { self.0.set(self.0.get() + 1); } }
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
impl Drop for Value { fn drop(&mut self) { println!("drop {}", self.0); } }
struct Message(Rc<Cell<usize>>, u8);
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "message({})", self.1) }
}
impl Drop for Message {
    fn drop(&mut self) { self.0.set(self.0.get() + 1); println!("message drop {}", self.1); }
}
fn main() {
    std::panic::set_hook(Box::new(|info| println!("panic: {}", info.payload_as_str().unwrap_or("?"))));
    empty_arena();
    let a = [1u8, 2, 3];
    let s: &[u8] = &a;
    let expected = &a;
    let tuple = (expected,);
    let holder = Holder { array: expected };
    let indexed = [expected, expected];
    let indirect = &expected;
    assert_eq!(s, expected);
    assert_eq!(expected, s);
    assert_eq!(holder.array, s);
    assert_eq!(s, tuple.0);
    assert_eq!(indexed[1], s);
    assert_eq!(s, *indirect);
    let other = [4u8, 5, 6];
    let mut changed = expected;
    assert_eq!(changed, s);
    changed = &other;
    assert_ne!(s, changed);
    holder.check(s);
    destructured((expected, s));
    nested(expected, s);
    let closure = |(a, s): (&[u8; 3], &[u8])| { assert_eq!(a, s); };
    closure((expected, s));
    for (i, n) in a.iter().enumerate() { assert_eq!(*n as usize, i + 1); }
    let Some(n) = Some(3usize) else { panic!("missing value"); };
    assert_eq!(n, 3);
    match (1u8, 2u8) { (x, y) => assert_eq!(x + 1, y) }
    let zero: &[u8; 0] = &[];
    assert_eq!(&[] as &[u8], zero);
    let short = &[1u8, 2];
    assert_ne!(short, s);
    assert_ne!(s, short);
    let bools = &[true, false];
    assert_eq!(&[true, false][..], bools);
    let chars = &['x', 'y'];
    assert_eq!(chars, &['x', 'y'][..]);
    let floats = &[1.0f64];
    assert_eq!(&[1.0f64][..], floats);
    let messages = Cell::new(0);
    assert_eq!(s, expected, "unused {}", { messages.set(100); 0 });
    assert_eq!(messages.get(), 0);
    let drops = Rc::new(Cell::new(0));
    let same = &[1u8, 2, 3];
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = Guard(drops.clone());
        assert_ne!(s, same, "failure {}", { messages.set(messages.get() + 1); 7u8 });
    })).is_err(), "place inequality must unwind");
    let different = &[1u8, 2, 4];
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = Guard(drops.clone());
        assert_eq!(different, s, "failure {}", { messages.set(messages.get() + 1); 8u8 });
    })).is_err(), "place equality must unwind");
    {
        let values = &[Value(3)];
        assert_eq!(&[Value(1)][..], values);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            let _guard = Guard(drops.clone());
            assert_eq!(&[Value(2)][..], values, "custom {}", { messages.set(messages.get() + 1); 9u8 });
        })).is_err(), "custom place comparison must unwind");
    }
    assert_eq!(messages.get(), 3);
    assert_eq!(drops.get(), 3);
    let message = String::from("noncopy");
    assert!(catch_unwind(AssertUnwindSafe(|| {
        assert_eq!(s, different, "{0} {0} {value:>width$}", message, value=7u8, width=3usize);
    })).is_err(), "noncopy message must remain borrowed");
    assert_eq!(message, "noncopy");
    let message_drops = Rc::new(Cell::new(0));
    assert!(catch_unwind(AssertUnwindSafe(|| {
        assert_eq!(different, s, "temporary {}", Message(message_drops.clone(), 9));
    })).is_err(), "temporary message must survive formatting");
    assert_eq!(message_drops.get(), 1);
    let order = Cell::new(0);
    assert_eq!({ order.set(order.get() * 10 + 1); s }, indexed[{ order.set(order.get() * 10 + 2); 0 }]);
    assert_eq!(indexed[{ order.set(order.get() * 10 + 3); 1 }], { order.set(order.get() * 10 + 4); s });
    assert_eq!(order.get(), 1234);
    println!("place assertions order {} messages {} drops {}", order.get(), messages.get(), drops.get());
}
