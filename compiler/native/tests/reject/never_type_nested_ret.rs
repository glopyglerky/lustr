// expect: L0603
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
// rustc E0658: `!` inside a return type (`Option<!>`) is not the whole
// return type, so it is the unstable never type.
fn f() -> Option<!> {
    None
}
fn main() {
    println!("{}", f().is_none());
}
