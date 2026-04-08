mod participant_methods;
mod publisher_methods;
mod reader_methods;
mod subscriber_methods;
mod topic_methods;
mod writer_methods;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        actor::Actor,
        channels::{
            mpsc::MpscSender,
            oneshot::{OneshotSender, oneshot},
        },
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
            parameter_id_values::{PID_ENDPOINT_GUID, PID_PARTICIPANT_GUID},
            spdp_discovered_participant_data::{
                BuiltinEndpointQos, BuiltinEndpointSet, ParticipantProxy,
                SpdpDiscoveredParticipantData,
            },
        },
        dcps_mail::{DcpsMail, EventServiceMail},
        listeners::domain_participant_listener::ListenerMail,
        status_condition::DcpsStatusCondition,
        status_condition_mail::DcpsStatusConditionMail,
        xtypes_glue::key_and_instance_handle::get_instance_handle_from_dynamic_data,
    },
    dds_async::{
        content_filtered_topic::ContentFilteredTopicAsync, data_reader::DataReaderAsync,
        data_writer::DataWriterAsync, domain_participant::DomainParticipantAsync,
        publisher::PublisherAsync, subscriber::SubscriberAsync, topic::TopicAsync,
        topic_description::TopicDescriptionAsync,
    },
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos,
            TopicQos,
        },
        qos_policy::{
            BUILT_IN_DATA_REPRESENTATION, DATA_REPRESENTATION_QOS_POLICY_ID,
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            DataRepresentationQosPolicy, DestinationOrderQosPolicyKind, DurabilityQosPolicy,
            DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, LifespanQosPolicy,
            OWNERSHIP_QOS_POLICY_ID, OwnershipQosPolicyKind, PRESENTATION_QOS_POLICY_ID,
            QosPolicyId, RELIABILITY_QOS_POLICY_ID, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            ResourceLimitsQosPolicy, TransportPriorityQosPolicy, XCDR_DATA_REPRESENTATION,
            XCDR2_DATA_REPRESENTATION,
        },
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleRejectedStatus, SampleRejectedStatusKind,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, Time},
        type_support::TypeSupport,
    },
    rtps::{
        message_receiver::MessageReceiver,
        stateful_reader::RtpsStatefulReader,
        stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader,
        stateless_writer::RtpsStatelessWriter,
        types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    },
    rtps_messages::{
        overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        submessages::{
            data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            heartbeat::HeartbeatSubmessage,
        },
    },
    runtime::{Clock, DdsRuntime, Spawner, Timer},
    transport::{
        self,
        interface::{RtpsTransportParticipant, WriteMessage},
        types::{
            BUILT_IN_READER_GROUP, BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC, BUILT_IN_WRITER_GROUP,
            BUILT_IN_WRITER_WITH_KEY, CacheChange, ChangeKind, DurabilityKind,
            ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN, EntityId, Guid, GuidPrefix, ReliabilityKind,
            TopicKind,
        },
    },
    xtypes::{
        deserializer::CdrDeserializer,
        dynamic_type::{DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember},
        error::XTypesError,
        serializer::{
            Cdr1BeSerializer, Cdr1LeSerializer, Cdr2BeSerializer, Cdr2LeSerializer,
            RtpsPlCdrSerializer,
        },
    },
};
use alloc::{
    boxed::Box,
    collections::{BTreeSet, VecDeque},
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{
    future::{Future, poll_fn},
    pin::{Pin, pin},
    task::Poll,
};
use regex::Regex;
use tracing::info;

const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_READER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_READER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_READER_WITH_KEY);

fn poll_timeout<T>(
    mut timer_handle: impl Timer,
    duration: core::time::Duration,
    mut future: Pin<Box<dyn Future<Output = T> + Send>>,
) -> impl Future<Output = DdsResult<T>> {
    poll_fn(move |cx| {
        let timeout = timer_handle.delay(duration);
        if let Poll::Ready(t) = pin!(&mut future).poll(cx) {
            return Poll::Ready(Ok(t));
        }
        if pin!(timeout).poll(cx).is_ready() {
            return Poll::Ready(Err(DdsError::Timeout));
        }

        Poll::Pending
    })
}

pub struct DcpsDomainParticipant<R: DdsRuntime> {
    transport: RtpsTransportParticipant,
    topic_counter: u16,
    reader_counter: u16,
    writer_counter: u16,
    publisher_counter: u8,
    subscriber_counter: u8,
    domain_participant: DomainParticipantEntity,
    clock_handle: R::ClockHandle,
    timer_handle: R::TimerHandle,
    spawner_handle: R::SpawnerHandle,
    dcps_sender: MpscSender<DcpsMail>,
}

