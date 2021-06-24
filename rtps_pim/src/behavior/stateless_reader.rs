use crate::{
    behavior::RTPSReader,
    messages::{submessage_elements::ParameterListSubmessageElementPIM, types::ParameterIdPIM},
    structure::types::{
        DataPIM, EntityIdPIM, InstanceHandlePIM, LocatorPIM, SequenceNumberPIM,
        GUIDPIM,
    },
};

use super::types::DurationPIM;

pub trait RTPSStatelessReader<
    PSM: InstanceHandlePIM
        + DataPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + LocatorPIM
        + DurationPIM
        + GUIDPIM
        + ParameterIdPIM
        + ParameterListSubmessageElementPIM,
>: RTPSReader<PSM>
{
}
