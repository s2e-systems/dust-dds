use crate::{
    implementation::{
        data_representation_builtin_endpoints::discovered_reader_data::{
            DiscoveredReaderData, DCPS_SUBSCRIPTION,
        },
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::submessages::AckNackSubmessage,
            stateful_writer::{
                RtpsStatefulWriter, DEFAULT_HEARTBEAT_PERIOD, DEFAULT_NACK_RESPONSE_DELAY,
                DEFAULT_NACK_SUPPRESSION_DURATION,
            },
            types::{
                EntityId, EntityKey, Guid, GuidPrefix, Locator, TopicKind, BUILT_IN_WRITER_GROUP,
            },
            writer::RtpsWriter,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::DdsResult,
        qos::{DataWriterQos, PublisherQos},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::NO_STATUS,
        time::{DurationKind, DURATION_ZERO},
    },
    topic_definition::type_support::DdsType,
};

use super::{
    builtin_stateful_writer::BuiltinStatefulWriter,
    builtin_stateless_writer::BuiltinStatelessWriter,
    domain_participant_impl::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
    },
    message_receiver::{MessageReceiver, PublisherMessageReceiver},
    topic_impl::TopicImpl,
    user_defined_data_writer::UserDefinedDataWriter,
};

pub struct BuiltinPublisher {
    _qos: PublisherQos,
    _rtps_group: RtpsGroup,
    spdp_builtin_participant_writer: DdsShared<BuiltinStatelessWriter>,
    sedp_builtin_topics_writer: DdsShared<BuiltinStatefulWriter>,
    sedp_builtin_publications_writer: DdsShared<BuiltinStatefulWriter>,
    sedp_builtin_subscriptions_writer: DdsShared<UserDefinedDataWriter<RtpsStatefulWriter>>,
    enabled: DdsRwLock<bool>,
}

impl BuiltinPublisher {
    pub fn new(
        guid_prefix: GuidPrefix,
        sedp_topic_topics: DdsShared<TopicImpl>,
        sedp_topic_publications: DdsShared<TopicImpl>,
        spdp_discovery_locator_list: &[Locator],
        sedp_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        let entity_id = EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_WRITER_GROUP);
        let guid = Guid::new(guid_prefix, entity_id);
        let rtps_group = RtpsGroup::new(guid);

        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);
        let spdp_builtin_participant_writer = BuiltinStatelessWriter::new(
            spdp_builtin_participant_writer_guid,
            spdp_discovery_locator_list,
        );

        let sedp_builtin_topics_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_writer = BuiltinStatefulWriter::new(
            sedp_builtin_topics_writer_guid,
            sedp_topic_topics,
            sedp_condvar.clone(),
        );

        let sedp_builtin_publications_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer = BuiltinStatefulWriter::new(
            sedp_builtin_publications_writer_guid,
            sedp_topic_publications,
            sedp_condvar.clone(),
        );

        let sedp_builtin_subscriptions_writer = UserDefinedDataWriter::new(
            create_builtin_stateful_writer(Guid::new(
                guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            )),
            None,
            NO_STATUS,
            DiscoveredReaderData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
        );

        DdsShared::new(BuiltinPublisher {
            _qos: PublisherQos::default(),
            _rtps_group: rtps_group,
            spdp_builtin_participant_writer,
            sedp_builtin_topics_writer,
            sedp_builtin_publications_writer,
            sedp_builtin_subscriptions_writer,
            enabled: DdsRwLock::new(false),
        })
    }
}

impl DdsShared<BuiltinPublisher> {
    pub fn spdp_builtin_participant_writer(&self) -> &DdsShared<BuiltinStatelessWriter> {
        &self.spdp_builtin_participant_writer
    }

    pub fn sedp_builtin_topics_writer(&self) -> &DdsShared<BuiltinStatefulWriter> {
        &self.sedp_builtin_topics_writer
    }

    pub fn sedp_builtin_publications_writer(&self) -> &DdsShared<BuiltinStatefulWriter> {
        &self.sedp_builtin_publications_writer
    }

    pub fn sedp_builtin_subscriptions_writer(
        &self,
    ) -> &DdsShared<UserDefinedDataWriter<RtpsStatefulWriter>> {
        &self.sedp_builtin_subscriptions_writer
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        self.spdp_builtin_participant_writer.enable()?;
        self.sedp_builtin_publications_writer.enable()?;
        self.sedp_builtin_topics_writer.enable()?;
        self.sedp_builtin_subscriptions_writer.enable();

        Ok(())
    }
}

impl PublisherMessageReceiver for DdsShared<BuiltinPublisher> {
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        self.sedp_builtin_publications_writer
            .on_acknack_submessage_received(acknack_submessage, message_receiver);
        self.sedp_builtin_subscriptions_writer
            .on_acknack_submessage_received(acknack_submessage, message_receiver);
        self.sedp_builtin_topics_writer
            .on_acknack_submessage_received(acknack_submessage, message_receiver);
    }

    fn on_nack_frag_submessage_received(
        &self,
        _nackfrag_submessage: &crate::implementation::rtps::messages::submessages::NackFragSubmessage,
        _message_receiver: &MessageReceiver,
    ) {
        // Only for user defined
        todo!()
    }
}

fn create_builtin_stateful_writer(guid: Guid) -> RtpsStatefulWriter {
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
    RtpsStatefulWriter::new(RtpsWriter::new(
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
    ))
}
