use rust_rtps_pim::{
    behavior::reader::writer_proxy::{RtpsWriterProxy, RtpsWriterProxyOperations},
    structure::types::SequenceNumber,
};

pub struct RtpsWriterProxyImpl;

impl RtpsWriterProxyOperations for RtpsWriterProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn new<L>(
        _remote_writer_guid: rust_rtps_pim::structure::types::Guid,
        _remote_group_entity_id: rust_rtps_pim::structure::types::EntityId,
        _unicast_locator_list: &L,
        _multicast_locator_list: &L,
        _data_max_size_serialized: Option<i32>,
    ) -> Self
    where
        for<'a> &'a L: IntoIterator<Item = &'a rust_rtps_pim::structure::types::Locator>,
    {
        todo!()
    }

    fn available_changes_max(&self) -> &rust_rtps_pim::structure::types::SequenceNumber {
        todo!()
    }

    fn irrelevant_change_set(
        &mut self,
        _a_seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn lost_changes_update(
        &mut self,
        _first_available_seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn missing_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn missing_changes_update(
        &mut self,
        _last_available_seq_num: rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn received_change_set(&mut self, _a_seq_num: rust_rtps_pim::structure::types::SequenceNumber) {
        todo!()
    }
}