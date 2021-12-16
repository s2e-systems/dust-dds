use crate::{
    messages::submessages::DataSubmessage,
    structure::types::{Guid, GuidPrefix},
};

use super::reader::{
    stateful_reader::RtpsStatefulReaderOperations, writer_proxy::RtpsWriterProxyOperations,
};

pub struct BestEffortStatefulReaderBehavior;

impl BestEffortStatefulReaderBehavior {
    pub fn receive_data<L, P>(
        stateful_reader: &impl RtpsStatefulReaderOperations<L,
            WriterProxyType = impl RtpsWriterProxyOperations,
        >,
        source_guid_prefix: GuidPrefix,
        data: &DataSubmessage<P, &[u8]>,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value); // writer_guid := {Receiver.SourceGuidPrefix, DATA.writerId};
        if let Some(writer_proxy) = stateful_reader.matched_writer_lookup(&writer_guid) {
            let expected_seq_nem = writer_proxy.available_changes_max(); // expected_seq_num := writer_proxy.available_changes_max() + 1;
        }
    }
}

pub struct ReliableStatefulReaderBehavior;

impl ReliableStatefulReaderBehavior {
    pub fn receive_data() {
        todo!("ReliableStatefulReaderBehavior receive data");
    }
}

// impl BestEffortWriterProxyBehavior {
//     pub fn try_process_message(
//         writer_proxy: &mut impl WriterProxy,
//         src_guid_prefix: GuidPrefix,
//         submessage: &mut Option<RtpsSubmessage>,
//         history_cache: &mut impl HistoryCache,
//     ) {
//         Self::waiting_state(writer_proxy, src_guid_prefix, submessage, history_cache);
//     }

//     fn waiting_state(
//         writer_proxy: &mut impl WriterProxy,
//         src_guid_prefix: GuidPrefix,
//         submessage: &mut Option<RtpsSubmessage>,
//         history_cache: &mut impl HistoryCache,
//     ) {
//         if let Some(inner_submessage) = submessage {
//             if Self::is_submessage_destination(writer_proxy, src_guid_prefix, inner_submessage) {
//                 match submessage.take().unwrap() {
//                     RtpsSubmessage::Data(data) => {
//                         Self::transition_t2(writer_proxy, history_cache, data)
//                     }
//                     RtpsSubmessage::Gap(gap) => Self::transition_t4(writer_proxy, gap),
//                     _ => panic!("Unexpected reader message received"),
//                 }
//             }
//         }
//     }

//     fn transition_t2(
//         writer_proxy: &mut impl WriterProxy,
//         history_cache: &mut impl HistoryCache,
//         data: Data,
//     ) {
//         let expected_seq_number = writer_proxy.available_changes_max() + 1;
//         if data.writer_sn >= expected_seq_number {
//             writer_proxy.received_change_set(data.writer_sn);
//             writer_proxy.lost_changes_update(data.writer_sn);
//             let cache_change =
//                 cache_change_from_data(data, &writer_proxy.remote_writer_guid().prefix());
//             history_cache.add_change(cache_change);
//         }
//     }

//     fn transition_t4(writer_proxy: &mut impl WriterProxy, gap: Gap) {
//         for seq_num in gap.gap_start..gap.gap_list.bitmap_base - 1 {
//             writer_proxy.irrelevant_change_set(seq_num);
//         }

//         todo!()

//         // for &seq_num in gap.gap_list.set() {
//         //     writer_proxy.irrelevant_change_set(seq_num);
//         // }
//     }

//     fn is_submessage_destination(
//         writer_proxy: &mut impl WriterProxy,
//         src_guid_prefix: GuidPrefix,
//         submessage: &RtpsSubmessage,
//     ) -> bool {
//         let writer_id = match submessage {
//             RtpsSubmessage::Data(data) => data.writer_id,
//             RtpsSubmessage::Gap(gap) => gap.writer_id,
//             _ => return false,
//         };

//         let writer_guid = GUID::new(src_guid_prefix, writer_id);
//         if writer_proxy.remote_writer_guid() == writer_guid {
//             true
//         } else {
//             false
//         }
//     }
// }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;
// //     use crate::messages::submessages::data_submessage::Payload;
// //     use crate::messages::types::{Endianness, KeyHash};
// //     use crate::types::constants::{
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //     };
// //     use crate::types::Locator;
// //     use crate::{
// //         behavior::change_kind_to_status_info,
// //         messages::submessages::submessage_elements::ParameterList, types::ChangeKind,
// //     };

