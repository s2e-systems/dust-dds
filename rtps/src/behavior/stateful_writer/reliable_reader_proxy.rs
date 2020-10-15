use std::time::Instant;
use std::convert::TryInto;
use std::collections::{BTreeSet, VecDeque};

use crate::types::{SequenceNumber, GuidPrefix, EntityId};
use crate::structure::HistoryCache;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::{Gap, Heartbeat, AckNack};
use crate::behavior::ReaderProxy;
use crate::messages::types::Count;

use crate::behavior::types::Duration;
use crate::behavior::{data_from_cache_change, BEHAVIOR_ENDIANNESS};

use super::stateful_writer::ReaderProxyOps;

pub struct ReliableReaderProxy{
    reader_proxy: ReaderProxy,
    writer_entity_id: EntityId,

    nack_response_delay: Duration,
    heartbeat_period: Duration,

    heartbeat_count: Count,
    time_last_sent_data: Instant,
    time_nack_received: Instant,
    highest_nack_count_received: Count,

    received_messages: VecDeque<(GuidPrefix, RtpsSubmessage)>,
    sent_messages: VecDeque<RtpsSubmessage>
}

impl ReliableReaderProxy {
    pub fn new(reader_proxy: ReaderProxy, writer_entity_id: EntityId, heartbeat_period: Duration, nack_response_delay: Duration) -> Self {
        Self {
            reader_proxy,
            writer_entity_id,
            nack_response_delay,
            heartbeat_period,
            heartbeat_count: 0,
            time_last_sent_data: Instant::now(),
            time_nack_received: Instant::now(),
            highest_nack_count_received: 0,
            received_messages: VecDeque::new(),
            sent_messages: VecDeque::new(),

        }
    }

    fn pushing_state(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        // This state is only valid if there are unsent changes
        debug_assert!(!self.reader_proxy.unsent_changes(last_change_sequence_number).is_empty());
    
        while let Some(next_unsent_seq_num) = self.reader_proxy.next_unsent_change(last_change_sequence_number) {
            self.transition_t4(history_cache, next_unsent_seq_num);
        }
        self.time_last_sent_data_reset();
    }

    fn transition_t4(&mut self, history_cache: &HistoryCache, next_unsent_seq_num: SequenceNumber) {
        if let Some(cache_change) = history_cache
            .changes().iter().find(|cc| cc.sequence_number() == next_unsent_seq_num)
        {
            let reader_id = self.reader_proxy.remote_reader_guid().entity_id();
            let data = data_from_cache_change(cache_change, reader_id);
            self.sent_messages.push_back(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                self.reader_proxy.remote_reader_guid().entity_id(), 
                self.writer_entity_id,
                next_unsent_seq_num,
            BTreeSet::new());

            self.sent_messages.push_back(RtpsSubmessage::Gap(gap));
        }
    }

    fn announcing_state(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber){
        if self.duration_since_last_sent_data() > self.heartbeat_period {
            self.transition_t7(history_cache, last_change_sequence_number);
            self.time_last_sent_data_reset();
        }
    }

    fn transition_t7(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        let first_sn = if let Some(seq_num) = history_cache.get_seq_num_min() {
            seq_num
        } else {
            last_change_sequence_number + 1
        };
        self.increment_heartbeat_count();

        let heartbeat = Heartbeat::new(
            BEHAVIOR_ENDIANNESS,
            self.reader_proxy.remote_reader_guid().entity_id(),
            self.writer_entity_id,
            first_sn,
            last_change_sequence_number,
            self.heartbeat_count,
            false,
            false,
        );

        self.sent_messages.push_back(RtpsSubmessage::Heartbeat(heartbeat));
    }
    
    fn waiting_state(&mut self) {
        let received = self.received_messages.pop_front();
        if let Some((_, RtpsSubmessage::AckNack(acknack))) = received {
            self.transition_t8(acknack);
            self.time_nack_received_reset();
        }
    }
    
    fn transition_t8(&mut self, acknack: AckNack) {
        self.reader_proxy.acked_changes_set(acknack.reader_sn_state().base() - 1);
        self.reader_proxy.requested_changes_set(acknack.reader_sn_state().set().clone());
    }
    