impl<R> DcpsDomainParticipant<R>
where
    R: DdsRuntime,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: DomainId,
        domain_tag: String,
        guid_prefix: GuidPrefix,
        domain_participant_qos: DomainParticipantQos,
        listener_sender: Option<MpscSender<ListenerMail>>,
        status_kind: Vec<StatusKind>,
        transport: RtpsTransportParticipant,
        dcps_sender: MpscSender<DcpsMail>,
        clock_handle: R::ClockHandle,
        timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
    ) -> Self {
        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        let participant_handle = InstanceHandle::new(guid.into());

        fn sedp_data_reader_qos() -> DataReaderQos {
            DataReaderQos {
                durability: DurabilityQosPolicy {
                    kind: DurabilityQosPolicyKind::TransientLocal,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
                },
                ..Default::default()
            }
        }

        fn sedp_data_writer_qos() -> DataWriterQos {
            DataWriterQos {
                durability: DurabilityQosPolicy {
                    kind: DurabilityQosPolicyKind::TransientLocal,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
                },
                representation: DataRepresentationQosPolicy {
                    value: vec![BUILT_IN_DATA_REPRESENTATION],
                },
                ..Default::default()
            }
        }

        // Create shared type information Arcs to avoid multiple allocations
        let spdp_participant_type = SpdpDiscoveredParticipantData::TYPE;
        let discovered_topic_type = DiscoveredTopicData::TYPE;
        let discovered_writer_type = DiscoveredWriterData::TYPE;
        let discovered_reader_type = DiscoveredReaderData::TYPE;

        let mut topic_list = Vec::new();

        let spdp_topic_participant_handle = [
            participant_handle[0],
            participant_handle[1],
            participant_handle[2],
            participant_handle[3],
            participant_handle[4],
            participant_handle[5],
            participant_handle[6],
            participant_handle[7],
            participant_handle[8],
            participant_handle[9],
            participant_handle[10],
            participant_handle[11],
            0,
            0,
            0,
            BUILT_IN_TOPIC,
        ];

        let spdp_topic_participant = TopicEntity::new(
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            String::from(DCPS_PARTICIPANT),
            InstanceHandle::new(spdp_topic_participant_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            spdp_participant_type,
        );

        topic_list.push(TopicDescriptionKind::Topic(spdp_topic_participant));

        let sedp_topic_topics_handle = [
            participant_handle[0],
            participant_handle[1],
            participant_handle[2],
            participant_handle[3],
            participant_handle[4],
            participant_handle[5],
            participant_handle[6],
            participant_handle[7],
            participant_handle[8],
            participant_handle[9],
            participant_handle[10],
            participant_handle[11],
            0,
            0,
            1,
            BUILT_IN_TOPIC,
        ];
        let sedp_topic_topics = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            String::from(DCPS_TOPIC),
            InstanceHandle::new(sedp_topic_topics_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            discovered_topic_type,
        );

        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_topics));

        let sedp_topic_publications_handle = [
            participant_handle[0],
            participant_handle[1],
            participant_handle[2],
            participant_handle[3],
            participant_handle[4],
            participant_handle[5],
            participant_handle[6],
            participant_handle[7],
            participant_handle[8],
            participant_handle[9],
            participant_handle[10],
            participant_handle[11],
            0,
            0,
            2,
            BUILT_IN_TOPIC,
        ];
        let sedp_topic_publications = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            String::from(DCPS_PUBLICATION),
            InstanceHandle::new(sedp_topic_publications_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            discovered_writer_type,
        );
        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_publications));

        let sedp_topic_subscriptions_handle = [
            participant_handle[0],
            participant_handle[1],
            participant_handle[2],
            participant_handle[3],
            participant_handle[4],
            participant_handle[5],
            participant_handle[6],
            participant_handle[7],
            participant_handle[8],
            participant_handle[9],
            participant_handle[10],
            participant_handle[11],
            0,
            0,
            3,
            BUILT_IN_TOPIC,
        ];
        let sedp_topic_subscriptions = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            String::from(DCPS_SUBSCRIPTION),
            InstanceHandle::new(sedp_topic_subscriptions_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            discovered_reader_type,
        );
        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_subscriptions));

        let spdp_writer_qos = DataWriterQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
            },
            representation: DataRepresentationQosPolicy {
                value: vec![BUILT_IN_DATA_REPRESENTATION],
            },
            ..Default::default()
        };
        let spdp_reader_qos = DataReaderQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
            },
            ..Default::default()
        };

        let rtps_stateless_reader = RtpsStatelessReader::new(Guid::new(
            guid_prefix,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
        ));

        let dcps_participant_reader = DataReaderEntity::new(
            InstanceHandle::new(rtps_stateless_reader.guid().into()),
            spdp_reader_qos,
            String::from(DCPS_PARTICIPANT),
            spdp_participant_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            RtpsReaderKind::Stateless(rtps_stateless_reader),
        );

        let dcps_topic_transport_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
            ReliabilityKind::Reliable,
        );

        let dcps_topic_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_topic_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_TOPIC),
            discovered_topic_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            RtpsReaderKind::Stateful(dcps_topic_transport_reader),
        );

        let dcps_publication_transport_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
            ReliabilityKind::Reliable,
        );

        let dcps_publication_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_publication_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_PUBLICATION),
            discovered_writer_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            RtpsReaderKind::Stateful(dcps_publication_transport_reader),
        );

        let dcps_subscription_transport_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
            ReliabilityKind::Reliable,
        );

        let dcps_subscription_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_subscription_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_SUBSCRIPTION),
            discovered_reader_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            RtpsReaderKind::Stateful(dcps_subscription_transport_reader),
        );

        let data_reader_list = vec![
            dcps_participant_reader,
            dcps_topic_reader,
            dcps_publication_reader,
            dcps_subscription_reader,
        ];
        let builtin_subscriber_handle = [
            participant_handle[0],
            participant_handle[1],
            participant_handle[2],
            participant_handle[3],
            participant_handle[4],
            participant_handle[5],
            participant_handle[6],
            participant_handle[7],
            participant_handle[8],
            participant_handle[9],
            participant_handle[10],
            participant_handle[11],
            0,
            0,
            0,
            BUILT_IN_READER_GROUP,
        ];
        let builtin_subscriber = SubscriberEntity::new(
            InstanceHandle::new(builtin_subscriber_handle),
            SubscriberQos::default(),
            data_reader_list,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
        );

        let mut dcps_participant_transport_writer = RtpsStatelessWriter::new(Guid::new(
            guid_prefix,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        ));
        for &discovery_locator in &transport.metatraffic_multicast_locator_list {
            dcps_participant_transport_writer.reader_locator_add(discovery_locator);
        }
        let dcps_participant_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_participant_transport_writer.guid().into()),
            RtpsWriterKind::Stateless(dcps_participant_transport_writer),
            String::from(DCPS_PARTICIPANT),
            "SpdpDiscoveredParticipantData".to_string(),
            spdp_participant_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            spdp_writer_qos,
        );

        let dcps_topics_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
            transport.fragment_size,
        );

        let dcps_topics_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_topics_transport_writer.guid().into()),
            RtpsWriterKind::Stateful(dcps_topics_transport_writer),
            String::from(DCPS_TOPIC),
            "DiscoveredTopicData".to_string(),
            discovered_topic_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        let dcps_publications_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
            transport.fragment_size,
        );

        let dcps_publications_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_publications_transport_writer.guid().into()),
            RtpsWriterKind::Stateful(dcps_publications_transport_writer),
            String::from(DCPS_PUBLICATION),
            "DiscoveredWriterData".to_string(),
            discovered_writer_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );

        let dcps_subscriptions_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
            transport.fragment_size,
        );
        let dcps_subscriptions_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_subscriptions_transport_writer.guid().into()),
            RtpsWriterKind::Stateful(dcps_subscriptions_transport_writer),
            String::from(DCPS_SUBSCRIPTION),
            "DiscoveredReaderData".to_string(),
            discovered_reader_type,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        let builtin_data_writer_list = vec![
            dcps_participant_writer,
            dcps_topics_writer,
            dcps_publications_writer,
            dcps_subscriptions_writer,
        ];
        let builtin_publisher_handle = [
            participant_handle[0],
            participant_handle[1],
            participant_handle[2],
            participant_handle[3],
            participant_handle[4],
            participant_handle[5],
            participant_handle[6],
            participant_handle[7],
            participant_handle[8],
            participant_handle[9],
            participant_handle[10],
            participant_handle[11],
            0,
            0,
            0,
            BUILT_IN_WRITER_GROUP,
        ];
        let builtin_publisher = PublisherEntity::new(
            PublisherQos::default(),
            InstanceHandle::new(builtin_publisher_handle),
            builtin_data_writer_list,
            None,
            vec![],
        );

        let domain_participant = DomainParticipantEntity::new(
            domain_id,
            domain_participant_qos,
            listener_sender,
            status_kind,
            participant_handle,
            builtin_publisher,
            builtin_subscriber,
            topic_list,
            domain_tag,
        );

        Self {
            transport,
            topic_counter: 0,
            reader_counter: 0,
            writer_counter: 0,
            publisher_counter: 0,
            subscriber_counter: 0,
            domain_participant,
            clock_handle,
            timer_handle,
            spawner_handle,
            dcps_sender,
        }
    }

    fn get_participant_async(&self) -> DomainParticipantAsync {
        DomainParticipantAsync::new(
            self.dcps_sender.clone(),
            self.domain_participant
                .builtin_subscriber()
                .status_condition
                .address(),
            self.domain_participant.domain_id,
            self.domain_participant.instance_handle,
        )
    }

    fn get_subscriber_async(
        &self,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<SubscriberAsync> {
        Ok(SubscriberAsync::new(
            subscriber_handle,
            self.domain_participant
                .user_defined_subscriber_list
                .iter()
                .find(|x| x.instance_handle == subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .status_condition
                .address(),
            self.get_participant_async(),
        ))
    }

    fn get_data_reader_async<Foo>(
        &self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderAsync<Foo>> {
        let data_reader = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataReaderAsync::new(
            data_reader_handle,
            data_reader.status_condition.address(),
            self.get_subscriber_async(subscriber_handle)?,
            self.get_topic_description_async(data_reader.topic_name.clone())?,
        ))
    }

    fn get_publisher_async(&self, publisher_handle: InstanceHandle) -> DdsResult<PublisherAsync> {
        Ok(PublisherAsync::new(
            publisher_handle,
            self.get_participant_async(),
        ))
    }

    fn get_data_writer_async<Foo>(
        &self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterAsync<Foo>> {
        let data_writer = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_writer_list
            .iter()
            .find(|x| x.instance_handle == data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataWriterAsync::new(
            data_writer_handle,
            data_writer.status_condition.address(),
            self.get_publisher_async(publisher_handle)?,
            self.get_topic_description_async(data_writer.topic_name.clone())?,
        ))
    }

    fn get_topic_description_async(&self, topic_name: String) -> DdsResult<TopicDescriptionAsync> {
        match self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        {
            Some(TopicDescriptionKind::Topic(topic)) => {
                Ok(TopicDescriptionAsync::Topic(TopicAsync::new(
                    topic.instance_handle,
                    topic.status_condition.address(),
                    topic.type_name.clone(),
                    topic_name,
                    self.get_participant_async(),
                )))
            }
            Some(TopicDescriptionKind::ContentFilteredTopic(t)) => {
                if let Some(TopicDescriptionKind::Topic(related_topic)) = self
                    .domain_participant
                    .topic_description_list
                    .iter()
                    .find(|x| x.topic_name() == t.related_topic_name)
                {
                    let name = t.topic_name.clone();
                    let topic = TopicAsync::new(
                        related_topic.instance_handle,
                        related_topic.status_condition.address(),
                        related_topic.type_name.clone(),
                        t.related_topic_name.clone(),
                        self.get_participant_async(),
                    );
                    Ok(TopicDescriptionAsync::ContentFilteredTopic(
                        ContentFilteredTopicAsync::new(name, topic),
                    ))
                } else {
                    Err(DdsError::AlreadyDeleted)
                }
            }
            None => Err(DdsError::AlreadyDeleted),
        }
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.domain_participant.instance_handle
    }

    pub fn get_builtin_subscriber_status_condition(&self) -> &Actor<DcpsStatusCondition> {
        &self.domain_participant.builtin_subscriber.status_condition
    }

    #[tracing::instrument(skip(self))]
    pub async fn announce_participant(&mut self) {
        if self.domain_participant.enabled {
            let builtin_topic_key = *self.domain_participant.instance_handle.as_ref();
            let guid = Guid::from(builtin_topic_key);
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: builtin_topic_key,
                },
                user_data: self.domain_participant.qos.user_data.clone(),
            };
            let participant_proxy = ParticipantProxy {
                domain_id: Some(self.domain_participant.domain_id),
                domain_tag: self.domain_participant.domain_tag.clone(),
                protocol_version: PROTOCOLVERSION,
                guid_prefix: guid.prefix(),
                vendor_id: VENDOR_ID_S2E,
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: self
                    .transport
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                metatraffic_multicast_locator_list: self
                    .transport
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                default_unicast_locator_list: self.transport.default_unicast_locator_list.to_vec(),
                default_multicast_locator_list: self
                    .transport
                    .default_multicast_locator_list
                    .to_vec(),
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: 0,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            };
            let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
                dds_participant_data: participant_builtin_topic_data,
                participant_proxy,
                lease_duration: Duration::new(100, 0),
                discovered_participant_list: self
                    .domain_participant
                    .discovered_participant_list
                    .iter()
                    .map(|p| InstanceHandle::new(p.dds_participant_data.key().value))
                    .collect(),
            };
            let data_writer_handle = InstanceHandle::new(
                Guid::new(
                    Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix(),
                    ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
                )
                .into(),
            );
            let timestamp = self.get_current_time();
            let (reply_sender, _) = oneshot();
            self.write_w_timestamp(
                self.domain_participant.builtin_publisher.instance_handle,
                data_writer_handle,
                spdp_discovered_participant_data.create_dynamic_sample(),
                timestamp,
                reply_sender,
            )
            .await;
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn announce_deleted_participant(&mut self) {
        if self.domain_participant.enabled {
            let timestamp = self.get_current_time();
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.topic_name == DCPS_PARTICIPANT)
            {
                let builtin_topic_key = *self.domain_participant.instance_handle.as_ref();
                let mut dynamic_data = DynamicDataFactory::create_data();
                dynamic_data
                    .set_complex_value(
                        PID_PARTICIPANT_GUID as u32,
                        BuiltInTopicKey {
                            value: builtin_topic_key,
                        }
                        .create_dynamic_sample(),
                    )
                    .unwrap();

                dw.unregister_w_timestamp(
                    dynamic_data,
                    timestamp,
                    self.transport.message_writer.as_ref(),
                    &self.clock_handle,
                )
                .await
                .ok();
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn announce_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return;
        };
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == data_writer.topic_name)
        else {
            return;
        };

        let topic_data = topic.qos.topic_data.clone();

        let dds_publication_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_writer.transport_writer.guid().into(),
            },
            participant_key: BuiltInTopicKey {
                value: self.domain_participant.instance_handle.into(),
            },
            topic_name: data_writer.topic_name.clone(),
            type_name: data_writer.type_name.clone(),
            durability: data_writer.qos.durability.clone(),
            deadline: data_writer.qos.deadline.clone(),
            latency_budget: data_writer.qos.latency_budget.clone(),
            liveliness: data_writer.qos.liveliness.clone(),
            reliability: data_writer.qos.reliability.clone(),
            lifespan: data_writer.qos.lifespan.clone(),
            user_data: data_writer.qos.user_data.clone(),
            ownership: data_writer.qos.ownership.clone(),
            ownership_strength: data_writer.qos.ownership_strength.clone(),
            destination_order: data_writer.qos.destination_order.clone(),
            presentation: publisher.qos.presentation.clone(),
            partition: publisher.qos.partition.clone(),
            topic_data,
            group_data: publisher.qos.group_data.clone(),
            representation: data_writer.qos.representation.clone(),
        };
        let writer_proxy = WriterProxy {
            remote_writer_guid: data_writer.transport_writer.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        };
        let discovered_writer_data = DiscoveredWriterData {
            dds_publication_data,
            writer_proxy,
        };
        let data_writer_handle = InstanceHandle::new(
            Guid::new(
                Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            )
            .into(),
        );
        let timestamp = self.get_current_time();
        let (reply_sender, _) = oneshot();
        self.write_w_timestamp(
            self.domain_participant.builtin_publisher.instance_handle,
            data_writer_handle,
            discovered_writer_data.create_dynamic_sample(),
            timestamp,
            reply_sender,
        )
        .await;
    }

    #[tracing::instrument(skip(self, data_writer))]
    async fn announce_deleted_data_writer(&mut self, data_writer: DataWriterEntity) {
        let timestamp = self.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_PUBLICATION)
        {
            let mut dynamic_data = DynamicDataFactory::create_data();
            dynamic_data
                .set_complex_value(
                    PID_ENDPOINT_GUID as u32,
                    BuiltInTopicKey {
                        value: data_writer.transport_writer.guid().into(),
                    }
                    .create_dynamic_sample(),
                )
                .unwrap();

            dw.unregister_w_timestamp(
                dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await
            .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    async fn announce_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return;
        };
        let Some(topic) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == data_reader.topic_name)
        else {
            return;
        };

        let (topic_name, type_name, topic_qos) = match topic {
            TopicDescriptionKind::Topic(t) => (&t.topic_name, &t.type_name, &t.qos),
            TopicDescriptionKind::ContentFilteredTopic(t) => {
                if let Some(TopicDescriptionKind::Topic(topic)) = self
                    .domain_participant
                    .topic_description_list
                    .iter()
                    .find(|x| x.topic_name() == t.related_topic_name)
                {
                    (&topic.topic_name, &topic.type_name, &topic.qos)
                } else {
                    return;
                }
            }
        };
        let guid = data_reader.transport_reader.guid();
        let dds_subscription_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey { value: guid.into() },
            participant_key: BuiltInTopicKey {
                value: self.domain_participant.instance_handle.into(),
            },
            topic_name: topic_name.clone(),
            type_name: type_name.clone(),
            durability: data_reader.qos.durability.clone(),
            deadline: data_reader.qos.deadline.clone(),
            latency_budget: data_reader.qos.latency_budget.clone(),
            liveliness: data_reader.qos.liveliness.clone(),
            reliability: data_reader.qos.reliability.clone(),
            ownership: data_reader.qos.ownership.clone(),
            destination_order: data_reader.qos.destination_order.clone(),
            user_data: data_reader.qos.user_data.clone(),
            time_based_filter: data_reader.qos.time_based_filter.clone(),
            presentation: subscriber.qos.presentation.clone(),
            partition: subscriber.qos.partition.clone(),
            topic_data: topic_qos.topic_data.clone(),
            group_data: subscriber.qos.group_data.clone(),
            representation: data_reader.qos.representation.clone(),
        };
        let reader_proxy = ReaderProxy {
            remote_reader_guid: data_reader.transport_reader.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        };
        let discovered_reader_data = DiscoveredReaderData {
            dds_subscription_data,
            reader_proxy,
        };
        let data_writer_handle = InstanceHandle::new(
            Guid::new(
                Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            )
            .into(),
        );
        let timestamp = self.get_current_time();
        let (reply_sender, _) = oneshot();
        self.write_w_timestamp(
            self.domain_participant.builtin_publisher.instance_handle,
            data_writer_handle,
            discovered_reader_data.create_dynamic_sample(),
            timestamp,
            reply_sender,
        )
        .await;
    }

    #[tracing::instrument(skip(self, data_reader))]
    async fn announce_deleted_data_reader(&mut self, data_reader: DataReaderEntity) {
        let timestamp = self.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_SUBSCRIPTION)
        {
            let mut dynamic_data = DynamicDataFactory::create_data();
            dynamic_data
                .set_complex_value(
                    PID_ENDPOINT_GUID as u32,
                    BuiltInTopicKey {
                        value: data_reader.transport_reader.guid().into(),
                    }
                    .create_dynamic_sample(),
                )
                .unwrap();
            dw.unregister_w_timestamp(
                dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await
            .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    async fn announce_topic(&mut self, topic_name: String) {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return;
        };

        let discovered_topic_data = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: topic.instance_handle.into(),
                },
                name: topic.topic_name.clone(),
                type_name: topic.type_name.clone(),
                durability: topic.qos.durability.clone(),
                deadline: topic.qos.deadline.clone(),
                latency_budget: topic.qos.latency_budget.clone(),
                liveliness: topic.qos.liveliness.clone(),
                reliability: topic.qos.reliability.clone(),
                transport_priority: topic.qos.transport_priority.clone(),
                lifespan: topic.qos.lifespan.clone(),
                destination_order: topic.qos.destination_order.clone(),
                history: topic.qos.history.clone(),
                resource_limits: topic.qos.resource_limits.clone(),
                ownership: topic.qos.ownership.clone(),
                topic_data: topic.qos.topic_data.clone(),
                representation: topic.qos.representation.clone(),
            },
        };

        let data_writer_handle = InstanceHandle::new(
            Guid::new(
                Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix(),
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            )
            .into(),
        );
        let timestamp = self.get_current_time();
        let (reply_sender, _) = oneshot();
        self.write_w_timestamp(
            self.domain_participant.builtin_publisher.instance_handle,
            data_writer_handle,
            discovered_topic_data.create_dynamic_sample(),
            timestamp,
            reply_sender,
        )
        .await;
    }

    #[tracing::instrument(skip(self))]
    async fn add_discovered_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.participant_proxy.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.participant_proxy.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return;
        };

        let is_any_name_matched = discovered_reader_data
            .dds_subscription_data
            .partition
            .name
            .iter()
            .any(|n| publisher.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_reader_data
            .dds_subscription_data
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                publisher
                    .qos
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = publisher
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                discovered_reader_data
                    .dds_subscription_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = discovered_reader_data.dds_subscription_data.partition
            == publisher.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let publisher_qos = publisher.qos.clone();
            let Some(data_writer) = publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == data_writer_handle)
            else {
                return;
            };

            let is_matched_topic_name =
                discovered_reader_data.dds_subscription_data.topic_name == data_writer.topic_name;
            let is_matched_type_name = discovered_reader_data.dds_subscription_data.get_type_name()
                == data_writer.type_name;

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_reader_incompatible_qos_policy_list(
                        &data_writer.qos,
                        &discovered_reader_data.dds_subscription_data,
                        &publisher_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_writer.add_matched_subscription(
                        discovered_reader_data.dds_subscription_data.clone(),
                    );

                    let unicast_locator_list = if discovered_reader_data
                        .reader_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        discovered_reader_data.reader_proxy.unicast_locator_list
                    };
                    let multicast_locator_list = if discovered_reader_data
                        .reader_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        discovered_reader_data.reader_proxy.multicast_locator_list
                    };
                    let reliability_kind = match discovered_reader_data
                        .dds_subscription_data
                        .reliability
                        .kind
                    {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind =
                        match discovered_reader_data.dds_subscription_data.durability.kind {
                            DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                            DurabilityQosPolicyKind::TransientLocal => {
                                DurabilityKind::TransientLocal
                            }
                            DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                            DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                        };

                    let reader_proxy = transport::types::ReaderProxy {
                        remote_reader_guid: discovered_reader_data.reader_proxy.remote_reader_guid,
                        remote_group_entity_id: discovered_reader_data
                            .reader_proxy
                            .remote_group_entity_id,
                        reliability_kind,
                        durability_kind,
                        unicast_locator_list,
                        multicast_locator_list,
                        expects_inline_qos: false,
                    };
                    if let RtpsWriterKind::Stateful(w) = &mut data_writer.transport_writer {
                        w.add_matched_reader(reader_proxy);
                    }

                    if data_writer
                        .listener_mask
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let status = data_writer.get_publication_matched_status();
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_writer.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .await
                                .ok();
                        }
                    } else if publisher
                        .listener_mask
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = &publisher.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .await
                                .ok();
                        }
                    }

                    let Some(publisher) = self
                        .domain_participant
                        .user_defined_publisher_list
                        .iter_mut()
                        .find(|x| x.instance_handle == publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) = publisher
                        .data_writer_list
                        .iter_mut()
                        .find(|x| x.instance_handle == data_writer_handle)
                    else {
                        return;
                    };
                    data_writer
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::PublicationMatched,
                        })
                        .await;
                } else {
                    data_writer.add_incompatible_subscription(
                        InstanceHandle::new(
                            discovered_reader_data.dds_subscription_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_writer
                        .listener_mask
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let status = data_writer.get_offered_incompatible_qos_status();
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_writer.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .await
                                .ok();
                        }
                    } else if publisher
                        .listener_mask
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = &publisher.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .await
                                .ok();
                        }
                    }

                    let Some(publisher) = self
                        .domain_participant
                        .user_defined_publisher_list
                        .iter_mut()
                        .find(|x| x.instance_handle == publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) = publisher
                        .data_writer_list
                        .iter_mut()
                        .find(|x| x.instance_handle == data_writer_handle)
                    else {
                        return;
                    };
                    data_writer
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::OfferedIncompatibleQos,
                        })
                        .await;
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn remove_discovered_reader(
        &mut self,
        subscription_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return;
        };
        if data_writer
            .matched_subscription_list
            .iter()
            .any(|x| subscription_handle.as_ref() == &x.key().value)
        {
            data_writer.remove_matched_subscription(&subscription_handle);

            data_writer
                .status_condition
                .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                    state: StatusKind::PublicationMatched,
                })
                .await;
        }
    }

    #[tracing::instrument(skip(self))]
    async fn add_discovered_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.participant_proxy.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.participant_proxy.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let is_any_name_matched = discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .any(|n| subscriber.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                subscriber
                    .qos
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = subscriber
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                discovered_writer_data
                    .dds_publication_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = discovered_writer_data.dds_publication_data.partition
            == subscriber.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let subscriber_qos = subscriber.qos.clone();
            let Some(data_reader) = subscriber
                .data_reader_list
                .iter_mut()
                .find(|x| x.instance_handle == data_reader_handle)
            else {
                return;
            };
            let Some(matched_topic) = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|t| t.topic_name() == data_reader.topic_name)
            else {
                return;
            };
            let (reader_topic_name, reader_type_name) = match matched_topic {
                TopicDescriptionKind::Topic(t) => (&t.topic_name, &t.type_name),
                TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic) => {
                    if let Some(TopicDescriptionKind::Topic(matched_topic)) = self
                        .domain_participant
                        .topic_description_list
                        .iter()
                        .find(|t| t.topic_name() == content_filtered_topic.related_topic_name)
                    {
                        (&matched_topic.topic_name, &matched_topic.type_name)
                    } else {
                        return;
                    }
                }
            };
            let is_matched_topic_name =
                &discovered_writer_data.dds_publication_data.topic_name == reader_topic_name;
            let is_matched_type_name =
                discovered_writer_data.dds_publication_data.get_type_name() == reader_type_name;

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_writer_incompatible_qos_policy_list(
                        data_reader,
                        &discovered_writer_data.dds_publication_data,
                        &subscriber_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_reader.add_matched_publication(
                        discovered_writer_data.dds_publication_data.clone(),
                    );
                    let unicast_locator_list = if discovered_writer_data
                        .writer_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        discovered_writer_data.writer_proxy.unicast_locator_list
                    };
                    let multicast_locator_list = if discovered_writer_data
                        .writer_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        discovered_writer_data.writer_proxy.multicast_locator_list
                    };
                    let reliability_kind = match data_reader.qos.reliability.kind {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match data_reader.qos.durability.kind {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };
                    let writer_proxy = transport::types::WriterProxy {
                        remote_writer_guid: discovered_writer_data.writer_proxy.remote_writer_guid,
                        remote_group_entity_id: discovered_writer_data
                            .writer_proxy
                            .remote_group_entity_id,
                        unicast_locator_list,
                        multicast_locator_list,
                        reliability_kind,
                        durability_kind,
                    };
                    if let RtpsReaderKind::Stateful(r) = &mut data_reader.transport_reader {
                        r.add_matched_writer(&writer_proxy);
                    }

                    if data_reader
                        .listener_mask
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if subscriber
                        .listener_mask
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber
                        .data_reader_list
                        .iter_mut()
                        .find(|x| x.instance_handle == data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::SubscriptionMatched,
                        })
                        .await;
                } else {
                    data_reader.add_requested_incompatible_qos(
                        InstanceHandle::new(
                            discovered_writer_data.dds_publication_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_reader
                        .listener_mask
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let status = data_reader.get_requested_incompatible_qos_status();
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if subscriber
                        .listener_mask
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber
                        .data_reader_list
                        .iter_mut()
                        .find(|x| x.instance_handle == data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::RequestedIncompatibleQos,
                        })
                        .await;
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn remove_discovered_writer(
        &mut self,
        publication_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return;
        };
        if data_reader
            .matched_publication_list
            .iter()
            .any(|x| &x.key().value == publication_handle.as_ref())
        {
            data_reader
                .remove_matched_publication(&publication_handle)
                .await;
        }
    }

    #[tracing::instrument(skip(self))]
    async fn add_cache_change(
        &mut self,
        cache_change: CacheChange,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let reader_guid = Guid::from(<[u8; 16]>::from(data_reader_handle));
        match reader_guid.entity_id() {
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER => {
                self.add_builtin_participants_detector_cache_change(cache_change)
                    .await
            }
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR => {
                self.add_builtin_publications_detector_cache_change(cache_change)
                    .await
            }
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR => {
                self.add_builtin_subscriptions_detector_cache_change(cache_change)
                    .await
            }
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR => {
                self.add_builtin_topics_detector_cache_change(cache_change)
                    .await
            }
            _ => {
                self.add_user_defined_cache_change(
                    cache_change,
                    subscriber_handle,
                    data_reader_handle,
                )
                .await
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_builtin_participants_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
    ) {
        let spdp_type_support =
            if let Some(TopicDescriptionKind::Topic(discovered_participant_data_type)) = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|n| n.topic_name() == DCPS_PARTICIPANT)
            {
                discovered_participant_data_type.type_support
            } else {
                return;
            };
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    spdp_type_support,
                    cache_change.data_value.as_ref(),
                ) {
                    let discovered_participant_data =
                        SpdpDiscoveredParticipantData::create_sample(dynamic_data);

                    self.add_discovered_participant(discovered_participant_data)
                        .await;
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                let discovered_participant_handle = if let Some(h) = cache_change.instance_handle {
                    InstanceHandle::new(h)
                } else if let Ok(dynamic_data) = CdrDeserializer::deserialize(
                    InstanceHandle::TYPE,
                    cache_change.data_value.as_ref(),
                ) {
                    InstanceHandle::create_sample(dynamic_data)
                } else {
                    return;
                };

                self.remove_discovered_participant(discovered_participant_handle)
                    .await;
            }
            ChangeKind::AliveFiltered | ChangeKind::NotAliveUnregistered => (), // Do nothing,
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_PARTICIPANT)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_builtin_publications_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
    ) {
        let sedp_writer_type_support =
            if let Some(TopicDescriptionKind::Topic(discovered_participant_data_type)) = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|n| n.topic_name() == DCPS_PUBLICATION)
            {
                discovered_participant_data_type.type_support
            } else {
                return;
            };
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    sedp_writer_type_support,
                    cache_change.data_value.as_ref(),
                ) {
                    let discovered_writer_data = DiscoveredWriterData::create_sample(dynamic_data);
                    let publication_builtin_topic_data =
                        &discovered_writer_data.dds_publication_data;
                    if self
                        .domain_participant
                        .find_topic(&publication_builtin_topic_data.topic_name)
                        .is_none()
                    {
                        let writer_topic = TopicBuiltinTopicData {
                            key: BuiltInTopicKey::default(),
                            name: publication_builtin_topic_data.topic_name.clone(),
                            type_name: publication_builtin_topic_data.type_name.clone(),
                            durability: publication_builtin_topic_data.durability().clone(),
                            deadline: publication_builtin_topic_data.deadline().clone(),
                            latency_budget: publication_builtin_topic_data.latency_budget().clone(),
                            liveliness: publication_builtin_topic_data.liveliness().clone(),
                            reliability: publication_builtin_topic_data.reliability().clone(),
                            transport_priority: TransportPriorityQosPolicy::default(),
                            lifespan: publication_builtin_topic_data.lifespan().clone(),
                            destination_order: publication_builtin_topic_data
                                .destination_order()
                                .clone(),
                            history: HistoryQosPolicy::default(),
                            resource_limits: ResourceLimitsQosPolicy::default(),
                            ownership: publication_builtin_topic_data.ownership().clone(),
                            topic_data: publication_builtin_topic_data.topic_data().clone(),
                            representation: publication_builtin_topic_data.representation().clone(),
                        };
                        self.domain_participant.add_discovered_topic(writer_topic);
                    }

                    self.domain_participant
                        .add_discovered_writer(discovered_writer_data.clone());
                    let mut handle_list = Vec::new();
                    for subscriber in &self.domain_participant.user_defined_subscriber_list {
                        for data_reader in subscriber.data_reader_list.iter() {
                            handle_list
                                .push((subscriber.instance_handle, data_reader.instance_handle));
                        }
                    }
                    for (subscriber_handle, data_reader_handle) in handle_list {
                        self.add_discovered_writer(
                            discovered_writer_data.clone(),
                            subscriber_handle,
                            data_reader_handle,
                        )
                        .await;
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                let discovered_writer_handle = if let Some(h) = cache_change.instance_handle {
                    InstanceHandle::new(h)
                } else if let Ok(dynamic_data) = CdrDeserializer::deserialize(
                    InstanceHandle::TYPE,
                    cache_change.data_value.as_ref(),
                ) {
                    InstanceHandle::create_sample(dynamic_data)
                } else {
                    return;
                };

                self.domain_participant
                    .remove_discovered_writer(&discovered_writer_handle);

                let mut handle_list = Vec::new();
                for subscriber in &self.domain_participant.user_defined_subscriber_list {
                    for data_reader in subscriber.data_reader_list.iter() {
                        handle_list.push((subscriber.instance_handle, data_reader.instance_handle));
                    }
                }
                for (subscriber_handle, data_reader_handle) in handle_list {
                    self.remove_discovered_writer(
                        discovered_writer_handle,
                        subscriber_handle,
                        data_reader_handle,
                    )
                    .await;
                }
            }
            ChangeKind::AliveFiltered | ChangeKind::NotAliveUnregistered => (),
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_PUBLICATION)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_builtin_subscriptions_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
    ) {
        let sedp_reader_type_support =
            if let Some(TopicDescriptionKind::Topic(discovered_participant_data_type)) = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|n| n.topic_name() == DCPS_SUBSCRIPTION)
            {
                discovered_participant_data_type.type_support
            } else {
                return;
            };
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    sedp_reader_type_support,
                    cache_change.data_value.as_ref(),
                ) {
                    let discovered_reader_data = DiscoveredReaderData::create_sample(dynamic_data);
                    if self
                        .domain_participant
                        .find_topic(&discovered_reader_data.dds_subscription_data.topic_name)
                        .is_none()
                    {
                        let reader_topic = TopicBuiltinTopicData {
                            key: BuiltInTopicKey::default(),
                            name: discovered_reader_data
                                .dds_subscription_data
                                .topic_name
                                .to_string(),
                            type_name: discovered_reader_data
                                .dds_subscription_data
                                .get_type_name()
                                .to_string(),

                            topic_data: discovered_reader_data
                                .dds_subscription_data
                                .topic_data()
                                .clone(),
                            durability: discovered_reader_data
                                .dds_subscription_data
                                .durability()
                                .clone(),
                            deadline: discovered_reader_data
                                .dds_subscription_data
                                .deadline()
                                .clone(),
                            latency_budget: discovered_reader_data
                                .dds_subscription_data
                                .latency_budget()
                                .clone(),
                            liveliness: discovered_reader_data
                                .dds_subscription_data
                                .liveliness()
                                .clone(),
                            reliability: discovered_reader_data
                                .dds_subscription_data
                                .reliability()
                                .clone(),
                            destination_order: discovered_reader_data
                                .dds_subscription_data
                                .destination_order()
                                .clone(),
                            history: HistoryQosPolicy::default(),
                            resource_limits: ResourceLimitsQosPolicy::default(),
                            transport_priority: TransportPriorityQosPolicy::default(),
                            lifespan: LifespanQosPolicy::default(),
                            ownership: discovered_reader_data
                                .dds_subscription_data
                                .ownership()
                                .clone(),
                            representation: discovered_reader_data
                                .dds_subscription_data
                                .representation()
                                .clone(),
                        };
                        self.domain_participant.add_discovered_topic(reader_topic);
                    }

                    self.domain_participant
                        .add_discovered_reader(discovered_reader_data.clone());
                    let mut handle_list = Vec::new();
                    for publisher in &self.domain_participant.user_defined_publisher_list {
                        for data_writer in publisher.data_writer_list.iter() {
                            handle_list
                                .push((publisher.instance_handle, data_writer.instance_handle));
                        }
                    }
                    for (publisher_handle, data_writer_handle) in handle_list {
                        self.add_discovered_reader(
                            discovered_reader_data.clone(),
                            publisher_handle,
                            data_writer_handle,
                        )
                        .await;
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                let discovered_reader_handle = if let Some(h) = cache_change.instance_handle {
                    InstanceHandle::new(h)
                } else if let Ok(dynamic_data) = CdrDeserializer::deserialize(
                    InstanceHandle::TYPE,
                    cache_change.data_value.as_ref(),
                ) {
                    InstanceHandle::create_sample(dynamic_data)
                } else {
                    return;
                };

                self.domain_participant
                    .remove_discovered_reader(&discovered_reader_handle);

                let mut handle_list = Vec::new();
                for publisher in &self.domain_participant.user_defined_publisher_list {
                    for data_writer in publisher.data_writer_list.iter() {
                        handle_list.push((publisher.instance_handle, data_writer.instance_handle));
                    }
                }

                for (publisher_handle, data_writer_handle) in handle_list {
                    self.remove_discovered_reader(
                        discovered_reader_handle,
                        publisher_handle,
                        data_writer_handle,
                    )
                    .await;
                }
            }
            ChangeKind::AliveFiltered | ChangeKind::NotAliveUnregistered => (),
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_SUBSCRIPTION)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_builtin_topics_detector_cache_change(&mut self, cache_change: CacheChange) {
        let sedp_topic_type_support =
            if let Some(TopicDescriptionKind::Topic(discovered_participant_data_type)) = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|n| n.topic_name() == DCPS_TOPIC)
            {
                discovered_participant_data_type.type_support
            } else {
                return;
            };
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    sedp_topic_type_support,
                    cache_change.data_value.as_ref(),
                ) {
                    let topic_builtin_topic_data =
                        TopicBuiltinTopicData::create_sample(dynamic_data);

                    self.domain_participant
                        .add_discovered_topic(topic_builtin_topic_data.clone());
                    for topic in self.domain_participant.topic_description_list.iter_mut() {
                        if let TopicDescriptionKind::Topic(topic) = topic {
                            if topic.topic_name == topic_builtin_topic_data.name()
                                && topic.type_name == topic_builtin_topic_data.get_type_name()
                                && !is_discovered_topic_consistent(
                                    &topic.qos,
                                    &topic_builtin_topic_data,
                                )
                            {
                                topic.inconsistent_topic_status.total_count += 1;
                                topic.inconsistent_topic_status.total_count_change += 1;
                                topic
                                    .status_condition
                                    .send_actor_mail(
                                        DcpsStatusConditionMail::AddCommunicationState {
                                            state: StatusKind::InconsistentTopic,
                                        },
                                    )
                                    .await;
                            }
                        }
                    }
                }
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (),
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_TOPIC)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_user_defined_cache_change(
        &mut self,
        cache_change: CacheChange,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let reception_timestamp = self.get_current_time();
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return;
        };
        let writer_instance_handle = InstanceHandle::new(cache_change.writer_guid.into());

        if data_reader
            .matched_publication_list
            .iter()
            .any(|x| &x.key().value == writer_instance_handle.as_ref())
        {
            let Some(reader_topic) = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|t| t.topic_name() == data_reader.topic_name)
            else {
                return;
            };

            if let TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic) = reader_topic
            {
                if cache_change.kind == ChangeKind::Alive {
                    let Ok(data) = CdrDeserializer::deserialize(
                        data_reader.type_support,
                        cache_change.data_value.as_ref(),
                    ) else {
                        return;
                    };
                    enum Operator {
                        LessThan,
                        Equal,
                    }

                    impl Operator {
                        fn to_str(&self) -> &'static str {
                            match self {
                                Self::Equal => "=",
                                Self::LessThan => "<=",
                            }
                        }

                        fn compare_string(&self, lhs: &String, rhs: &String) -> bool {
                            match self {
                                Self::Equal => lhs == rhs,
                                Self::LessThan => lhs <= rhs,
                            }
                        }
                        fn compare_int32(&self, lhs: &i32, rhs: &i32) -> bool {
                            match self {
                                Self::Equal => lhs == rhs,
                                Self::LessThan => lhs <= rhs,
                            }
                        }
                    }

                    let mut operators = [Operator::LessThan, Operator::Equal].iter();
                    let filter = loop {
                        if let Some(operator) = operators.next() {
                            if let Some((variable_name, _)) = content_filtered_topic
                                .filter_expression
                                .split_once(operator.to_str())
                            {
                                break Some((variable_name, operator));
                            }
                        } else {
                            break None;
                        };
                    };

                    if let Some((variable_name, comparison_function)) = filter {
                        let Some(member_id) = data
                            .get_member_id_by_name(data_reader.type_support, variable_name.trim())
                        else {
                            return;
                        };
                        let Ok(member_descriptor) =
                            data.get_descriptor(data_reader.type_support, member_id)
                        else {
                            return;
                        };
                        match member_descriptor.r#type.get_kind() {
                            crate::xtypes::dynamic_type::TypeKind::NONE => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::BOOLEAN => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::BYTE => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::INT16 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::INT32 => {
                                let member_value = data.get_int32_value(member_id).unwrap();
                                if !comparison_function.compare_int32(
                                    member_value,
                                    &content_filtered_topic.expression_parameters[0]
                                        .parse()
                                        .expect("valid number"),
                                ) {
                                    return;
                                }
                            }
                            crate::xtypes::dynamic_type::TypeKind::INT64 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::UINT16 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::UINT32 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::UINT64 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::FLOAT32 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::FLOAT64 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::FLOAT128 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::INT8 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::UINT8 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::CHAR8 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::CHAR16 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::STRING8 => {
                                let member_value = data.get_string_value(member_id).unwrap();
                                if !comparison_function.compare_string(
                                    member_value,
                                    &content_filtered_topic.expression_parameters[0],
                                ) {
                                    return;
                                }
                            }
                            crate::xtypes::dynamic_type::TypeKind::STRING16 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::ALIAS => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::ENUM => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::BITMASK => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::ANNOTATION => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::STRUCTURE => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::UNION => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::BITSET => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::SEQUENCE => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::ARRAY => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::MAP => todo!(),
                        }
                    } else {
                        return;
                    };
                }
            }

            let participant_handle = self.domain_participant.instance_handle;
            match data_reader.add_reader_change(cache_change, reception_timestamp) {
                Ok(AddChangeResult::Added(change_instance_handle)) => {
                    info!("New change added");
                    if let DurationKind::Finite(deadline_missed_period) =
                        data_reader.qos.deadline.period
                    {
                        let mut timer_handle = self.timer_handle.clone();
                        let dcps_sender = self.dcps_sender.clone();

                        self.spawner_handle.spawn(async move {
                            loop {
                                timer_handle.delay(deadline_missed_period.into()).await;
                                dcps_sender
                                    .send(DcpsMail::Event(
                                        EventServiceMail::RequestedDeadlineMissed {
                                            participant_handle,
                                            subscriber_handle,
                                            data_reader_handle,
                                            change_instance_handle,
                                        },
                                    ))
                                    .await
                                    .ok();
                            }
                        });
                    }
                    let data_reader_on_data_available_active = data_reader
                        .listener_mask
                        .contains(&StatusKind::DataAvailable);

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };

                    if subscriber
                        .listener_mask
                        .contains(&StatusKind::DataOnReaders)
                    {
                        let Ok(the_subscriber) = self.get_subscriber_async(subscriber_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::DataOnReaders { the_subscriber })
                                .await
                                .ok();
                        }
                    } else if data_reader_on_data_available_active {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_reader.listener_sender {
                            info!("Triggering data reader DataAvailable listener");
                            l.send(ListenerMail::DataAvailable { the_reader })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };

                    subscriber
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::DataOnReaders,
                        })
                        .await;
                    let Some(data_reader) = subscriber
                        .data_reader_list
                        .iter_mut()
                        .find(|x| x.instance_handle == data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::DataAvailable,
                        })
                        .await;
                }
                Ok(AddChangeResult::NotAdded) => (), // Do nothing
                Ok(AddChangeResult::Rejected(instance_handle, sample_rejected_status_kind)) => {
                    info!("Change rejected");
                    data_reader.increment_sample_rejected_status(
                        instance_handle,
                        sample_rejected_status_kind,
                    );

                    if data_reader
                        .listener_mask
                        .contains(&StatusKind::SampleRejected)
                    {
                        let status = data_reader.get_sample_rejected_status();
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::SampleRejected { the_reader, status })
                                .await
                                .ok();
                        };
                    } else if subscriber
                        .listener_mask
                        .contains(&StatusKind::SampleRejected)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_sample_rejected_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::SampleRejected { status, the_reader })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .contains(&StatusKind::SampleRejected)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_sample_rejected_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::SampleRejected { status, the_reader })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };

                    let Some(data_reader) = subscriber
                        .data_reader_list
                        .iter_mut()
                        .find(|x| x.instance_handle == data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::SampleRejected,
                        })
                        .await;
                }
                Err(_) => (),
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove_writer_change(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        sequence_number: i64,
    ) {
        if let Some(p) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        {
            if let Some(dw) = p
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == data_writer_handle)
            {
                dw.transport_writer.remove_change(sequence_number).await;
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn offered_deadline_missed(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
    ) {
        let current_time = self.get_current_time();
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return;
        };

        if let DurationKind::Finite(deadline) = data_writer.qos.deadline.period {
            match data_writer.get_instance_write_time(change_instance_handle) {
                Some(t) => {
                    if current_time - t < deadline {
                        return;
                    }
                }
                None => return,
            }
        } else {
            return;
        }

        data_writer
            .offered_deadline_missed_status
            .last_instance_handle = change_instance_handle;
        data_writer.offered_deadline_missed_status.total_count += 1;
        data_writer
            .offered_deadline_missed_status
            .total_count_change += 1;

        if data_writer
            .listener_mask
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let status = data_writer.get_offered_deadline_missed_status().await;
            let Ok(the_writer) = self.get_data_writer_async(publisher_handle, data_writer_handle)
            else {
                return;
            };

            let Some(publisher) = self
                .domain_participant
                .user_defined_publisher_list
                .iter_mut()
                .find(|x| x.instance_handle == publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == data_writer_handle)
            else {
                return;
            };

            if let Some(l) = &data_writer.listener_sender {
                l.send(ListenerMail::OfferedDeadlineMissed { the_writer, status })
                    .await
                    .ok();
            }
        } else if publisher
            .listener_mask
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let Ok(the_writer) = self.get_data_writer_async(publisher_handle, data_writer_handle)
            else {
                return;
            };
            let Some(publisher) = self
                .domain_participant
                .user_defined_publisher_list
                .iter_mut()
                .find(|x| x.instance_handle == publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == data_writer_handle)
            else {
                return;
            };
            let status = data_writer.get_offered_deadline_missed_status().await;
            if let Some(l) = &publisher.listener_sender {
                l.send(ListenerMail::OfferedDeadlineMissed { the_writer, status })
                    .await
                    .ok();
            }
        } else if self
            .domain_participant
            .listener_mask
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let Ok(the_writer) = self.get_data_writer_async(publisher_handle, data_writer_handle)
            else {
                return;
            };

            let Some(publisher) = self
                .domain_participant
                .user_defined_publisher_list
                .iter_mut()
                .find(|x| x.instance_handle == publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.instance_handle == data_writer_handle)
            else {
                return;
            };
            let status = data_writer.get_offered_deadline_missed_status().await;
            if let Some(l) = &self.domain_participant.listener_sender {
                l.send(ListenerMail::OfferedDeadlineMissed { the_writer, status })
                    .await
                    .ok();
            }
        }

        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return;
        };
        data_writer
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                state: StatusKind::OfferedDeadlineMissed,
            })
            .await;
    }

    #[tracing::instrument(skip(self))]
    pub async fn requested_deadline_missed(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
    ) {
        let current_time = self.get_current_time();
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return;
        };

        if let DurationKind::Finite(deadline) = data_reader.qos.deadline.period {
            if let Some(t) = data_reader.get_instance_received_time(&change_instance_handle) {
                if current_time - t < deadline {
                    return;
                }
            } else {
                return;
            }
        }

        data_reader.remove_instance_ownership(&change_instance_handle);
        data_reader.increment_requested_deadline_missed_status(change_instance_handle);

        if data_reader
            .listener_mask
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let status = data_reader.get_requested_deadline_missed_status();
            let Ok(the_reader) = self.get_data_reader_async(subscriber_handle, data_reader_handle)
            else {
                return;
            };
            let Some(subscriber) = self
                .domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber
                .data_reader_list
                .iter_mut()
                .find(|x| x.instance_handle == data_reader_handle)
            else {
                return;
            };
            if let Some(l) = &data_reader.listener_sender {
                l.send(ListenerMail::RequestedDeadlineMissed { the_reader, status })
                    .await
                    .ok();
            }
        } else if subscriber
            .listener_mask
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let Ok(the_reader) = self.get_data_reader_async(subscriber_handle, data_reader_handle)
            else {
                return;
            };

            let Some(subscriber) = self
                .domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber
                .data_reader_list
                .iter_mut()
                .find(|x| x.instance_handle == data_reader_handle)
            else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = &subscriber.listener_sender {
                l.send(ListenerMail::RequestedDeadlineMissed { status, the_reader })
                    .await
                    .ok();
            }
        } else if self
            .domain_participant
            .listener_mask
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let Ok(the_reader) = self.get_data_reader_async(subscriber_handle, data_reader_handle)
            else {
                return;
            };

            let Some(subscriber) = self
                .domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber
                .data_reader_list
                .iter_mut()
                .find(|x| x.instance_handle == data_reader_handle)
            else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = &self.domain_participant.listener_sender {
                l.send(ListenerMail::RequestedDeadlineMissed { status, the_reader })
                    .await
                    .ok();
            }
        }
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return;
        };

        data_reader
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                state: StatusKind::RequestedDeadlineMissed,
            })
            .await;
    }

    #[tracing::instrument(skip(self))]
    async fn add_discovered_participant(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // IN CASE no domain id was transmitted the a local domain id is assumed
        // (as specified in Table 9.19 - ParameterId mapping and default values)
        let is_domain_id_matching = match discovered_participant_data.participant_proxy.domain_id {
            Some(id) => id == self.domain_participant.domain_id,
            None => true,
        };
        let is_domain_tag_matching = discovered_participant_data.participant_proxy.domain_tag
            == self.domain_participant.domain_tag;
        let is_participant_discovered = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .any(|p| {
                p.dds_participant_data.key().value
                    == discovered_participant_data.dds_participant_data.key.value
            });
        let is_participant_ignored = self
            .domain_participant
            .ignored_participants
            .iter()
            .any(|handle| handle == &discovered_participant_data.dds_participant_data.key.value);

        if is_domain_id_matching
            && is_domain_tag_matching
            && !is_participant_discovered
            && !is_participant_ignored
        {
            self.add_matched_publications_detector(&discovered_participant_data);
            self.add_matched_publications_announcer(&discovered_participant_data);
            self.add_matched_subscriptions_detector(&discovered_participant_data);
            self.add_matched_subscriptions_announcer(&discovered_participant_data);
            self.add_matched_topics_detector(&discovered_participant_data);
            self.add_matched_topics_announcer(&discovered_participant_data);

            self.announce_participant().await;

            self.domain_participant
                .add_discovered_participant(discovered_participant_data);
        }
    }

    /// Remove discovered [domain participant](SpdpDiscoveredParticipantData) with the speficied [handle](InstanceHandle).
    #[tracing::instrument(skip(self))]
    async fn remove_discovered_participant(&mut self, handle: InstanceHandle) {
        self.domain_participant
            .discovered_participant_list
            .retain(|domain_participant| {
                domain_participant.dds_participant_data.key.value != handle
            });

        let prefix = Guid::from(<[u8; 16]>::from(handle)).prefix();

        for subscriber in &mut self.domain_participant.user_defined_subscriber_list {
            for data_reader in &mut subscriber.data_reader_list {
                // Remove samples
                data_reader
                    .sample_list
                    .retain(|sample| sample.writer_guid[..12] != prefix);

                for matched_publication in &data_reader.matched_publication_list {
                    if matched_publication.key.value[0..12] == prefix {
                        // Remove matched writers
                        if let RtpsReaderKind::Stateful(stateful_reader) =
                            &mut data_reader.transport_reader
                        {
                            stateful_reader
                                .delete_matched_writer(matched_publication.key.value.into());
                        }
                    }
                }
            }
        }

        for publisher in &mut self.domain_participant.user_defined_publisher_list {
            for data_writer in &mut publisher.data_writer_list {
                for matched_subscription in &data_writer.matched_subscription_list {
                    if matched_subscription.key.value[..12] == prefix {
                        // Remove readers
                        if let RtpsWriterKind::Stateful(stateful_writer) =
                            &mut data_writer.transport_writer
                        {
                            stateful_writer
                                .delete_matched_reader(matched_subscription.key.value.into());
                        }
                    }
                }
                data_writer
                    .matched_subscription_list
                    .retain(|subscription| subscription.key.value[..12] != prefix);
            }
        }

        self.remove_matched_publications_detector(prefix);
        self.remove_matched_publications_announcer(prefix);

        self.remove_matched_subscriptions_detector(prefix);
        self.remove_matched_subscriptions_announcer(prefix);

        self.remove_matched_topics_detector(prefix);
        self.remove_matched_topics_announcer(prefix);
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_publications_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_publications_detector(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| {
                dw.transport_writer.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            })
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_publications_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_publications_announcer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| {
                dr.transport_reader.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR
            })
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_subscriptions_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_subscriptions_detector(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| {
                dw.transport_writer.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            })
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_subscriptions_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_subscriptions_announcer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| {
                dr.transport_reader.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
            })
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_topics_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_topics_detector(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| {
                dw.transport_writer.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER
            })
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_topics_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_topics_announcer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| {
                dr.transport_reader.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
            })
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self, data_message))]
    pub async fn handle_data(&mut self, data_message: Arc<[u8]>) {
        if let Ok(rtps_message) = RtpsMessageRead::try_from(data_message.as_ref()) {
            let mut message_receiver = MessageReceiver::new(&rtps_message);

            while let Some(submessage) = message_receiver.next() {
                match submessage {
                    RtpsSubmessageReadKind::Data(data_submessage) => {
                        self.handle_data_submessage(&message_receiver, data_submessage)
                            .await;
                    }
                    RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                        self.handle_data_frag_submessage(&message_receiver, data_frag_submessage)
                            .await;
                    }
                    RtpsSubmessageReadKind::Gap(gap_submessage) => {
                        self.handle_gap_submessage(&message_receiver, gap_submessage);
                    }
                    RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
                        self.handle_heartbeat_submessage(&message_receiver, heartbeat_submessage)
                            .await;
                    }
                    RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => {
                        for subscriber in self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .chain(core::iter::once(
                                &mut self.domain_participant.builtin_subscriber,
                            ))
                        {
                            for dr in &mut subscriber.data_reader_list {
                                match &mut dr.transport_reader {
                                    RtpsReaderKind::Stateful(r) => {
                                        let writer_guid = Guid::new(
                                            message_receiver.source_guid_prefix(),
                                            heartbeat_frag_submessage.writer_id(),
                                        );
                                        if let Some(writer_proxy) =
                                            r.matched_writer_lookup(writer_guid)
                                        {
                                            if writer_proxy.last_received_heartbeat_count()
                                                < heartbeat_frag_submessage.count()
                                            {
                                                writer_proxy
                                                    .set_last_received_heartbeat_frag_count(
                                                        heartbeat_frag_submessage.count(),
                                                    );
                                            }
                                        }
                                    }
                                    RtpsReaderKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::AckNack(ack_nack_submessage) => {
                        for publisher in self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .chain(core::iter::once(
                                &mut self.domain_participant.builtin_publisher,
                            ))
                        {
                            for dw in &mut publisher.data_writer_list {
                                match &mut dw.transport_writer {
                                    RtpsWriterKind::Stateful(w) => {
                                        if w.on_acknack_submessage_received(
                                            ack_nack_submessage,
                                            message_receiver.source_guid_prefix(),
                                            self.transport.message_writer.as_ref(),
                                            &self.clock_handle,
                                        )
                                        .await
                                        .is_some()
                                        {
                                            if let Some(x) = dw.acknowledgement_notification.take()
                                            {
                                                x.send(());
                                            }

                                            if w.is_change_acknowledged(
                                                dw.last_change_sequence_number,
                                            ) {
                                                for n in dw
                                                    .wait_for_acknowledgments_notification
                                                    .drain(..)
                                                {
                                                    n.send(Ok(()));
                                                }
                                            }
                                        }
                                    }
                                    RtpsWriterKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::NackFrag(nack_frag_submessage) => {
                        for publisher in self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .chain(core::iter::once(
                                &mut self.domain_participant.builtin_publisher,
                            ))
                        {
                            for dw in &mut publisher.data_writer_list {
                                match &mut dw.transport_writer {
                                    RtpsWriterKind::Stateful(w) => {
                                        w.on_nack_frag_submessage_received(
                                            nack_frag_submessage,
                                            message_receiver.source_guid_prefix(),
                                            self.transport.message_writer.as_ref(),
                                        )
                                        .await
                                    }
                                    RtpsWriterKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    async fn handle_data_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        data_submessage: &DataSubmessage,
    ) {
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        let writer_guid = Guid::new(
                            message_receiver.source_guid_prefix(),
                            data_submessage.writer_id(),
                        );
                        let sequence_number = data_submessage.writer_sn();
                        let reliability = r.reliability();
                        if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                            match reliability {
                                ReliabilityKind::BestEffort => {
                                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                                    if sequence_number >= expected_seq_num {
                                        writer_proxy.received_change_set(sequence_number);
                                        if sequence_number > expected_seq_num {
                                            writer_proxy.lost_changes_update(sequence_number);
                                        }

                                        if let Ok(change) = CacheChange::try_from_data_submessage(
                                            data_submessage,
                                            message_receiver.source_guid_prefix(),
                                            message_receiver.source_timestamp(),
                                        ) {
                                            let subscriber_handle = subscriber.instance_handle;
                                            let reader_handle = dr.instance_handle;
                                            return self
                                                .add_cache_change(
                                                    change,
                                                    subscriber_handle,
                                                    reader_handle,
                                                )
                                                .await;
                                        }
                                    }
                                }
                                ReliabilityKind::Reliable => {
                                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                                    if sequence_number == expected_seq_num {
                                        writer_proxy.received_change_set(sequence_number);

                                        if let Ok(change) = CacheChange::try_from_data_submessage(
                                            data_submessage,
                                            message_receiver.source_guid_prefix(),
                                            message_receiver.source_timestamp(),
                                        ) {
                                            let subscriber_handle = subscriber.instance_handle;
                                            let reader_handle = dr.instance_handle;
                                            return self
                                                .add_cache_change(
                                                    change,
                                                    subscriber_handle,
                                                    reader_handle,
                                                )
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    RtpsReaderKind::Stateless(r) => {
                        if data_submessage.reader_id() == ENTITYID_UNKNOWN
                            || data_submessage.reader_id() == r.guid().entity_id()
                        {
                            if let Ok(change) = CacheChange::try_from_data_submessage(
                                data_submessage,
                                message_receiver.source_guid_prefix(),
                                message_receiver.source_timestamp(),
                            ) {
                                // Stateless reader behavior. We add the change if the data is correct. No error is printed
                                // because all readers would get changes marked with ENTITYID_UNKNOWN
                                let subscriber_handle = subscriber.instance_handle;
                                let reader_handle = dr.instance_handle;
                                return self
                                    .add_cache_change(change, subscriber_handle, reader_handle)
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn handle_gap_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        gap_submessage: &GapSubmessage,
    ) {
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        let writer_guid = Guid::new(
                            message_receiver.source_guid_prefix(),
                            gap_submessage.writer_id(),
                        );
                        if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                            for seq_num in
                                gap_submessage.gap_start()..gap_submessage.gap_list().base()
                            {
                                writer_proxy.irrelevant_change_set(seq_num)
                            }

                            for seq_num in gap_submessage.gap_list().set() {
                                writer_proxy.irrelevant_change_set(seq_num)
                            }
                        }
                    }
                    RtpsReaderKind::Stateless(_) => (),
                }
            }
        }
    }

    async fn handle_heartbeat_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        heartbeat_submessage: &HeartbeatSubmessage,
    ) {
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        let writer_guid = Guid::new(
                            message_receiver.source_guid_prefix(),
                            heartbeat_submessage.writer_id(),
                        );
                        let reader_guid = r.guid();
                        if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                            if writer_proxy.last_received_heartbeat_count()
                                < heartbeat_submessage.count()
                            {
                                writer_proxy.set_last_received_heartbeat_count(
                                    heartbeat_submessage.count(),
                                );
                                writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                                writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());

                                let must_send_acknacks = !heartbeat_submessage.final_flag()
                                    || (!heartbeat_submessage.liveliness_flag()
                                        && writer_proxy.missing_changes().count() > 0);
                                writer_proxy.set_must_send_acknacks(must_send_acknacks);

                                writer_proxy
                                    .write_message(
                                        &reader_guid,
                                        self.transport.message_writer.as_ref(),
                                    )
                                    .await;
                            }
                        }
                    }
                    RtpsReaderKind::Stateless(_) => (),
                }
            }
        }
    }

    async fn handle_data_frag_submessage(
        &mut self,
        message_receiver: &MessageReceiver<'_>,
        data_frag_submessage: &DataFragSubmessage,
    ) {
        for subscriber in self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_subscriber,
            ))
        {
            for dr in &mut subscriber.data_reader_list {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => {
                        let writer_guid = Guid::new(
                            message_receiver.source_guid_prefix(),
                            data_frag_submessage.writer_id(),
                        );
                        let sequence_number = data_frag_submessage.writer_sn();
                        let reliability = r.reliability();
                        if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                            match reliability {
                                ReliabilityKind::BestEffort => {
                                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                                    if sequence_number >= expected_seq_num {
                                        writer_proxy.push_data_frag(data_frag_submessage.clone());
                                    }
                                }
                                ReliabilityKind::Reliable => {
                                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                                    if sequence_number == expected_seq_num {
                                        writer_proxy.push_data_frag(data_frag_submessage.clone());
                                    }
                                }
                            }

                            if let Some(data_submessage) =
                                writer_proxy.reconstruct_data_from_frag(sequence_number)
                            {
                                writer_proxy.delete_data_fragments(data_submessage.writer_sn());

                                return self
                                    .handle_data_submessage(message_receiver, &data_submessage)
                                    .await;
                            }
                        };
                    }
                    RtpsReaderKind::Stateless(_) => (),
                }
            }
        }
    }

    pub async fn poke(&mut self) {
        for publisher in self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .chain(core::iter::once(
                &mut self.domain_participant.builtin_publisher,
            ))
        {
            for dw in &mut publisher.data_writer_list {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(writer) => {
                        writer
                            .write_message(
                                self.transport.message_writer.as_ref(),
                                &self.clock_handle,
                            )
                            .await
                    }
                    RtpsWriterKind::Stateless(_writer) => {}
                }
            }
        }
    }
}

#[tracing::instrument(skip(type_support))]
fn get_topic_kind(type_support: &dyn DynamicType) -> TopicKind {
    for index in 0..type_support.get_member_count() {
        if let Ok(m) = type_support.get_member_by_index(index) {
            if let Ok(d) = m.get_descriptor() {
                if d.is_key {
                    return TopicKind::WithKey;
                }
            }
        }
    }
    TopicKind::NoKey
}

#[tracing::instrument]
fn get_discovered_reader_incompatible_qos_policy_list(
    writer_qos: &DataWriterQos,
    discovered_reader_data: &SubscriptionBuiltinTopicData,
    publisher_qos: &PublisherQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();
    if &writer_qos.durability < discovered_reader_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if publisher_qos.presentation.access_scope < discovered_reader_data.presentation().access_scope
        || publisher_qos.presentation.coherent_access
            != discovered_reader_data.presentation().coherent_access
        || publisher_qos.presentation.ordered_access
            != discovered_reader_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &writer_qos.deadline > discovered_reader_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &writer_qos.latency_budget > discovered_reader_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &writer_qos.liveliness < discovered_reader_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if writer_qos.reliability.kind < discovered_reader_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &writer_qos.destination_order < discovered_reader_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if writer_qos.ownership.kind != discovered_reader_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = writer_qos
        .representation
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !(discovered_reader_data
        .representation()
        .value
        .contains(writer_offered_representation)
        || (writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && discovered_reader_data.representation().value.is_empty()))
    {
        incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID);
    }

    incompatible_qos_policy_list
}

#[tracing::instrument(skip(data_reader))]
fn get_discovered_writer_incompatible_qos_policy_list(
    data_reader: &DataReaderEntity,
    publication_builtin_topic_data: &PublicationBuiltinTopicData,
    subscriber_qos: &SubscriberQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();

    if subscriber_qos.presentation.access_scope
        > publication_builtin_topic_data.presentation().access_scope
        || subscriber_qos.presentation.coherent_access
            != publication_builtin_topic_data
                .presentation()
                .coherent_access
        || subscriber_qos.presentation.ordered_access
            != publication_builtin_topic_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &data_reader.qos.durability > publication_builtin_topic_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos.deadline < publication_builtin_topic_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &data_reader.qos.latency_budget < publication_builtin_topic_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &data_reader.qos.liveliness > publication_builtin_topic_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if data_reader.qos.reliability.kind > publication_builtin_topic_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos.destination_order > publication_builtin_topic_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if data_reader.qos.ownership.kind != publication_builtin_topic_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = publication_builtin_topic_data
        .representation()
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !data_reader
        .qos
        .representation
        .value
        .contains(writer_offered_representation)
    {
        // Empty list is interpreted as containing XCDR_DATA_REPRESENTATION
        if !(writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && data_reader.qos.representation.value.is_empty())
        {
            incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID)
        }
    }

    incompatible_qos_policy_list
}

