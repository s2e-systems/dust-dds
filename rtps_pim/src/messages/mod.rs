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
pub struct RtpsSubmessageHeader {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader;
}

pub trait RtpsMessage {
    type SubmessageType;

    fn new<T: IntoIterator<Item = Self::SubmessageType>>(
        header: &RtpsMessageHeader,
        submessages: T,
    ) -> Self;

    fn header(&self) -> RtpsMessageHeader;

    fn submessages(&self) -> &[Self::SubmessageType];
}
