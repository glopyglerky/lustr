use alloc::vec::Vec;
// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Source: rust-lang/rust@5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_serialize/src/opaque/mem_encoder.rs
// Module imports adjusted; generic Encodable specialization and unused fixed-size wrapper excluded.
use super::{codec::Encoder, leb128};
pub struct MemEncoder {
    pub data: Vec<u8>,
}

impl MemEncoder {
    pub fn new() -> MemEncoder {
        MemEncoder { data: vec![] }
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.data.len()
    }

    pub fn finish(self) -> Vec<u8> {
        self.data
    }

    /// Write up to `N` bytes to this encoder.
    ///
    /// This function can be used to avoid the overhead of calling memcpy for writes that
    /// have runtime-variable length, but are small and have a small fixed upper bound.
    ///
    /// This can be used to do in-place encoding as is done for leb128 (without this function
    /// we would need to write to a temporary buffer then memcpy into the encoder), and it can
    /// also be used to implement the varint scheme we use for rmeta and dep graph encoding,
    /// where we only want to encode the first few bytes of an integer. Note that common
    /// architectures support fixed-size writes up to 8 bytes with one instruction, so while this
    /// does in some sense do wasted work, we come out ahead.
    #[inline]
    pub fn write_with<const N: usize>(&mut self, visitor: impl FnOnce(&mut [u8; N]) -> usize) {
        self.data.reserve(N);

        let old_len = self.data.len();

        // SAFETY: The above `reserve` ensures that there is enough
        // room to write the encoded value to the vector's internal buffer.
        // The memory is also initialized as 0.
        let buf = unsafe {
            let buf = self.data.as_mut_ptr().add(old_len) as *mut [u8; N];
            *buf = [0; N];
            &mut *buf
        };
        let written = visitor(buf);
        if written > N {
            Self::panic_invalid_write::<N>(written);
        }
        unsafe { self.data.set_len(old_len + written) };
    }

    #[cold]
    #[inline(never)]
    fn panic_invalid_write<const N: usize>(written: usize) {
        panic!("MemEncoder::write_with::<{N}> cannot be used to write {written} bytes");
    }

    /// Helper for calls where [`MemEncoder::write_with`] always writes the whole array.
    #[inline]
    pub fn write_array<const N: usize>(&mut self, buf: [u8; N]) {
        self.write_with(|dest| {
            *dest = buf;
            N
        })
    }
}

macro_rules! write_leb128 {
    ($this_fn:ident, $int_ty:ty, $write_leb_fn:ident) => {
        #[inline]
        fn $this_fn(&mut self, v: $int_ty) {
            self.write_with(|buf| leb128::$write_leb_fn(buf, v))
        }
    };
}

impl Encoder for MemEncoder {
    write_leb128!(emit_usize, usize, write_usize_leb128);
    write_leb128!(emit_u128, u128, write_u128_leb128);
    write_leb128!(emit_u64, u64, write_u64_leb128);
    write_leb128!(emit_u32, u32, write_u32_leb128);

    #[inline]
    fn emit_u16(&mut self, v: u16) {
        self.write_array(v.to_le_bytes());
    }

    #[inline]
    fn emit_u8(&mut self, v: u8) {
        self.write_array([v]);
    }

    write_leb128!(emit_isize, isize, write_isize_leb128);
    write_leb128!(emit_i128, i128, write_i128_leb128);
    write_leb128!(emit_i64, i64, write_i64_leb128);
    write_leb128!(emit_i32, i32, write_i32_leb128);

    #[inline]
    fn emit_i16(&mut self, v: i16) {
        self.write_array(v.to_le_bytes());
    }

    #[inline]
    fn emit_raw_bytes(&mut self, s: &[u8]) {
        self.data.extend_from_slice(s);
    }
}
