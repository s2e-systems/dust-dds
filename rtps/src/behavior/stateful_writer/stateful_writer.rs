use std::collections::HashMap;

use crate::types::{ReliabilityKind, GUID, GuidPrefix};
use crate::messages::RtpsSubmessage;
use crate::behavior::RtpsWriter;
use crate::behavior::types::Duration;
use crate::behavior::endpoint_traits::{DestinedMessages, AcknowldegmentReceiver, CacheChangeSender};

use super::reader_proxy::ReaderProxy;
use super::reliable_reader_proxy::ReliableReaderProxy;
use super::best_effort_reader_proxy::BestEffortReaderProxy;
use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

enum ReaderProxyFlavor {
    BestEffort(BestEffortReaderProxy),
    Reliable(ReliableReaderProxy),
}

impl std::ops::Deref for ReaderProxyFlavor {
    type Target = ReaderProxy;

    fn deref(&self) -> &Self::Target {
        match self {
            ReaderProxyFlavor::BestEffort(rp) => rp,
            ReaderProxyFlavor::Reliable(rp) => rp,
        }
    }
}

pub struct StatefulWriter {
    pub writer: RtpsWriter,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    matched_readers: HashMap<GUID, ReaderProxyFlavor>,
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        writer_cache: HistoryCache,
        data_max_sized_serialized: Option<i32>,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        
        let writer = RtpsWriter::new(guid, topic_kind, reliability_level, push_mode, writer_cache, data_max_sized_serialized);
            Self {
                writer,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                matched_readers: HashMap::new()
        }
    }
    
    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        let remote_reader_guid = a_reader_proxy.remote_reader_guid;
        let reader_proxy = match self.writer.endpoint.reliability_level {
            ReliabilityKind::Reliable => ReaderProxyFlavor::Reliable(ReliableReaderProxy::new(a_reader_proxy)),
            ReliabilityKind::BestEffort => ReaderProxyFlavor::BestEffort(BestEffortReaderProxy::new(a_reader_proxy)),
        };
        self.matched_readers.insert(remote_reader_guid, reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers.remove(reader_proxy_guid);
    }

    pub fn matched_reader_lookup(&self, a_reader_guid: GUID) -> Option<&ReaderProxy> {
        match self.matched_readers.get(&a_reader_guid) {
            Some(rp) => Some(rp),
            None => None,
        }
    }


    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl CacheChangeSender for StatefulWriter {
    fn produce_messages(&mut self) -> Vec<DestinedMessages> {
        let mut output = Vec::new();
        for (_reader_guid, reader_proxy) in self.matched_readers.iter_mut() {
            let messages = match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.produce_messages(&self.writer.writer_cache, self.writer.endpoint.entity.guid.entity_id(), self.writer.last_change_sequence_number, self.heartbeat_period, self.nack_response_delay),
                ReaderProxyFlavor::BestEffort(best_effort_reader_proxy) => best_effort_reader_proxy.produce_messages(&self.writer.writer_cache, self.writer.endpoint.entity.guid.entity_id(), self.writer.last_change_sequence_number),
            };

            if !messages.is_empty() {
                output.push(DestinedMessages::MultiDestination{
                    unicast_locator_list: reader_proxy.unicast_locator_list.clone(),
                    multicast_locator_list: reader_proxy.multicast_locator_list.clone(),
                    messages,
                });   
            }
        }
        output
    }
}

impl AcknowldegmentReceiver for StatefulWriter {
    fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        for (_reader_guid, reader_proxy) in self.matched_readers.iter_mut(){
            match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.try_process_message(src_guid_prefix, submessage),
                ReaderProxyFlavor::BestEffort(_) => ()
            }
        }
    }
}

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