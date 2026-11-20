// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Source: rust-lang/rust@5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/compiler/rustc_serialize/src/serialize.rs
// Only scalar/string encoding interface; hash-map ordering and all decoder/collection adapters are excluded.
const STR_SENTINEL: u8 = 0xC1;
const BYTE_STR_SENTINEL: u8 = 0xC2;
pub trait Encoder {
    fn emit_usize(&mut self, v: usize);
    fn emit_u128(&mut self, v: u128);
    fn emit_u64(&mut self, v: u64);
    fn emit_u32(&mut self, v: u32);
    fn emit_u16(&mut self, v: u16);
    fn emit_u8(&mut self, v: u8);

    fn emit_isize(&mut self, v: isize);
    fn emit_i128(&mut self, v: i128);
    fn emit_i64(&mut self, v: i64);
    fn emit_i32(&mut self, v: i32);
    fn emit_i16(&mut self, v: i16);

    #[inline]
    fn emit_i8(&mut self, v: i8) {
        self.emit_u8(v as u8);
    }

    #[inline]
    fn emit_bool(&mut self, v: bool) {
        self.emit_u8(if v { 1 } else { 0 });
    }

    #[inline]
    fn emit_char(&mut self, v: char) {
        self.emit_u32(v as u32);
    }

    #[inline]
    fn emit_str(&mut self, v: &str) {
        self.emit_usize(v.len());
        self.emit_raw_bytes(v.as_bytes());
        self.emit_u8(STR_SENTINEL);
    }

    #[inline]
    fn emit_byte_str(&mut self, v: &[u8]) {
        self.emit_usize(v.len());
        self.emit_raw_bytes(v);
        self.emit_u8(BYTE_STR_SENTINEL);
    }

    fn emit_raw_bytes(&mut self, s: &[u8]);

}
