// expect: L0106
trait Project {
    type Output;
}

struct Marker;

impl Project for Marker {
    type Output<'a, 'a> = &'a u32;
}

fn main() {}
