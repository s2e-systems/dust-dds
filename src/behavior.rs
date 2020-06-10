use std::convert::{TryInto, TryFrom};

use crate::types::{GUID, SequenceNumber, ChangeKind};
use crate::behavior_types::Duration;
use crate::cache::{HistoryCache, CacheChange};
use crate::serdes::EndianessFlag;
use crate::messages::submessage_elements::{Parameter, ParameterList};
use crate::messages::{RtpsMessage, RtpsSubmessage, Heartbeat, InfoTs, Data, Gap, Payload};
use crate::messages::types::Time;
use crate::stateless_writer::ReaderLocator;
use crate::stateful_writer::ReaderProxy;
use crate::serialized_payload::SerializedPayload;
use crate::inline_qos_types::{KeyHash, StatusInfo};
use crate::types::constants::ENTITYID_UNKNOWN;

pub struct StatefulWriterBehaviour {}

impl StatefulWriterBehaviour {
    pub fn run_best_effort(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Option<Vec<RtpsSubmessage>> {
        if !reader_proxy.unsent_changes(last_change_sequence_number).is_empty() {
            Some(StatefulWriterBehaviour::run_pushing_state(reader_proxy, writer_guid, history_cache, last_change_sequence_number))
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
            Some(StatefulWriterBehaviour::run_pushing_state(reader_proxy, writer_guid, history_cache, last_change_sequence_number))
        } else if !reader_proxy.unacked_changes(last_change_sequence_number).is_empty() {
            StatefulWriterBehaviour::run_announcing_state(reader_proxy, writer_guid, history_cache, last_change_sequence_number, heartbeat_period)
        } else {
            None
        };
    
        let repairing_submessages = if reader_proxy.requested_changes().is_empty() {
            StatefulWriterBehaviour::run_waiting_state(reader_proxy, writer_guid, received_message);
            None // No data is sent by the waiting state
        } else {
            StatefulWriterBehaviour::run_must_repair_state(reader_proxy, writer_guid, received_message);
            if reader_proxy.duration_since_nack_received() > nack_response_delay {
                Some(StatefulWriterBehaviour::run_repairing_state(reader_proxy, writer_guid, history_cache))
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
    
        let endianness = EndianessFlag::LittleEndian;
        let mut submessages = Vec::with_capacity(2); // TODO: Probably can be preallocated with the correct size
    
        let time = Time::now();
        let infots = InfoTs::new(Some(time), endianness);
        submessages.push(RtpsSubmessage::InfoTs(infots));
    
        while let Some(next_unsent_seq_num) = reader_proxy.next_unsent_change(last_change_sequence_number) {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_unsent_seq_num)
            {
                let change_kind = *cache_change.change_kind();
    
                let mut parameter = Vec::new();
    
                let payload = match change_kind {
                    ChangeKind::Alive => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        parameter.push(Parameter::new(KeyHash(*cache_change.instance_handle()), endianness));
                        Payload::Data(SerializedPayload(cache_change.data().unwrap().to_vec()))
                    },
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        Payload::Key(SerializedPayload(cache_change.instance_handle().to_vec()))
                    }
                };
                let inline_qos_parameter_list = ParameterList::new(parameter);
                let data = Data::new(
                    EndianessFlag::LittleEndian.into(),
                    *reader_proxy.remote_reader_guid().entity_id(),
                    *writer_guid.entity_id(),
                    *cache_change.sequence_number(),
                    Some(inline_qos_parameter_list), 
                    payload,
                );
    
                submessages.push(RtpsSubmessage::Data(data));
            } else {
                let gap = Gap::new(
                    *reader_proxy.remote_reader_guid().entity_id(), 
                    *writer_guid.entity_id(),
                    next_unsent_seq_num,
                    EndianessFlag::LittleEndian);
    
                submessages.push(RtpsSubmessage::Gap(gap));
            }
        }
    
        submessages
    }

