// expect: L0806
// A removed box pattern nested in a tuple-variant fragment is invalid
// even when the macro discards the fragment before type checking.
macro_rules! discard {
    ($pattern:pat) => { () };
}
fn main() {
    discard!(Some(box value));
}
