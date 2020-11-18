use std::collections::HashMap;

use crate::types::{ReliabilityKind, GUID, GuidPrefix};
use crate::messages::RtpsSubmessage;
use crate::behavior::RtpsWriter;
use crate::behavior::endpoint_traits::DestinedMessages;

use super::reader_proxy::ReaderProxy;
use super::reliable_reader_proxy::ReliableReaderProxy;
use super::best_effort_reader_proxy::BestEffortReaderProxy;

use rust_dds_interface::history_cache::HistoryCache;

enum ReaderProxyFlavor {
    BestEffort(BestEffortReaderProxy),
    Reliable(ReliableReaderProxy),
}

pub struct StatefulWriter {
    pub writer: RtpsWriter,
    matched_readers: HashMap<GUID, ReaderProxyFlavor>,
}

impl StatefulWriter {
    pub fn new(
        writer: RtpsWriter,
    ) -> Self {
            Self {
                writer,
                matched_readers: HashMap::new()
        }
    }

    pub fn produce_messages(&mut self, writer_cache: &mut HistoryCache) -> Vec<DestinedMessages> {
        let mut output = Vec::new();
        for (_reader_guid, reader_proxy) in self.matched_readers.iter_mut() {
            match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => {
                    let messages = reliable_reader_proxy.produce_messages(writer_cache, self.writer.last_change_sequence_number, self.writer.endpoint.entity.guid.entity_id(), self.writer.heartbeat_period, self.writer.nack_response_delay);
                    if !messages.is_empty() {
                        output.push(DestinedMessages::MultiDestination{
                            unicast_locator_list: reliable_reader_proxy.unicast_locator_list().clone(),
                            multicast_locator_list: reliable_reader_proxy.multicast_locator_list().clone(),
                            messages,
                        });   
                    }
                }
                ReaderProxyFlavor::BestEffort(best_effort_reader_proxy) => {
                    let messages = best_effort_reader_proxy.produce_messages(writer_cache, self.writer.last_change_sequence_number, self.writer.endpoint.entity.guid.entity_id());
                    if !messages.is_empty() {
                        output.push(DestinedMessages::MultiDestination{
                            unicast_locator_list: best_effort_reader_proxy.unicast_locator_list().clone(),
                            multicast_locator_list: best_effort_reader_proxy.multicast_locator_list().clone(),
                            messages,
                        });   
                    }
                }
            };
        }
        output
    }

    pub fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        for (_reader_guid, reader_proxy) in self.matched_readers.iter_mut(){
            match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.try_process_message(src_guid_prefix, submessage),
                ReaderProxyFlavor::BestEffort(_) => ()
            }
        }
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        let remote_reader_guid = a_reader_proxy.remote_reader_guid().clone();
        let reader_proxy = match self.writer.endpoint.reliability_level {
            ReliabilityKind::Reliable => ReaderProxyFlavor::Reliable(ReliableReaderProxy::new(a_reader_proxy)),
            ReliabilityKind::BestEffort => ReaderProxyFlavor::BestEffort(BestEffortReaderProxy::new(a_reader_proxy)),
        };
        self.matched_readers.insert(remote_reader_guid, reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers.remove(reader_proxy_guid);
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

// impl RtpsMessageSender for StatefulWriter {
//     fn output_queues(&mut self) -> Vec<OutputQueue> {
//         let mut output_queues = Vec::new();        

//         for (_, reader_proxy) in self.matched_readers.iter_mut() {
//             let (unicast_locator_list, multicast_locator_list, message_queue) = match reader_proxy {
//                 ReaderProxyFlavor::BestEffort(best_effort_proxy) => (
//                     best_effort_proxy.reader_proxy().unicast_locator_list().clone(),
//                     best_effort_proxy.reader_proxy().multicast_locator_list().clone(),
//                     best_effort_proxy.output_queue_mut().drain(..).collect()
//                 ),
//                 ReaderProxyFlavor::Reliable(reliable_proxy) => (
//                     reliable_proxy.reader_proxy().unicast_locator_list().clone(),
//                     reliable_proxy.reader_proxy().multicast_locator_list().clone(),
//                     reliable_proxy.output_queue_mut().drain(..).collect()
//                 )
//             };           

//             output_queues.push(
//                 OutputQueue::MultiDestination{
//                     unicast_locator_list,
//                     multicast_locator_list,
//                     message_queue,
//                 })
//         }
//         output_queues
//     }
// }

// impl RtpsCommunication for StatefulWriter {
//     fn try_push_message(&mut self, src_locator: Locator, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
//         for (_,reader_proxy) in &mut self.matched_readers {
//             match reader_proxy {
//                 ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.try_push_message(src_locator, src_guid_prefix, submessage),
//                 ReaderProxyFlavor::BestEffort(_) => (),
//             }
//         }
//     }
// }

// impl Receiver for StatefulWriter {
//     fn push_receive_message(&mut self, _src_locator: Locator, source_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) {
//         let (_, destination_reader) = self.matched_readers.iter_mut()
//             .find(|(_, reader)| reader.is_submessage_destination(&source_guid_prefix, &submessage) ).unwrap();

//         destination_reader.push_receive_message(source_guid_prefix, submessage);
//     }
    
//     fn is_submessage_destination(&self, _src_locator: &Locator, src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
//         self.matched_readers.iter().find(|&(_, reader)| reader.is_submessage_destination(src_guid_prefix, submessage)).is_some()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::types::constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;

//     #[test]
//     fn stateful_writer_new_change() {
//         let writer_cache = HistoryCache::default();
//         let writer = StatefulWriter::new(
//             GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
//             TopicKind::WithKey,
//             ReliabilityKind::BestEffort,
//             writer_cache,
//             true,
//             Duration::from_millis(500),
//             Duration::from_millis(200),
//             Duration::from_millis(0),
//         );

//         let cache_change_seq1 = writer.new_change(
//             ChangeKind::Alive,
//             Some(vec![1, 2, 3]),
//             None,
//             [1; 16],
//         );

//         let cache_change_seq2 = writer.new_change(
//             ChangeKind::NotAliveUnregistered,
//             None, 
//             None,
//             [1; 16],
//         );

//         assert_eq!(cache_change_seq1.sequence_number(), 1);
//         assert_eq!(cache_change_seq1.change_kind(), ChangeKind::Alive);
//         assert_eq!(cache_change_seq1.inline_qos().unwrap().parameter.len(), 0);
//         assert_eq!(cache_change_seq1.instance_handle(), [1; 16]);

//         assert_eq!(cache_change_seq2.sequence_number(), 2);
//         assert_eq!(
//             cache_change_seq2.change_kind(),
//             ChangeKind::NotAliveUnregistered
//         );
//         assert_eq!(cache_change_seq2.inline_qos().unwrap().parameter.len(), 0);
//         assert_eq!(cache_change_seq2.instance_handle(), [1; 16]);
//     }

//     #[test]
//     fn run_best_effort_send_data() {
        // let mut writer_qos = DataWriterQos::default();
        // writer_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;

        // let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        // let stateful_writer = StatefulWriter::new(
        //     writer_guid,
        //     TopicKind::WithKey,
        //     &writer_qos,
        // );

        // let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        // let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        // stateful_writer.matched_reader_add(reader_proxy);

        // let instance_handle = [1;16];
        // let cache_change_seq1 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![1,2,3]), None, instance_handle);
        // let cache_change_seq2 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![2,3,4]), None, instance_handle);
        // stateful_writer.writer_cache().add_change(cache_change_seq1).unwrap();
        // stateful_writer.writer_cache().add_change(cache_change_seq2).unwrap();

        // stateful_writer.run();
        // let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();

        // if let RtpsSubmessage::Data(data) = &messages[0] {
        //     assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
        //     assert_eq!(data.writer_id(), writer_guid.entity_id());
        //     assert_eq!(data.writer_sn(), 1);
        // } else {
        //     panic!("Wrong message sent");
        // }

        // if let RtpsSubmessage::Data(data) = &messages[1] {
        //     assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
        //     assert_eq!(data.writer_id(), writer_guid.entity_id());
        //     assert_eq!(data.writer_sn(), 2);
        // } else {
        //     panic!("Wrong message sent");
        // }

        // // Check that no heartbeat is sent using best effort writers
        // stateful_writer.run();
        // assert!(stateful_writer.pop_send_message().is_none());
        // sleep(stateful_writer.heartbeat_period.into());
        // stateful_writer.run();
        // assert!(stateful_writer.pop_send_message().is_none());
    // }

    // #[test]
    // fn run_reliable_send_data() {
        // let mut writer_qos = DataWriterQos::default();
        // writer_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        // let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        // let stateful_writer = StatefulWriter::new(
        //     writer_guid,
        //     TopicKind::WithKey,
        //     &writer_qos
        // );

        // let remote_reader_guid_prefix = [1,2,3,4,5,6,7,8,9,10,11,12];
        // let remote_reader_guid = GUID::new(remote_reader_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        // let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        // stateful_writer.matched_reader_add(reader_proxy);

        // let instance_handle = [1;16];
        // let cache_change_seq1 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![1,2,3]), None, instance_handle);
        // let cache_change_seq2 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![2,3,4]), None, instance_handle);
        // stateful_writer.writer_cache().add_change(cache_change_seq1).unwrap();
        // stateful_writer.writer_cache().add_change(cache_change_seq2).unwrap();

        // stateful_writer.run();
        // let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();

        // if let RtpsSubmessage::Data(data) = &messages[0] {
        //     assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
        //     assert_eq!(data.writer_id(), writer_guid.entity_id());
        //     assert_eq!(data.writer_sn(), 1);
        // } else {
        //     panic!("Wrong message sent");
        // }

        // if let RtpsSubmessage::Data(data) = &messages[1] {
        //     assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
        //     assert_eq!(data.writer_id(), writer_guid.entity_id());
        //     assert_eq!(data.writer_sn(), 2);
        // } else {
        //     panic!("Wrong message sent. Expected Data");
        // }

        // // Check that heartbeat is sent while there are unacked changes
        // stateful_writer.run();
        // assert!(stateful_writer.pop_send_message().is_none());
        // sleep(stateful_writer.heartbeat_period.into());
        // stateful_writer.run();
        // let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();
        // if let RtpsSubmessage::Heartbeat(heartbeat) = &messages[0] {
        //     assert_eq!(heartbeat.is_final(), false);
        // } else {
        //     panic!("Wrong message sent. Expected Heartbeat");
        // }

        // let acknack = AckNack::new(
        //     Endianness::LittleEndian,
        //     remote_reader_guid.entity_id(),
        //     writer_guid.entity_id(),
        //         3,
        //         BTreeSet::new(),
        //         1,
        //         false,
        // );

        // stateful_writer.push_receive_message( LOCATOR_INVALID, remote_reader_guid_prefix, RtpsSubmessage::AckNack(acknack));

        // // Check that no heartbeat is sent if there are no new changes
        // stateful_writer.run();
        // assert!(stateful_writer.pop_send_message().is_none());
        // sleep(stateful_writer.heartbeat_period.into());
        // stateful_writer.run();
        // assert!(stateful_writer.pop_send_message().is_none());
//     }
// }