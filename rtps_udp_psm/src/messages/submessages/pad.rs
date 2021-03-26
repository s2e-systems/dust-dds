use rust_rtps_pim::messages::submessages::SubmessageHeader;

use crate::messages::types::{SubmessageFlag, SubmessageKind};

pub struct Pad {}

impl rust_rtps_pim::messages::submessages::pad::Pad for Pad {
    fn new() -> Self {
        Self {}
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for Pad {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
