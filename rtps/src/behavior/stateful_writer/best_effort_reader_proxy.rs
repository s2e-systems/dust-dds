use std::collections::{BTreeSet, VecDeque};

use crate::types::{EntityId, SequenceNumber};
use crate::structure::HistoryCache;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::Gap;
use crate::behavior::ReaderProxy;
use crate::behavior::{data_from_cache_change, BEHAVIOR_ENDIANNESS};

use super::stateful_writer::ReaderProxyOps;

pub struct BestEffortReaderProxy {
    reader_proxy: ReaderProxy,
    writer_entity_id: EntityId,
    send_messages: VecDeque<RtpsSubmessage>
}

impl BestEffortReaderProxy {
    pub fn new(reader_proxy: ReaderProxy, writer_entity_id: EntityId) -> Self {
        Self{
            reader_proxy,
            writer_entity_id,
            send_messages: VecDeque::new(),
        }
    }
    

    fn pushing_state(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        // This state is only valid if there are unsent changes
        debug_assert!(!self.reader_proxy.unsent_changes(last_change_sequence_number).is_empty());
    
        while let Some(next_unsent_seq_num) = self.reader_proxy.next_unsent_change(last_change_sequence_number) {
            self.transition_t4(history_cache, next_unsent_seq_num);
        }
    }

    fn transition_t4(&mut self, history_cache: &HistoryCache, next_unsent_seq_num: SequenceNumber) {
        if let Some(cache_change) = history_cache
            .changes().iter().find(|cc| cc.sequence_number() == next_unsent_seq_num)
        {
            let reader_id = self.reader_proxy.remote_reader_guid().entity_id();
            let data = data_from_cache_change(cache_change, reader_id);
            self.send_messages.push_back(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                self.reader_proxy.remote_reader_guid().entity_id(), 
                self.writer_entity_id,
                next_unsent_seq_num,
            BTreeSet::new());
            self.send_messages.push_back(RtpsSubmessage::Gap(gap))
        }
    }
}

impl ReaderProxyOps for BestEffortReaderProxy {
    fn run(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        if !self.reader_proxy.unsent_changes(last_change_sequence_number).is_empty() {
            self.pushing_state(history_cache, last_change_sequence_number);
        }
    }

    fn push_receive_message(&mut self, _src_guid_prefix: crate::types::GuidPrefix, _submessage: RtpsSubmessage) {
        assert!(false)
    }

    fn is_submessage_destination(&self, _src_guid_prefix: &crate::types::GuidPrefix, _submessage: &RtpsSubmessage) -> bool {
        // The best effor reader proxy doesn't receive any message
        false
    }

    fn pop_send_message(&mut self) -> Option<(Vec<crate::types::Locator>, VecDeque<RtpsSubmessage>)> {
        if !self.send_messages.is_empty() {
            let mut send_message_queue = VecDeque::new();
            std::mem::swap(&mut send_message_queue, &mut self.send_messages);
            
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
    use super::*;
    use crate::types::{ChangeKind, GUID, Locator};
    use crate::types::constants::{ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER};
    use crate::structure::CacheChange;
    
    use rust_dds_interface::qos_policy::ResourceLimitsQosPolicy;

    #[test]
    fn run() {
        let reader_entity_id = ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR;
        let remote_reader_guid = GUID::new([1;12], reader_entity_id);
        let reader_locator = Locator::new(0, 7400, [1;16]);
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![reader_locator], vec![], expects_inline_qos, is_active);

        let writer_entity_id = ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER;
        let mut best_effort_reader_proxy = BestEffortReaderProxy::new(reader_proxy, writer_entity_id);

        let history_cache = HistoryCache::new(&ResourceLimitsQosPolicy::default());
        
        // Run without any change being created or added in the cache. No message should be sent
        let last_change_sequence_number = 0;
        best_effort_reader_proxy.run(&history_cache, last_change_sequence_number);

        assert!(best_effort_reader_proxy.send_messages.is_empty());

        // Add one change to the history cache and run with that change as the last one. One Data submessage should be sent
        let writer_guid = GUID::new([5;12], writer_entity_id);
        let instance_handle = [1;16];
        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let expected_data_submessage = data_from_cache_change(&cache_change_seq1, reader_entity_id);
        history_cache.add_change(cache_change_seq1).unwrap();

        let last_change_sequence_number = 1;
        best_effort_reader_proxy.run(&history_cache, last_change_sequence_number);

        let expected_submessage = RtpsSubmessage::Data(expected_data_submessage);
        let sent_message = best_effort_reader_proxy.send_messages.pop_front().unwrap();
        assert!(best_effort_reader_proxy.send_messages.is_empty());
        assert_eq!(sent_message, expected_submessage);

        // Run with the next sequence number without adding any change to the history cache. One Gap submessage should be sent
        let last_change_sequence_number = 2;
        best_effort_reader_proxy.run(&history_cache, last_change_sequence_number);

        let expected_submessage = RtpsSubmessage::Gap(Gap::new(BEHAVIOR_ENDIANNESS, reader_entity_id, writer_entity_id, 2, BTreeSet::new()));
        let sent_message = best_effort_reader_proxy.send_messages.pop_front().unwrap();
        assert!(best_effort_reader_proxy.send_messages.is_empty());
        assert_eq!(sent_message, expected_submessage);

        // Add one change to the history cache skipping one sequence number. One Gap and one Data submessage should be sent
        let cache_change_seq4 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 4, Some(vec![4,5,6]), None);
        let expected_data_submessage = data_from_cache_change(&cache_change_seq4, reader_entity_id);
        history_cache.add_change(cache_change_seq4).unwrap();

        let last_change_sequence_number = 4;
        best_effort_reader_proxy.run(&history_cache, last_change_sequence_number);

        let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(BEHAVIOR_ENDIANNESS, reader_entity_id, writer_entity_id, 3, BTreeSet::new()));
        let expected_data_submessage = RtpsSubmessage::Data(expected_data_submessage);

        let sent_message_1 = best_effort_reader_proxy.send_messages.pop_front().unwrap();
        let sent_message_2 = best_effort_reader_proxy.send_messages.pop_front().unwrap();
        assert_eq!(sent_message_1, expected_gap_submessage);
        assert_eq!(sent_message_2, expected_data_submessage);
    }
}