fn is_discovered_topic_consistent(
    topic_qos: &TopicQos,
    topic_builtin_topic_data: &TopicBuiltinTopicData,
) -> bool {
    &topic_qos.topic_data == topic_builtin_topic_data.topic_data()
        && &topic_qos.durability == topic_builtin_topic_data.durability()
        && &topic_qos.deadline == topic_builtin_topic_data.deadline()
        && &topic_qos.latency_budget == topic_builtin_topic_data.latency_budget()
        && &topic_qos.liveliness == topic_builtin_topic_data.liveliness()
        && &topic_qos.reliability == topic_builtin_topic_data.reliability()
        && &topic_qos.destination_order == topic_builtin_topic_data.destination_order()
        && &topic_qos.history == topic_builtin_topic_data.history()
        && &topic_qos.resource_limits == topic_builtin_topic_data.resource_limits()
        && &topic_qos.transport_priority == topic_builtin_topic_data.transport_priority()
        && &topic_qos.lifespan == topic_builtin_topic_data.lifespan()
        && &topic_qos.ownership == topic_builtin_topic_data.ownership()
}

fn fnmatch_to_regex(pattern: &str) -> String {
    fn flush_literal(out: &mut String, lit: &mut String) {
        if !lit.is_empty() {
            out.push_str(&regex::escape(lit));
            lit.clear();
        }
    }

    let mut out = String::from("^");
    let mut literal = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // backslash escapes next char literally
            '\\' => {
                if let Some(next) = chars.next() {
                    literal.push(next);
                } else {
                    literal.push('\\');
                }
            }

            // glob wildcards
            '*' => {
                flush_literal(&mut out, &mut literal);
                out.push_str(".*");
            }
            '?' => {
                flush_literal(&mut out, &mut literal);
                out.push('.');
            }

            // character class
            '[' => {
                flush_literal(&mut out, &mut literal);

                let mut class = String::from("[");
                // handle fnmatch negation [!...] -> regex [^...]
                if let Some(&next) = chars.peek() {
                    if next == '!' {
                        chars.next();
                        class.push('^');
                    } else if next == '^' {
                        // treat ^ the same if user used it
                        chars.next();
                        class.push('^');
                    }
                }

                let mut closed = false;
                while let Some(ch) = chars.next() {
                    class.push(ch);
                    if ch == ']' {
                        closed = true;
                        break;
                    }
                    // preserve escaped chars inside class
                    if ch == '\\' {
                        if let Some(esc) = chars.next() {
                            class.push(esc);
                        }
                    }
                }

                if closed {
                    out.push_str(&class);
                } else {
                    // unclosed '[' — treat as literal
                    literal.push('[');
                    literal.push_str(&class[1..]); // append rest as literal
                }
            }

            '+' => {
                flush_literal(&mut out, &mut literal);
                out.push('+'); // regex plus (quantifier)
            }

            // default: accumulate literal characters (will be escaped when flushed)
            other => literal.push(other),
        }
    }

    flush_literal(&mut out, &mut literal);
    out.push('$');
    out
}

