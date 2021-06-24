use crate::{
    behavior::RTPSReader,
    messages::{submessage_elements::ParameterListSubmessageElementPIM},
    structure::types::{
        DataPIM, InstanceHandlePIM,
    },
};

use super::types::DurationPIM;

pub trait RTPSStatelessReader<
    PSM: InstanceHandlePIM
        + DataPIM
        + DurationPIM
        + ParameterListSubmessageElementPIM,
>: RTPSReader<PSM>
{
}
