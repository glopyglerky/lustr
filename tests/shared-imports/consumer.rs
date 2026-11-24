extern crate peer;

// This original body is checked, borrow checked and emitted by Lust. Its
// nominal argument and result have the actual imported Rust identity.
fn round_trip(value: peer::Word) -> peer::Word {
    let returned = peer::step(value);
    peer::Word(returned.0 ^ 0xa5a5a5a5a5a5a5a5)
}

fn verify(input: u64) {
    let original = peer::Word(input);
    let result = round_trip(original);
    let expected = input.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407) ^ 0xa5a5a5a5a5a5a5a5;
    assert_eq!(result.0, expected);
}

fn main() {
    for value in 0..65536u64 {
        verify(value);
    }
    for value in [u64::MAX, u64::MAX - 1, 1u64 << 63, (1u64 << 63) - 1, 1u64 << 32] {
        verify(value);
    }
    println!("shared nominal calls: 65541");
}
