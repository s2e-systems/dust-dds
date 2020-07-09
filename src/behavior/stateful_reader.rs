use crate::types::{GUID, };
use crate::behavior::types::Duration;
use crate::messages::{RtpsMessage, RtpsSubmessage, AckNack};
use crate::messages::submessage_elements;
use crate::cache::{HistoryCache};
use crate::stateful_reader::WriterProxy;

use crate::serdes::Endianness;
use super::cache_change_from_data;

pub struct StatefulReaderBehavior {}

impl StatefulReaderBehavior {
    pub fn run_best_effort(writer_proxy: &mut WriterProxy, _reader_guid: &GUID, history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) -> Option<Vec<RtpsSubmessage>> {
        StatefulReaderBehavior::run_waiting_state(writer_proxy, history_cache, received_message);
        None
    }

    pub fn run_reliable(writer_proxy: &mut WriterProxy, reader_guid: &GUID, history_cache: &mut HistoryCache, heartbeat_response_delay: Duration, received_message: Option<&RtpsMessage>) -> Option<Vec<RtpsSubmessage>>{
        StatefulReaderBehavior::run_ready_state(writer_proxy, history_cache, received_message);
        if writer_proxy.must_send_ack() {
            // This is the only case in which a message is sent by the stateful reader
            StatefulReaderBehavior::run_must_send_ack_state(writer_proxy, reader_guid, heartbeat_response_delay)
        } else {
            StatefulReaderBehavior::run_waiting_heartbeat_state(writer_proxy, received_message);
            None
        }
    }

