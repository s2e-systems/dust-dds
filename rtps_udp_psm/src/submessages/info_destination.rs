use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct InfoDestination {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    guid_prefix:
        rust_rtps_pim::messages::submessage_elements::GuidPrefix<<Self as Submessage>::PSM>,
}

impl Submessage for InfoDestination {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::InfoDestination for InfoDestination {
    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        guid_prefix: rust_rtps_pim::messages::submessage_elements::GuidPrefix<Self::PSM>,
    ) -> Self {
        Self {
            endianness_flag,
            guid_prefix,
        }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
    }

    fn guid_prefix(&self) -> &rust_rtps_pim::messages::submessage_elements::GuidPrefix<Self::PSM> {
        &self.guid_prefix
    }
}
