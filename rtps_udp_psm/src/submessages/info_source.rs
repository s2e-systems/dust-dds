use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct InfoSource {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    protocol_version:
        rust_rtps_pim::messages::submessage_elements::ProtocolVersion<<Self as Submessage>::PSM>,
    vendor_id: rust_rtps_pim::messages::submessage_elements::VendorId<<Self as Submessage>::PSM>,
    guid_prefix:
        rust_rtps_pim::messages::submessage_elements::GuidPrefix<<Self as Submessage>::PSM>,
}

impl Submessage for InfoSource {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::InfoSource for InfoSource {
    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        protocol_version: rust_rtps_pim::messages::submessage_elements::ProtocolVersion<Self::PSM>,
        vendor_id: rust_rtps_pim::messages::submessage_elements::VendorId<Self::PSM>,
        guid_prefix: rust_rtps_pim::messages::submessage_elements::GuidPrefix<Self::PSM>,
    ) -> Self {
        Self {
            endianness_flag,
            protocol_version,
            vendor_id,
            guid_prefix,
        }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
    }

    fn protocol_version(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::ProtocolVersion<Self::PSM> {
        &self.protocol_version
    }

    fn vendor_id(&self) -> &rust_rtps_pim::messages::submessage_elements::VendorId<Self::PSM> {
        &self.vendor_id
    }

    fn guid_prefix(&self) -> &rust_rtps_pim::messages::submessage_elements::GuidPrefix<Self::PSM> {
        &self.guid_prefix
    }
}
