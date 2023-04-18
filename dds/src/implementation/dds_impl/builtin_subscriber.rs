use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
            spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
        },
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{
                    DataFragSubmessage, DataSubmessage, GapSubmessage, HeartbeatFragSubmessage,
                    HeartbeatSubmessage,
                },
            },
            reader::RtpsReader,
            stateful_reader::{
                RtpsStatefulReader, DEFAULT_HEARTBEAT_RESPONSE_DELAY,
                DEFAULT_HEARTBEAT_SUPPRESSION_DURATION,
            },
            stateless_reader::{RtpsStatelessReader, StatelessReaderDataReceivedResult},
            transport::TransportWrite,
            types::{EntityId, EntityKey, Guid, GuidPrefix, TopicKind, BUILT_IN_READER_GROUP},
        },
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        condition::StatusCondition,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::{StatusKind, NO_STATUS},
        time::{DurationKind, DURATION_ZERO},
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    dds_data_reader::{DdsDataReader, UserDefinedReaderDataSubmessageReceivedResult},
    domain_participant_impl::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
    },
    message_receiver::{MessageReceiver, SubscriberSubmessageReceiver},
    status_listener::StatusListener,
};

pub struct BuiltInSubscriber {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    spdp_builtin_participant_reader: DdsShared<DdsDataReader<RtpsStatelessReader>>,
    sedp_builtin_topics_reader: DdsShared<DdsDataReader<RtpsStatefulReader>>,
    sedp_builtin_publications_reader: DdsShared<DdsDataReader<RtpsStatefulReader>>,
    sedp_builtin_subscriptions_reader: DdsShared<DdsDataReader<RtpsStatefulReader>>,
    enabled: DdsRwLock<bool>,
}

impl BuiltInSubscriber {
    pub fn new(guid_prefix: GuidPrefix) -> DdsShared<Self> {
        let qos = SubscriberQos::default();

        let entity_id = EntityId::new(EntityKey::new([0, 0, 0]), BUILT_IN_READER_GROUP);
        let guid = Guid::new(guid_prefix, entity_id);
        let rtps_group = RtpsGroup::new(guid);

        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);

        let spdp_builtin_participant_reader = DdsDataReader::new(
            create_builtin_stateless_reader::<SpdpDiscoveredParticipantData>(
                spdp_builtin_participant_reader_guid,
            ),
            SpdpDiscoveredParticipantData::type_name(),
            String::from(DCPS_PARTICIPANT),
            None,
            NO_STATUS,
        );

        let sedp_builtin_topics_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader = DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredTopicData>(sedp_builtin_topics_guid),
            DiscoveredTopicData::type_name(),
            String::from(DCPS_TOPIC),
            None,
            NO_STATUS,
        );

        let sedp_builtin_publications_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader = DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredWriterData>(sedp_builtin_publications_guid),
            DiscoveredWriterData::type_name(),
            String::from(DCPS_PUBLICATION),
            None,
            NO_STATUS,
        );

        let sedp_builtin_subscriptions_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader = DdsDataReader::new(
            create_builtin_stateful_reader::<DiscoveredWriterData>(
                sedp_builtin_subscriptions_reader_guid,
            ),
            DiscoveredReaderData::type_name(),
            String::from(DCPS_SUBSCRIPTION),
            None,
            NO_STATUS,
        );

        DdsShared::new(BuiltInSubscriber {
            qos,
            rtps_group,
            spdp_builtin_participant_reader,
            enabled: DdsRwLock::new(false),
            sedp_builtin_topics_reader,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_reader,
        })
    }
}

impl DdsShared<BuiltInSubscriber> {
    pub fn spdp_builtin_participant_reader(
        &self,
    ) -> &DdsShared<DdsDataReader<RtpsStatelessReader>> {
        &self.spdp_builtin_participant_reader
    }

    pub fn sedp_builtin_topics_reader(&self) -> &DdsShared<DdsDataReader<RtpsStatefulReader>> {
        &self.sedp_builtin_topics_reader
    }

    pub fn sedp_builtin_publications_reader(
        &self,
    ) -> &DdsShared<DdsDataReader<RtpsStatefulReader>> {
        &self.sedp_builtin_publications_reader
    }

    pub fn sedp_builtin_subscriptions_reader(
        &self,
    ) -> &DdsShared<DdsDataReader<RtpsStatefulReader>> {
        &self.sedp_builtin_subscriptions_reader
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;

        self.spdp_builtin_participant_reader.enable()?;
        self.sedp_builtin_topics_reader.enable()?;
        self.sedp_builtin_subscriptions_reader.enable()?;
        self.sedp_builtin_publications_reader.enable()?;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        self.sedp_builtin_topics_reader
            .send_message(header, transport);
        self.sedp_builtin_publications_reader
            .send_message(header, transport);
        self.sedp_builtin_subscriptions_reader
            .send_message(header, transport);
    }
}

