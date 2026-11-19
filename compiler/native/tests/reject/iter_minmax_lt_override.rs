// expect: L0640
// Iterator min/max at the declared Rust pin call PartialOrd::lt.
// Native comparison glue cannot call a written lt override yet, so
// preserve an explicit capability rejection instead of ignoring it.
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
struct Key(i32);

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
    fn lt(&self, other: &Self) -> bool {
        println!("lt {} {}", self.0, other.0);
        self.0 < other.0
    }
}

fn main() {
    println!("{:?}", [Key(3), Key(1), Key(2)].into_iter().min());
}
