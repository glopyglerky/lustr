#![no_std]
pub fn combine(left: u64, right: u64) -> u64 { left | right }
fn helper(value: u64) -> u64 { true }
pub fn transform(value: u64) -> u64 { helper(value) ^ 85u64 }
