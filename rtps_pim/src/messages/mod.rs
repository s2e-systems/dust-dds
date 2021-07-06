pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

use self::{submessage_elements::{EntityIdSubmessageElementPIM, ParameterListSubmessageElementPIM, SequenceNumberSubmessageElementPIM, SerializedDataSubmessageElementPIM}, submessages::{
        AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
        HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
        InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
        NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
    }, types::{ProtocolIdPIM, SubmessageFlag, SubmessageKindPIM}};

pub trait RtpsMessageHeaderPIM {
    type RtpsMessageHeaderType;
}

pub trait RtpsMessageHeaderType<PSM>
where
    PSM: ProtocolIdPIM,
{
    fn protocol(&self) -> &PSM::ProtocolIdType;
    fn version(&self) -> &ProtocolVersion;
    fn vendor_id(&self) -> &VendorId;
    fn guid_prefix(&self) -> &GuidPrefix;
}

pub trait RtpsSubmessageHeaderType<PSM>
where
    PSM: SubmessageKindPIM,
{
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage
{
    type RtpsSubmessageHeaderType;
    fn submessage_header(&self) -> Self::RtpsSubmessageHeaderType;
}

pub trait RTPSMessagePIM<'a, PSM> {
    type RTPSMessageType;
}

pub trait RTPSMessage<'a, PSM>
where
    PSM: ProtocolIdPIM
        + RtpsMessageHeaderPIM
        + AckNackSubmessagePIM
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
        + PadSubmessagePIM
{
    fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, PSM>>>(
        protocol: PSM::ProtocolIdType,
        version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
        submessages: T,
    ) -> Self;

    fn header(&self) -> PSM::RtpsMessageHeaderType;

    fn submessages(&self) -> &[RtpsSubmessageType<'a, PSM>];
}
