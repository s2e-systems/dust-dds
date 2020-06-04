use crate::cache::{CacheChange, HistoryCache};
use crate::types::{EntityId, LocatorList, SequenceNumber, GUID};
use std::collections::{HashMap, HashSet};
use crate::behavior_types::{ChangeForReaderStatusKind, ChangeFromWriterStatusKind, };


#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ChangeFromWriter {
    pub status: ChangeFromWriterStatusKind,
    pub is_relevant: bool,
}

impl Default for ChangeFromWriter {
    fn default() -> Self {
        ChangeFromWriter {
            status: ChangeFromWriterStatusKind::Unknown,
            is_relevant: false,
        }
    }
}

impl ChangeFromWriter {
    pub fn new(status: ChangeFromWriterStatusKind, is_relevant: bool) -> Self {
        ChangeFromWriter {
            status,
            is_relevant,
        }
    }

    pub fn is_status(&self, status: ChangeFromWriterStatusKind) -> bool {
        if self.status == status {
            return true;
        }
        return false;
    }

    pub fn is_relevant(&self) -> bool {
        self.is_relevant
    }
}

pub struct WriterProxy {
    remote_writer_guid: GUID,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    data_max_size_serialized: Option<i32>,
    remote_group_entity_id: EntityId,
    changes_from_writer: HashMap<CacheChange, ChangeFromWriter>,
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        data_max_size_serialized: Option<i32>,
        remote_group_entity_id: EntityId,
    ) -> Self {
        let changes_from_writer = HashMap::new();
        WriterProxy {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
            remote_group_entity_id,
            changes_from_writer,
        }
    }

    pub fn remote_writer_guid(&self) -> GUID {
        self.remote_writer_guid
    }

    pub fn available_changes_max(&self, history_cache: &HistoryCache) -> Option<SequenceNumber> {
        history_cache
            .get_changes()
            .iter()
            .filter(|&cc| cc.get_writer_guid() == &self.remote_writer_guid)
            .filter(|&cc| {
                self.is_change_status(cc, ChangeFromWriterStatusKind::Received)
                    || self.is_change_status(cc, ChangeFromWriterStatusKind::Lost)
            })
            .max()
            .map(|cc| *cc.get_sequence_number())
    }

    pub fn irrelevant_change_set(
        &mut self,
        history_cache: &HistoryCache,
        a_seq_num: SequenceNumber,
    ) {
        history_cache
            .get_changes()
            .iter()
            .filter(|cc| cc.get_writer_guid() == &self.remote_writer_guid)
            .find(|cc| cc.get_sequence_number() == &a_seq_num)
            .map(|cc| {
                self.changes_from_writer.insert(
                    cc.clone(),
                    ChangeFromWriter::new(ChangeFromWriterStatusKind::Received, false),
                )
            });
    }

    pub fn lost_changes_update(
        &mut self,
        history_cache: &HistoryCache,
        _first_available_seq_num: &SequenceNumber,
    ) {
        let history_cache_changes_lock = history_cache.get_changes();

        let mut lost_change_set = history_cache_changes_lock
            .iter()
            .filter(|cc| cc.get_writer_guid() == &self.remote_writer_guid)
            .filter(|cc| {
                self.is_change_status(cc, ChangeFromWriterStatusKind::Unknown)
                    || self.is_change_status(cc, ChangeFromWriterStatusKind::Missing)
            })
            .map(|cc| cc.clone_without_data())
            .collect::<Vec<CacheChange>>();

        for lost_change in lost_change_set.drain(..) {
            self.changes_from_writer.insert(
                lost_change,
                ChangeFromWriter::new(ChangeFromWriterStatusKind::Lost, true),
            );
        }
    }

    pub fn missing_changes(&self, history_cache: &HistoryCache) -> HashSet<SequenceNumber> //TODO: Check this return type (should be SequenceNumberSet)
    {
        history_cache
            .get_changes()
            .iter()
            .filter(|cc| cc.get_writer_guid() == &self.remote_writer_guid)
            .filter(|cc| self.is_change_status(cc, ChangeFromWriterStatusKind::Missing))
            .map(|cc| *cc.get_sequence_number())
            .collect()
    }

    pub fn missing_changes_update(
        &mut self,
        history_cache: &HistoryCache,
        last_available_seq_num: SequenceNumber,
    ) {
        let history_cache_changes_lock = history_cache.get_changes();

        let mut missing_change_set = history_cache_changes_lock
            .iter()
            .filter(|cc| cc.get_writer_guid() == &self.remote_writer_guid)
            .filter(|cc| self.is_change_status(cc, ChangeFromWriterStatusKind::Unknown))
            .filter(|cc| cc.get_sequence_number() <= &last_available_seq_num)
            .map(|cc| cc.clone_without_data())
            .collect::<Vec<CacheChange>>();

        for missing_change in missing_change_set.drain(..) {
            self.changes_from_writer.insert(
                missing_change,
                ChangeFromWriter::new(ChangeFromWriterStatusKind::Missing, true),
            );
        }
    }

    pub fn received_change_set(&mut self, history_cache: &HistoryCache, a_seq_num: SequenceNumber) {
        let cache_changes_lock = history_cache.get_changes();
        let _reader_cache_change = cache_changes_lock
            .iter()
            .filter(|cc| cc.get_writer_guid() == &self.remote_writer_guid)
            .find(|cc| cc.get_sequence_number() == &a_seq_num)
            .map(|cc| {
                self.changes_from_writer.insert(
                    cc.clone(),
                    ChangeFromWriter::new(ChangeFromWriterStatusKind::Received, true),
                )
            });
    }

    fn is_change_status(&self, cc: &CacheChange, status: ChangeFromWriterStatusKind) -> bool {
        let cfw = self.changes_from_writer.get(cc);
        match cfw {
            Some(cfw) => cfw.is_status(status),
            None => ChangeFromWriter::default().is_status(status),
        }
    }

    fn is_change_relevant(&self, cc: &CacheChange) -> bool {
        let cfw = self.changes_from_writer.get(cc);
        match cfw {
            Some(cfw) => cfw.is_relevant(),
            None => false,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ChangeForReader {
    pub status: ChangeForReaderStatusKind,
    pub is_relevant: bool,
}

impl Default for ChangeForReader {
    fn default() -> Self {
        ChangeForReader {
            status: ChangeForReaderStatusKind::Unsent,
            is_relevant: false,
        }
    }
}

impl ChangeForReader {
    pub fn new(status: ChangeForReaderStatusKind, is_relevant: bool) -> Self {
        ChangeForReader {
            status,
            is_relevant,
        }
    }
}

pub struct ReaderProxy {
    remote_reader_guid: GUID,
    remote_group_entity_id: EntityId,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    expects_inline_qos: bool,
    is_active: bool,
    highest_sequence_number_sent: SequenceNumber,
    highest_sequence_number_acked: SequenceNumber,
    sequence_numbers_requested: HashSet<SequenceNumber>,
}

impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        remote_group_entity_id: EntityId,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self {
        //IF ( DDS_FILTER(this, change) ) THEN change.is_relevant := FALSE;
        //ELSE change.is_relevant := TRUE;
        ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
            highest_sequence_number_sent: 0,
            highest_sequence_number_acked: 0,
            sequence_numbers_requested: HashSet::new(),
        }
    }

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_sequence_number_acked = committed_seq_num;
    }

    pub fn next_requested_change<'a>(
        &self,
        history_cache: &'a HistoryCache,
    ) -> Option<&'a CacheChange> {
        let min_requested_sequence_number = self.sequence_numbers_requested.iter().min()?;
        history_cache
            .get_changes()
            .iter()
            .find(|cc| cc.get_sequence_number() == min_requested_sequence_number)
    }

    pub fn next_unsent_change<'a>(
        &self,
        history_cache: &'a HistoryCache,
    ) -> Option<&'a CacheChange> {
        history_cache
            .get_changes()
            .iter()
            .filter(|cc| cc.get_sequence_number() > &self.highest_sequence_number_sent)
            .min()
    }

    pub fn unsent_changes<'a>(&self, history_cache: &'a HistoryCache) -> HashSet<&'a CacheChange> {
        history_cache
            .get_changes()
            .iter()
            .filter(|cc| cc.get_sequence_number() > &self.highest_sequence_number_sent)
            .collect()
    }

    pub fn requested_changes<'a>(
        &self,
        history_cache: &'a HistoryCache,
    ) -> HashSet<&'a CacheChange> {
        let mut requested_changes = HashSet::new();
        for rsn in self.sequence_numbers_requested.iter() {
            if let Some(cc) = history_cache
                .get_changes()
                .iter()
                .find(|cc| cc.get_sequence_number() == rsn)
            {
                requested_changes.insert(cc);
            }
        }
        requested_changes
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: HashSet<SequenceNumber>) {
        for rsn in req_seq_num_set.iter() {
            self.sequence_numbers_requested.insert(*rsn);
        }
    }

    pub fn unacked_changes<'a>(&self, history_cache: &'a HistoryCache) -> HashSet<&'a CacheChange> {
        history_cache
            .get_changes()
            .iter()
            .filter(|cc| cc.get_sequence_number() > &self.highest_sequence_number_acked)
            .collect()
    }

    pub fn is_acked(&self, sequence_number : SequenceNumber) -> bool
    {
        sequence_number <= self.highest_sequence_number_acked
    }

    pub fn remote_reader_guid(&self) -> &GUID{
        &self.remote_reader_guid
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::{ChangeKind, EntityId, ENTITYID_UNKNOWN};

    #[test]
    fn test_writer_proxy_available_changes_max() {
        let mut hc = HistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );

        let mut writer_proxy =
            WriterProxy::new(writer_guid.clone(), vec![], vec![], None, ENTITYID_UNKNOWN);

        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = CacheChange::new(
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
        let other_cc = CacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = CacheChange::new(
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

        let cc = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Received),
            false
        );

        writer_proxy.received_change_set(&hc, sequence_number);

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Received),
            true
        );

        let result = writer_proxy.available_changes_max(&hc);
        assert_eq!(result, Some(sequence_number));

        let cc = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );
        hc.add_change(cc);

        writer_proxy.received_change_set(&hc, sequence_number + 1);
        let result = writer_proxy.available_changes_max(&hc);
        assert_eq!(result, Some(sequence_number + 1));
    }

    #[test]
    fn test_writer_proxy_irrelevant_change_set() {
        let mut hc = HistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = CacheChange::new(
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
        let other_cc = CacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = CacheChange::new(
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
        let mut writer_proxy = WriterProxy::new(
            writer_guid,
            Vec::new(),
            Vec::new(),
            None,
            remote_group_entity_id,
        );

        let cc = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Received),
            false
        );
        writer_proxy.irrelevant_change_set(&hc, sequence_number);

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Received),
            true
        );
        assert_eq!(writer_proxy.is_change_relevant(&cc), false);
    }

    #[test]
    fn test_writer_proxy_lost_changes_update() {
        let mut hc = HistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );
        let cc3 = CacheChange::new(
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
        let other_cc = CacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = CacheChange::new(
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
        let mut writer_proxy = WriterProxy::new(
            writer_guid,
            Vec::new(),
            Vec::new(),
            None,
            remote_group_entity_id,
        );

        let cc = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Lost),
            false
        );
        assert_eq!(
            writer_proxy.is_change_status(&cc2, ChangeFromWriterStatusKind::Lost),
            false
        );

        writer_proxy.lost_changes_update(&hc, &(sequence_number + 2));

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Lost),
            true
        );
        assert_eq!(
            writer_proxy.is_change_status(&cc2, ChangeFromWriterStatusKind::Lost),
            true
        );
    }

    #[test]
    fn test_writer_proxy_missing_changes() {
        let mut hc = HistoryCache::new();

        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );
        let cc3 = CacheChange::new(
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
        let other_cc = CacheChange::new(
            ChangeKind::Alive,
            other_writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let yet_other_cc = CacheChange::new(
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
        let mut writer_proxy = WriterProxy::new(
            writer_guid,
            Vec::new(),
            Vec::new(),
            None,
            remote_group_entity_id,
        );

        let cc = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Missing),
            false
        );

        assert_eq!(
            writer_proxy.is_change_status(&cc2, ChangeFromWriterStatusKind::Missing),
            false
        );
        writer_proxy.missing_changes_update(&hc, sequence_number + 1);

        assert_eq!(
            writer_proxy.is_change_status(&cc, ChangeFromWriterStatusKind::Missing),
            true
        );

        assert_eq!(
            writer_proxy.is_change_status(&cc2, ChangeFromWriterStatusKind::Missing),
            true
        );

        let missing_changes_sequence_set = writer_proxy.missing_changes(&hc);
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

    #[test]
    fn test_reader_proxy_acked_changes() {
        let mut hc = HistoryCache::new();
        let writer_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 1),
        );
        let sequence_number = 2;
        let instance_handle = [1; 16];
        let cc1 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number,
            None,
            None,
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 1,
            None,
            None,
        );
        let cc3 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            instance_handle,
            sequence_number + 2,
            None,
            None,
        );
        
        hc.add_change(cc1);
        hc.add_change(cc2);
        hc.add_change(cc3);

        let remote_reader_guid = GUID::new(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            EntityId::new([0, 1, 0], 2),
        );

        let remote_group_entity_id = EntityId::new([0, 1, 0], 2);
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            Vec::new(),
            Vec::new(),
            false, /*expects_inline_qos*/
            true,  /*is_active*/
        );

        assert_eq!(reader_proxy.unacked_changes(&hc).len(), 3);
        assert_eq!(reader_proxy.unacked_changes(&hc).iter().min().unwrap().get_sequence_number(), &sequence_number);

        reader_proxy.acked_changes_set(sequence_number + 1);
        assert_eq!(reader_proxy.unacked_changes(&hc).len(), 1);
        assert_eq!(reader_proxy.unacked_changes(&hc).iter().min().unwrap().get_sequence_number(), &(sequence_number + 2));
        
    }
}
