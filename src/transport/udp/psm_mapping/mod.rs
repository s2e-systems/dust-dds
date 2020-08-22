mod rtps_message;

#[derive(Debug)]
pub enum UdpPsmMappingError {
    WrongSize,
    InvalidProtocolId,
    MessageTooSmall,
    InvalidEnumRepresentation,
    IoError(std::io::Error),
}

impl From<std::io::Error> for UdpPsmMappingError {
    fn from(error: std::io::Error) -> Self {
        UdpPsmMappingError::IoError(error)
    }
}

impl From<std::array::TryFromSliceError> for UdpPsmMappingError {
    fn from(_error: std::array::TryFromSliceError) -> Self {
        UdpPsmMappingError::WrongSize
    }
}

pub type UdpPsmMappingResult<T> = std::result::Result<T, UdpPsmMappingError>;

pub use  rtps_message::{serialize_rtps_message, deserialize_rtps_message};

pub struct SizeSerializer {
    size: usize,
}

impl SizeSerializer {
    pub fn new() -> Self {
        SizeSerializer {
            size: 0,
        }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

impl std::io::Write for SizeSerializer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>{
        let size = buf.len();
        self.size += size;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()>{
        Ok(())
    }
}