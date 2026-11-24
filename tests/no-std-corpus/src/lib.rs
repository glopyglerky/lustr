#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Length, Checksum, Incomplete }

/// Incremental length-prefixed framing with bounded payload and checksum.
pub struct Decoder<const N: usize> {
    bytes: [u8; N],
    used: usize,
    wanted: Option<usize>,
    checksum: u8,
}

impl<const N: usize> Decoder<N> {
    pub const fn new() -> Self {
        Self { bytes: [0; N], used: 0, wanted: None, checksum: 0 }
    }
    pub fn push(&mut self, byte: u8) -> Result<Option<&[u8]>, Error> {
        match self.wanted {
            None => {
                let count = usize::from(byte);
                if count > N { return Err(Error::Length); }
                self.wanted = Some(count);
                self.checksum = byte;
                Ok(None)
            }
            Some(count) if self.used < count => {
                self.bytes[self.used] = byte;
                self.used += 1;
                self.checksum ^= byte;
                Ok(None)
            }
            Some(count) => {
                self.wanted = None;
                self.used = 0;
                if byte != self.checksum { return Err(Error::Checksum); }
                Ok(Some(&self.bytes[..count]))
            }
        }
    }
    pub fn finish(&self) -> Result<(), Error> {
        if self.wanted.is_some() { Err(Error::Incomplete) } else { Ok(()) }
    }
}

pub fn exercise() -> bool {
    let mut decoder = Decoder::<8>::new();
    for byte in [3u8, 11, 22, 33] {
        if decoder.push(byte) != Ok(None) { return false; }
    }
    if decoder.finish() != Err(Error::Incomplete) { return false; }
    if decoder.push(3 ^ 11 ^ 22 ^ 33) != Ok(Some(&[11, 22, 33][..])) { return false; }
    if decoder.finish() != Ok(()) { return false; }
    if decoder.push(9) != Err(Error::Length) { return false; }
    if decoder.push(0) != Ok(None) { return false; }
    if decoder.push(1) != Err(Error::Checksum) { return false; }
    decoder.finish() == Ok(())
}

#[cfg(feature = "alloc")]
pub fn collect_payload(input: &[u8]) -> Result<alloc::vec::Vec<u8>, Error> {
    let mut decoder = Decoder::<255>::new();
    for &byte in input {
        if let Some(payload) = decoder.push(byte)? { return Ok(payload.into()); }
    }
    Err(Error::Incomplete)
}
