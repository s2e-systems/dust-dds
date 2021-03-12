use rust_rtps::{
    behavior::{
        stateful_writer::reader_proxy::RTPSChangeForReader, types::ChangeForReaderStatusKind,
        RTPSReaderProxy,
    },
    types::{EntityId, Locator, SequenceNumber, GUID},
};

pub struct ChangeForReader {
    change: SequenceNumber,
    status: ChangeForReaderStatusKind,
    is_relevant: bool,
}

impl RTPSChangeForReader for ChangeForReader {
    type CacheChangeRepresentation = SequenceNumber;

    fn change(&self) -> &Self::CacheChangeRepresentation {
        &self.change
    }

    fn status(&self) -> ChangeForReaderStatusKind {
        self.status
    }

    fn is_relevant(&self) -> bool {
        self.is_relevant
    }
}

pub struct ReaderProxy {
    remote_reader_guid: GUID,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    is_active: bool,

    next_unsent_change: SequenceNumber,
}

impl RTPSReaderProxy for ReaderProxy {
    type ChangeForReaderType = ChangeForReader;

    fn remote_reader_guid(&self) -> GUID {
        self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }

    fn changes_for_reader(
        &self,
    ) -> &[<Self::ChangeForReaderType as RTPSChangeForReader>::CacheChangeRepresentation] {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn new(
        remote_reader_guid: GUID,
        remote_group_entity_id: EntityId,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            expects_inline_qos,
            is_active,
            next_unsent_change: 0,
        }
    }

    fn acked_changes_set(&mut self, _committed_seq_num: SequenceNumber) {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<&Self::ChangeForReaderType> {
        todo!()
    }

    fn next_unsent_change(&mut self) -> Option<&Self::ChangeForReaderType> {
        todo!()
    }

    fn unsent_changes(&self) -> &[Self::ChangeForReaderType] {
        todo!()
    }

    fn requested_changes(&self) -> &[Self::ChangeForReaderType] {
        todo!()
    }

    fn requested_changes_set(&mut self, _req_seq_num_set: &[SequenceNumber]) {
        todo!()
    }

    fn unacked_changes(&self) -> &[Self::ChangeForReaderType] {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_getters() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
        );

        assert_eq!(reader_proxy.remote_reader_guid(), remote_reader_guid);
        assert_eq!(reader_proxy.remote_group_entity_id(), remote_group_entity_id);
        assert_eq!(reader_proxy.unicast_locator_list(), unicast_locator_list);
        assert_eq!(reader_proxy.multicast_locator_list(), multicast_locator_list);
        assert_eq!(reader_proxy.expects_inline_qos(), expects_inline_qos);
        assert_eq!(reader_proxy.is_active(), is_active);
    }
}
