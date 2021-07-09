pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

use self::{
    submessages::{RtpsSubmessagePIM, RtpsSubmessageType},
    types::{ProtocolId, SubmessageFlag, SubmessageKind},
};

pub trait RtpsMessageHeaderType {
    fn new(
        protocol: ProtocolId,
        version: &ProtocolVersion,
        vendor_id: &VendorId,
        guid_prefix: &GuidPrefix,
    ) -> Self;

    fn protocol(&self) -> ProtocolId;
    fn version(&self) -> ProtocolVersion;
    fn vendor_id(&self) -> VendorId;
    fn guid_prefix(&self) -> GuidPrefix;
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
    type RtpsMessageHeaderType: RtpsMessageHeaderType;
    type PSM: RtpsSubmessagePIM<'a>;
    type Constructed;

    fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, Self::PSM>>>(
        header: &Self::RtpsMessageHeaderType,
        submessages: T,
    ) -> Self::Constructed;

    fn header(&self) -> Self::RtpsMessageHeaderType;

    fn submessages(&self) -> &[RtpsSubmessageType<'a, Self::PSM>];
}
