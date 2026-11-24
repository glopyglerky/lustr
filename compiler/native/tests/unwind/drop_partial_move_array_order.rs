// The same sixteen partial-ownership states must keep array index order
// during panic cleanup. No destructor panics, so every remaining element
// must be dropped exactly once before catch_unwind returns.
use std::panic;

struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn partial(mask: u8) {
    let a = [D(0), D(1), D(2), D(3)];
    if mask & 1 != 0 {
        let [x, _, _, _] = a;
        drop(x);
    }
    if mask & 2 != 0 {
        let [_, x, _, _] = a;
        drop(x);
    }
    if mask & 4 != 0 {
        let [_, _, x, _] = a;
        drop(x);
    }
    if mask & 8 != 0 {
        let [_, _, _, x] = a;
        drop(x);
    }
    println!("unwinding");
    panic!("partial array");
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    for mask in 0..16 {
        println!("mask {mask}");
        let result = panic::catch_unwind(|| partial(mask));
        assert!(result.is_err());
        println!("caught");
    }
}