    fn must_repair_state(&mut self) {
        let received = self.received_messages.pop_front();
        if let Some((_, RtpsSubmessage::AckNack(acknack))) = received {
            self.transition_t8(acknack);
        }
    }
    
    fn repairing_state(&mut self, history_cache: &HistoryCache) {
        // This state is only valid if there are requested changes
        debug_assert!(!self.reader_proxy.requested_changes().is_empty());
    
        while let Some(next_requested_seq_num) = self.reader_proxy.next_requested_change() {
            self.transition_t12(history_cache, next_requested_seq_num);
        }
    }

    fn transition_t12(&mut self, history_cache: &HistoryCache, next_requested_seq_num: SequenceNumber) {
        if let Some(cache_change) = history_cache.changes().iter().find(|cc| cc.sequence_number() == next_requested_seq_num)
        {
            let data = data_from_cache_change(cache_change, self.reader_proxy.remote_reader_guid().entity_id());
            self.sent_messages.push_back(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                self.reader_proxy.remote_reader_guid().entity_id(), 
                self.writer_entity_id,
                next_requested_seq_num,
                BTreeSet::new());

            self.sent_messages.push_back(RtpsSubmessage::Gap(gap));
        }
    }

    fn nack_received_set(&mut self, highest_nack_count_received: Count) {
        self.highest_nack_count_received = highest_nack_count_received;
    }

    fn time_last_sent_data_reset(&mut self) {
        self.time_last_sent_data = Instant::now();
    }

    fn time_nack_received_reset(&mut self) {
        self.time_nack_received = Instant::now();
    }

    fn duration_since_last_sent_data(&self) -> Duration {
        self.time_last_sent_data.elapsed().try_into().unwrap()
    }

    fn duration_since_nack_received(&self) -> Duration {
        self.time_nack_received.elapsed().try_into().unwrap()
    }

    fn increment_heartbeat_count(&mut self) {
        self.heartbeat_count += 1;
    }
}

impl ReaderProxyOps for ReliableReaderProxy {
    fn run(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        if self.reader_proxy.unacked_changes(last_change_sequence_number).is_empty() {
            // Idle
        } else if !self.reader_proxy.unsent_changes(last_change_sequence_number).is_empty() {
            self.pushing_state(history_cache, last_change_sequence_number);
        } else if !self.reader_proxy.unacked_changes(last_change_sequence_number).is_empty() {
            self.announcing_state(history_cache, last_change_sequence_number);
        }
    
        if self.reader_proxy.requested_changes().is_empty() {
            self.waiting_state();
        } else {
            self.must_repair_state();
            if self.duration_since_nack_received() > self.nack_response_delay {
                self.repairing_state(history_cache);
            }
        }
    }

    fn push_receive_message(&mut self, src_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) {
        todo!()
    }

    fn is_submessage_destination(&self, src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        todo!()
    }

