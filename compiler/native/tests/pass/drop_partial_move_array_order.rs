// Every subset of four conditional moves must leave the remaining array
// elements in ascending index order. Nested field moves and reassignment
// exercise the same ownership states without changing their drop order.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn elements(mask: u8, refill: bool) {
    let mut a = [D(0), D(1), D(2), D(3)];
    if mask & 1 != 0 {
        let [x, _, _, _] = a;
        drop(x);
    }
    if mask & 2 != 0 {
        let [_, x, _, _] = a;
        drop(x);
    }
    if mask & 4 != 0 {
        let [_, _, x, _] = a;
        drop(x);
    }
    if mask & 8 != 0 {
        let [_, _, _, x] = a;
        drop(x);
    }
    println!("remaining");
    if refill {
        a = [D(10), D(11), D(12), D(13)];
        println!("refilled {}", a[0].0);
    }
}

fn fields(mask: u8) {
    let a = [(D(0), D(1)), (D(2), D(3)), (D(4), D(5)), (D(6), D(7))];
    if mask & 1 != 0 {
        let [(x, _), _, _, _] = a;
        drop(x);
    }
    if mask & 2 != 0 {
        let [_, (x, _), _, _] = a;
        drop(x);
    }
    if mask & 4 != 0 {
        let [_, _, (x, _), _] = a;
        drop(x);
    }
    if mask & 8 != 0 {
        let [_, _, _, (x, _)] = a;
        drop(x);
    }
    println!("remaining");
}

fn main() {
    for mask in 0..16 {
        println!("elements {mask}");
        elements(mask, false);
        println!("refill {mask}");
        elements(mask, true);
        println!("fields {mask}");
        fields(mask);
    }
}
