pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

use self::{
    submessages::{RtpsSubmessagePIM, RtpsSubmessageType},
    types::{ProtocolId, SubmessageFlag, SubmessageKind},
};

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

pub trait RTPSMessage<'a> {
    type PSM: RtpsSubmessagePIM<'a>;
    type Constructed;

    fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, Self::PSM>>>(
        header: &RtpsMessageHeader,
        submessages: T,
    ) -> Self::Constructed;

    fn header(&self) -> RtpsMessageHeader;

    fn submessages(&self) -> &[RtpsSubmessageType<'a, Self::PSM>];
}
