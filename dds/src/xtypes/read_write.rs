use crate::xtypes::error::{XTypesError, XTypesResult};

pub trait Read {
    fn read_exact(&mut self, size: usize) -> XTypesResult<&[u8]>;

    fn read_array<const N: usize>(&mut self) -> XTypesResult<&[u8; N]> {
        self.read_exact(N)?
            .try_into()
            .map_err(|_e| XTypesError::InvalidData)
    }
}
