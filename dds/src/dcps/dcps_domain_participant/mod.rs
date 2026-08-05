mod builtin_publisher;
mod builtin_subscriber;
mod communication_methods;
mod discovery_methods;
mod participant_methods;
mod publisher_methods;
mod reader_methods;
mod status_condition_methods;
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
        channels::{mpsc::MpscSender, oneshot::OneshotSender},
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
            type_lookup::{TypeLookupReply, TypeLookupRequest},
        },
        listeners::domain_participant_listener::ListenerMail,
        status_condition::DcpsStatusCondition,
        status_mask::StatusMask,
        xtypes_glue::key_and_instance_handle::{
            KeyHolderData, get_instance_handle_from_key_holder_data,
        },
    },
    dds_async::domain_participant_factory::DcpsSender,
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos,
            TopicQos,
        },
        qos_policy::{
            DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy,
            DestinationOrderQosPolicyKind, DurabilityQosPolicy, DurabilityQosPolicyKind,
            HistoryQosPolicy, HistoryQosPolicyKind, LatencyBudgetQosPolicy, Length,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipQosPolicyKind,
            OwnershipStrengthQosPolicy, QosPolicyId, ReaderDataLifecycleQosPolicy,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, ResourceLimitsQosPolicy,
            TimeBasedFilterQosPolicy, TransportPriorityQosPolicy,
            TypeConsistencyEnforcementQosPolicy, UserDataQosPolicy, WriterDataLifecycleQosPolicy,
            XCDR_DATA_REPRESENTATION, XCDR2_DATA_REPRESENTATION,
        },
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleRejectedStatus, SampleRejectedStatusKind,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, TIME_INVALID_NSEC, TIME_INVALID_SEC, Time},
    },
    rtps::{
        stateful_reader::RtpsStatefulReader, stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader, stateless_writer::RtpsStatelessWriter,
    },
    runtime::{DdsRuntime, Timer},
    transport::{
        interface::{RtpsTransportParticipant, WriteMessage},
        types::{
            BUILT_IN_READER_GROUP, BUILT_IN_READER_NO_KEY, BUILT_IN_READER_WITH_KEY,
            BUILT_IN_TOPIC, BUILT_IN_WRITER_GROUP, BUILT_IN_WRITER_NO_KEY,
            BUILT_IN_WRITER_WITH_KEY, CacheChange, ChangeKind, ENTITYID_PARTICIPANT, EntityId,
            Guid, GuidPrefix, Locator, TopicKind, USER_DEFINED_TOPIC,
        },
    },
    xtypes::{
        dynamic_type::{DynamicData, DynamicType},
        serializer::{serialize_cdr1_be, serialize_cdr1_le, serialize_cdr2_be, serialize_cdr2_le},
        type_object::{TypeInformation, TypeObject},
        type_support::{Type, TypeSupport},
    },
};
use alloc::{
    boxed::Box,
    collections::{BTreeSet, VecDeque},
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use builtin_publisher::BuiltinPublisher;
use builtin_subscriber::BuiltinSubscriber;
use core::{
    future::{Future, poll_fn},
    pin::{Pin, pin},
    task::Poll,
};

const ENTITYID_SPDP_TOPIC: EntityId = EntityId::new([0, 0, 0], BUILT_IN_TOPIC);
const ENTITYID_SEDP_TOPICS_TOPIC: EntityId = EntityId::new([0, 0, 1], BUILT_IN_TOPIC);
const ENTITYID_SEDP_PUBLICATIONS_TOPIC: EntityId = EntityId::new([0, 0, 2], BUILT_IN_TOPIC);
const ENTITYID_SEDP_SUBSCRIPTIONS_TOPIC: EntityId = EntityId::new([0, 0, 3], BUILT_IN_TOPIC);
const ENTITYID_TL_SVC_REQ_TOPIC: EntityId = EntityId::new([0, 0, 4], BUILT_IN_TOPIC);
const ENTITYID_TL_SVC_RPL_TOPIC: EntityId = EntityId::new([0, 0, 5], BUILT_IN_TOPIC);

const ENTITYID_BUILTIN_SUBSCRIBER: EntityId = EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP);
pub(crate) const ENTITYID_BUILTIN_PUBLISHER: EntityId =
    EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP);

pub(crate) const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub(crate) const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_READER_WITH_KEY);

pub(crate) const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_READER_WITH_KEY);

pub(crate) const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_WRITER_WITH_KEY);

const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_READER_WITH_KEY);

// XTypes Table 61 – Built-in Endpoints added by the XTYPES specification

pub(crate) const ENTITYID_TL_SVC_REQ_WRITER: EntityId =
    EntityId::new([0x00, 0x03, 0x00], BUILT_IN_WRITER_NO_KEY);

const ENTITYID_TL_SVC_REQ_READER: EntityId =
    EntityId::new([0x00, 0x03, 0x00], BUILT_IN_READER_NO_KEY);

pub(crate) const ENTITYID_TL_SVC_REPLY_WRITER: EntityId =
    EntityId::new([0x00, 0x03, 0x01], BUILT_IN_WRITER_NO_KEY);

const ENTITYID_TL_SVC_REPLY_READER: EntityId =
    EntityId::new([0x00, 0x03, 0x01], BUILT_IN_READER_NO_KEY);

pub(crate) const TYPE_LOOKUP_REQUEST_TOPIC_NAME: &str = "TypeLookupRequest";
pub(crate) const TYPE_LOOKUP_REPLY_TOPIC_NAME: &str = "TypeLookupReply";

const SPDP_READER_QOS: DataReaderQos = DataReaderQos {
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
    deadline: DeadlineQosPolicy::const_default(),
    latency_budget: LatencyBudgetQosPolicy::const_default(),
    liveliness: LivelinessQosPolicy::const_default(),
    destination_order: DestinationOrderQosPolicy::const_default(),
    resource_limits: ResourceLimitsQosPolicy::const_default(),
    user_data: UserDataQosPolicy::const_default(),
    ownership: OwnershipQosPolicy::const_default(),
    time_based_filter: TimeBasedFilterQosPolicy::const_default(),
    reader_data_lifecycle: ReaderDataLifecycleQosPolicy::const_default(),
    representation: DataRepresentationQosPolicy::const_default(),
    type_consistency: TypeConsistencyEnforcementQosPolicy::const_default(),
};

