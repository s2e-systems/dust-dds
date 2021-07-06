use crate::{submessage_header::SubmessageHeader};

#[derive(Debug, PartialEq)]
pub struct Pad;

impl rust_rtps_pim::messages::submessages::PadSubmessage for Pad {}

impl rust_rtps_pim::messages::Submessage for Pad {
    type RtpsSubmessageHeaderType = SubmessageHeader;

    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
