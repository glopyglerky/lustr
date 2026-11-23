// expect: L0110
trait Project {
    type Item;
}

struct Marker;

impl Project for Marker {
    type Item<'extra> = &'extra u32;
}

fn main() {}
