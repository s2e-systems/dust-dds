pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefixPIM, ProtocolVersionPIM, VendorIdPIM};

use self::types::{ProtocolIdPIM, SubmessageFlagPIM, SubmessageKindPIM};

pub trait Header<PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM> {
    fn protocol(&self) -> PSM::ProtocolIdType;
    fn version(&self) -> PSM::ProtocolVersionType;
    fn vendor_id(&self) -> PSM::VendorIdType;
    fn guid_prefix(&self) -> PSM::GuidPrefixType;
}

pub trait SubmessageHeader<PSM: SubmessageFlagPIM + SubmessageKindPIM> {
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [PSM::SubmessageFlagType; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage<PSM: SubmessageFlagPIM + SubmessageKindPIM> {
    type SubmessageHeader: SubmessageHeader<PSM>;
    fn submessage_header(&self) -> Self::SubmessageHeader;
}
