// A second destructor panic while completing a partial drop must abort.
// Catching the first panic must not turn the second one into a recoverable
// result or continue executing user code after catch_unwind.
use std::panic;

struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
        if self.0 == 1 || self.0 == 2 {
            panic!("destructor {}", self.0);
        }
    }
}

fn partial() {
    let a = [D(0), D(1), D(2), D(3)];
    let [moved, _, _, _] = a;
    std::mem::forget(moved);
}

fn main() {
    panic::set_hook(Box::new(|_| println!("panic")));
    let _ = panic::catch_unwind(partial);
    println!("unexpected recovery");
}
