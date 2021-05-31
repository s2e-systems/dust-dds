use crate::{
    behavior::RTPSReader,
    messages::types::ParameterIdType,
    structure::types::{
        DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType, LocatorType,
        ParameterListType, SequenceNumberType,
    },
};

use super::types::DurationType;

pub trait RTPSWriterProxy<
    PSM: GuidPrefixType + EntityIdType + LocatorType + EntityIdType + GUIDType<PSM> + SequenceNumberType,
>
{
    type SequenceNumberVector: IntoIterator<Item = PSM::SequenceNumber>;

    fn remote_writer_guid(&self) -> &PSM::GUID;
    fn remote_group_entity_id(&self) -> &PSM::EntityId;
    fn unicast_locator_list(&self) -> &[PSM::Locator];
    fn multicast_locator_list(&self) -> &[PSM::Locator];
    fn data_max_size_serialized(&self) -> i32;

    fn available_changes_max(&self) -> &PSM::SequenceNumber;
    fn irrelevant_change_set(&mut self, a_seq_num: &PSM::SequenceNumber);
    fn lost_changes_update(&mut self, first_available_seq_num: &PSM::SequenceNumber);
    fn missing_changes(&self) -> Self::SequenceNumberVector;
    fn missing_changes_update(&mut self, last_available_seq_num: PSM::SequenceNumber);
    fn received_change_set(&mut self, a_seq_num: PSM::SequenceNumber);
}

pub trait RTPSStatefulReader<
    PSM: InstanceHandleType
        + GuidPrefixType
        + DataType
        + EntityIdType
        + SequenceNumberType
        + LocatorType
        + DurationType
        + GUIDType<PSM>
        + ParameterIdType
        + ParameterListType<PSM>,
>: RTPSReader<PSM>
{
    type WriterProxyType;

    fn matched_writers(&self) -> &[Self::WriterProxyType];
    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &PSM::GUID);
    fn matched_writer_lookup(&self, a_writer_guid: PSM::GUID) -> Option<&Self::WriterProxyType>;
}