const SEDP_DATA_READER_QOS: DataReaderQos = DataReaderQos {
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
    deadline: DeadlineQosPolicy::const_default(),
    latency_budget: LatencyBudgetQosPolicy::const_default(),
    liveliness: LivelinessQosPolicy::const_default(),
    destination_order: DestinationOrderQosPolicy::const_default(),
    resource_limits: ResourceLimitsQosPolicy::const_default(),
    user_data: UserDataQosPolicy::const_default(),
    ownership: OwnershipQosPolicy::const_default(),
    time_based_filter: TimeBasedFilterQosPolicy::const_default(),
    reader_data_lifecycle: ReaderDataLifecycleQosPolicy::const_default(),
    representation: DataRepresentationQosPolicy::const_default(),
    type_consistency: TypeConsistencyEnforcementQosPolicy::const_default(),
};

// DDS RPC default QoS as specified in DDS-RPC standard 7.10.2 Default QoS
const TYPE_LOOKUP_READER_QOS: DataReaderQos = DataReaderQos {
    durability: DurabilityQosPolicy {
        kind: DurabilityQosPolicyKind::Volatile,
    },
    history: HistoryQosPolicy {
        kind: HistoryQosPolicyKind::KeepAll,
    },
    reliability: ReliabilityQosPolicy {
        kind: ReliabilityQosPolicyKind::Reliable,
        max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
    },
    deadline: DeadlineQosPolicy::const_default(),
    latency_budget: LatencyBudgetQosPolicy::const_default(),
    liveliness: LivelinessQosPolicy::const_default(),
    destination_order: DestinationOrderQosPolicy::const_default(),
    resource_limits: ResourceLimitsQosPolicy::const_default(),
    user_data: UserDataQosPolicy::const_default(),
    ownership: OwnershipQosPolicy::const_default(),
    time_based_filter: TimeBasedFilterQosPolicy::const_default(),
    reader_data_lifecycle: ReaderDataLifecycleQosPolicy::const_default(),
    representation: DataRepresentationQosPolicy::const_default(),
    type_consistency: TypeConsistencyEnforcementQosPolicy::const_default(),
};

pub(crate) const TYPE_LOOKUP_WRITER_QOS: DataWriterQos = DataWriterQos {
    durability: DurabilityQosPolicy {
        kind: DurabilityQosPolicyKind::Volatile,
    },
    history: HistoryQosPolicy {
        kind: HistoryQosPolicyKind::KeepAll,
    },
    reliability: ReliabilityQosPolicy {
        kind: ReliabilityQosPolicyKind::Reliable,
        max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
    },
    deadline: DeadlineQosPolicy::const_default(),
    latency_budget: LatencyBudgetQosPolicy::const_default(),
    liveliness: LivelinessQosPolicy::const_default(),
    destination_order: DestinationOrderQosPolicy::const_default(),
    resource_limits: ResourceLimitsQosPolicy::const_default(),
    user_data: UserDataQosPolicy::const_default(),
    ownership: OwnershipQosPolicy::const_default(),
    ownership_strength: OwnershipStrengthQosPolicy::const_default(),
    lifespan: LifespanQosPolicy::const_default(),
    transport_priority: TransportPriorityQosPolicy::const_default(),
    writer_data_lifecycle: WriterDataLifecycleQosPolicy::const_default(),
    representation: DataRepresentationQosPolicy::const_default(),
};

struct DiscoveredParticipantInfo {
    dds_participant_data: ParticipantBuiltinTopicData,
    guid_prefix: GuidPrefix,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    lease_duration: Duration,
    last_communication_timestamp: Time,
}

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

#[derive(Debug, Clone, TypeSupport)]
pub struct BuiltInKeyHolder {
    #[dust_dds(key)]
    pub(crate) key: BuiltInTopicKey,
}

pub struct DcpsDomainParticipant {
    transport: RtpsTransportParticipant,

    reader_counter: u16,
    writer_counter: u16,
    publisher_counter: u8,
    subscriber_counter: u8,
    domain_participant: DomainParticipantEntity,
    dcps_sender: DcpsSender,
}

