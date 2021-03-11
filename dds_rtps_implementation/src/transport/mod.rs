pub mod memory;

use rust_rtps::{messages::RtpsMessage, types::Locator};

#[derive(Debug)]
pub enum TransportError {
    IoError(std::io::Error),
    InterfaceNotFound(String),
    Other(String),
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        TransportError::IoError(error)
    }
}

pub type TransportResult<T> = std::result::Result<T, TransportError>;
pub trait Transport<'a>: Send + Sync {
    fn write(&self, message: RtpsMessage<'a>, destination_locator: &Locator);

    fn read(&self) -> TransportResult<Option<(RtpsMessage<'a>, Locator)>>;

    fn unicast_locator_list(&self) -> &Vec<Locator>;

    fn multicast_locator_list(&self) -> &Vec<Locator>;
}
