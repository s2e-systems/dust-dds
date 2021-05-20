pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::PIM;

pub use types::Types;

pub struct Header<PSM: PIM> {
    pub protocol: PSM::ProtocolId,
    pub version: PSM::ProtocolVersion,
    pub vendor_id: PSM::VendorId,
    pub guid_prefix: PSM::GuidPrefix,
}

pub struct SubmessageHeader<PSM: PIM> {
    pub submessage_id: PSM::SubmessageKind,
    pub flags: [PSM::SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage<PSM: PIM> {
    fn submessage_header(&self) -> SubmessageHeader<PSM>;
}