impl DcpsDomainParticipant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: DomainId,
        domain_tag: String,
        guid_prefix: GuidPrefix,
        domain_participant_qos: DomainParticipantQos,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
        transport: RtpsTransportParticipant,
        dcps_sender: DcpsSender,
    ) -> Self {
        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        let participant_handle = InstanceHandle::new(guid.into());

        const NUMBER_BUILTIN_ENTITIES: usize = 6;
        let mut topic_list = Vec::with_capacity(NUMBER_BUILTIN_ENTITIES);

        topic_list.push(TopicEntity::new(
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            String::from(DCPS_PARTICIPANT),
            InstanceHandle::new(Guid::new(guid_prefix, ENTITYID_SPDP_TOPIC).into()),
            DcpsStatusCondition::default(),
            None,
            StatusMask::default(),
            ParticipantBuiltinTopicData::TYPE,
        ));

        topic_list.push(TopicEntity::new(
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            String::from(DCPS_TOPIC),
            InstanceHandle::new(Guid::new(guid_prefix, ENTITYID_SEDP_TOPICS_TOPIC).into()),
            DcpsStatusCondition::default(),
            None,
            StatusMask::default(),
            TopicBuiltinTopicData::TYPE,
        ));

        topic_list.push(TopicEntity::new(
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            String::from(DCPS_PUBLICATION),
            InstanceHandle::new(Guid::new(guid_prefix, ENTITYID_SEDP_PUBLICATIONS_TOPIC).into()),
            DcpsStatusCondition::default(),
            None,
            StatusMask::default(),
            PublicationBuiltinTopicData::TYPE,
        ));

        topic_list.push(TopicEntity::new(
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            String::from(DCPS_SUBSCRIPTION),
            InstanceHandle::new(Guid::new(guid_prefix, ENTITYID_SEDP_SUBSCRIPTIONS_TOPIC).into()),
            DcpsStatusCondition::default(),
            None,
            StatusMask::default(),
            SubscriptionBuiltinTopicData::TYPE,
        ));

        topic_list.push(TopicEntity::new(
            TopicQos::default(),
            "TypeLookup_Request".to_string(),
            String::from(TYPE_LOOKUP_REQUEST_TOPIC_NAME),
            InstanceHandle::new(Guid::new(guid_prefix, ENTITYID_TL_SVC_REQ_TOPIC).into()),
            DcpsStatusCondition::default(),
            None,
            StatusMask::default(),
            TypeLookupRequest::TYPE,
        ));

        topic_list.push(TopicEntity::new(
            TopicQos::default(),
            "TypeLookup_Reply".to_string(),
            String::from(TYPE_LOOKUP_REPLY_TOPIC_NAME),
            InstanceHandle::new(Guid::new(guid_prefix, ENTITYID_TL_SVC_RPL_TOPIC).into()),
            DcpsStatusCondition::default(),
            None,
            StatusMask::default(),
            TypeLookupReply::TYPE,
        ));

        let builtin_subscriber = BuiltinSubscriber::new(guid_prefix);
        let builtin_publisher = BuiltinPublisher::new(guid_prefix, &transport);

        let domain_participant = DomainParticipantEntity::new(
            domain_id,
            domain_participant_qos,
            listener_sender,
            listener_mask,
            participant_handle,
            builtin_publisher,
            builtin_subscriber,
            topic_list,
            domain_tag,
        );

        Self {
            transport,
            reader_counter: 0,
            writer_counter: 0,
            publisher_counter: 0,
            subscriber_counter: 0,
            domain_participant,
            dcps_sender,
        }
    }

    pub fn domain_id(&self) -> DomainId {
        self.domain_participant.domain_id
    }

    pub fn time_until_stale_participant(&self, now: Time) -> Option<Duration> {
        self.domain_participant
            .discovered_participant_list
            .iter()
            .map(|dp| dp.lease_duration - (now - dp.last_communication_timestamp))
            .min()
    }

    pub fn time_until_missed_reader_deadline(&self, now: Time) -> Option<Duration> {
        self.domain_participant
            .user_defined_subscriber_list
            .iter()
            .flat_map(|subscriber| subscriber.data_reader_list.iter())
            .filter_map(|data_reader| {
                if let DurationKind::Finite(deadline) = data_reader.qos.deadline.period {
                    data_reader
                        .instance_ownership
                        .iter()
                        .map(|instance| deadline - (now - instance.last_received_time))
                        .min()
                } else {
                    None
                }
            })
            .min()
    }

    pub fn time_until_missed_writer_deadline(&self, now: Time) -> Option<Duration> {
        self.domain_participant
            .user_defined_publisher_list
            .iter()
            .flat_map(|publisher| publisher.data_writer_list.iter())
            .filter_map(|data_writer| {
                if let DurationKind::Finite(deadline) = data_writer.qos.deadline.period {
                    data_writer
                        .registered_instance_info
                        .iter()
                        .filter_map(|instance| instance.last_write_time)
                        .map(|last_write_time| deadline - (now - last_write_time))
                        .min()
                } else {
                    None
                }
            })
            .min()
    }

    pub fn time_until_stale_writer_sample(&self, now: Time) -> Option<Duration> {
        self.domain_participant
            .user_defined_publisher_list
            .iter()
            .flat_map(|publisher| publisher.data_writer_list.iter())
            .filter_map(|data_writer| {
                if let DurationKind::Finite(lifespan) = data_writer.qos.lifespan.duration {
                    data_writer
                        .transport_writer
                        .changes()
                        .iter()
                        .filter_map(|cc| cc.source_timestamp)
                        .map(|source_timestamp| Time::from(source_timestamp) + lifespan - now)
                        .min()
                } else {
                    None
                }
            })
            .min()
    }

    pub fn get_instance_handle(&self) -> &InstanceHandle {
        &self.domain_participant.instance_handle
    }
}

const BUILT_IN_TOPIC_NAME_LIST: [&str; 6] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
    TYPE_LOOKUP_REQUEST_TOPIC_NAME,
    TYPE_LOOKUP_REPLY_TOPIC_NAME,
];

struct FindTopicNotification {
    topic_name: String,
    deadline: Time,
    type_support: DynamicType<'static>,
    reply_sender: OneshotSender<DdsResult<(InstanceHandle, String)>>,
}

struct DomainParticipantEntity {
    domain_id: DomainId,
    domain_tag: String,
    topic_counter: u16,
    instance_handle: InstanceHandle,
    qos: DomainParticipantQos,
    builtin_subscriber: BuiltinSubscriber,
    builtin_publisher: BuiltinPublisher,
    user_defined_subscriber_list: Vec<UserDefinedSubscriber>,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<PublisherEntity>,
    default_publisher_qos: PublisherQos,
    locally_created_topic_list: Vec<TopicEntity>,
    content_filtered_topic_list: Vec<ContentFilteredTopicEntity>,
    default_topic_qos: TopicQos,
    discovered_participant_list: Vec<DiscoveredParticipantInfo>,
    discovered_topic_list: Vec<TopicBuiltinTopicData>,
    discovered_reader_list: Vec<DiscoveredReaderData>,
    discovered_writer_list: Vec<DiscoveredWriterData>,
    enabled: bool,
    ignored_participants: BTreeSet<InstanceHandle>,
    ignored_publications: BTreeSet<InstanceHandle>,
    ignored_subscriptions: BTreeSet<InstanceHandle>,
    _ignored_topic_list: BTreeSet<InstanceHandle>,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: StatusMask,
    find_topic_sender_list: Vec<FindTopicNotification>,
}

