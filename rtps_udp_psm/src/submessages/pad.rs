use crate::{submessage_header::SubmessageHeader};

#[derive(Debug, PartialEq)]
pub struct PadUdp;

impl rust_rtps_pim::messages::submessages::PadSubmessage for PadUdp {}

impl rust_rtps_pim::messages::Submessage for PadUdp {
    type RtpsSubmessageHeaderType = SubmessageHeader;

    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
