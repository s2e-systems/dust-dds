use rtps_pim::structure::types::Locator;
use rtps_udp_psm::messages::overall_structure::RtpsMessage;

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessage, destination_locator: Locator);
}

pub trait TransportRead {
    fn read(&mut self) -> Option<(Locator, RtpsMessage)>;
}
