use std::collections::{HashMap, BTreeSet};
use std::sync::{Mutex, MutexGuard};

use crate::cache::{CacheChange, HistoryCache};
use crate::messages::{RtpsMessage, ParameterList};
use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, };
use crate::behavior::stateless_writer::StatelessWriterBehavior;

pub struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    expects_inline_qos: bool,
    highest_sequence_number_sent: Mutex<SequenceNumber>,
}

impl ReaderLocator {
    fn new(expects_inline_qos: bool) -> Self {
        ReaderLocator {
            expects_inline_qos,
            highest_sequence_number_sent: Mutex::new(0),
        }
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
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    unicast_locator_list: Vec<Locator>,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: Vec<Locator>,

    //Writer class:
    push_mode: bool,
    
    last_change_sequence_number: Mutex<SequenceNumber>,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    reader_locators: HashMap<Locator, ReaderLocator>,
}

impl StatelessWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        push_mode: bool,
    ) -> Self {
        StatelessWriter {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            last_change_sequence_number: Mutex::new(0),
            writer_cache: HistoryCache::new(),
            data_max_sized_serialized: None,
            reader_locators: HashMap::new(),
        }
    }

    fn last_change_sequence_number(&self) -> MutexGuard<SequenceNumber> {
        self.last_change_sequence_number.lock().unwrap()
    }

    pub fn new_change(
        &self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        *self.last_change_sequence_number() += 1;

        CacheChange::new(
            kind,
            self.guid,
            handle,
            *self.last_change_sequence_number(),
            data,
            inline_qos,
        )
    }

    pub fn history_cache(&mut self) -> &mut HistoryCache {
        &mut self.writer_cache
    }

    pub fn reader_locator_add(&mut self, a_locator: Locator) {
        self.reader_locators
            .insert(a_locator, ReaderLocator::new(false /*expects_inline_qos*/));
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.remove(a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for (_, rl) in self.reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }

    pub fn run(&mut self, a_locator: &Locator) -> Option<RtpsMessage> {
        let last_change_sequence_number = *self.last_change_sequence_number();
        let reader_locator = self.reader_locators.get_mut(a_locator).unwrap();

        let submessages = match self.reliability_level {
            ReliabilityKind::BestEffort => StatelessWriterBehavior::run_best_effort(reader_locator, &self.guid, &self.writer_cache, last_change_sequence_number),
            ReliabilityKind::Reliable => panic!("Reliable StatelessWriter not implemented!"),
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
    use crate::types::constants::*;
    use crate::messages::RtpsSubmessage;
    use crate::types::*;

    #[test]
    fn test_writer_new_change() {
        let writer = StatelessWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0; 16])], /*unicast_locator_list*/
            vec![],                               /*multicast_locator_list*/
            false,                                /*push_mode*/
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
    fn test_best_effort_stateless_writer_run() {
        let mut writer = StatelessWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0; 16])], 
            vec![],                               
            false,
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

        writer.history_cache().add_change(cache_change_seq1);
        writer.history_cache().add_change(cache_change_seq2);

        let writer_data = writer.run(&locator).unwrap();
        assert_eq!(writer_data.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
            assert_eq!(data_message_1.reader_id(), ENTITYID_UNKNOWN);
            assert_eq!(data_message_1.writer_id(), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn(), 1);
            assert_eq!(data_message_1.serialized_payload(), Some(&vec![1, 2, 3]));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
            assert_eq!(data_message_2.reader_id(), ENTITYID_UNKNOWN);
            assert_eq!(data_message_2.writer_id(), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn(), 2);
            assert_eq!(data_message_2.serialized_payload(), Some(&vec![4, 5, 6]));
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        let writer_data = writer.run(&locator);
        assert!(writer_data.is_none());
    }
}
