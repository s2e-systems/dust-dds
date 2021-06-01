use crate::{
    behavior::RTPSReader,
    messages::types::ParameterIdPIM,
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ParameterListPIM,
        SequenceNumberPIM, GUIDPIM,
    },
};

use super::types::DurationPIM;

pub trait RTPSStatelessReader<
    PSM: InstanceHandlePIM
        + GuidPrefixPIM
        + DataPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + LocatorPIM
        + DurationPIM
        + GUIDPIM<PSM>
        + ParameterIdPIM
        + ParameterListPIM<PSM>,
>: RTPSReader<PSM>
{
}
