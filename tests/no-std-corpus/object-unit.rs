#![no_std]

fn round(value: u64, key: u64) -> u64 {
    value.rotate_left(7) ^ key.wrapping_mul(0x100000001b3)
}

pub fn compress(left: u64, right: u64) -> u64 {
    round(round(left, right), left)
}
