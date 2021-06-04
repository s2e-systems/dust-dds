use crate::{RtpsUdpPsm, SubmessageFlag, Time};

use super::SubmessageHeader;

pub struct InfoTimestamp;

impl rust_rtps_pim::messages::submessages::InfoTimestampSubmessage<RtpsUdpPsm> for InfoTimestamp {
    type Timestamp = Time;

    fn new(
        _endianness_flag: SubmessageFlag,
        _invalidate_flag: SubmessageFlag,
        _timestamp: Self::Timestamp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn invalidate_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn timestamp(&self) -> &Self::Timestamp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoTimestamp {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