    fn run_announcing_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache,  last_change_sequence_number: SequenceNumber, heartbeat_period: Duration) -> Option<Vec<RtpsSubmessage>> {

        if reader_proxy.duration_since_last_sent_data() > heartbeat_period {
            let mut submessages = Vec::new();
    
            let time = Time::now();
            let infots = InfoTs::new(Some(time), EndianessFlag::LittleEndian);
    
            submessages.push(RtpsSubmessage::InfoTs(infots));
    
            let first_sn = if let Some(seq_num) = history_cache.get_seq_num_min() {
                seq_num
            } else {
                last_change_sequence_number + 1
            };
            reader_proxy.increment_heartbeat_count();
    
            let heartbeat = Heartbeat::new(
                *reader_proxy.remote_reader_guid().entity_id(),
                *writer_guid.entity_id(),
                first_sn,
                last_change_sequence_number,
                *reader_proxy.heartbeat_count(),
                false,
                false,
                EndianessFlag::LittleEndian,
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
            StatefulWriterBehaviour::process_repair_message(reader_proxy, writer_guid, received_message);
            reader_proxy.time_nack_received_reset();
        }
    }
    
    fn run_must_repair_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            StatefulWriterBehaviour::process_repair_message(reader_proxy, writer_guid, received_message);
        }
    }
    
    fn process_repair_message(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, received_message: &RtpsMessage) {
        let guid_prefix = *received_message.header().guid_prefix();
    
        for submessage in received_message.submessages().iter() {
            if let RtpsSubmessage::AckNack(acknack) = submessage {
                let reader_guid = GUID::new(guid_prefix, *acknack.reader_id());
                if reader_guid == *reader_proxy.remote_reader_guid() &&
                   writer_guid.entity_id() == acknack.writer_id() &&
                   *acknack.count() > *reader_proxy.nack_received() {
                    reader_proxy.acked_changes_set(*acknack.reader_sn_state().base() - 1);
                    reader_proxy.requested_changes_set(acknack.reader_sn_state().set().clone());
                    reader_proxy.nack_received_set(*acknack.count());
                }
            }
        }
    }
    
    fn run_repairing_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache) -> Vec<RtpsSubmessage> {
        // This state is only valid if there are requested changes
        assert!(!reader_proxy.requested_changes().is_empty());
    
        let mut submessages = Vec::with_capacity(2); // TODO: Pre-allocate with right size
    
        let endianness = EndianessFlag::LittleEndian;
        let time = Time::now();
        let infots = InfoTs::new(Some(time), endianness);
        submessages.push(RtpsSubmessage::InfoTs(infots));
    
        while let Some(next_requested_seq_num) = reader_proxy.next_requested_change() {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_requested_seq_num)
            {
                let change_kind = *cache_change.change_kind();
    
                let mut parameter = Vec::new();
    
                let payload = match change_kind {
                    ChangeKind::Alive => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        parameter.push(Parameter::new(KeyHash(*cache_change.instance_handle()), endianness));
                        Payload::Data(SerializedPayload(cache_change.data().unwrap().to_vec()))
                    },
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        Payload::Key(SerializedPayload(cache_change.instance_handle().to_vec()))
                    }
                };
                let inline_qos_parameter_list = ParameterList::new(parameter);
                let data = Data::new(
                    EndianessFlag::LittleEndian.into(),
                    *reader_proxy.remote_reader_guid().entity_id(),
                    *writer_guid.entity_id(),
                    *cache_change.sequence_number(),
                    Some(inline_qos_parameter_list), 
                    payload,
                );
    
                submessages.push(RtpsSubmessage::Data(data));
            } else {
                let gap = Gap::new(
                    *reader_proxy.remote_reader_guid().entity_id(), 
                    *writer_guid.entity_id(),
                    next_requested_seq_num,
                    EndianessFlag::LittleEndian);
    
                submessages.push(RtpsSubmessage::Gap(gap));
            }
        }
    
        submessages
    }
}

pub struct StatelessWriterBehavior {}

