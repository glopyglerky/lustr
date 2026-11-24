#![no_std]
#[inline(always)]
pub fn add(x:u64)->u64 { x.wrapping_add(1) }
