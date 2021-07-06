pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefix, VendorId};

use self::{
    submessages::{
        AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
        HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
        InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
        NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
    },
    types::{SubmessageFlag, SubmessageKindPIM},
};

pub trait RtpsMessageHeaderPIM {
    type RtpsMessageHeaderType;
}

pub trait RtpsMessageHeaderType {
    type ProtocolIdType;
    type ProtocolVersionType;
    type VendorIdType;
    type GuidPrefixType;

    const PROTOCOL_RTPS: Self::ProtocolIdType;

    fn protocol(&self) -> &Self::ProtocolIdType;
    fn version(&self) -> &Self::ProtocolVersionType;
    fn vendor_id(&self) -> &Self::VendorIdType;
    fn guid_prefix(&self) -> &Self::GuidPrefixType;
}

pub trait RtpsSubmessageHeaderType<PSM>
where
    PSM: SubmessageKindPIM,
{
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage {
    type RtpsSubmessageHeaderType;
    fn submessage_header(&self) -> Self::RtpsSubmessageHeaderType;
}

pub trait RTPSMessagePIM<'a> {
    type RTPSMessageType: RTPSMessage<'a>;
}

pub trait RTPSMessage<'a> {
    type RtpsMessageHeaderType: RtpsMessageHeaderType;
    type PSM:
            AckNackSubmessagePIM
            + DataSubmessagePIM<'a>
            + DataFragSubmessagePIM<'a>
            + GapSubmessagePIM
            + HeartbeatSubmessagePIM
            + HeartbeatFragSubmessagePIM
            + InfoDestinationSubmessagePIM
            + InfoReplySubmessagePIM
            + InfoSourceSubmessagePIM
            + InfoTimestampSubmessagePIM
            + NackFragSubmessagePIM
            + PadSubmessagePIM;

    fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, Self::PSM>>>(
        // protocol: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::ProtocolIdType,
        // version: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::ProtocolVersionType,
        // vendor_id: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::VendorIdType,
        // guid_prefix: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::GuidPrefixType,
        submessages: T,
    ) -> Self;


    fn header(&self) -> Self::RtpsMessageHeaderType;

    fn submessages<PSM>(&self) -> &[RtpsSubmessageType<'a, PSM>]
    where
        PSM: AckNackSubmessagePIM
            + DataSubmessagePIM<'a>
            + DataFragSubmessagePIM<'a>
            + GapSubmessagePIM
            + HeartbeatSubmessagePIM
            + HeartbeatFragSubmessagePIM
            + InfoDestinationSubmessagePIM
            + InfoReplySubmessagePIM
            + InfoSourceSubmessagePIM
            + InfoTimestampSubmessagePIM
            + NackFragSubmessagePIM
            + PadSubmessagePIM;
}
