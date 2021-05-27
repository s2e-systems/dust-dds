use crate::{GuidPrefix, ProtocolVersion, RtpsUdpPsm, SubmessageFlag, VendorId};

pub struct InfoSource;

impl rust_rtps_pim::messages::submessages::InfoSource<RtpsUdpPsm> for InfoSource {
    type GuidPrefix = GuidPrefix;
    type ProtocolVersion = ProtocolVersion;
    type VendorId = VendorId;

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn protocol_version(&self) -> Self::ProtocolVersion {
        todo!()
    }

    fn vendor_id(&self) -> &Self::VendorId {
        todo!()
    }

    fn guid_prefix(&self) -> &Self::GuidPrefix {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoSource {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
