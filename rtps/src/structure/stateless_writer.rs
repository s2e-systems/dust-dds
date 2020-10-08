use std::collections::{HashMap, BTreeSet, VecDeque};
use std::sync::{RwLock, RwLockReadGuard, Mutex, MutexGuard};

use crate::structure::history_cache::HistoryCache;
use crate::structure::cache_change::CacheChange;
use crate::serialized_payload::ParameterList;
use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior::stateless_writer::BestEffortStatelessWriterBehavior;
use crate::messages::RtpsSubmessage;
use crate::messages::message_sender::Sender;

use rust_dds_interface::qos::DataWriterQos;

pub struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    locator: Locator,
    expects_inline_qos: bool,

    highest_sequence_number_sent: Mutex<SequenceNumber>,

    send_messages: Mutex<VecDeque<RtpsSubmessage>>,
}

impl ReaderLocator {
    fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            highest_sequence_number_sent: Mutex::new(0),
            send_messages: Mutex::new(VecDeque::new()),
        }
    }

    pub fn locator(&self) -> &Locator {
        &self.locator
    }

    fn highest_sequence_number_sent(&self) -> MutexGuard<SequenceNumber> {
        self.highest_sequence_number_sent.lock().unwrap()
    }

    fn unsent_changes_reset(&self) {
        *self.highest_sequence_number_sent() = 0;
    }

    pub fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            *self.highest_sequence_number_sent() + 1..=last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    pub fn next_unsent_change(&self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = *self.highest_sequence_number_sent() + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            *self.highest_sequence_number_sent() = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }
}

pub struct StatelessWriter {
    /// Entity base class (contains the GUID)
    guid: GUID,
    // entity: Entity,

    // Endpoint base class:
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,

    // The stateless writer does not receive messages so these fields are left out

    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    // unicast_locator_list: Vec<Locator>,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    // multicast_locator_list: Vec<Locator>,

    
    last_change_sequence_number: Mutex<SequenceNumber>,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    reader_locators: RwLock<HashMap<Locator, ReaderLocator>>,
}

impl StatelessWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        writer_qos: &DataWriterQos
    ) -> Self {
        StatelessWriter {
            guid,
            topic_kind,
            reliability_level: ReliabilityKind::BestEffort,
            last_change_sequence_number: Mutex::new(0),
            writer_cache: HistoryCache::new(&writer_qos.resource_limits),
            data_max_sized_serialized: None,
            reader_locators: RwLock::new(HashMap::new()),
        }
    }

    pub fn last_change_sequence_number(&self) -> SequenceNumber {
        *self.last_change_sequence_number.lock().unwrap()
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

    pub fn guid(&self) -> &GUID {
        &self.guid
    }

    pub fn writer_cache(&self) -> &HistoryCache {
        &self.writer_cache
    }

    pub fn reader_locator_add(&self, a_locator: Locator) {
        self.reader_locators.write().unwrap()
            .insert(a_locator, ReaderLocator::new(a_locator, false /*expects_inline_qos*/));
    }

    pub fn reader_locator_remove(&self, a_locator: &Locator) {
        self.reader_locators.write().unwrap().remove(a_locator);
    }

    pub fn reader_locators(&self) -> RwLockReadGuard<HashMap<Locator, ReaderLocator>>{
        self.reader_locators.read().unwrap()
    }

    pub fn unsent_changes_reset(&self) {
        for (_, rl) in self.reader_locators().iter() {
            rl.unsent_changes_reset();
        }
    }

    pub fn run(&self) {
        assert!(self.reliability_level == ReliabilityKind::BestEffort);

        for (_, reader_locator) in self.reader_locators().iter() {
            BestEffortStatelessWriterBehavior::run(reader_locator, &self);
        }
    }
}

impl Sender for StatelessWriter {
    fn push_send_message(&self, dst_locator: &Locator, _dst_guid: &GUID, submessage: RtpsSubmessage) {
        self.reader_locators().get(dst_locator).unwrap().send_messages.lock().unwrap().push_back(submessage)
    }

    fn pop_send_message(&self) -> Option<(Vec<Locator>, VecDeque<RtpsSubmessage>)> {
        for (&locator, reader_locator) in self.reader_locators().iter() {
            let mut reader_locator_send_messages = reader_locator.send_messages.lock().unwrap();
            if !reader_locator_send_messages.is_empty() {
                let mut send_message_queue = VecDeque::new();
                std::mem::swap(&mut send_message_queue, &mut reader_locator_send_messages);
                
                return Some((vec![locator], send_message_queue));
            }
        }
        None
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::*;
    use crate::types::*;

    #[test]
    fn test_writer_new_change() {
        let writer_qos = DataWriterQos::default();
        let writer = StatelessWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            &writer_qos
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
    fn test_best_effort_stateless_writer_run() {
        let writer_qos = DataWriterQos::default();

        let writer = StatelessWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            &writer_qos
        );

        let locator = Locator::new(0, 7400, [1; 16]);

        writer.reader_locator_add(locator);

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]), 
            None,                
            [1; 16],             
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![4, 5, 6]), 
            None,                
            [1; 16],             
        );

        writer.writer_cache().add_change(cache_change_seq1);
        writer.writer_cache().add_change(cache_change_seq2);

        writer.run();

        let (_dst_locators, messages) = writer.pop_send_message().unwrap();
        assert!(writer.pop_send_message().is_none());

        if let RtpsSubmessage::Data(data_message_1) = &messages[0] {
            assert_eq!(data_message_1.reader_id(), ENTITYID_UNKNOWN);
            assert_eq!(data_message_1.writer_id(), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn(), 1);
            assert_eq!(data_message_1.serialized_payload(), &vec![1, 2, 3]);
        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &messages[1] {
            assert_eq!(data_message_2.reader_id(), ENTITYID_UNKNOWN);
            assert_eq!(data_message_2.writer_id(), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn(), 2);
            assert_eq!(data_message_2.serialized_payload(), &vec![4, 5, 6]);
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        writer.run();
        assert!(writer.pop_send_message().is_none());
    }
}
