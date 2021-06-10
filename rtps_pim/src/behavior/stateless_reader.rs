use crate::{
    behavior::RTPSReader,
    messages::{submessage_elements::ParameterListSubmessageElementPIM, types::ParameterIdPIM},
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, SequenceNumberPIM,
        GUIDPIM,
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
        + ParameterListSubmessageElementPIM<PSM>,
>: RTPSReader<PSM>
{
}
