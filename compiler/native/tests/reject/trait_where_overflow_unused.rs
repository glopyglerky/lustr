// expect: L0611
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
// rustc E0275 at the impl: proving its own where clause `G<Option<T>>: Tr`
// evaluates the impl again at a larger type, past the recursion limit.
// Nothing uses the impl.
struct G<T>(T);
trait Tr {}
impl<T> Tr for G<T> where G<Option<T>>: Tr {}
fn main() {
    println!("ok");
}