    fn pop_send_message(&mut self) -> Option<(Vec<crate::types::Locator>, VecDeque<RtpsSubmessage>)> {
        if !self.sent_messages.is_empty() {
            let mut send_message_queue = VecDeque::new();
            std::mem::swap(&mut send_message_queue, &mut self.sent_messages);
            
            let mut locator_list = Vec::new();
            locator_list.extend(self.reader_proxy.unicast_locator_list());
            locator_list.extend(self.reader_proxy.multicast_locator_list());

            Some((locator_list, send_message_queue))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::GUID;
    // use crate::behavior::types::constants::DURATION_ZERO;
    // use crate::types::constants::{
    //     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR };
    // use crate::messages::{AckNack};
    // use crate::messages::receiver::WriterReceiveMessage;
    // use crate::stateful_writer::StatefulWriter;

    // use rust_dds_interface::qos_policy::ResourceLimitsQosPolicy;
    // use std::collections::BTreeSet;
    // use std::thread::sleep;

    // #[test]
    // fn process_repair_message_acknowledged_and_requests() {
    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        
    //     let acknack = AckNack::new(
    //         *remote_reader_guid.entity_id(),
    //        *writer_guid.entity_id(),
    //        3,
    //         vec![3, 5, 6].iter().cloned().collect(),
    //        1,
    //         true,
    //         Endianness::LittleEndian);
    //     let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), vec![RtpsSubmessage::AckNack(acknack)]);

    //     StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

    //     assert_eq!(reader_proxy.acked_changes(), 2);
    //     assert_eq!(reader_proxy.requested_changes(), vec![3, 5, 6].iter().cloned().collect());
    // }

    // #[test]
    // fn process_repair_message_different_conditions() {
    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);

    //     // Test message with different reader guid
    //     let mut submessages = Vec::new();
    //     let other_reader_guid = GUID::new([9;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let acknack = AckNack::new(
    //         *other_reader_guid.entity_id(),
    //        *writer_guid.entity_id(),
    //        3,
    //         vec![3, 5, 6].iter().cloned().collect(),
    //        1,
    //         true,
    //         Endianness::LittleEndian);
    //     submessages.push(RtpsSubmessage::AckNack(acknack));
    //     let received_message = RtpsMessage::new(*other_reader_guid.prefix(), submessages);
    //     StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);
        
    //     // Verify that message was ignored
    //     // assert_eq!(reader_proxy.highest_sequence_number_acknowledged, 0);
    //     assert!(reader_proxy.requested_changes().is_empty());

    //     // Test message with different writer guid
    //     let mut submessages = Vec::new();
    //     let acknack = AckNack::new(
    //         *remote_reader_guid.entity_id(),
    //         ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    //        3, 
    //        vec![5, 6].iter().cloned().collect(),
    //        1,
    //         true,
    //         Endianness::LittleEndian);
    //     submessages.push(RtpsSubmessage::AckNack(acknack));
    //     let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);

    //     StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

    //     // Verify that message was ignored
    //     assert_eq!(reader_proxy.acked_changes(), 0);
    //     assert!(reader_proxy.requested_changes().is_empty());


    //     // Test duplicate acknack message
    //     let mut submessages = Vec::new();
    //     let acknack = AckNack::new(
    //         *remote_reader_guid.entity_id(),
    //         *writer_guid.entity_id(),
    //        3, 
    //         vec![3, 5, 6].iter().cloned().collect(),
    //        1,
    //         true,
    //         Endianness::LittleEndian);
    //     submessages.push(RtpsSubmessage::AckNack(acknack));
    //     let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);

    //     StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

    //     // Verify message was correctly processed
    //     assert_eq!(reader_proxy.acked_changes(), 2);
    //     assert_eq!(reader_proxy.requested_changes(), vec![3, 5, 6].iter().cloned().collect());

    //     // Clear the requested sequence numbers and reprocess the message
    //     while  reader_proxy.next_requested_change() != None {
    //         // do nothing
    //     }
    //     StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

    //     // Verify that the requested sequence numbers remain empty
    //     assert!(reader_proxy.requested_changes().is_empty());
    // }

    // #[test]
    // fn process_repair_message_only_acknowledged() {
    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);

    //     let mut submessages = Vec::new();
    //     let acknack = AckNack::new(
    //         *remote_reader_guid.entity_id(),
    //         *writer_guid.entity_id(),
    //        5, 
    //        vec![].iter().cloned().collect(),
    //        1,
    //         true,
    //         Endianness::LittleEndian);
    //     submessages.push(RtpsSubmessage::AckNack(acknack));

    //     let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), submessages);
    //     StatefulWriterBehavior::process_repair_message(&mut reader_proxy, &writer_guid, &received_message);

    //     assert_eq!(reader_proxy.acked_changes(), 4);
    //     assert!(reader_proxy.requested_changes().is_empty());
    // }

    // // #[test]
    // fn run_pushing_state_only_data_messages() {
    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let history_cache = HistoryCache::new();

    //     let instance_handle = [1;16];

    //     let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
    //     let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
    //     history_cache.add_change(cache_change_seq1);
    //     history_cache.add_change(cache_change_seq2);
    //     let last_change_sequence_number  = 2;

    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

