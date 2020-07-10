use crate::types::{GUID, SequenceNumber, ChangeKind};
use crate::messages::{RtpsMessage, RtpsSubmessage, InfoTs, Data, Payload, Gap, Heartbeat};
use crate::cache::{HistoryCache};
use crate::stateful_writer::ReaderProxy;
use crate::behavior::types::Duration;
use crate::serdes::Endianness;
use crate::messages::types::{Time};
use crate::messages::submessage_elements;
use crate::inline_qos_types::{KeyHash, StatusInfo};
use super::data_from_cache_change;

pub struct StatefulWriterBehavior {}

impl StatefulWriterBehavior {
    pub fn run_best_effort(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Option<Vec<RtpsSubmessage>> {
        if !reader_proxy.unsent_changes(last_change_sequence_number).is_empty() {
            Some(StatefulWriterBehavior::run_pushing_state(reader_proxy, writer_guid, history_cache, last_change_sequence_number))
        } else {
            None
        }
    }
    
    pub fn run_reliable(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber, heartbeat_period: Duration, nack_response_delay: Duration, received_message: Option<&RtpsMessage>) -> Option<Vec<RtpsSubmessage>> {
        let sending_submessages =
        if reader_proxy.unacked_changes(last_change_sequence_number).is_empty() {
            // Idle
            None
        } else if !reader_proxy.unsent_changes(last_change_sequence_number).is_empty() {
            Some(StatefulWriterBehavior::run_pushing_state(reader_proxy, writer_guid, history_cache, last_change_sequence_number))
        } else if !reader_proxy.unacked_changes(last_change_sequence_number).is_empty() {
            StatefulWriterBehavior::run_announcing_state(reader_proxy, writer_guid, history_cache, last_change_sequence_number, heartbeat_period)
        } else {
            None
        };
    
        let repairing_submessages = if reader_proxy.requested_changes().is_empty() {
            StatefulWriterBehavior::run_waiting_state(reader_proxy, writer_guid, received_message);
            None // No data is sent by the waiting state
        } else {
            StatefulWriterBehavior::run_must_repair_state(reader_proxy, writer_guid, received_message);
            if reader_proxy.duration_since_nack_received() > nack_response_delay {
                Some(StatefulWriterBehavior::run_repairing_state(reader_proxy, writer_guid, history_cache))
            } else {
                None
            }
        };
    
        match (sending_submessages, repairing_submessages) {
            (Some(mut sending_submessages), Some(mut repairing_submessages)) => {
                sending_submessages.append(&mut repairing_submessages);
                Some(sending_submessages)
            },
            (Some(sending_submessages), None) => Some(sending_submessages),
            (None, Some(repairing_submessages)) => Some(repairing_submessages),
            (None, None) => None,
        }
    }

    fn run_pushing_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Vec<RtpsSubmessage> {

        // This state is only valid if there are unsent changes
        assert!(!reader_proxy.unsent_changes(last_change_sequence_number).is_empty());
    
        let endianness = Endianness::LittleEndian;
        let mut submessages = Vec::with_capacity(2); // TODO: Probably can be preallocated with the correct size
    
        let time = Time::now();
        let infots = InfoTs::new(Some(submessage_elements::Timestamp(time)), endianness);
        submessages.push(RtpsSubmessage::InfoTs(infots));
    
        while let Some(next_unsent_seq_num) = reader_proxy.next_unsent_change(last_change_sequence_number) {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_unsent_seq_num)
            {
                let reader_id = *reader_proxy.remote_reader_guid().entity_id();
                let data = data_from_cache_change(cache_change, endianness, reader_id);       
                submessages.push(RtpsSubmessage::Data(data));
            } else {
                let gap = Gap::new(
                    submessage_elements::EntityId(*reader_proxy.remote_reader_guid().entity_id()), 
                    submessage_elements::EntityId(*writer_guid.entity_id()),
                    submessage_elements::SequenceNumber(next_unsent_seq_num),
                    Endianness::LittleEndian);
    
                submessages.push(RtpsSubmessage::Gap(gap));
            }
        }
    
        submessages
    }

