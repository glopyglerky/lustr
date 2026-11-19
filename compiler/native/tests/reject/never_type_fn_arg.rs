// expect: L0603
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
// rustc E0658: the `!` type is experimental outside a fn return type (hunt5).
fn f(x: !) -> i32 { x }
fn main() { println!("hi"); }
