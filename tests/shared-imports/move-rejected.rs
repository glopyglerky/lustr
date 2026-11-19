extern crate peer;

fn main() {
    let value = peer::Word(7);
    let returned = peer::step(value);
    println!("{} {}", returned.0, value.0);
}
