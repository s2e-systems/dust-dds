use super::{messages::RtpsMessage, types::Locator};

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessage<'_>, destination_locator: Locator);
}

pub trait TransportRead<'a> {
    fn read(&'a mut self) -> Option<(Locator, RtpsMessage<'a>)>;
}
