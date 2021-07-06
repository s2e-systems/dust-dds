use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{psm::RtpsUdpPsm, submessage_elements::Time, submessage_header::SubmessageHeader};

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

impl rust_rtps_pim::messages::Submessage for InfoTimestamp {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
