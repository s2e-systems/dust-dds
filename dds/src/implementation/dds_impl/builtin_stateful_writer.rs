use crate::{
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            endpoint::RtpsEndpoint,
            messages::{overall_structure::RtpsMessageHeader, submessages::AckNackSubmessage},
            stateful_writer::{
                RtpsStatefulWriter, DEFAULT_HEARTBEAT_PERIOD, DEFAULT_NACK_RESPONSE_DELAY,
                DEFAULT_NACK_SUPPRESSION_DURATION,
            },
            transport::TransportWrite,
            types::{Guid, TopicKind},
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
        qos_policy::ReliabilityQosPolicyKind,
        time::Time,
    },
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use super::{
    message_receiver::MessageReceiver, participant_discovery::ParticipantDiscovery,
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
            DataWriterQos::default(),
        ));

        DdsShared::new(BuiltinStatefulWriter {
            rtps_writer: DdsRwLock::new(rtps_writer),
            topic,
            enabled: DdsRwLock::new(false),
            sedp_condvar,
        })
    }

    /// NOTE: This function is only useful for the SEDP writers so we probably need a separate
    /// type for those.
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
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn write_w_timestamp<Foo>(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .write_w_timestamp(data, handle, timestamp)?;

        self.sedp_condvar.notify_all();
        Ok(())
    }

    pub fn dispose_w_timestamp<Foo>(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        self.rtps_writer
            .write_lock()
            .dispose_w_timestamp(data, handle, timestamp)?;
        self.sedp_condvar.notify_all();
        Ok(())
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        Ok(())
    }
}

impl DdsShared<BuiltinStatefulWriter> {
    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        self.rtps_writer
            .write_lock()
            .send_message(header, transport);
    }
}
