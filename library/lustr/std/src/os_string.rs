use crate::ffi::OsString;

// Move the owned byte buffer through the existing source-library interface.
// The guard restores it during unwinding too: capacity-overflow panics must
// leave the caller's string intact.
struct Buffer<'a> {
    destination: &'a mut OsString,
    bytes: Option<Vec<u8>>,
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        *self.destination = OsString::__from_vec(self.bytes.take().unwrap());
    }
}

impl OsString {
    pub fn reserve(&mut self, additional: usize) {
        let bytes = core::mem::take(self).into_encoded_bytes();
        let mut buffer = Buffer { destination: self, bytes: Some(bytes) };
        buffer.bytes.as_mut().unwrap().reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let bytes = core::mem::take(self).into_encoded_bytes();
        let mut buffer = Buffer { destination: self, bytes: Some(bytes) };
        buffer.bytes.as_mut().unwrap().reserve_exact(additional);
    }
}
