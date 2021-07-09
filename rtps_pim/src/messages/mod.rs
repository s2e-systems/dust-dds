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

pub trait RtpsSubmessageHeaderType {
    fn submessage_id(&self) -> SubmessageKind;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage {
    type RtpsSubmessageHeaderType;
    fn submessage_header(&self) -> Self::RtpsSubmessageHeaderType;
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
