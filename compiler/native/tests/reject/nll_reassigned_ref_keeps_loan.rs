// expect: L0312
// Accepted by declared Rust source 5b7b37aef; native capability rejection.
// Historical rejection comments below refer to the Rust 1.99 oracle.
// rustc E0506: region values are location-insensitive sets. `r` held `&x`
// (through `t`), so `&x` lives as long as `r` does, even after `r = &y`.
// Polonius would accept this program; rustc's NLL does not (ARCH 5.2).
fn main() {
    let mut x = 1;
    let y = 5;
    let mut r = &x;
    let t = r;
    r = &y;
    println!("{}", t);
    x = 2;
    println!("{} {}", r, x);
}