    //     let submessages = StatefulWriterBehavior::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
    //     assert_eq!(submessages.len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
    //         assert_eq!(data_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_1.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_1.writer_sn(), 1);
    //         assert_eq!(data_message_1.serialized_payload(), Some(&vec![1, 2, 3]));

    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
    //         assert_eq!(data_message_2.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_2.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_2.writer_sn(), 2);
    //         assert_eq!(data_message_2.serialized_payload(), Some(&vec![2, 3, 4]));
    //     } else {
    //         panic!("Wrong message type");
    //     };
    // }

    // #[test]
    // fn run_pushing_state_only_gap_message() {
    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let history_cache = HistoryCache::new();

    //     // Don't add any change to the history cache so that gap message has to be sent
    //     // let instance_handle = [1;16];

    //     // let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, None, Some(vec![1,2,3]));
    //     // let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, None, Some(vec![2,3,4]));
    //     // history_cache.add_change(cache_change_seq1);
    //     // history_cache.add_change(cache_change_seq2);

    //     let last_change_sequence_number  = 2;

    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

    //     let submessages = StatefulWriterBehavior::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
    //     assert_eq!(submessages.len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Gap(gap_message_1) = &submessages[1] {
    //         assert_eq!(gap_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(gap_message_1.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(gap_message_1.gap_start(), 1);
    //     } else {
    //         panic!("Wrong message type");
    //     };
    //     if let RtpsSubmessage::Gap(gap_message_2) = &submessages[2] {
    //         assert_eq!(gap_message_2.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(gap_message_2.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(gap_message_2.gap_start(), 2);
    //     } else {
    //         panic!("Wrong message type");
    //     };
    // }

    // #[test]
    // fn run_pushing_state_gap_and_data_message() {
    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let history_cache = HistoryCache::new();

    //     // Add one change to the history cache so that data and gap messages have to be sent
    //     let instance_handle = [1;16];

    //     // let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, None, Some(vec![1,2,3]));
    //     let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
    //     // history_cache.add_change(cache_change_seq1);
    //     history_cache.add_change(cache_change_seq2);

    //     let last_change_sequence_number  = 2;

    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

    //     let submessages = StatefulWriterBehavior::run_pushing_state(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number);
    //     assert_eq!(submessages.len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Gap(gap_message) = &submessages[1] {
    //         assert_eq!(gap_message.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(gap_message.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(gap_message.gap_start(), 1);
    //     } else {
    //         panic!("Wrong message type");
    //     };
    //     if let RtpsSubmessage::Data(data_message) = &submessages[2] {
    //         assert_eq!(data_message.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message.writer_sn(), 2);
    //         assert_eq!(data_message.serialized_payload(), Some(&vec![2, 3, 4]));
    //     } else {
    //         panic!("Wrong message type");
    //     };
    // }



    // #[test]
    // fn run_repairing_state_only_data_messages() {
    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let history_cache = HistoryCache::new();

    //     let instance_handle = [1;16];

    //     let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
    //     let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
    //     history_cache.add_change(cache_change_seq1);
    //     history_cache.add_change(cache_change_seq2);

    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
    //     reader_proxy.requested_changes_set(vec![1, 2].iter().cloned().collect());

    //     let submessages = StatefulWriterBehavior::run_repairing_state(&mut reader_proxy, &writer_guid, &history_cache);
    //     assert_eq!(submessages.len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
    //         assert_eq!(data_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_1.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_1.writer_sn(), 1);
    //         assert_eq!(data_message_1.serialized_payload(), Some(&vec![1, 2, 3]));

    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
    //         assert_eq!(data_message_2.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_2.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_2.writer_sn(), 2);
    //         assert_eq!(data_message_2.serialized_payload(), Some(&vec![2, 3, 4]));
    //     } else {
    //         panic!("Wrong message type");
    //     };
    // }

    // #[test]
    // fn run_repairing_state_only_gap_messages() {
    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let history_cache = HistoryCache::new();

    //     let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
    //     reader_proxy.requested_changes_set(vec![1, 2].iter().cloned().collect());

