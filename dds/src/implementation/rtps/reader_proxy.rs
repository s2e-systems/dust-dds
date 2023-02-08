use crate::infrastructure::{instance::InstanceHandle, time::Time};

use super::{
    history_cache::{RtpsParameter, RtpsWriterCacheChange, WriterHistoryCache},
    messages::{
        submessage_elements::SequenceNumberSet,
        submessages::{AckNackSubmessage, GapSubmessage, NackFragSubmessage},
    },
    types::{ChangeKind, Count, EntityId, Guid, Locator, SequenceNumber},
};

/// ChangeForReaderStatusKind
/// Enumeration used to indicate the status of a ChangeForReader. It can take the values:
/// UNSENT, UNACKNOWLEDGED, REQUESTED, ACKNOWLEDGED, UNDERWAY
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}
#[derive(Debug, PartialEq, Eq)]
pub struct RtpsReaderProxy {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    changes_for_reader: Vec<RtpsChangeForReader>,
    expects_inline_qos: bool,
    is_active: bool,
    last_received_acknack_count: Count,
    last_received_nack_frag_count: Count,
}

impl RtpsReaderProxy {
    pub fn changes_for_reader_mut(&mut self) -> &mut Vec<RtpsChangeForReader> {
        &mut self.changes_for_reader
    }
}

impl RtpsReaderProxy {
    pub fn new(
        remote_reader_guid: Guid,
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
            changes_for_reader: vec![],
            expects_inline_qos,
            is_active,
            last_received_acknack_count: Count::new(0),
            last_received_nack_frag_count: Count::new(0),
        }
    }
}

impl RtpsReaderProxy {
    pub fn remote_reader_guid(&self) -> Guid {
        self.remote_reader_guid
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_slice()
    }

    pub fn changes_for_reader(&self) -> &[RtpsChangeForReader] {
        self.changes_for_reader.as_slice()
    }

    pub fn reliable_receive_acknack(&mut self, acknack_submessage: &AckNackSubmessage) {
        if acknack_submessage.count > self.last_received_acknack_count {
            self.acked_changes_set(acknack_submessage.reader_sn_state.base - 1);
            self.requested_changes_set(acknack_submessage.reader_sn_state.set.as_ref());

            self.last_received_acknack_count = acknack_submessage.count;
        }
    }

    pub fn reliable_receive_nack_frag(&mut self, nack_frag_submessage: &NackFragSubmessage) {
        if nack_frag_submessage.count > self.last_received_nack_frag_count {
            self.requested_changes_set(&[nack_frag_submessage.writer_sn]);
            self.last_received_nack_frag_count = nack_frag_submessage.count;
        }
    }
}

impl From<RtpsChangeForReaderCacheChange<'_>> for SequenceNumber {
    fn from(v: RtpsChangeForReaderCacheChange<'_>) -> Self {
        v.change_for_reader.sequence_number
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RtpsChangeForReader {
    status: ChangeForReaderStatusKind,
    is_relevant: bool,
    sequence_number: SequenceNumber,
}

impl RtpsChangeForReader {
    pub fn new(
        status: ChangeForReaderStatusKind,
        is_relevant: bool,
        sequence_number: SequenceNumber,
    ) -> Self {
        Self {
            status,
            is_relevant,
            sequence_number,
        }
    }

    pub fn status(&self) -> ChangeForReaderStatusKind {
        self.status
    }

    pub fn is_relevant(&self) -> bool {
        self.is_relevant
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }
}

pub struct RtpsChangeForReaderCacheChange<'a> {
    change_for_reader: RtpsChangeForReader,
    cache_change: &'a RtpsWriterCacheChange,
}

impl<'a> RtpsChangeForReaderCacheChange<'a> {
    pub fn new(
        change_for_reader: RtpsChangeForReader,
        writer_cache: &'a WriterHistoryCache,
    ) -> Self {
        let cache_change = writer_cache
            .changes()
            .iter()
            .find(|cc| cc.sequence_number() == change_for_reader.sequence_number)
            .unwrap();
        RtpsChangeForReaderCacheChange {
            change_for_reader,
            cache_change,
        }
    }

