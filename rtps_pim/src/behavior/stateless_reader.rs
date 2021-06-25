use crate::{
    behavior::RTPSReader, messages::submessage_elements::ParameterListSubmessageElementPIM,
};

use super::types::DurationPIM;

pub trait RTPSStatelessReader<PSM: DurationPIM + ParameterListSubmessageElementPIM>:
    RTPSReader<PSM>
{
}
