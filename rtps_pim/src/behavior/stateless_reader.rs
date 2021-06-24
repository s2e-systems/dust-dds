use crate::{
    behavior::RTPSReader,
    messages::{submessage_elements::ParameterListSubmessageElementPIM, types::ParameterIdPIM},
    structure::types::{
        DataPIM, InstanceHandlePIM,
    },
};

use super::types::DurationPIM;

pub trait RTPSStatelessReader<
    PSM: InstanceHandlePIM
        + DataPIM
        + DurationPIM
        + ParameterIdPIM
        + ParameterListSubmessageElementPIM,
>: RTPSReader<PSM>
{
}
