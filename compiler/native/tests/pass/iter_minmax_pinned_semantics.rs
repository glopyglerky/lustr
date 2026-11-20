// The pinned Rust defaults for iterator min/max call PartialOrd::lt
// as `candidate < best`, including keyed reductions. Calls are visible
// even when PartialOrd agrees with Ord.
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
struct Key(i32);

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        println!("compare {} {}", self.0, other.0);
        Some(self.0.cmp(&other.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Generic<T>(T);

fn main() {
    let values = [Key(3), Key(1), Key(2)];
    println!("max {:?}", values.iter().max());
    println!("min {:?}", values.iter().min());
    println!("max key {:?}", [3, 1, 2].into_iter().max_by_key(|v| Key(*v)));
    println!("min key {:?}", [3, 1, 2].into_iter().min_by_key(|v| Key(*v)));
    println!("tuple {:?}", [(Key(2), 0), (Key(1), 1)].into_iter().min());
    println!("generic {:?}", [Generic(Key(2)), Generic(Key(1))].into_iter().max());
    println!("empty {:?}", Vec::<Key>::new().into_iter().min());
    let tie = [(7, "first"), (7, "second")];
    println!("ties {:?} {:?}", tie.into_iter().min_by_key(|v| v.0), tie.into_iter().max_by_key(|v| v.0));
}
