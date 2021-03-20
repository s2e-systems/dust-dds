use super::SubmessageHeader;
use rust_rtps_pim::messages::submessages::Submessage;

pub struct Pad {}

impl Submessage for Pad {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::pad::Pad for Pad {}
