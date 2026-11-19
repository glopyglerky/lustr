// After a partial move, a destructor panic must not skip the remaining
// elements or fields. Cover every move subset and every possible panic
// position, including a panic position that was moved out or is absent.
use std::panic;

struct D(u8, u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
        if self.0 == self.1 {
            panic!("destructor");
        }
    }
}

fn elements(mask: u8, panic_at: u8) {
    let a = [D(0, panic_at), D(1, panic_at), D(2, panic_at), D(3, panic_at)];
    if mask & 1 != 0 {
        let [x, _, _, _] = a;
        std::mem::forget(x);
    }
    if mask & 2 != 0 {
        let [_, x, _, _] = a;
        std::mem::forget(x);
    }
    if mask & 4 != 0 {
        let [_, _, x, _] = a;
        std::mem::forget(x);
    }
    if mask & 8 != 0 {
        let [_, _, _, x] = a;
        std::mem::forget(x);
    }
}

fn fields(mask: u8, panic_at: u8) {
    let a = [
        (D(0, panic_at), D(1, panic_at)),
        (D(2, panic_at), D(3, panic_at)),
        (D(4, panic_at), D(5, panic_at)),
        (D(6, panic_at), D(7, panic_at)),
    ];
    if mask & 1 != 0 {
        let [(x, _), _, _, _] = a;
        std::mem::forget(x);
    }
    if mask & 2 != 0 {
        let [_, (x, _), _, _] = a;
        std::mem::forget(x);
    }
    if mask & 4 != 0 {
        let [_, _, (x, _), _] = a;
        std::mem::forget(x);
    }
    if mask & 8 != 0 {
        let [_, _, _, (x, _)] = a;
        std::mem::forget(x);
    }
}

fn ranges(mask: u8, panic_at: u8) {
    let a = [D(0, panic_at), D(1, panic_at), D(2, panic_at), D(3, panic_at), D(4, panic_at), D(5, panic_at), D(6, panic_at), D(7, panic_at)];
    if mask & 1 != 0 {
        let [x, _, _, _, _, _, _, _] = a;
        std::mem::forget(x);
    }
    if mask & 2 != 0 {
        let [_, _, _, x, _, _, _, _] = a;
        std::mem::forget(x);
    }
}

enum E {
    A(D, D),
    B(D, D),
}

struct P {
    first: E,
    last: D,
}

fn variants(other: bool, panic_at: u8) {
    let p = P {
        first: if other { E::B(D(0, panic_at), D(1, panic_at)) } else { E::A(D(0, panic_at), D(1, panic_at)) },
        last: D(2, panic_at),
    };
    if let E::A(x, _) = p.first {
        std::mem::forget(x);
    }
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    for mask in 0..16 {
        for panic_at in 0..5 {
            println!("elements {mask} {panic_at}");
            let result = panic::catch_unwind(|| elements(mask, panic_at));
            println!("caught {}", result.is_err());
        }
        for panic_at in 0..9 {
            println!("fields {mask} {panic_at}");
            let result = panic::catch_unwind(|| fields(mask, panic_at));
            println!("caught {}", result.is_err());
        }
    }
    for mask in 0..4 {
        for panic_at in 0..9 {
            println!("ranges {mask} {panic_at}");
            let result = panic::catch_unwind(|| ranges(mask, panic_at));
            println!("caught {}", result.is_err());
        }
        for other in [false, true] {
            println!("variants {other} {mask}");
            let result = panic::catch_unwind(|| variants(other, mask));
            println!("caught {}", result.is_err());
        }
    }
}
