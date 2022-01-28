use rust_rtps_pim::{
    behavior::reader::writer_proxy::{
        RtpsWriterProxy, RtpsWriterProxyAttributes, RtpsWriterProxyConstructor,
        RtpsWriterProxyOperations,
    },
    structure::types::{EntityId, Guid, Locator, SequenceNumber},
};

#[derive(Debug, PartialEq)]
pub struct RtpsWriterProxyImpl(RtpsWriterProxy<Vec<Locator>>);

impl RtpsWriterProxyConstructor for RtpsWriterProxyImpl {
    fn new(
        remote_writer_guid: Guid,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        data_max_size_serialized: Option<i32>,
        remote_group_entity_id: EntityId,
    ) -> Self {
        Self(RtpsWriterProxy {
            remote_writer_guid,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            data_max_size_serialized,
            remote_group_entity_id,
        })
    }
}

impl RtpsWriterProxyAttributes for RtpsWriterProxyImpl {
    fn remote_writer_guid(&self) -> &Guid {
        &self.0.remote_writer_guid
    }
}

impl RtpsWriterProxyOperations for RtpsWriterProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn available_changes_max(&self) -> &SequenceNumber {
        todo!()
    }

    fn irrelevant_change_set(&mut self, _a_seq_num: &SequenceNumber) {
        todo!()
    }

    fn lost_changes_update(&mut self, _first_available_seq_num: &SequenceNumber) {
        todo!()
    }

    fn missing_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn missing_changes_update(&mut self, _last_available_seq_num: &SequenceNumber) {
        todo!()
    }

    fn received_change_set(&mut self, _a_seq_num: &SequenceNumber) {
        todo!()
    }
}
