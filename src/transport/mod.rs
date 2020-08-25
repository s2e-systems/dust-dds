
use crate::types::Locator;
use crate::messages::RtpsMessage;
use crate::serialized_payload::CdrEndianness;

pub mod udp;
pub mod memory;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TransportEndianness {
    BigEndian = 0,
    LittleEndian = 1,
}

impl From<bool> for TransportEndianness {
    fn from(value: bool) -> Self {
        match value {
            true => TransportEndianness::LittleEndian,
            false => TransportEndianness::BigEndian,
        }
    }
}

impl From<TransportEndianness> for bool {
    fn from(value: TransportEndianness) -> Self {
        match value {
            TransportEndianness::LittleEndian => true,
            TransportEndianness::BigEndian => false,
        }
    }
}

impl From<TransportEndianness> for CdrEndianness {
    fn from(value: TransportEndianness) -> Self {
        match value {
            TransportEndianness::LittleEndian => CdrEndianness::LittleEndian,
            TransportEndianness::BigEndian => CdrEndianness::BigEndian,
        }
    }
}

#[derive(Debug)]
pub enum TransportError {
    IoError(std::io::Error),
    Other,
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        TransportError::IoError(error)
    }
}

pub type TransportResult<T> = std::result::Result<T, TransportError>;
pub trait Transport {
    fn write(&self, message: RtpsMessage, destination_locator_list: &[Locator]);

    fn read(&self) -> TransportResult<Option<(RtpsMessage, Locator)>>;

    fn unicast_locator_list(&self) -> Vec<Locator>;

    fn multicast_locator_list(&self) -> Vec<Locator>;
}