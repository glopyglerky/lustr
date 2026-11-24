#![no_std]

// Each public shape is imported from actual metadata before native checking.

#[repr(transparent)]
pub struct U8(pub u8);

#[inline(never)]
pub fn combine_u8(value: U8, bias: u8, other: U8, a: u8, b: u8, c: u8, d: u8, e: u8) -> U8 {
    U8(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_u8(value: u8) -> U8 { U8(value) }

#[inline(never)]
pub fn extract_u8(value: U8) -> u8 { value.0 }

#[inline(never)]
pub fn seed_u8() -> u8 { 17 }

#[repr(transparent)]
pub struct I8(pub i8);

#[inline(never)]
pub fn combine_i8(value: I8, bias: i8, other: I8, a: i8, b: i8, c: i8, d: i8, e: i8) -> I8 {
    I8(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_i8(value: i8) -> I8 { I8(value) }

#[inline(never)]
pub fn extract_i8(value: I8) -> i8 { value.0 }

#[inline(never)]
pub fn seed_i8() -> i8 { 17 }

#[repr(transparent)]
pub struct U16(pub u16);

#[inline(never)]
pub fn combine_u16(value: U16, bias: u16, other: U16, a: u16, b: u16, c: u16, d: u16, e: u16) -> U16 {
    U16(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_u16(value: u16) -> U16 { U16(value) }

#[inline(never)]
pub fn extract_u16(value: U16) -> u16 { value.0 }

#[inline(never)]
pub fn seed_u16() -> u16 { 17 }

#[repr(transparent)]
pub struct I16(pub i16);

#[inline(never)]
pub fn combine_i16(value: I16, bias: i16, other: I16, a: i16, b: i16, c: i16, d: i16, e: i16) -> I16 {
    I16(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_i16(value: i16) -> I16 { I16(value) }

#[inline(never)]
pub fn extract_i16(value: I16) -> i16 { value.0 }

#[inline(never)]
pub fn seed_i16() -> i16 { 17 }

#[repr(transparent)]
pub struct U32(pub u32);

#[inline(never)]
pub fn combine_u32(value: U32, bias: u32, other: U32, a: u32, b: u32, c: u32, d: u32, e: u32) -> U32 {
    U32(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_u32(value: u32) -> U32 { U32(value) }

#[inline(never)]
pub fn extract_u32(value: U32) -> u32 { value.0 }

#[inline(never)]
pub fn seed_u32() -> u32 { 17 }

#[repr(transparent)]
pub struct I32(pub i32);

#[inline(never)]
pub fn combine_i32(value: I32, bias: i32, other: I32, a: i32, b: i32, c: i32, d: i32, e: i32) -> I32 {
    I32(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_i32(value: i32) -> I32 { I32(value) }

#[inline(never)]
pub fn extract_i32(value: I32) -> i32 { value.0 }

#[inline(never)]
pub fn seed_i32() -> i32 { 17 }

#[repr(transparent)]
pub struct U64(pub u64);

#[inline(never)]
pub fn combine_u64(value: U64, bias: u64, other: U64, a: u64, b: u64, c: u64, d: u64, e: u64) -> U64 {
    U64(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_u64(value: u64) -> U64 { U64(value) }

#[inline(never)]
pub fn extract_u64(value: U64) -> u64 { value.0 }

#[inline(never)]
pub fn seed_u64() -> u64 { 17 }

#[repr(transparent)]
pub struct I64(pub i64);

#[inline(never)]
pub fn combine_i64(value: I64, bias: i64, other: I64, a: i64, b: i64, c: i64, d: i64, e: i64) -> I64 {
    I64(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_i64(value: i64) -> I64 { I64(value) }

#[inline(never)]
pub fn extract_i64(value: I64) -> i64 { value.0 }

#[inline(never)]
pub fn seed_i64() -> i64 { 17 }

#[repr(transparent)]
pub struct USIZE(pub usize);

#[inline(never)]
pub fn combine_usize(value: USIZE, bias: usize, other: USIZE, a: usize, b: usize, c: usize, d: usize, e: usize) -> USIZE {
    USIZE(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_usize(value: usize) -> USIZE { USIZE(value) }

#[inline(never)]
pub fn extract_usize(value: USIZE) -> usize { value.0 }

#[inline(never)]
pub fn seed_usize() -> usize { 17 }

#[repr(transparent)]
pub struct ISIZE(pub isize);

#[inline(never)]
pub fn combine_isize(value: ISIZE, bias: isize, other: ISIZE, a: isize, b: isize, c: isize, d: isize, e: isize) -> ISIZE {
    ISIZE(value.0.wrapping_add(bias.wrapping_mul(3)).wrapping_sub(other.0.wrapping_mul(5))
        .wrapping_add(a).wrapping_sub(b).wrapping_add(c).wrapping_sub(d).wrapping_add(e))
}

#[inline(never)]
pub fn wrap_isize(value: isize) -> ISIZE { ISIZE(value) }

#[inline(never)]
pub fn extract_isize(value: ISIZE) -> isize { value.0 }

#[inline(never)]
pub fn seed_isize() -> isize { 17 }
