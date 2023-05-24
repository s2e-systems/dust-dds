use super::{messages::RtpsMessageWrite, types::Locator};

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessageWrite<'_>, destination_locator_list: &[Locator]);
}
