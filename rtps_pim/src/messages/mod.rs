pub mod submessage_elements;
pub mod submessages;
mod types;

use crate::RtpsPim;

pub use types::Types;

pub struct Header<PSM: RtpsPim> {
    pub protocol: PSM::ProtocolId,
    pub version: PSM::ProtocolVersion,
    pub vendor_id: PSM::VendorId,
    pub guid_prefix: PSM::GuidPrefix,
}

pub struct SubmessageHeader<PSM: RtpsPim> {
    pub submessage_id: PSM::SubmessageKind,
    pub flags: [PSM::SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage {
    type PSM: RtpsPim;
    fn submessage_header(&self) -> SubmessageHeader<Self::PSM>;
}
