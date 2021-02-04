use std::{convert::TryInto, time::Instant};

use crate::{
    behavior::{cache_change_from_data, types::Duration, BEHAVIOR_ENDIANNESS},
    messages::{
        submessages::{AckNack, Data, Gap, Heartbeat},
        types::Count,
        RtpsSubmessage,
    },
    structure::HistoryCache,
    types::{EntityId, GuidPrefix, GUID},
};

use super::WriterProxy;

pub struct ReliableWriterProxy {
    writer_proxy: WriterProxy,

    must_send_ack: bool,
    time_heartbeat_received: Instant,
    ackanck_count: Count,
    // highest_received_heartbeat_count: Count,
}

impl ReliableWriterProxy {
    pub fn new(writer_proxy: WriterProxy) -> Self {
        Self {
            writer_proxy,
            must_send_ack: false,
            time_heartbeat_received: Instant::now(),
            ackanck_count: 0,
            // highest_received_heartbeat_count: 0,
        }
    }

    pub fn try_process_message(
        &mut self,
        src_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
        history_cache: &mut HistoryCache,
    ) {
        if let Some(inner_submessage) = submessage {
            if self.is_submessage_destination(src_guid_prefix, inner_submessage) {
                // If Waiting state (marked by the must_send_ack flag)
                if !self.must_send_ack {
                    self.waiting_heartbeat_state(inner_submessage);
                }

                self.ready_state(submessage, history_cache);
            }
        }
    }

    pub fn produce_messages(
        &mut self,
        reader_entity_id: EntityId,
        heartbeat_response_delay: Duration,
    ) -> Vec<RtpsSubmessage> {
        let mut output_queue = Vec::new();
        if self.must_send_ack {
            self.must_send_ack_state(
                reader_entity_id,
                heartbeat_response_delay,
                &mut output_queue,
            );
        }
        output_queue
    }

    fn ready_state(
        &mut self,
        submessage: &mut Option<RtpsSubmessage>,
        history_cache: &mut HistoryCache,
    ) {
        match submessage.take().unwrap() {
            RtpsSubmessage::Data(data) => self.transition_t8(history_cache, data),
            RtpsSubmessage::Gap(gap) => self.transition_t9(gap),
            RtpsSubmessage::Heartbeat(heartbeat) => self.transition_t7(heartbeat),
            _ => panic!("Unexpected reader message received"),
        }
    }

    fn is_submessage_destination(
        &self,
        src_guid_prefix: GuidPrefix,
        submessage: &RtpsSubmessage,
    ) -> bool {
        let writer_id = match submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            RtpsSubmessage::Gap(gap) => gap.writer_id(),
            RtpsSubmessage::Heartbeat(heartbeat) => heartbeat.writer_id(),
            _ => return false,
        };

        let writer_guid = GUID::new(src_guid_prefix, writer_id);
        if self.remote_writer_guid == writer_guid {
            true
        } else {
            false
        }
    }

    fn transition_t8(&mut self, history_cache: &mut HistoryCache, data: Data) {
        let expected_seq_number = self.available_changes_max() + 1;
        if data.writer_sn() >= expected_seq_number {
            self.received_change_set(data.writer_sn());
            let cache_change = cache_change_from_data(data, &self.remote_writer_guid.prefix());
            history_cache.add_change(cache_change);
        }
    }

    fn transition_t9(&mut self, gap: Gap) {
        for seq_num in gap.gap_start()..gap.gap_list().base() - 1 {
            self.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            self.irrelevant_change_set(seq_num);
        }
    }

    fn transition_t7(&mut self, heartbeat: Heartbeat) {
        self.missing_changes_update(heartbeat.last_sn());
        self.lost_changes_update(heartbeat.first_sn());
    }

    fn waiting_heartbeat_state(&mut self, submessage: &RtpsSubmessage) {
        if let RtpsSubmessage::Heartbeat(heartbeat) = submessage {
            if !heartbeat.is_final() || (heartbeat.is_final() && !self.missing_changes().is_empty())
            {
                self.time_heartbeat_received = Instant::now();
                self.must_send_ack = true;
            }
        }
    }

    fn must_send_ack_state(
        &mut self,
        reader_entity_id: EntityId,
        heartbeat_response_delay: Duration,
        output_queue: &mut Vec<RtpsSubmessage>,
    ) {
        let duration_since_heartbeat_received: Duration =
            self.time_heartbeat_received.elapsed().try_into().unwrap();
        if duration_since_heartbeat_received > heartbeat_response_delay {
            self.transition_t5(reader_entity_id, output_queue)
        }
    }

    fn transition_t5(
        &mut self,
        reader_entity_id: EntityId,
        output_queue: &mut Vec<RtpsSubmessage>,
    ) {
        self.must_send_ack = false;

        self.ackanck_count += 1;
        let acknack = AckNack::new(
            BEHAVIOR_ENDIANNESS,
            reader_entity_id,
            self.remote_writer_guid.entity_id(),
            self.available_changes_max(),
            self.missing_changes().clone(),
            self.ackanck_count,
            true,
        );

        output_queue.push(RtpsSubmessage::AckNack(acknack));
    }
}

impl std::ops::Deref for ReliableWriterProxy {
    type Target = WriterProxy;

    fn deref(&self) -> &Self::Target {
        &self.writer_proxy
    }
}

