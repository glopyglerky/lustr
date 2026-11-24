// expect: L0306
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
// rustc E0597: `&mut I` yields `I`'s items, so the set's elements are `&'a u8`.
use std::collections::HashSet;
fn g<'a, I: Iterator<Item = &'a u8>>(it: &mut I, x0: u8) -> usize {
    let mut v = HashSet::new();
    v.extend(it.skip(0));
    let y = x0;
    v.insert(&y);
    v.len()
}
fn main() {
    let s = vec![1u8, 2];
    println!("{}", g(&mut s.iter(), 9u8));
}
