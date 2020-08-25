mod rtps_message;
mod submessage_elements;
mod parameter_list;
mod rtps_submessage;
mod info_timestamp_submessage;
mod data_submessage;

pub use rtps_message::{serialize_rtps_message, deserialize_rtps_message};

#[derive(Debug)]
pub enum UdpPsmMappingError {
    WrongSize,
    InvalidProtocolId,
    MessageTooSmall,
    InvalidEnumRepresentation,
    UnknownSubmessageId,
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

pub trait SizeCheck {
    fn check_size_equal(&self, expected_size: usize) -> UdpPsmMappingResult<()>;
    fn check_size_bigger_equal_than(&self, expected_size: usize) -> UdpPsmMappingResult<()>;
}

impl SizeCheck for &[u8] {
    #[inline]
    fn check_size_equal(&self, expected_size: usize) -> UdpPsmMappingResult<()> {
        if self.len() != expected_size {
            Err(UdpPsmMappingError::WrongSize)
        } else {
            Ok(())
        }
    }
    
    #[inline]
    fn check_size_bigger_equal_than(&self, expected_size: usize) -> UdpPsmMappingResult<()> {
        if self.len() >= expected_size {
            Ok(())
        } else {
            Err(UdpPsmMappingError::MessageTooSmall)
        }
    }
}