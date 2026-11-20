// expect: L0307
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
fn f(a: &i32) -> Vec<&i32> {
    let x = 5;
    let mut r = &x;
    println!("{}", r);
    r = a;
    let mut v: Vec<&i32> = Vec::new();
    v.push(r);
    v
}

fn main() {
    let a = 2;
    println!("{}", f(&a)[0]);
}
