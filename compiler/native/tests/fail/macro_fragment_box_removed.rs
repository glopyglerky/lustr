// expect: L0806
// Rust source 5b7b37aef removed box patterns, including discarded
// pattern fragments. A token-tree fallback cannot hide that error.
macro_rules! classify {
    ($pattern:pat) => { "pattern" };
    ($($tokens:tt)*) => { "tokens" };
}
fn main() {
    println!("{}", classify!(box value));
}