impl std::ops::DerefMut for ReliableWriterProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer_proxy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::submessages::data_submessage::Payload;
    use crate::messages::types::{Endianness, KeyHash};
    use crate::types::constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    };
    use crate::types::Locator;
    use crate::{
        behavior::change_kind_to_status_info,
        messages::submessages::submessage_elements::ParameterList, types::ChangeKind,
    };

    #[test]
    fn process_none_submessage() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let source_guid_prefix = [5; 12];
        let mut submessage = None;
        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut submessage,
            &mut history_cache,
        );
    }

    #[test]
    fn process_data_submessage() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let source_guid_prefix = [5; 12];
        let status_info = change_kind_to_status_info(ChangeKind::Alive);
        let key_hash = KeyHash([1; 16]);
        let mut inline_qos = ParameterList::new();
        inline_qos.parameter.push(key_hash.into());
        inline_qos.parameter.push(status_info.into());
        let data_submessage = Data::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            Some(inline_qos),
            Payload::Data(vec![0, 1, 2]),
        );
        let expected_cache_change =
            cache_change_from_data(data_submessage.clone(), &source_guid_prefix);

        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Data(data_submessage)),
            &mut history_cache,
        );
        let received_change = history_cache.get_change(1).unwrap();
        assert_eq!(received_change, &expected_cache_change);
    }

    #[test]
    fn process_non_final_heartbeat_with_missing_changes() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let heartbeat_submessage = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            3,
            6,
            1,
            false,
            false,
        );

        let source_guid_prefix = [5; 12];
        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
            &mut history_cache,
        );

        assert_eq!(
            reliable_writer_proxy.missing_changes(),
            [3, 4, 5, 6].iter().cloned().collect()
        );
        assert_eq!(reliable_writer_proxy.must_send_ack, true);
    }

    #[test]
    fn process_final_heartbeat_with_missing_changes() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let heartbeat_submessage = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            3,
            6,
            1,
            true,
            false,
        );

        let source_guid_prefix = [5; 12];
        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
            &mut history_cache,
        );

        assert_eq!(
            reliable_writer_proxy.missing_changes(),
            [3, 4, 5, 6].iter().cloned().collect()
        );
        assert_eq!(reliable_writer_proxy.must_send_ack, false);
    }

    #[test]
    fn process_final_heartbeat_without_missing_changes() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let heartbeat_submessage = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            0,
            1,
            true,
            false,
        );

        let source_guid_prefix = [5; 12];
        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
            &mut history_cache,
        );

        assert_eq!(
            reliable_writer_proxy.missing_changes(),
            [].iter().cloned().collect()
        );
        assert_eq!(reliable_writer_proxy.must_send_ack, false);
    }

    #[test]
    fn produce_acknack_heartbeat_response_without_missing_changes() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let heartbeat_submessage = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            0,
            1,
            false,
            false,
        );

        let source_guid_prefix = [5; 12];
        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
            &mut history_cache,
        );

        let heartbeat_response_delay = Duration::from_millis(200);
        let produced_messages_empty = reliable_writer_proxy.produce_messages(
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            heartbeat_response_delay,
        );

        std::thread::sleep(heartbeat_response_delay.into());

        let produced_messages = reliable_writer_proxy.produce_messages(
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            heartbeat_response_delay,
        );

        let expected_acknack_submessage = RtpsSubmessage::AckNack(AckNack::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            0,
            [].iter().cloned().collect(),
            1,
            true,
        ));

        assert!(produced_messages_empty.is_empty());
        assert_eq!(produced_messages.len(), 1);
        assert!(produced_messages.contains(&expected_acknack_submessage));
    }

    #[test]
    fn produce_acknack_heartbeat_response_with_missing_changes() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let heartbeat_submessage = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            3,
            6,
            1,
            false,
            false,
        );

        let source_guid_prefix = [5; 12];
        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
            &mut history_cache,
        );

        let heartbeat_response_delay = Duration::from_millis(0);

        let produced_messages = reliable_writer_proxy.produce_messages(
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            heartbeat_response_delay,
        );

        let expected_acknack_submessage = RtpsSubmessage::AckNack(AckNack::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            2,
            [3, 4, 5, 6].iter().cloned().collect(),
            1,
            true,
        ));

        assert_eq!(produced_messages.len(), 1);
        assert!(produced_messages.contains(&expected_acknack_submessage));
    }

    #[test]
    fn produce_acknack_heartbeat_response_with_partial_missing_changes() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        reliable_writer_proxy.received_change_set(1);
        reliable_writer_proxy.received_change_set(2);
        reliable_writer_proxy.received_change_set(3);
        reliable_writer_proxy.received_change_set(4);

        let heartbeat_submessage = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            3,
            6,
            1,
            false,
            false,
        );

        let source_guid_prefix = [5; 12];
        reliable_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
            &mut history_cache,
        );

        let heartbeat_response_delay = Duration::from_millis(0);

        let produced_messages = reliable_writer_proxy.produce_messages(
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            heartbeat_response_delay,
        );

        let expected_acknack_submessage = RtpsSubmessage::AckNack(AckNack::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            4,
            [5, 6].iter().cloned().collect(),
            1,
            true,
        ));

        assert_eq!(produced_messages.len(), 1);
        assert!(produced_messages.contains(&expected_acknack_submessage));
    }
}
