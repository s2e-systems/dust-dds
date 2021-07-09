use rust_rtps_pim::messages::RtpsSubmessageHeader;

#[derive(Debug, PartialEq)]
pub struct PadUdp;

impl rust_rtps_pim::messages::submessages::PadSubmessage for PadUdp {}

impl rust_rtps_pim::messages::Submessage for PadUdp {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
