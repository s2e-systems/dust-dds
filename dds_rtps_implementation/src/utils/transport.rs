use rust_rtps_pim::structure::types::Locator;
use rust_rtps_psm::messages::overall_structure::{RtpsMessageRead, RtpsMessageWrite};

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessageWrite, destination_locator: &Locator);
}

pub trait TransportRead {
    fn read(&mut self) -> Option<(Locator, RtpsMessageRead)>;
}