const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

struct DomainParticipantEntity {
    domain_id: DomainId,
    domain_tag: String,
    instance_handle: InstanceHandle,
    qos: DomainParticipantQos,
    builtin_subscriber: SubscriberEntity,
    builtin_publisher: PublisherEntity,
    user_defined_subscriber_list: Vec<SubscriberEntity>,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<PublisherEntity>,
    default_publisher_qos: PublisherQos,
    topic_description_list: Vec<TopicDescriptionKind>,
    default_topic_qos: TopicQos,
    discovered_participant_list: Vec<SpdpDiscoveredParticipantData>,
    discovered_topic_list: Vec<TopicBuiltinTopicData>,
    discovered_reader_list: Vec<DiscoveredReaderData>,
    discovered_writer_list: Vec<DiscoveredWriterData>,
    enabled: bool,
    ignored_participants: BTreeSet<InstanceHandle>,
    ignored_publications: BTreeSet<InstanceHandle>,
    ignored_subscriptions: BTreeSet<InstanceHandle>,
    _ignored_topic_list: BTreeSet<InstanceHandle>,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: Vec<StatusKind>,
}

impl DomainParticipantEntity {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: Vec<StatusKind>,
        instance_handle: InstanceHandle,
        builtin_publisher: PublisherEntity,
        builtin_subscriber: SubscriberEntity,
        topic_description_list: Vec<TopicDescriptionKind>,
        domain_tag: String,
    ) -> Self {
        Self {
            domain_id,
            instance_handle,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            default_subscriber_qos: SubscriberQos::const_default(),
            user_defined_publisher_list: Vec::new(),
            default_publisher_qos: PublisherQos::const_default(),
            topic_description_list,
            default_topic_qos: TopicQos::const_default(),
            discovered_participant_list: Vec::new(),
            discovered_topic_list: Vec::new(),
            discovered_reader_list: Vec::new(),
            discovered_writer_list: Vec::new(),
            enabled: false,
            ignored_participants: BTreeSet::new(),
            ignored_publications: BTreeSet::new(),
            ignored_subscriptions: BTreeSet::new(),
            _ignored_topic_list: BTreeSet::new(),
            listener_sender,
            listener_mask,
            domain_tag,
        }
    }

    fn builtin_subscriber(&self) -> &SubscriberEntity {
        &self.builtin_subscriber
    }

    fn add_discovered_topic(&mut self, topic_builtin_topic_data: TopicBuiltinTopicData) {
        match self
            .discovered_topic_list
            .iter_mut()
            .find(|t| t.key() == topic_builtin_topic_data.key())
        {
            Some(x) => *x = topic_builtin_topic_data,
            None => self.discovered_topic_list.push(topic_builtin_topic_data),
        }
    }

    fn remove_discovered_writer(&mut self, discovered_writer_handle: &InstanceHandle) {
        self.discovered_writer_list
            .retain(|x| &x.dds_publication_data.key().value != discovered_writer_handle.as_ref());
    }

    fn get_discovered_topic_data(
        &self,
        topic_handle: &InstanceHandle,
    ) -> Option<&TopicBuiltinTopicData> {
        self.discovered_topic_list
            .iter()
            .find(|x| &x.key().value == topic_handle.as_ref())
    }

    fn find_topic(&self, topic_name: &str) -> Option<&TopicBuiltinTopicData> {
        self.discovered_topic_list
            .iter()
            .find(|&discovered_topic_data| discovered_topic_data.name() == topic_name)
    }

    fn add_discovered_participant(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        match self.discovered_participant_list.iter_mut().find(|p| {
            p.dds_participant_data.key() == discovered_participant_data.dds_participant_data.key()
        }) {
            Some(x) => *x = discovered_participant_data,
            None => self
                .discovered_participant_list
                .push(discovered_participant_data),
        }
    }

    fn add_discovered_reader(&mut self, discovered_reader_data: DiscoveredReaderData) {
        match self.discovered_reader_list.iter_mut().find(|x| {
            x.dds_subscription_data.key() == discovered_reader_data.dds_subscription_data.key()
        }) {
            Some(x) => *x = discovered_reader_data,
            None => self.discovered_reader_list.push(discovered_reader_data),
        }
    }

    fn remove_discovered_reader(&mut self, discovered_reader_handle: &InstanceHandle) {
        self.discovered_reader_list
            .retain(|x| &x.dds_subscription_data.key().value != discovered_reader_handle.as_ref());
    }

    fn add_discovered_writer(&mut self, discovered_writer_data: DiscoveredWriterData) {
        match self.discovered_writer_list.iter_mut().find(|x| {
            x.dds_publication_data.key() == discovered_writer_data.dds_publication_data.key()
        }) {
            Some(x) => *x = discovered_writer_data,
            None => self.discovered_writer_list.push(discovered_writer_data),
        }
    }

    fn remove_subscriber(&mut self, handle: &InstanceHandle) -> Option<SubscriberEntity> {
        let i = self
            .user_defined_subscriber_list
            .iter()
            .position(|x| &x.instance_handle == handle)?;

        Some(self.user_defined_subscriber_list.remove(i))
    }

    fn remove_publisher(&mut self, handle: &InstanceHandle) -> Option<PublisherEntity> {
        let i = self
            .user_defined_publisher_list
            .iter()
            .position(|x| &x.instance_handle == handle)?;

        Some(self.user_defined_publisher_list.remove(i))
    }

    fn is_empty(&self) -> bool {
        let no_user_defined_topics = self
            .topic_description_list
            .iter()
            .filter(|t| !BUILT_IN_TOPIC_NAME_LIST.contains(&t.topic_name()))
            .count()
            == 0;

        self.user_defined_publisher_list.is_empty()
            && self.user_defined_subscriber_list.is_empty()
            && no_user_defined_topics
    }
}

