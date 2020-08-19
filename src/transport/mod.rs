
use crate::types::Locator;
use crate::messages::RtpsMessage;

pub mod udp_transport;
pub mod memory_transport;

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
    fn new(unicast_locator: Locator, multicast_locator: Option<Locator>) -> TransportResult<Self> where Self: std::marker::Sized;

    fn write(&self, message: RtpsMessage, unicast_locator_list: &[Locator], multicast_locator_list: &[Locator]);

    fn read(&self) -> TransportResult<Option<(RtpsMessage, Locator)>>;

    fn unicast_locator_list(&self) -> Vec<Locator>;

    fn multicast_locator_list(&self) -> Vec<Locator>;
}