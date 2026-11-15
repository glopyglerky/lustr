use crate::ffi::OsString;
use crate::path::PathBuf;

struct Buffer<'a> {
    destination: &'a mut PathBuf,
    string: Option<OsString>,
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        *self.destination = PathBuf::from_os_string(self.string.take().unwrap());
    }
}

impl PathBuf {
    pub fn reserve(&mut self, additional: usize) {
        let string = core::mem::take(self).into_os_string();
        let mut buffer = Buffer { destination: self, string: Some(string) };
        buffer.string.as_mut().unwrap().reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let string = core::mem::take(self).into_os_string();
        let mut buffer = Buffer { destination: self, string: Some(string) };
        buffer.string.as_mut().unwrap().reserve_exact(additional);
    }
}
