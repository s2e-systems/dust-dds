pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

use self::types::{ProtocolId, SubmessageFlag, SubmessageKind};

#[derive(Debug, PartialEq)]
pub struct RtpsMessageHeader {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

#[derive(Debug, PartialEq)]
pub struct RtpsSubmessageHeader {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}

#[derive(Debug, PartialEq)]
pub struct RtpsMessage<M> {
    pub header: RtpsMessageHeader,
    pub submessages: M,
}