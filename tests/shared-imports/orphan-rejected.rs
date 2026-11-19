extern crate peer;

impl Clone for peer::Word {
    fn clone(&self) -> Self { peer::Word(self.0) }
}

fn main() { println!("{}", peer::Word(7).clone().0); }
