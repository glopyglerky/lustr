// expect: L0303
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
fn first_or_push(v: &mut Vec<i32>) -> &i32 {
    if let Some(x) = v.first() {
        return x;
    }
    v.push(1);
    &v[0]
}
fn main() {
    let mut v = Vec::new();
    println!("{}", first_or_push(&mut v));
}
