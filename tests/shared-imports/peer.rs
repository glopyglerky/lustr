#![no_std]

// The value is deliberately non-Copy, despite containing one integer. The
// consumer must move it using the imported ownership fact, not infer Copy
// from its machine representation.
#[repr(transparent)]
pub struct Word(pub u64);

#[inline(never)]
pub fn step(value: Word) -> Word {
    Word(value.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407))
}