    //     let submessages = StatefulWriterBehavior::run_repairing_state(&mut reader_proxy, &writer_guid, &history_cache);
    //     assert_eq!(submessages.len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Gap(gap_message_1) = &submessages[1] {
    //         assert_eq!(gap_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(gap_message_1.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(gap_message_1.gap_start(), 1);
    //     } else {
    //         panic!("Wrong message type");
    //     };
    //     if let RtpsSubmessage::Gap(gap_message_2) = &submessages[2] {
    //         assert_eq!(gap_message_2.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(gap_message_2.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(gap_message_2.gap_start(), 2);
    //     } else {
    //         panic!("Wrong message type");
    //     };
    // }

    // #[test]
    // fn run_best_effort_reader_proxy() {
    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let history_cache = HistoryCache::new();

    //     let remote_reader_guid = GUID::new([1;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
    //     let last_change_sequence_number = 0;

    //     assert!(StatefulWriterBehavior::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).is_none());

    //     let instance_handle = [1;16];

    //     let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
    //     let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
    //     history_cache.add_change(cache_change_seq1);
    //     history_cache.add_change(cache_change_seq2);
    //     let last_change_sequence_number = 2;

    //     let submessages = StatefulWriterBehavior::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).unwrap();
    //     assert_eq!(submessages.len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
    //         assert_eq!(data_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_1.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_1.writer_sn(), 1);
    //         assert_eq!(data_message_1.serialized_payload(), Some(&vec![1, 2, 3]));

    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
    //         assert_eq!(data_message_2.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_2.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_2.writer_sn(), 2);
    //         assert_eq!(data_message_2.serialized_payload(), Some(&vec![2, 3, 4]));
    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     assert!(StatefulWriterBehavior::run_best_effort(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number).is_none());
    // }

    // #[test]
    // fn run_reliable_reader_proxy() {
    //     let heartbeat_period = Duration::from_millis(200);
    //     let nack_response_delay = Duration::from_millis(200);
    //     let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let history_cache = HistoryCache::new();

    //     let remote_reader_guid = GUID::new([1;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
    //     let last_change_sequence_number = 0;

    //     // Check that immediately after creation no message is sent
    //     assert!(StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).is_none());

    //     // Add two changes to the history cache and check that two data messages are sent
    //     let instance_handle = [1;16];

    //     let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
    //     let cache_change_seq2 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 2, Some(vec![2,3,4]), None);
    //     history_cache.add_change(cache_change_seq1);
    //     history_cache.add_change(cache_change_seq2);
    //     let last_change_sequence_number = 2;

