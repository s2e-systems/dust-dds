use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use crate::types::{ProtocolVersion, SequenceNumber, Locator, GUID, TopicKind, LocatorList, ReliabilityKind, Duration, ChangeKind, InstanceHandle, ParameterList, ENTITYID_UNKNOWN, Time, VENDOR_ID};
use crate::entity::Entity;
use crate::cache::{CacheChange, HistoryCache};
use crate::messages::{RtpsMessage, RtpsSubmessage, Data, Payload, InlineQosParameter, InfoTs};

pub struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    expects_inline_qos: bool,
    highest_sequence_number_sent: SequenceNumber,
    sequence_numbers_requested: HashSet<SequenceNumber>,
}

impl ReaderLocator {
    pub fn new(expects_inline_qos: bool) -> Self {
        ReaderLocator {
            expects_inline_qos,
            highest_sequence_number_sent: 0,
            sequence_numbers_requested: HashSet::new(),
        }
    }

    fn unsent_changes_reset(&mut self) {
        self.highest_sequence_number_sent = 0;
    }
}

pub struct StatelessWriter {
    /// Entity base class (contains the GUID)
    entity: Entity,

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

    pub reader_locators: HashMap<Locator, ReaderLocator>,
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
            entity: Entity{guid},
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
            reader_locators: HashMap::new(),
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList<InlineQosParameter>>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        CacheChange::new(
            kind,
            self.entity.guid,
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
        self.reader_locators.insert(
            a_locator,
            ReaderLocator::new(false /*expects_inline_qos*/),
        );
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.remove(a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for (_, rl) in self.reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }

    pub fn next_unsent_change(&mut self, a_locator: Locator) -> Option<SequenceNumber>
    {
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

        for unsent_sequence_number in reader_locator.highest_sequence_number_sent+1..=self.last_change_sequence_number {
            unsent_changes_set.insert(unsent_sequence_number);
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

    pub fn requested_changes_set(&mut self, a_locator: Locator, req_seq_num_set: HashSet<SequenceNumber>) {
        let reader_locator = self.reader_locators.get_mut(&a_locator).unwrap();
        reader_locator.sequence_numbers_requested = req_seq_num_set;
    }

    pub fn get_data_to_send(&mut self, a_locator: Locator) -> RtpsMessage {
        let mut message = RtpsMessage::new(
            *self.entity.guid.prefix(),
            VENDOR_ID,
            ProtocolVersion{major:2, minor: 4});

        if self.has_unsent_changes(a_locator) {
            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let time = Time{seconds: current_time.as_secs() as u32 , fraction: current_time.as_nanos() as u32};
            let infots = InfoTs::new(Some(time));

            message.push(RtpsSubmessage::InfoTs(infots));

            while let Some(next_unsent_seq_num) = self.next_unsent_change(a_locator)
            {
                if let Some(cache_change) = self.writer_cache.get_change_with_sequence_number(&next_unsent_seq_num) {
                    let payload_data = Data::new(
                        ENTITYID_UNKNOWN, /*reader_id*/
                        *self.entity.guid.entity_id(), /*writer_id*/
                        *cache_change.get_instance_handle(), /*key_hash*/
                        *cache_change.get_sequence_number(), /*writer_sn*/
                        None, /*inline_qos*/
                        Payload::Data(cache_change.data().unwrap().to_vec()) /*serialized_payload*/);

                    message.push(RtpsSubmessage::Data(payload_data));

                } else {
                    panic!("GAP not implemented yet");
                    // let gap = Gap::new(ENTITYID_UNKNOWN /*reader_id*/,ENTITYID_UNKNOWN /*writer_id*/, 0 /*gap_start*/, BTreeMap::new() /*gap_list*/);

                    // message.push(RtpsSubmessage::Gap(gap));
                }
            }
        }
        
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_writer_new_change() {
        let mut writer = StatelessWriter::new(
             GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
             TopicKind::WithKey,
             ReliabilityKind::BestEffort,
             vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
             vec![], /*multicast_locator_list*/
             false, /*push_mode*/
             DURATION_ZERO,  /* heartbeat_period */
             DURATION_ZERO, /* nack_response_delay */
             DURATION_ZERO, /* nack_suppression_duration */
            );

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1,2,3]), /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None, /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        assert_eq!(cache_change_seq1.get_sequence_number(), &1);
        assert_eq!(cache_change_seq1.get_change_kind(), &ChangeKind::Alive);
        assert_eq!(cache_change_seq1.get_inline_qos(), &None);
        assert_eq!(cache_change_seq1.get_instance_handle(), &[1;16]);

        assert_eq!(cache_change_seq2.get_sequence_number(), &2);
        assert_eq!(cache_change_seq2.get_change_kind(), &ChangeKind::NotAliveUnregistered);
        assert_eq!(cache_change_seq2.get_inline_qos(), &None);
        assert_eq!(cache_change_seq2.get_instance_handle(), &[1;16]);
    }

    #[test]
    fn test_stateless_writer_unsent_changes() {
        let mut writer = StatelessWriter::new(
             GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
             TopicKind::WithKey,
             ReliabilityKind::BestEffort,
             vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
             vec![], /*multicast_locator_list*/
             false, /*push_mode*/
             DURATION_ZERO,  /* heartbeat_period */
             DURATION_ZERO, /* nack_response_delay */
             DURATION_ZERO, /* nack_suppression_duration */
            );

        let locator = Locator::new(0, 7400, [1;16]);

        writer.reader_locator_add(locator);

        // Verify next unsent change without any changes created
        assert_eq!(writer.unsent_changes(locator), HashSet::new());
        assert_eq!(writer.next_unsent_change(locator), None);

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1,2,3]), /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );
        
        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None, /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        let hash_set_2_changes : HashSet<SequenceNumber> = [1,2].iter().cloned().collect();
        assert_eq!(writer.unsent_changes(locator), hash_set_2_changes);
        assert_eq!(writer.next_unsent_change(locator), Some(1));

