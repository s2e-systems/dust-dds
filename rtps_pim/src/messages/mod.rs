pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure;

pub use types::Types;

pub struct Header<PSM: structure::Types + self::Types> {
    pub protocol: PSM::ProtocolId,
    pub version: PSM::ProtocolVersion,
    pub vendor_id: PSM::VendorId,
    pub guid_prefix: PSM::GuidPrefix,
}

pub struct SubmessageHeader<PSM: self::Types> {
    pub submessage_id: PSM::SubmessageKind,
    pub flags: [PSM::SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage<PSM: self::Types> {
    fn submessage_header(&self) -> SubmessageHeader<PSM>;
}
