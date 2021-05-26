pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefixType, ProtocolVersionType, VendorIdType};

use self::types::{ProtocolIdType, SubmessageFlagType, SubmessageKindType};

pub struct Header<PSM: ProtocolIdType + ProtocolVersionType + VendorIdType + GuidPrefixType> {
    pub protocol: PSM::ProtocolId,
    pub version: PSM::ProtocolVersion,
    pub vendor_id: PSM::VendorId,
    pub guid_prefix: PSM::GuidPrefix,
}

pub struct SubmessageHeader<PSM: SubmessageFlagType + SubmessageKindType> {
    pub submessage_id: PSM::SubmessageKind,
    pub flags: [PSM::SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage<PSM: SubmessageFlagType + SubmessageKindType> {
    fn submessage_header(&self) -> SubmessageHeader<PSM>;
}