impl SubscriberSubmessageReceiver for DdsShared<BuiltInSubscriber> {
    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.sedp_builtin_topics_reader
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        self.sedp_builtin_publications_reader
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        self.sedp_builtin_subscriptions_reader
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
    }

    fn on_heartbeat_frag_submessage_received(
        &self,
        _heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        _source_guid_prefix: GuidPrefix,
    ) {
        // Maybe necessary for user data
        todo!()
    }

    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        match self
            .sedp_builtin_topics_reader()
            .on_data_submessage_received(
                data_submessage,
                message_receiver,
                &mut StatusListener::new(None, NO_STATUS),
                participant_status_listener,
            ) {
            UserDefinedReaderDataSubmessageReceivedResult::NoChange => (),
            UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable => {
                self.sedp_builtin_topics_reader()
                    .get_statuscondition()
                    .write_lock()
                    .add_communication_state(StatusKind::DataAvailable);
                return;
            }
        }

        match self
            .sedp_builtin_publications_reader()
            .on_data_submessage_received(
                data_submessage,
                message_receiver,
                &mut StatusListener::new(None, NO_STATUS),
                participant_status_listener,
            ) {
            UserDefinedReaderDataSubmessageReceivedResult::NoChange => (),
            UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable => {
                self.sedp_builtin_publications_reader()
                    .get_statuscondition()
                    .write_lock()
                    .add_communication_state(StatusKind::DataAvailable);
                return;
            }
        }

        match self
            .sedp_builtin_subscriptions_reader()
            .on_data_submessage_received(
                data_submessage,
                message_receiver,
                &mut StatusListener::new(None, NO_STATUS),
                participant_status_listener,
            ) {
            UserDefinedReaderDataSubmessageReceivedResult::NoChange => (),
            UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable => {
                self.sedp_builtin_subscriptions_reader()
                    .get_statuscondition()
                    .write_lock()
                    .add_communication_state(StatusKind::DataAvailable);
                return;
            }
        }

        match self
            .spdp_builtin_participant_reader()
            .on_data_submessage_received(data_submessage, message_receiver)
        {
            StatelessReaderDataReceivedResult::NotForThisReader
            | StatelessReaderDataReceivedResult::SampleRejected(_, _)
            | StatelessReaderDataReceivedResult::InvalidData(_) => (),
            StatelessReaderDataReceivedResult::NewSampleAdded(_) => {
                self.spdp_builtin_participant_reader
                    .get_statuscondition()
                    .write_lock()
                    .add_communication_state(StatusKind::DataAvailable);
            }
        };
    }

    fn on_data_frag_submessage_received(
        &self,
        _data_frag_submessage: &DataFragSubmessage<'_>,
        _message_receiver: &MessageReceiver,
        _participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        // Not for builtin types
    }

    fn on_gap_submessage_received(
        &self,
        gap_submessage: &GapSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        self.sedp_builtin_topics_reader
            .on_gap_submessage_received(gap_submessage, message_receiver.source_guid_prefix());
        self.sedp_builtin_publications_reader
            .on_gap_submessage_received(gap_submessage, message_receiver.source_guid_prefix());
        self.sedp_builtin_subscriptions_reader
            .on_gap_submessage_received(gap_submessage, message_receiver.source_guid_prefix());
    }
}

fn create_builtin_stateless_reader<Foo>(guid: Guid) -> RtpsStatelessReader
where
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
{
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];
    let qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(DURATION_ZERO),
        },
        ..Default::default()
    };
    let reader = RtpsReader::new::<Foo>(
        RtpsEndpoint::new(
            guid,
            TopicKind::WithKey,
            unicast_locator_list,
            multicast_locator_list,
        ),
        DURATION_ZERO,
        DURATION_ZERO,
        false,
        qos,
    );
    RtpsStatelessReader::new(reader)
}

fn create_builtin_stateful_reader<Foo>(guid: Guid) -> RtpsStatefulReader
where
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
{
    let qos = DataReaderQos {
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
    let topic_kind = TopicKind::WithKey;
    let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
    let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION;
    let expects_inline_qos = false;
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsStatefulReader::new(RtpsReader::new::<Foo>(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            unicast_locator_list,
            multicast_locator_list,
        ),
        heartbeat_response_delay,
        heartbeat_suppression_duration,
        expects_inline_qos,
        qos,
    ))
}
