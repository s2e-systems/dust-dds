use std::convert::TryInto;
use std::time::Instant;

use crate::types::constants::LOCATOR_INVALID;
use crate::structure::{WriterProxy, StatefulReader};
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::{AckNack, Data, Gap, Heartbeat,};
use crate::messages::types::Count;
use crate::messages::message_receiver::Receiver;
use crate::messages::message_sender::Sender;

use super::types::Duration;
use super::{cache_change_from_data, BEHAVIOR_ENDIANNESS};

pub struct StatefulReaderBehavior {
    must_send_ack: bool,
    time_heartbeat_received: Instant,
    ackanck_count: Count,
    highest_received_heartbeat_count: Count,
}

impl StatefulReaderBehavior {
    pub fn new() -> Self {
        Self {
            must_send_ack: false,
            time_heartbeat_received: Instant::now(),
            ackanck_count: 0,
            highest_received_heartbeat_count: 0,
        }
    }

    fn must_send_ack(&self) -> bool {
        self.must_send_ack
    }

    fn set_must_send_ack(&mut self) {
        self.time_heartbeat_received  = Instant::now();
        self.must_send_ack = true;
    }

    fn reset_must_send_ack(&mut self) {
        self.must_send_ack = false;
    }

    fn duration_since_heartbeat_received(&self) -> Duration {
        self.time_heartbeat_received.elapsed().try_into().unwrap()
    }

    fn ackanck_count(&self) -> &Count {
        &self.ackanck_count
    }

    pub fn increment_acknack_count(&mut self) {
        self.ackanck_count += 1;
    }
}

pub struct BestEfforStatefulReaderBehavior {}

impl BestEfforStatefulReaderBehavior {
    pub fn run(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader) {
        Self::waiting_state(writer_proxy, stateful_reader);
    }

    fn waiting_state(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader) {
        if let Some((_, received_message)) = stateful_reader.pop_receive_message(writer_proxy.remote_writer_guid()) {
            match received_message {
                RtpsSubmessage::Data(data) => Self::transition_t2(writer_proxy, stateful_reader, data),
                RtpsSubmessage::Gap(gap) => Self::transition_t4(writer_proxy, &gap),
                RtpsSubmessage::Heartbeat(_) => (),
                _ => panic!("Unexpected reader message received"),
            }
        }
    }

    fn transition_t2(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader, data: Data) {
        let expected_seq_number = writer_proxy.available_changes_max() + 1;
        if data.writer_sn() >= expected_seq_number {
            writer_proxy.received_change_set(data.writer_sn());
            writer_proxy.lost_changes_update(data.writer_sn());
            let cache_change = cache_change_from_data(data, &writer_proxy.remote_writer_guid().prefix());
            stateful_reader.reader_cache().add_change(cache_change);
        }
    }

    fn transition_t4(writer_proxy: &WriterProxy, gap: &Gap) {
        for seq_num in gap.gap_start() .. gap.gap_list().base() - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }
}
pub struct ReliableStatefulReaderBehavior {}

impl ReliableStatefulReaderBehavior {
    pub fn run(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader) {
        // The heartbeat message triggers also a transition in the parallel state-machine
        // relating to the acknack sending so it is returned from the ready_state for
        // further processing.
        let heartbeat = Self::ready_state(writer_proxy, stateful_reader);
        if writer_proxy.behavior().must_send_ack() {
            Self::must_send_ack_state(writer_proxy, stateful_reader)
        } else {
            Self::waiting_heartbeat_state(writer_proxy, heartbeat);
        }
    }

    fn ready_state(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader) -> Option<Heartbeat>{
        if let Some((_, received_message)) = stateful_reader.pop_receive_message(writer_proxy.remote_writer_guid()) {
            match received_message {
                RtpsSubmessage::Data(data) => {
                    Self::transition_t8(writer_proxy, stateful_reader, data);
                    None
                },
                RtpsSubmessage::Gap(gap) => {
                    Self::transition_t9(writer_proxy, &gap);
                    None
                },
                RtpsSubmessage::Heartbeat(heartbeat) => {
                    Self::transition_t7(writer_proxy, &heartbeat);
                    Some(heartbeat)
                },
                _ => panic!("Unexpected reader message received"),
            }
        } else {
            None
        }
    }

    fn transition_t8(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader, data: Data) {
        let expected_seq_number = writer_proxy.available_changes_max() + 1;
        if data.writer_sn() >= expected_seq_number {
            writer_proxy.received_change_set(data.writer_sn());
            let cache_change = cache_change_from_data(data, &writer_proxy.remote_writer_guid().prefix());
            stateful_reader.reader_cache().add_change(cache_change);
            
        }
    }