impl DomainParticipantEntity {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
        instance_handle: InstanceHandle,
        builtin_publisher: BuiltinPublisher,
        builtin_subscriber: BuiltinSubscriber,
        locally_created_topic_list: Vec<TopicEntity>,
        domain_tag: String,
    ) -> Self {
        Self {
            domain_id,
            instance_handle,
            topic_counter: 0,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            default_subscriber_qos: SubscriberQos::const_default(),
            user_defined_publisher_list: Vec::new(),
            default_publisher_qos: PublisherQos::const_default(),
            locally_created_topic_list,
            content_filtered_topic_list: Vec::new(),
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
            find_topic_sender_list: Vec::new(),
        }
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

    fn find_topic(
        &mut self,
        topic_name: &str,
        type_support: DynamicType<'static>,
    ) -> Option<(InstanceHandle, String)> {
        if let Some(topic) = self
            .locally_created_topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
        {
            Some((topic.instance_handle, topic.type_name.clone()))
        } else if let Some(discovered_topic_data) = self
            .discovered_topic_list
            .iter()
            .find(|&discovered_topic_data| discovered_topic_data.name() == topic_name)
        {
            let qos = TopicQos {
                topic_data: discovered_topic_data.topic_data().clone(),
                durability: discovered_topic_data.durability().clone(),
                deadline: discovered_topic_data.deadline().clone(),
                latency_budget: discovered_topic_data.latency_budget().clone(),
                liveliness: discovered_topic_data.liveliness().clone(),
                reliability: discovered_topic_data.reliability().clone(),
                destination_order: discovered_topic_data.destination_order().clone(),
                history: discovered_topic_data.history().clone(),
                resource_limits: discovered_topic_data.resource_limits().clone(),
                transport_priority: discovered_topic_data.transport_priority().clone(),
                lifespan: discovered_topic_data.lifespan().clone(),
                ownership: discovered_topic_data.ownership().clone(),
                representation: discovered_topic_data.representation().clone(),
            };
            let type_name = discovered_topic_data.type_name.clone();
            let topic_handle = InstanceHandle::new([
                self.instance_handle[0],
                self.instance_handle[1],
                self.instance_handle[2],
                self.instance_handle[3],
                self.instance_handle[4],
                self.instance_handle[5],
                self.instance_handle[6],
                self.instance_handle[7],
                self.instance_handle[8],
                self.instance_handle[9],
                self.instance_handle[10],
                self.instance_handle[11],
                0,
                self.topic_counter.to_ne_bytes()[0],
                self.topic_counter.to_ne_bytes()[1],
                USER_DEFINED_TOPIC,
            ]);
            self.topic_counter += 1;
            let status_condition = DcpsStatusCondition::default();
            let mut topic = TopicEntity::new(
                qos,
                type_name.clone().into(),
                String::from(topic_name),
                topic_handle,
                status_condition,
                None,
                StatusMask::default(),
                type_support,
            );
            topic.enabled = true;

            match self
                .locally_created_topic_list
                .iter_mut()
                .find(|x| x.topic_name == topic.topic_name)
            {
                Some(x) => *x = topic,
                None => self.locally_created_topic_list.push(topic),
            }

            Some((topic_handle, type_name.into()))
        } else {
            None
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

    fn remove_subscriber(&mut self, handle: &InstanceHandle) -> Option<UserDefinedSubscriber> {
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
            .locally_created_topic_list
            .iter()
            .filter(|t| !BUILT_IN_TOPIC_NAME_LIST.contains(&t.topic_name.as_str()))
            .count()
            == 0;

        self.user_defined_publisher_list.is_empty()
            && self.user_defined_subscriber_list.is_empty()
            && self.content_filtered_topic_list.is_empty()
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

pub(crate) struct SubscriberEntity {
    instance_handle: InstanceHandle,
    qos: SubscriberQos,
    enabled: bool,
}

impl SubscriberEntity {
    pub(crate) fn new(instance_handle: InstanceHandle, qos: SubscriberQos) -> Self {
        Self {
            instance_handle,
            qos,
            enabled: false,
        }
    }
}

pub(crate) struct UserDefinedSubscriber {
    pub(crate) subscriber_entity: SubscriberEntity,
    pub(crate) default_data_reader_qos: DataReaderQos,
    pub(crate) status_condition: DcpsStatusCondition,
    pub(crate) listener_sender: Option<MpscSender<ListenerMail>>,
    pub(crate) listener_mask: StatusMask,
    pub(crate) data_reader_list: Vec<UserDefinedDataReader>,
}

impl UserDefinedSubscriber {
    pub(crate) fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
    ) -> Self {
        Self {
            subscriber_entity: SubscriberEntity::new(instance_handle, qos),
            default_data_reader_qos: DataReaderQos::const_default(),
            status_condition: DcpsStatusCondition::default(),
            listener_sender,
            listener_mask,
            data_reader_list: Vec::new(),
        }
    }
}

impl core::ops::Deref for UserDefinedSubscriber {
    type Target = SubscriberEntity;

    fn deref(&self) -> &Self::Target {
        &self.subscriber_entity
    }
}

impl core::ops::DerefMut for UserDefinedSubscriber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.subscriber_entity
    }
}

#[allow(clippy::large_enum_variant)]
pub enum DiscoveredTypeRepresentationState {
    Requested,
    Discovered(TypeObject),
}

struct TopicEntity {
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    instance_handle: InstanceHandle,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: DcpsStatusCondition,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: StatusMask,
    type_support: DynamicType<'static>,
    type_information: TypeInformation,
    discovered_type_representation: Vec<(TypeInformation, DiscoveredTypeRepresentationState)>,
}

impl TopicEntity {
    #[allow(clippy::too_many_arguments)]
    fn new(
        qos: TopicQos,
        type_name: String,
        topic_name: String,
        instance_handle: InstanceHandle,
        status_condition: DcpsStatusCondition,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
        type_support: DynamicType<'static>,
    ) -> Self {
        Self {
            qos,
            type_name,
            topic_name,
            instance_handle,
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::const_default(),
            status_condition,
            listener_sender,
            listener_mask,
            type_support,
            type_information: TypeInformation::from(type_support),
            discovered_type_representation: Vec::new(),
        }
    }
}

pub(crate) fn get_topic_type_support<'a>(
    topic_name: &str,
    content_filtered_topic_list: &[ContentFilteredTopicEntity],
    locally_created_topic_list: &'a [TopicEntity],
) -> Option<&'a DynamicType<'static>> {
    let resolved_topic_name = if let Some(cf_topic) = content_filtered_topic_list
        .iter()
        .find(|t| t.topic_name == topic_name)
    {
        cf_topic.related_topic_name.as_str()
    } else {
        topic_name
    };

    match resolved_topic_name {
        DCPS_PARTICIPANT => Some(&ParticipantBuiltinTopicData::TYPE),
        DCPS_TOPIC => Some(&TopicBuiltinTopicData::TYPE),
        DCPS_PUBLICATION => Some(&PublicationBuiltinTopicData::TYPE),
        DCPS_SUBSCRIPTION => Some(&SubscriptionBuiltinTopicData::TYPE),
        TYPE_LOOKUP_REQUEST_TOPIC_NAME => Some(&TypeLookupRequest::TYPE),
        TYPE_LOOKUP_REPLY_TOPIC_NAME => Some(&TypeLookupReply::TYPE),
        _ => locally_created_topic_list
            .iter()
            .find(|t| t.topic_name == resolved_topic_name)
            .map(|t| &t.type_support),
    }
}

