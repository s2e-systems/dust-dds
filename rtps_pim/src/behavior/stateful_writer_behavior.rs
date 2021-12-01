// use crate::{
//     behavior::data_from_cache_change,
//     messages::{
//         submessages::{submessage_elements::SequenceNumberSet, AckNack, Gap, Heartbeat},
//         types::Count,
//         RtpsSubmessage,
//     },
//     structure::HistoryCache,
//     types::{EntityId, GuidPrefix, SequenceNumber, GUID},
// };

// use super::{reader_proxy::ChangeForReader, ReaderProxy};

// pub struct ReliableReaderProxyBehavior;

// impl ReliableReaderProxyBehavior {
//     pub fn produce_messages<T: ChangeForReader<CacheChangeRepresentation = SequenceNumber>>(
//         reader_proxy: &mut impl ReaderProxy<ChangeForReaderType = T>,
//         history_cache: &impl HistoryCache,
//         writer_entity_id: EntityId,
//         last_change_sequence_number: SequenceNumber,
//         heartbeat_count: Count,
//     ) -> Vec<RtpsSubmessage> {
//         let mut message_queue = Vec::new();

//         if reader_proxy.unacked_changes().is_empty() {
//             // Idle
//         } else if !reader_proxy.unsent_changes().is_empty() {
//             Self::pushing_state(
//                 reader_proxy,
//                 history_cache,
//                 writer_entity_id,
//                 &mut message_queue,
//             );
//         } else if !reader_proxy.unacked_changes().is_empty() {
//             Self::announcing_state(
//                 reader_proxy,
//                 history_cache,
//                 last_change_sequence_number,
//                 writer_entity_id,
//                 heartbeat_count,
//                 &mut message_queue,
//             );
//         }

//         if !reader_proxy.requested_changes().is_empty() {
//             // let duration_since_nack_received: Duration = reader_proxy
//             //     .behavior
//             //     .time_nack_received
//             //     .elapsed()
//             //     .try_into()
//             //     .unwrap();
//             // if duration_since_nack_received > nack_response_delay {
//             Self::repairing_state(
//                 reader_proxy,
//                 history_cache,
//                 writer_entity_id,
//                 &mut message_queue,
//             );
//             // }
//         }

//         message_queue
//     }

