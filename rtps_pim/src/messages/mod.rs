pub mod submessage_elements;
pub mod submessages;
mod types;

use crate::{messages, structure};

pub use types::Types;

pub struct Header<PSM: messages::Types + structure::Types> {
    pub protocol: PSM::ProtocolId,
    pub version: PSM::ProtocolVersion,
    pub vendor_id: PSM::VendorId,
    pub guid_prefix: PSM::GuidPrefix,
}

pub struct SubmessageHeader<PSM: messages::Types + structure::Types> {
    pub submessage_id: PSM::SubmessageKind,
    pub flags: [PSM::SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage {
    type PSM: messages::Types + structure::Types;
    fn submessage_header(&self) -> SubmessageHeader<Self::PSM>;
}
