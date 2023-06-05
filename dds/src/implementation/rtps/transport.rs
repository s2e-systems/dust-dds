use super::{messages::overall_structure::RtpsMessageWrite, types::Locator};

pub trait TransportWrite {
    fn write(&self, message: &RtpsMessageWrite, destination_locator_list: &[Locator]);
}
