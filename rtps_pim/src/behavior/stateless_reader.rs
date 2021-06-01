use crate::{
    behavior::RTPSReader,
    messages::types::ParameterIdType,
    structure::types::{
        DataType, EntityIdPIM, GUIDType, GuidPrefixPIM, InstanceHandleType, LocatorType,
        ParameterListType, SequenceNumberType,
    },
};

use super::types::DurationType;

pub trait RTPSStatelessReader<
    PSM: InstanceHandleType
        + GuidPrefixPIM
        + DataType
        + EntityIdPIM
        + SequenceNumberType
        + LocatorType
        + DurationType
        + GUIDType<PSM>
        + ParameterIdType
        + ParameterListType<PSM>,
>: RTPSReader<PSM>
{
}