struct ContentFilteredTopicEntity {
    topic_name: String,
    related_topic_name: String,
    filter_expression: String,
    expression_parameters: Vec<String>,
}

impl ContentFilteredTopicEntity {
    fn new(
        name: String,
        related_topic_name: String,
        filter_expression: String,
        expression_parameters: Vec<String>,
    ) -> Self {
        Self {
            topic_name: name,
            related_topic_name,
            filter_expression,
            expression_parameters,
        }
    }
}

struct SubscriberEntity {
    instance_handle: InstanceHandle,
    qos: SubscriberQos,
    data_reader_list: Vec<DataReaderEntity>,
    enabled: bool,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<DcpsStatusCondition>,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: Vec<StatusKind>,
}

impl SubscriberEntity {
    const fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        data_reader_list: Vec<DataReaderEntity>,
        status_condition: Actor<DcpsStatusCondition>,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            instance_handle,
            qos,
            data_reader_list,
            enabled: false,
            default_data_reader_qos: DataReaderQos::const_default(),
            status_condition,
            listener_sender,
            listener_mask,
        }
    }
}

enum TopicDescriptionKind {
    Topic(TopicEntity),
    ContentFilteredTopic(ContentFilteredTopicEntity),
}

impl TopicDescriptionKind {
    fn topic_name(&self) -> &str {
        match self {
            TopicDescriptionKind::Topic(t) => &t.topic_name,
            TopicDescriptionKind::ContentFilteredTopic(t) => &t.topic_name,
        }
    }
}

