use rust_rtps_pim::{
    behavior::types::DurationType,
    messages::types::{ParameterIdPIM, SubmessageFlagType, SubmessageKindType},
    structure::types::{
        DataPIM, EntityIdPIM, GUIDPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM,
        ParameterListPIM, ProtocolVersionPIM, SequenceNumberPIM, VendorIdPIM,
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
    + DurationType
    + InstanceHandlePIM
    + LocatorPIM
    + DataPIM
    + GUIDPIM<Self>
    + ParameterIdPIM
    + ParameterListPIM<Self>
    + SubmessageKindType
    + SubmessageFlagType
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
            + DurationType
            + InstanceHandlePIM
            + LocatorPIM
            + DataPIM
            + GUIDPIM<Self>
            + ParameterIdPIM
            + ParameterListPIM<Self>
            + SubmessageKindType
            + SubmessageFlagType
            + Sized
            + 'static,
    > PIM for T
{
}
