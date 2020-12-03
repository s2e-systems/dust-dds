use std::collections::BTreeSet;
use std::convert::TryInto;
use std::time::Instant;

use crate::behavior::ReaderProxy;
use crate::messages::submessages::{AckNack, Gap, Heartbeat};
use crate::messages::types::Count;
use crate::messages::RtpsSubmessage;
use crate::types::{EntityId, GuidPrefix, GUID};

use crate::behavior::types::Duration;
use crate::behavior::{data_from_cache_change, BEHAVIOR_ENDIANNESS};

use crate::structure::HistoryCache;
use rust_dds_api::types::SequenceNumber;

pub struct ReliableReaderProxy {
    reader_proxy: ReaderProxy,

    heartbeat_count: Count,
    time_last_sent_data: Instant,
    time_nack_received: Instant,
    highest_nack_count_received: Count,
}

impl ReliableReaderProxy {
    pub fn new(reader_proxy: ReaderProxy) -> Self {
        Self {
            reader_proxy,
            heartbeat_count: 0,
            time_last_sent_data: Instant::now(),
            time_nack_received: Instant::now(),
            highest_nack_count_received: 0,
        }
    }

    pub fn produce_messages(
        &mut self,
        history_cache: &HistoryCache,
        writer_entity_id: EntityId,
        last_change_sequence_number: SequenceNumber,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
    ) -> Vec<RtpsSubmessage> {
        let mut message_queue = Vec::new();

        if self
            .reader_proxy
            .unacked_changes(last_change_sequence_number)
            .is_empty()
        {
            // Idle
        } else if !self
            .reader_proxy
            .unsent_changes(last_change_sequence_number)
            .is_empty()
        {
            self.pushing_state(
                history_cache,
                last_change_sequence_number,
                writer_entity_id,
                &mut message_queue,
            );
        } else if !self
            .reader_proxy
            .unacked_changes(last_change_sequence_number)
            .is_empty()
        {
            self.announcing_state(
                history_cache,
                last_change_sequence_number,
                writer_entity_id,
                heartbeat_period,
                &mut message_queue,
            );
        }

        if !self.reader_proxy.requested_changes().is_empty() {
            let duration_since_nack_received: Duration =
                self.time_nack_received.elapsed().try_into().unwrap();
            if duration_since_nack_received > nack_response_delay {
                self.repairing_state(history_cache, writer_entity_id, &mut message_queue);
            }
        }

        message_queue
    }

    pub fn try_process_message(
        &mut self,
        src_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
    ) {
        if let Some(RtpsSubmessage::AckNack(acknack)) = submessage {
            let reader_guid = GUID::new(src_guid_prefix, acknack.reader_id());
            if self.reader_proxy.remote_reader_guid == reader_guid {
                if let RtpsSubmessage::AckNack(acknack) = submessage.take().unwrap() {
                    if acknack.count() > self.highest_nack_count_received {
                        self.highest_nack_count_received = acknack.count();
                        if self.reader_proxy.requested_changes().is_empty() {
                            self.waiting_state(acknack);
                        } else {
                            self.must_repair_state(acknack);
                        }
                    }
                }
            }
        }
    }

    fn pushing_state(
        &mut self,
        history_cache: &HistoryCache,
        last_change_sequence_number: SequenceNumber,
        writer_entity_id: EntityId,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        while let Some(next_unsent_seq_num) = self
            .reader_proxy
            .next_unsent_change(last_change_sequence_number)
        {
            self.transition_t4(
                history_cache,
                next_unsent_seq_num,
                writer_entity_id,
                message_queue,
            );
        }
        self.time_last_sent_data = Instant::now();
    }

    fn transition_t4(
        &mut self,
        history_cache: &HistoryCache,
        next_unsent_seq_num: SequenceNumber,
        writer_entity_id: EntityId,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        if let Some(cache_change) = history_cache.get_change(next_unsent_seq_num) {
            let reader_id = self.reader_proxy.remote_reader_guid.entity_id();
            let data = data_from_cache_change(cache_change, reader_id);
            let mut dst_locator = self.reader_proxy.unicast_locator_list.clone();
            dst_locator.extend(&self.reader_proxy.unicast_locator_list);
            dst_locator.extend(&self.reader_proxy.multicast_locator_list);
            message_queue.push(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                self.reader_proxy.remote_reader_guid.entity_id(),
                writer_entity_id,
                next_unsent_seq_num,
                BTreeSet::new(),
            );

            let mut dst_locator = self.reader_proxy.unicast_locator_list.clone();
            dst_locator.extend(&self.reader_proxy.unicast_locator_list);
            dst_locator.extend(&self.reader_proxy.multicast_locator_list);
            message_queue.push(RtpsSubmessage::Gap(gap));
        }
    }

