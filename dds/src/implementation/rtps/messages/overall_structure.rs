use crate::implementation::rtps::types::{GuidPrefix, ProtocolVersion, VendorId};

use super::types::{ProtocolId, SubmessageFlag, SubmessageKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RtpsMessageHeader {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsSubmessageHeader {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}
