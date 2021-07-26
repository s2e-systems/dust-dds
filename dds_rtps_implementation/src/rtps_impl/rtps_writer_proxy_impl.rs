use rust_rtps_pim::{
    behavior::reader::writer_proxy::{RTPSWriterProxy, RTPSWriterProxyOperations},
    structure::types::SequenceNumber,
};

pub struct RTPSWriterProxyImpl;

impl RTPSWriterProxyOperations for RTPSWriterProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn new<L>(
        remote_writer_guid: rust_rtps_pim::structure::types::GUID,
        remote_group_entity_id: rust_rtps_pim::structure::types::EntityId,
        unicast_locator_list: L,
        multicast_locator_list: L,
        data_max_size_serialized: Option<i32>,
    ) -> Self
    where
        L: IntoIterator<Item = rust_rtps_pim::structure::types::Locator>,
    {
        todo!()
    }

    fn available_changes_max(&self) -> &rust_rtps_pim::structure::types::SequenceNumber {
        todo!()
    }

    fn irrelevant_change_set(
        &mut self,
        a_seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn lost_changes_update(
        &mut self,
        first_available_seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn missing_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn missing_changes_update(
        &mut self,
        last_available_seq_num: rust_rtps_pim::structure::types::SequenceNumber,
    ) {
        todo!()
    }

    fn received_change_set(&mut self, a_seq_num: rust_rtps_pim::structure::types::SequenceNumber) {
        todo!()
    }
}

impl RTPSWriterProxy for RTPSWriterProxyImpl {
    fn remote_writer_guid(&self) -> &rust_rtps_pim::structure::types::GUID {
        todo!()
    }

    fn unicast_locator_list(&self) -> &[rust_rtps_pim::structure::types::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[rust_rtps_pim::structure::types::Locator] {
        todo!()
    }

    fn data_max_size_serialized(&self) -> &Option<i32> {
        todo!()
    }

    fn remote_group_entity_id(&self) -> &rust_rtps_pim::structure::types::EntityId {
        todo!()
    }
}
