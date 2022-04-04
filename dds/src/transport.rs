use rtps_pim::{messages::overall_structure::RtpsMessage, structure::types::Locator};
use rtps_udp_psm::messages::overall_structure::RtpsSubmessageType;

pub trait TransportWrite {
    fn write(
        &mut self,
        message: &RtpsMessage<Vec<RtpsSubmessageType<'_>>>,
        destination_locator: Locator,
    );
}

pub trait TransportRead {
    fn read(&mut self) -> Option<(Locator, RtpsMessage<Vec<RtpsSubmessageType<'_>>>)>;
}
