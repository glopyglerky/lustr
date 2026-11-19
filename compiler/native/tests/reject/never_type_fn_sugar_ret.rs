// expect: L0603
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
// rustc E0658: `!` after `->` in `Fn() -> !` sugar is still the unstable
// never type; only fn, closure, and fn pointer return types are stable.
fn never() -> ! {
    panic!("x")
}
fn call(g: impl Fn() -> !) -> ! {
    g()
}
fn main() {
    if std::env::args().count() > 5 {
        call(never);
    }
}
