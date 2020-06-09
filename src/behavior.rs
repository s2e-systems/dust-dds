use crate::types::{GUID, SequenceNumber};
use crate::behavior_types::Duration;
use crate::cache::HistoryCache;
use crate::serdes::EndianessFlag;
use crate::messages::{RtpsSubmessage, Heartbeat, InfoTs};
use crate::messages::types::Time;
use crate::stateful_writer::ReaderProxy;

pub fn run_announcing_state(reader_proxy: &mut ReaderProxy, writer_guid: &GUID, history_cache: &HistoryCache,  last_change_sequence_number: SequenceNumber, heartbeat_period: Duration) -> Option<Vec<RtpsSubmessage>> {

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SequenceNumber, ChangeKind, GuidPrefix};
    use crate::behavior_types::constants::DURATION_ZERO;
    use crate::messages::types::Count;
    use crate::types::constants::{ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR};
    use crate::cache::CacheChange;
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
        let message = run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period);
        assert_eq!(message, None);

        // Wait for heaartbeat period and check the heartbeat message
        sleep(heartbeat_period.into());
        let submessages = run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period).unwrap();

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
        let submessages = run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, no_change_sequence_number, heartbeat_period).unwrap();
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
        let submessages = run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, two_changes_sequence_number, heartbeat_period).unwrap();
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

        let submessages = run_announcing_state(&mut reader_proxy, &writer_guid, &history_cache, two_changes_sequence_number, heartbeat_period).unwrap();
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
}