struct TopicEntity {
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    instance_handle: InstanceHandle,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: Actor<DcpsStatusCondition>,
    _listener_sender: Option<MpscSender<ListenerMail>>,
    _status_kind: Vec<StatusKind>,
    type_support: &'static dyn DynamicType,
}

impl TopicEntity {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        qos: TopicQos,
        type_name: String,
        topic_name: String,
        instance_handle: InstanceHandle,
        status_condition: Actor<DcpsStatusCondition>,
        listener_sender: Option<MpscSender<ListenerMail>>,
        status_kind: Vec<StatusKind>,
        type_support: &'static dyn DynamicType,
    ) -> Self {
        Self {
            qos,
            type_name,
            topic_name,
            instance_handle,
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::const_default(),
            status_condition,
            _listener_sender: listener_sender,
            _status_kind: status_kind,
            type_support,
        }
    }
}

struct PublisherEntity {
    qos: PublisherQos,
    instance_handle: InstanceHandle,
    data_writer_list: Vec<DataWriterEntity>,
    enabled: bool,
    default_datawriter_qos: DataWriterQos,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: Vec<StatusKind>,
}

impl PublisherEntity {
    const fn new(
        qos: PublisherQos,
        instance_handle: InstanceHandle,
        data_writer_list: Vec<DataWriterEntity>,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            qos,
            instance_handle,
            data_writer_list,
            enabled: false,
            default_datawriter_qos: DataWriterQos::const_default(),
            listener_sender,
            listener_mask,
        }
    }
}

enum RtpsWriterKind {
    Stateful(RtpsStatefulWriter),
    Stateless(RtpsStatelessWriter),
}

impl RtpsWriterKind {
    fn guid(&self) -> Guid {
        match self {
            RtpsWriterKind::Stateful(w) => w.guid(),
            RtpsWriterKind::Stateless(w) => w.guid(),
        }
    }

    async fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
    ) {
        match self {
            RtpsWriterKind::Stateful(w) => w.add_change(cache_change, message_writer, clock).await,
            RtpsWriterKind::Stateless(w) => w.add_change(cache_change, message_writer).await,
        }
    }

    async fn remove_change(&mut self, sequence_number: i64) {
        match self {
            RtpsWriterKind::Stateful(w) => w.remove_change(sequence_number),
            RtpsWriterKind::Stateless(w) => w.remove_change(sequence_number),
        }
    }
}

struct InstancePublicationTime {
    instance: InstanceHandle,
    last_write_time: Time,
}

struct InstanceSamples {
    instance: InstanceHandle,
    samples: VecDeque<i64>,
}

struct DataWriterEntity {
    instance_handle: InstanceHandle,
    transport_writer: RtpsWriterKind,
    topic_name: String,
    type_name: String,
    type_support: &'static dyn DynamicType,
    matched_subscription_list: Vec<SubscriptionBuiltinTopicData>,
    publication_matched_status: PublicationMatchedStatus,
    incompatible_subscription_list: Vec<InstanceHandle>,
    offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
    enabled: bool,
    status_condition: Actor<DcpsStatusCondition>,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: Vec<StatusKind>,
    max_seq_num: Option<i64>,
    last_change_sequence_number: i64,
    qos: DataWriterQos,
    registered_instance_list: Vec<InstanceHandle>,
    offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    instance_publication_time: Vec<InstancePublicationTime>,
    instance_samples: Vec<InstanceSamples>,
    /// Member used for notifying reliable writers which are waiting to send
    /// their samples without losing data
    acknowledgement_notification: Option<OneshotSender<()>>,
    /// Member used to notify the external user which called the
    /// wait_for_acknowledgments method
    wait_for_acknowledgments_notification: Vec<OneshotSender<DdsResult<()>>>,
}

