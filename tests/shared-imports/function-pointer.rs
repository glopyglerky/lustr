extern crate peer;

fn invoke(function: fn(peer::Word) -> peer::Word, value: peer::Word) -> peer::Word {
    function(value)
}

fn verify(input: u64) {
    let constructor: fn(u64) -> peer::Word = peer::Word;
    let function: fn(peer::Word) -> peer::Word = peer::step;
    let result = invoke(function, constructor(input));
    let expected = input.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    assert_eq!(result.0, expected);
}

fn main() {
    for value in 0..65536u64 { verify(value); }
    for value in [u64::MAX, u64::MAX - 1, 1u64 << 63, (1u64 << 63) - 1, 1u64 << 32] {
        verify(value);
    }
    println!("shared function pointers: 65541");
}
