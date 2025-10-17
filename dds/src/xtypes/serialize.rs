use alloc::vec::Vec;

/// A trait to Write bytes into a potentially growing buffer
pub trait Write {
    fn write(&mut self, buf: &[u8]);
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend_from_slice(buf)
    }
}
