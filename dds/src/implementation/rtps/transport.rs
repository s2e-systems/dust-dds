use super::{messages::RtpsMessage, types::Locator};

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessage<'_>, destination_locator: Locator);
}
