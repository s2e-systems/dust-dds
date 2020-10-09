use std::collections::{HashMap, VecDeque};
use std::sync::{RwLock, RwLockReadGuard, Mutex};

use crate::types::{ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID, GuidPrefix};
use crate::behavior::types::Duration;
use crate::messages::RtpsSubmessage;
use crate::messages::message_receiver::Receiver;
use crate::messages::message_sender::Sender;
use crate::structure::HistoryCache;
use crate::structure::CacheChange;
use crate::serialized_payload::ParameterList;
use super::reader_proxy::ReaderProxy;
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter, ProtocolEndpoint};
use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::types::{Data, Time, ReturnCode};


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
    last_change_sequence_number: Mutex<SequenceNumber>,
    writer_cache: HistoryCache,
    data_max_sized_serialized: Option<i32>,

    matched_readers: RwLock<HashMap<GUID, ReaderProxy>>,
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

            StatefulWriter {
                guid,
                topic_kind,
                reliability_level: writer_qos.reliability.kind.into(),
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                last_change_sequence_number: Mutex::new(0),
                writer_cache: HistoryCache::new(&writer_qos.resource_limits),
                data_max_sized_serialized: None,
                matched_readers: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_change(
        &self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        *self.last_change_sequence_number.lock().unwrap() += 1;
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
        *self.last_change_sequence_number.lock().unwrap()
    }

    pub fn guid(&self) -> &GUID {
        &self.guid
    }

    pub fn writer_cache(&self) -> &HistoryCache {
        &self.writer_cache
    }

    pub fn matched_reader_add(&self, a_reader_proxy: ReaderProxy) {
        self.matched_readers.write().unwrap().insert(a_reader_proxy.remote_reader_guid, a_reader_proxy);
    }

    pub fn matched_reader_remove(&self, reader_proxy_guid: &GUID) {
        self.matched_readers.write().unwrap().remove(reader_proxy_guid);
    }
    
    pub fn matched_readers(&self) -> RwLockReadGuard<HashMap<GUID, ReaderProxy>> {
        self.matched_readers.read().unwrap()
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

    pub fn run(&self) {
        todo!()
        // for (_, reader_proxy) in self.matched_readers().iter() {
        //     match self.reliability_level {
        //         ReliabilityKind::BestEffort => BestEffortStatefulWriterBehavior::run(reader_proxy, &self),
        //         ReliabilityKind::Reliable => ReliableStatefulWriterBehavior::run(reader_proxy, &self),
        //     };
        // }
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

impl ProtocolEndpoint for StatefulWriter {}
impl ProtocolWriter for StatefulWriter {
    // fn new(
    //     _parent_instance_handle: InstanceHandle,
    //     _entity_type: EntityType,
    //     _topic_kind: TopicKind,
    //     _writer_qos: DataWriterQos,
    // ) -> Self {
    //     todo!()
    // }

    fn write(&self, instance_handle: InstanceHandle, data: Data, _timestamp: Time) -> ReturnCode<()>{
        let cc = self.new_change(ChangeKind::Alive, Some(data), None, instance_handle);
        self.writer_cache().add_change(cc)?;
        Ok(())
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

impl Receiver for StatefulWriter {
    fn push_receive_message(&self, src_locator: Locator, source_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) {
        let reader_id = match &submessage {
            RtpsSubmessage::AckNack(acknack) => acknack.reader_id(),
            _ => panic!("Unsupported message received by stateful writer"),
        };
        let guid = GUID::new(source_guid_prefix, reader_id);
        self.matched_readers().get(&guid).unwrap().received_messages.lock().unwrap().push_back((source_guid_prefix, submessage));
    }
    
    fn is_submessage_destination(&self, _src_locator: &Locator, src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        let reader_id = match &submessage {
            RtpsSubmessage::AckNack(acknack) => acknack.reader_id(),
            _ => return false,
        };

        let reader_guid = GUID::new(*src_guid_prefix, reader_id);

        self.matched_readers().get(&reader_guid).is_some()
    }
}

impl Sender for StatefulWriter {
    fn pop_send_message(&self) -> Option<(Vec<Locator>, VecDeque<RtpsSubmessage>)> {
        for (_, reader_proxy) in self.matched_readers().iter() {
            let mut reader_proxy_send_messages = reader_proxy.send_messages.lock().unwrap();
            if !reader_proxy_send_messages.is_empty() {
                let mut send_message_queue = VecDeque::new();
                std::mem::swap(&mut send_message_queue, &mut reader_proxy_send_messages);
                
                let mut locator_list = Vec::new();
                locator_list.extend(reader_proxy.unicast_locator_list());
                locator_list.extend(reader_proxy.multicast_locator_list());

                return Some((locator_list, send_message_queue));
            }
        }
        None
    }
}

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
        let writer = StatefulWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            &writer_qos
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
        assert_eq!(cache_change_seq1.change_kind(), &ChangeKind::Alive);
        assert_eq!(cache_change_seq1.inline_qos().len(), 0);
        assert_eq!(cache_change_seq1.instance_handle(), &[1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), 2);
        assert_eq!(
            cache_change_seq2.change_kind(),
            &ChangeKind::NotAliveUnregistered
        );
        assert_eq!(cache_change_seq2.inline_qos().len(), 0);
        assert_eq!(cache_change_seq2.instance_handle(), &[1; 16]);
    }

    #[test]
    fn run_best_effort_send_data() {
        let mut writer_qos = DataWriterQos::default();
        writer_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;

        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let stateful_writer = StatefulWriter::new(
            writer_guid,
            TopicKind::WithKey,
            &writer_qos,
        );

        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        stateful_writer.matched_reader_add(reader_proxy);

        let instance_handle = [1;16];
        let cache_change_seq1 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![1,2,3]), None, instance_handle);
        let cache_change_seq2 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![2,3,4]), None, instance_handle);
        stateful_writer.writer_cache().add_change(cache_change_seq1).unwrap();
        stateful_writer.writer_cache().add_change(cache_change_seq2).unwrap();

        stateful_writer.run();
        let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();

        if let RtpsSubmessage::Data(data) = &messages[0] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 1);
        } else {
            panic!("Wrong message sent");
        }

        if let RtpsSubmessage::Data(data) = &messages[1] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 2);
        } else {
            panic!("Wrong message sent");
        }

        // Check that no heartbeat is sent using best effort writers
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
        sleep(stateful_writer.heartbeat_period.into());
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
    }

    #[test]
    fn run_reliable_send_data() {
        let mut writer_qos = DataWriterQos::default();
        writer_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

        let writer_guid = GUID::new([2;12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let stateful_writer = StatefulWriter::new(
            writer_guid,
            TopicKind::WithKey,
            &writer_qos
        );

        let remote_reader_guid_prefix = [1,2,3,4,5,6,7,8,9,10,11,12];
        let remote_reader_guid = GUID::new(remote_reader_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        stateful_writer.matched_reader_add(reader_proxy);

        let instance_handle = [1;16];
        let cache_change_seq1 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![1,2,3]), None, instance_handle);
        let cache_change_seq2 = stateful_writer.new_change(ChangeKind::Alive, Some(vec![2,3,4]), None, instance_handle);
        stateful_writer.writer_cache().add_change(cache_change_seq1).unwrap();
        stateful_writer.writer_cache().add_change(cache_change_seq2).unwrap();

        stateful_writer.run();
        let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();

        if let RtpsSubmessage::Data(data) = &messages[0] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 1);
        } else {
            panic!("Wrong message sent");
        }

        if let RtpsSubmessage::Data(data) = &messages[1] {
            assert_eq!(data.reader_id(), remote_reader_guid.entity_id());
            assert_eq!(data.writer_id(), writer_guid.entity_id());
            assert_eq!(data.writer_sn(), 2);
        } else {
            panic!("Wrong message sent. Expected Data");
        }

        // Check that heartbeat is sent while there are unacked changes
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
        sleep(stateful_writer.heartbeat_period.into());
        stateful_writer.run();
        let (_dst_locators, messages) = stateful_writer.pop_send_message().unwrap();
        if let RtpsSubmessage::Heartbeat(heartbeat) = &messages[0] {
            assert_eq!(heartbeat.is_final(), false);
        } else {
            panic!("Wrong message sent. Expected Heartbeat");
        }

        let acknack = AckNack::new(
            Endianness::LittleEndian,
            remote_reader_guid.entity_id(),
            writer_guid.entity_id(),
                3,
                BTreeSet::new(),
                1,
                false,
        );

        stateful_writer.push_receive_message( LOCATOR_INVALID, remote_reader_guid_prefix, RtpsSubmessage::AckNack(acknack));

        // Check that no heartbeat is sent if there are no new changes
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
        sleep(stateful_writer.heartbeat_period.into());
        stateful_writer.run();
        assert!(stateful_writer.pop_send_message().is_none());
    }
}