    fn announcing_state(
        &mut self,
        history_cache: &HistoryCache,
        last_change_sequence_number: SequenceNumber,
        writer_entity_id: EntityId,
        heartbeat_period: Duration,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        let duration_since_last_sent_data: Duration =
            self.time_last_sent_data.elapsed().try_into().unwrap();
        if duration_since_last_sent_data > heartbeat_period {
            self.transition_t7(
                history_cache,
                last_change_sequence_number,
                writer_entity_id,
                message_queue,
            );
            self.time_last_sent_data = Instant::now();
        }
    }

    fn transition_t7(
        &mut self,
        history_cache: &HistoryCache,
        last_change_sequence_number: SequenceNumber,
        writer_entity_id: EntityId,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        let first_sn = if let Some(seq_num) = history_cache.get_seq_num_min() {
            seq_num
        } else {
            last_change_sequence_number + 1
        };
        self.heartbeat_count += 1;

        let heartbeat = Heartbeat::new(
            BEHAVIOR_ENDIANNESS,
            self.reader_proxy.remote_reader_guid.entity_id(),
            writer_entity_id,
            first_sn,
            last_change_sequence_number,
            self.heartbeat_count,
            false,
            false,
        );

        let mut dst_locator = self.reader_proxy.unicast_locator_list.clone();
        dst_locator.extend(&self.reader_proxy.unicast_locator_list);
        dst_locator.extend(&self.reader_proxy.multicast_locator_list);

        message_queue.push(RtpsSubmessage::Heartbeat(heartbeat));
    }

    fn waiting_state(&mut self, acknack: AckNack) {
        self.transition_t8(acknack);
        self.time_nack_received = Instant::now();
    }

    fn transition_t8(&mut self, acknack: AckNack) {
        self.reader_proxy
            .acked_changes_set(acknack.reader_sn_state().base() - 1);
        self.reader_proxy
            .requested_changes_set(acknack.reader_sn_state().set().clone());
    }

    fn must_repair_state(&mut self, acknack: AckNack) {
        self.transition_t8(acknack);
    }

    fn repairing_state(
        &mut self,
        history_cache: &HistoryCache,
        writer_entity_id: EntityId,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        // This state is only valid if there are requested changes
        debug_assert!(!self.reader_proxy.requested_changes().is_empty());

        while let Some(next_requested_seq_num) = self.reader_proxy.next_requested_change() {
            self.transition_t12(
                history_cache,
                next_requested_seq_num,
                writer_entity_id,
                message_queue,
            );
        }
    }

    fn transition_t12(
        &mut self,
        history_cache: &HistoryCache,
        next_requested_seq_num: SequenceNumber,
        writer_entity_id: EntityId,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        if let Some(cache_change) = history_cache.get_change(next_requested_seq_num) {
            let data = data_from_cache_change(
                cache_change,
                self.reader_proxy.remote_reader_guid.entity_id(),
            );
            message_queue.push(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                self.reader_proxy.remote_reader_guid.entity_id(),
                writer_entity_id,
                next_requested_seq_num,
                BTreeSet::new(),
            );
            message_queue.push(RtpsSubmessage::Gap(gap));
        }
    }
}

impl std::ops::Deref for ReliableReaderProxy {
    type Target = ReaderProxy;

    fn deref(&self) -> &Self::Target {
        &self.reader_proxy
    }
}