    pub fn cache_change(self) -> &'a RtpsWriterCacheChange {
        self.cache_change
    }

    pub fn status(&self) -> ChangeForReaderStatusKind {
        self.change_for_reader.status
    }

    pub fn is_relevant(&self) -> bool {
        self.change_for_reader.is_relevant
    }

    pub fn kind(&self) -> ChangeKind {
        self.cache_change.kind()
    }

    pub fn writer_guid(&self) -> Guid {
        self.cache_change.writer_guid()
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.cache_change.instance_handle()
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.cache_change.sequence_number()
    }

    pub fn data_value(&self) -> &[u8] {
        self.cache_change.data_value()
    }

    pub fn inline_qos(&self) -> &[RtpsParameter] {
        self.cache_change.inline_qos()
    }

    pub fn timestamp(&self) -> Time {
        self.cache_change.timestamp()
    }

    pub fn as_gap_message(&self, reader_id: EntityId) -> GapSubmessage {
        GapSubmessage {
            endianness_flag: true,
            reader_id,
            writer_id: self.cache_change.writer_guid().entity_id(),
            gap_start: self.cache_change.sequence_number(),
            gap_list: SequenceNumberSet {
                base: self.cache_change.sequence_number(),
                set: vec![],
            },
        }
    }
}

impl RtpsReaderProxy {
    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // "FOR_EACH change in this.changes_for_reader
        // SUCH-THAT (change.sequenceNumber <= committed_seq_num) DO
        // change.status := ACKNOWLEDGED;"
        for change in &mut self.changes_for_reader {
            if change.sequence_number <= committed_seq_num {
                change.status = ChangeForReaderStatusKind::Acknowledged;
            }
        }
    }

    pub fn next_requested_change<'a>(
        &mut self,
        writer_cache: &'a WriterHistoryCache,
    ) -> RtpsChangeForReaderCacheChange<'a> {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()}
        //  return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.requested_changes().iter().min().cloned().unwrap();

        let change = self
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)
            .unwrap();

        // Following 8.4.9.2.12 Transition T12 of Reliable Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_requested_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_requested_change() as such
        // the modification of the status is done always.
        change.status = ChangeForReaderStatusKind::Underway;

        // After ackNackSuppressionDuration = 0
        change.status = ChangeForReaderStatusKind::Unacknowledged;

        RtpsChangeForReaderCacheChange::new(change.clone(), writer_cache)
    }

    pub fn next_unsent_change<'a>(
        &mut self,
        writer_cache: &'a WriterHistoryCache,
    ) -> RtpsChangeForReaderCacheChange<'a> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.unsent_changes().iter().min().cloned().unwrap();

        let change = self
            .changes_for_reader
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)
            .unwrap();

        // Following 8.4.9.1.4 Transition T14 of BestEffort Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_unsent_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_unsent_change() as such
        // the modification of the status is done always.
        change.status = ChangeForReaderStatusKind::Underway;

        // After ackNackSuppressionDuration = 0
        change.status = ChangeForReaderStatusKind::Unacknowledged;

        RtpsChangeForReaderCacheChange::new(change.clone(), writer_cache)
    }

    pub fn unsent_changes(&self) -> Vec<SequenceNumber> {
        // "return change IN this.changes_for_reader SUCH-THAT (change.status == UNSENT);"
        self.changes_for_reader
            .iter()
            .filter_map(|cc| {
                if cc.status == ChangeForReaderStatusKind::Unsent {
                    Some(cc.sequence_number)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn requested_changes(&self) -> Vec<SequenceNumber> {
        // "return change IN this.changes_for_reader
        //      SUCH-THAT (change.status == REQUESTED);"
        let requested_changes_for_reader: Vec<_> = self
            .changes_for_reader
            .iter()
            .filter(|&change_for_reader| {
                change_for_reader.status == ChangeForReaderStatusKind::Requested
            })
            .collect();
        requested_changes_for_reader
            .iter()
            .map(|change_for_reader| change_for_reader.sequence_number)
            .collect()
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // "FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END"
        for &seq_num in req_seq_num_set {
            for change_for_reader in &mut self
                .changes_for_reader
                .iter_mut()
                .filter(|change_for_reader| change_for_reader.sequence_number == seq_num)
            {
                change_for_reader.status = ChangeForReaderStatusKind::Requested;
            }
        }
    }

    pub fn unacked_changes(&self) -> Vec<SequenceNumber> {
        //"return change IN this.changes_for_reader
        //    SUCH-THAT (change.status == UNACKNOWLEDGED);"
        self.changes_for_reader
            .iter()
            .filter_map(|cc| {
                if cc.status == ChangeForReaderStatusKind::Unacknowledged {
                    Some(cc.sequence_number)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        implementation::rtps::{
            history_cache::RtpsWriterCacheChange,
            types::{ENTITYID_UNKNOWN, GUID_UNKNOWN},
        },
        infrastructure::{instance::HANDLE_NIL, time::TIME_INVALID},
    };

    fn add_new_change_push_mode_true(
        writer_cache: &mut WriterHistoryCache,
        reader_proxy: &mut RtpsReaderProxy,
        sequence_number: SequenceNumber,
    ) {
        writer_cache.add_change(RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            sequence_number,
            TIME_INVALID,
            vec![],
            vec![],
        ));
        reader_proxy.changes_for_reader.push(RtpsChangeForReader {
            status: ChangeForReaderStatusKind::Unsent,
            is_relevant: true,
            sequence_number,
        });
    }

    fn add_new_change_push_mode_false(
        writer_cache: &mut WriterHistoryCache,
        reader_proxy: &mut RtpsReaderProxy,
        sequence_number: SequenceNumber,
    ) {
        writer_cache.add_change(RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            sequence_number,
            TIME_INVALID,
            vec![],
            vec![],
        ));
        reader_proxy.changes_for_reader.push(RtpsChangeForReader {
            status: ChangeForReaderStatusKind::Unacknowledged,
            is_relevant: true,
            sequence_number,
        })
    }

    #[test]
    fn next_requested_change() {
        let mut reader_proxy =
            RtpsReaderProxy::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);

        let mut writer_cache = WriterHistoryCache::new();
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(1),
        );
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(2),
        );
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(4),
        );
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(6),
        );

        reader_proxy.requested_changes_set(&[SequenceNumber::new(2), SequenceNumber::new(4)]);

        let result = reader_proxy.next_requested_change(&writer_cache);
        assert_eq!(
            result.change_for_reader.sequence_number,
            SequenceNumber::new(2)
        );

        let result = reader_proxy.next_requested_change(&writer_cache);
        assert_eq!(
            result.change_for_reader.sequence_number,
            SequenceNumber::new(4)
        );
    }

    #[test]
    fn unsent_changes() {
        let mut reader_proxy =
            RtpsReaderProxy::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = WriterHistoryCache::new();
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy, SequenceNumber::new(1));
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy, SequenceNumber::new(3));
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy, SequenceNumber::new(4));

        assert_eq!(
            reader_proxy.unsent_changes(),
            vec![
                SequenceNumber::new(1),
                SequenceNumber::new(3),
                SequenceNumber::new(4)
            ]
        );
    }

    #[test]
    fn next_unsent_change() {
        let mut reader_proxy =
            RtpsReaderProxy::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = WriterHistoryCache::new();
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy, SequenceNumber::new(1));
        add_new_change_push_mode_true(&mut writer_cache, &mut reader_proxy, SequenceNumber::new(2));

        let result = reader_proxy.next_unsent_change(&writer_cache);
        assert_eq!(
            result.change_for_reader.sequence_number,
            SequenceNumber::new(1)
        );

        let result = reader_proxy.next_unsent_change(&writer_cache);
        assert_eq!(
            result.change_for_reader.sequence_number,
            SequenceNumber::new(2)
        );

        // let result = std::panic::catch_unwind(|| reader_proxy.next_unsent_change());
        // assert!(result.is_err());
    }

    #[test]
    fn unacked_changes() {
        let mut reader_proxy =
            RtpsReaderProxy::new(GUID_UNKNOWN, ENTITYID_UNKNOWN, &[], &[], false, true);
        let mut writer_cache = WriterHistoryCache::new();
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(1),
        );
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(2),
        );
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(4),
        );
        add_new_change_push_mode_false(
            &mut writer_cache,
            &mut reader_proxy,
            SequenceNumber::new(6),
        );

        reader_proxy.acked_changes_set(SequenceNumber::new(2));

        assert_eq!(
            reader_proxy.unacked_changes(),
            vec![SequenceNumber::new(4), SequenceNumber::new(6)]
        );
    }
}
