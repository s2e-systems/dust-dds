use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{SubmessageFlag, SubmessageKind},
    types::{GuidPrefix, ProtocolVersion, VendorId},
};

pub struct InfoSource {
    endianness_flag: SubmessageFlag,
    protocol_version: submessage_elements::ProtocolVersion<ProtocolVersion>,
    vendor_id: submessage_elements::VendorId<VendorId>,
    guid_prefix: submessage_elements::GuidPrefix<GuidPrefix>,
}

impl rust_rtps_pim::messages::submessages::info_source_submessage::InfoSource for InfoSource {
    type ProtocolVersion = ProtocolVersion;
    type VendorId = VendorId;
    type GuidPrefix = GuidPrefix;

    fn new(
        endianness_flag: SubmessageFlag,
        protocol_version: submessage_elements::ProtocolVersion<Self::ProtocolVersion>,
        vendor_id: submessage_elements::VendorId<Self::VendorId>,
        guid_prefix: submessage_elements::GuidPrefix<Self::GuidPrefix>,
    ) -> Self {
        Self {
            endianness_flag,
            protocol_version,
            vendor_id,
            guid_prefix,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn protocol_version(&self) -> &submessage_elements::ProtocolVersion<Self::ProtocolVersion> {
        &self.protocol_version
    }

    fn vendor_id(&self) -> &submessage_elements::VendorId<Self::VendorId> {
        &self.vendor_id
    }

    fn guid_prefix(&self) -> &submessage_elements::GuidPrefix<Self::GuidPrefix> {
        &self.guid_prefix
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for InfoSource {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
