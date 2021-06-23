pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefixPIM, ProtocolVersionPIM, VendorIdPIM};

use self::{
    submessages::{
        AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
        HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
        InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
        NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
    },
    types::{ProtocolIdPIM, SubmessageFlag, SubmessageKindPIM},
};

pub trait RtpsMessageHeaderPIM<PSM> {
    type RtpsMessageHeaderType;
}

pub trait RtpsMessageHeaderType<PSM>
where
    PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM,
{
    fn protocol(&self) -> &PSM::ProtocolIdType;
    fn version(&self) -> &PSM::ProtocolVersionType;
    fn vendor_id(&self) -> &PSM::VendorIdType;
    fn guid_prefix(&self) -> &PSM::GuidPrefixType;
}

pub trait RtpsSubmessageHeaderPIM<PSM> {
    type RtpsSubmessageHeaderType;
}

pub trait RtpsSubmessageHeaderType<PSM>
where
    PSM: SubmessageKindPIM,
{
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage<PSM>
where
    PSM: RtpsSubmessageHeaderPIM<PSM>,
{
    fn submessage_header(&self) -> PSM::RtpsSubmessageHeaderType;
}

pub trait RTPSMessagePIM<'a, PSM> {
    type RTPSMessageType;
}

pub trait RTPSMessage<'a, PSM>
where
    PSM: ProtocolIdPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + RtpsMessageHeaderPIM<PSM>
        + AckNackSubmessagePIM<PSM>
        + DataSubmessagePIM<'a, PSM>
        + DataFragSubmessagePIM<'a, PSM>
        + GapSubmessagePIM<PSM>
        + HeartbeatSubmessagePIM<PSM>
        + HeartbeatFragSubmessagePIM<PSM>
        + InfoDestinationSubmessagePIM<PSM>
        + InfoReplySubmessagePIM<PSM>
        + InfoSourceSubmessagePIM<PSM>
        + InfoTimestampSubmessagePIM<PSM>
        + NackFragSubmessagePIM<PSM>
        + PadSubmessagePIM<PSM>,
{
    fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, PSM>>>(
        protocol: PSM::ProtocolIdType,
        version: PSM::ProtocolVersionType,
        vendor_id: PSM::VendorIdType,
        guid_prefix: PSM::GuidPrefixType,
        submessages: T,
    ) -> Self;

    fn header(&self) -> PSM::RtpsMessageHeaderType;

    fn submessages(&self) -> &[RtpsSubmessageType<'a, PSM>];
}
