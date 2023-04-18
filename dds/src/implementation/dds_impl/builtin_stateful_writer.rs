use crate::{
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            endpoint::RtpsEndpoint,
            history_cache::{RtpsParameter, RtpsWriterCacheChange},
            messages::submessages::AckNackSubmessage,
            reader_proxy::RtpsReaderProxy,
            stateful_writer::{
                RtpsStatefulWriter, DEFAULT_HEARTBEAT_PERIOD, DEFAULT_NACK_RESPONSE_DELAY,
                DEFAULT_NACK_SUPPRESSION_DURATION,
            },
            types::{ChangeKind, Guid, GuidPrefix, Locator, TopicKind},
            writer::RtpsWriter,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        time::{Duration, DurationKind, Time, DURATION_ZERO},
    },
    topic_definition::type_support::{DdsSerializedKey, DdsType},
};

use super::{
    domain_participant_impl::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
    },
    iterators::{ReaderProxyListIntoIter, WriterChangeListIntoIter},
    message_receiver::MessageReceiver,
    participant_discovery::ParticipantDiscovery,
    topic_impl::TopicImpl,
};

pub struct BuiltinStatefulWriter {
    rtps_writer: DdsRwLock<RtpsStatefulWriter>,
    topic: DdsShared<TopicImpl>,
    enabled: DdsRwLock<bool>,
    sedp_condvar: DdsCondvar,
}

impl BuiltinStatefulWriter {
    pub fn new(
        guid: Guid,
        topic: DdsShared<TopicImpl>,
        sedp_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let topic_kind = TopicKind::WithKey;
        let push_mode = true;
        let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
        let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
        let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
        let data_max_size_serialized = usize::MAX;
        let qos = DataWriterQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Finite(DURATION_ZERO),
            },
            ..Default::default()
        };
        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                unicast_locator_list,
                multicast_locator_list,
            ),
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
            qos,
        ));

        DdsShared::new(BuiltinStatefulWriter {
            rtps_writer: DdsRwLock::new(rtps_writer),
            topic,
            enabled: DdsRwLock::new(false),
            sedp_condvar,
        })
    }

    pub fn guid(&self) -> Guid {
        self.rtps_writer.read_lock().guid()
    }

    pub fn _unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.read_lock().unicast_locator_list().to_vec()
    }

    pub fn _multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer
            .read_lock()
            .multicast_locator_list()
            .to_vec()
    }

    pub fn _push_mode(&self) -> bool {
        self.rtps_writer.read_lock().push_mode()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.rtps_writer.read_lock().heartbeat_period()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.rtps_writer.read_lock().data_max_size_serialized()
    }

    pub fn _new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.rtps_writer
            .write_lock()
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn change_list(&self) -> WriterChangeListIntoIter {
        WriterChangeListIntoIter::new(self.rtps_writer.read_lock())
    }

    pub fn _add_change(&self, change: RtpsWriterCacheChange) {
        self.rtps_writer.write_lock().add_change(change)
    }

    pub fn _remove_change<F>(&self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.rtps_writer.write_lock().remove_change(f)
    }

    pub fn _matched_reader_add(&self, a_reader_proxy: RtpsReaderProxy) {
        self.rtps_writer
            .write_lock()
            .matched_reader_add(a_reader_proxy)
    }

    pub fn _matched_reader_remove(&self, a_reader_guid: Guid) {
        self.rtps_writer
            .write_lock()
            .matched_reader_remove(a_reader_guid)
    }

    pub fn matched_reader_list(&self) -> ReaderProxyListIntoIter {
        ReaderProxyListIntoIter::new(self.rtps_writer.write_lock())
    }

    pub fn _is_acked_by_all(&self, a_change: &RtpsWriterCacheChange) -> bool {
        self.rtps_writer.read_lock().is_acked_by_all(a_change)
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        if rtps_writer_lock.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            rtps_writer_lock.on_acknack_submessage_received(
                acknack_submessage,
                message_receiver.source_guid_prefix(),
            );
        }
    }

    pub fn add_matched_participant(&self, participant_discovery: &ParticipantDiscovery) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let type_name = self.topic.get_type_name();
        if type_name == DiscoveredWriterData::type_name() {
            participant_discovery
                .discovered_participant_add_publications_writer(&mut rtps_writer_lock);
        } else if type_name == DiscoveredReaderData::type_name() {
            participant_discovery
                .discovered_participant_add_subscriptions_writer(&mut rtps_writer_lock);
        } else if type_name == DiscoveredTopicData::type_name() {
            participant_discovery.discovered_participant_add_topics_writer(&mut rtps_writer_lock);
        }
        self.sedp_condvar.notify_all();
    }

    pub fn _remove_matched_participant(&self, participant_guid_prefix: GuidPrefix) {
        let type_name = self.topic.get_type_name();
        if type_name == DiscoveredWriterData::type_name() {
            self.rtps_writer
                .write_lock()
                .matched_reader_remove(Guid::new(
                    participant_guid_prefix,
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ));
        } else if type_name == DiscoveredReaderData::type_name() {
            self.rtps_writer
                .write_lock()
                .matched_reader_remove(Guid::new(
                    participant_guid_prefix,
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ));
        } else if type_name == DiscoveredTopicData::type_name() {
            self.rtps_writer
                .write_lock()
                .matched_reader_remove(Guid::new(
                    participant_guid_prefix,
                    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
                ));
        }
    }

    pub fn write_w_timestamp(
        &self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer.write_lock().write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )?;

        self.sedp_condvar.notify_all();
        Ok(())
    }

    pub fn dispose_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.rtps_writer.write_lock().dispose_w_timestamp(
            instance_serialized_key,
            handle,
            timestamp,
        )?;
        self.sedp_condvar.notify_all();
        Ok(())
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}
