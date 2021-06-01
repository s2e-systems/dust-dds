use rust_rtps_pim::{
    behavior::types::DurationType,
    messages::types::{ParameterIdType, SubmessageFlagType, SubmessageKindType},
    structure::types::{
        DataType, EntityIdPIM, GUIDType, GuidPrefixPIM, InstanceHandleType, LocatorType,
        ParameterListType, ProtocolVersionType, SequenceNumberType, VendorIdType,
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
    + VendorIdType
    + EntityIdPIM
    + SequenceNumberType
    + ProtocolVersionType
    + DurationType
    + InstanceHandleType
    + LocatorType
    + DataType
    + GUIDType<Self>
    + ParameterIdType
    + ParameterListType<Self>
    + SubmessageKindType
    + SubmessageFlagType
    + Sized
    + 'static
{
}

impl<
        T: GuidPrefixPIM
            + VendorIdType
            + EntityIdPIM
            + SequenceNumberType
            + ProtocolVersionType
            + DurationType
            + InstanceHandleType
            + LocatorType
            + DataType
            + GUIDType<Self>
            + ParameterIdType
            + ParameterListType<Self>
            + SubmessageKindType
            + SubmessageFlagType
            + Sized
            + 'static,
    > PIM for T
{
}
