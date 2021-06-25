use crate::{
    behavior::RTPSReader,
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::types::{EntityId, InstanceHandlePIM, Locator, SequenceNumber, GUID},
};

use super::types::DurationPIM;

pub trait RTPSWriterProxy {
    type SequenceNumberVector: IntoIterator<Item = SequenceNumber>;

    fn remote_writer_guid(&self) -> &GUID;
    fn remote_group_entity_id(&self) -> &EntityId;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn data_max_size_serialized(&self) -> i32;

    fn available_changes_max(&self) -> &SequenceNumber;
    fn irrelevant_change_set(&mut self, a_seq_num: &SequenceNumber);
    fn lost_changes_update(&mut self, first_available_seq_num: &SequenceNumber);
    fn missing_changes(&self) -> Self::SequenceNumberVector;
    fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber);
    fn received_change_set(&mut self, a_seq_num: SequenceNumber);
}

pub trait RTPSStatefulReader<
    PSM: InstanceHandlePIM + DurationPIM + ParameterListSubmessageElementPIM,
>: RTPSReader<PSM>
{
    type WriterProxyType;

    fn matched_writers(&self) -> &[Self::WriterProxyType];
    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &GUID);
    fn matched_writer_lookup(&self, a_writer_guid: GUID) -> Option<&Self::WriterProxyType>;
}
