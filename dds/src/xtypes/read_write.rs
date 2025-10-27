use crate::xtypes::error::{XTypesError, XTypesResult};

pub trait Read {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]>;

    fn read_array<const N: usize>(&mut self) -> XTypesResult<&[u8; N]> {
        self.read_exact(N)?
            .try_into()
            .map_err(|_e| XTypesError::InvalidData)
    }
}

/// A trait to Write bytes into a potentially growing buffer
pub trait Write {
    fn write(&mut self, buf: &[u8]);
}

impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend_from_slice(buf)
    }
}
