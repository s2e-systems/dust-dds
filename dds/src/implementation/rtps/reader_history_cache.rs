use super::{
    behavior_types::InstanceHandle,
    messages::submessage_elements::{Data, ParameterList},
    types::{ChangeKind, Guid},
};

#[derive(Debug)]
pub struct RtpsCacheChange {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub data_value: Data,
    pub inline_qos: ParameterList,
}
