use super::{
    submessage_elements::{
        GuidPrefixSubmessageElement, ProtocolVersionSubmessageElement, VendorIdSubmessageElement,
    },
    types::{ProtocolId, SubmessageFlag, SubmessageKind},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RtpsMessageHeader {
    pub protocol: ProtocolId,
    pub version: ProtocolVersionSubmessageElement,
    pub vendor_id: VendorIdSubmessageElement,
    pub guid_prefix: GuidPrefixSubmessageElement,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsSubmessageHeader {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}
