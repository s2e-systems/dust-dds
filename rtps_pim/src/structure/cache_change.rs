use crate::{
    messages::{
        self,
        submessages::submessage_elements::{self, Parameter},
    },
    types,
};

pub struct RTPSCacheChange<
    GuidPrefix: types::GuidPrefix,
    EntityId: types::EntityId,
    InstanceHandle: types::InstanceHandle,
    SequenceNumber: types::SequenceNumber,
    Data,
    // ParameterId: messages::types::ParameterId,
    // ParameterValue: AsRef<[u8]>,
    // ParameterList: IntoIterator<Item = Parameter<ParameterId, ParameterValue>>,
> {
    pub kind: types::ChangeKind,
    pub writer_guid: types::GUID<GuidPrefix, EntityId>,
    pub instance_handle: InstanceHandle,
    pub sequence_number: SequenceNumber,
    pub data_value: Data,
    // pub inline_qos: submessage_elements::ParameterList<ParameterId, ParameterValue, ParameterList>,
}