    fn transition_t9(writer_proxy: &WriterProxy, gap: &Gap) {
        for seq_num in gap.gap_start() .. gap.gap_list().base() - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }

    fn transition_t7(writer_proxy: &WriterProxy, heartbeat: &Heartbeat) {
        writer_proxy.missing_changes_update(heartbeat.last_sn());
        writer_proxy.lost_changes_update(heartbeat.first_sn());
    }

    fn waiting_heartbeat_state(writer_proxy: &WriterProxy, heartbeat_message: Option<Heartbeat>) {            
        if let Some(heartbeat) = heartbeat_message {
            if !heartbeat.is_final() || 
                (heartbeat.is_final() && !writer_proxy.missing_changes().is_empty()) {
                writer_proxy.behavior().set_must_send_ack();
            } 
        }
    }

    fn must_send_ack_state(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader) {
        if writer_proxy.behavior().duration_since_heartbeat_received() >  stateful_reader.heartbeat_response_delay() {
            Self::transition_t5(writer_proxy, stateful_reader)
        }
    }

    fn transition_t5(writer_proxy: &WriterProxy, stateful_reader: &StatefulReader) {
        writer_proxy.behavior().reset_must_send_ack();
 
        writer_proxy.behavior().increment_acknack_count();
        let acknack = AckNack::new(
            BEHAVIOR_ENDIANNESS,
            stateful_reader.guid().entity_id(), 
            writer_proxy.remote_writer_guid().entity_id(),
            writer_proxy.available_changes_max(),
            writer_proxy.missing_changes().clone(),
            *writer_proxy.behavior().ackanck_count(),
            true);

        stateful_reader.push_send_message(&LOCATOR_INVALID, writer_proxy.remote_writer_guid(), RtpsSubmessage::AckNack(acknack));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeKind, TopicKind, GUID};
    use crate::types::constants::{
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, };
    use crate::structure::CacheChange;
    use crate::messages::submessages::data_submessage::Payload;
    use crate::serialized_payload::ParameterList;
    use crate::inline_qos_types::KeyHash;
    use crate::messages::Endianness;
    use super::super::change_kind_to_status_info;

    use rust_dds_interface::qos::DataReaderQos;
    use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

    #[test]
    fn run_best_effort_data_only() {
        let reader_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_qos = DataReaderQos::default();
        reader_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;

        let stateful_reader = StatefulReader::new(
            reader_guid,
            TopicKind::WithKey,
            &reader_qos);

        let remote_writer_guid_prefix = [1;12];
        let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
        stateful_reader.matched_writer_add(writer_proxy);

        let mut inline_qos = ParameterList::new();
        inline_qos.push(change_kind_to_status_info(ChangeKind::Alive));
        inline_qos.push(KeyHash([1;16]));

        let data1 = Data::new(
            Endianness::LittleEndian,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, 
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
            3,
            Some(inline_qos),
            Payload::Data(vec![1,2,3]));
        stateful_reader.push_receive_message(remote_writer_guid_prefix, RtpsSubmessage::Data(data1));

        let matched_writers = stateful_reader.matched_writers();
        let writer_proxy = matched_writers.get(&remote_writer_guid).unwrap();
        BestEfforStatefulReaderBehavior::run(writer_proxy, &stateful_reader);

        let expected_change_1 = CacheChange::new(
            ChangeKind::Alive,
            remote_writer_guid,
            [1;16],
            3,
            Some(vec![1,2,3]),
            None,
        );

        assert_eq!(stateful_reader.reader_cache().changes().len(), 1);
        assert!(stateful_reader.reader_cache().changes().contains(&expected_change_1));
        assert_eq!(writer_proxy.available_changes_max(), 3);

        // Run waiting state without any received message and verify nothing changes
        BestEfforStatefulReaderBehavior::run(&writer_proxy, &stateful_reader);
        assert_eq!(stateful_reader.reader_cache().changes().len(), 1);
        assert_eq!(writer_proxy.available_changes_max(), 3);
    }

