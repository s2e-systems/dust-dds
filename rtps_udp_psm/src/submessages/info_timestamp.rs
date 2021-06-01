use crate::{RtpsUdpPsm, SubmessageFlag, Time};

use super::SubmessageHeader;

pub struct InfoTimestamp;

impl rust_rtps_pim::messages::submessages::InfoTimestampSubmessage<RtpsUdpPsm> for InfoTimestamp {
    type Timestamp = Time;

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
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
