use crate::{
    behavior::RTPSReader,
    messages::types::ParameterIdType,
    structure::types::{
        DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType, LocatorType,
        ParameterListType, SequenceNumberType,
    },
};

use super::types::DurationType;

pub trait RTPSStatelessReader<
    PSM: InstanceHandleType
        + GuidPrefixType
        + DataType
        + EntityIdType
        + SequenceNumberType
        + LocatorType
        + DurationType
        + GUIDType<PSM>
        + ParameterIdType
        + ParameterListType<PSM>,
>: RTPSReader<PSM>
{
}
