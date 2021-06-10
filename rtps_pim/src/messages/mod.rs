pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefixPIM, ProtocolVersionPIM, VendorIdPIM};

use self::types::{ProtocolIdPIM, SubmessageFlag, SubmessageKindPIM};

pub trait RtpsMessageHeader<PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM> {
    fn protocol(&self) -> &PSM::ProtocolIdType;
    fn version(&self) -> &PSM::ProtocolVersionType;
    fn vendor_id(&self) -> &PSM::VendorIdType;
    fn guid_prefix(&self) -> &PSM::GuidPrefixType;
}

pub trait SubmessageHeaderPIM<PSM: SubmessageKindPIM> {
    type SubmessageHeaderType: SubmessageHeader<PSM>;
}

pub trait SubmessageHeader<PSM: SubmessageKindPIM> {
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage<PSM: SubmessageKindPIM + SubmessageHeaderPIM<PSM>> {
    fn submessage_header(&self) -> PSM::SubmessageHeaderType;
}

pub trait RTPSMessagePIM<
    'a,
    PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM + SubmessageKindPIM + 'a,
>
{
    type RTPSMessageType: RTPSMessage<'a, PSM>;
}

pub trait RTPSMessage<
    'a,
    PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM + SubmessageKindPIM + 'a,
>
{
    type RTPSMessageHeaderType: RtpsMessageHeader<PSM>;
    type RTPSSubmessageVectorType: IntoIterator<Item = &'a dyn Submessage<PSM>>;

    fn new<T: IntoIterator<Item = &'a dyn Submessage<PSM>>>(
        protocol: &'a PSM::ProtocolIdType,
        version: &'a PSM::ProtocolVersionType,
        vendor_id: &'a PSM::VendorIdType,
        guid_prefix: &'a PSM::GuidPrefixType,
        submessages: T,
    ) -> Self;

    fn header(&self) -> Self::RTPSMessageHeaderType;
}
