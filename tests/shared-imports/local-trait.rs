extern crate peer;

trait LocalView { fn value(self) -> u64; }
impl LocalView for peer::Word {
    fn value(self) -> u64 { self.0 }
}
fn require<T: Send + Sync + Unpin>() {}

fn main() {
    require::<peer::Word>();
    let value = peer::step(peer::Word(7));
    println!("{}", value.value());
}
