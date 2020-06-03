use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::convert::TryInto;

use crate::cache::{CacheChange, HistoryCache};
use crate::inline_qos::{InlineQosParameter, InlineQosParameterList};
use crate::messages::{Data, Heartbeat, InfoTs, Payload, RtpsMessage, RtpsSubmessage, Header};
use crate::serdes::EndianessFlag;
use crate::types::constants::{ENTITYID_UNKNOWN};
use crate::types::{
    ChangeKind, Duration, InstanceHandle, Locator, LocatorList, ReliabilityKind, SequenceNumber,
    SerializedPayload, Time, TopicKind, GUID, ParameterList, StatusInfo, KeyHash, Count,
};

struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    expects_inline_qos: bool,
    highest_sequence_number_sent: SequenceNumber,
    sequence_numbers_requested: HashSet<SequenceNumber>,
    time_last_sent_data: Instant,
    heartbeat_count: Count,
}

impl ReaderLocator {
    fn new(expects_inline_qos: bool) -> Self {
        ReaderLocator {
            expects_inline_qos,
            highest_sequence_number_sent: SequenceNumber(0),
            sequence_numbers_requested: HashSet::new(),
            time_last_sent_data: Instant::now(),
            heartbeat_count: Count(1),
        }
    }

    fn unsent_changes_reset(&mut self) {
        self.highest_sequence_number_sent = SequenceNumber(0);
    }

    fn time_last_sent_data_reset(&mut self) {
        self.time_last_sent_data = Instant::now();
    }

    fn duration_since_last_sent_data(&self) -> Duration {
        self.time_last_sent_data.elapsed().try_into().unwrap()
    }

    fn increment_heartbeat_count(&mut self) {
        self.heartbeat_count += 1;
    }

    fn heartbeat_count(&self) -> Count {
        self.heartbeat_count
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
    unicast_locator_list: LocatorList,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: LocatorList,

    //Writer class:
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    reader_locators: HashMap<Locator, ReaderLocator>,
}

impl StatelessWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        StatelessWriter {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: SequenceNumber(0),
            writer_cache: HistoryCache::new(),
            data_max_sized_serialized: None,
            reader_locators: HashMap::new(),
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<InlineQosParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        CacheChange::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number,
            inline_qos,
            data,
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

