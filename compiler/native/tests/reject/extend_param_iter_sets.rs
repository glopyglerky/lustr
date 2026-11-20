// expect: L0306
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
// rustc E0597: an adapter over a type param iterator (`take`, `chain`) passes its fixed
// items on, into a library collection (`BTreeSet`) as into a builtin one.
use std::collections::BTreeSet;
fn g<'a, I: Iterator<Item = &'a u8>>(it: I, x0: u8) -> usize {
    let mut v = BTreeSet::new();
    v.extend(it.take(5).chain(None));
    v.insert(&x0);
    v.len()
}
fn main() {
    let s = vec![1u8, 2];
    println!("{}", g(s.iter(), 9u8));
}
