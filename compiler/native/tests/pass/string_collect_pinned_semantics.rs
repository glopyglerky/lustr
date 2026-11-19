// Rust source 5b7b37aef: FromIterator<char> allocates from its first
// size_hint, then Extend<char> asks again. References use String::new
// and Extend<char> only. These calls and capacities are observable.
struct Chars {
    next: u8,
    end: u8,
}

impl Iterator for Chars {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.next == self.end {
            None
        } else {
            let c = char::from(b'a' + self.next);
            self.next += 1;
            Some(c)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        println!("hint {} {}", self.next, self.end);
        let remaining = (self.end - self.next) as usize;
        (remaining, Some(remaining))
    }
}

fn main() {
    let owned: String = ['a', 'b'].into_iter().collect();
    assert_eq!(owned, "ab");
    assert_eq!(owned.capacity(), 2);
    let source = ['a', 'b'];
    let borrowed: String = source.iter().collect();
    assert_eq!(borrowed, "ab");
    assert_eq!(borrowed.capacity(), 8);
    println!("owned {} borrowed {}", owned.capacity(), borrowed.capacity());

    let collected: String = Chars { next: 0, end: 3 }.collect();
    assert_eq!(collected, "abc");
    assert_eq!(collected.capacity(), 3);
    println!("collected {} {}", collected, collected.capacity());
    let empty: String = Chars { next: 0, end: 0 }.collect();
    assert_eq!(empty.capacity(), 0);
    println!("empty {}", empty.capacity());

    let mut extended = String::new();
    extended.extend(Chars { next: 0, end: 3 });
    assert_eq!(extended, "abc");
    assert_eq!(extended.capacity(), 8);
    println!("extended {} {}", extended, extended.capacity());

    let utf8: String = ['a', '\u{00e9}', '\u{1f600}'].into_iter().collect();
    assert_eq!(utf8, "a\u{00e9}\u{1f600}");
    println!("utf8 {} {}", utf8.len(), utf8.capacity());
}