    pub fn next_unsent_change(&mut self, a_locator: Locator) -> Option<SequenceNumber> {
        let reader_locator = self.reader_locators.get_mut(&a_locator)?;

        let next_unsent_sequence_number = reader_locator.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > self.last_change_sequence_number {
            None
        } else {
            reader_locator.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn unsent_changes(&self, a_locator: Locator) -> HashSet<SequenceNumber> {
        let reader_locator = self.reader_locators.get(&a_locator).unwrap();

        let mut unsent_changes_set = HashSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            reader_locator.highest_sequence_number_sent.0 + 1..=self.last_change_sequence_number.0
        {
            unsent_changes_set.insert(SequenceNumber(unsent_sequence_number));
        }

        unsent_changes_set
    }

    pub fn has_unsent_changes(&self, a_locator: Locator) -> bool {
        let reader_locator = self.reader_locators.get(&a_locator).unwrap();

        if reader_locator.highest_sequence_number_sent + 1 >= self.last_change_sequence_number {
            false
        } else {
            true
        }
    }

    pub fn requested_changes_set(
        &mut self,
        a_locator: Locator,
        req_seq_num_set: HashSet<SequenceNumber>,
    ) {
        let reader_locator = self.reader_locators.get_mut(&a_locator).unwrap();
        reader_locator.sequence_numbers_requested = req_seq_num_set;
    }

    fn time_since_last_data_sent(&self, a_locator: Locator) -> Duration {
        let reader_locator = self.reader_locators.get(&a_locator).unwrap();
        reader_locator.duration_since_last_sent_data()
    }

    fn time_last_sent_data_reset(&mut self, a_locator: Locator) {
        let reader_locator = self.reader_locators.get_mut(&a_locator).unwrap();
        reader_locator.time_last_sent_data_reset();
    }

    fn heartbeat_count(&self, a_locator: Locator) -> Count {
        let reader_locator = self.reader_locators.get(&a_locator).unwrap();
        reader_locator.heartbeat_count()
    }

    fn increment_heartbeat_count(&mut self, a_locator: Locator) {
        let reader_locator = self.reader_locators.get_mut(&a_locator).unwrap();
        reader_locator.increment_heartbeat_count();
    }

    pub fn get_data_to_send(&mut self, a_locator: Locator) -> RtpsMessage {
        let mut message = RtpsMessage::new(
            Header::new(*self.guid.prefix()), 
            Vec::new() );

        if self.has_unsent_changes(a_locator) {
            let time = Time::now();
            let infots = InfoTs::new(Some(time), EndianessFlag::LittleEndian);

            message.push(RtpsSubmessage::InfoTs(infots));

            while let Some(next_unsent_seq_num) = self.next_unsent_change(a_locator) {
                if let Some(cache_change) = self
                    .writer_cache
                    .get_change_with_sequence_number(&next_unsent_seq_num)
                {
                    let change_kind = *cache_change.change_kind();

                    let mut inline_qos_parameter_list : InlineQosParameterList = ParameterList::new();

                    let payload = match change_kind {
                        ChangeKind::Alive => {
                            inline_qos_parameter_list.push(InlineQosParameter::StatusInfo(StatusInfo::from(change_kind)));
                            inline_qos_parameter_list.push(InlineQosParameter::KeyHash(KeyHash(*cache_change.instance_handle())));
                            Payload::Data(SerializedPayload(cache_change.data().unwrap().to_vec()))
                        },
                        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
                            inline_qos_parameter_list.push(InlineQosParameter::StatusInfo(StatusInfo::from(change_kind)));
                            Payload::Key(SerializedPayload(cache_change.instance_handle().to_vec()))
                        }
                    };

                    let data = Data::new(
                        EndianessFlag::LittleEndian.into(),
                        ENTITYID_UNKNOWN,
                        *self.guid.entity_id(),
                        *cache_change.sequence_number(),
                        Some(inline_qos_parameter_list), 
                        payload,
                    );

                    message.push(RtpsSubmessage::Data(data));
                } else {
                    panic!("GAP not implemented yet");
                    // let gap = Gap::new(ENTITYID_UNKNOWN /*reader_id*/,ENTITYID_UNKNOWN /*writer_id*/, 0 /*gap_start*/, BTreeMap::new() /*gap_list*/);

                    // message.push(RtpsSubmessage::Gap(gap));
                }
            }

            self.time_last_sent_data_reset(a_locator);
        } else {
            if self.reliability_level == ReliabilityKind::Reliable {
                if self.time_since_last_data_sent(a_locator) >= self.heartbeat_period {
                    let time = Time::now();
                    let infots = InfoTs::new(Some(time), EndianessFlag::LittleEndian);

                    message.push(RtpsSubmessage::InfoTs(infots));

                    let first_sn = if let Some(seq_num) = self.writer_cache.get_seq_num_min() {
                        seq_num
                    } else {
                        self.last_change_sequence_number + 1
                    };

                    let heartbeat = Heartbeat::new(
                        ENTITYID_UNKNOWN,
                        *self.guid.entity_id(),
                        first_sn,
                        self.last_change_sequence_number,
                        self.heartbeat_count(a_locator),
                        true,
                        false,
                        EndianessFlag::LittleEndian,
                    );

                    message.push(RtpsSubmessage::Heartbeat(heartbeat));

                    self.increment_heartbeat_count(a_locator);
                    self.time_last_sent_data_reset(a_locator);
                }
            }
        }

        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::*;
    use crate::types::*;
    use std::thread::sleep;

    #[test]
    fn test_writer_new_change() {
        let mut writer = StatelessWriter::new(
            GUID::new(GuidPrefix([0; 12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            LocatorList(vec![Locator::new(0, 7400, [0; 16])]), /*unicast_locator_list*/
            LocatorList(vec![]),                               /*multicast_locator_list*/
            false,                                /*push_mode*/
            DURATION_ZERO,                        /* heartbeat_period */
            DURATION_ZERO,                        /* nack_response_delay */
            DURATION_ZERO,                        /* nack_suppression_duration */
        );

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]), /*data*/
            None,                /*inline_qos*/
            [1; 16],             /*handle*/
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None,    /*data*/
            None,    /*inline_qos*/
            [1; 16], /*handle*/
        );

        assert_eq!(cache_change_seq1.sequence_number(), &SequenceNumber(1));
        assert_eq!(cache_change_seq1.change_kind(), &ChangeKind::Alive);
        assert_eq!(cache_change_seq1.inline_qos(), &None);
        assert_eq!(cache_change_seq1.instance_handle(), &[1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), &SequenceNumber(2));
        assert_eq!(
            cache_change_seq2.change_kind(),
            &ChangeKind::NotAliveUnregistered
        );
        assert_eq!(cache_change_seq2.inline_qos(), &None);
        assert_eq!(cache_change_seq2.instance_handle(), &[1; 16]);
    }

    #[test]
    fn test_stateless_writer_unsent_changes() {
        let mut writer = StatelessWriter::new(
            GUID::new(GuidPrefix([0; 12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            LocatorList(vec![Locator::new(0, 7400, [0; 16])]), /*unicast_locator_list*/
            LocatorList(vec![]),                               /*multicast_locator_list*/
            false,                                /*push_mode*/
            DURATION_ZERO,                        /* heartbeat_period */
            DURATION_ZERO,                        /* nack_response_delay */
            DURATION_ZERO,                        /* nack_suppression_duration */
        );

        let locator = Locator::new(0, 7400, [1; 16]);

        writer.reader_locator_add(locator);

        // Verify next unsent change without any changes created
        assert_eq!(writer.unsent_changes(locator), HashSet::new());
        assert_eq!(writer.next_unsent_change(locator), None);

        writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]), /*data*/
            None,                /*inline_qos*/
            [1; 16],             /*handle*/
        );

        writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None,    /*data*/
            None,    /*inline_qos*/
            [1; 16], /*handle*/
        );

        // let hash_set_2_changes : HashSet<SequenceNumber> = [1,2].iter().cloned().collect();
        // assert_eq!(writer.unsent_changes(locator), hash_set_2_changes);
        assert_eq!(writer.next_unsent_change(locator), Some(SequenceNumber(1)));

        // let hash_set_1_change : HashSet<SequenceNumber> = [2].iter().cloned().collect();
        // assert_eq!(writer.unsent_changes(locator), hash_set_1_change);
        assert_eq!(writer.next_unsent_change(locator), Some(SequenceNumber(2)));

        assert_eq!(writer.unsent_changes(locator), HashSet::new());
        assert_eq!(writer.next_unsent_change(locator), None);
        assert_eq!(writer.next_unsent_change(locator), None);

        writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]), 
            None,                
            [1; 16],             
        );

