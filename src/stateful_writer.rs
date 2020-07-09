use std::collections::{BTreeSet, HashMap};
use std::time::{Instant};
use std::convert::TryInto;

use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior::types::Duration;
use crate::behavior::StatefulWriterBehavior;
use crate::messages::types::Count;
use crate::cache::{CacheChange, HistoryCache, };
use crate::messages::RtpsMessage;
use crate::messages::submessage_elements::ParameterList;

pub struct ReaderProxy {
    remote_reader_guid: GUID,
    // remoteGroupEntityId: EntityId_t,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // changes_for_reader: CacheChange[*], 
    expects_inline_qos: bool,
    is_active: bool,

    // Additional fields:
    highest_sequence_number_sent: SequenceNumber,
    highest_sequence_number_acknowledged: SequenceNumber,
    sequence_numbers_requested: BTreeSet<SequenceNumber>,
    heartbeat_count: Count,

    time_last_sent_data: Instant,
    time_nack_received: Instant,
    highest_nack_count_received: Count,
}


impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
        is_active: bool) -> Self {
            ReaderProxy {
                remote_reader_guid,
                unicast_locator_list,
                multicast_locator_list,
                expects_inline_qos,
                is_active,
                highest_sequence_number_sent: 0,
                highest_sequence_number_acknowledged: 0,
                sequence_numbers_requested: BTreeSet::new(),
                heartbeat_count: 0,
                time_last_sent_data: Instant::now(),
                time_nack_received: Instant::now(),
                highest_nack_count_received: 0,
        }
    }

    pub fn remote_reader_guid(&self) -> &GUID {
        &self.remote_reader_guid
    }

    pub fn heartbeat_count(&self) -> &Count {
        &self.heartbeat_count
    }

    pub fn next_unsent_change(&mut self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = self.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            self.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            self.highest_sequence_number_sent + 1..=last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    pub fn acked_changes(&self) -> SequenceNumber {
        self.highest_sequence_number_acknowledged
    }

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_sequence_number_acknowledged = committed_seq_num;
    }

    pub fn unacked_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unacked_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            self.highest_sequence_number_acknowledged + 1..=last_change_sequence_number
        {
            unacked_changes_set.insert(unsent_sequence_number);
        }

        unacked_changes_set
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: BTreeSet<SequenceNumber>) {
        let mut new_set = req_seq_num_set;
        self.sequence_numbers_requested.append(&mut new_set);
    }

    pub fn requested_changes(&self) -> BTreeSet<SequenceNumber> {
        self.sequence_numbers_requested.clone()
    }

    pub fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        let next_requested_change = *self.sequence_numbers_requested.iter().next()?;

        self.sequence_numbers_requested.remove(&next_requested_change);

        Some(next_requested_change)
    }

    pub fn nack_received(&self) -> &Count {
        &self.highest_nack_count_received
    }

    pub fn nack_received_set(&mut self, highest_nack_count_received: Count) {
        self.highest_nack_count_received = highest_nack_count_received;
    }

    pub fn time_last_sent_data_reset(&mut self) {
        self.time_last_sent_data = Instant::now();
    }

    pub fn time_nack_received_reset(&mut self) {
        self.time_nack_received = Instant::now();
    }

    pub fn duration_since_last_sent_data(&self) -> Duration {
        self.time_last_sent_data.elapsed().try_into().unwrap()
    }

    pub fn duration_since_nack_received(&self) -> Duration {
        self.time_nack_received.elapsed().try_into().unwrap()
    }

    pub fn increment_heartbeat_count(&mut self) {
        self.heartbeat_count += 1;
    }
}

pub struct StatefulWriter {
    /// Entity base class (contains the GUID)
    guid: GUID,
    // entity: Entity,

    // Endpoint base class:
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    unicast_locator_list: Vec<Locator>,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: Vec<Locator>,

