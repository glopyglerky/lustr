// expect: L0617
trait Project<'required> {
    type Item: 'required;
}

struct Marker<'first, 'second> {
    left: &'first u32,
    right: &'second u32,
}

impl<'left, 'right> Project<'right> for Marker<'left, 'right> {
    type Item = &'left u32;
}

fn main() {}