        assert_eq!(writer.next_unsent_change(locator), Some(SequenceNumber(3)));

        writer.unsent_changes_reset();
        // let hash_set_3_changes : HashSet<SequenceNumber> = [1,2,3].iter().cloned().collect();
        // assert_eq!(writer.unsent_changes(locator), hash_set_3_changes);
    }

    #[test]
    fn test_best_effort_stateless_writer_get_data_to_send() {
        let mut writer = StatelessWriter::new(
            GUID::new(GuidPrefix([0; 12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            LocatorList(vec![Locator::new(0, 7400, [0; 16])]), 
            LocatorList(vec![]),                               
            false,                                
            DURATION_ZERO,                        
            DURATION_ZERO,                        
            DURATION_ZERO,                        
        );

        let locator = Locator::new(0, 7400, [1; 16]);

        writer.reader_locator_add(locator);

        // Verify next unsent change without any changes created
        assert_eq!(writer.unsent_changes(locator), HashSet::new());
        assert_eq!(writer.next_unsent_change(locator), None);

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

        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![4, 5, 6])));
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 0);
    }

    #[test]
    fn test_reliable_stateless_writer_get_data_to_send() {
        let heartbeat_period = Duration::from_millis(500);

        let mut writer = StatelessWriter::new(
            GUID::new(GuidPrefix([0; 12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            LocatorList(vec![Locator::new(0, 7400, [0; 16])]), 
            LocatorList(vec![]),
            false,                                
            heartbeat_period,                        
            DURATION_ZERO,                        
            DURATION_ZERO,                        
        );

        let locator = Locator::new(0, 7400, [1; 16]);

        writer.reader_locator_add(locator);

        // Test for heartbeat message before any data is added
        sleep(heartbeat_period.into());

        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 2);
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &writer_data.submessages()[1] {
            assert_eq!(heartbeat_message.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(heartbeat_message.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(heartbeat_message.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat_message.last_sn(), &SequenceNumber(0));
            assert_eq!(heartbeat_message.count(), &Count(1));
            assert_eq!(heartbeat_message.is_final(), true);

        } else {
            panic!("Wrong message type");
        };

        // Verify next unsent change without any changes created
        assert_eq!(writer.unsent_changes(locator), HashSet::new());
        assert_eq!(writer.next_unsent_change(locator), None);

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

        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![4, 5, 6])));
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 0);

        sleep(heartbeat_period.into());

        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 2);
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &writer_data.submessages()[1] {
            assert_eq!(heartbeat_message.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(heartbeat_message.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(heartbeat_message.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat_message.last_sn(), &SequenceNumber(2));
            assert_eq!(heartbeat_message.count(), &Count(2));
            assert_eq!(heartbeat_message.is_final(), true);

        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the heartbeat
        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 0);

        sleep(heartbeat_period.into());

        let writer_data = writer.get_data_to_send(locator);
        assert_eq!(writer_data.submessages().len(), 2);
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &writer_data.submessages()[1] {
            assert_eq!(heartbeat_message.reader_id(), &ENTITYID_UNKNOWN);
            assert_eq!(heartbeat_message.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(heartbeat_message.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat_message.last_sn(), &SequenceNumber(2));
            assert_eq!(heartbeat_message.count(), &Count(3));
            assert_eq!(heartbeat_message.is_final(), true);

        } else {
            panic!("Wrong message type");
        };

    }
}