impl StatelessWriterBehavior{
    pub fn run_best_effort(reader_locator: &mut ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Option<Vec<RtpsSubmessage>> {
        if !reader_locator.unsent_changes(last_change_sequence_number).is_empty() {
            Some(StatelessWriterBehavior::run_pushing_state(reader_locator, writer_guid, history_cache, last_change_sequence_number))
        } else {
            None
        }
    }

    fn run_pushing_state(reader_locator: &mut ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Vec<RtpsSubmessage> {

        // This state is only valid if there are unsent changes
        assert!(!reader_locator.unsent_changes(last_change_sequence_number).is_empty());
    
        let endianness = EndianessFlag::LittleEndian;
        let mut submessages = Vec::with_capacity(2); // TODO: Probably can be preallocated with the correct size
    
        let time = Time::now();
        let infots = InfoTs::new(Some(time), endianness);
        submessages.push(RtpsSubmessage::InfoTs(infots));
    
        while let Some(next_unsent_seq_num) = reader_locator.next_unsent_change(last_change_sequence_number) {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_unsent_seq_num)
            {
                let change_kind = *cache_change.change_kind();
    
                let mut parameter = Vec::new();
    
                let payload = match change_kind {
                    ChangeKind::Alive => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        parameter.push(Parameter::new(KeyHash(*cache_change.instance_handle()), endianness));
                        Payload::Data(SerializedPayload(cache_change.data().unwrap().to_vec()))
                    },
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        Payload::Key(SerializedPayload(cache_change.instance_handle().to_vec()))
                    }
                };
                let inline_qos_parameter_list = ParameterList::new(parameter);
                let data = Data::new(
                    EndianessFlag::LittleEndian.into(),
                    ENTITYID_UNKNOWN,
                    *writer_guid.entity_id(),
                    *cache_change.sequence_number(),
                    Some(inline_qos_parameter_list), 
                    payload,
                );
    
                submessages.push(RtpsSubmessage::Data(data));
            } else {
                let gap = Gap::new(
                    ENTITYID_UNKNOWN, 
                    *writer_guid.entity_id(),
                    next_unsent_seq_num,
                    EndianessFlag::LittleEndian);
    
                submessages.push(RtpsSubmessage::Gap(gap));
            }
        }
    
        submessages
    }


}

pub struct StatelessReaderBehavior {}

impl StatelessReaderBehavior {
    pub fn run_best_effort(history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>){
        StatelessReaderBehavior::run_waiting_state(history_cache, received_message);
    }

    pub fn run_waiting_state(history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) {

        if let Some(received_message) = received_message {
            let guid_prefix = *received_message.header().guid_prefix();
            let mut _source_time = None;

            for submessage in received_message.submessages().iter() {
                if let RtpsSubmessage::Data(data) = submessage {
                    // Check if the message is for this reader and process it if that is the case
                    if data.reader_id() == &ENTITYID_UNKNOWN {

                        let change_kind = StatelessReaderBehavior::change_kind(&data);

                        let key_hash = StatelessReaderBehavior::key_hash(&data).unwrap();
                        
                        let cache_change = CacheChange::new(
                            change_kind,
                            GUID::new(guid_prefix, *data.writer_id() ),
                            key_hash.0,
                            *data.writer_sn(),
                            None,
                            None,
                        );

                        history_cache.add_change(cache_change);
                    }
                }
                else if let RtpsSubmessage::InfoTs(infots) = submessage {
                    _source_time = *infots.get_timestamp();
                }
            }
        }
    }

    fn change_kind(data_submessage: &Data) -> ChangeKind{
        if data_submessage.data_flag().is_set() && !data_submessage.key_flag().is_set() {
            ChangeKind::Alive
        } else if !data_submessage.data_flag().is_set() && data_submessage.key_flag().is_set() {
            let inline_qos = data_submessage.inline_qos().as_ref().unwrap();
            let endianness = data_submessage.endianness_flag().into();
            let status_info = inline_qos.find::<StatusInfo>(endianness).unwrap();           

            ChangeKind::try_from(status_info).unwrap()
        }
        else {
            panic!("Invalid change kind combination")
        }
    }

    fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
        if data_submessage.data_flag().is_set() && !data_submessage.key_flag().is_set() {
            data_submessage.inline_qos().as_ref()?.find::<KeyHash>(data_submessage.endianness_flag().into())
        } else if !data_submessage.data_flag().is_set() && data_submessage.key_flag().is_set() {
            let payload = data_submessage.serialized_payload().as_ref()?; 
            Some(KeyHash(payload.0[0..16].try_into().ok()?))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SequenceNumber, ChangeKind, GuidPrefix, TopicKind, ReliabilityKind, Locator};
    use crate::behavior_types::constants::DURATION_ZERO;
    use crate::messages::types::Count;
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
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number  = SequenceNumber(2);

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let heartbeat_period = Duration::from_millis(200);