    #[test]
    fn run_reliable_data_only() {
        let reader_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_qos = DataReaderQos::default();
        reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        let stateful_reader = StatefulReader::new(
            reader_guid,
            TopicKind::WithKey,
            &reader_qos);

        let remote_writer_guid_prefix = [1;12];
        let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
        stateful_reader.matched_writer_add(writer_proxy);

        let mut inline_qos = ParameterList::new();
        inline_qos.push(change_kind_to_status_info(ChangeKind::Alive));
        inline_qos.push(KeyHash([1;16]));

        let data1 = Data::new(
            Endianness::LittleEndian,
            reader_guid.entity_id(), 
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
            3,
            Some(inline_qos),
            Payload::Data(vec![1,2,3]));

        stateful_reader.push_receive_message(remote_writer_guid_prefix, RtpsSubmessage::Data(data1));

        let matched_writers = stateful_reader.matched_writers();
        let writer_proxy = matched_writers.get(&remote_writer_guid).unwrap();
        ReliableStatefulReaderBehavior::run(writer_proxy, &stateful_reader);

        let expected_change_1 = CacheChange::new(
            ChangeKind::Alive,
            remote_writer_guid,
            [1;16],
            3,
            Some(vec![1,2,3]),
            None,
        );

        assert_eq!(stateful_reader.reader_cache().changes().len(), 1);
        assert!(stateful_reader.reader_cache().changes().contains(&expected_change_1));
        assert_eq!(writer_proxy.available_changes_max(), 0);

        // Run ready state without any received message and verify nothing changes
        let matched_writers = stateful_reader.matched_writers();
        let writer_proxy = matched_writers.get(&remote_writer_guid).unwrap();
        ReliableStatefulReaderBehavior::ready_state(writer_proxy, &stateful_reader);
        assert_eq!(stateful_reader.reader_cache().changes().len(), 1);
        assert_eq!(writer_proxy.available_changes_max(), 0);
    }

    #[test]
    fn run_reliable_non_final_heartbeat() {
        let reader_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_qos = DataReaderQos::default();
        reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        let stateful_reader = StatefulReader::new(
            reader_guid,
            TopicKind::WithKey,
            &reader_qos);

        let remote_writer_guid_prefix = [1;12];
        let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
        stateful_reader.matched_writer_add(writer_proxy);

        let heartbeat = Heartbeat::new(
            Endianness::LittleEndian,
            reader_guid.entity_id(),
            remote_writer_guid.entity_id(),
            3,
            6,
            1,
            false,
            false,
        );
    
        stateful_reader.push_receive_message(remote_writer_guid_prefix, RtpsSubmessage::Heartbeat(heartbeat));

        let matched_writers = stateful_reader.matched_writers();
        let writer_proxy = matched_writers.get(&remote_writer_guid).unwrap();
        ReliableStatefulReaderBehavior::run(writer_proxy, &stateful_reader);
        assert_eq!(writer_proxy.missing_changes(), [3, 4, 5, 6].iter().cloned().collect());
        assert_eq!(writer_proxy.behavior().must_send_ack(), true);
    }
    
    #[test]
    fn run_reliable_final_heartbeat_with_missing_changes() {
        let reader_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_qos = DataReaderQos::default();
        reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        let stateful_reader = StatefulReader::new(
            reader_guid,
            TopicKind::WithKey,
            &reader_qos);

        let remote_writer_guid_prefix = [1;12];
        let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
        stateful_reader.matched_writer_add(writer_proxy);

        let heartbeat = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            remote_writer_guid.entity_id(),
            2,
            3,
            1,
            true,
            false,
        );
        stateful_reader.push_receive_message(remote_writer_guid_prefix, RtpsSubmessage::Heartbeat(heartbeat));

        let matched_writers = stateful_reader.matched_writers();
        let writer_proxy = matched_writers.get(&remote_writer_guid).unwrap();

        let heartbeat_response_delay = Duration::from_millis(300);
        ReliableStatefulReaderBehavior::run(writer_proxy, &stateful_reader);
        assert_eq!(writer_proxy.missing_changes(), [2, 3].iter().cloned().collect());
        assert_eq!(writer_proxy.behavior().must_send_ack(), true);

        std::thread::sleep(heartbeat_response_delay.into());

        
        ReliableStatefulReaderBehavior::run(writer_proxy, &stateful_reader);
        assert_eq!(writer_proxy.behavior().must_send_ack(), false);

        // TODO: Test that AckNack is sent after duration
    }

    #[test]
    fn run_reliable_final_heartbeat_without_missing_changes() {
        let reader_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let mut reader_qos = DataReaderQos::default();
        reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        let stateful_reader = StatefulReader::new(
            reader_guid,
            TopicKind::WithKey,
            &reader_qos);

        let remote_writer_guid_prefix = [1;12];
        let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
        stateful_reader.matched_writer_add(writer_proxy);

        let heartbeat = Heartbeat::new(
            Endianness::LittleEndian,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            remote_writer_guid.entity_id(),
            1,
            0,
            1,
            true,
            false,
        );
        stateful_reader.push_receive_message(remote_writer_guid_prefix, RtpsSubmessage::Heartbeat(heartbeat));

        let matched_writers = stateful_reader.matched_writers();
        let writer_proxy = matched_writers.get(&remote_writer_guid).unwrap();
        ReliableStatefulReaderBehavior::run(writer_proxy, &stateful_reader);
        assert_eq!(writer_proxy.missing_changes(), [].iter().cloned().collect());
        assert_eq!(writer_proxy.behavior().must_send_ack(), false);
    }
}