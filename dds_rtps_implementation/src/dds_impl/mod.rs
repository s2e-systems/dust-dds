use rust_rtps_pim::{
    behavior::types::DurationPIM,
    messages::{
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
            AckNackSubmessage, AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM,
            GapSubmessagePIM, HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM,
            InfoDestinationSubmessagePIM, InfoReplySubmessagePIM, InfoSourceSubmessagePIM,
            InfoTimestampSubmessagePIM, NackFragSubmessagePIM, PadSubmessagePIM,
        },
        types::{
            CountPIM, FragmentNumberPIM, ParameterIdPIM, ProtocolIdPIM, SubmessageKindPIM, TimePIM,
        },
        RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM,
    },
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ProtocolVersionPIM,
        SequenceNumberPIM, VendorIdPIM, GUIDPIM,
    },
};

pub mod data_reader_impl;
pub mod data_writer_impl;
pub mod domain_participant_impl;
pub mod publisher_impl;
pub mod subscriber_impl;
pub mod topic_impl;

pub mod writer_factory;
pub mod writer_group_factory;

pub trait PIM:
    GuidPrefixPIM
    + VendorIdPIM
    + EntityIdPIM
    + SequenceNumberPIM
    + ProtocolVersionPIM
    + DurationPIM
    + InstanceHandlePIM
    + LocatorPIM
    + DataPIM
    + CountPIM
    + FragmentNumberPIM
    + GUIDPIM<Self>
    + ParameterIdPIM
    + ParameterListSubmessageElementPIM<Self>
    + SubmessageKindPIM
    + ProtocolIdPIM
    + TimePIM
    + RtpsSubmessageHeaderPIM<Self>
    + EntityIdSubmessageElementPIM<Self>
    + SequenceNumberSubmessageElementPIM<Self>
    + SequenceNumberSetSubmessageElementPIM<Self>
    + CountSubmessageElementPIM<Self>
    + FragmentNumberSubmessageElementPIM<Self>
    + GuidPrefixSubmessageElementPIM<Self>
    + UShortSubmessageElementPIM
    + ULongSubmessageElementPIM
    + for<'a> SerializedDataSubmessageElementPIM<'a>
    + for<'a> SerializedDataFragmentSubmessageElementPIM<'a>
    + LocatorListSubmessageElementPIM<Self>
    + ProtocolVersionSubmessageElementPIM<Self>
    + VendorIdSubmessageElementPIM<Self>
    + TimestampSubmessageElementPIM<Self>
    + FragmentNumberSetSubmessageElementPIM<Self>
    + RtpsMessageHeaderPIM<Self>
    + AckNackSubmessagePIM<Self>
    + for<'a> DataSubmessagePIM<'a, Self>
    + for<'a> DataFragSubmessagePIM<'a, Self>
    + GapSubmessagePIM<Self>
    + HeartbeatSubmessagePIM<Self>
    + HeartbeatFragSubmessagePIM<Self>
    + InfoDestinationSubmessagePIM<Self>
    + InfoReplySubmessagePIM<Self>
    + InfoSourceSubmessagePIM<Self>
    + InfoTimestampSubmessagePIM<Self>
    + NackFragSubmessagePIM<Self>
    + PadSubmessagePIM<Self>
    + for<'a> RTPSMessagePIM<'a, Self>
    + Sized
    + 'static
{
}

impl<
        T: GuidPrefixPIM
            + VendorIdPIM
            + EntityIdPIM
            + SequenceNumberPIM
            + ProtocolVersionPIM
            + DurationPIM
            + InstanceHandlePIM
            + LocatorPIM
            + DataPIM
            + CountPIM
            + FragmentNumberPIM
            + GUIDPIM<Self>
            + ParameterIdPIM
            + ParameterListSubmessageElementPIM<Self>
            + SubmessageKindPIM
            + ProtocolIdPIM
            + TimePIM
            + RtpsSubmessageHeaderPIM<Self>
            + EntityIdSubmessageElementPIM<Self>
            + SequenceNumberSubmessageElementPIM<Self>
            + SequenceNumberSetSubmessageElementPIM<Self>
            + CountSubmessageElementPIM<Self>
            + FragmentNumberSubmessageElementPIM<Self>
            + GuidPrefixSubmessageElementPIM<Self>
            + UShortSubmessageElementPIM
            + ULongSubmessageElementPIM
            + for<'a> SerializedDataSubmessageElementPIM<'a>
            + for<'a> SerializedDataFragmentSubmessageElementPIM<'a>
            + LocatorListSubmessageElementPIM<Self>
            + ProtocolVersionSubmessageElementPIM<Self>
            + VendorIdSubmessageElementPIM<Self>
            + TimestampSubmessageElementPIM<Self>
            + FragmentNumberSetSubmessageElementPIM<Self>
            + RtpsMessageHeaderPIM<Self>
            + AckNackSubmessagePIM<Self>
            + for<'a> DataSubmessagePIM<'a, Self>
            + for<'a> DataFragSubmessagePIM<'a, Self>
            + GapSubmessagePIM<Self>
            + HeartbeatSubmessagePIM<Self>
            + HeartbeatFragSubmessagePIM<Self>
            + InfoDestinationSubmessagePIM<Self>
            + InfoReplySubmessagePIM<Self>
            + InfoSourceSubmessagePIM<Self>
            + InfoTimestampSubmessagePIM<Self>
            + NackFragSubmessagePIM<Self>
            + PadSubmessagePIM<Self>
            + for<'a> RTPSMessagePIM<'a, Self>
            + Sized
            + 'static,
    > PIM for T
{
}