    //     let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).unwrap();
    //     assert_eq!(submessages.len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Data(data_message_1) = &submessages[1] {
    //         assert_eq!(data_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_1.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_1.writer_sn(), 1);
    //         assert_eq!(data_message_1.serialized_payload(), Some(&vec![1, 2, 3]));

    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     if let RtpsSubmessage::Data(data_message_2) = &submessages[2] {
    //         assert_eq!(data_message_2.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_2.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_2.writer_sn(), 2);
    //         assert_eq!(data_message_2.serialized_payload(), Some(&vec![2, 3, 4]));
    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     // Check that immediately after sending the data nothing else is sent
    //     assert!(StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).is_none());

    //     // Check that a heartbeat is sent after the heartbeat period
    //     sleep(heartbeat_period.into());

    //     let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, None).unwrap();
    //     assert_eq!(submessages.len(), 2);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
    //         assert_eq!(heartbeat_message.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(heartbeat_message.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(heartbeat_message.first_sn(), 1);
    //         assert_eq!(heartbeat_message.last_sn(), 2);
    //         assert_eq!(heartbeat_message.count(), 1);
    //         assert_eq!(heartbeat_message.is_final(), false);

    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     // Check that if a sample is requested it gets sent after the nack_response_delay. In this case it comes together with a heartbeat
    //     let acknack = AckNack::new(
    //         *remote_reader_guid.entity_id(),
    //         *writer_guid.entity_id(),
    //        1, 
    //        vec![2].iter().cloned().collect(),
    //        1,
    //        true,
    //        Endianness::LittleEndian);
    //     let received_message = RtpsMessage::new(*remote_reader_guid.prefix(), vec![RtpsSubmessage::AckNack(acknack)]);

    //     let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, Some(&received_message));
    //     assert!(submessages.is_none());

    //     sleep(nack_response_delay.into());

    //     let submessages = StatefulWriterBehavior::run_reliable(&mut reader_proxy, &writer_guid, &history_cache, last_change_sequence_number, heartbeat_period, nack_response_delay, Some(&received_message)).unwrap();
    //     assert_eq!(submessages.len(), 4);
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Heartbeat(heartbeat_message) = &submessages[1] {
    //         assert_eq!(heartbeat_message.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(heartbeat_message.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(heartbeat_message.first_sn(), 1);
    //         assert_eq!(heartbeat_message.last_sn(), 2);
    //         assert_eq!(heartbeat_message.count(), 2);
    //         assert_eq!(heartbeat_message.is_final(), false);
    //     }
    //     if let RtpsSubmessage::InfoTs(message_1) = &submessages[2] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Data(data_message_1) = &submessages[3] {
    //         assert_eq!(data_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //         assert_eq!(data_message_1.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //         assert_eq!(data_message_1.writer_sn(), 2);
    //         assert_eq!(data_message_1.serialized_payload(), Some(&vec![2, 3, 4]));

    //     } else {
    //         panic!("Wrong message type");
    //     };

    // }

    // #[test]
    // fn best_effort_stateful_writer_run() {
    //     let mut writer = StatefulWriter::new(
    //         GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::BestEffort,
    //         vec![Locator::new(0, 7400, [0; 16])], 
    //         vec![],                               
    //         false,                                
    //         DURATION_ZERO,                        
    //         DURATION_ZERO,                        
    //         DURATION_ZERO,                        
    //     );

    //     let reader_guid = GUID::new([1;12], ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
    //     let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);

    //     writer.matched_reader_add(reader_proxy);

    //     let cache_change_seq1 = writer.new_change(
    //         ChangeKind::Alive,
    //         Some(vec![1, 2, 3]), 
    //         None,                
    //         [1; 16],             
    //     );

    //     let cache_change_seq2 = writer.new_change(
    //         ChangeKind::Alive,
    //         Some(vec![4, 5, 6]), 
    //         None,                
    //         [1; 16],             
    //     );

    //     writer.writer_cache().add_change(cache_change_seq1);
    //     writer.writer_cache().add_change(cache_change_seq2);

    //     // let reader_proxy = writer.matched_reader_lookup(& reader_guid).unwrap();
    //     let writer_data = writer.run(&reader_guid, None).unwrap();
    //     assert_eq!(writer_data.submessages().len(), 3);
    //     if let RtpsSubmessage::InfoTs(message_1) = &writer_data.submessages()[0] {
    //         println!("{:?}", message_1);
    //     } else {
    //         panic!("Wrong message type");
    //     }
    //     if let RtpsSubmessage::Data(data_message_1) = &writer_data.submessages()[1] {
    //         assert_eq!(data_message_1.reader_id(), ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
    //         assert_eq!(data_message_1.writer_id(), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
    //         assert_eq!(data_message_1.writer_sn(), 1);
    //         assert_eq!(data_message_1.serialized_payload(), Some(&vec![1, 2, 3]));

    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     if let RtpsSubmessage::Data(data_message_2) = &writer_data.submessages()[2] {
    //         assert_eq!(data_message_2.reader_id(), ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
    //         assert_eq!(data_message_2.writer_id(), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
    //         assert_eq!(data_message_2.writer_sn(), 2);
    //         assert_eq!(data_message_2.serialized_payload(), Some(&vec![4, 5, 6]));
    //     } else {
    //         panic!("Wrong message type");
    //     };

    //     // Test that nothing more is sent after the first time
    //     let writer_data = writer.run(&reader_guid, None);
    //     assert_eq!(writer_data.is_none(), true);
    // }

}