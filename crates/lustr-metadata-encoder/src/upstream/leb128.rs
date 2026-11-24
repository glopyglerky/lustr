// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Source: rust-lang/rust@5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_serialize/src/leb128.rs
// Encoder-only extraction; decoder and its dependencies are excluded; module import adjusted.
use super::int_overflow::DebugStrictAdd;
/// Returns the length of the longest LEB128 encoding for `T`, assuming `T` is an integer type
pub const fn max_leb128_len<T>() -> usize {
    // The longest LEB128 encoding for an integer uses 7 bits per byte.
    (size_of::<T>() * 8).div_ceil(7)
}

/// Returns the length of the longest LEB128 encoding of all supported integer types.
pub const fn largest_max_leb128_len() -> usize {
    max_leb128_len::<u128>()
}

macro_rules! impl_write_unsigned_leb128 {
    ($fn_name:ident, $int_ty:ty) => {
        #[inline]
        pub fn $fn_name(out: &mut [u8; max_leb128_len::<$int_ty>()], mut value: $int_ty) -> usize {
            let mut i = 0;

            loop {
                if value < 0x80 {
                    unsafe {
                        *out.get_unchecked_mut(i) = value as u8;
                    }

                    i = i.debug_strict_add(1);
                    break;
                } else {
                    unsafe {
                        *out.get_unchecked_mut(i) = ((value & 0x7f) | 0x80) as u8;
                    }

                    value >>= 7;
                    i = i.debug_strict_add(1);
                }
            }

            i
        }
    };
}

impl_write_unsigned_leb128!(write_u16_leb128, u16);
impl_write_unsigned_leb128!(write_u32_leb128, u32);
impl_write_unsigned_leb128!(write_u64_leb128, u64);
impl_write_unsigned_leb128!(write_u128_leb128, u128);
impl_write_unsigned_leb128!(write_usize_leb128, usize);

macro_rules! impl_write_signed_leb128 {
    ($fn_name:ident, $int_ty:ty) => {
        #[inline]
        pub fn $fn_name(out: &mut [u8; max_leb128_len::<$int_ty>()], mut value: $int_ty) -> usize {
            let mut i = 0;

            loop {
                let mut byte = (value as u8) & 0x7f;
                value >>= 7;
                let more = !(((value == 0) && ((byte & 0x40) == 0))
                    || ((value == -1) && ((byte & 0x40) != 0)));

                if more {
                    byte |= 0x80; // Mark this byte to show that more bytes will follow.
                }

                unsafe {
                    *out.get_unchecked_mut(i) = byte;
                }

                i = i.debug_strict_add(1);

                if !more {
                    break;
                }
            }

            i
        }
    };
}

impl_write_signed_leb128!(write_i16_leb128, i16);
impl_write_signed_leb128!(write_i32_leb128, i32);
impl_write_signed_leb128!(write_i64_leb128, i64);
impl_write_signed_leb128!(write_i128_leb128, i128);
impl_write_signed_leb128!(write_isize_leb128, isize);
