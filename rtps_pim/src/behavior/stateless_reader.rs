use crate::{
    behavior::RTPSReader, messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::types::InstanceHandlePIM,
};

use super::types::DurationPIM;

pub trait RTPSStatelessReader<
    PSM: InstanceHandlePIM + DurationPIM + ParameterListSubmessageElementPIM,
>: RTPSReader<PSM>
{
}