//     pub fn try_process_message<T: ChangeForReader<CacheChangeRepresentation = SequenceNumber>>(
//         reader_proxy: &mut impl ReaderProxy<ChangeForReaderType = T>,
//         src_guid_prefix: GuidPrefix,
//         submessage: &mut Option<RtpsSubmessage>,
//         highest_nack_count_received: &mut Count,
//     ) {
//         if let Some(RtpsSubmessage::AckNack(acknack)) = submessage {
//             let reader_guid = GUID::new(src_guid_prefix, acknack.reader_id);
//             if reader_proxy.remote_reader_guid() == reader_guid {
//                 if let RtpsSubmessage::AckNack(acknack) = submessage.take().unwrap() {
//                     if &acknack.count > highest_nack_count_received {
//                         *highest_nack_count_received = acknack.count;
//                         if reader_proxy.requested_changes().is_empty() {
//                             Self::waiting_state(reader_proxy, acknack);
//                         } else {
//                             Self::must_repair_state(reader_proxy, acknack);
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     fn pushing_state<T: ChangeForReader<CacheChangeRepresentation = SequenceNumber>>(
//         reader_proxy: &mut impl ReaderProxy<ChangeForReaderType = T>,
//         history_cache: &impl HistoryCache,
//         writer_entity_id: EntityId,
//         message_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         while let Some(next_unsent_change) = reader_proxy.next_unsent_change() {
//             let next_unsent_seq_num = next_unsent_change.change().clone();
//             Self::transition_t4(
//                 reader_proxy,
//                 history_cache,
//                 next_unsent_seq_num,
//                 writer_entity_id,
//                 message_queue,
//             );
//         }
//         // reader_proxy.behavior.time_last_sent_data = Instant::now();
//     }

//     fn transition_t4(
//         reader_proxy: &mut impl ReaderProxy,
//         history_cache: &impl HistoryCache,
//         next_unsent_seq_num: SequenceNumber,
//         writer_entity_id: EntityId,
//         message_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         if let Some(cache_change) = history_cache.get_change(next_unsent_seq_num) {
//             let reader_id = reader_proxy.remote_reader_guid().entity_id();
//             let data = data_from_cache_change(cache_change, reader_id);
//             message_queue.push(RtpsSubmessage::Data(data));
//         } else {
//             let gap = Gap {
//                 endianness_flag: false,
//                 reader_id: reader_proxy.remote_reader_guid().entity_id(),
//                 writer_id: writer_entity_id,
//                 gap_start: next_unsent_seq_num,
//                 gap_list: SequenceNumberSet {
//                     bitmap_base: next_unsent_seq_num,
//                     bitmap: [0; 8],
//                 },
//             };

//             message_queue.push(RtpsSubmessage::Gap(gap));
//         }
//     }

//     fn announcing_state(
//         reader_proxy: &mut impl ReaderProxy,
//         history_cache: &impl HistoryCache,
//         last_change_sequence_number: SequenceNumber,
//         writer_entity_id: EntityId,
//         heartbeat_count: Count,
//         message_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         // let duration_since_last_sent_data: Duration = reader_proxy
//         //     .behavior
//         //     .time_last_sent_data
//         //     .elapsed()
//         //     .try_into()
//         //     .unwrap();
//         // if duration_since_last_sent_data > heartbeat_period {
//         Self::transition_t7(
//             reader_proxy,
//             history_cache,
//             last_change_sequence_number,
//             writer_entity_id,
//             heartbeat_count,
//             message_queue,
//         );
//         // reader_proxy.behavior.time_last_sent_data = Instant::now();
//         // }
//     }

//     fn transition_t7(
//         reader_proxy: &mut impl ReaderProxy,
//         history_cache: &impl HistoryCache,
//         last_change_sequence_number: SequenceNumber,
//         writer_entity_id: EntityId,
//         heartbeat_count: Count,
//         message_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         let first_sn = if let Some(seq_num) = history_cache.get_seq_num_min() {
//             seq_num
//         } else {
//             last_change_sequence_number + 1
//         };
//         // reader_proxy.behavior.heartbeat_count += 1;

//         let heartbeat = Heartbeat {
//             endianness_flag: false,
//             final_flag: false,
//             liveliness_flag: false,
//             reader_id: reader_proxy.remote_reader_guid().entity_id(),
//             writer_id: writer_entity_id,
//             first_sn,
//             last_sn: last_change_sequence_number,
//             count: heartbeat_count,
//         };
//         message_queue.push(RtpsSubmessage::Heartbeat(heartbeat));
//     }

//     fn waiting_state(reader_proxy: &mut impl ReaderProxy, acknack: AckNack) {
//         Self::transition_t8(reader_proxy, acknack);
//     }

//     fn transition_t8(reader_proxy: &mut impl ReaderProxy, acknack: AckNack) {
//         reader_proxy.acked_changes_set(acknack.reader_sn_state.bitmap_base - 1);
//         // reader_proxy.requested_changes_set(acknack.reader_sn_state());
//         todo!()
//     }

//     fn must_repair_state(reader_proxy: &mut impl ReaderProxy, acknack: AckNack) {
//         Self::transition_t8(reader_proxy, acknack);
//     }

//     fn repairing_state<T: ChangeForReader<CacheChangeRepresentation = SequenceNumber>>(
//         reader_proxy: &mut impl ReaderProxy<ChangeForReaderType = T>,
//         history_cache: &impl HistoryCache,
//         writer_entity_id: EntityId,
//         message_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         while let Some(next_requested_change) = reader_proxy.next_requested_change() {
//             let next_requested_seq_num = *next_requested_change.change();
//             Self::transition_t12(
//                 reader_proxy,
//                 history_cache,
//                 next_requested_seq_num,
//                 writer_entity_id,
//                 message_queue,
//             );
//         }
//     }

//     fn transition_t12(
//         reader_proxy: &mut impl ReaderProxy,
//         history_cache: &impl HistoryCache,
//         next_requested_seq_num: SequenceNumber,
//         writer_entity_id: EntityId,
//         message_queue: &mut Vec<RtpsSubmessage>,
//     ) {
//         if let Some(cache_change) = history_cache.get_change(next_requested_seq_num) {
//             let data =
//                 data_from_cache_change(cache_change, reader_proxy.remote_reader_guid().entity_id());
//             message_queue.push(RtpsSubmessage::Data(data));
//         } else {
//             let gap = Gap {
//                 endianness_flag: false,
//                 reader_id: reader_proxy.remote_reader_guid().entity_id(),
//                 writer_id: writer_entity_id,
//                 gap_start: next_requested_seq_num,
//                 gap_list: SequenceNumberSet {
//                     bitmap_base: next_requested_seq_num,
//                     bitmap: [0; 8],
//                 },
//             };
//             message_queue.push(RtpsSubmessage::Gap(gap));
//         }
//     }
// }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;
// //     use crate::types::constants::{
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //     };
// //     use crate::types::{Locator, GUID};
// //     use crate::{messages::types::Endianness, types::ChangeKind};

// //     use crate::structure::CacheChange;

// // #[test]
// // fn produce_empty() {
// //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let history_cache = HistoryCache::default();

// //     // Run without any change being created or added in the cache
// //     let heartbeat_period = Duration::from_secs(1);
// //     let nack_response_delay = Duration::from_secs(1);
// //     let last_change_sequence_number = 0;
// //     let messages_vec = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     assert!(messages_vec.is_empty());
// // }

// // #[test]
// // fn produce_data_message() {
// //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let mut history_cache = HistoryCache::default();

// //     // Add one change to the history cache
// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change1 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         1,
// //         Some(vec![1, 2, 3]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change1.clone());

// //     // Run with the last change sequence number equal to the added cache change
// //     let last_change_sequence_number = 1;
// //     let heartbeat_period = Duration::from_secs(1);
// //     let nack_response_delay = Duration::from_secs(1);

// //     let messages_vec = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
// //         &cache_change1,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //     ));
// //     assert_eq!(messages_vec.len(), 1);
// //     assert!(messages_vec.contains(&expected_data_submessage));
// // }

// // #[test]
// // fn produce_gap_message() {
// //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let history_cache = HistoryCache::default();

// //     // Run with the a sequence number of 1 without adding any change to the history cache
// //     let last_change_sequence_number = 1;
// //     let heartbeat_period = Duration::from_secs(1);
// //     let nack_response_delay = Duration::from_secs(1);
// //     let messages_vec = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
// //         BEHAVIOR_ENDIANNESS,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         writer_entity_id,
// //         1,
// //         &[],
// //     ));
// //     assert_eq!(messages_vec.len(), 1);
// //     assert!(messages_vec.contains(&expected_gap_submessage));
// // }

// // #[test]
// // fn produce_data_and_gap_messages() {
// //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let mut history_cache = HistoryCache::default();

// //     // Add one change to the history cache
// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change1 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         1,
// //         Some(vec![1, 2, 3]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change1.clone());

// //     // Run with the last change sequence number one above the added cache change
// //     let last_change_sequence_number = 2;
// //     let heartbeat_period = Duration::from_secs(1);
// //     let nack_response_delay = Duration::from_secs(1);
// //     let messages_vec = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
// //         &cache_change1,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //     ));
// //     let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
// //         BEHAVIOR_ENDIANNESS,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         writer_entity_id,
// //         2,
// //         &[],
// //     ));
// //     assert_eq!(messages_vec.len(), 2);
// //     assert!(messages_vec.contains(&expected_data_submessage));
// //     assert!(messages_vec.contains(&expected_gap_submessage));
// // }

// // #[test]
// // fn try_process_acknack_message_only_acknowledge() {
// //     let remote_reader_guid_prefix = [5; 12];
// //     let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
// //     let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;

// //     let acknack = AckNack::new(
// //         Endianness::LittleEndian,
// //         remote_reader_guid.entity_id(),
// //         writer_entity_id,
// //         2,
// //         vec![].iter().cloned().collect(),
// //         1,
// //         true,
// //     );

// //     let mut highest_nack_count_received = 0;
// //     let mut time_nack_received = Instant::now();

// //     ReliableReaderProxyBehavior::try_process_message(
// //         &mut reader_proxy,
// //         remote_reader_guid_prefix,
// //         &mut Some(RtpsSubmessage::AckNack(acknack)),
// //         &mut highest_nack_count_received,
// //         &mut time_nack_received,
// //     );

// //     assert_eq!(highest_nack_count_received, 1);
// //     assert!(reader_proxy.unacked_changes(1).is_empty()); // If 1 is the last change sequence number there are no unacked changes
// //     assert!(reader_proxy.unacked_changes(2).contains(&2)); // If 2 is the last change sequence number, then 2 is an unacked change
// // }

// // #[test]
// // fn try_process_acknack_message_acknowledge_and_request() {
// //     let remote_reader_guid_prefix = [5; 12];
// //     let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
// //     let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;

// //     let acknack = AckNack::new(
// //         Endianness::LittleEndian,
// //         remote_reader_guid.entity_id(),
// //         writer_entity_id,
// //         4,
// //         vec![1, 3].iter().cloned().collect(),
// //         1,
// //         true,
// //     );

// //     let mut highest_nack_count_received = 0;
// //     let mut time_nack_received = Instant::now();

// //     ReliableReaderProxyBehavior::try_process_message(
// //         &mut reader_proxy,
// //         remote_reader_guid_prefix,
// //         &mut Some(RtpsSubmessage::AckNack(acknack)),
// //         &mut highest_nack_count_received,
// //         &mut time_nack_received,
// //     );

// //     let requested_changes = reader_proxy.requested_changes();
// //     assert!(requested_changes.contains(&1));
// //     assert!(requested_changes.contains(&3));
// // }

// // #[test]
// // fn ignore_try_process_acknack_message_with_same_count_number() {
// //     let remote_reader_guid_prefix = [5; 12];
// //     let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
// //     let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;

// //     let acknack = AckNack::new(
// //         Endianness::LittleEndian,
// //         remote_reader_guid.entity_id(),
// //         writer_entity_id,
// //         4,
// //         vec![1, 3].iter().cloned().collect(),
// //         1,
// //         true,
// //     );

// //     let mut highest_nack_count_received = 0;
// //     let mut time_nack_received = Instant::now();

// //     ReliableReaderProxyBehavior::try_process_message(
// //         &mut reader_proxy,
// //         remote_reader_guid_prefix,
// //         &mut Some(RtpsSubmessage::AckNack(acknack)),
// //         &mut highest_nack_count_received,
// //         &mut time_nack_received,
// //     );

// //     let acknack_same_count = AckNack::new(
// //         Endianness::LittleEndian,
// //         remote_reader_guid.entity_id(),
// //         writer_entity_id,
// //         6,
// //         vec![1, 2, 3].iter().cloned().collect(),
// //         1,
// //         true,
// //     );

// //     let mut highest_nack_count_received = 0;
// //     let mut time_nack_received = Instant::now();

// //     ReliableReaderProxyBehavior::try_process_message(
// //         &mut reader_proxy,
// //         remote_reader_guid_prefix,
// //         &mut Some(RtpsSubmessage::AckNack(acknack_same_count)),
// //         &mut highest_nack_count_received,
// //         &mut time_nack_received,
// //     );

// //     assert_eq!(highest_nack_count_received, 1);
// //     let requested_changes = reader_proxy.requested_changes();
// //     assert!(requested_changes.contains(&1));
// //     assert!(requested_changes.contains(&3));
// // }

// // #[test]
// // fn produce_heartbeat_message() {
// //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let mut history_cache = HistoryCache::default();

// //     // Add one change to the history cache
// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change1 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         1,
// //         Some(vec![1, 2, 3]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change1.clone());

// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change2 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         2,
// //         Some(vec![4, 5, 6]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change2.clone());

// //     let last_change_sequence_number = 2;
// //     let heartbeat_period = Duration::from_secs(0);
// //     let nack_response_delay = Duration::from_secs(1);

// //     // The first produce should generate the data/gap messages and no heartbeat so we ignore it
// //     ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let messages_vec1 = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let messages_vec2 = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let expected_heartbeat_message1 = RtpsSubmessage::Heartbeat(Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         1,
// //         2,
// //         1,
// //         false,
// //         false,
// //     ));

// //     let expected_heartbeat_message2 = RtpsSubmessage::Heartbeat(Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         1,
// //         2,
// //         2,
// //         false,
// //         false,
// //     ));

// //     assert_eq!(messages_vec1.len(), 1);
// //     assert!(messages_vec1.contains(&expected_heartbeat_message1));
// //     assert_eq!(messages_vec2.len(), 1);
// //     assert!(messages_vec2.contains(&expected_heartbeat_message2));
// // }

// // #[test]
// // fn produce_heartbeat_message_period() {
// //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let mut history_cache = HistoryCache::default();

// //     // Add one change to the history cache
// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change1 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         1,
// //         Some(vec![1, 2, 3]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change1.clone());

// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change2 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         2,
// //         Some(vec![4, 5, 6]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change2.clone());

// //     let last_change_sequence_number = 2;
// //     let heartbeat_period = Duration::from_millis(200);
// //     let nack_response_delay = Duration::from_secs(1);

// //     // The first produce should generate the data/gap messages and no heartbeat so we ignore it
// //     ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let messages_vec1 = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     std::thread::sleep(heartbeat_period.into());

// //     let messages_vec2 = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let expected_heartbeat_message = RtpsSubmessage::Heartbeat(Heartbeat::new(
// //         Endianness::LittleEndian,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// //         1,
// //         2,
// //         1,
// //         false,
// //         false,
// //     ));

// //     assert!(messages_vec1.is_empty());
// //     assert_eq!(messages_vec2.len(), 1);
// //     assert!(messages_vec2.contains(&expected_heartbeat_message));
// // }

// // #[test]
// // fn produce_requested_changes_messages() {
// //     let remote_reader_guid_prefix = [5; 12];
// //     let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
// //     let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let mut history_cache = HistoryCache::default();

// //     // Add one change to the history cache
// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change1 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         1,
// //         Some(vec![1, 2, 3]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change1.clone());

// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change2 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         2,
// //         Some(vec![4, 5, 6]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change2.clone());

// //     let last_change_sequence_number = 2;
// //     let heartbeat_period = Duration::from_secs(1);
// //     let nack_response_delay = Duration::from_secs(0);

// //     let acknack = AckNack::new(
// //         Endianness::LittleEndian,
// //         remote_reader_guid.entity_id(),
// //         writer_entity_id,
// //         3,
// //         vec![1].iter().cloned().collect(),
// //         1,
// //         true,
// //     );

// //     ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let mut highest_nack_count_received = 0;
// //     let mut time_nack_received = Instant::now();

// //     ReliableReaderProxyBehavior::try_process_message(
// //         &mut reader_proxy,
// //         remote_reader_guid_prefix,
// //         &mut Some(RtpsSubmessage::AckNack(acknack)),
// //         &mut highest_nack_count_received,
// //         &mut time_nack_received,
// //     );

// //     let messages_vec = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
// //         &cache_change1,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //     ));
// //     assert_eq!(messages_vec.len(), 1);
// //     assert!(messages_vec.contains(&expected_data_submessage));
// // }

// // #[test]
// // fn produce_requested_changes_with_nack_response_delay() {
// //     let remote_reader_guid_prefix = [5; 12];
// //     let remote_reader_guid_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;
// //     let remote_reader_guid = GUID::new(remote_reader_guid_prefix, remote_reader_guid_entity_id);
// //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
// //     let multicast_locator_list = vec![];
// //     let expects_inline_qos = false;
// //     let is_active = true;
// //     let mut reader_proxy = ReaderProxy::new(
// //         remote_reader_guid,
// //         unicast_locator_list,
// //         multicast_locator_list,
// //         expects_inline_qos,
// //         is_active,
// //     );

// //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
// //     let mut history_cache = HistoryCache::default();

// //     // Add one change to the history cache
// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change1 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         1,
// //         Some(vec![1, 2, 3]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change1.clone());

// //     let writer_guid = GUID::new([5; 12], writer_entity_id);
// //     let instance_handle = [1; 16];
// //     let cache_change2 = CacheChange::new(
// //         ChangeKind::Alive,
// //         writer_guid.into(),
// //         instance_handle,
// //         2,
// //         Some(vec![4, 5, 6]),
// //         None,
// //     );
// //     history_cache.add_change(cache_change2.clone());

// //     let last_change_sequence_number = 2;
// //     let heartbeat_period = Duration::from_secs(1);
// //     let nack_response_delay = Duration::from_millis(200);

// //     let acknack = AckNack::new(
// //         Endianness::LittleEndian,
// //         remote_reader_guid.entity_id(),
// //         writer_entity_id,
// //         3,
// //         vec![1].iter().cloned().collect(),
// //         1,
// //         true,
// //     );

// //     ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let mut highest_nack_count_received = 0;
// //     let mut time_nack_received = Instant::now();

// //     ReliableReaderProxyBehavior::try_process_message(
// //         &mut reader_proxy,
// //         remote_reader_guid_prefix,
// //         &mut Some(RtpsSubmessage::AckNack(acknack)),
// //         &mut highest_nack_count_received,
// //         &mut time_nack_received,
// //     );

// //     let messages_vec_expected_empty = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     std::thread::sleep(nack_response_delay.into());

// //     let messages_vec = ReliableReaderProxyBehavior::produce_messages(
// //         &mut reader_proxy,
// //         &history_cache,
// //         writer_entity_id,
// //         last_change_sequence_number,
// //         heartbeat_period,
// //         nack_response_delay,
// //     );

// //     let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
// //         &cache_change1,
// //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
// //     ));

// //     assert!(messages_vec_expected_empty.is_empty());
// //     assert_eq!(messages_vec.len(), 1);
// //     assert!(messages_vec.contains(&expected_data_submessage));
// // }
// // }

// use crate::{
    //     behavior::data_from_cache_change,
    //     messages::{
    //         submessages::{submessage_elements::SequenceNumberSet, Gap},
    //         RtpsSubmessage,
    //     },
    //     structure::HistoryCache,
    //     types::{EntityId, SequenceNumber},
    // };

    // use super::{reader_proxy::ChangeForReader, ReaderProxy};

    // pub struct BestEffortReaderProxyBehavior;

    // impl BestEffortReaderProxyBehavior {
    //     pub fn produce_messages<T: ChangeForReader<CacheChangeRepresentation = SequenceNumber>>(
    //         reader_proxy: &mut impl ReaderProxy<ChangeForReaderType = T>,
    //         history_cache: &impl HistoryCache,
    //         writer_entity_id: EntityId,
    //     ) -> Vec<RtpsSubmessage> {
    //         let mut messages = Vec::new();
    //         if !reader_proxy.unsent_changes().is_empty() {
    //             Self::pushing_state(reader_proxy, history_cache, writer_entity_id, &mut messages);
    //         }
    //         messages
    //     }

    //     fn pushing_state<T: ChangeForReader<CacheChangeRepresentation = SequenceNumber>>(
    //         reader_proxy: &mut impl ReaderProxy<ChangeForReaderType = T>,
    //         history_cache: &impl HistoryCache,
    //         writer_entity_id: EntityId,
    //         message_queue: &mut Vec<RtpsSubmessage>,
    //     ) {
    //         while let Some(next_unsent_change) = reader_proxy.next_unsent_change() {
    //             let next_unsent_seq_num = *next_unsent_change.change();
    //             Self::transition_t4(
    //                 reader_proxy,
    //                 history_cache,
    //                 next_unsent_seq_num,
    //                 writer_entity_id,
    //                 message_queue,
    //             );
    //         }
    //     }

    //     fn transition_t4(
    //         reader_proxy: &mut impl ReaderProxy,
    //         history_cache: &impl HistoryCache,
    //         next_unsent_seq_num: SequenceNumber,
    //         writer_entity_id: EntityId,
    //         message_queue: &mut Vec<RtpsSubmessage>,
    //     ) {
    //         if let Some(cache_change) = history_cache.get_change(next_unsent_seq_num) {
    //             let reader_id = reader_proxy.remote_reader_guid().entity_id();
    //             let data = data_from_cache_change(cache_change, reader_id);
    //             message_queue.push(RtpsSubmessage::Data(data));
    //         } else {
    //             let gap = Gap {
    //                 endianness_flag: false,
    //                 reader_id: reader_proxy.remote_reader_guid().entity_id(),
    //                 writer_id: writer_entity_id,
    //                 gap_start: next_unsent_seq_num,
    //                 gap_list: SequenceNumberSet {
    //                     bitmap_base: next_unsent_seq_num,
    //                     bitmap: [0; 8],
    //                 },
    //             };
    //             message_queue.push(RtpsSubmessage::Gap(gap));
    //         }
    //     }
    // }

    // // #[cfg(test)]
    // // mod tests {
    // //     use super::*;
    // //     use crate::types::{
    // //         constants::{
    // //             ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    // //             ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    // //         },
    // //         ChangeKind,
    // //     };
    // //     use crate::types::{Locator, GUID};

    // //     use crate::structure::CacheChange;

    // // #[test]
    // // fn produce_empty() {
    // //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
    // //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
    // //     let multicast_locator_list = vec![];
    // //     let expects_inline_qos = false;
    // //     let is_active = true;
    // //     let mut reader_proxy = ReaderProxy::new(
    // //         remote_reader_guid,
    // //         unicast_locator_list,
    // //         multicast_locator_list,
    // //         expects_inline_qos,
    // //         is_active,
    // //     );

    // //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    // //     let history_cache = HistoryCache::default();

    // //     // Run without any change being created or added in the cache
    // //     let last_change_sequence_number = 0;
    // //     let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
    // //         &mut reader_proxy,
    // //         &history_cache,
    // //         writer_entity_id,
    // //         last_change_sequence_number,
    // //     );

    // //     assert!(messages_vec.is_empty());
    // // }

    // // #[test]
    // // fn produce_data_message() {
    // //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
    // //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
    // //     let multicast_locator_list = vec![];
    // //     let expects_inline_qos = false;
    // //     let is_active = true;
    // //     let mut reader_proxy = ReaderProxy::new(
    // //         remote_reader_guid,
    // //         unicast_locator_list,
    // //         multicast_locator_list,
    // //         expects_inline_qos,
    // //         is_active,
    // //     );

    // //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    // //     let mut history_cache = HistoryCache::default();

    // //     // Add one change to the history cache
    // //     let writer_guid = GUID::new([5; 12], writer_entity_id);
    // //     let instance_handle = [1; 16];
    // //     let cache_change1 = CacheChange::new(
    // //         ChangeKind::Alive,
    // //         writer_guid.into(),
    // //         instance_handle,
    // //         1,
    // //         Some(vec![1, 2, 3]),
    // //         None,
    // //     );
    // //     history_cache.add_change(cache_change1.clone());

    // //     // Run with the last change sequence number equal to the added cache change
    // //     let last_change_sequence_number = 1;
    // //     let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
    // //         &mut reader_proxy,
    // //         &history_cache,
    // //         writer_entity_id,
    // //         last_change_sequence_number,
    // //     );

    // //     let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
    // //         &cache_change1,
    // //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    // //     ));
    // //     assert_eq!(messages_vec.len(), 1);
    // //     assert!(messages_vec.contains(&expected_data_submessage));
    // // }

    // // #[test]
    // // fn produce_gap_message() {
    // //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
    // //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
    // //     let multicast_locator_list = vec![];
    // //     let expects_inline_qos = false;
    // //     let is_active = true;
    // //     let mut reader_proxy = ReaderProxy::new(
    // //         remote_reader_guid,
    // //         unicast_locator_list,
    // //         multicast_locator_list,
    // //         expects_inline_qos,
    // //         is_active,
    // //     );

    // //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    // //     let history_cache = HistoryCache::default();

    // //     // Run with the a sequence number of 1 without adding any change to the history cache
    // //     let last_change_sequence_number = 1;
    // //     let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
    // //         &mut reader_proxy,
    // //         &history_cache,
    // //         writer_entity_id,
    // //         last_change_sequence_number,
    // //     );

    // //     let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
    // //         BEHAVIOR_ENDIANNESS,
    // //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    // //         writer_entity_id,
    // //         1,
    // //         &[]
    // //     ));
    // //     assert_eq!(messages_vec.len(), 1);
    // //     assert!(messages_vec.contains(&expected_gap_submessage));
    // // }

    // // #[test]
    // // fn produce_data_and_gap_messages() {
    // //     let remote_reader_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
    // //     let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
    // //     let multicast_locator_list = vec![];
    // //     let expects_inline_qos = false;
    // //     let is_active = true;
    // //     let mut reader_proxy = ReaderProxy::new(
    // //         remote_reader_guid,
    // //         unicast_locator_list,
    // //         multicast_locator_list,
    // //         expects_inline_qos,
    // //         is_active,
    // //     );

    // //     let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    // //     let mut history_cache = HistoryCache::default();

    // //     // Add one change to the history cache
    // //     let writer_guid = GUID::new([5; 12], writer_entity_id);
    // //     let instance_handle = [1; 16];
    // //     let cache_change1 = CacheChange::new(
    // //         ChangeKind::Alive,
    // //         writer_guid.into(),
    // //         instance_handle,
    // //         1,
    // //         Some(vec![1, 2, 3]),
    // //         None,
    // //     );
    // //     history_cache.add_change(cache_change1.clone());

    // //     // Run with the last change sequence number one above the added cache change
    // //     let last_change_sequence_number = 2;
    // //     let messages_vec = BestEffortReaderProxyBehavior::produce_messages(
    // //         &mut reader_proxy,
    // //         &history_cache,
    // //         writer_entity_id,
    // //         last_change_sequence_number,
    // //     );

    // //     let expected_data_submessage = RtpsSubmessage::Data(data_from_cache_change(
    // //         &cache_change1,
    // //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    // //     ));
    // //     let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
    // //         BEHAVIOR_ENDIANNESS,
    // //         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    // //         writer_entity_id,
    // //         2,
    // //         &[],
    // //     ));
    // //     assert_eq!(messages_vec.len(), 2);
    // //     assert!(messages_vec.contains(&expected_data_submessage));
    // //     assert!(messages_vec.contains(&expected_gap_submessage));
    // // }
    // // }
