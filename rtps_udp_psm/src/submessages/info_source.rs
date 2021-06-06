use crate::{GuidPrefix, ProtocolVersion, RtpsUdpPsm, SubmessageFlag, VendorId};

use super::SubmessageHeader;

pub struct InfoSource;

impl rust_rtps_pim::messages::submessages::InfoSourceSubmessage<RtpsUdpPsm> for InfoSource {
    type GuidPrefix = GuidPrefix;
    type ProtocolVersion = ProtocolVersion;
    type VendorId = VendorId;

    fn new(
        _endianness_flag: SubmessageFlag,
        _protocol_version: Self::ProtocolVersion,
        _vendor_id: Self::VendorId,
        _guid_prefix: Self::GuidPrefix,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn protocol_version(&self) -> &Self::ProtocolVersion {
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
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
