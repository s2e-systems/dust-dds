use rust_rtps_pim::{
    behavior::types::DurationType,
    messages::types::ParameterIdType,
    structure::types::{
        DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType, LocatorType,
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
    GuidPrefixType
    + VendorIdType
    + EntityIdType
    + SequenceNumberType
    + ProtocolVersionType
    + DurationType
    + InstanceHandleType
    + LocatorType
    + DataType
    + GUIDType<Self>
    + ParameterIdType
    + ParameterListType<Self>
    + Sized
    + 'static
{
}

impl<
        T: GuidPrefixType
            + VendorIdType
            + EntityIdType
            + SequenceNumberType
            + ProtocolVersionType
            + DurationType
            + InstanceHandleType
            + LocatorType
            + DataType
            + GUIDType<Self>
            + ParameterIdType
            + ParameterListType<Self>
            + Sized
            + 'static,
    > PIM for T
{
}