struct PublisherEntity {
    qos: PublisherQos,
    instance_handle: InstanceHandle,
    data_writer_list: Vec<UserDefinedDataWriter>,
    enabled: bool,
    default_datawriter_qos: DataWriterQos,
    listener_sender: Option<MpscSender<ListenerMail>>,
    listener_mask: StatusMask,
}

impl PublisherEntity {
    const fn new(
        qos: PublisherQos,
        instance_handle: InstanceHandle,
        data_writer_list: Vec<UserDefinedDataWriter>,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
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

pub trait RtpsWriter {
    fn guid(&self) -> Guid;
    fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        runtime: &impl DdsRuntime,
    );
    fn remove_change(&mut self, sequence_number: i64);
    fn changes(&self) -> &[CacheChange];
    fn changes_mut(&mut self) -> &mut Vec<CacheChange>;
}

impl RtpsWriter for RtpsStatefulWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        runtime: &impl DdsRuntime,
    ) {
        self.add_change(cache_change, message_writer, &runtime.clock())
    }

    fn remove_change(&mut self, sequence_number: i64) {
        self.remove_change(sequence_number)
    }

    fn changes(&self) -> &[CacheChange] {
        self.changes()
    }

    fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        self.changes_mut()
    }
}

impl RtpsWriter for RtpsStatelessWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        _runtime: &impl DdsRuntime,
    ) {
        self.add_change(cache_change, message_writer)
    }

    fn remove_change(&mut self, sequence_number: i64) {
        self.remove_change(sequence_number)
    }

    fn changes(&self) -> &[CacheChange] {
        self.changes()
    }

    fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        self.changes_mut()
    }
}

pub trait RtpsReader {
    fn guid(&self) -> Guid;
    fn changes_mut(&mut self) -> &mut Vec<CacheChange>;
}

impl RtpsReader for RtpsStatefulReader {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        self.changes_mut()
    }
}

impl RtpsReader for RtpsStatelessReader {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        self.changes_mut()
    }
}

struct RegisteredInstanceInfo {
    instance_handle: InstanceHandle,
    last_write_time: Option<Time>,
    samples: VecDeque<i64>,
}

#[derive(Default)]
pub(crate) struct IncompatibleSubscriptions {
    incompatible_subscription_list: Vec<InstanceHandle>,
    offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
}

pub(crate) struct DataWriterEntity<T> {
    instance_handle: InstanceHandle,
    transport_writer: T,
    topic_name: String,
    enabled: bool,
    last_change_sequence_number: i64,
    qos: DataWriterQos,
    registered_instance_info: Vec<RegisteredInstanceInfo>,
}

