use std::collections::BTreeSet;


use crate::{behavior::{BEHAVIOR_ENDIANNESS, data_from_cache_change}, messages::{RtpsSubmessage, submessages::Gap}, structure::HistoryCache, types::{EntityId, SequenceNumber}};

use super::ReaderProxy;

pub struct BestEffortReaderProxyBehavior;

impl BestEffortReaderProxyBehavior {
    pub fn produce_messages(
        reader_proxy: &mut ReaderProxy,
        history_cache: &HistoryCache,
        writer_entity_id: EntityId,
        last_change_sequence_number: SequenceNumber,
    ) -> Vec<RtpsSubmessage> {
        let mut messages = Vec::new();
        if !reader_proxy.unsent_changes(last_change_sequence_number).is_empty() {
            Self::pushing_state(
                reader_proxy,
                history_cache,
                last_change_sequence_number,
                writer_entity_id,
                &mut messages,
            );
        }
        messages
    }

    fn pushing_state(
        reader_proxy: &mut ReaderProxy,
        history_cache: &HistoryCache,
        last_change_sequence_number: SequenceNumber,
        writer_entity_id: EntityId,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        while let Some(next_unsent_seq_num) = reader_proxy.next_unsent_change(last_change_sequence_number) {
            Self::transition_t4(
                reader_proxy,
                history_cache,
                next_unsent_seq_num,
                writer_entity_id,
                message_queue,
            );
        }
    }

    fn transition_t4(
        reader_proxy: &mut ReaderProxy,
        history_cache: &HistoryCache,
        next_unsent_seq_num: SequenceNumber,
        writer_entity_id: EntityId,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        if let Some(cache_change) = history_cache.get_change(next_unsent_seq_num) {
            let reader_id = reader_proxy.remote_reader_guid.entity_id();
            let data = data_from_cache_change(cache_change, reader_id);
            let mut dst_locator = reader_proxy.unicast_locator_list.clone();
            dst_locator.extend(&reader_proxy.unicast_locator_list);
            dst_locator.extend(&reader_proxy.multicast_locator_list);
            message_queue.push(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                reader_proxy.remote_reader_guid.entity_id(),
                writer_entity_id,
                next_unsent_seq_num,
                BTreeSet::new(),
            );
            let mut dst_locator = reader_proxy.unicast_locator_list.clone();
            dst_locator.extend(&reader_proxy.unicast_locator_list);
            dst_locator.extend(&reader_proxy.multicast_locator_list);
            message_queue.push(RtpsSubmessage::Gap(gap));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeKind, constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    }};
    use crate::types::{Locator, GUID};

    use crate::structure::CacheChange;

    #[test]
    fn produce_empty() {
        let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let history_cache = HistoryCache::default();

        // Run without any change being created or added in the cache
        let last_change_sequence_number = 0;
        let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
            &mut reader_proxy,
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        assert!(messages_vec.is_empty());
    }

    #[test]
    fn produce_data_message() {
        let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let mut history_cache = HistoryCache::default();

        // Add one change to the history cache
        let writer_guid = GUID::new([5; 12], writer_entity_id);
        let instance_handle = [1; 16];
        let cache_change1 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            1,
            Some(vec![1, 2, 3]),
            None,
        );
        history_cache.add_change(cache_change1.clone());

        // Run with the last change sequence number equal to the added cache change
        let last_change_sequence_number = 1;
        let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
            &mut reader_proxy,
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
            &cache_change1,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
        ));
        assert_eq!(messages_vec.len(), 1);
        assert!(messages_vec.contains(&expected_data_submessage));
    }

    #[test]
    fn produce_gap_message() {
        let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let history_cache = HistoryCache::default();

        // Run with the a sequence number of 1 without adding any change to the history cache
        let last_change_sequence_number = 1;
        let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
            &mut reader_proxy,
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
            BEHAVIOR_ENDIANNESS,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            writer_entity_id,
            1,
            BTreeSet::new(),
        ));
        assert_eq!(messages_vec.len(), 1);
        assert!(messages_vec.contains(&expected_gap_submessage));
    }

    #[test]
    fn produce_data_and_gap_messages() {
        let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let mut history_cache = HistoryCache::default();

        // Add one change to the history cache
        let writer_guid = GUID::new([5; 12], writer_entity_id);
        let instance_handle = [1; 16];
        let cache_change1 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            1,
            Some(vec![1, 2, 3]),
            None,
        );
        history_cache.add_change(cache_change1.clone());

        // Run with the last change sequence number one above the added cache change
        let last_change_sequence_number = 2;
        let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
            &mut reader_proxy,
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
            &cache_change1,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
        ));
        let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
            BEHAVIOR_ENDIANNESS,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            writer_entity_id,
            2,
            BTreeSet::new(),
        ));
        assert_eq!(messages_vec.len(), 2);
        assert!(messages_vec.contains(&expected_data_submessage));
        assert!(messages_vec.contains(&expected_gap_submessage));
    }
}
