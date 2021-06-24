use crate::{
    behavior::RTPSReader,
    messages::{submessage_elements::ParameterListSubmessageElementPIM, types::ParameterIdPIM},
    structure::types::{
        DataPIM, InstanceHandlePIM, LocatorPIM,
        GUIDPIM,
    },
};

use super::types::DurationPIM;

pub trait RTPSStatelessReader<
    PSM: InstanceHandlePIM
        + DataPIM
        + LocatorPIM
        + DurationPIM
        + GUIDPIM
        + ParameterIdPIM
        + ParameterListSubmessageElementPIM,
>: RTPSReader<PSM>
{
}
