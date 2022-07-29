#![no_std]

use messages::RtpsMessage;
use types::Locator;
extern crate alloc;

pub mod messages;
pub mod types;

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessage<'_>, destination_locator: Locator);
}

pub trait TransportRead<'a> {
    fn read(&'a mut self) -> Option<(Locator, RtpsMessage<'a>)>;
}
