extern crate peer;

fn main() {
    let returned = peer::step(7u64);
    println!("{}", returned.0);
}
