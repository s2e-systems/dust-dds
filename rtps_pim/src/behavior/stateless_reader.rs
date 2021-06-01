use crate::{
    behavior::RTPSReader,
    messages::types::ParameterIdPIM,
    structure::types::{
        DataPIM, EntityIdPIM, GUIDPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM,
        ParameterListPIM, SequenceNumberPIM,
    },
};

use super::types::DurationType;

pub trait RTPSStatelessReader<
    PSM: InstanceHandlePIM
        + GuidPrefixPIM
        + DataPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + LocatorPIM
        + DurationType
        + GUIDPIM<PSM>
        + ParameterIdPIM
        + ParameterListPIM<PSM>,
>: RTPSReader<PSM>
{
}
