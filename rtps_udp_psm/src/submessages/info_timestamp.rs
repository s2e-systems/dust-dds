use crate::{RtpsUdpPsm, SubmessageFlag, Time};

pub struct InfoTimestamp;

impl rust_rtps_pim::messages::submessages::InfoTimestamp<RtpsUdpPsm> for InfoTimestamp {
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
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
