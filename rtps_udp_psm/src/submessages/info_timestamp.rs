use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{submessage_elements::TimeUdp, submessage_header::SubmessageHeader};

#[derive(Debug, PartialEq)]
pub struct InfoTimestampUdp;

impl<'a> rust_rtps_pim::messages::submessages::InfoTimestampSubmessage for InfoTimestampUdp {
    type TimestampSubmessageElementType = TimeUdp;

    fn new(
        _endianness_flag: SubmessageFlag,
        _invalidate_flag: SubmessageFlag,
        _timestamp: TimeUdp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn invalidate_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn timestamp(&self) -> &TimeUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoTimestampUdp {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