impl DataWriterEntity {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        instance_handle: InstanceHandle,
        transport_writer: RtpsWriterKind,
        topic_name: String,
        type_name: String,
        type_support: &'static dyn DynamicType,
        status_condition: Actor<DcpsStatusCondition>,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: Vec<StatusKind>,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            instance_handle,
            transport_writer,
            topic_name,
            type_name,
            type_support,
            matched_subscription_list: Vec::new(),
            publication_matched_status: PublicationMatchedStatus::const_default(),
            incompatible_subscription_list: Vec::new(),
            offered_incompatible_qos_status: OfferedIncompatibleQosStatus::const_default(),
            enabled: false,
            status_condition,
            listener_sender,
            listener_mask,
            max_seq_num: None,
            last_change_sequence_number: 0,
            qos,
            registered_instance_list: Vec::new(),
            offered_deadline_missed_status: OfferedDeadlineMissedStatus::const_default(),
            instance_publication_time: Vec::new(),
            instance_samples: Vec::new(),
            acknowledgement_notification: None,
            wait_for_acknowledgments_notification: Vec::new(),
        }
    }

    async fn dispose_w_timestamp(
        &mut self,
        mut dynamic_data: DynamicData,
        timestamp: Time,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let has_key = {
            let mut has_key = false;
            for index in 0..self.type_support.get_member_count() {
                if self
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle =
            get_instance_handle_from_dynamic_data(self.type_support, dynamic_data.clone())?;
        if !self.registered_instance_list.contains(&instance_handle) {
            return Err(DdsError::BadParameter);
        }

        if let Some(i) = self
            .instance_publication_time
            .iter()
            .position(|x| x.instance == instance_handle)
        {
            self.instance_publication_time.remove(i);
        }

        dynamic_data.clear_nonkey_values(self.type_support)?;
        let serialized_key = serialize(self.type_support, &dynamic_data, &self.qos.representation)?;

        self.last_change_sequence_number += 1;
        let cache_change = CacheChange {
            kind: ChangeKind::NotAliveDisposed,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer
            .add_change(cache_change, message_writer, clock)
            .await;

        Ok(())
    }

    async fn unregister_w_timestamp(
        &mut self,
        mut dynamic_data: DynamicData,
        timestamp: Time,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let has_key = {
            let mut has_key = false;
            for index in 0..self.type_support.get_member_count() {
                if self
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle =
            get_instance_handle_from_dynamic_data(self.type_support, dynamic_data.clone())?;
        if !self.registered_instance_list.contains(&instance_handle) {
            return Err(DdsError::BadParameter);
        }

        if let Some(i) = self
            .instance_publication_time
            .iter()
            .position(|x| x.instance == instance_handle)
        {
            self.instance_publication_time.remove(i);
        }

        dynamic_data.clear_nonkey_values(self.type_support)?;
        let serialized_key = serialize(self.type_support, &dynamic_data, &self.qos.representation)?;

        self.last_change_sequence_number += 1;
        let kind = if self
            .qos
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            ChangeKind::NotAliveDisposedUnregistered
        } else {
            ChangeKind::NotAliveUnregistered
        };
        let cache_change = CacheChange {
            kind,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer
            .add_change(cache_change, message_writer, clock)
            .await;
        Ok(())
    }

    fn add_matched_subscription(
        &mut self,
        subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    ) {
        match self
            .matched_subscription_list
            .iter_mut()
            .find(|x| x.key() == subscription_builtin_topic_data.key())
        {
            Some(x) => *x = subscription_builtin_topic_data,
            None => self
                .matched_subscription_list
                .push(subscription_builtin_topic_data),
        };
        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change += 1;
        self.publication_matched_status.total_count += 1;
        self.publication_matched_status.total_count_change += 1;
    }

    fn remove_matched_subscription(&mut self, subscription_handle: &InstanceHandle) {
        let Some(i) = self
            .matched_subscription_list
            .iter()
            .position(|x| &x.key().value == subscription_handle.as_ref())
        else {
            return;
        };
        self.matched_subscription_list.remove(i);

        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change -= 1;
    }

    fn add_incompatible_subscription(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_subscription_list.contains(&handle) {
            self.offered_incompatible_qos_status.total_count += 1;
            self.offered_incompatible_qos_status.total_count_change += 1;
            self.offered_incompatible_qos_status.last_policy_id = incompatible_qos_policy_list[0];

            self.incompatible_subscription_list.push(handle);
            for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                if let Some(policy_count) = self
                    .offered_incompatible_qos_status
                    .policies
                    .iter_mut()
                    .find(|x| x.policy_id == incompatible_qos_policy)
                {
                    policy_count.count += 1;
                } else {
                    self.offered_incompatible_qos_status
                        .policies
                        .push(QosPolicyCount {
                            policy_id: incompatible_qos_policy,
                            count: 1,
                        })
                }
            }
        }
    }

    fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = self.offered_incompatible_qos_status.clone();
        self.offered_incompatible_qos_status.total_count_change = 0;
        status
    }

    fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        let status = self.publication_matched_status.clone();
        self.publication_matched_status.current_count_change = 0;
        self.publication_matched_status.total_count_change = 0;

        status
    }

    fn get_instance_write_time(&self, instance_handle: InstanceHandle) -> Option<Time> {
        self.instance_publication_time
            .iter()
            .find(|x| x.instance == instance_handle)
            .map(|x| x.last_write_time)
    }

    async fn get_offered_deadline_missed_status(&mut self) -> OfferedDeadlineMissedStatus {
        let status = self.offered_deadline_missed_status.clone();
        self.offered_deadline_missed_status.total_count_change = 0;
        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::OfferedDeadlineMissed,
            })
            .await;

        status
    }
}

type SampleList = Vec<(Option<DynamicData>, SampleInfo)>;

enum AddChangeResult {
    Added(InstanceHandle),
    NotAdded,
    Rejected(InstanceHandle, SampleRejectedStatusKind),
}

struct InstanceState {
    handle: InstanceHandle,
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
}

impl InstanceState {
    fn new(handle: InstanceHandle) -> Self {
        Self {
            handle,
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
        }
    }

    fn update_state(&mut self, change_kind: ChangeKind) {
        match self.instance_state {
            InstanceStateKind::Alive => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveDisposedUnregistered
                {
                    self.instance_state = InstanceStateKind::NotAliveDisposed;
                } else if change_kind == ChangeKind::NotAliveUnregistered {
                    self.instance_state = InstanceStateKind::NotAliveNoWriters;
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_disposed_generation_count += 1;
                }
            }
            InstanceStateKind::NotAliveNoWriters => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_no_writers_generation_count += 1;
                }
            }
        }

        match self.view_state {
            ViewStateKind::New => (),
            ViewStateKind::NotNew => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveUnregistered
                {
                    self.view_state = ViewStateKind::New;
                }
            }
        }
    }

    fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }

    fn handle(&self) -> InstanceHandle {
        self.handle
    }
}

#[derive(Debug)]
struct ReaderSample {
    kind: ChangeKind,
    writer_guid: [u8; 16],
    instance_handle: InstanceHandle,
    source_timestamp: Option<Time>,
    data_value: DynamicData,
    sample_state: SampleStateKind,
    disposed_generation_count: i32,
    no_writers_generation_count: i32,
    reception_timestamp: Time,
}

struct IndexedSample {
    index: usize,
    sample: (Option<DynamicData>, SampleInfo),
}

enum RtpsReaderKind {
    Stateful(RtpsStatefulReader),
    Stateless(RtpsStatelessReader),
}

impl RtpsReaderKind {
    fn guid(&self) -> Guid {
        match self {
            RtpsReaderKind::Stateful(r) => r.guid(),
            RtpsReaderKind::Stateless(r) => r.guid(),
        }
    }
}

struct InstanceOwnership {
    instance_handle: InstanceHandle,
    owner_handle: [u8; 16],
    last_received_time: Time,
}

struct DataReaderEntity {
    instance_handle: InstanceHandle,
    sample_list: Vec<ReaderSample>,
    qos: DataReaderQos,
    topic_name: String,
    type_support: &'static dyn DynamicType,
    requested_deadline_missed_status: RequestedDeadlineMissedStatus,
    requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    sample_rejected_status: SampleRejectedStatus,
    subscription_matched_status: SubscriptionMatchedStatus,
    matched_publication_list: Vec<PublicationBuiltinTopicData>,
    enabled: bool,
    data_available_status_changed_flag: bool,
    incompatible_writer_list: Vec<InstanceHandle>,
    status_condition: Actor<DcpsStatusCondition>,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: Vec<StatusKind>,
    instances: Vec<InstanceState>,
    instance_ownership: Vec<InstanceOwnership>,
    transport_reader: RtpsReaderKind,
}

