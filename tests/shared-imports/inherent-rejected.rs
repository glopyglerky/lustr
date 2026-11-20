extern crate peer;

impl peer::Word {
    fn external_inherent(self) -> u64 { self.0 }
}

fn main() { println!("{}", peer::Word(7).external_inherent()); }