impl<T: RtpsWriter> DataWriterEntity<T> {
    pub(crate) fn new(
        instance_handle: InstanceHandle,
        transport_writer: T,
        topic_name: String,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            instance_handle,
            transport_writer,
            topic_name,
            enabled: false,
            last_change_sequence_number: 0,
            qos,
            registered_instance_info: Vec::new(),
        }
    }

    fn write_w_timestamp(
        &mut self,
        sample_instance_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        sample_timestamp: Time,
        now: Time,
        message_writer: &(impl WriteMessage + ?Sized),
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        if !self
            .registered_instance_info
            .iter()
            .any(|x| x.instance_handle == sample_instance_handle)
        {
            if self.registered_instance_info.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_info.push(RegisteredInstanceInfo {
                    instance_handle: sample_instance_handle,
                    last_write_time: None,
                    samples: VecDeque::new(),
                });
            } else {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            self.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match self.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth)
                    if depth as i32 <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = self
                        .registered_instance_info
                        .iter()
                        .find(|x| x.instance_handle == sample_instance_handle)
                    {
                        // Only Alive changes count towards the resource limits
                        if s.samples.len() >= max_samples_per_instance as usize {
                            return Err(DdsError::OutOfResources);
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = self.qos.resource_limits.max_samples {
            let total_samples = self
                .registered_instance_info
                .iter()
                .fold(0, |acc, x| acc + x.samples.len());

            if total_samples >= max_samples as usize {
                return Err(DdsError::OutOfResources);
            }
        }

        self.last_change_sequence_number += 1;
        let change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(sample_timestamp.into()),
            instance_handle: Some(sample_instance_handle.into()),
            data_value: serialized_data.into(),
        };

        let instance_info = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == sample_instance_handle)
            .expect("Instance info must exist");

        match &mut instance_info.last_write_time {
            Some(last_write_time) => {
                if *last_write_time < sample_timestamp {
                    *last_write_time = sample_timestamp;
                }
            }
            None => {
                instance_info.last_write_time = Some(sample_timestamp);
            }
        }

        instance_info.samples.push_back(change.sequence_number);

        if let DurationKind::Finite(lifespan_duration) = self.qos.lifespan.duration {
            let duration_until_expired = sample_timestamp - now + lifespan_duration;
            if duration_until_expired <= Duration::new(0, 0) {
                return Ok(());
            }
        }

        self.transport_writer
            .add_change(change, message_writer, runtime);

        Ok(())
    }

    fn dispose_w_timestamp(
        &mut self,
        dynamic_data: &DynamicData<'static>,
        type_support: &DynamicType<'static>,
        timestamp: Time,
        message_writer: &(impl WriteMessage + ?Sized),
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let mut member_list = Vec::new();
        let key_holder_data = KeyHolderData::from_dynamic_data(dynamic_data, &mut member_list)?;

        if TopicKind::from(type_support) == TopicKind::NoKey {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle = get_instance_handle_from_key_holder_data(&key_holder_data)?;

        let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        else {
            return Err(DdsError::BadParameter);
        };

        instance_info.last_write_time = None;

        let serialized_key =
            serialize(key_holder_data.as_dynamic_data(), &self.qos.representation)?;

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
            .add_change(cache_change, message_writer, runtime);

        Ok(())
    }

    fn register_w_timestamp(
        &mut self,
        dynamic_data: &DynamicData<'static>,
        type_support: &DynamicType<'static>,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let mut member_list = Vec::new();
        let key_holder_data = KeyHolderData::from_dynamic_data(dynamic_data, &mut member_list)?;

        if TopicKind::from(type_support) == TopicKind::NoKey {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle = get_instance_handle_from_key_holder_data(&key_holder_data)?;

        if let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        {
            instance_info.last_write_time = Some(timestamp);
        } else if self.registered_instance_info.len() < self.qos.resource_limits.max_instances {
            self.registered_instance_info.push(RegisteredInstanceInfo {
                instance_handle,
                last_write_time: Some(timestamp),
                samples: VecDeque::new(),
            });
        } else {
            return Err(DdsError::OutOfResources);
        }

        Ok(Some(instance_handle))
    }

    fn unregister_w_timestamp(
        &mut self,
        dynamic_data: &DynamicData<'static>,
        type_support: &DynamicType<'static>,
        timestamp: Time,
        message_writer: &(impl WriteMessage + ?Sized),
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let mut member_list = Vec::new();
        let key_holder_data = KeyHolderData::from_dynamic_data(dynamic_data, &mut member_list)?;

        if TopicKind::from(type_support) == TopicKind::NoKey {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle = get_instance_handle_from_key_holder_data(&key_holder_data)?;
        let Some(instance_info) = self
            .registered_instance_info
            .iter_mut()
            .find(|x| x.instance_handle == instance_handle)
        else {
            return Err(DdsError::BadParameter);
        };

        instance_info.last_write_time = None;

        let serialized_key =
            serialize(key_holder_data.as_dynamic_data(), &self.qos.representation)?;

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
            .add_change(cache_change, message_writer, runtime);
        Ok(())
    }
}

pub(crate) struct UserDefinedDataWriter {
    pub(crate) rtps_writer: DataWriterEntity<RtpsStatefulWriter>,
    pub(crate) listener_sender: Option<MpscSender<ListenerMail>>,
    pub(crate) listener_mask: StatusMask,
    pub(crate) status_condition: DcpsStatusCondition,
    pub(crate) matched_subscription_list: Vec<SubscriptionBuiltinTopicData>,
    pub(crate) publication_matched_status: PublicationMatchedStatus,
    pub(crate) incompatible_subscriptions: IncompatibleSubscriptions,
    pub(crate) offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    /// Member used for notifying reliable writers which are waiting to send
    /// their samples without losing data
    pub(crate) acknowledgement_notification: Option<OneshotSender<()>>,
    /// Member used to notify the external user which called the
    /// wait_for_acknowledgments method
    pub(crate) wait_for_acknowledgments_notification: Vec<OneshotSender<DdsResult<()>>>,
}

impl core::ops::Deref for UserDefinedDataWriter {
    type Target = DataWriterEntity<RtpsStatefulWriter>;
    fn deref(&self) -> &Self::Target {
        &self.rtps_writer
    }
}

impl core::ops::DerefMut for UserDefinedDataWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rtps_writer
    }
}

impl UserDefinedDataWriter {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        instance_handle: InstanceHandle,
        transport_writer: RtpsStatefulWriter,
        topic_name: String,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            rtps_writer: DataWriterEntity::new(instance_handle, transport_writer, topic_name, qos),
            listener_sender,
            listener_mask,
            status_condition: DcpsStatusCondition::default(),
            matched_subscription_list: Vec::new(),
            publication_matched_status: PublicationMatchedStatus::const_default(),
            incompatible_subscriptions: IncompatibleSubscriptions::default(),
            offered_deadline_missed_status: OfferedDeadlineMissedStatus::const_default(),
            acknowledgement_notification: None,
            wait_for_acknowledgments_notification: Vec::new(),
        }
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

    fn get_offered_deadline_missed_status(&mut self) -> OfferedDeadlineMissedStatus {
        let status = self.offered_deadline_missed_status.clone();
        self.offered_deadline_missed_status.total_count_change = 0;
        self.status_condition
            .remove_communication_state(StatusKind::OfferedDeadlineMissed);

        status
    }
}

type SampleList = Vec<(Arc<[u8]>, SampleInfo)>;

enum AddChangeResult {
    Added,
    NotAdded,
    Rejected(InstanceHandle, SampleRejectedStatusKind),
}

struct InstanceState {
    handle: InstanceHandle,
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
    last_received_time_stamp: Time,
}

impl InstanceState {
    fn new(handle: InstanceHandle) -> Self {
        Self {
            handle,
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
            last_received_time_stamp: Time::new(TIME_INVALID_SEC, TIME_INVALID_NSEC),
        }
    }

    fn update_state(&mut self, change_kind: ChangeKind, now: Option<Time>) {
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
        if let Some(t) = now {
            self.last_received_time_stamp = t;
        }
    }

    fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }

    fn handle(&self) -> &InstanceHandle {
        &self.handle
    }

    fn last_received_time_stamp(&self) -> Time {
        self.last_received_time_stamp
    }
}

#[derive(Debug)]
struct ReaderSample {
    kind: ChangeKind,
    writer_guid: [u8; 16],
    instance_handle: InstanceHandle,
    source_timestamp: Option<Time>,
    data_value: Arc<[u8]>,
    sample_state: SampleStateKind,
    disposed_generation_count: i32,
    no_writers_generation_count: i32,
}

