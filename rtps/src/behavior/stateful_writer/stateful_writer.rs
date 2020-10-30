use std::collections::{HashMap, VecDeque};

use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, GuidPrefix};
use crate::behavior::types::Duration;
use crate::messages::RtpsSubmessage;
use crate::structure::{HistoryCache, CacheChange, RtpsEndpoint, RtpsEntity, RtpsCommunication, RtpsMessageSender, OutputQueue};
use crate::serialized_payload::ParameterList;
use super::reader_proxy::ReaderProxy;
use super::reliable_reader_proxy::ReliableReaderProxy;
use super::best_effort_reader_proxy::BestEffortReaderProxy;
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter};
use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::types::{Data, Time, ReturnCode};

enum ReaderProxyFlavor {
    BestEffort(BestEffortReaderProxy),
    Reliable(ReliableReaderProxy),
}

pub struct StatefulWriter {
    /// Entity base class (contains the GUID)
    guid: GUID,
    // entity: Entity,

    // Endpoint base class:
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,

    // All messages are received from the reader proxies so these fields are not used
    // /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    // unicast_locator_list: Vec<Locator>,
    // /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    // multicast_locator_list: Vec<Locator>,

    //Writer class:
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    matched_readers: HashMap<GUID, ReaderProxyFlavor>,
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        writer_qos: &DataWriterQos) -> Self {

            let push_mode = true;
            let heartbeat_period = Duration::from_millis(500);
            let nack_response_delay = Duration::from_millis(200);
            let nack_suppression_duration = Duration::from_millis(0);

            Self {
                guid,
                topic_kind,
                reliability_level: writer_qos.reliability.kind.into(),
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                last_change_sequence_number: 0,
                writer_cache: HistoryCache::default(),
                data_max_sized_serialized: None,
                matched_readers: HashMap::new()
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number += 1;
        CacheChange::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number(),
            data,
            inline_qos,
        )
    }

    pub fn last_change_sequence_number(&self) -> SequenceNumber {
        self.last_change_sequence_number
    }

    pub fn run(&mut self) {
        for (_reader_guid, reader_proxy) in self.matched_readers.iter_mut(){
            match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.process(&self.writer_cache, self.last_change_sequence_number),
                ReaderProxyFlavor::BestEffort(best_effort_reader_proxy) => best_effort_reader_proxy.process(&self.writer_cache, self.last_change_sequence_number),
            }
        }
    }

    pub fn guid(&self) -> &GUID {
        &self.guid
    }

    pub fn writer_cache(&self) -> &HistoryCache {
        &self.writer_cache
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        let remote_reader_guid = a_reader_proxy.remote_reader_guid().clone();
        let reader_proxy = match self.reliability_level {
            ReliabilityKind::Reliable => ReaderProxyFlavor::Reliable(ReliableReaderProxy::new(a_reader_proxy, self.guid.entity_id(), self.heartbeat_period, self.nack_response_delay)),
            ReliabilityKind::BestEffort => ReaderProxyFlavor::BestEffort(BestEffortReaderProxy::new(a_reader_proxy, self.guid.entity_id())),
        };
        self.matched_readers.insert(remote_reader_guid, reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers.remove(reader_proxy_guid);
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    pub fn nack_response_delay(&self) -> Duration {
        self.nack_response_delay
    }
}

impl RtpsMessageSender for StatefulWriter {
    fn output_queues(&mut self) -> Vec<OutputQueue> {
        let mut output_queues = Vec::new();        

        for (_, reader_proxy) in self.matched_readers.iter_mut() {
            let (unicast_locator_list, multicast_locator_list, message_queue) = match reader_proxy {
                ReaderProxyFlavor::BestEffort(best_effort_proxy) => (
                    best_effort_proxy.reader_proxy().unicast_locator_list().clone(),
                    best_effort_proxy.reader_proxy().multicast_locator_list().clone(),
                    best_effort_proxy.output_queue_mut().drain(..).collect()
                ),
                ReaderProxyFlavor::Reliable(reliable_proxy) => (
                    reliable_proxy.reader_proxy().unicast_locator_list().clone(),
                    reliable_proxy.reader_proxy().multicast_locator_list().clone(),
                    reliable_proxy.output_queue_mut().drain(..).collect()
                )
            };           

            output_queues.push(
                OutputQueue::MultiDestination{
                    unicast_locator_list,
                    multicast_locator_list,
                    message_queue,
                })
        }
        output_queues
    }
}
impl ProtocolEntity for StatefulWriter {
    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }

    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }
}

impl ProtocolWriter for StatefulWriter {

    fn write(&mut self, instance_handle: InstanceHandle, data: Data, _timestamp: Time) -> ReturnCode<()>{
        todo!()
        // if self.reliability_level == ReliabilityKind::BestEffort {
        //     let cc = self.new_change(ChangeKind::Alive, Some(data), None, instance_handle);
        //     self.writer_cache().add_change(cc)?;
        //     Ok(())
        // } else {
        //     todo!() // Blocking until wirtier_cache available
        // }
    }

    fn dispose(&self, _instance_handle: InstanceHandle, _timestamp: Time) -> ReturnCode<()> {
        todo!()
    }

    fn unregister(&self, _instance_handle: InstanceHandle, _timestamp: Time) -> ReturnCode<()> {
        todo!()
    }

    fn register(&self, _instance_handle: InstanceHandle, _timestamp: Time) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    fn lookup_instance(&self, _instance_handle: InstanceHandle) -> Option<InstanceHandle> {
        todo!()
    }
}

impl RtpsEntity for StatefulWriter {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl RtpsEndpoint for StatefulWriter {
    fn unicast_locator_list(&self) -> Vec<Locator> {
        todo!()
    }

    fn multicast_locator_list(&self) -> Vec<Locator> {
        todo!()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        todo!()
    }

    fn topic_kind(&self) -> &TopicKind {
        todo!()
    }
}

impl RtpsCommunication for StatefulWriter {
    fn try_push_message(&mut self, src_locator: Locator, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        for (_,reader_proxy) in &mut self.matched_readers {
            match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.try_push_message(src_locator, src_guid_prefix, submessage),
                ReaderProxyFlavor::BestEffort(_) => (),
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        LOCATOR_INVALID};

    use crate::messages::submessages::AckNack;
    use crate::messages::Endianness;
    
    use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;
    
    use std::collections::BTreeSet;
    use std::thread::sleep;

    #[test]
    fn stateful_writer_new_change() {
        let writer_qos = DataWriterQos::default();
        let mut writer = StatefulWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            &writer_qos,
        );

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]),
            None,
            [1; 16],
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None, 
            None,
            [1; 16],
        );

        assert_eq!(cache_change_seq1.sequence_number(), 1);
        assert_eq!(cache_change_seq1.change_kind(), ChangeKind::Alive);
        assert_eq!(cache_change_seq1.inline_qos().len(), 0);
        assert_eq!(cache_change_seq1.instance_handle(), [1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), 2);
        assert_eq!(
            cache_change_seq2.change_kind(),
            ChangeKind::NotAliveUnregistered
        );
        assert_eq!(cache_change_seq2.inline_qos().len(), 0);
        assert_eq!(cache_change_seq2.instance_handle(), [1; 16]);
    }

    #[test]
    fn run_best_effort_send_data() {
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
    }

    #[test]
    fn run_reliable_send_data() {
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
    }
}