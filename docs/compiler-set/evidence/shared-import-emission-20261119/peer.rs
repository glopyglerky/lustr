#![no_std]
#[repr(transparent)]pub struct Word(pub i8);
pub fn add(value:Word,offset:i8)->Word{Word(value.0.wrapping_add(offset))}
