use crate::{
    behavior::RTPSReader,
    messages::{submessage_elements::ParameterListSubmessageElementPIM, types::ParameterIdPIM},
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, SequenceNumberPIM,
        GUIDPIM,
    },
};

use super::types::DurationPIM;

pub trait RTPSWriterProxy<
    PSM: GuidPrefixPIM + EntityIdPIM + LocatorPIM + EntityIdPIM + GUIDPIM + SequenceNumberPIM,
>
{
    type SequenceNumberVector: IntoIterator<Item = PSM::SequenceNumberType>;

    fn remote_writer_guid(&self) -> &PSM::GUIDType;
    fn remote_group_entity_id(&self) -> &PSM::EntityIdType;
    fn unicast_locator_list(&self) -> &[PSM::LocatorType];
    fn multicast_locator_list(&self) -> &[PSM::LocatorType];
    fn data_max_size_serialized(&self) -> i32;

    fn available_changes_max(&self) -> &PSM::SequenceNumberType;
    fn irrelevant_change_set(&mut self, a_seq_num: &PSM::SequenceNumberType);
    fn lost_changes_update(&mut self, first_available_seq_num: &PSM::SequenceNumberType);
    fn missing_changes(&self) -> Self::SequenceNumberVector;
    fn missing_changes_update(&mut self, last_available_seq_num: PSM::SequenceNumberType);
    fn received_change_set(&mut self, a_seq_num: PSM::SequenceNumberType);
}

pub trait RTPSStatefulReader<
    PSM: InstanceHandlePIM
        + GuidPrefixPIM
        + DataPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + LocatorPIM
        + DurationPIM
        + GUIDPIM
        + ParameterIdPIM
        + ParameterListSubmessageElementPIM,
>: RTPSReader<PSM>
{
    type WriterProxyType;

    fn matched_writers(&self) -> &[Self::WriterProxyType];
    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &PSM::GUIDType);
    fn matched_writer_lookup(&self, a_writer_guid: PSM::GUIDType)
        -> Option<&Self::WriterProxyType>;
}
