use crate::{RtpsUdpPsm, SubmessageFlag, Time};

use super::SubmessageHeader;

pub struct InfoTimestamp;

impl<'a> rust_rtps_pim::messages::submessages::InfoTimestampSubmessage<'a, RtpsUdpPsm> for InfoTimestamp {
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

impl<'a> rust_rtps_pim::messages::Submessage<'a, RtpsUdpPsm> for InfoTimestamp {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