struct InstanceOwnership {
    instance_handle: InstanceHandle,
    owner_handle: [u8; 16],
    last_received_time: Time,
}

pub(crate) struct DataReaderEntity<T> {
    pub(crate) instance_handle: InstanceHandle,
    pub(crate) sample_list: Vec<ReaderSample>,
    pub(crate) qos: DataReaderQos,
    pub(crate) topic_name: String,
    pub(crate) matched_publication_list: Vec<PublicationBuiltinTopicData>,
    pub(crate) enabled: bool,
    pub(crate) instances: Vec<InstanceState>,
    pub(crate) instance_ownership: Vec<InstanceOwnership>,
    pub(crate) transport_reader: T,
}

impl<T> DataReaderEntity<T> {
    fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        transport_reader: T,
    ) -> Self {
        Self {
            instance_handle,
            sample_list: Vec::new(),
            qos,
            topic_name,
            matched_publication_list: Vec::new(),
            enabled: false,
            instances: Vec::new(),
            instance_ownership: Vec::new(),
            transport_reader,
        }
    }

    fn create_sample_collection(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
        take: bool,
    ) -> DdsResult<SampleList> {
        if let Some(h) = specific_instance_handle {
            if !self.instances.iter().any(|x| x.handle() == h) {
                return Err(DdsError::BadParameter);
            }
        };

        let mut samples = Vec::new();

        let mut instances_in_collection = Vec::<InstanceState>::new();
        self.sample_list.retain_mut(|cache_change| {
            if samples.len() as i32 == max_samples {
                return true;
            }

            if let Some(h) = specific_instance_handle {
                if &cache_change.instance_handle != h {
                    return true;
                }
            };

            let Some(instance) = self
                .instances
                .iter()
                .find(|x| x.handle == cache_change.instance_handle)
            else {
                return true;
            };

            if !(sample_states.contains(&cache_change.sample_state)
                && view_states.contains(&instance.view_state)
                && instance_states.contains(&instance.instance_state))
            {
                return true;
            }

            if !instances_in_collection
                .iter()
                .any(|x| x.handle() == &cache_change.instance_handle)
            {
                instances_in_collection.push(InstanceState::new(cache_change.instance_handle));
            }

            let instance_from_collection = instances_in_collection
                .iter_mut()
                .find(|x| x.handle() == &cache_change.instance_handle)
                .expect("Instance must exist");
            instance_from_collection.update_state(cache_change.kind, None);
            let sample_state = cache_change.sample_state;
            let view_state = instance.view_state;
            let instance_state = instance.instance_state;

            let absolute_generation_rank = (instance.most_recent_disposed_generation_count
                + instance.most_recent_no_writers_generation_count)
                - (instance_from_collection.most_recent_disposed_generation_count
                    + instance_from_collection.most_recent_no_writers_generation_count);

            let (data, valid_data) = match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    (cache_change.data_value.clone(), true)
                }
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => {
                    (cache_change.data_value.clone(), false)
                }
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

            samples.push((data, sample_info));

            if take {
                false
            } else {
                cache_change.sample_state = SampleStateKind::Read;
                true
            }
        });

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.iter().map(|x| x.handle()) {
            let most_recent_sample_absolute_generation_rank = samples
                .iter()
                .filter(|(_, sample_info)| &sample_info.instance_handle == handle)
                .map(|(_, sample_info)| sample_info.absolute_generation_rank)
                .next_back()
                .expect("Instance handle must exist on collection");

            let mut total_instance_samples_in_collection = samples
                .iter()
                .filter(|(_, sample_info)| &sample_info.instance_handle == handle)
                .count();

            for (_, sample_info) in samples
                .iter_mut()
                .filter(|(_, sample_info)| &sample_info.instance_handle == handle)
            {
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

        if samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(samples)
        }
    }

    fn next_instance(
        &mut self,
        previous_handle: &Option<InstanceHandle>,
    ) -> Option<InstanceHandle> {
        match previous_handle {
            Some(p) => self
                .instances
                .iter()
                .map(|x| x.handle())
                .filter(|&h| h > p)
                .min()
                .cloned(),
            None => self.instances.iter().map(|x| x.handle()).min().cloned(),
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_reader_change(
        &mut self,
        writer_guid: Guid,
        data_value: Arc<[u8]>,
        change_kind: ChangeKind,
        change_instance_handle: [u8; 16],
        change_source_timestamp: Option<Time>,
        reception_timestamp: Time,
    ) -> DdsResult<AddChangeResult> {
        let instance_handle = InstanceHandle::new(change_instance_handle);
        // Update the state of the instance before creating since this has direct impact on
        // the information that is stored on the sample
        match change_kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == &instance_handle)
                {
                    Some(x) => x.update_state(change_kind, Some(reception_timestamp)),
                    None => {
                        let mut s = InstanceState::new(instance_handle);
                        s.update_state(change_kind, Some(reception_timestamp));
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
                    .find(|x| x.handle() == &instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(change_kind, Some(reception_timestamp));
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
            .find(|x| x.handle() == &instance_handle)
            .expect("Sample with handle must exist");
        let sample = ReaderSample {
            kind: change_kind,
            writer_guid: writer_guid.into(),
            instance_handle,
            source_timestamp: change_source_timestamp,
            data_value,
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: instance.most_recent_disposed_generation_count,
            no_writers_generation_count: instance.most_recent_no_writers_generation_count,
        };

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
                    .find(|x| x.handle() == &sample.instance_handle)
                {
                    Some(x) => x.update_state(sample.kind, Some(reception_timestamp)),
                    None => {
                        let mut s = InstanceState::new(sample.instance_handle);
                        s.update_state(sample.kind, Some(reception_timestamp));
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
                    .find(|x| x.handle() == &sample.instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(sample.kind, Some(reception_timestamp));
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

        match self.qos.destination_order.kind {
            DestinationOrderQosPolicyKind::BySourceTimestamp => {
                // Insert the element at the place where the first source timestamp is bigger than the currently received one
                let insert_position = self
                    .sample_list
                    .iter()
                    .position(|x| x.source_timestamp > sample.source_timestamp)
                    .unwrap_or(0);
                self.sample_list.insert(insert_position, sample);
            }
            DestinationOrderQosPolicyKind::ByReceptionTimestamp => self.sample_list.push(sample),
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
        Ok(AddChangeResult::Added)
    }

    fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect()
    }

    fn read(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.create_sample_collection(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
            false,
        )
    }

    fn take(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.create_sample_collection(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
            true,
        )
    }

    fn take_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: &Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => self.take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                &Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }

    fn read_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: &Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => self.read(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                &Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }
}

pub(crate) struct BuiltinDataReader<T> {
    pub(crate) rtps_reader: DataReaderEntity<T>,
}

impl<T> core::ops::Deref for BuiltinDataReader<T> {
    type Target = DataReaderEntity<T>;
    fn deref(&self) -> &Self::Target {
        &self.rtps_reader
    }
}

impl<T> core::ops::DerefMut for BuiltinDataReader<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rtps_reader
    }
}

impl<T> BuiltinDataReader<T> {
    pub(crate) fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        transport_reader: T,
    ) -> Self {
        Self {
            rtps_reader: DataReaderEntity::new(instance_handle, qos, topic_name, transport_reader),
        }
    }
}

pub(crate) struct UserDefinedDataReader {
    pub(crate) rtps_reader: DataReaderEntity<RtpsStatefulReader>,
    pub(crate) listener_sender: Option<MpscSender<ListenerMail>>,
    pub(crate) listener_mask: StatusMask,
    pub(crate) status_condition: DcpsStatusCondition,
    pub(crate) requested_deadline_missed_status: RequestedDeadlineMissedStatus,
    pub(crate) requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    pub(crate) sample_rejected_status: SampleRejectedStatus,
    pub(crate) subscription_matched_status: SubscriptionMatchedStatus,
    pub(crate) incompatible_writer_list: Vec<InstanceHandle>,
}

impl core::ops::Deref for UserDefinedDataReader {
    type Target = DataReaderEntity<RtpsStatefulReader>;
    fn deref(&self) -> &Self::Target {
        &self.rtps_reader
    }
}

impl core::ops::DerefMut for UserDefinedDataReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rtps_reader
    }
}

