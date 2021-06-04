pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefixPIM, ProtocolVersionPIM, VendorIdPIM};

use self::types::{ProtocolIdPIM, SubmessageFlagPIM, SubmessageKindPIM};

pub trait Header<PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM> {
    fn protocol(&self) -> PSM::ProtocolIdType;
    fn version(&self) -> PSM::ProtocolVersionType;
    fn vendor_id(&self) -> PSM::VendorIdType;
    fn guid_prefix(&self) -> PSM::GuidPrefixType;
}

pub trait SubmessageHeaderPIM<PSM: SubmessageFlagPIM + SubmessageKindPIM> {
    type SubmessageHeaderType: SubmessageHeader<PSM>;
}

pub trait SubmessageHeader<PSM: SubmessageFlagPIM + SubmessageKindPIM> {
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [PSM::SubmessageFlagType; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage<PSM: SubmessageFlagPIM + SubmessageKindPIM + SubmessageHeaderPIM<PSM>> {
    fn submessage_header(&self) -> PSM::SubmessageHeaderType;
}

pub trait RTPSMessagePIM<
    'a,
    PSM: ProtocolIdPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + SubmessageFlagPIM
        + SubmessageKindPIM
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
        + SubmessageFlagPIM
        + SubmessageKindPIM
        + 'a,
>
{
    type RTPSMessageHeaderType: Header<PSM>;
    type RTPSSubmessageVectorType: IntoIterator<Item = &'a dyn Submessage<PSM>>;

    fn new<T: IntoIterator<Item = &'a dyn Submessage<PSM>>>(
        protocol: PSM::ProtocolIdType,
        version: PSM::ProtocolVersionType,
        vendor_id: PSM::VendorIdType,
        guid_prefix: PSM::GuidPrefixType,
        submessages: T,
    ) -> Self;

    fn header(&self) -> Self::RTPSMessageHeaderType;
}
