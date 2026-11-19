// expect: L0104
// This public API requires Rust's panic_can_unwind feature. The native
// subset does not support feature attributes and must reject its use.
fn main() {
    std::panic::set_hook(Box::new(|info| {
        let _ = info.can_unwind();
    }));
}