// // #[test]
// // fn process_none_submessage() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let source_guid_prefix = [5; 12];
// //     let mut submessage = None;
// //     BestEffortWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut submessage,
// //         &mut history_cache,
// //     );
// // }

// // #[test]
// // fn process_data_submessage() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let source_guid_prefix = [5; 12];
// //     let status_info = change_kind_to_status_info(ChangeKind::Alive);
// //     let key_hash = KeyHash([1; 16]);
// //     let mut inline_qos = ParameterList::new();
// //     inline_qos.parameter.push(key_hash.into());
// //     inline_qos.parameter.push(status_info.into());
// //     let data_submessage = Data::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         1,
// //         Some(inline_qos),
// //         Payload::Data(vec![0, 1, 2]),
// //     );
// //     let expected_cache_change =
// //         cache_change_from_data(data_submessage.clone(), &source_guid_prefix);

// //     BestEffortWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Data(data_submessage)),
// //         &mut history_cache,
// //     );
// //     let received_change = history_cache.get_change(1).unwrap();
// //     assert_eq!(received_change, &expected_cache_change);
// // }
// // }

// use crate::{
//     behavior::cache_change_from_data,
//     messages::{
//         submessages::{submessage_elements::SequenceNumberSet, AckNack, Data, Gap, Heartbeat},
//         types::Count,
//         RtpsSubmessage,
//     },
//     structure::HistoryCache,
//     types::{EntityId, GuidPrefix, GUID},
// };

// use super::WriterProxy;

// pub struct ReliableWriterProxyBehavior;

// impl ReliableWriterProxyBehavior {
//     pub fn try_process_message(
//         writer_proxy: &mut impl WriterProxy,
//         src_guid_prefix: GuidPrefix,
//         submessage: &mut Option<RtpsSubmessage>,
//         history_cache: &mut impl HistoryCache,
//     ) {
//         if let Some(inner_submessage) = submessage {
//             if Self::is_submessage_destination(writer_proxy, src_guid_prefix, inner_submessage) {
//                 // If Waiting state (marked by the must_send_ack flag)
//                 // if !writer_proxy.behavior.must_send_ack {
//                 Self::waiting_heartbeat_state(writer_proxy, inner_submessage);
//                 // }

//                 Self::ready_state(writer_proxy, submessage, history_cache);
//             }
//         }
//     }

//     pub fn produce_messages(
//         writer_proxy: &mut impl WriterProxy,
//         reader_entity_id: EntityId,
//         acknack_count: Count,
//     ) -> Vec<RtpsSubmessage> {
//         let mut output_queue = Vec::new();
//         // if writer_proxy.behavior.must_send_ack {
//         Self::must_send_ack_state(
//             writer_proxy,
//             reader_entity_id,
//             acknack_count,
//             &mut output_queue,
//         );
//         // }
//         output_queue
//     }

//     fn ready_state(
//         writer_proxy: &mut impl WriterProxy,
//         submessage: &mut Option<RtpsSubmessage>,
//         history_cache: &mut impl HistoryCache,
//     ) {
//         match submessage.take().unwrap() {
//             RtpsSubmessage::Data(data) => Self::transition_t8(writer_proxy, history_cache, data),
//             RtpsSubmessage::Gap(gap) => Self::transition_t9(writer_proxy, gap),
//             RtpsSubmessage::Heartbeat(heartbeat) => Self::transition_t7(writer_proxy, heartbeat),
//             _ => panic!("Unexpected reader message received"),
//         }
//     }

//     fn is_submessage_destination(
//         writer_proxy: &mut impl WriterProxy,
//         src_guid_prefix: GuidPrefix,
//         submessage: &RtpsSubmessage,
//     ) -> bool {
//         let writer_id = match submessage {
//             RtpsSubmessage::Data(data) => data.writer_id,
//             RtpsSubmessage::Gap(gap) => gap.writer_id,
//             RtpsSubmessage::Heartbeat(heartbeat) => heartbeat.writer_id,
//             _ => return false,
//         };

//         let writer_guid = GUID::new(src_guid_prefix, writer_id);
//         if writer_proxy.remote_writer_guid() == writer_guid {
//             true
//         } else {
//             false
//         }
//     }

//     fn transition_t8(
//         writer_proxy: &mut impl WriterProxy,
//         history_cache: &mut impl HistoryCache,
//         data: Data,
//     ) {
//         let expected_seq_number = writer_proxy.available_changes_max() + 1;
//         if data.writer_sn >= expected_seq_number {
//             writer_proxy.received_change_set(data.writer_sn);
//             let cache_change =
//                 cache_change_from_data(data, &writer_proxy.remote_writer_guid().prefix());
//             history_cache.add_change(cache_change);
//         }
//     }