    //Writer class:
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    matched_readers: HashMap<GUID, ReaderProxy>,
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,) -> Self {
        StatefulWriter {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0,
            writer_cache: HistoryCache::new(),
            data_max_sized_serialized: None,
            matched_readers: HashMap::new(),
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        CacheChange::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number,
            data,
            inline_qos,
        )
    }

    pub fn writer_cache(&mut self) -> &mut HistoryCache {
        &mut self.writer_cache
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        self.matched_readers.insert(a_reader_proxy.remote_reader_guid, a_reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, a_reader_proxy: &ReaderProxy) {
        self.matched_readers.remove(&a_reader_proxy.remote_reader_guid);
    }
    
    pub fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&ReaderProxy> {
        self.matched_readers.get(a_reader_guid)
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }

    pub fn run(&mut self, a_reader_guid: &GUID, received_message: Option<&RtpsMessage>) -> Option<RtpsMessage> {
        let reader_proxy = self.matched_readers.get_mut(a_reader_guid).unwrap();

        let submessages = match self.reliability_level {
            ReliabilityKind::BestEffort => StatefulWriterBehavior::run_best_effort(reader_proxy, &self.guid, &self.writer_cache, self.last_change_sequence_number),
            ReliabilityKind::Reliable => StatefulWriterBehavior::run_reliable(reader_proxy, &self.guid, &self.writer_cache, self.last_change_sequence_number, self.heartbeat_period, self.nack_response_delay, received_message),
        };

        match submessages {
            Some(submessages) => Some(RtpsMessage::new(*self.guid.prefix(), submessages)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, };
    use crate::behavior::types::constants::DURATION_ZERO;
    use crate::types::GuidPrefix;

    #[test]
    fn stateful_writer_new_change() {
        let mut writer = StatefulWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0; 16])],
            vec![],
            false,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
        );

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]),
            None,
            [1; 16],
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None, 
            None,
            [1; 16],
        );

        assert_eq!(cache_change_seq1.sequence_number(), &1);
        assert_eq!(cache_change_seq1.change_kind(), &ChangeKind::Alive);
        assert!(cache_change_seq1.inline_qos().is_none());
        assert_eq!(cache_change_seq1.instance_handle(), &[1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), &2);
        assert_eq!(
            cache_change_seq2.change_kind(),
            &ChangeKind::NotAliveUnregistered
        );
        assert!(cache_change_seq2.inline_qos().is_none());
        assert_eq!(cache_change_seq2.instance_handle(), &[1; 16]);
    }

    #[test]
    fn reader_proxy_unsent_changes_operations() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        // Check that a reader proxy that has no changes marked as sent doesn't reports no changes
        let no_change_in_writer_sequence_number = 0;
        assert_eq!(reader_proxy.next_unsent_change(no_change_in_writer_sequence_number), None);
        assert!(reader_proxy.unsent_changes(no_change_in_writer_sequence_number).is_empty());

        // Check the behaviour for a reader proxy starting with no changes sent and two changes in writer
        let two_changes_in_writer_sequence_number = 2;
        assert_eq!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).len(), 2);
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&1));
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&2));

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), Some(1));
        assert_eq!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).len(), 1);
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&2));

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), Some(2));
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).is_empty());

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), None);
    }

    #[test]
    fn reader_proxy_requested_changes_operations() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        // Check that a reader proxy that has no changes marked as sent doesn't reports no changes
        assert!(reader_proxy.requested_changes().is_empty());
        assert_eq!(reader_proxy.next_requested_change(), None);

        // Insert some requested changes
        let mut requested_changes = BTreeSet::new();
        requested_changes.insert(2);
        requested_changes.insert(3);
        requested_changes.insert(6);
        reader_proxy.requested_changes_set(requested_changes);

        // Verify that the changes were correctly inserted and are removed in the correct order
        assert_eq!(reader_proxy.requested_changes().len(), 3);
        assert!(reader_proxy.requested_changes().contains(&2));
        assert!(reader_proxy.requested_changes().contains(&3));
        assert!(reader_proxy.requested_changes().contains(&6));

        assert_eq!(reader_proxy.next_requested_change(), Some(2));
        assert_eq!(reader_proxy.next_requested_change(), Some(3));
        assert_eq!(reader_proxy.requested_changes().len(), 1);
        assert!(reader_proxy.requested_changes().contains(&6));
        assert_eq!(reader_proxy.next_requested_change(), Some(6));
        assert_eq!(reader_proxy.next_requested_change(), None);


        // Verify that if requested changes are inserted when there are already requested changes
        // that the sets are not replaced
        let mut requested_changes_1 = BTreeSet::new();
        requested_changes_1.insert(2);
        requested_changes_1.insert(3);
        reader_proxy.requested_changes_set(requested_changes_1);

        let mut requested_changes_2 = BTreeSet::new();
        requested_changes_2.insert(2); // Repeated number
        requested_changes_2.insert(7);
        requested_changes_2.insert(9);
        reader_proxy.requested_changes_set(requested_changes_2);
        
        assert_eq!(reader_proxy.requested_changes().len(), 4);
        assert!(reader_proxy.requested_changes().contains(&2));
        assert!(reader_proxy.requested_changes().contains(&3));
        assert!(reader_proxy.requested_changes().contains(&7));
        assert!(reader_proxy.requested_changes().contains(&9));
    }

    #[test]
    fn reader_proxy_unacked_changes_operations() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let no_change_in_writer = 0;
        assert!(reader_proxy.unacked_changes(no_change_in_writer).is_empty());

        let two_changes_in_writer = 2;
        assert_eq!(reader_proxy.unacked_changes(two_changes_in_writer).len(), 2);
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&1));
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&2));

        reader_proxy.acked_changes_set(1);
        assert_eq!(reader_proxy.unacked_changes(two_changes_in_writer).len(), 1);
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&2));
    }
    
}