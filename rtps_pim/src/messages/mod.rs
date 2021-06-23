pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{
    DataPIM, EntityIdPIM, GuidPrefixPIM, LocatorPIM, ProtocolVersionPIM, SequenceNumberPIM,
    VendorIdPIM,
};

use self::{
    submessage_elements::{
        CountSubmessageElementPIM, EntityIdSubmessageElementPIM,
        FragmentNumberSetSubmessageElementPIM, FragmentNumberSubmessageElementPIM,
        GuidPrefixSubmessageElementPIM, LocatorListSubmessageElementPIM,
        ParameterListSubmessageElementPIM, ProtocolVersionSubmessageElementPIM,
        SequenceNumberSetSubmessageElementPIM, SequenceNumberSubmessageElementPIM,
        SerializedDataFragmentSubmessageElementPIM, SerializedDataSubmessageElementPIM,
        TimestampSubmessageElementPIM, ULongSubmessageElementPIM, UShortSubmessageElementPIM,
        VendorIdSubmessageElementPIM,
    },
    submessages::{
        AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
        HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
        InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
        NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
    },
    types::{
        CountPIM, FragmentNumberPIM, ParameterIdPIM, ProtocolIdPIM, SubmessageFlag,
        SubmessageKindPIM, TimePIM,
    },
};

pub trait RtpsMessageHeaderPIM<
    PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM,
>
{
    type RtpsMessageHeaderType: RtpsMessageHeaderType<PSM>;
}

pub trait RtpsMessageHeaderType<
    PSM: ProtocolIdPIM + ProtocolVersionPIM + VendorIdPIM + GuidPrefixPIM,
>
{
    fn protocol(&self) -> &PSM::ProtocolIdType;
    fn version(&self) -> &PSM::ProtocolVersionType;
    fn vendor_id(&self) -> &PSM::VendorIdType;
    fn guid_prefix(&self) -> &PSM::GuidPrefixType;
}

pub trait RtpsSubmessageHeaderPIM<PSM> {
    type RtpsSubmessageHeaderType;
}

pub trait RtpsSubmessageHeaderType<PSM: SubmessageKindPIM> {
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage<PSM: RtpsSubmessageHeaderPIM<PSM>> {
    fn submessage_header(&self) -> PSM::RtpsSubmessageHeaderType;
}

pub trait RTPSMessagePIM<'a, PSM>
where
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + ParameterIdPIM
        + DataPIM
        + FragmentNumberPIM
        + UShortSubmessageElementPIM
        + ULongSubmessageElementPIM
        + GuidPrefixPIM
        + LocatorPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + TimePIM
        + ProtocolIdPIM
        + ParameterListSubmessageElementPIM<PSM>
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>
        + SerializedDataSubmessageElementPIM<'a>
        + SerializedDataFragmentSubmessageElementPIM<'a>
        + FragmentNumberSubmessageElementPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>
        + LocatorListSubmessageElementPIM<PSM>
        + ProtocolVersionSubmessageElementPIM<PSM>
        + VendorIdSubmessageElementPIM<PSM>
        + TimestampSubmessageElementPIM<PSM>
        + FragmentNumberSetSubmessageElementPIM<PSM>
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
        + PadSubmessagePIM<PSM>
        + RtpsMessageHeaderPIM<PSM>
        + RTPSMessagePIM<'a, PSM>,
{
    type RTPSMessageType: RTPSMessage<'a, PSM>;
}

pub trait RTPSMessage<'a, PSM>
where
    PSM: SubmessageKindPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + ParameterIdPIM
        + DataPIM
        + FragmentNumberPIM
        + UShortSubmessageElementPIM
        + ULongSubmessageElementPIM
        + GuidPrefixPIM
        + LocatorPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + TimePIM
        + ProtocolIdPIM
        + ParameterListSubmessageElementPIM<PSM>
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>
        + SerializedDataSubmessageElementPIM<'a>
        + SerializedDataFragmentSubmessageElementPIM<'a>
        + FragmentNumberSubmessageElementPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>
        + LocatorListSubmessageElementPIM<PSM>
        + ProtocolVersionSubmessageElementPIM<PSM>
        + VendorIdSubmessageElementPIM<PSM>
        + TimestampSubmessageElementPIM<PSM>
        + FragmentNumberSetSubmessageElementPIM<PSM>
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
        + PadSubmessagePIM<PSM>
        + RtpsMessageHeaderPIM<PSM>,
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
