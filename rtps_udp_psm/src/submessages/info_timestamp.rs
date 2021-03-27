use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct InfoTimestamp {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    invalidate_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    timestamp: rust_rtps_pim::messages::submessage_elements::Timestamp<<Self as Submessage>::PSM>,
}

impl Submessage for InfoTimestamp {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::InfoTimestamp for InfoTimestamp {
    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        invalidate_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        timestamp: rust_rtps_pim::messages::submessage_elements::Timestamp<Self::PSM>,
    ) -> Self {
        Self {
            endianness_flag,
            invalidate_flag,
            timestamp,
        }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
    }

    fn invalidate_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.invalidate_flag
    }

    fn timestamp(&self) -> &rust_rtps_pim::messages::submessage_elements::Timestamp<Self::PSM> {
        &self.timestamp
    }
}
