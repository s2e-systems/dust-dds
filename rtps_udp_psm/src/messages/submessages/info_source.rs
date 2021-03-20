use super::SubmessageHeader;
use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

pub struct InfoSource {
    endianness_flag: SubmessageFlag,
    protocol_version: <Self as rust_rtps_pim::messages::submessages::info_source_submessage::InfoSource>::ProtocolVersion,
    vendor_id: <Self as rust_rtps_pim::messages::submessages::info_source_submessage::InfoSource>::VendorId,
    guid_prefix: <Self as rust_rtps_pim::messages::submessages::info_source_submessage::InfoSource>::GuidPrefix,
}

impl Submessage for InfoSource {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::info_source_submessage::InfoSource for InfoSource {
    type ProtocolVersion = submessage_elements::ProtocolVersion;
    type VendorId = submessage_elements::VendorId;
    type GuidPrefix = submessage_elements::GuidPrefix;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn protocol_version(&self) -> &Self::ProtocolVersion {
        &self.protocol_version
    }

    fn vendor_id(&self) -> &Self::VendorId {
        &self.vendor_id
    }

    fn guid_prefix(&self) -> &Self::GuidPrefix {
        &self.guid_prefix
    }
}