impl DataReaderEntity {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        type_support: &'static dyn DynamicType,
        status_condition: Actor<DcpsStatusCondition>,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: Vec<StatusKind>,
        transport_reader: RtpsReaderKind,
    ) -> Self {
        Self {
            instance_handle,
            sample_list: Vec::new(),
            qos,
            topic_name,
            type_support,
            requested_deadline_missed_status: RequestedDeadlineMissedStatus::const_default(),
            requested_incompatible_qos_status: RequestedIncompatibleQosStatus::const_default(),
            sample_rejected_status: SampleRejectedStatus::const_default(),
            subscription_matched_status: SubscriptionMatchedStatus::const_default(),
            matched_publication_list: Vec::new(),
            enabled: false,
            data_available_status_changed_flag: false,
            incompatible_writer_list: Vec::new(),
            status_condition,
            listener_sender,
            listener_mask,
            instances: Vec::new(),
            instance_ownership: Vec::new(),
            transport_reader,
        }
    }

    fn create_indexed_sample_collection(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<IndexedSample>> {
        if let Some(h) = specific_instance_handle {
            if !self.instances.iter().any(|x| x.handle() == h) {
                return Err(DdsError::BadParameter);
            }
        };

        let mut indexed_samples = Vec::new();

        let mut instances_in_collection = Vec::<InstanceState>::new();
        for (index, cache_change) in self.sample_list.iter().enumerate() {
            if let Some(h) = specific_instance_handle {
                if cache_change.instance_handle != h {
                    continue;
                }
            };

            let Some(instance) = self
                .instances
                .iter()
                .find(|x| x.handle == cache_change.instance_handle)
            else {
                continue;
            };

            if !(sample_states.contains(&cache_change.sample_state)
                && view_states.contains(&instance.view_state)
                && instance_states.contains(&instance.instance_state))
            {
                continue;
            }

            if !instances_in_collection
                .iter()
                .any(|x| x.handle() == cache_change.instance_handle)
            {
                instances_in_collection.push(InstanceState::new(cache_change.instance_handle));
            }

            let instance_from_collection = instances_in_collection
                .iter_mut()
                .find(|x| x.handle() == cache_change.instance_handle)
                .expect("Instance must exist");
            instance_from_collection.update_state(cache_change.kind);
            let sample_state = cache_change.sample_state;
            let view_state = instance.view_state;
            let instance_state = instance.instance_state;

            let absolute_generation_rank = (instance.most_recent_disposed_generation_count
                + instance.most_recent_no_writers_generation_count)
                - (instance_from_collection.most_recent_disposed_generation_count
                    + instance_from_collection.most_recent_no_writers_generation_count);

            let (data, valid_data) = match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    (Some(cache_change.data_value.clone()), true)
                }
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => (None, false),
            };

            let sample_info = SampleInfo {
                sample_state,
                view_state,
                instance_state,
                disposed_generation_count: cache_change.disposed_generation_count,
                no_writers_generation_count: cache_change.no_writers_generation_count,
                sample_rank: 0,     // To be filled up after collection is created
                generation_rank: 0, // To be filled up after collection is created
                absolute_generation_rank,
                source_timestamp: cache_change.source_timestamp,
                instance_handle: cache_change.instance_handle,
                publication_handle: InstanceHandle::new(cache_change.writer_guid),
                valid_data,
            };

            let sample = (data, sample_info);

            indexed_samples.push(IndexedSample { index, sample });

            if indexed_samples.len() as i32 == max_samples {
                break;
            }
        }

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.iter().map(|x| x.handle()) {
            let most_recent_sample_absolute_generation_rank = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .map(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.absolute_generation_rank,
                )
                .next_back()
                .expect("Instance handle must exist on collection");

            let mut total_instance_samples_in_collection = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .count();

            for IndexedSample {
                sample: (_, sample_info),
                ..
            } in indexed_samples.iter_mut().filter(
                |IndexedSample {
                     sample: (_, sample_info),
                     ..
                 }| sample_info.instance_handle == handle,
            ) {
                sample_info.generation_rank = sample_info.absolute_generation_rank
                    - most_recent_sample_absolute_generation_rank;

                total_instance_samples_in_collection -= 1;
                sample_info.sample_rank = total_instance_samples_in_collection as i32;
            }

            self.instances
                .iter_mut()
                .find(|x| x.handle() == handle)
                .expect("Sample must exist")
                .mark_viewed()
        }

        if indexed_samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(indexed_samples)
        }
    }

    fn next_instance(&mut self, previous_handle: Option<InstanceHandle>) -> Option<InstanceHandle> {
        match previous_handle {
            Some(p) => self
                .instances
                .iter()
                .map(|x| x.handle())
                .filter(|&h| h > p)
                .min(),
            None => self.instances.iter().map(|x| x.handle()).min(),
        }
    }

    fn convert_cache_change_to_sample(
        &mut self,
        cache_change: CacheChange,
        reception_timestamp: Time,
    ) -> DdsResult<ReaderSample> {
        struct KeyHolder<'a> {
            descriptor: &'a crate::xtypes::dynamic_type::TypeDescriptor,
            member_list: Vec<&'a DynamicTypeMember>,
        }

        impl<'a> DynamicType for KeyHolder<'a> {
            fn get_descriptor(&self) -> &crate::xtypes::dynamic_type::TypeDescriptor {
                self.descriptor
            }

            fn get_name(&self) -> crate::xtypes::dynamic_type::ObjectName<'static> {
                self.descriptor.name
            }

            fn get_kind(&self) -> crate::xtypes::dynamic_type::TypeKind {
                self.descriptor.kind
            }

            fn get_member_by_name(
                &self,
                name: crate::xtypes::dynamic_type::ObjectName,
            ) -> Result<
                &crate::xtypes::dynamic_type::DynamicTypeMember,
                crate::xtypes::error::XTypesError,
            > {
                self.member_list
                    .iter()
                    .find(|x| x.get_name() == name)
                    .copied()
                    .ok_or(XTypesError::InvalidName)
            }

            fn get_member(
                &self,
                id: crate::xtypes::dynamic_type::MemberId,
            ) -> Result<
                &crate::xtypes::dynamic_type::DynamicTypeMember,
                crate::xtypes::error::XTypesError,
            > {
                self.member_list
                    .iter()
                    .find(|x| x.get_id() == id)
                    .copied()
                    .ok_or(XTypesError::InvalidId(id))
            }

            fn get_member_count(&self) -> u32 {
                self.member_list.len() as u32
            }

            fn get_member_by_index(
                &self,
                index: u32,
            ) -> Result<
                &crate::xtypes::dynamic_type::DynamicTypeMember,
                crate::xtypes::error::XTypesError,
            > {
                self.member_list
                    .get(index as usize)
                    .copied()
                    .ok_or(XTypesError::InvalidIndex(index))
            }
        }

        fn create_key_holder<'a>(foo_type: &'a dyn DynamicType) -> DdsResult<KeyHolder<'a>> {
            let key_holder_type_descriptor = foo_type.get_descriptor();
            let mut key_member_list = Vec::new();
            for member_index in 0..foo_type.get_member_count() {
                let member = foo_type.get_member_by_index(member_index)?;
                if member.get_descriptor()?.is_key {
                    key_member_list.push(member);
                }
            }
            Ok(KeyHolder {
                descriptor: key_holder_type_descriptor,
                member_list: key_member_list,
            })
        }

        let (data_value, instance_handle) = match cache_change.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                let data_value = CdrDeserializer::deserialize(
                    self.type_support,
                    cache_change.data_value.as_ref(),
                )?;
                let instance_handle =
                    get_instance_handle_from_dynamic_data(self.type_support, data_value.clone())?;
                (data_value, instance_handle)
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => match cache_change.instance_handle {
                Some(i) => {
                    let data_value = DynamicDataFactory::create_data();
                    let instance_handle = InstanceHandle::new(i);
                    (data_value, instance_handle)
                }
                None => {
                    let key_holder = create_key_holder(self.type_support)?;
                    let data_value = CdrDeserializer::deserialize(
                        &key_holder,
                        cache_change.data_value.as_ref(),
                    )?;
                    let instance_handle =
                        get_instance_handle_from_dynamic_data(&key_holder, data_value.clone())?;
                    (data_value, instance_handle)
                }
            },
        };

        // Update the state of the instance before creating since this has direct impact on
        // the information that is stored on the sample
        match cache_change.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == instance_handle)
                {
                    Some(x) => x.update_state(cache_change.kind),
                    None => {
                        let mut s = InstanceState::new(instance_handle);
                        s.update_state(cache_change.kind);
                        self.instances.push(s);
                    }
                }
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(cache_change.kind);
                        Ok(())
                    }
                    None => Err(DdsError::Error(
                        "Received message changing state of unknown instance".to_string(),
                    )),
                }
            }
        }?;
        let instance = self
            .instances
            .iter()
            .find(|x| x.handle() == instance_handle)
            .expect("Sample with handle must exist");
        Ok(ReaderSample {
            kind: cache_change.kind,
            writer_guid: cache_change.writer_guid.into(),
            instance_handle,
            source_timestamp: cache_change.source_timestamp.map(Into::into),
            data_value,
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: instance.most_recent_disposed_generation_count,
            no_writers_generation_count: instance.most_recent_no_writers_generation_count,
            reception_timestamp,
        })
    }

    fn add_reader_change(
        &mut self,
        cache_change: CacheChange,
        reception_timestamp: Time,
    ) -> DdsResult<AddChangeResult> {
        let sample = self.convert_cache_change_to_sample(cache_change, reception_timestamp)?;
        let change_instance_handle = sample.instance_handle;
        // data_reader exclusive access if the writer is not the allowed to write the sample do an early return
        if self.qos.ownership.kind == OwnershipQosPolicyKind::Exclusive {
            // Get the InstanceHandle of the data writer owning this instance
            if let Some(instance_owner) = self
                .instance_ownership
                .iter()
                .find(|x| x.instance_handle == sample.instance_handle)
            {
                let instance_writer = InstanceHandle::new(sample.writer_guid);
                let Some(sample_owner) = self
                    .matched_publication_list
                    .iter()
                    .find(|x| x.key().value == instance_owner.owner_handle.as_ref())
                else {
                    return Ok(AddChangeResult::NotAdded);
                };
                let Some(sample_writer) = self
                    .matched_publication_list
                    .iter()
                    .find(|x| &x.key().value == instance_writer.as_ref())
                else {
                    return Ok(AddChangeResult::NotAdded);
                };
                if instance_owner.owner_handle != sample.writer_guid
                    && sample_writer.ownership_strength().value
                        <= sample_owner.ownership_strength().value
                {
                    return Ok(AddChangeResult::NotAdded);
                }
            }

            match self
                .instance_ownership
                .iter_mut()
                .find(|x| x.instance_handle == sample.instance_handle)
            {
                Some(x) => {
                    x.owner_handle = sample.writer_guid;
                }
                None => self.instance_ownership.push(InstanceOwnership {
                    instance_handle: sample.instance_handle,
                    owner_handle: sample.writer_guid,
                    last_received_time: reception_timestamp,
                }),
            }
        }

        if matches!(
            sample.kind,
            ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered
        ) {
            if let Some(i) = self
                .instance_ownership
                .iter()
                .position(|x| x.instance_handle == sample.instance_handle)
            {
                self.instance_ownership.remove(i);
            }
        }

        let is_sample_of_interest_based_on_time = {
            let closest_timestamp_before_received_sample = self
                .sample_list
                .iter()
                .filter(|cc| cc.instance_handle == sample.instance_handle)
                .filter(|cc| cc.source_timestamp <= sample.source_timestamp)
                .map(|cc| cc.source_timestamp)
                .max();

            if let Some(Some(t)) = closest_timestamp_before_received_sample {
                if let Some(sample_source_time) = sample.source_timestamp {
                    let sample_separation = sample_source_time - t;
                    DurationKind::Finite(sample_separation)
                        >= self.qos.time_based_filter.minimum_separation
                } else {
                    true
                }
            } else {
                true
            }
        };

        if !is_sample_of_interest_based_on_time {
            return Ok(AddChangeResult::NotAdded);
        }

        let is_max_samples_limit_reached = {
            let total_samples = self
                .sample_list
                .iter()
                .filter(|cc| cc.kind == ChangeKind::Alive)
                .count();

            total_samples == self.qos.resource_limits.max_samples
        };
        let is_max_instances_limit_reached = {
            let mut instance_handle_list = Vec::new();
            for sample_handle in self.sample_list.iter().map(|x| x.instance_handle) {
                if !instance_handle_list.contains(&sample_handle) {
                    instance_handle_list.push(sample_handle);
                }
            }

            if instance_handle_list.contains(&sample.instance_handle) {
                false
            } else {
                instance_handle_list.len() == self.qos.resource_limits.max_instances
            }
        };
        let is_max_samples_per_instance_limit_reached = {
            let total_samples_of_instance = self
                .sample_list
                .iter()
                .filter(|cc| cc.instance_handle == sample.instance_handle)
                .count();

            total_samples_of_instance == self.qos.resource_limits.max_samples_per_instance
        };
        if is_max_samples_limit_reached {
            return Ok(AddChangeResult::Rejected(
                sample.instance_handle,
                SampleRejectedStatusKind::RejectedBySamplesLimit,
            ));
        } else if is_max_instances_limit_reached {
            return Ok(AddChangeResult::Rejected(
                sample.instance_handle,
                SampleRejectedStatusKind::RejectedByInstancesLimit,
            ));
        } else if is_max_samples_per_instance_limit_reached {
            return Ok(AddChangeResult::Rejected(
                sample.instance_handle,
                SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit,
            ));
        }
        let num_alive_samples_of_instance = self
            .sample_list
            .iter()
            .filter(|cc| {
                cc.instance_handle == sample.instance_handle && cc.kind == ChangeKind::Alive
            })
            .count() as u32;

        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if depth == num_alive_samples_of_instance {
                let index_sample_to_remove = self
                    .sample_list
                    .iter()
                    .position(|cc| {
                        cc.instance_handle == sample.instance_handle && cc.kind == ChangeKind::Alive
                    })
                    .expect("Samples must exist");
                self.sample_list.remove(index_sample_to_remove);
            }
        }

        match sample.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == sample.instance_handle)
                {
                    Some(x) => x.update_state(sample.kind),
                    None => {
                        let mut s = InstanceState::new(sample.instance_handle);
                        s.update_state(sample.kind);
                        self.instances.push(s);
                    }
                }
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == sample.instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(sample.kind);
                        Ok(())
                    }
                    None => Err(DdsError::Error(
                        "Received message changing state of unknown instance".to_string(),
                    )),
                }
            }
        }?;

        let sample_writer_guid = sample.writer_guid;
        tracing::debug!(cache_change = ?sample, "Adding change to data reader history cache");
        self.sample_list.push(sample);
        self.data_available_status_changed_flag = true;

        match self.qos.destination_order.kind {
            DestinationOrderQosPolicyKind::BySourceTimestamp => {
                self.sample_list.sort_by(|a, b| {
                    a.source_timestamp
                        .as_ref()
                        .expect("Missing source timestamp")
                        .cmp(
                            b.source_timestamp
                                .as_ref()
                                .expect("Missing source timestamp"),
                        )
                });
            }
            DestinationOrderQosPolicyKind::ByReceptionTimestamp => self
                .sample_list
                .sort_by(|a, b| a.reception_timestamp.cmp(&b.reception_timestamp)),
        }

        match self
            .instance_ownership
            .iter_mut()
            .find(|x| x.instance_handle == change_instance_handle)
        {
            Some(x) => {
                if x.last_received_time < reception_timestamp {
                    x.last_received_time = reception_timestamp;
                }
            }
            None => self.instance_ownership.push(InstanceOwnership {
                instance_handle: change_instance_handle,
                last_received_time: reception_timestamp,
                owner_handle: sample_writer_guid,
            }),
        }
        Ok(AddChangeResult::Added(change_instance_handle))
    }

    fn add_matched_publication(
        &mut self,
        publication_builtin_topic_data: PublicationBuiltinTopicData,
    ) {
        match self
            .matched_publication_list
            .iter_mut()
            .find(|x| x.key() == publication_builtin_topic_data.key())
        {
            Some(x) => *x = publication_builtin_topic_data,
            None => self
                .matched_publication_list
                .push(publication_builtin_topic_data),
        }
        self.subscription_matched_status.current_count +=
            self.matched_publication_list.len() as i32;
        self.subscription_matched_status.current_count_change += 1;
        self.subscription_matched_status.total_count += 1;
        self.subscription_matched_status.total_count_change += 1;
    }

    fn increment_requested_deadline_missed_status(&mut self, instance_handle: InstanceHandle) {
        self.requested_deadline_missed_status.total_count += 1;
        self.requested_deadline_missed_status.total_count_change += 1;
        self.requested_deadline_missed_status.last_instance_handle = instance_handle;
    }

    fn get_requested_deadline_missed_status(&mut self) -> RequestedDeadlineMissedStatus {
        let status = self.requested_deadline_missed_status.clone();
        self.requested_deadline_missed_status.total_count_change = 0;
        status
    }

    fn remove_instance_ownership(&mut self, instance_handle: &InstanceHandle) {
        if let Some(i) = self
            .instance_ownership
            .iter()
            .position(|x| &x.instance_handle == instance_handle)
        {
            self.instance_ownership.remove(i);
        }
    }

    fn add_requested_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_writer_list.contains(&handle) {
            self.incompatible_writer_list.push(handle);
            self.requested_incompatible_qos_status.total_count += 1;
            self.requested_incompatible_qos_status.total_count_change += 1;
            self.requested_incompatible_qos_status.last_policy_id = incompatible_qos_policy_list[0];
            for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                if let Some(policy_count) = self
                    .requested_incompatible_qos_status
                    .policies
                    .iter_mut()
                    .find(|x| x.policy_id == incompatible_qos_policy)
                {
                    policy_count.count += 1;
                } else {
                    self.requested_incompatible_qos_status
                        .policies
                        .push(QosPolicyCount {
                            policy_id: incompatible_qos_policy,
                            count: 1,
                        })
                }
            }
        }
    }

    fn get_requested_incompatible_qos_status(&mut self) -> RequestedIncompatibleQosStatus {
        let status = self.requested_incompatible_qos_status.clone();
        self.requested_incompatible_qos_status.total_count_change = 0;
        status
    }

    fn increment_sample_rejected_status(
        &mut self,
        sample_handle: InstanceHandle,
        sample_rejected_status_kind: SampleRejectedStatusKind,
    ) {
        self.sample_rejected_status.last_instance_handle = sample_handle;
        self.sample_rejected_status.last_reason = sample_rejected_status_kind;
        self.sample_rejected_status.total_count += 1;
        self.sample_rejected_status.total_count_change += 1;
    }

    fn get_sample_rejected_status(&mut self) -> SampleRejectedStatus {
        let status = self.sample_rejected_status.clone();
        self.sample_rejected_status.total_count_change = 0;

        status
    }

    fn get_subscription_matched_status(&mut self) -> SubscriptionMatchedStatus {
        let status = self.subscription_matched_status.clone();

        self.subscription_matched_status.total_count_change = 0;
        self.subscription_matched_status.current_count_change = 0;

        status
    }

    fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect()
    }

    fn get_instance_received_time(&self, instance_handle: &InstanceHandle) -> Option<Time> {
        self.instance_ownership
            .iter()
            .find(|x| &x.instance_handle == instance_handle)
            .map(|x| x.last_received_time)
    }

    async fn remove_matched_publication(&mut self, publication_handle: &InstanceHandle) {
        let Some(i) = self
            .matched_publication_list
            .iter()
            .position(|x| &x.key().value == publication_handle.as_ref())
        else {
            return;
        };
        self.matched_publication_list.remove(i);

        self.subscription_matched_status.current_count = self.matched_publication_list.len() as i32;
        self.subscription_matched_status.current_count_change -= 1;
        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                state: StatusKind::SubscriptionMatched,
            })
            .await;
    }

    async fn read(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            })
            .await;

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )?;

        let change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        for index in change_index_list {
            self.sample_list[index].sample_state = SampleStateKind::Read;
        }

        Ok(samples)
    }

    async fn take(
        &mut self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            &sample_states,
            &view_states,
            &instance_states,
            specific_instance_handle,
        )?;

        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            })
            .await;

        let mut change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        while let Some(index) = change_index_list.pop() {
            self.sample_list.remove(index);
        }

        Ok(samples)
    }

    async fn take_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => {
                self.take(
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    Some(next_handle),
                )
                .await
            }
            None => Err(DdsError::NoData),
        }
    }

    async fn read_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => {
                self.read(
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    Some(next_handle),
                )
                .await
            }
            None => Err(DdsError::NoData),
        }
    }
}

fn serialize(
    dynamic_type: &dyn DynamicType,
    dynamic_data: &DynamicData,
    representation: &DataRepresentationQosPolicy,
) -> DdsResult<Vec<u8>> {
    Ok(
        if representation.value.is_empty() || representation.value[0] == XCDR_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                Cdr1BeSerializer::serialize(dynamic_type, dynamic_data)?
            } else {
                Cdr1LeSerializer::serialize(dynamic_type, dynamic_data)?
            }
        } else if representation.value[0] == XCDR2_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                Cdr2BeSerializer::serialize(dynamic_type, dynamic_data)?
            } else {
                Cdr2LeSerializer::serialize(dynamic_type, dynamic_data)?
            }
        } else if representation.value[0] == BUILT_IN_DATA_REPRESENTATION {
            RtpsPlCdrSerializer::serialize(dynamic_type, dynamic_data)?
        } else {
            panic!("Invalid data representation")
        },
    )
}
