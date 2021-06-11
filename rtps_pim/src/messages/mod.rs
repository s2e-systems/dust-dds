pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefixPIM, ProtocolVersionPIM, VendorIdPIM};

use self::{
    submessage_elements::SubmessageElements,
    types::{ProtocolIdPIM, SubmessageFlag, SubmessageKindPIM},
};

pub trait RtpsMessageHeaderPIM<
    'a,
    PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM,
>
{
    type RtpsMessageHeaderType: RtpsMessageHeaderType<'a, PSM>;
}

pub trait RtpsMessageHeaderType<
    'a,
    PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM,
>
{
    fn protocol(&self) -> &PSM::ProtocolIdType;
    fn version(&self) -> &PSM::ProtocolVersionType;
    fn vendor_id(&self) -> &PSM::VendorIdType;
    fn guid_prefix(&self) -> &PSM::GuidPrefixType;
}

pub trait RtpsSubmessageHeaderPIM<PSM: SubmessageKindPIM> {
    type RtpsSubmessageHeaderType: RtpsSubmessageHeaderType<PSM>;
}

pub trait RtpsSubmessageHeaderType<PSM: SubmessageKindPIM> {
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage<PSM: SubmessageKindPIM + RtpsSubmessageHeaderPIM<PSM>> {
    fn submessage_header(&self) -> PSM::RtpsSubmessageHeaderType;

    fn submessage_elements(&self) -> &[SubmessageElements];
}

pub trait RTPSMessagePIM<
    'a,
    PSM: ProtocolIdPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + SubmessageKindPIM
        + RtpsMessageHeaderPIM<'a, PSM>
        + 'a,
>
{
    type RTPSMessageType: RTPSMessage<'a, PSM>;
}

pub trait RTPSMessage<
    'a,
    PSM: ProtocolIdPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + SubmessageKindPIM
        + RtpsMessageHeaderPIM<'a, PSM>
        + 'a,
>
{
    fn new(
        protocol: &'a PSM::ProtocolIdType,
        version: &'a PSM::ProtocolVersionType,
        vendor_id: &'a PSM::VendorIdType,
        guid_prefix: &'a PSM::GuidPrefixType,
        submessages: &[&'a dyn Submessage<PSM>],
    ) -> Self;

    fn header(&self) -> PSM::RtpsMessageHeaderType;

    fn submessages(&self) -> &[&'a dyn Submessage<PSM>];
}