    fn run_announcing_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache,  last_change_sequence_number: SequenceNumber, heartbeat_period: Duration) -> Option<Vec<RtpsSubmessage>> {

        if reader_proxy.duration_since_last_sent_data() > heartbeat_period {
            let mut submessages = Vec::new();
    
            let time = Time::now();
            let infots = InfoTs::new(Some(submessage_elements::Timestamp(time)), Endianness::LittleEndian);
    
            submessages.push(RtpsSubmessage::InfoTs(infots));
    
            let first_sn = if let Some(seq_num) = history_cache.get_seq_num_min() {
                seq_num
            } else {
                last_change_sequence_number + 1
            };
            reader_proxy.increment_heartbeat_count();
    
            let heartbeat = Heartbeat::new(
                submessage_elements::EntityId(*reader_proxy.remote_reader_guid().entity_id()),
                submessage_elements::EntityId(*writer_guid.entity_id()),
                submessage_elements::SequenceNumber(first_sn),
                submessage_elements::SequenceNumber(last_change_sequence_number),
                submessage_elements::Count(*reader_proxy.heartbeat_count()),
                false,
                false,
                Endianness::LittleEndian,
            );
    
            submessages.push(RtpsSubmessage::Heartbeat(heartbeat));
    
            
            reader_proxy.time_last_sent_data_reset();
            
            Some(submessages)
        } else {
            None
        }
    }
    
    fn run_waiting_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            StatefulWriterBehavior::process_repair_message(reader_proxy, writer_guid, received_message);
            reader_proxy.time_nack_received_reset();
        }
    }
    
    fn run_must_repair_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            StatefulWriterBehavior::process_repair_message(reader_proxy, writer_guid, received_message);
        }
    }
    
    fn process_repair_message(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, received_message: &RtpsMessage) {
        let guid_prefix = *received_message.header().guid_prefix();
    
        for submessage in received_message.submessages().iter() {
            if let RtpsSubmessage::AckNack(acknack) = submessage {
                let reader_guid = GUID::new(guid_prefix, acknack.reader_id().0);
                if reader_guid == *reader_proxy.remote_reader_guid() &&
                   writer_guid.entity_id() == &acknack.writer_id().0 &&
                   &acknack.count().0 > reader_proxy.nack_received() {
                    reader_proxy.acked_changes_set(*acknack.reader_sn_state().base() - 1);
                    reader_proxy.requested_changes_set(acknack.reader_sn_state().set().clone());
                    reader_proxy.nack_received_set(acknack.count().0);
                }
            }
        }
    }
    
    fn run_repairing_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache) -> Vec<RtpsSubmessage> {
        // This state is only valid if there are requested changes
        assert!(!reader_proxy.requested_changes().is_empty());
    
        let mut submessages = Vec::with_capacity(2); // TODO: Pre-allocate with right size
    
        let endianness = Endianness::LittleEndian;
        let time = Time::now();
        let infots = InfoTs::new(Some(submessage_elements::Timestamp(time)), endianness);
        submessages.push(RtpsSubmessage::InfoTs(infots));
    
        while let Some(next_requested_seq_num) = reader_proxy.next_requested_change() {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_requested_seq_num)
            {
                let change_kind = *cache_change.change_kind();
                let mut inline_qos = submessage_elements::ParameterList::new();
                inline_qos.push(StatusInfo::from(change_kind));
    
                let payload = match change_kind {
                    ChangeKind::Alive => {
                        inline_qos.push(KeyHash(*cache_change.instance_handle()));
                        Payload::Data(cache_change.data_value().unwrap().to_vec())
                    },
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
                        Payload::Key(cache_change.instance_handle().to_vec())
                    }
                };

                let data = Data::new(
                    Endianness::LittleEndian.into(),
                    submessage_elements::EntityId(*reader_proxy.remote_reader_guid().entity_id()),
                    submessage_elements::EntityId(*writer_guid.entity_id()),
                    submessage_elements::SequenceNumber(*cache_change.sequence_number()),
                    Some(inline_qos), 
                    payload,
                );
    
                submessages.push(RtpsSubmessage::Data(data));
            } else {
                let gap = Gap::new(
                    submessage_elements::EntityId(*reader_proxy.remote_reader_guid().entity_id()), 
                    submessage_elements::EntityId(*writer_guid.entity_id()),
                    submessage_elements::SequenceNumber(next_requested_seq_num),
                    Endianness::LittleEndian);
    
                submessages.push(RtpsSubmessage::Gap(gap));
            }
        }
    
        submessages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeKind, TopicKind, ReliabilityKind, Locator};
    use crate::behavior::types::constants::DURATION_ZERO;
    use crate::types::constants::{
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, 
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, };
    use crate::cache::CacheChange;
    use crate::messages::{AckNack};
    use crate::messages::submessage_elements::SequenceNumberSet;
    use crate::stateful_writer::StatefulWriter;

    use std::thread::sleep;

    #[test]
    fn run_announcing_state_valid_data() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number  = 2;

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let heartbeat_period = Duration::from_millis(200);

        // Reset time and check that no heartbeat is sent immediatelly after
        reader_proxy.time_last_sent_data_reset();
        let message = StatefulWriterBehavior::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period);
        assert_eq!(message, None);

        // Wait for heaartbeat period and check the heartbeat message
        sleep(heartbeat_period.into());
        let submessages = StatefulWriterBehavior::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period).unwrap();

        assert_eq!(submessages.len(), 2);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
            assert_eq!(heartbeat_message.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat_message.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat_message.first_sn().0, 1);
            assert_eq!(heartbeat_message.last_sn().0, 2);
            assert_eq!(heartbeat_message.count().0, 1);
            assert_eq!(heartbeat_message.is_final(), false);
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_announcing_state_multiple_data_combinations() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let heartbeat_period = DURATION_ZERO;
        
        // Test no data in the history cache and no changes written
        let no_change_sequence_number = 0;
        let submessages = StatefulWriterBehavior::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, no_change_sequence_number, heartbeat_period).unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &submessages[1] {
            assert_eq!(heartbeat.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat.first_sn().0, 1);
            assert_eq!(heartbeat.last_sn().0, 0);
            assert_eq!(heartbeat.count().0, 1);
            assert_eq!(heartbeat.is_final(), false);
        } else {
            assert!(false);
        }

        // Test no data in the history cache and two changes written
        let two_changes_sequence_number = 2;
        let submessages = StatefulWriterBehavior::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, two_changes_sequence_number, heartbeat_period).unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &submessages[1] {
            assert_eq!(heartbeat.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat.first_sn().0, 3);
            assert_eq!(heartbeat.last_sn().0, 2);
            assert_eq!(heartbeat.count().0, 2);
            assert_eq!(heartbeat.is_final(), false);
        } else {
            assert!(false);
        }

        // Test two changes in the history cache and two changes written
        let instance_handle = [1;16];
        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);

        let submessages = StatefulWriterBehavior::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, two_changes_sequence_number, heartbeat_period).unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &submessages[1] {
            assert_eq!(heartbeat.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat.first_sn().0, 1);
            assert_eq!(heartbeat.last_sn().0, 2);
            assert_eq!(heartbeat.count().0, 3);
            assert_eq!(heartbeat.is_final(), false);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn process_repair_message_acknowledged_and_requests() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        
        let acknack = AckNack::new(
            submessage_elements::EntityId(*remote_reader_guid.entity_id()),
           submessage_elements::EntityId(*writer_guid.entity_id()),
           submessage_elements::SequenceNumberSet::from_set(vec![3, 5, 6].iter().cloned().collect()),
           submessage_elements::Count(1),
            true,
            Endianness::LittleEndian);
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), vec![RtpsSubmessage::AckNack(acknack)]);

        StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        assert_eq!(reader_proxy.acked_changes(), 2);
        assert_eq!(reader_proxy.requested_changes(), vec![3, 5, 6].iter().cloned().collect());
    }

    #[test]
    fn process_repair_message_different_conditions() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);

        // Test message with different reader guid
        let mut submessages = Vec::new();
        let other_reader_guid = GUID::new([9;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let acknack = AckNack::new(
            submessage_elements::EntityId(*other_reader_guid.entity_id()),
           submessage_elements::EntityId(*writer_guid.entity_id()),
           submessage_elements::SequenceNumberSet::from_set(vec![3, 5, 6].iter().cloned().collect()),
           submessage_elements::Count(1),
            true,
            Endianness::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));
        let received_message = RtpsMessage::new(*other_reader_guid.prefix(), submessages);
        StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);
        
        // Verify that message was ignored
        // assert_eq!(reader_proxy.highest_sequence_number_acknowledged, 0);
        assert!(reader_proxy.requested_changes().is_empty());

        // Test message with different writer guid
        let mut submessages = Vec::new();
        let acknack = AckNack::new(
            submessage_elements::EntityId(*remote_reader_guid.entity_id()),
            submessage_elements::EntityId(ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
           SequenceNumberSet::from_set(vec![3, 5, 6].iter().cloned().collect()),
           submessage_elements::Count(1),
            true,
            Endianness::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);

        StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        // Verify that message was ignored
        assert_eq!(reader_proxy.acked_changes(), 0);
        assert!(reader_proxy.requested_changes().is_empty());


        // Test duplicate acknack message
        let mut submessages = Vec::new();
        let acknack = AckNack::new(
            submessage_elements::EntityId(*remote_reader_guid.entity_id()),
            submessage_elements::EntityId(*writer_guid.entity_id()),
           SequenceNumberSet::from_set(vec![3, 5, 6].iter().cloned().collect()),
           submessage_elements::Count(1),
            true,
            Endianness::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);

        StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        // Verify message was correctly processed
        assert_eq!(reader_proxy.acked_changes(), 2);
        assert_eq!(reader_proxy.requested_changes(), vec![3, 5, 6].iter().cloned().collect());

        // Clear the requested sequence numbers and reprocess the message
        while  reader_proxy.next_requested_change() != None {
            // do nothing
        }
        StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        // Verify that the requested sequence numbers remain empty
        assert!(reader_proxy.requested_changes().is_empty());
    }

    #[test]
    fn process_repair_message_only_acknowledged() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);

        let mut submessages = Vec::new();
        let acknack = AckNack::new(
            submessage_elements::EntityId(*remote_reader_guid.entity_id()),
            submessage_elements::EntityId(*writer_guid.entity_id()),
           SequenceNumberSet::new(5, vec![].iter().cloned().collect()),
           submessage_elements::Count(1),
            true,
            Endianness::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));

        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);
        StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        assert_eq!(reader_proxy.acked_changes(), 4);
        assert!(reader_proxy.requested_changes().is_empty());
    }

    // #[test]
    fn run_pushing_state_only_data_messages() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number  = 2;

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let submessages = StatefulWriterBehavior::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn().0, 1);
            assert_eq!(data_message_1.serialized_payload(), &Some(submessage_elements::SerializedData(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn().0, 2);
            assert_eq!(data_message_2.serialized_payload(), &Some(submessage_elements::SerializedData(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_pushing_state_only_gap_message() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let history_cache = HistoryCache::new();

        // Don't add any change to the history cache so that gap message has to be sent
        // let instance_handle = [1;16];

        // let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, None, Some(vec![1,2,3]));
        // let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, None, Some(vec![2,3,4]));
        // history_cache.add_change(cache_change_seq1);
        // history_cache.add_change(cache_change_seq2);

        let last_change_sequence_number  = 2;

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let submessages = StatefulWriterBehavior::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Gap(gap_message_1) = &submessages[1] {
            assert_eq!(gap_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_1.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_1.gap_start().0, 1);
        } else {
            panic!("Wrong message type");
        };
        if let RtpsSubmessage::Gap(gap_message_2) = &submessages[2] {
            assert_eq!(gap_message_2.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_2.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_2.gap_start().0, 2);
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_pushing_state_gap_and_data_message() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        // Add one change to the history cache so that data and gap messages have to be sent
        let instance_handle = [1;16];

        // let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
        // history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);

        let last_change_sequence_number  = 2;

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let submessages = StatefulWriterBehavior::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Gap(gap_message) = &submessages[1] {
            assert_eq!(gap_message.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message.gap_start().0, 1);
        } else {
            panic!("Wrong message type");
        };
        if let RtpsSubmessage::Data(data_message) = &submessages[2] {
            assert_eq!(data_message.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message.writer_sn().0, 2);
            assert_eq!(data_message.serialized_payload(), &Some(submessage_elements::SerializedData(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };
    }



    #[test]
    fn run_repairing_state_only_data_messages() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        reader_proxy.requested_changes_set(vec![1, 2].iter().cloned().collect());

        let submessages = StatefulWriterBehavior::run_repairing_state(&mut reader_proxy, &writer_guid, &history_cache);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn().0, 1);
            assert_eq!(data_message_1.serialized_payload(), &Some(submessage_elements::SerializedData(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn().0, 2);
            assert_eq!(data_message_2.serialized_payload(), &Some(submessage_elements::SerializedData(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_repairing_state_only_gap_messages() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        reader_proxy.requested_changes_set(vec![1, 2].iter().cloned().collect());

        let submessages = StatefulWriterBehavior::run_repairing_state(&mut reader_proxy, &writer_guid, &history_cache);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Gap(gap_message_1) = &submessages[1] {
            assert_eq!(gap_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_1.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_1.gap_start().0, 1);
        } else {
            panic!("Wrong message type");
        };
        if let RtpsSubmessage::Gap(gap_message_2) = &submessages[2] {
            assert_eq!(gap_message_2.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_2.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_2.gap_start().0, 2);
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_best_effort_reader_proxy() {
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new([1;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        let last_change_sequence_number = 0;

        assert!(StatefulWriterBehavior::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).is_none());

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number = 2;

        let submessages = StatefulWriterBehavior::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).unwrap();
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn().0, 1);
            assert_eq!(data_message_1.serialized_payload(), &Some(submessage_elements::SerializedData(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn().0, 2);
            assert_eq!(data_message_2.serialized_payload(), &Some(submessage_elements::SerializedData(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };

        assert!(StatefulWriterBehavior::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).is_none());
    }

    #[test]
    fn run_reliable_reader_proxy() {
        let heartbeat_period = Duration::from_millis(200);
        let nack_response_delay = Duration::from_millis(200);
        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new([1;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        let last_change_sequence_number = 0;

        // Check that immediately after creation no message is sent
        assert!(StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).is_none());

        // Add two changes to the history cache and check that two data messages are sent
        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number = 2;

        let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).unwrap();
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn().0, 1);
            assert_eq!(data_message_1.serialized_payload(), &Some(submessage_elements::SerializedData(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn().0, 2);
            assert_eq!(data_message_2.serialized_payload(), &Some(submessage_elements::SerializedData(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };

        // Check that immediately after sending the data nothing else is sent
        assert!(StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).is_none());

        // Check that a heartbeat is sent after the heartbeat period
        sleep(heartbeat_period.into());

        let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).unwrap();
        assert_eq!(submessages.len(), 2);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
            assert_eq!(heartbeat_message.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat_message.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat_message.first_sn().0, 1);
            assert_eq!(heartbeat_message.last_sn().0, 2);
            assert_eq!(heartbeat_message.count().0, 1);
            assert_eq!(heartbeat_message.is_final(), false);

        } else {
            panic!("Wrong message type");
        };

        // Check that if a sample is requested it gets sent after the nack_response_delay. In this case it comes together with a heartbeat
        let acknack = AckNack::new(
            submessage_elements::EntityId(*remote_reader_guid.entity_id()),
            submessage_elements::EntityId(*writer_guid.entity_id()),
           SequenceNumberSet::new(1, vec![2].iter().cloned().collect()),
           submessage_elements::Count(1),
           true,
           Endianness::LittleEndian);
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), vec![RtpsSubmessage::AckNack(acknack)]);

        let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, Some(&received_message));
        assert!(submessages.is_none());

        sleep(nack_response_delay.into());

        let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, Some(&received_message)).unwrap();
        assert_eq!(submessages.len(), 4);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
            assert_eq!(heartbeat_message.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat_message.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat_message.first_sn().0, 1);
            assert_eq!(heartbeat_message.last_sn().0, 2);
            assert_eq!(heartbeat_message.count().0, 2);
            assert_eq!(heartbeat_message.is_final(), false);
        }
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[2] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[3] {
            assert_eq!(data_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id().0, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn().0, 2);
            assert_eq!(data_message_1.serialized_payload(), &Some(submessage_elements::SerializedData(vec![2, 3, 4])));

        } else {
            panic!("Wrong message type");
        };

    }

    #[test]
    fn best_effort_stateful_writer_run() {
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

        let reader_guid = GUID::new([1;12], ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);

        writer.matched_reader_add(reader_proxy);

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

        // let reader_proxy = writer.matched_reader_lookup(& reader_guid).unwrap();
        let writer_data = writer.run(&reader_guid, None).unwrap();
        assert_eq!(writer_data.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
            assert_eq!(data_message_1.reader_id().0, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id().0, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn().0, 1);
            assert_eq!(data_message_1.serialized_payload(), &Some(submessage_elements::SerializedData(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
            assert_eq!(data_message_2.reader_id().0, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id().0, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn().0, 2);
            assert_eq!(data_message_2.serialized_payload(), &Some(submessage_elements::SerializedData(vec![4, 5, 6])));
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        let writer_data = writer.run(&reader_guid, None);
        assert_eq!(writer_data.is_none(), true);
    }

}