impl UserDefinedDataReader {
    pub(crate) fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: StatusMask,
        transport_reader: RtpsStatefulReader,
    ) -> Self {
        Self {
            rtps_reader: DataReaderEntity::new(instance_handle, qos, topic_name, transport_reader),
            listener_sender,
            listener_mask,
            status_condition: DcpsStatusCondition::default(),
            requested_deadline_missed_status: RequestedDeadlineMissedStatus::const_default(),
            requested_incompatible_qos_status: RequestedIncompatibleQosStatus::const_default(),
            sample_rejected_status: SampleRejectedStatus::const_default(),
            subscription_matched_status: SubscriptionMatchedStatus::const_default(),
            incompatible_writer_list: Vec::new(),
        }
    }

    pub(crate) fn add_matched_publication(
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
        self.subscription_matched_status.current_count = self.matched_publication_list.len() as i32;
        self.subscription_matched_status.current_count_change += 1;
        self.subscription_matched_status.total_count += 1;
        self.subscription_matched_status.total_count_change += 1;
    }

    pub(crate) fn remove_matched_publication(&mut self, publication_handle: &InstanceHandle) {
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
            .add_communication_state(StatusKind::SubscriptionMatched);
    }

    pub(crate) fn add_requested_incompatible_qos(
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

    pub(crate) fn get_requested_incompatible_qos_status(
        &mut self,
    ) -> RequestedIncompatibleQosStatus {
        let status = self.requested_incompatible_qos_status.clone();
        self.requested_incompatible_qos_status.total_count_change = 0;
        status
    }

    pub(crate) fn increment_sample_rejected_status(
        &mut self,
        sample_handle: InstanceHandle,
        sample_rejected_status_kind: SampleRejectedStatusKind,
    ) {
        self.sample_rejected_status.last_instance_handle = sample_handle;
        self.sample_rejected_status.last_reason = sample_rejected_status_kind;
        self.sample_rejected_status.total_count += 1;
        self.sample_rejected_status.total_count_change += 1;
    }

    pub(crate) fn get_sample_rejected_status(&mut self) -> SampleRejectedStatus {
        let status = self.sample_rejected_status.clone();
        self.sample_rejected_status.total_count_change = 0;
        status
    }

    pub(crate) fn get_subscription_matched_status(&mut self) -> SubscriptionMatchedStatus {
        let status = self.subscription_matched_status.clone();
        self.subscription_matched_status.total_count_change = 0;
        self.subscription_matched_status.current_count_change = 0;
        status
    }

    pub(crate) fn read(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        self.status_condition
            .remove_communication_state(StatusKind::DataAvailable);
        self.rtps_reader.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub(crate) fn take(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        self.status_condition
            .remove_communication_state(StatusKind::DataAvailable);
        self.rtps_reader.take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub(crate) fn take_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: &Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => self.take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                &Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }

    pub(crate) fn read_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: &Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => self.read(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                &Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }
}

fn serialize<'a>(
    dynamic_data: &DynamicData<'a>,
    representation: &DataRepresentationQosPolicy,
) -> DdsResult<Vec<u8>> {
    Ok(
        if representation.value.is_empty() || representation.value[0] == XCDR_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                serialize_cdr1_be(dynamic_data)
            } else {
                serialize_cdr1_le(dynamic_data)
            }
        } else if representation.value[0] == XCDR2_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                serialize_cdr2_be(dynamic_data)
            } else {
                serialize_cdr2_le(dynamic_data)
            }
        } else {
            panic!("Invalid data representation")
        }?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_until_stale_writer_sample_calculation() {
        let source_timestamp = crate::transport::types::Time::new(100, 0);
        let lifespan = Duration::new(10, 0);
        let now = Time::new(105, 0);

        let remaining = Time::from(source_timestamp) + lifespan - now;
        assert_eq!(remaining, Duration::new(5, 0));
    }
}
