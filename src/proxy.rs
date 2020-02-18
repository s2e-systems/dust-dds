use crate::cache::{
    CacheChangeOperations, ChangeFromWriterStatusKind, HistoryCache, ReaderCacheChange,
    ReaderHistoryCache,
};
use crate::types::{EntityId, LocatorList, SequenceNumber, SequenceNumberSet, GUID};
use std::collections::HashSet;

pub struct WriterProxy<'a> {
    remote_writer_guid: GUID,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    data_max_size_serialized: Option<i32>,
    changes_from_writer: &'a ReaderHistoryCache,
    remote_group_entity_id: EntityId,
}

impl WriterProxy<'_> {
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        data_max_size_serialized: Option<i32>,
        changes_from_writer: &ReaderHistoryCache,
        remote_group_entity_id: EntityId,
    ) -> WriterProxy {
        WriterProxy {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
            changes_from_writer,
            remote_group_entity_id,
        }
    }

    pub fn remote_writer_guid(&self) -> GUID {
        self.remote_writer_guid
    }

    pub fn available_changes_max(&self) -> Option<SequenceNumber> {
        let cache_changes_lock = self.changes_from_writer.get_changes();

        let cache_change = cache_changes_lock
            .iter()
            .filter(|&rcc| (rcc.1.cache_change().writer_guid == self.remote_writer_guid))
            .filter(|&rcc| {
                (rcc.1.is_status(ChangeFromWriterStatusKind::Received)
                    || rcc.1.is_status(ChangeFromWriterStatusKind::Lost))
            })
            .max();

        match cache_change {
            None => None,
            Some(a) => Some(a.1.cache_change().sequence_number),
        }
    }

    pub fn irrelevant_change_set(&self, a_seq_num: SequenceNumber) {
        let mut cache_changes_lock = self.changes_from_writer.get_changes();
        let reader_cache_change = cache_changes_lock
            .iter_mut()
            .filter(|rcc| rcc.1.cache_change().writer_guid == self.remote_writer_guid)
            .find(|rcc| rcc.1.cache_change().sequence_number == a_seq_num)
            .unwrap();

        reader_cache_change.1.change_from_writer_mut().status =
            ChangeFromWriterStatusKind::Received;
        reader_cache_change.1.change_from_writer_mut().is_relevant = false;
    }

    pub fn lost_changes_update(&self, first_available_seq_num: SequenceNumber) {
        let mut cache_changes_lock = self.changes_from_writer.get_changes();
        for reader_cache_change in cache_changes_lock
            .iter_mut()
            .filter(|rcc| rcc.1.cache_change().writer_guid == self.remote_writer_guid)
            .filter(|rcc| {
                (rcc.1.is_status(ChangeFromWriterStatusKind::Unknown)
                    || rcc.1.is_status(ChangeFromWriterStatusKind::Missing))
            })
            .filter(|rcc| rcc.1.cache_change().sequence_number < first_available_seq_num)
        {
            reader_cache_change.1.change_from_writer_mut().status =
                ChangeFromWriterStatusKind::Lost;
        }
    }

    pub fn missing_changes(&self) -> HashSet<SequenceNumber> //TODO: Check this return type (should be SequenceNumberSet)
    {
        let mut missing_sequence_number = HashSet::new();

        let cache_changes_lock = self.changes_from_writer.get_changes();
        for reader_cache_change in cache_changes_lock
            .iter()
            .filter(|rcc| rcc.1.cache_change().writer_guid == self.remote_writer_guid)
            .filter(|rcc| rcc.1.is_status(ChangeFromWriterStatusKind::Missing))
        {
            missing_sequence_number.insert(reader_cache_change.1.cache_change().sequence_number);
        }

        missing_sequence_number
    }

    pub fn missing_changes_update(&self, last_available_seq_num: SequenceNumber) {
        let mut cache_changes_lock = self.changes_from_writer.get_changes();
        for reader_cache_change in cache_changes_lock
            .iter_mut()
            .filter(|rcc| rcc.1.cache_change().writer_guid == self.remote_writer_guid)
            .filter(|rcc| rcc.1.is_status(ChangeFromWriterStatusKind::Unknown))
            .filter(|rcc| rcc.1.cache_change().sequence_number <= last_available_seq_num)
        {
            reader_cache_change.1.change_from_writer_mut().status =
                ChangeFromWriterStatusKind::Missing;
        }
    }

    pub fn received_change_set(&self, a_seq_num: SequenceNumber) {
        let mut cache_changes_lock = self.changes_from_writer.get_changes();
        let reader_cache_change = cache_changes_lock
            .iter_mut()
            .filter(|rcc| rcc.1.cache_change().writer_guid == self.remote_writer_guid)
            .find(|rcc| rcc.1.cache_change().sequence_number == a_seq_num)
            .unwrap();

        reader_cache_change.1.change_from_writer_mut().status =
            ChangeFromWriterStatusKind::Received;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::{ChangeKind, EntityId, InstanceHandle};

    #[test]
    fn test_writer_proxy_available_changes_max() {
        let hc = ReaderHistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let other_writer_guid = GUID::new(
            [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number + 2,
            None,
            None,
        );

        hc.add_change(cc);
        hc.add_change(other_cc);
        hc.add_change(yet_other_cc);

        assert_eq!(hc.get_changes().len(), 3);

        let remote_group_entity_id = EntityId::new([0, 1, 0], 2);
        let writer_proxy = WriterProxy::new(
            writer_guid,
            Vec::new(),
            Vec::new(),
            None,
            &hc,
            remote_group_entity_id,
        );

        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        assert_eq!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Received),
            false
        );
        writer_proxy.received_change_set(sequence_number);
        assert_eq!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Received),
            true
        );

        let result = writer_proxy.available_changes_max();
        assert_eq!(result, Some(sequence_number));

        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );
        hc.add_change(cc);

        writer_proxy.received_change_set(sequence_number + 1);
        let result = writer_proxy.available_changes_max();
        assert_eq!(result, Some(sequence_number + 1));
    }

    #[test]
    fn test_writer_proxy_irrelevant_change_set() {
        let hc = ReaderHistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let other_writer_guid = GUID::new(
            [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number + 2,
            None,
            None,
        );

        hc.add_change(cc);
        hc.add_change(other_cc);
        hc.add_change(yet_other_cc);

        assert_eq!(hc.get_changes().len(), 3);

        let remote_group_entity_id = EntityId::new([0, 1, 0], 2);
        let writer_proxy = WriterProxy::new(
            writer_guid,
            Vec::new(),
            Vec::new(),
            None,
            &hc,
            remote_group_entity_id,
        );

        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        assert_eq!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Received),
            false
        );
        writer_proxy.irrelevant_change_set(sequence_number);
        assert_eq!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Received),
            true
        );
        assert_eq!(
            hc.get_changes()[&cc.cache_change()]
                .change_from_writer()
                .is_relevant,
            false
        );
    }

    #[test]
    fn test_writer_proxy_lost_changes_update() {
        let hc = ReaderHistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );
        let cc3 = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 2,
            None,
            None,
        );
        let other_writer_guid = GUID::new(
            [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number + 2,
            None,
            None,
        );

        hc.add_change(cc);
        hc.add_change(cc2);
        hc.add_change(cc3);
        hc.add_change(other_cc);
        hc.add_change(yet_other_cc);

        assert_eq!(hc.get_changes().len(), 5);

        let remote_group_entity_id = EntityId::new([0, 1, 0], 2);
        let writer_proxy = WriterProxy::new(
            writer_guid,
            Vec::new(),
            Vec::new(),
            None,
            &hc,
            remote_group_entity_id,
        );

        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );

        assert_ne!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Lost),
            true
        );
        assert_ne!(
            hc.get_changes()[&cc2.cache_change()].is_status(ChangeFromWriterStatusKind::Lost),
            true
        );

        writer_proxy.lost_changes_update(sequence_number + 2);

        assert_eq!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Lost),
            true
        );
        assert_eq!(
            hc.get_changes()[&cc2.cache_change()].is_status(ChangeFromWriterStatusKind::Lost),
            true
        );
    }

    #[test]
    fn test_writer_proxy_missing_changes() {
        let hc = ReaderHistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );
        let cc3 = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 2,
            None,
            None,
        );
        let other_writer_guid = GUID::new(
            [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number + 2,
            None,
            None,
        );

        hc.add_change(cc);
        hc.add_change(cc2);
        hc.add_change(cc3);
        hc.add_change(other_cc);
        hc.add_change(yet_other_cc);

        assert_eq!(hc.get_changes().len(), 5);

        let remote_group_entity_id = EntityId::new([0, 1, 0], 2);
        let writer_proxy = WriterProxy::new(
            writer_guid,
            Vec::new(),
            Vec::new(),
            None,
            &hc,
            remote_group_entity_id,
        );

        let cc = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = ReaderCacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );

        assert_ne!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Missing),
            true
        );
        assert_ne!(
            hc.get_changes()[&cc2.cache_change()].is_status(ChangeFromWriterStatusKind::Missing),
            true
        );

        writer_proxy.missing_changes_update(sequence_number + 1);

        assert_eq!(
            hc.get_changes()[&cc.cache_change()].is_status(ChangeFromWriterStatusKind::Missing),
            true
        );
        assert_eq!(
            hc.get_changes()[&cc2.cache_change()].is_status(ChangeFromWriterStatusKind::Missing),
            true
        );

        let missing_changes_sequence_set = writer_proxy.missing_changes();
        assert_eq!(missing_changes_sequence_set.len(), 2);
        assert_eq!(
            missing_changes_sequence_set.contains(&sequence_number),
            true
        );
        assert_eq!(
            missing_changes_sequence_set.contains(&(sequence_number + 1)),
            true
        );
        assert_eq!(
            missing_changes_sequence_set.contains(&(sequence_number + 2)),
            false
        );
    }
}
