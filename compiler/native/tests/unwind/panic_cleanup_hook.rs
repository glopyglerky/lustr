// Cleanup's final nounwind panic runs the hook before aborting. A
// custom hook forwarding that panic to the default hook must retain its
// force-no-backtrace behavior, including with RUST_BACKTRACE=1.
use std::panic;

struct Bomb;
impl Drop for Bomb {
    fn drop(&mut self) {
        println!("drop");
        panic!("cleanup");
    }
}

fn main() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        println!("{} {}", info.payload_as_str().unwrap(), std::thread::panicking());
        if info.payload_as_str() == Some("panic in a destructor during cleanup") {
            previous(info);
        }
    }));
    let _ = panic::catch_unwind(|| {
        let _guard = Bomb;
        panic!("first");
    });
    println!("unexpected recovery");
}