    fn run_waiting_state(writer_proxy: &mut WriterProxy, history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            let guid_prefix = received_message.header().guid_prefix();
            for submessage in received_message.submessages().iter() {                
                if let RtpsSubmessage::Data(data) = submessage {
                    let expected_seq_number = writer_proxy.available_changes_max() + 1;
                    if data.writer_sn().0 >= expected_seq_number {
                        let cache_change = cache_change_from_data(data, guid_prefix);
                        history_cache.add_change(cache_change);
                        writer_proxy.received_change_set(data.writer_sn().0);
                        writer_proxy.lost_changes_update(data.writer_sn().0);
                    }
                } else if let RtpsSubmessage::Gap(gap) = submessage {
                    for seq_num in gap.gap_start().0 .. gap.gap_list().base() - 1 {
                        writer_proxy.irrelevant_change_set(seq_num);
                    }

                    for &seq_num in gap.gap_list().set() {
                        writer_proxy.irrelevant_change_set(seq_num);
                    }
                }
            }
        }
    }

    fn run_ready_state(writer_proxy: &mut WriterProxy, history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            let guid_prefix = received_message.header().guid_prefix();
            for submessage in received_message.submessages().iter() {                
                if let RtpsSubmessage::Data(data) = submessage {
                    let expected_seq_number = writer_proxy.available_changes_max() + 1;
                    if data.writer_sn().0 >= expected_seq_number {
                        let cache_change = cache_change_from_data(data, guid_prefix);
                        history_cache.add_change(cache_change);
                        writer_proxy.received_change_set(data.writer_sn().0);
                    }
                } else if let RtpsSubmessage::Gap(gap) = submessage {
                    // for seq_num in gap.gap_start() .. gap.gap_list().base() - 1 {
                    //     writer_proxy.irrelevant_change_set(seq_num);
                    // }

                    for &seq_num in gap.gap_list().set() {
                        writer_proxy.irrelevant_change_set(seq_num);
                    }
                } 
                // The heartbeat reception is moved to the waiting state since it has to be read there anyway
            }
        }
    }

    fn run_waiting_heartbeat_state(writer_proxy: &mut WriterProxy, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            let _guid_prefix = received_message.header().guid_prefix();
            for submessage in received_message.submessages().iter() {                
                if let RtpsSubmessage::Heartbeat(heartbeat) = submessage {
                    writer_proxy.missing_changes_update(heartbeat.last_sn().0);
                    writer_proxy.lost_changes_update(heartbeat.first_sn().0);
                    if !heartbeat.is_final() || 
                        (heartbeat.is_final() && !writer_proxy.missing_changes().is_empty()) {
                        writer_proxy.set_must_send_ack(true);
                        writer_proxy.time_heartbeat_received_reset();
                    } 
                }
            }
        }
    }

    fn run_must_send_ack_state(writer_proxy: &mut WriterProxy, reader_guid: &GUID, heartbeat_response_delay: Duration) -> Option<Vec<RtpsSubmessage>> {
        if writer_proxy.duration_since_heartbeat_received() >  heartbeat_response_delay {
            writer_proxy.set_must_send_ack(false);
            let reader_sn_state = submessage_elements::SequenceNumberSet::new(
                writer_proxy.available_changes_max(),
                writer_proxy.missing_changes().clone()
            );
            writer_proxy.increment_acknack_count();
            let acknack = AckNack::new(
                submessage_elements::EntityId(*reader_guid.entity_id()), 
                submessage_elements::EntityId(*writer_proxy.remote_writer_guid().entity_id()),
                reader_sn_state,
                submessage_elements::Count(*writer_proxy.ackanck_count()),
                true,
                Endianness::LittleEndian);

            Some(vec![RtpsSubmessage::AckNack(acknack)])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::{SequenceNumber, ChangeKind, GuidPrefix};
    // use crate::messages::types::Count;
    // use crate::types::constants::{
    //     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, };
    // use crate::cache::CacheChange;
    // use crate::messages::{Data, Payload, Heartbeat};
    // use crate::messages::submessage_elements::ParameterList;
    // use crate::serdes::Endianness;
    // use crate::serialized_payload::SerializedPayload;
    // use crate::inline_qos_types::{KeyHash, StatusInfo, };

    // #[test]
    // fn run_waiting_state_data_only() {
    //     let mut history_cache = HistoryCache::new();
    //     let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

    //     let mut submessages = Vec::new();
    //     let mut inline_qos = ParameterList::new();
    //     inline_qos.push(StatusInfo::from(ChangeKind::Alive));
    //     inline_qos.push(KeyHash([1;16]));

    //     let data1 = Data::new(
    //         Endianness::LittleEndian, 
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, 
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
    //         SequenceNumber(3),
    //         Some(inline_qos),
    //         Payload::Data(SerializedPayload(vec![1,2,3])));
    //     submessages.push(RtpsSubmessage::Data(data1));

    //     let received_message = RtpsMessage::new(*remote_writer_guid.prefix(), submessages);

    //     StatefulReaderBehavior::run_waiting_state(&mut writer_proxy, &mut history_cache, Some(&received_message));

    //     let expected_change_1 = CacheChange::new(
    //         ChangeKind::Alive,
    //         remote_writer_guid,
    //         [1;16],
    //         SequenceNumber(3),
    //         Some(vec![1,2,3]),
    //         None,
    //     );

    //     assert_eq!(history_cache.get_changes().len(), 1);
    //     assert!(history_cache.get_changes().contains(&expected_change_1));
    //     assert_eq!(writer_proxy.available_changes_max(), SequenceNumber(3));

    //     // Run waiting state without any received message and verify nothing changes
    //     StatefulReaderBehavior::run_waiting_state(&mut writer_proxy, &mut history_cache, None);
    //     assert_eq!(history_cache.get_changes().len(), 1);
    //     assert_eq!(writer_proxy.available_changes_max(), SequenceNumber(3));
    // }

    // #[test]
    // fn run_ready_state_data_only() {
    //     let mut history_cache = HistoryCache::new();
    //     let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

    //     let mut submessages = Vec::new();
    //     let mut inline_qos = ParameterList::new();
    //     inline_qos.push(StatusInfo::from(ChangeKind::Alive));
    //     inline_qos.push(KeyHash([1;16]));

    //     let data1 = Data::new(
    //         Endianness::LittleEndian, 
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, 
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
    //         SequenceNumber(3),
    //         Some(inline_qos),
    //         Payload::Data(SerializedPayload(vec![1,2,3])));
    //     submessages.push(RtpsSubmessage::Data(data1));

    //     let received_message = RtpsMessage::new(*remote_writer_guid.prefix(), submessages);

    //     StatefulReaderBehavior::run_ready_state(&mut writer_proxy, &mut history_cache, Some(&received_message));

    //     let expected_change_1 = CacheChange::new(
    //         ChangeKind::Alive,
    //         remote_writer_guid,
    //         [1;16],
    //         SequenceNumber(3),
    //         Some(vec![1,2,3]),
    //         None,
    //     );

    //     assert_eq!(history_cache.get_changes().len(), 1);
    //     assert!(history_cache.get_changes().contains(&expected_change_1));
    //     assert_eq!(writer_proxy.available_changes_max(), SequenceNumber(0));

    //     // Run waiting state without any received message and verify nothing changes
    //     StatefulReaderBehavior::run_waiting_state(&mut writer_proxy, &mut history_cache, None);
    //     assert_eq!(history_cache.get_changes().len(), 1);
    //     assert_eq!(writer_proxy.available_changes_max(), SequenceNumber(0));
    // }

    // #[test]
    // fn run_waiting_heartbeat_state_non_final_heartbeat() {
    //     let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

    //     let mut submessages = Vec::new();
    //     let heartbeat = Heartbeat::new(
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    //         *remote_writer_guid.entity_id(),
    //         SequenceNumber(3),
    //         SequenceNumber(6),
    //         Count(1),
    //         false,
    //         false,
    //         Endianness::LittleEndian,
    //     );
    //     submessages.push(RtpsSubmessage::Heartbeat(heartbeat));
    //     let received_message = RtpsMessage::new(*remote_writer_guid.prefix(), submessages);       

    //     StatefulReaderBehavior::run_waiting_heartbeat_state(&mut writer_proxy, Some(&received_message));
    //     assert_eq!(writer_proxy.missing_changes(), &[SequenceNumber(3), SequenceNumber(4), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect());
    //     assert_eq!(writer_proxy.must_send_ack(), true);
    // }
    
    // #[test]
    // fn run_waiting_heartbeat_state_final_heartbeat_with_missing_changes() {
    //     let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

    //     let mut submessages = Vec::new();
    //     let heartbeat = Heartbeat::new(
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    //         *remote_writer_guid.entity_id(),
    //         SequenceNumber(2),
    //         SequenceNumber(3),
    //         Count(1),
    //         true,
    //         false,
    //         Endianness::LittleEndian,
    //     );
    //     submessages.push(RtpsSubmessage::Heartbeat(heartbeat));
    //     let received_message = RtpsMessage::new(*remote_writer_guid.prefix(), submessages);       

    //     StatefulReaderBehavior::run_waiting_heartbeat_state(&mut writer_proxy, Some(&received_message));
    //     assert_eq!(writer_proxy.missing_changes(), &[SequenceNumber(2), SequenceNumber(3)].iter().cloned().collect());
    //     assert_eq!(writer_proxy.must_send_ack(), true);
    // }

    // #[test]
    // fn run_waiting_heartbeat_state_final_heartbeat_without_missing_changes() {
    //     let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

    //     let mut submessages = Vec::new();
    //     let heartbeat = Heartbeat::new(
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    //         *remote_writer_guid.entity_id(),
    //         SequenceNumber(1),
    //         SequenceNumber(0),
    //         Count(1),
    //         true,
    //         false,
    //         Endianness::LittleEndian,
    //     );
    //     submessages.push(RtpsSubmessage::Heartbeat(heartbeat));
    //     let received_message = RtpsMessage::new(*remote_writer_guid.prefix(), submessages);       

    //     StatefulReaderBehavior::run_waiting_heartbeat_state(&mut writer_proxy, Some(&received_message));
    //     assert_eq!(writer_proxy.missing_changes(), &[].iter().cloned().collect());
    //     assert_eq!(writer_proxy.must_send_ack(), false);
    // }

    // #[test]
    // fn run_waiting_heartbeat_state_and_must_send_ack_state() {

    //     let reader_guid = GUID::new(GuidPrefix([2;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
    //     let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

    //     let mut submessages = Vec::new();
    //     let heartbeat = Heartbeat::new(
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    //         *remote_writer_guid.entity_id(),
    //         SequenceNumber(3),
    //         SequenceNumber(6),
    //         Count(1),
    //         false,
    //         false,
    //         Endianness::LittleEndian,
    //     );
    //     submessages.push(RtpsSubmessage::Heartbeat(heartbeat));
    //     let received_message = RtpsMessage::new(*remote_writer_guid.prefix(), submessages);       

    //     StatefulReaderBehavior::run_waiting_heartbeat_state(&mut writer_proxy, Some(&received_message));
    //     assert_eq!(writer_proxy.missing_changes(), &[SequenceNumber(3), SequenceNumber(4), SequenceNumber(5), SequenceNumber(6)].iter().cloned().collect());
    //     assert_eq!(writer_proxy.must_send_ack(), true);

    //     let heartbeat_response_delay = Duration::from_millis(300);
    //     let message = StatefulReaderBehavior::run_must_send_ack_state(&mut writer_proxy, &reader_guid, heartbeat_response_delay);
    //     assert!(message.is_none());

    //     std::thread::sleep(heartbeat_response_delay.into());

    //     let message = StatefulReaderBehavior::run_must_send_ack_state(&mut writer_proxy, &reader_guid, heartbeat_response_delay).unwrap();
    //     assert_eq!(message.len(), 1);
    //     if let RtpsSubmessage::AckNack(acknack) = &message[0] {
    //         assert_eq!(acknack.writer_id(), remote_writer_guid.entity_id());
    //         assert_eq!(acknack.reader_id(), reader_guid.entity_id());
    //         assert_eq!(acknack.count(), &Count(1));
    //         assert_eq!(acknack.reader_sn_state().base(), &SequenceNumber(2));
    //         assert_eq!(acknack.reader_sn_state().set(), writer_proxy.missing_changes());
    //     } else {
    //         panic!("Wrong message type");
    //     }
    // }
}