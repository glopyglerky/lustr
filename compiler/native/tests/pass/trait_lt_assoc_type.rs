// A lifetime-parameterized trait can declare an associated type and return a borrowed value.
trait Source<'a> {
    type Item;
    fn first(&self) -> Option<&'a u8>;
}
struct S;
impl<'a> Source<'a> for S {
    type Item = u8;
    fn first(&self) -> Option<&'a u8> {
        None
    }
}
fn main() {
    println!("{:?}", S.first());
}
