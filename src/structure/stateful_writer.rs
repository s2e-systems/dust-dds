use std::collections::{BTreeSet, HashMap, VecDeque};
use std::sync::{RwLock, RwLockReadGuard, Mutex, MutexGuard};

use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, GuidPrefix};
use crate::behavior::types::Duration;
use crate::behavior::stateful_writer::{StatefulWriterBehavior, BestEffortStatefulWriterBehavior, ReliableStatefulWriterBehavior};
use crate::messages::{RtpsSubmessage};
use crate::messages::message_receiver::Receiver;
use crate::messages::message_sender::Sender;
use crate::structure::history_cache::HistoryCache;
use crate::structure::cache_change::CacheChange;
use crate::messages::{ParameterList};

struct ChangeForReader {
    highest_sequence_number_sent: SequenceNumber,
    highest_sequence_number_acknowledged: SequenceNumber,
    sequence_numbers_requested: BTreeSet<SequenceNumber>,
}

impl ChangeForReader {
    fn new() -> Self {
        Self {
            highest_sequence_number_sent: 0,
            highest_sequence_number_acknowledged: 0,
            sequence_numbers_requested: BTreeSet::new(),
        }
    }

    fn next_unsent_change(&mut self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = self.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            self.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            self.highest_sequence_number_sent + 1..=last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    fn acked_changes(&self) -> SequenceNumber {
        self.highest_sequence_number_acknowledged
    }

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_sequence_number_acknowledged = committed_seq_num;
    }

    fn unacked_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unacked_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            self.highest_sequence_number_acknowledged + 1..=last_change_sequence_number
        {
            unacked_changes_set.insert(unsent_sequence_number);
        }

        unacked_changes_set
    }

    fn requested_changes_set(&mut self, req_seq_num_set: BTreeSet<SequenceNumber>) {
        let mut new_set = req_seq_num_set;
        self.sequence_numbers_requested.append(&mut new_set);
    }

    fn requested_changes(&self) -> BTreeSet<SequenceNumber> {
        self.sequence_numbers_requested.clone()
    }

    fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        let next_requested_change = *self.sequence_numbers_requested.iter().next()?;

        self.sequence_numbers_requested.remove(&next_requested_change);

        Some(next_requested_change)
    }
}


pub struct ReaderProxy {
    remote_reader_guid: GUID,
    // remoteGroupEntityId: EntityId_t,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    changes_for_reader: Mutex<ChangeForReader>,
    expects_inline_qos: bool,
    is_active: bool,
    stateful_writer_behavior: Mutex<StatefulWriterBehavior>,

    send_messages: Mutex<VecDeque<RtpsSubmessage>>,
    received_messages: Mutex<VecDeque<(GuidPrefix, RtpsSubmessage)>>,
}


impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
        is_active: bool) -> Self {
            Self {
                remote_reader_guid,
                unicast_locator_list,
                multicast_locator_list,
                expects_inline_qos,
                is_active,
                changes_for_reader: Mutex::new(ChangeForReader::new()),
                stateful_writer_behavior: Mutex::new(StatefulWriterBehavior::new()),
                send_messages: Mutex::new(VecDeque::new()),
                received_messages: Mutex::new(VecDeque::new()),
        }
    }

    pub fn acked_changes_set(&self, committed_seq_num: SequenceNumber) {
        self.changes_for_reader.lock().unwrap().acked_changes_set(committed_seq_num);
    }

    pub fn next_requested_change(&self) -> Option<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().next_requested_change()
    }

    pub fn next_unsent_change(&self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().next_unsent_change(last_change_sequence_number)
    }

    pub fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().unsent_changes(last_change_sequence_number)
    }

    pub fn requested_changes(&self) -> BTreeSet<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().requested_changes()
    }

    pub fn requested_changes_set(&self, req_seq_num_set: BTreeSet<SequenceNumber>) {
        self.changes_for_reader.lock().unwrap().requested_changes_set(req_seq_num_set);
    }

    pub fn unacked_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().unacked_changes(last_change_sequence_number)
    }

    pub fn unicast_locator_list(&self) -> &Vec<Locator> {
        &self.unicast_locator_list
    }

    pub fn multicast_locator_list(&self) -> &Vec<Locator> {
        &self.multicast_locator_list
    }

    pub fn remote_reader_guid(&self) -> &GUID {
        &self.remote_reader_guid
    }

    pub fn behavior(&self) -> MutexGuard<StatefulWriterBehavior> {
        self.stateful_writer_behavior.lock().unwrap()
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

    // All messages are received from the reader proxies so these fields are not used
    // /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    // unicast_locator_list: Vec<Locator>,
    // /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    // multicast_locator_list: Vec<Locator>,

    //Writer class:
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: Mutex<SequenceNumber>,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    matched_readers: RwLock<HashMap<GUID, ReaderProxy>>,
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,) -> Self {
        StatefulWriter {
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: Mutex::new(0),
            writer_cache: HistoryCache::new(),
            data_max_sized_serialized: None,
            matched_readers: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_change(
        &self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        *self.last_change_sequence_number.lock().unwrap() += 1;
        CacheChange::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number(),
            data,
            inline_qos,
        )
    }

    pub fn last_change_sequence_number(&self) -> SequenceNumber {
        *self.last_change_sequence_number.lock().unwrap()
    }

    pub fn guid(&self) -> &GUID {
        &self.guid
    }

    pub fn writer_cache(&self) -> &HistoryCache {
        &self.writer_cache
    }

    pub fn matched_reader_add(&self, a_reader_proxy: ReaderProxy) {
        self.matched_readers.write().unwrap().insert(a_reader_proxy.remote_reader_guid, a_reader_proxy);
    }

    pub fn matched_reader_remove(&self, reader_proxy_guid: &GUID) {
        self.matched_readers.write().unwrap().remove(reader_proxy_guid);
    }
    
    pub fn matched_readers(&self) -> RwLockReadGuard<HashMap<GUID, ReaderProxy>> {
        self.matched_readers.read().unwrap()
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    pub fn nack_response_delay(&self) -> Duration {
        self.nack_response_delay
    }

    pub fn run(&self) {
        for (_, reader_proxy) in self.matched_readers().iter() {
            match self.reliability_level {
                ReliabilityKind::BestEffort => BestEffortStatefulWriterBehavior::run(reader_proxy, &self),
                ReliabilityKind::Reliable => ReliableStatefulWriterBehavior::run(reader_proxy, &self),
            };
        }
    }
}

impl Receiver for StatefulWriter {
    fn push_receive_message(&self, source_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) {
        let reader_id = match &submessage {
            RtpsSubmessage::AckNack(acknack) => acknack.reader_id(),
            _ => panic!("Unsupported message received by stateful writer"),
        };
        let guid = GUID::new(source_guid_prefix, reader_id);
        self.matched_readers().get(&guid).unwrap().received_messages.lock().unwrap().push_back((source_guid_prefix, submessage));
    }
    
    fn pop_receive_message(&self, guid: &GUID) -> Option<(GuidPrefix, RtpsSubmessage)> {
        self.matched_readers().get(guid).unwrap().received_messages.lock().unwrap().pop_front()
    }

    fn is_submessage_destination(&self, _src_locator: &Locator, src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        let reader_id = match &submessage {
            RtpsSubmessage::AckNack(acknack) => acknack.reader_id(),
            _ => return false,
        };

        let reader_guid = GUID::new(*src_guid_prefix, reader_id);

        self.matched_readers().get(&reader_guid).is_some()
    }
}

impl Sender for StatefulWriter {
    fn push_send_message(&self, _dst_locator: &Locator, dst_guid: &GUID, submessage: RtpsSubmessage) {
       self.matched_readers().get(dst_guid).unwrap().send_messages.lock().unwrap().push_back(submessage)
    }

    fn pop_send_message(&self) -> Option<(Vec<Locator>, VecDeque<RtpsSubmessage>)> {
        for (_, reader_proxy) in self.matched_readers().iter() {
            let mut reader_proxy_send_messages = reader_proxy.send_messages.lock().unwrap();
            if !reader_proxy_send_messages.is_empty() {
                let mut send_message_queue = VecDeque::new();
                std::mem::swap(&mut send_message_queue, &mut reader_proxy_send_messages);
                
                let mut locator_list = Vec::new();
                locator_list.extend(reader_proxy.unicast_locator_list());
                locator_list.extend(reader_proxy.multicast_locator_list());

                return Some((locator_list, send_message_queue));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,};

    use crate::behavior::types::constants::DURATION_ZERO;
    use crate::messages::Endianness;
    use crate::messages::submessages::AckNack;

    use std::thread::sleep;

    #[test]
    fn stateful_writer_new_change() {
        let writer = StatefulWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
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

        assert_eq!(cache_change_seq1.sequence_number(), 1);
        assert_eq!(cache_change_seq1.change_kind(), &ChangeKind::Alive);
        assert_eq!(cache_change_seq1.inline_qos().len(), 0);
        assert_eq!(cache_change_seq1.instance_handle(), &[1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), 2);
        assert_eq!(
            cache_change_seq2.change_kind(),
            &ChangeKind::NotAliveUnregistered
        );
        assert_eq!(cache_change_seq2.inline_qos().len(), 0);
        assert_eq!(cache_change_seq2.instance_handle(), &[1; 16]);
    }

    #[test]
    fn reader_proxy_unsent_changes_operations() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

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
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

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
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

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

    #[test]
    fn run_best_effort_send_data() {
        let heartbeat_period = Duration::from_millis(200);

        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let stateful_writer = StatefulWriter::new(
            writer_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            true,
            heartbeat_period,
            DURATION_ZERO,
            DURATION_ZERO
        );

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        stateful_writer.matched_reader_add(reader_proxy);

        let instance_handle = [1;16];
        let cache_change_seq1 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![1,2,3]), None, instance_handle);
        let cache_change_seq2 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![2,3,4]), None, instance_handle);
        stateful_writer.writer_cache().add_change(cache_change_seq1);
        stateful_writer.writer_cache().add_change(cache_change_seq2);

        stateful_writer.run();
        let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();

        if let RtpsSubmessage::Data(data) = &messages[0] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 1);
        } else {
            panic!("Wrong message sent");
        }

        if let RtpsSubmessage::Data(data) = &messages[1] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 2);
        } else {
            panic!("Wrong message sent");
        }

        // Check that no heartbeat is sent using best effort writers
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
        sleep(heartbeat_period.into());
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
    }

    #[test]
    fn run_reliable_send_data() {
        let heartbeat_period = Duration::from_millis(200);

        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let stateful_writer = StatefulWriter::new(
            writer_guid,
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            true,
            heartbeat_period,
            DURATION_ZERO,
            DURATION_ZERO
        );

        let remote_reader_guid_prefix = [1,2,3,4,5,6,7,8,9,10,11,12];
        let remote_reader_guid = GUID::new(remote_reader_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        stateful_writer.matched_reader_add(reader_proxy);

        let instance_handle = [1;16];
        let cache_change_seq1 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![1,2,3]), None, instance_handle);
        let cache_change_seq2 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![2,3,4]), None, instance_handle);
        stateful_writer.writer_cache().add_change(cache_change_seq1);
        stateful_writer.writer_cache().add_change(cache_change_seq2);

        stateful_writer.run();
        let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();

        if let RtpsSubmessage::Data(data) = &messages[0] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 1);
        } else {
            panic!("Wrong message sent");
        }

        if let RtpsSubmessage::Data(data) = &messages[1] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 2);
        } else {
            panic!("Wrong message sent. Expected Data");
        }

        // Check that heartbeat is sent while there are unacked changes
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
        sleep(heartbeat_period.into());
        stateful_writer.run();
        let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &messages[0] {
            assert_eq!(heartbeat.is_final(), false);
        } else {
            panic!("Wrong message sent. Expected Heartbeat");
        }

        let acknack = AckNack::new(
            remote_reader_guid.entity_id(),
            writer_guid.entity_id(),
                3,
                BTreeSet::new(),
                1,
                false,
                Endianness::LittleEndian,
        );

        stateful_writer.push_receive_message(remote_reader_guid_prefix, RtpsSubmessage::AckNack(acknack));

        // Check that no heartbeat is sent if there are no new changes
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
        sleep(heartbeat_period.into());
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
    }
}