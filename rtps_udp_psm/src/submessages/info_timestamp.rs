use crate::{RtpsUdpPsm, SubmessageFlag, Time};

use super::SubmessageHeader;

#[derive(Debug, PartialEq)]
pub struct InfoTimestamp;

impl rust_rtps_pim::messages::submessages::InfoTimestampSubmessage<RtpsUdpPsm> for InfoTimestamp {
    fn new(
        _endianness_flag: SubmessageFlag,
        _invalidate_flag: SubmessageFlag,
        _timestamp: Time,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn invalidate_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn timestamp(&self) -> &Time {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoTimestamp {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
