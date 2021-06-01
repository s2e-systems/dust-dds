pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefixPIM, ProtocolVersionType, VendorIdType};

use self::types::{ProtocolIdType, SubmessageFlagType, SubmessageKindType};

pub trait Header<PSM: ProtocolIdType + ProtocolVersionType + VendorIdType + GuidPrefixPIM> {
    fn protocol(&self) -> PSM::ProtocolId;
    fn version(&self) -> PSM::ProtocolVersion;
    fn vendor_id(&self) -> PSM::VendorId;
    fn guid_prefix(&self) -> PSM::GuidPrefixType;
}

pub trait SubmessageHeader<PSM: SubmessageFlagType + SubmessageKindType> {
    fn submessage_id(&self) -> PSM::SubmessageKind;
    fn flags(&self) -> [PSM::SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage<PSM: SubmessageFlagType + SubmessageKindType> {
    type SubmessageHeader: SubmessageHeader<PSM>;
    fn submessage_header(&self) -> Self::SubmessageHeader;
}