        // Reset time and check that no heartbeat is sent immediatelly after
        reader_proxy.time_last_sent_data_reset();
        let message = StatefulWriterBehaviour::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period);
        assert_eq!(message, None);

        // Wait for heaartbeat period and check the heartbeat message
        sleep(heartbeat_period.into());
        let submessages = StatefulWriterBehaviour::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period).unwrap();

        assert_eq!(submessages.len(), 2);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
            assert_eq!(heartbeat_message.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat_message.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat_message.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat_message.last_sn(), &SequenceNumber(2));
            assert_eq!(heartbeat_message.count(), &Count(1));
            assert_eq!(heartbeat_message.is_final(), false);
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_announcing_state_multiple_data_combinations() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let heartbeat_period = DURATION_ZERO;
        
        // Test no data in the history cache and no changes written
        let no_change_sequence_number = SequenceNumber(0);
        let submessages = StatefulWriterBehaviour::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, no_change_sequence_number, heartbeat_period).unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &submessages[1] {
            assert_eq!(heartbeat.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat.last_sn(), &SequenceNumber(0));
            assert_eq!(heartbeat.count(), &Count(1));
            assert_eq!(heartbeat.is_final(), false);
        } else {
            assert!(false);
        }

        // Test no data in the history cache and two changes written
        let two_changes_sequence_number = SequenceNumber(2);
        let submessages = StatefulWriterBehaviour::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, two_changes_sequence_number, heartbeat_period).unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &submessages[1] {
            assert_eq!(heartbeat.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat.first_sn(), &SequenceNumber(3));
            assert_eq!(heartbeat.last_sn(), &SequenceNumber(2));
            assert_eq!(heartbeat.count(), &Count(2));
            assert_eq!(heartbeat.is_final(), false);
        } else {
            assert!(false);
        }

        // Test two changes in the history cache and two changes written
        let instance_handle = [1;16];
        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);

        let submessages = StatefulWriterBehaviour::run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, two_changes_sequence_number, heartbeat_period).unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &submessages[1] {
            assert_eq!(heartbeat.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat.last_sn(), &SequenceNumber(2));
            assert_eq!(heartbeat.count(), &Count(3));
            assert_eq!(heartbeat.is_final(), false);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn process_repair_message_acknowledged_and_requests() {
        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        
        let acknack = AckNack::new(
           *remote_reader_guid.entity_id(),
           *writer_guid.entity_id(),
           SequenceNumberSet::from_set(vec![SequenceNumber(3), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect()),
           Count(1),
            true,
            EndianessFlag::LittleEndian);
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), vec![RtpsSubmessage::AckNack(acknack)]);

        StatefulWriterBehaviour::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        assert_eq!(reader_proxy.acked_changes(), SequenceNumber(2));
        assert_eq!(reader_proxy.requested_changes(), vec![SequenceNumber(3), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect());
    }

    #[test]
    fn process_repair_message_different_conditions() {
        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);

        // Test message with different reader guid
        let mut submessages = Vec::new();
        let other_reader_guid = GUID::new(GuidPrefix([9;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let acknack = AckNack::new(
           *other_reader_guid.entity_id(),
           *writer_guid.entity_id(),
           SequenceNumberSet::from_set(vec![SequenceNumber(3), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect()),
           Count(1),
            true,
            EndianessFlag::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));
        let received_message = RtpsMessage::new(*other_reader_guid.prefix(), submessages);
        StatefulWriterBehaviour::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);
        
        // Verify that message was ignored
        // assert_eq!(reader_proxy.highest_sequence_number_acknowledged, SequenceNumber(0));
        assert!(reader_proxy.requested_changes().is_empty());

        // Test message with different writer guid
        let mut submessages = Vec::new();
        let acknack = AckNack::new(
           *remote_reader_guid.entity_id(),
           ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
           SequenceNumberSet::from_set(vec![SequenceNumber(3), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect()),
           Count(1),
            true,
            EndianessFlag::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);

        StatefulWriterBehaviour::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        // Verify that message was ignored
        assert_eq!(reader_proxy.acked_changes(), SequenceNumber(0));
        assert!(reader_proxy.requested_changes().is_empty());


        // Test duplicate acknack message
        let mut submessages = Vec::new();
        let acknack = AckNack::new(
           *remote_reader_guid.entity_id(),
           *writer_guid.entity_id(),
           SequenceNumberSet::from_set(vec![SequenceNumber(3), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect()),
           Count(1),
            true,
            EndianessFlag::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);

        StatefulWriterBehaviour::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        // Verify message was correctly processed
        assert_eq!(reader_proxy.acked_changes(), SequenceNumber(2));
        assert_eq!(reader_proxy.requested_changes(), vec![SequenceNumber(3), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect());

        // Clear the requested sequence numbers and reprocess the message
        while  reader_proxy.next_requested_change() != None {
            // do nothing
        }
        StatefulWriterBehaviour::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        // Verify that the requested sequence numbers remain empty
        assert!(reader_proxy.requested_changes().is_empty());
    }

    #[test]
    fn process_repair_message_only_acknowledged() {
        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);

        let mut submessages = Vec::new();
        let acknack = AckNack::new(
           *remote_reader_guid.entity_id(),
           *writer_guid.entity_id(),
           SequenceNumberSet::new(SequenceNumber(5), vec![].iter().cloned().collect()),
           Count(1),
            true,
            EndianessFlag::LittleEndian);
        submessages.push(RtpsSubmessage::AckNack(acknack));

        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);
        StatefulWriterBehaviour::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

        assert_eq!(reader_proxy.acked_changes(), SequenceNumber(4));
        assert!(reader_proxy.requested_changes().is_empty());
    }

    // #[test]
    fn run_pushing_state_only_data_messages() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number  = SequenceNumber(2);

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let submessages = StatefulWriterBehaviour::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_pushing_state_only_gap_message() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let history_cache = HistoryCache::new();

        // Don't add any change to the history cache so that gap message has to be sent
        // let instance_handle = [1;16];

        // let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        // let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        // history_cache.add_change(cache_change_seq1);
        // history_cache.add_change(cache_change_seq2);

        let last_change_sequence_number  = SequenceNumber(2);

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let submessages = StatefulWriterBehaviour::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Gap(gap_message_1) = &submessages[1] {
            assert_eq!(gap_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_1.gap_start(), &SequenceNumber(1));
        } else {
            panic!("Wrong message type");
        };
        if let RtpsSubmessage::Gap(gap_message_2) = &submessages[2] {
            assert_eq!(gap_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_2.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_2.gap_start(), &SequenceNumber(2));
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_pushing_state_gap_and_data_message() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        // Add one change to the history cache so that data and gap messages have to be sent
        let instance_handle = [1;16];

        // let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        // history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);

        let last_change_sequence_number  = SequenceNumber(2);

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let submessages = StatefulWriterBehaviour::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Gap(gap_message) = &submessages[1] {
            assert_eq!(gap_message.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message.gap_start(), &SequenceNumber(1));
        } else {
            panic!("Wrong message type");
        };
        if let RtpsSubmessage::Data(data_message) = &submessages[2] {
            assert_eq!(data_message.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message.serialized_payload(), &Some(SerializedPayload(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };
    }



    #[test]
    fn run_repairing_state_only_data_messages() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        reader_proxy.requested_changes_set(vec![SequenceNumber(1), SequenceNumber(2)].iter().cloned().collect());

        let submessages = StatefulWriterBehaviour::run_repairing_state(&mut reader_proxy, &writer_guid, &history_cache);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_repairing_state_only_gap_messages() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new(GuidPrefix([1,2,3,4,5,6,7,8,9,10,11,12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        reader_proxy.requested_changes_set(vec![SequenceNumber(1), SequenceNumber(2)].iter().cloned().collect());

        let submessages = StatefulWriterBehaviour::run_repairing_state(&mut reader_proxy, &writer_guid, &history_cache);
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Gap(gap_message_1) = &submessages[1] {
            assert_eq!(gap_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_1.gap_start(), &SequenceNumber(1));
        } else {
            panic!("Wrong message type");
        };
        if let RtpsSubmessage::Gap(gap_message_2) = &submessages[2] {
            assert_eq!(gap_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(gap_message_2.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(gap_message_2.gap_start(), &SequenceNumber(2));
        } else {
            panic!("Wrong message type");
        };
    }

    #[test]
    fn run_best_effort_reader_proxy() {
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        let last_change_sequence_number = SequenceNumber(0);

        assert!(StatefulWriterBehaviour::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).is_none());

        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number = SequenceNumber(2);

        let submessages = StatefulWriterBehaviour::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).unwrap();
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };

        assert!(StatefulWriterBehaviour::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).is_none());
    }

    #[test]
    fn run_reliable_reader_proxy() {
        let heartbeat_period = Duration::from_millis(200);
        let nack_response_delay = Duration::from_millis(200);
        let writer_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut history_cache = HistoryCache::new();

        let remote_reader_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        let last_change_sequence_number = SequenceNumber(0);

        // Check that immediately after creation no message is sent
        assert!(StatefulWriterBehaviour::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).is_none());

        // Add two changes to the history cache and check that two data messages are sent
        let instance_handle = [1;16];

        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(1), None, Some(vec![1,2,3]));
        let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, SequenceNumber(2), None, Some(vec![2,3,4]));
        history_cache.add_change(cache_change_seq1);
        history_cache.add_change(cache_change_seq2);
        let last_change_sequence_number = SequenceNumber(2);

        let submessages = StatefulWriterBehaviour::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).unwrap();
        assert_eq!(submessages.len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![2, 3, 4])));
        } else {
            panic!("Wrong message type");
        };

        // Check that immediately after sending the data nothing else is sent
        assert!(StatefulWriterBehaviour::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).is_none());

        // Check that a heartbeat is sent after the heartbeat period
        sleep(heartbeat_period.into());

        let submessages = StatefulWriterBehaviour::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).unwrap();
        assert_eq!(submessages.len(), 2);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
            assert_eq!(heartbeat_message.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat_message.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat_message.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat_message.last_sn(), &SequenceNumber(2));
            assert_eq!(heartbeat_message.count(), &Count(1));
            assert_eq!(heartbeat_message.is_final(), false);

        } else {
            panic!("Wrong message type");
        };

        // Check that if a sample is requested it gets sent after the nack_response_delay. In this case it comes together with a heartbeat
        let acknack = AckNack::new(
           *remote_reader_guid.entity_id(),
           *writer_guid.entity_id(),
           SequenceNumberSet::new(SequenceNumber(1), vec![SequenceNumber(2)].iter().cloned().collect()),
           Count(1),
           true,
           EndianessFlag::LittleEndian);
        let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), vec![RtpsSubmessage::AckNack(acknack)]);

        let submessages = StatefulWriterBehaviour::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, Some(&received_message));
        assert!(submessages.is_none());

        sleep(nack_response_delay.into());

        let submessages = StatefulWriterBehaviour::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, Some(&received_message)).unwrap();
        assert_eq!(submessages.len(), 4);
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
            assert_eq!(heartbeat_message.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(heartbeat_message.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(heartbeat_message.first_sn(), &SequenceNumber(1));
            assert_eq!(heartbeat_message.last_sn(), &SequenceNumber(2));
            assert_eq!(heartbeat_message.count(), &Count(2));
            assert_eq!(heartbeat_message.is_final(), false);
        }
        if let RtpsSubmessage::InfoTs(message_1) = &submessages[2] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &submessages[3] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![2, 3, 4])));

        } else {
            panic!("Wrong message type");
        };

    }

    #[test]
    fn best_effort_stateful_writer_run() {
        let mut writer = StatefulWriter::new(
            GUID::new(GuidPrefix([0; 12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0; 16])], 
            vec![],                               
            false,                                
            DURATION_ZERO,                        
            DURATION_ZERO,                        
            DURATION_ZERO,                        
        );

        let reader_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
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

        writer.history_cache().add_change(cache_change_seq1);
        writer.history_cache().add_change(cache_change_seq2);

        // let reader_proxy = writer.matched_reader_lookup(& reader_guid).unwrap();
        let writer_data = writer.run(&reader_guid, None).unwrap();
        assert_eq!(writer_data.submessages().len(), 3);
        if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
            println!("{:?}", message_1);
        } else {
            panic!("Wrong message type");
        }
        if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
            assert_eq!(data_message_1.reader_id(), &ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            assert_eq!(data_message_1.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_1.writer_sn(), &SequenceNumber(1));
            assert_eq!(data_message_1.serialized_payload(), &Some(SerializedPayload(vec![1, 2, 3])));

        } else {
            panic!("Wrong message type");
        };

        if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
            assert_eq!(data_message_2.reader_id(), &ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
            assert_eq!(data_message_2.writer_id(), &ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
            assert_eq!(data_message_2.writer_sn(), &SequenceNumber(2));
            assert_eq!(data_message_2.serialized_payload(), &Some(SerializedPayload(vec![4, 5, 6])));
        } else {
            panic!("Wrong message type");
        };

        // Test that nothing more is sent after the first time
        let writer_data = writer.run(&reader_guid, None);
        assert_eq!(writer_data.is_none(), true);
    }

}