//     fn transition_t9(writer_proxy: &mut impl WriterProxy, gap: Gap) {
//         for seq_num in gap.gap_start..gap.gap_list.bitmap_base - 1 {
//             writer_proxy.irrelevant_change_set(seq_num);
//         }

//         todo!()
//         // for &seq_num in gap.gap_list.set() {
//         //     writer_proxy.irrelevant_change_set(seq_num);
//         // }
//     }

//     fn transition_t7(writer_proxy: &mut impl WriterProxy, heartbeat: Heartbeat) {
//         writer_proxy.missing_changes_update(heartbeat.last_sn);
//         writer_proxy.lost_changes_update(heartbeat.first_sn);
//     }

//     fn waiting_heartbeat_state(writer_proxy: &mut impl WriterProxy, submessage: &RtpsSubmessage) {
//         if let RtpsSubmessage::Heartbeat(heartbeat) = submessage {
//             if !heartbeat.final_flag
//                 || (heartbeat.final_flag && !writer_proxy.missing_changes().is_empty())
//             {
//                 todo!()
//                 // writer_proxy.behavior.time_heartbeat_received = Instant::now();
//                 // writer_proxy.behavior.must_send_ack = true;
//             }
//         }
//     }

//     fn must_send_ack_state(
//         writer_proxy: &mut impl WriterProxy,
//         reader_entity_id: EntityId,
//         acknack_count: Count,
//         output_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         // let duration_since_heartbeat_received: Duration = writer_proxy
//         //     .behavior
//         //     .time_heartbeat_received
//         //     .elapsed()
//         //     .try_into()
//         //     .unwrap();
//         // if duration_since_heartbeat_received > heartbeat_response_delay {
//         Self::transition_t5(writer_proxy, reader_entity_id, acknack_count, output_queue)
//         // }
//     }

//     fn transition_t5(
//         writer_proxy: &mut impl WriterProxy,
//         reader_entity_id: EntityId,
//         acknack_count: Count,
//         output_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         // writer_proxy.behavior.must_send_ack = false;

//         // writer_proxy.behavior.ackanck_count += 1;
//         let acknack = AckNack {
//             endianness_flag: false,
//             reader_id: reader_entity_id,
//             writer_id: writer_proxy.remote_writer_guid().entity_id(),
//             reader_sn_state: SequenceNumberSet {
//                 bitmap_base: writer_proxy.available_changes_max(),
//                 bitmap: [0; 8],
//             },
//             count: acknack_count,
//             final_flag: true,
//         };

//         output_queue.push(RtpsSubmessage::AckNack(acknack));
//     }
// }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;
// //     use crate::messages::submessages::data_submessage::Payload;
// //     use crate::messages::types::{Endianness, KeyHash};
// //     use crate::types::constants::{
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //     };
// //     use crate::types::Locator;
// //     use crate::{
// //         behavior::change_kind_to_status_info,
// //         messages::submessages::submessage_elements::ParameterList, types::ChangeKind,
// //     };

// // #[test]
// // fn process_none_submessage() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let source_guid_prefix = [5; 12];
// //     let mut submessage = None;
// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut submessage,
// //         &mut history_cache,
// //     );
// // }

// // #[test]
// // fn process_data_submessage() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let source_guid_prefix = [5; 12];
// //     let status_info = change_kind_to_status_info(ChangeKind::Alive);
// //     let key_hash = KeyHash([1; 16]);
// //     let mut inline_qos = ParameterList::new();
// //     inline_qos.parameter.push(key_hash.into());
// //     inline_qos.parameter.push(status_info.into());
// //     let data_submessage = Data::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         1,
// //         Some(inline_qos),
// //         Payload::Data(vec![0, 1, 2]),
// //     );
// //     let expected_cache_change =
// //         cache_change_from_data(data_submessage.clone(), &source_guid_prefix);

// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Data(data_submessage)),
// //         &mut history_cache,
// //     );
// //     let received_change = history_cache.get_change(1).unwrap();
// //     assert_eq!(received_change, &expected_cache_change);
// // }

// // #[test]
// // fn process_non_final_heartbeat_with_missing_changes() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let heartbeat_submessage = Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         3,
// //         6,
// //         1,
// //         false,
// //         false,
// //     );

// //     let source_guid_prefix = [5; 12];

// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
// //         &mut history_cache,
// //     );

// //     assert_eq!(
// //         writer_proxy.missing_changes(),
// //         [3, 4, 5, 6].iter().cloned().collect()
// //     );
// //     assert_eq!(writer_proxy.behavior.must_send_ack, true);
// // }