        let hash_set_1_change : HashSet<SequenceNumber> = [2].iter().cloned().collect();
        assert_eq!(writer.unsent_changes(locator), hash_set_1_change);
        assert_eq!(writer.next_unsent_change(locator), Some(2));

        assert_eq!(writer.unsent_changes(locator), HashSet::new());    
        assert_eq!(writer.next_unsent_change(locator), None);
        assert_eq!(writer.next_unsent_change(locator), None);        

        let cache_change_seq3 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1,2,3]), /*data*/
            None, /*inline_qos*/
            [1;16], /*handle*/
        );

        assert_eq!(writer.next_unsent_change(locator), Some(3));

        writer.unsent_changes_reset();
        let hash_set_3_changes : HashSet<SequenceNumber> = [1,2,3].iter().cloned().collect();
        assert_eq!(writer.unsent_changes(locator), hash_set_3_changes);

    }

    #[test]
    fn test_stateless_writer_get_data_to_send() {
        let mut writer = StatelessWriter::new(
            GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
            vec![], /*multicast_locator_list*/
            false, /*push_mode*/
            DURATION_ZERO,  /* heartbeat_period */
            DURATION_ZERO, /* nack_response_delay */
            DURATION_ZERO, /* nack_suppression_duration */
           );

       let locator = Locator::new(0, 7400, [1;16]);

       writer.reader_locator_add(locator);

       // Verify next unsent change without any changes created
       assert_eq!(writer.unsent_changes(locator), HashSet::new());
       assert_eq!(writer.next_unsent_change(locator), None);

       let cache_change_seq1 = writer.new_change(
           ChangeKind::Alive,
           Some(vec![1,2,3]), /*data*/
           None, /*inline_qos*/
           [1;16], /*handle*/
       );
       
       let cache_change_seq2 = writer.new_change(
           ChangeKind::Alive,
           Some(vec!(4,5,6)), /*data*/
           None, /*inline_qos*/
           [1;16], /*handle*/
       );

       writer.history_cache().add_change(cache_change_seq1);
       writer.history_cache().add_change(cache_change_seq2);

       let writer_data = writer.get_data_to_send(locator);
       assert_eq!(writer_data.get_submessages().len(), 3);
       if let RtpsSubmessage::InfoTs(message_1) = &writer_data.get_submessages()[0] {
           println!("{:?}", message_1);
       } else {
           panic!("Wrong message type");
       }
       if let RtpsSubmessage::Data(message_2) = &writer_data.get_submessages()[1] {
           
       }
       else {
           panic!("Wrong message type");
       };

       if let RtpsSubmessage::Data(message_3) = &writer_data.get_submessages()[2] {
           
        }
        else {
            panic!("Wrong message type");
        };

    }
}