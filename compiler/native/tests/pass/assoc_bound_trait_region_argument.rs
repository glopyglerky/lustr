trait Project<'required> {
    type Item: 'required;
}

struct Marker<'first, 'second> {
    left: &'first u32,
    right: &'second u32,
}

impl<'left, 'right> Project<'right> for Marker<'left, 'right> {
    type Item = &'right u32;
}

struct Proven<'first, 'second> {
    left: &'first u32,
    right: &'second u32,
}

impl<'left, 'right> Project<'right> for Proven<'left, 'right> where 'left: 'right {
    type Item = &'left u32;
}

struct Owned;

impl Project<'static> for Owned {
    type Item = &'static u32;
}

fn main() {}
