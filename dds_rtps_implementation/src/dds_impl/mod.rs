use rust_rtps_pim::{behavior::types::DurationPIM, messages::{RTPSMessagePIM, SubmessageHeaderPIM, submessages::{DataSubmessagePIM, GapSubmessagePIM}, types::{ParameterIdPIM, ProtocolIdPIM, SubmessageFlagPIM, SubmessageKindPIM}}, structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ParameterListPIM,
        ProtocolVersionPIM, SequenceNumberPIM, VendorIdPIM, GUIDPIM,
    }};

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
    + GUIDPIM
    + ParameterIdPIM
    + ParameterListPIM<Self>
    + SubmessageKindPIM
    + SubmessageFlagPIM
    + ProtocolIdPIM
    + SubmessageHeaderPIM<Self>
    + for<'a> RTPSMessagePIM<'a, Self>
    + for<'a> DataSubmessagePIM<'a, Self>
    + GapSubmessagePIM<Self>
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
            + GUIDPIM
            + ParameterIdPIM
            + ParameterListPIM<Self>
            + SubmessageKindPIM
            + SubmessageFlagPIM
            + ProtocolIdPIM
            + SubmessageHeaderPIM<Self>
            + for<'a> RTPSMessagePIM<'a, Self>
            + for<'a> DataSubmessagePIM<'a, Self>
            + GapSubmessagePIM<Self>
            + Sized
            + 'static,
    > PIM for T
{
}