// // #[test]
// // fn process_final_heartbeat_with_missing_changes() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let heartbeat_submessage = Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         3,
// //         6,
// //         1,
// //         true,
// //         false,
// //     );

// //     let source_guid_prefix = [5; 12];
// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
// //         &mut history_cache,
// //     );

// //     assert_eq!(
// //         writer_proxy.missing_changes(),
// //         [3, 4, 5, 6].iter().cloned().collect()
// //     );
// //     assert_eq!(writer_proxy.behavior.must_send_ack, false);
// // }

// // #[test]
// // fn process_final_heartbeat_without_missing_changes() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let heartbeat_submessage = Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         1,
// //         0,
// //         1,
// //         true,
// //         false,
// //     );

// //     let source_guid_prefix = [5; 12];
// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
// //         &mut history_cache,
// //     );

// //     assert_eq!(writer_proxy.missing_changes(), [].iter().cloned().collect());
// //     assert_eq!(writer_proxy.behavior.must_send_ack, false);
// // }

// // #[test]
// // fn produce_acknack_heartbeat_response_without_missing_changes() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let heartbeat_submessage = Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         1,
// //         0,
// //         1,
// //         false,
// //         false,
// //     );

// //     let source_guid_prefix = [5; 12];
// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
// //         &mut history_cache,
// //     );

// //     let heartbeat_response_delay = Duration::from_millis(200);
// //     let produced_messages_empty = ReliableWriterProxyBehavior::produce_messages(
// //         &mut writer_proxy,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         heartbeat_response_delay,
// //     );

// //     std::thread::sleep(heartbeat_response_delay.into());

// //     let produced_messages = ReliableWriterProxyBehavior::produce_messages(
// //         &mut writer_proxy,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         heartbeat_response_delay,
// //     );

// //     let expected_acknack_submessage = RtpsSubmessage::AckNack(AckNack::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         0,
// //         [].iter().cloned().collect(),
// //         1,
// //         true,
// //     ));

// //     assert!(produced_messages_empty.is_empty());
// //     assert_eq!(produced_messages.len(), 1);
// //     assert!(produced_messages.contains(&expected_acknack_submessage));
// // }

// // #[test]
// // fn produce_acknack_heartbeat_response_with_missing_changes() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     let heartbeat_submessage = Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         3,
// //         6,
// //         1,
// //         false,
// //         false,
// //     );

// //     let source_guid_prefix = [5; 12];
// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
// //         &mut history_cache,
// //     );

// //     let heartbeat_response_delay = Duration::from_millis(0);
// //     let produced_messages = ReliableWriterProxyBehavior::produce_messages(
// //         &mut writer_proxy,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         heartbeat_response_delay,
// //     );

// //     let expected_acknack_submessage = RtpsSubmessage::AckNack(AckNack::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         2,
// //         [3, 4, 5, 6].iter().cloned().collect(),
// //         1,
// //         true,
// //     ));

// //     assert_eq!(produced_messages.len(), 1);
// //     assert!(produced_messages.contains(&expected_acknack_submessage));
// // }

// // #[test]
// // fn produce_acknack_heartbeat_response_with_partial_missing_changes() {
// //     let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let mut writer_proxy = WriterProxy::new(
// //         remote_writer_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //     );

// //     let mut history_cache = HistoryCache::default();

// //     writer_proxy.received_change_set(1);
// //     writer_proxy.received_change_set(2);
// //     writer_proxy.received_change_set(3);
// //     writer_proxy.received_change_set(4);

// //     let heartbeat_submessage = Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         3,
// //         6,
// //         1,
// //         false,
// //         false,
// //     );

// //     let source_guid_prefix = [5; 12];
// //     ReliableWriterProxyBehavior::try_process_message(
// //         &mut writer_proxy,
// //         source_guid_prefix,
// //         &mut Some(RtpsSubmessage::Heartbeat(heartbeat_submessage)),
// //         &mut history_cache,
// //     );

// //     let heartbeat_response_delay = Duration::from_millis(0);
// //     let produced_messages = ReliableWriterProxyBehavior::produce_messages(
// //         &mut writer_proxy,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         heartbeat_response_delay,
// //     );

// //     let expected_acknack_submessage = RtpsSubmessage::AckNack(AckNack::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         4,
// //         [5, 6].iter().cloned().collect(),
// //         1,
// //         true,
// //     ));

// //     assert_eq!(produced_messages.len(), 1);
// //     assert!(produced_messages.contains(&expected_acknack_submessage));
// // }
// // }