impl std::ops::DerefMut for ReliableReaderProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader_proxy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::types::Endianness;
    use crate::types::constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    };
    use crate::types::{Locator, GUID};

    use crate::structure::CacheChange;
    use rust_dds_api::types::ChangeKind;

    #[test]
    fn produce_empty() {
        let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let history_cache = HistoryCache::default();

        // Run without any change being created or added in the cache
        let heartbeat_period = Duration::from_secs(1);
        let nack_response_delay = Duration::from_secs(1);
        let last_change_sequence_number = 0;
        let messages_vec = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
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
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

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
        history_cache.add_change(cache_change1.clone()).unwrap();

        // Run with the last change sequence number equal to the added cache change
        let last_change_sequence_number = 1;
        let heartbeat_period = Duration::from_secs(1);
        let nack_response_delay = Duration::from_secs(1);
        let messages_vec = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
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
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let history_cache = HistoryCache::default();

        // Run with the a sequence number of 1 without adding any change to the history cache
        let last_change_sequence_number = 1;
        let heartbeat_period = Duration::from_secs(1);
        let nack_response_delay = Duration::from_secs(1);
        let messages_vec = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
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
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

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
        history_cache.add_change(cache_change1.clone()).unwrap();

        // Run with the last change sequence number one above the added cache change
        let last_change_sequence_number = 2;
        let heartbeat_period = Duration::from_secs(1);
        let nack_response_delay = Duration::from_secs(1);
        let messages_vec = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
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

    #[test]
    fn try_process_acknack_message_only_acknowledge() {
        let remote_reader_guid_prefix = [5; 12];
        let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
        let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;

        let acknack = AckNack::new(
            Endianness::LittleEndian,
            remote_reader_guid.entity_id(),
            writer_entity_id,
            2,
            vec![].iter().cloned().collect(),
            1,
            true,
        );

        reliable_reader_proxy.try_process_message(
            remote_reader_guid_prefix,
            &mut Some(RtpsSubmessage::AckNack(acknack)),
        );

        assert_eq!(reliable_reader_proxy.highest_nack_count_received, 1);
        assert!(reliable_reader_proxy
            .reader_proxy
            .unacked_changes(1)
            .is_empty()); // If 1 is the last change sequence number there are no unacked changes
        assert!(reliable_reader_proxy
            .reader_proxy
            .unacked_changes(2)
            .contains(&2)); // If 2 is the last change sequence number, then 2 is an unacked change
    }

    #[test]
    fn try_process_acknack_message_acknowledge_and_request() {
        let remote_reader_guid_prefix = [5; 12];
        let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
        let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;

        let acknack = AckNack::new(
            Endianness::LittleEndian,
            remote_reader_guid.entity_id(),
            writer_entity_id,
            4,
            vec![1, 3].iter().cloned().collect(),
            1,
            true,
        );

        reliable_reader_proxy.try_process_message(
            remote_reader_guid_prefix,
            &mut Some(RtpsSubmessage::AckNack(acknack)),
        );

        let requested_changes = reliable_reader_proxy.reader_proxy.requested_changes();
        assert!(requested_changes.contains(&1));
        assert!(requested_changes.contains(&3));
    }

    #[test]
    fn ignore_try_process_acknack_message_with_same_count_number() {
        let remote_reader_guid_prefix = [5; 12];
        let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
        let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;

        let acknack = AckNack::new(
            Endianness::LittleEndian,
            remote_reader_guid.entity_id(),
            writer_entity_id,
            4,
            vec![1, 3].iter().cloned().collect(),
            1,
            true,
        );

        reliable_reader_proxy.try_process_message(
            remote_reader_guid_prefix,
            &mut Some(RtpsSubmessage::AckNack(acknack)),
        );

        let acknack_same_count = AckNack::new(
            Endianness::LittleEndian,
            remote_reader_guid.entity_id(),
            writer_entity_id,
            6,
            vec![1, 2, 3].iter().cloned().collect(),
            1,
            true,
        );

        reliable_reader_proxy.try_process_message(
            remote_reader_guid_prefix,
            &mut Some(RtpsSubmessage::AckNack(acknack_same_count)),
        );

        assert_eq!(reliable_reader_proxy.highest_nack_count_received, 1);
        let requested_changes = reliable_reader_proxy.reader_proxy.requested_changes();
        assert!(requested_changes.contains(&1));
        assert!(requested_changes.contains(&3));
    }

    #[test]
    fn produce_heartbeat_message() {
        let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

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
        history_cache.add_change(cache_change1.clone()).unwrap();

        let writer_guid = GUID::new([5; 12], writer_entity_id);
        let instance_handle = [1; 16];
        let cache_change2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            2,
            Some(vec![4, 5, 6]),
            None,
        );
        history_cache.add_change(cache_change2.clone()).unwrap();

        let last_change_sequence_number = 2;
        let heartbeat_period = Duration::from_secs(0);
        let nack_response_delay = Duration::from_secs(1);

        // The first produce should generate the data/gap messages and no heartbeat so we ignore it
        reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        let messages_vec1 = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        let messages_vec2 = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        let expected_heartbeat_message1 = RtpsSubmessage::Heartbeat(Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            2,
            1,
            false,
            false,
        ));

        let expected_heartbeat_message2 = RtpsSubmessage::Heartbeat(Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            2,
            2,
            false,
            false,
        ));

        assert_eq!(messages_vec1.len(), 1);
        assert!(messages_vec1.contains(&expected_heartbeat_message1));
        assert_eq!(messages_vec2.len(), 1);
        assert!(messages_vec2.contains(&expected_heartbeat_message2));
    }

    #[test]
    fn produce_heartbeat_message_period() {
        let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

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
        history_cache.add_change(cache_change1.clone()).unwrap();

        let writer_guid = GUID::new([5; 12], writer_entity_id);
        let instance_handle = [1; 16];
        let cache_change2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            2,
            Some(vec![4, 5, 6]),
            None,
        );
        history_cache.add_change(cache_change2.clone()).unwrap();

        let last_change_sequence_number = 2;
        let heartbeat_period = Duration::from_millis(200);
        let nack_response_delay = Duration::from_secs(1);

        // The first produce should generate the data/gap messages and no heartbeat so we ignore it
        reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        let messages_vec1 = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        std::thread::sleep(heartbeat_period.into());

        let messages_vec2 = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        let expected_heartbeat_message = RtpsSubmessage::Heartbeat(Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            2,
            1,
            false,
            false,
        ));

        assert!(messages_vec1.is_empty());
        assert_eq!(messages_vec2.len(), 1);
        assert!(messages_vec2.contains(&expected_heartbeat_message));
    } 

    #[test]
    fn produce_requested_changes_messages() {
        let remote_reader_guid_prefix = [5; 12];
        let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
        let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

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
        history_cache.add_change(cache_change1.clone()).unwrap();

        let writer_guid = GUID::new([5; 12], writer_entity_id);
        let instance_handle = [1; 16];
        let cache_change2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            2,
            Some(vec![4, 5, 6]),
            None,
        );
        history_cache.add_change(cache_change2.clone()).unwrap();

        let last_change_sequence_number = 2;
        let heartbeat_period = Duration::from_secs(1);
        let nack_response_delay = Duration::from_secs(0);

        let acknack = AckNack::new(
            Endianness::LittleEndian,
            remote_reader_guid.entity_id(),
            writer_entity_id,
            3,
            vec![1].iter().cloned().collect(),
            1,
            true,
        );

        reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        reliable_reader_proxy.try_process_message(
            remote_reader_guid_prefix,
            &mut Some(RtpsSubmessage::AckNack(acknack)),
        );

        let messages_vec = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
            &cache_change1,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
        ));
        assert_eq!(messages_vec.len(), 1);
        assert!(messages_vec.contains(&expected_data_submessage));
    }

    #[test]
    fn produce_requested_changes_with_nack_response_delay() {
        let remote_reader_guid_prefix = [5; 12];
        let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
        let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let mut reliable_reader_proxy = ReliableReaderProxy::new(reader_proxy);

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
        history_cache.add_change(cache_change1.clone()).unwrap();

        let writer_guid = GUID::new([5; 12], writer_entity_id);
        let instance_handle = [1; 16];
        let cache_change2 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            2,
            Some(vec![4, 5, 6]),
            None,
        );
        history_cache.add_change(cache_change2.clone()).unwrap();

        let last_change_sequence_number = 2;
        let heartbeat_period = Duration::from_secs(1);
        let nack_response_delay = Duration::from_millis(200);

        let acknack = AckNack::new(
            Endianness::LittleEndian,
            remote_reader_guid.entity_id(),
            writer_entity_id,
            3,
            vec![1].iter().cloned().collect(),
            1,
            true,
        );

        reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        reliable_reader_proxy.try_process_message(
            remote_reader_guid_prefix,
            &mut Some(RtpsSubmessage::AckNack(acknack)),
        );

        let messages_vec_expected_empty = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );
    
        std::thread::sleep(nack_response_delay.into());

        let messages_vec = reliable_reader_proxy.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
            heartbeat_period,
            nack_response_delay,
        );

        let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
            &cache_change1,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
        ));

        assert!(messages_vec_expected_empty.is_empty());
        assert_eq!(messages_vec.len(), 1);
        assert!(messages_vec.contains(&expected_data_submessage));
    }
}
