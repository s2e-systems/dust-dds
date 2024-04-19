use super::{
    behavior_types::InstanceHandle,
    messages::{
        self,
        submessage_elements::{Data, ParameterList},
    },
    types::{ChangeKind, Guid},
};
use crate::subscription::sample_info::SampleStateKind;

#[derive(Debug)]
pub struct RtpsReaderCacheChange {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub data: Data,
    pub inline_qos: ParameterList,
    pub source_timestamp: Option<messages::types::Time>,
    pub sample_state: SampleStateKind,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
    pub reception_timestamp: messages::types::Time,
}
