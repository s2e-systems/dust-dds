pub mod submessage_elements;
pub mod submessages;
mod types;

use crate::{messages, structure, RtpsPim};

pub use types::Types;

pub struct Header<PIM: RtpsPim> {
    pub protocol: <PIM as messages::Types>::ProtocolId,
    pub version: <PIM as structure::Types>::ProtocolVersion,
    pub vendor_id: <PIM as structure::Types>::VendorId,
    pub guid_prefix: <PIM as structure::Types>::GuidPrefix,
}

pub struct SubmessageHeader<PIM: RtpsPim> {
    pub submessage_id: <PIM as messages::Types>::SubmessageKind,
    pub flags: [<PIM as messages::Types>::SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage {
    type PIM: RtpsPim;
    fn submessage_header(&self) -> SubmessageHeader<Self::PIM>;
}