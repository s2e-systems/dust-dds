use rust_rtps_pim::{
    behavior::writer::reader_proxy::{RtpsReaderProxy, RtpsReaderProxyOperations},
    structure::types::{EntityId, Locator, SequenceNumber, Guid},
};

pub struct RtpsReaderProxyImpl {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    is_active: bool,
    _last_sent_sequence_number: SequenceNumber,
}

impl RtpsReaderProxy for RtpsReaderProxyImpl {
    fn remote_reader_guid(&self) -> &Guid {
        &self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> &EntityId {
        &self.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl RtpsReaderProxyOperations for RtpsReaderProxyImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn new<L>(
        remote_reader_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: L,
        multicast_locator_list: L,
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self
    where
        L: IntoIterator<Item = Locator>,
    {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.into_iter().collect(),
            multicast_locator_list: multicast_locator_list.into_iter().collect(),
            expects_inline_qos,
            is_active,
            _last_sent_sequence_number: 0.into(),
        }
    }

    fn acked_changes_set(&mut self, _committed_seq_num: SequenceNumber) {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        todo!()
    }

    fn next_unsent_change(&mut self) -> Option<SequenceNumber> {
        todo!()
    }

    fn unsent_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes_set(&mut self, _req_seq_num_set: Self::SequenceNumberVector) {
        todo!()
    }

    fn unacked_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }
}
