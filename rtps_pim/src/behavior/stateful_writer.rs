use crate::{
    behavior::{types::DurationType, RTPSWriter},
    messages::types::ParameterIdType,
    structure::types::{
        DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType, LocatorType,
        ParameterListType, SequenceNumberType,
    },
};

pub trait RTPSReaderProxy<
    PSM: GuidPrefixType + EntityIdType + LocatorType + EntityIdType + GUIDType<PSM> + SequenceNumberType,
>
{
    type SequenceNumberVector: IntoIterator<Item = PSM::SequenceNumber>;

    fn remote_reader_guid(&self) -> &PSM::GUID;
    fn remote_group_entity_id(&self) -> &PSM::EntityId;
    fn unicast_locator_list(&self) -> &[PSM::Locator];
    fn multicast_locator_list(&self) -> &[PSM::Locator];
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;

    fn acked_changes_set(&mut self, committed_seq_num: PSM::SequenceNumber);
    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber>;
    fn next_unsent_change(&mut self) -> Option<PSM::SequenceNumber>;
    fn unsent_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes_set(&mut self, req_seq_num_set: Self::SequenceNumberVector);
    fn unacked_changes(&self) -> Self::SequenceNumberVector;
}

pub trait RTPSStatefulWriter<
    PSM: GuidPrefixType
        + EntityIdType
        + LocatorType
        + EntityIdType
        + DurationType
        + SequenceNumberType
        + DataType
        + ParameterListType<PSM>
        + GUIDType<PSM>
        + InstanceHandleType
        + ParameterIdType,
>: RTPSWriter<PSM>
{
    type ReaderProxyType: RTPSReaderProxy<PSM>;
    fn matched_readers(&self) -> &[Self::ReaderProxyType];
    fn matched_reader_add(&mut self, guid: PSM::GUID);
    fn matched_reader_remove(&mut self, reader_proxy_guid: &PSM::GUID);
    fn matched_reader_lookup(&self, a_reader_guid: &PSM::GUID) -> Option<&Self::ReaderProxyType>;
    fn is_acked_by_all(&self) -> bool;
}
