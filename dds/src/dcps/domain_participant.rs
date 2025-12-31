use super::domain_participant_mail::{
    DcpsDomainParticipantMail, EventServiceMail, MessageServiceMail,
};
use crate::{
    builtin_topics::{
        BuiltInTopicKey, DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        actor::{Actor, ActorAddress},
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
        domain_participant_mail::WriterServiceMail,
        listeners::{
            data_reader_listener::DcpsDataReaderListener,
            data_writer_listener::DcpsDataWriterListener,
            domain_participant_listener::{DcpsDomainParticipantListener, ListenerMail},
            publisher_listener::DcpsPublisherListener,
            subscriber_listener::DcpsSubscriberListener,
            topic_listener::DcpsTopicListener,
        },
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
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        qos_policy::{
            BUILT_IN_DATA_REPRESENTATION, DATA_REPRESENTATION_QOS_POLICY_ID,
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            DataRepresentationQosPolicy, DestinationOrderQosPolicyKind, DurabilityQosPolicy,
            DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, Length, LifespanQosPolicy,
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
        history_cache::HistoryCache,
        message_receiver::MessageReceiver,
        stateful_reader::RtpsStatefulReader,
        stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader,
        stateless_writer::RtpsStatelessWriter,
        types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    },
    rtps_messages::overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
    runtime::{Clock, DdsRuntime, Either, Spawner, Timer, select_future},
    transport::{
        self,
        interface::{RtpsTransportParticipant, WriteMessage},
        types::{
            BUILT_IN_READER_GROUP, BUILT_IN_READER_NO_KEY, BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC,
            BUILT_IN_WRITER_GROUP, BUILT_IN_WRITER_NO_KEY, BUILT_IN_WRITER_WITH_KEY, CacheChange,
            ChangeKind, DurabilityKind,
            ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN, EntityId, Guid, GuidPrefix, ReliabilityKind,
            SequenceNumber, TopicKind, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY,
            USER_DEFINED_READER_WITH_KEY, USER_DEFINED_TOPIC, USER_DEFINED_WRITER_GROUP,
            USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY,
        },
    },
    xtypes::{
        binding::XTypesBinding,
        deserializer::CdrDeserializer,
        dynamic_type::{DynamicData, DynamicDataFactory, DynamicType},
        serializer::{
            Cdr1BeSerializer, Cdr1LeSerializer, Cdr2BeSerializer, Cdr2LeSerializer,
            RtpsPlCdrSerializer,
        },
    },
};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet, VecDeque},
    format,
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

// TypeLookup service EntityIds - XTypes 1.3 Table 61
#[cfg(feature = "type_lookup")]
const ENTITYID_TL_SVC_REQ_WRITER: EntityId =
    EntityId::new([0x00, 0x03, 0x00], BUILT_IN_WRITER_NO_KEY);

#[cfg(feature = "type_lookup")]
const ENTITYID_TL_SVC_REQ_READER: EntityId =
    EntityId::new([0x00, 0x03, 0x00], BUILT_IN_READER_NO_KEY);

#[cfg(feature = "type_lookup")]
const ENTITYID_TL_SVC_REPLY_WRITER: EntityId =
    EntityId::new([0x00, 0x03, 0x01], BUILT_IN_WRITER_NO_KEY);

#[cfg(feature = "type_lookup")]
const ENTITYID_TL_SVC_REPLY_READER: EntityId =
    EntityId::new([0x00, 0x03, 0x01], BUILT_IN_READER_NO_KEY);

struct DcpsParticipantReaderHistoryCache {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsParticipantReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinParticipantsDetectorCacheChange {
                        cache_change,
                        participant_address: self.participant_address.clone(),
                    },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

struct DcpsTopicsReaderHistoryCache {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsTopicsReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinTopicsDetectorCacheChange { cache_change },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

struct DcpsSubscriptionsReaderHistoryCache {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsSubscriptionsReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinSubscriptionsDetectorCacheChange {
                        cache_change,
                        participant_address: self.participant_address.clone(),
                    },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

struct DcpsPublicationsReaderHistoryCache {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsPublicationsReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinPublicationsDetectorCacheChange {
                        cache_change,
                        participant_address: self.participant_address.clone(),
                    },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

#[cfg(feature = "type_lookup")]
struct TypeLookupRequestReaderHistoryCache {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
}

#[cfg(feature = "type_lookup")]
impl HistoryCache for TypeLookupRequestReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddTypeLookupRequestCacheChange {
                        cache_change,
                        participant_address: self.participant_address.clone(),
                    },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

#[cfg(feature = "type_lookup")]
struct TypeLookupReplyReaderHistoryCache {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
}

#[cfg(feature = "type_lookup")]
impl HistoryCache for TypeLookupReplyReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddTypeLookupReplyCacheChange { cache_change },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
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
    /// Tracks when each discovered participant was last seen (for lease expiry)
    discovered_participant_last_seen: BTreeMap<InstanceHandle, Time>,
    /// Pending TypeLookup requests awaiting replies (CLIENT side)
    /// Key: (writer_guid as bytes, sequence_number) of the request we sent
    /// Value: oneshot sender to deliver the reply to the waiting caller
    #[cfg(feature = "type_lookup")]
    pending_type_requests: BTreeMap<
        ([u8; 16], SequenceNumber),
        OneshotSender<DdsResult<crate::dcps::data_representation_builtin_endpoints::type_lookup::TypeLookupReply>>,
    >,
    /// Sequence number counter for TypeLookup requests
    #[cfg(feature = "type_lookup")]
    type_lookup_sequence_number: SequenceNumber,
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
        participant_sender: MpscSender<DcpsDomainParticipantMail>,
        clock_handle: R::ClockHandle,
        timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
    ) -> Self {
        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        let participant_instance_handle = InstanceHandle::new(guid.into());

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

        let mut topic_list = Vec::new();

        let spdp_topic_participant_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
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
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
        );

        topic_list.push(TopicDescriptionKind::Topic(spdp_topic_participant));

        let sedp_topic_topics_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
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
            Arc::new(DiscoveredTopicData::get_type()),
        );

        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_topics));

        let sedp_topic_publications_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
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
            Arc::new(DiscoveredWriterData::get_type()),
        );
        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_publications));

        let sedp_topic_subscriptions_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
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
            Arc::new(DiscoveredReaderData::get_type()),
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

        let rtps_stateless_reader = RtpsStatelessReader::new(
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
            Box::new(DcpsParticipantReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
        );

        let dcps_participant_reader = DataReaderEntity::new(
            InstanceHandle::new(rtps_stateless_reader.guid().into()),
            spdp_reader_qos.clone(),
            String::from(DCPS_PARTICIPANT),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateless(rtps_stateless_reader),
        );

        let dcps_topic_transport_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
            Box::new(DcpsTopicsReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
            ReliabilityKind::Reliable,
        );

        let dcps_topic_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_topic_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_TOPIC),
            Arc::new(DiscoveredTopicData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_topic_transport_reader),
        );

        let dcps_publication_transport_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
            Box::new(DcpsPublicationsReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
            ReliabilityKind::Reliable,
        );

        let dcps_publication_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_publication_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_PUBLICATION),
            Arc::new(DiscoveredWriterData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_publication_transport_reader),
        );

        let dcps_subscription_transport_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
            Box::new(DcpsSubscriptionsReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
            ReliabilityKind::Reliable,
        );

        let dcps_subscription_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_subscription_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_SUBSCRIPTION),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_subscription_transport_reader),
        );

        let mut data_reader_list = vec![
            dcps_participant_reader,
            dcps_topic_reader,
            dcps_publication_reader,
            dcps_subscription_reader,
        ];

        // TypeLookup service endpoints (XTypes 1.3)
        #[cfg(feature = "type_lookup")]
        {
            use crate::dcps::data_representation_builtin_endpoints::type_lookup::{
                TypeLookupReply, TypeLookupRequest,
            };
            use crate::infrastructure::type_support::TypeSupport;

            // TypeLookup request reader - receives TypeLookup_Request messages (SERVER)
            let type_lookup_req_transport_reader = RtpsStatelessReader::new(
                Guid::new(guid_prefix, ENTITYID_TL_SVC_REQ_READER),
                Box::new(TypeLookupRequestReaderHistoryCache {
                    participant_address: participant_sender.clone(),
                }),
            );
            let type_lookup_req_reader = DataReaderEntity::new(
                InstanceHandle::new(type_lookup_req_transport_reader.guid().into()),
                spdp_reader_qos.clone(),
                String::from("TypeLookup_Request"),
                Arc::new(TypeLookupRequest::get_type()),
                Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
                None,
                Vec::new(),
                TransportReaderKind::Stateless(type_lookup_req_transport_reader),
            );
            data_reader_list.push(type_lookup_req_reader);

            // TypeLookup reply reader - receives TypeLookup_Reply messages (CLIENT)
            let type_lookup_reply_transport_reader = RtpsStatelessReader::new(
                Guid::new(guid_prefix, ENTITYID_TL_SVC_REPLY_READER),
                Box::new(TypeLookupReplyReaderHistoryCache {
                    participant_address: participant_sender.clone(),
                }),
            );
            let type_lookup_reply_reader = DataReaderEntity::new(
                InstanceHandle::new(type_lookup_reply_transport_reader.guid().into()),
                spdp_reader_qos.clone(),
                String::from("TypeLookup_Reply"),
                Arc::new(TypeLookupReply::get_type()),
                Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
                None,
                Vec::new(),
                TransportReaderKind::Stateless(type_lookup_reply_transport_reader),
            );
            data_reader_list.push(type_lookup_reply_reader);
        }

        let builtin_subscriber_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
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
            TransportWriterKind::Stateless(dcps_participant_transport_writer),
            String::from(DCPS_PARTICIPANT),
            "SpdpDiscoveredParticipantData".to_string(),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            spdp_writer_qos.clone(),
        );

        let dcps_topics_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
            transport.fragment_size,
        );

        let dcps_topics_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_topics_transport_writer.guid().into()),
            TransportWriterKind::Stateful(dcps_topics_transport_writer),
            String::from(DCPS_TOPIC),
            "DiscoveredTopicData".to_string(),
            Arc::new(DiscoveredTopicData::get_type()),
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
            TransportWriterKind::Stateful(dcps_publications_transport_writer),
            String::from(DCPS_PUBLICATION),
            "DiscoveredWriterData".to_string(),
            Arc::new(DiscoveredWriterData::get_type()),
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
            TransportWriterKind::Stateful(dcps_subscriptions_transport_writer),
            String::from(DCPS_SUBSCRIPTION),
            "DiscoveredReaderData".to_string(),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        let mut builtin_data_writer_list = vec![
            dcps_participant_writer,
            dcps_topics_writer,
            dcps_publications_writer,
            dcps_subscriptions_writer,
        ];

        // TypeLookup service writers (XTypes 1.3)
        #[cfg(feature = "type_lookup")]
        {
            use crate::dcps::data_representation_builtin_endpoints::type_lookup::{
                TypeLookupReply, TypeLookupRequest,
            };
            use crate::infrastructure::type_support::TypeSupport;

            // TypeLookup reply writer - sends TypeLookup_Reply messages (SERVER)
            let type_lookup_reply_transport_writer = RtpsStatelessWriter::new(Guid::new(
                guid_prefix,
                ENTITYID_TL_SVC_REPLY_WRITER,
            ));
            let type_lookup_reply_writer = DataWriterEntity::new(
                InstanceHandle::new(type_lookup_reply_transport_writer.guid().into()),
                TransportWriterKind::Stateless(type_lookup_reply_transport_writer),
                String::from("TypeLookup_Reply"),
                "TypeLookup_Reply".to_string(),
                Arc::new(TypeLookupReply::get_type()),
                Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
                None,
                vec![],
                spdp_writer_qos.clone(),
            );
            builtin_data_writer_list.push(type_lookup_reply_writer);

            // TypeLookup request writer - sends TypeLookup_Request messages (CLIENT)
            let type_lookup_req_transport_writer = RtpsStatelessWriter::new(Guid::new(
                guid_prefix,
                ENTITYID_TL_SVC_REQ_WRITER,
            ));
            let type_lookup_req_writer = DataWriterEntity::new(
                InstanceHandle::new(type_lookup_req_transport_writer.guid().into()),
                TransportWriterKind::Stateless(type_lookup_req_transport_writer),
                String::from("TypeLookup_Request"),
                "TypeLookup_Request".to_string(),
                Arc::new(TypeLookupRequest::get_type()),
                Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
                None,
                vec![],
                spdp_writer_qos.clone(),
            );
            builtin_data_writer_list.push(type_lookup_req_writer);
        }

        let builtin_publisher_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
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
            participant_instance_handle,
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
            discovered_participant_last_seen: BTreeMap::new(),
            #[cfg(feature = "type_lookup")]
            pending_type_requests: BTreeMap::new(),
            #[cfg(feature = "type_lookup")]
            type_lookup_sequence_number: 1,
        }
    }

    fn get_participant_async(
        &self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DomainParticipantAsync {
        DomainParticipantAsync::new(
            participant_address,
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
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
            self.get_participant_async(participant_address),
        ))
    }

    fn get_data_reader_async<Foo>(
        &self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
            self.get_subscriber_async(participant_address.clone(), subscriber_handle)?,
            self.get_topic_description_async(participant_address, data_reader.topic_name.clone())?,
        ))
    }

    fn get_publisher_async(
        &self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherAsync> {
        Ok(PublisherAsync::new(
            publisher_handle,
            self.get_participant_async(participant_address),
        ))
    }

    fn get_data_writer_async<Foo>(
        &self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
            self.get_publisher_async(participant_address.clone(), publisher_handle)?,
            self.get_topic_description_async(participant_address, data_writer.topic_name.clone())?,
        ))
    }

    fn get_topic_description_async(
        &self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        topic_name: String,
    ) -> DdsResult<TopicDescriptionAsync> {
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
                    self.get_participant_async(participant_address),
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
                        self.get_participant_async(participant_address),
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
    pub async fn get_inconsistent_topic_status(
        &mut self,
        topic_name: String,
    ) -> DdsResult<InconsistentTopicStatus> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let status = topic.inconsistent_topic_status.clone();
        topic.inconsistent_topic_status.total_count_change = 0;
        topic
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::InconsistentTopic,
            })
            .await;

        Ok(status)
    }

    #[tracing::instrument(skip(self))]
    pub fn set_topic_qos(
        &mut self,
        topic_name: String,
        topic_qos: QosKind<TopicQos>,
    ) -> DdsResult<()> {
        let qos = match topic_qos {
            QosKind::Default => self.domain_participant.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;

        if topic.enabled
            && (topic.qos.durability != qos.durability
                || topic.qos.liveliness != qos.liveliness
                || topic.qos.reliability != qos.reliability
                || topic.qos.destination_order != qos.destination_order
                || topic.qos.history != qos.history
                || topic.qos.resource_limits != qos.resource_limits
                || topic.qos.ownership != qos.ownership)
        {
            return Err(DdsError::ImmutablePolicy);
        }

        topic.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_topic_qos(&mut self, topic_name: String) -> DdsResult<TopicQos> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(topic.qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable_topic(
        &mut self,
        topic_name: String,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<()> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !topic.enabled {
            topic.enabled = true;
            self.announce_topic(topic_name, participant_address).await;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_type_support(&mut self, topic_name: String) -> DdsResult<Arc<DynamicType>> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(topic.type_support.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn create_user_defined_publisher(
        &mut self,
        qos: QosKind<PublisherQos>,
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<InstanceHandle> {
        let publisher_qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let publisher_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            self.publisher_counter,
            0,
            0,
            USER_DEFINED_WRITER_GROUP,
        ]);
        self.publisher_counter += 1;
        let data_writer_list = Default::default();
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        let mut publisher = PublisherEntity::new(
            publisher_qos,
            publisher_handle,
            data_writer_list,
            listener_sender,
            mask,
        );

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            publisher.enabled = true;
        }

        self.domain_participant
            .user_defined_publisher_list
            .push(publisher);

        Ok(publisher_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_publisher(
        &mut self,
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<()> {
        if participant_handle != self.domain_participant.instance_handle {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !publisher.data_writer_list.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }
        let Some(_) = self.domain_participant.remove_publisher(&publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn create_user_defined_subscriber(
        &mut self,
        qos: QosKind<SubscriberQos>,
        dcps_listener: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)> {
        let subscriber_qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let status_condition =
            Actor::spawn::<R>(DcpsStatusCondition::default(), &self.spawner_handle);
        let subscriber_status_condition_address = status_condition.address();
        let subscriber_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            self.subscriber_counter,
            0,
            0,
            USER_DEFINED_READER_GROUP,
        ]);
        self.subscriber_counter += 1;

        let listener_mask = mask.to_vec();
        let data_reader_list = Default::default();

        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        let mut subscriber = SubscriberEntity::new(
            subscriber_handle,
            subscriber_qos,
            data_reader_list,
            status_condition,
            listener_sender,
            listener_mask,
        );

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            subscriber.enabled = true;
        }

        self.domain_participant
            .user_defined_subscriber_list
            .push(subscriber);

        Ok((subscriber_handle, subscriber_status_condition_address))
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_subscriber(
        &mut self,
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<()> {
        if self.domain_participant.instance_handle != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !subscriber.data_reader_list.is_empty() {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }
        let Some(_) = self
            .domain_participant
            .remove_subscriber(&subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, dcps_listener, type_support))]
    pub async fn create_topic(
        &mut self,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        dcps_listener: Option<DcpsTopicListener>,
        mask: Vec<StatusKind>,
        type_support: Arc<DynamicType>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)> {
        if self
            .domain_participant
            .topic_description_list
            .iter()
            .any(|x| x.topic_name() == topic_name)
        {
            return Err(DdsError::PreconditionNotMet(format!(
                "Topic with name {topic_name} already exists.
         To access this topic call the lookup_topicdescription method.",
            )));
        }

        let status_condition =
            Actor::spawn::<R>(DcpsStatusCondition::default(), &self.spawner_handle);
        let topic_status_condition_address = status_condition.address();
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let topic_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            0,
            self.topic_counter.to_ne_bytes()[0],
            self.topic_counter.to_ne_bytes()[1],
            USER_DEFINED_TOPIC,
        ]);
        self.topic_counter += 1;
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        let topic = TopicEntity::new(
            qos,
            type_name,
            topic_name.clone(),
            topic_handle,
            status_condition,
            listener_sender,
            mask,
            type_support,
        );

        self.domain_participant
            .topic_description_list
            .push(TopicDescriptionKind::Topic(topic));

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            self.enable_topic(topic_name, participant_address).await?;
        }

        Ok((topic_handle, topic_status_condition_address))
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_topic(
        &mut self,
        participant_handle: InstanceHandle,
        topic_name: String,
    ) -> DdsResult<()> {
        if self.domain_participant.instance_handle != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        if BUILT_IN_TOPIC_NAME_LIST.contains(&topic_name.as_str()) {
            return Ok(());
        }

        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if Arc::strong_count(&topic.type_support) > 1 {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data writer or data reader".to_string(),
            ));
        }

        self.domain_participant
            .topic_description_list
            .retain(|x| x.topic_name() != topic_name);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_content_filtered_topic(
        &mut self,
        participant_handle: InstanceHandle,
        name: String,
        related_topic_name: String,
        filter_expression: String,
        expression_parameters: Vec<String>,
    ) -> DdsResult<InstanceHandle> {
        if !self
            .domain_participant
            .topic_description_list
            .iter()
            .any(|x| x.topic_name() == related_topic_name)
        {
            return Err(DdsError::PreconditionNotMet(format!(
                "Related topic with name {related_topic_name} does not exist."
            )));
        }

        let topic_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            0,
            self.topic_counter.to_ne_bytes()[0],
            self.topic_counter.to_ne_bytes()[1],
            USER_DEFINED_TOPIC,
        ]);
        self.topic_counter += 1;

        let topic = ContentFilteredTopicEntity::new(
            name,
            related_topic_name,
            filter_expression,
            expression_parameters,
        );
        self.domain_participant
            .topic_description_list
            .push(TopicDescriptionKind::ContentFilteredTopic(topic));

        Ok(topic_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_content_filtered_topic(
        &mut self,
        participant_handle: InstanceHandle,
        name: String,
    ) -> DdsResult<()> {
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip(self, type_support))]
    pub fn find_topic(
        &mut self,
        topic_name: String,
        type_support: Arc<DynamicType>,
    ) -> DdsResult<Option<(InstanceHandle, ActorAddress<DcpsStatusCondition>, String)>> {
        if let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        {
            Ok(Some((
                topic.instance_handle,
                topic.status_condition.address(),
                topic.type_name.clone(),
            )))
        } else {
            if let Some(discovered_topic_data) = self.domain_participant.find_topic(&topic_name) {
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
                    self.domain_participant.instance_handle[0],
                    self.domain_participant.instance_handle[1],
                    self.domain_participant.instance_handle[2],
                    self.domain_participant.instance_handle[3],
                    self.domain_participant.instance_handle[4],
                    self.domain_participant.instance_handle[5],
                    self.domain_participant.instance_handle[6],
                    self.domain_participant.instance_handle[7],
                    self.domain_participant.instance_handle[8],
                    self.domain_participant.instance_handle[9],
                    self.domain_participant.instance_handle[10],
                    self.domain_participant.instance_handle[11],
                    0,
                    self.topic_counter.to_ne_bytes()[0],
                    self.topic_counter.to_ne_bytes()[1],
                    USER_DEFINED_TOPIC,
                ]);
                self.topic_counter += 1;
                let status_condition =
                    Actor::spawn::<R>(DcpsStatusCondition::default(), &self.spawner_handle);
                let mut topic = TopicEntity::new(
                    qos,
                    type_name.clone(),
                    topic_name.clone(),
                    topic_handle,
                    status_condition,
                    None,
                    vec![],
                    type_support,
                );
                topic.enabled = true;
                let topic_status_condition_address = topic.status_condition.address();

                match self
                    .domain_participant
                    .topic_description_list
                    .iter_mut()
                    .find(|x| x.topic_name() == topic.topic_name)
                {
                    Some(TopicDescriptionKind::Topic(x)) => *x = topic,
                    Some(TopicDescriptionKind::ContentFilteredTopic(_)) => {
                        return Err(DdsError::IllegalOperation);
                    }
                    None => self
                        .domain_participant
                        .topic_description_list
                        .push(TopicDescriptionKind::Topic(topic)),
                }

                return Ok(Some((
                    topic_handle,
                    topic_status_condition_address,
                    type_name,
                )));
            }
            Ok(None)
        }
    }

    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub fn lookup_topicdescription(
        &mut self,
        topic_name: String,
    ) -> DdsResult<Option<(String, InstanceHandle, ActorAddress<DcpsStatusCondition>)>> {
        if let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        {
            Ok(Some((
                topic.type_name.clone(),
                topic.instance_handle,
                topic.status_condition.address(),
            )))
        } else {
            Ok(None)
        }
    }

    /// Ignore participant with the specified [`handle`](InstanceHandle).
    #[tracing::instrument(skip(self))]
    pub async fn ignore_participant(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        // Check enabled
        if !self.domain_participant.enabled {
            return Err(DdsError::NotEnabled);
        }

        // Add to ignored participants
        if !self.domain_participant.ignored_participants.insert(handle) {
            // Already ignored
            return Ok(());
        }

        // Remove participant
        self.remove_discovered_participant(handle).await;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn ignore_publication(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if !self.domain_participant.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.domain_participant.ignored_publications.insert(handle);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn ignore_subscription(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if !self.domain_participant.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.domain_participant.ignored_subscriptions.insert(handle);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_participant_contained_entities(&mut self) -> DdsResult<()> {
        let deleted_publisher_list: Vec<PublisherEntity> = self
            .domain_participant
            .user_defined_publisher_list
            .drain(..)
            .collect();
        for mut publisher in deleted_publisher_list {
            for data_writer in publisher.data_writer_list.drain(..) {
                self.announce_deleted_data_writer(data_writer).await;
            }
        }

        let deleted_subscriber_list: Vec<SubscriberEntity> = self
            .domain_participant
            .user_defined_subscriber_list
            .drain(..)
            .collect();
        for mut subscriber in deleted_subscriber_list {
            for data_reader in subscriber.data_reader_list.drain(..) {
                self.announce_deleted_data_reader(data_reader).await;
            }
        }

        self.domain_participant
            .topic_description_list
            .retain(|x| BUILT_IN_TOPIC_NAME_LIST.contains(&x.topic_name()));

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_publisher_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.default_publisher_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_publisher_qos(&mut self) -> DdsResult<PublisherQos> {
        Ok(self.domain_participant.default_publisher_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_subscriber_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.default_subscriber_qos = qos;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_subscriber_qos(&mut self) -> DdsResult<SubscriberQos> {
        Ok(self.domain_participant.default_subscriber_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_topic_qos(&mut self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };

        qos.is_consistent()?;
        self.domain_participant.default_topic_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.domain_participant.default_topic_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participants(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .domain_participant
            .discovered_participant_list
            .iter()
            .map(|p| InstanceHandle::new(p.dds_participant_data.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participant_data(
        &mut self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| &p.dds_participant_data.key().value == participant_handle.as_ref())
        else {
            return Err(DdsError::BadParameter);
        };
        Ok(handle.dds_participant_data.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topics(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .domain_participant
            .discovered_topic_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topic_data(
        &mut self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .get_discovered_topic_data(&topic_handle)
        else {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Topic with this handle not discovered",
            )));
        };

        Ok(handle.clone())
    }

    /// Get discovered writer info for a topic (used for type discovery).
    ///
    /// Returns the participant prefix and type_information bytes if a writer is found.
    #[cfg(feature = "type_lookup")]
    #[tracing::instrument(skip(self))]
    pub fn get_discovered_writer_for_topic(
        &self,
        topic_name: &str,
    ) -> Option<([u8; 12], Option<Vec<u8>>)> {
        self.domain_participant
            .discovered_writer_list
            .iter()
            .find(|w| w.dds_publication_data.topic_name == topic_name)
            .map(|w| {
                (
                    w.writer_proxy.remote_writer_guid.prefix(),
                    w.type_information.clone(),
                )
            })
    }

    #[tracing::instrument(skip(self))]
    pub fn get_current_time(&mut self) -> Time {
        self.clock_handle.now()
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_domain_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.qos = qos;
        if self.domain_participant.enabled {
            self.announce_participant(participant_address).await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_domain_participant_qos(&mut self) -> DdsResult<DomainParticipantQos> {
        Ok(self.domain_participant.qos.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn set_domain_participant_listener(
        &mut self,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        self.domain_participant.listener_sender = listener_sender;
        self.domain_participant.listener_mask = status_kind;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable_domain_participant(
        &mut self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<()> {
        if !self.domain_participant.enabled {
            for t in &mut self.domain_participant.topic_description_list {
                if let TopicDescriptionKind::Topic(t) = t {
                    t.enabled = true;
                }
            }
            for dw in &mut self.domain_participant.builtin_publisher.data_writer_list {
                dw.enabled = true;
            }
            self.domain_participant.builtin_publisher.enabled = true;

            for dr in &mut self.domain_participant.builtin_subscriber.data_reader_list {
                dr.enabled = true;
            }
            self.domain_participant.builtin_subscriber.enabled = true;
            self.domain_participant.enabled = true;

            self.announce_participant(participant_address).await;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn is_participant_empty(&mut self) -> bool {
        self.domain_participant.is_empty()
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, dcps_listener, domain_participant_address))]
    pub async fn create_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        dcps_listener: Option<DcpsDataReaderListener>,
        mask: Vec<StatusKind>,
        domain_participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)> {
        struct UserDefinedReaderHistoryCache {
            domain_participant_address: MpscSender<DcpsDomainParticipantMail>,
            subscriber_handle: InstanceHandle,
            data_reader_handle: InstanceHandle,
        }

        impl HistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(
                &mut self,
                cache_change: CacheChange,
            ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
                Box::pin(async move {
                    self.domain_participant_address
                        .send(DcpsDomainParticipantMail::Message(
                            MessageServiceMail::AddCacheChange {
                                participant_address: self.domain_participant_address.clone(),
                                cache_change,
                                subscriber_handle: self.subscriber_handle,
                                data_reader_handle: self.data_reader_handle,
                            },
                        ))
                        .await
                        .ok();
                })
            }

            fn remove_change(
                &mut self,
                _sequence_number: i64,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                todo!()
            }
        }

        let Some(topic) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic = match topic {
            TopicDescriptionKind::Topic(t) => t,
            TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic) => {
                if let Some(TopicDescriptionKind::Topic(topic)) = self
                    .domain_participant
                    .topic_description_list
                    .iter()
                    .find(|x| x.topic_name() == content_filtered_topic.related_topic_name)
                {
                    topic
                } else {
                    return Err(DdsError::AlreadyDeleted);
                }
            }
        };

        let topic_kind = get_topic_kind(topic.type_support.as_ref());

        let type_support = topic.type_support.clone();
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };

        let entity_kind = match topic_kind {
            TopicKind::NoKey => USER_DEFINED_READER_NO_KEY,
            TopicKind::WithKey => USER_DEFINED_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(
            [
                subscriber.instance_handle[12],
                self.reader_counter.to_ne_bytes()[0],
                self.reader_counter.to_ne_bytes()[1],
            ],
            entity_kind,
        );
        let reader_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            entity_id.entity_key()[0],
            entity_id.entity_key()[1],
            entity_id.entity_key()[2],
            entity_id.entity_kind(),
        ]);
        self.reader_counter += 1;
        let reliablity_kind = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        };
        let guid_prefix = Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix();
        let guid = Guid::new(guid_prefix, entity_id);

        let transport_reader = TransportReaderKind::Stateful(RtpsStatefulReader::new(
            guid,
            Box::new(UserDefinedReaderHistoryCache {
                domain_participant_address: domain_participant_address.clone(),
                subscriber_handle: subscriber.instance_handle,
                data_reader_handle: reader_handle,
            }),
            reliablity_kind,
        ));

        let status_condition =
            Actor::spawn::<R>(DcpsStatusCondition::default(), &self.spawner_handle);
        let reader_status_condition_address = status_condition.address();
        let listener_mask = mask.to_vec();
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        let data_reader = DataReaderEntity::new(
            reader_handle,
            qos,
            topic_name,
            type_support,
            status_condition,
            listener_sender,
            listener_mask,
            transport_reader,
        );

        let data_reader_handle = data_reader.instance_handle;

        // Capture type info for TypeLookup registration before moving data_reader
        #[cfg(feature = "type_lookup")]
        let type_for_registry = data_reader.type_support.as_ref().clone();

        subscriber.data_reader_list.push(data_reader);

        let should_enable =
            subscriber.enabled && subscriber.qos.entity_factory.autoenable_created_entities;

        // Register the type in the type registry for TypeLookup service
        // (done after subscriber borrow is released)
        #[cfg(feature = "type_lookup")]
        {
            use crate::dcps::data_representation_builtin_endpoints::type_lookup::compute_equivalence_hash;
            let hash = compute_equivalence_hash(&type_for_registry);
            self.domain_participant.register_type(hash, type_for_registry);
        }

        if should_enable {
            self.enable_data_reader(
                subscriber_handle,
                data_reader_handle,
                domain_participant_address,
            )
            .await?;
        }
        Ok((data_reader_handle, reader_status_condition_address))
    }

    /// Creates a dynamic data reader for runtime type discovery.
    ///
    /// Unlike `create_data_reader`, this method accepts a `DynamicType` directly,
    /// allowing readers to be created for types discovered at runtime via TypeLookup.
    #[tracing::instrument(skip(self, dynamic_type))]
    pub async fn create_dynamic_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        topic_name: String,
        dynamic_type: DynamicType,
        qos: QosKind<DataReaderQos>,
        domain_participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)> {
        struct UserDefinedReaderHistoryCache {
            domain_participant_address: MpscSender<DcpsDomainParticipantMail>,
            subscriber_handle: InstanceHandle,
            data_reader_handle: InstanceHandle,
        }

        impl HistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(
                &mut self,
                cache_change: CacheChange,
            ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
                Box::pin(async move {
                    self.domain_participant_address
                        .send(DcpsDomainParticipantMail::Message(
                            MessageServiceMail::AddCacheChange {
                                participant_address: self.domain_participant_address.clone(),
                                cache_change,
                                subscriber_handle: self.subscriber_handle,
                                data_reader_handle: self.data_reader_handle,
                            },
                        ))
                        .await
                        .ok();
                })
            }

            fn remove_change(
                &mut self,
                _sequence_number: i64,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                todo!()
            }
        }

        let topic_kind = get_topic_kind(&dynamic_type);
        let type_support = Arc::new(dynamic_type);

        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };

        let entity_kind = match topic_kind {
            TopicKind::NoKey => USER_DEFINED_READER_NO_KEY,
            TopicKind::WithKey => USER_DEFINED_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(
            [
                subscriber.instance_handle[12],
                self.reader_counter.to_ne_bytes()[0],
                self.reader_counter.to_ne_bytes()[1],
            ],
            entity_kind,
        );
        let reader_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            entity_id.entity_key()[0],
            entity_id.entity_key()[1],
            entity_id.entity_key()[2],
            entity_id.entity_kind(),
        ]);
        self.reader_counter += 1;
        let reliablity_kind = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        };
        let guid_prefix = Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix();
        let guid = Guid::new(guid_prefix, entity_id);

        let transport_reader = TransportReaderKind::Stateful(RtpsStatefulReader::new(
            guid,
            Box::new(UserDefinedReaderHistoryCache {
                domain_participant_address: domain_participant_address.clone(),
                subscriber_handle: subscriber.instance_handle,
                data_reader_handle: reader_handle,
            }),
            reliablity_kind,
        ));

        let status_condition =
            Actor::spawn::<R>(DcpsStatusCondition::default(), &self.spawner_handle);
        let reader_status_condition_address = status_condition.address();
        // No listener for dynamic readers (simplification)
        let listener_sender = None;
        let listener_mask = Vec::new();
        let data_reader = DataReaderEntity::new(
            reader_handle,
            qos,
            topic_name,
            type_support.clone(),
            status_condition,
            listener_sender,
            listener_mask,
            transport_reader,
        );

        let data_reader_handle = data_reader.instance_handle;

        // Capture type info for TypeLookup registration before moving data_reader
        #[cfg(feature = "type_lookup")]
        let type_for_registry = data_reader.type_support.as_ref().clone();

        subscriber.data_reader_list.push(data_reader);

        let should_enable =
            subscriber.enabled && subscriber.qos.entity_factory.autoenable_created_entities;

        // Register the type in the type registry for TypeLookup service
        #[cfg(feature = "type_lookup")]
        {
            use crate::dcps::data_representation_builtin_endpoints::type_lookup::compute_equivalence_hash;
            let hash = compute_equivalence_hash(&type_for_registry);
            self.domain_participant.register_type(hash, type_for_registry);
        }

        if should_enable {
            self.enable_data_reader(
                subscriber_handle,
                data_reader_handle,
                domain_participant_address,
            )
            .await?;
        }
        Ok((data_reader_handle, reader_status_condition_address))
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        datareader_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x: &&mut SubscriberEntity| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if let Some(index) = subscriber
            .data_reader_list
            .iter()
            .position(|x| x.instance_handle == datareader_handle)
        {
            let data_reader = subscriber.data_reader_list.remove(index);
            self.announce_deleted_data_reader(data_reader).await;
        } else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub fn lookup_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        topic_name: String,
    ) -> DdsResult<Option<(InstanceHandle, ActorAddress<DcpsStatusCondition>)>> {
        if !self
            .domain_participant
            .topic_description_list
            .iter()
            .any(|x| x.topic_name() == topic_name)
        {
            return Err(DdsError::BadParameter);
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.domain_participant.instance_handle == subscriber_handle {
            Ok(self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| dr.topic_name == topic_name)
                .map(|x| (x.instance_handle, x.status_condition.address())))
        } else {
            let Some(s) = self
                .domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return Err(DdsError::AlreadyDeleted);
            };
            Ok(s.data_reader_list
                .iter_mut()
                .find(|dr| dr.topic_name == topic_name)
                .map(|x| (x.instance_handle, x.status_condition.address())))
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => DataReaderQos::default(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;
        subscriber.default_data_reader_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.default_data_reader_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_subscriber_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if subscriber.enabled {
            subscriber.qos.check_immutability(&qos)?;
        }
        subscriber.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_subscriber_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<SubscriberQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.qos.clone())
    }

    #[tracing::instrument(skip(self, listener_sender_task))]
    pub fn set_subscriber_listener(
        &mut self,
        subscriber_handle: InstanceHandle,
        listener_sender_task: Option<DcpsSubscriberListener>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let listener_sender = listener_sender_task.map(|l| l.spawn::<R>(&self.spawner_handle));
        subscriber.listener_sender = listener_sender;
        subscriber.listener_mask = mask;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, dcps_listener, participant_address))]
    pub async fn create_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        dcps_listener: Option<DcpsDataWriterListener>,
        mask: Vec<StatusKind>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<DcpsStatusCondition>)> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic_kind = get_topic_kind(topic.type_support.as_ref());
        let type_support = topic.type_support.clone();
        let type_name = topic.type_name.clone();

        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_WRITER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_WRITER_NO_KEY,
        };

        let entity_id = EntityId::new(
            [
                publisher.instance_handle[12],
                self.writer_counter.to_le_bytes()[0],
                self.writer_counter.to_le_bytes()[1],
            ],
            entity_kind,
        );

        let writer_handle = InstanceHandle::new([
            self.domain_participant.instance_handle[0],
            self.domain_participant.instance_handle[1],
            self.domain_participant.instance_handle[2],
            self.domain_participant.instance_handle[3],
            self.domain_participant.instance_handle[4],
            self.domain_participant.instance_handle[5],
            self.domain_participant.instance_handle[6],
            self.domain_participant.instance_handle[7],
            self.domain_participant.instance_handle[8],
            self.domain_participant.instance_handle[9],
            self.domain_participant.instance_handle[10],
            self.domain_participant.instance_handle[11],
            entity_id.entity_key()[0],
            entity_id.entity_key()[1],
            entity_id.entity_key()[2],
            entity_id.entity_kind(),
        ]);
        self.writer_counter += 1;

        let status_condition =
            Actor::spawn::<R>(DcpsStatusCondition::default(), &self.spawner_handle);
        let writer_status_condition_address = status_condition.address();
        let qos = match qos {
            QosKind::Default => publisher.default_datawriter_qos.clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };
        let guid = Guid::from(*self.domain_participant.instance_handle.as_ref());
        let transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid.prefix(), entity_id),
            self.transport.fragment_size,
        );
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        let data_writer = DataWriterEntity::new(
            writer_handle,
            TransportWriterKind::Stateful(transport_writer),
            topic_name,
            type_name,
            type_support,
            status_condition,
            listener_sender,
            mask,
            qos,
        );
        let data_writer_handle = data_writer.instance_handle;

        // Capture type info for TypeLookup registration before moving data_writer
        #[cfg(feature = "type_lookup")]
        let type_for_registry = data_writer.type_support.as_ref().clone();

        publisher.data_writer_list.push(data_writer);

        let should_enable =
            publisher.enabled && publisher.qos.entity_factory.autoenable_created_entities;

        // Register the type in the type registry for TypeLookup service
        // (done after publisher borrow is released)
        #[cfg(feature = "type_lookup")]
        {
            use crate::dcps::data_representation_builtin_endpoints::type_lookup::compute_equivalence_hash;
            let hash = compute_equivalence_hash(&type_for_registry);
            self.domain_participant.register_type(hash, type_for_registry);
        }

        if should_enable {
            self.enable_data_writer(publisher_handle, writer_handle, participant_address)
                .await?;
        }

        Ok((data_writer_handle, writer_status_condition_address))
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        datawriter_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if let Some(index) = publisher
            .data_writer_list
            .iter()
            .position(|x| x.instance_handle == datawriter_handle)
        {
            let data_writer = publisher.data_writer_list.remove(index);
            self.announce_deleted_data_writer(data_writer).await;
            Ok(())
        } else {
            return Err(DdsError::AlreadyDeleted);
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_datawriter_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(publisher.default_datawriter_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_datawriter_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;
        publisher.default_datawriter_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        publisher.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(publisher.qos.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn set_publisher_listener(
        &mut self,
        publisher_handle: InstanceHandle,
        dcps_listener: Option<DcpsPublisherListener>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        publisher.listener_sender = listener_sender;
        publisher.listener_mask = mask;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<PublicationMatchedStatus> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let status = data_writer.get_publication_matched_status();

        data_writer
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            })
            .await;
        Ok(status)
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn set_listener_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataWriterListener>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Ok(());
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Ok(());
        };

        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        data_writer.listener_sender = listener_sender;
        data_writer.listener_mask = listener_mask;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_writer_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_subscriptions(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer
            .matched_subscription_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_subscription_data(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_writer
            .matched_subscription_list
            .iter()
            .find(|x| subscription_handle.as_ref() == &x.key().value)
            .ok_or(DdsError::BadParameter)
            .cloned()
    }

    #[tracing::instrument(skip(self))]
    pub async fn unregister_instance(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_writer
            .unregister_w_timestamp(
                dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await
    }

    #[tracing::instrument(skip(self))]
    pub fn lookup_instance(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
    ) -> DdsResult<Option<InstanceHandle>> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }

        let instance_handle = match get_instance_handle_from_dynamic_data(dynamic_data) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(data_writer
            .registered_instance_list
            .contains(&instance_handle)
            .then_some(instance_handle))
    }

    #[tracing::instrument(skip(self, participant_address, reply_sender))]
    pub async fn write_w_timestamp(
        &mut self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    ) {
        let now = self.get_current_time();
        let Some(publisher) = core::iter::once(&mut self.domain_participant.builtin_publisher)
            .chain(&mut self.domain_participant.user_defined_publisher_list)
            .find(|x| x.instance_handle == publisher_handle)
        else {
            reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        if !data_writer.enabled {
            reply_sender.send(Err(DdsError::NotEnabled));
            return;
        }

        let instance_handle = match get_instance_handle_from_dynamic_data(dynamic_data.clone()) {
            Ok(h) => h,
            Err(e) => {
                reply_sender.send(Err(e.into()));
                return;
            }
        };

        if !data_writer
            .registered_instance_list
            .contains(&instance_handle)
        {
            if data_writer.registered_instance_list.len()
                < data_writer.qos.resource_limits.max_instances
            {
                data_writer.registered_instance_list.push(instance_handle);
            } else {
                reply_sender.send(Err(DdsError::OutOfResources));
                return;
            }
        }

        if let Length::Limited(max_instances) = data_writer.qos.resource_limits.max_instances {
            if !data_writer
                .instance_samples
                .iter()
                .any(|x| x.instance == instance_handle)
                && data_writer.instance_samples.len() == max_instances as usize
            {
                reply_sender.send(Err(DdsError::OutOfResources));
                return;
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            data_writer.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match data_writer.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth)
                    if depth as i32 <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = data_writer
                        .instance_samples
                        .iter()
                        .find(|x| x.instance == instance_handle)
                    {
                        // Only Alive changes count towards the resource limits
                        if s.samples.len() >= max_samples_per_instance as usize {
                            reply_sender.send(Err(DdsError::OutOfResources));
                            return;
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = data_writer.qos.resource_limits.max_samples {
            let total_samples = data_writer
                .instance_samples
                .iter()
                .fold(0, |acc, x| acc + x.samples.len());

            if total_samples >= max_samples as usize {
                reply_sender.send(Err(DdsError::OutOfResources));
                return;
            }
        }

        let serialized_data = match serialize(&dynamic_data, &data_writer.qos.representation) {
            Ok(s) => s,
            Err(e) => {
                reply_sender.send(Err(e));
                return;
            }
        };

        if let HistoryQosPolicyKind::KeepLast(depth) = data_writer.qos.history.kind {
            if let Some(s) = data_writer
                .instance_samples
                .iter_mut()
                .find(|x| x.instance == instance_handle)
            {
                if s.samples.len() == depth as usize {
                    if let Some(&smallest_seq_num_instance) = s.samples.front() {
                        if data_writer.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
                            if let TransportWriterKind::Stateful(w) = &data_writer.transport_writer
                            {
                                if !w.is_change_acknowledged(smallest_seq_num_instance) {
                                    if data_writer.acknowledgement_notification.is_some() {
                                        reply_sender.send(Err(DdsError::Error(String::from(
                                            "Another writer already waiting for acknowledgements.",
                                        ))));
                                        return;
                                    }
                                    let mut timer_handle = self.timer_handle.clone();
                                    let max_blocking_time =
                                        data_writer.qos.reliability.max_blocking_time;
                                    let (
                                        acknowledgment_notification_sender,
                                        acknowledgment_notification_receiver,
                                    ) = oneshot::<()>();
                                    data_writer.acknowledgement_notification =
                                        Some(acknowledgment_notification_sender);
                                    let participant_address_clone = participant_address.clone();
                                    self.spawner_handle.spawn(async move {
                                        if let DurationKind::Finite(t) = max_blocking_time {
                                            let max_blocking_time_wait =
                                                timer_handle.delay(t.into());
                                            match select_future(
                                                acknowledgment_notification_receiver,
                                                max_blocking_time_wait,
                                            )
                                            .await
                                            {
                                                Either::A(_) => {
                                                    participant_address_clone
                                                        .send(DcpsDomainParticipantMail::Writer(
                                                            WriterServiceMail::WriteWTimestamp {
                                                                participant_address:
                                                                    participant_address_clone
                                                                        .clone(),
                                                                publisher_handle,
                                                                data_writer_handle,
                                                                dynamic_data,
                                                                timestamp,
                                                                reply_sender,
                                                            },
                                                        ))
                                                        .await
                                                        .ok();
                                                }
                                                Either::B(_) => {
                                                    reply_sender.send(Err(DdsError::Timeout))
                                                }
                                            };
                                        } else {
                                            acknowledgment_notification_receiver.await.ok();
                                            participant_address_clone
                                                .send(DcpsDomainParticipantMail::Writer(
                                                    WriterServiceMail::WriteWTimestamp {
                                                        participant_address:
                                                            participant_address_clone.clone(),
                                                        publisher_handle,
                                                        data_writer_handle,
                                                        dynamic_data,
                                                        timestamp,
                                                        reply_sender,
                                                    },
                                                ))
                                                .await
                                                .ok();
                                        }
                                    });
                                    return;
                                }
                            }
                        }
                    }
                    if let Some(smallest_seq_num_instance) = s.samples.pop_front() {
                        data_writer
                            .transport_writer
                            .remove_change(smallest_seq_num_instance)
                            .await;
                    }
                }
            }
        }

        data_writer.last_change_sequence_number += 1;
        let change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: data_writer.transport_writer.guid(),
            sequence_number: data_writer.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_data.into(),
        };
        let seq_num = change.sequence_number;

        if seq_num > data_writer.max_seq_num.unwrap_or(0) {
            data_writer.max_seq_num = Some(seq_num)
        }

        match data_writer
            .instance_publication_time
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(x) => {
                if x.last_write_time < timestamp {
                    x.last_write_time = timestamp;
                }
            }
            None => data_writer
                .instance_publication_time
                .push(InstancePublicationTime {
                    instance: instance_handle,
                    last_write_time: timestamp,
                }),
        }

        match data_writer
            .instance_samples
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(s) => s.samples.push_back(change.sequence_number),
            None => {
                let s = InstanceSamples {
                    instance: instance_handle,
                    samples: VecDeque::from([change.sequence_number]),
                };
                data_writer.instance_samples.push(s);
            }
        }

        if let DurationKind::Finite(deadline_missed_period) = data_writer.qos.deadline.period {
            let mut timer_handle = self.timer_handle.clone();
            let participant_address_clone = participant_address.clone();
            self.spawner_handle.spawn(async move {
                loop {
                    timer_handle.delay(deadline_missed_period.into()).await;
                    participant_address_clone
                        .send(DcpsDomainParticipantMail::Event(
                            EventServiceMail::OfferedDeadlineMissed {
                                publisher_handle,
                                data_writer_handle,
                                change_instance_handle: instance_handle,
                                participant_address: participant_address_clone.clone(),
                            },
                        ))
                        .await
                        .ok();
                }
            });
        }

        let sequence_number = data_writer.last_change_sequence_number;

        if let DurationKind::Finite(lifespan_duration) = data_writer.qos.lifespan.duration {
            let sleep_duration = timestamp - now + lifespan_duration;
            let mut timer_handle = self.timer_handle.clone();
            if sleep_duration <= Duration::new(0, 0) {
                reply_sender.send(Ok(()));
                return;
            }

            self.spawner_handle.spawn(async move {
                timer_handle.delay(sleep_duration.into()).await;
                participant_address
                    .send(DcpsDomainParticipantMail::Message(
                        MessageServiceMail::RemoveWriterChange {
                            publisher_handle,
                            data_writer_handle,
                            sequence_number,
                        },
                    ))
                    .await
                    .ok();
            });
        }

        data_writer
            .transport_writer
            .add_change(
                change,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await;

        reply_sender.send(Ok(()));
    }

    #[tracing::instrument(skip(self))]
    pub async fn dispose_w_timestamp(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_writer
            .dispose_w_timestamp(
                dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.get_offered_deadline_missed_status().await)
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn enable_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_writer.enabled {
            data_writer.enabled = true;

            let discovered_reader_list: Vec<_> =
                self.domain_participant.discovered_reader_list.to_vec();
            for discovered_reader_data in discovered_reader_list {
                self.add_discovered_reader(
                    discovered_reader_data,
                    publisher_handle,
                    data_writer_handle,
                    participant_address.clone(),
                )
                .await;
            }

            self.announce_data_writer(publisher_handle, data_writer_handle, participant_address)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_data_writer_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => publisher.default_datawriter_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;
        if data_writer.enabled {
            data_writer.qos.check_immutability(&qos)?;
        }
        data_writer.qos = qos;

        if data_writer.enabled {
            self.announce_data_writer(publisher_handle, data_writer_handle, participant_address)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn are_all_changes_acknowledged(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_writer) = publisher
            .data_writer_list
            .iter()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer.are_all_changes_acknowledged().await)
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn read(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let subscriber = if subscriber_handle == self.domain_participant.instance_handle {
            Some(&mut self.domain_participant.builtin_subscriber)
        } else {
            self.domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
        };

        let Some(subscriber) = subscriber else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_reader
            .read(
                max_samples,
                &sample_states,
                &view_states,
                &instance_states,
                specific_instance_handle,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn take(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn read_next_instance(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .read_next_instance(
                max_samples,
                previous_handle,
                &sample_states,
                &view_states,
                &instance_states,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn take_next_instance(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .take_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_subscription_matched_status(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionMatchedStatus> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let status = data_reader.get_subscription_matched_status();
        data_reader
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::SubscriptionMatched,
            })
            .await;
        Ok(status)
    }

    //#[tracing::instrument(skip(self, participant_address))]
    pub fn wait_for_historical_data(
        &mut self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_wait: Duration,
    ) -> Pin<Box<dyn Future<Output = DdsResult<()>> + Send>> {
        let timer_handle = self.timer_handle.clone();
        Box::pin(async move {
            poll_timeout(
                timer_handle,
                max_wait.into(),
                Box::pin(async move {
                    loop {
                        let (reply_sender, reply_receiver) = oneshot();
                        participant_address
                            .send(DcpsDomainParticipantMail::Message(
                                MessageServiceMail::IsHistoricalDataReceived {
                                    subscriber_handle,
                                    data_reader_handle,
                                    reply_sender,
                                },
                            ))
                            .await?;

                        let reply = reply_receiver.await;
                        match reply {
                            Ok(historical_data_received) => match historical_data_received {
                                Ok(true) => return Ok(()),
                                Ok(false) => (),
                                Err(e) => return Err(e),
                            },
                            Err(_) => return Err(DdsError::Error(String::from("Channel error"))),
                        }
                    }
                }),
            )
            .await?
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_publication_data(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            return Err(DdsError::NotEnabled);
        }

        data_reader
            .matched_publication_list
            .iter()
            .find(|x| &x.key().value == publication_handle.as_ref())
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_publications(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_reader.get_matched_publications())
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;
        if data_reader.enabled {
            data_reader.qos.check_immutability(&qos)?
        }

        data_reader.qos = qos;

        if data_reader.enabled {
            self.announce_data_reader(subscriber_handle, data_reader_handle, participant_address)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_reader.qos.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn set_data_reader_listener(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataReaderListener>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        data_reader.listener_sender = listener_sender;
        data_reader.listener_mask = listener_mask;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn is_historical_data_received(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            return Err(DdsError::NotEnabled);
        };

        match data_reader.qos.durability.kind {
            DurabilityQosPolicyKind::Volatile => {
                return Err(DdsError::IllegalOperation);
            }
            DurabilityQosPolicyKind::TransientLocal
            | DurabilityQosPolicyKind::Transient
            | DurabilityQosPolicyKind::Persistent => (),
        };

        if let TransportReaderKind::Stateful(r) = &data_reader.transport_reader {
            Ok(r.is_historical_data_received())
        } else {
            Ok(true)
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn enable_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            data_reader.enabled = true;

            let discovered_writer_list: Vec<_> =
                self.domain_participant.discovered_writer_list.to_vec();
            for discovered_writer_data in discovered_writer_list {
                self.add_discovered_writer(
                    discovered_writer_data,
                    subscriber_handle,
                    data_reader_handle,
                    participant_address.clone(),
                )
                .await;
            }

            self.announce_data_reader(subscriber_handle, data_reader_handle, participant_address)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn announce_participant(
        &mut self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) {
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
                lease_duration: self
                    .domain_participant
                    .qos
                    .discovery_config
                    .participant_lease_duration,
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
                participant_address,
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
                let mut dynamic_data =
                    DynamicDataFactory::create_data(SpdpDiscoveredParticipantData::get_type());
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
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
        // Generate TypeInformation from the data_writer's type
        #[cfg(feature = "type_lookup")]
        let type_information = Some(data_writer.type_support.to_type_information_bytes());
        #[cfg(not(feature = "type_lookup"))]
        let type_information = None;

        let discovered_writer_data = DiscoveredWriterData {
            dds_publication_data,
            writer_proxy,
            type_information,
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
            participant_address,
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
            let mut dynamic_data =
                DynamicDataFactory::create_data(DiscoveredWriterData::get_type());
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
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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

        // For DynamicDataReader, there may be no Topic in the topic_description_list.
        // In that case, use the type_support directly to get the type name.
        let topic = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == data_reader.topic_name);

        let (topic_name, type_name, topic_qos): (String, String, TopicQos) = match topic {
            Some(TopicDescriptionKind::Topic(t)) => {
                (t.topic_name.clone(), t.type_name.clone(), t.qos.clone())
            }
            Some(TopicDescriptionKind::ContentFilteredTopic(t)) => {
                if let Some(TopicDescriptionKind::Topic(topic)) = self
                    .domain_participant
                    .topic_description_list
                    .iter()
                    .find(|x| x.topic_name() == t.related_topic_name)
                {
                    (
                        topic.topic_name.clone(),
                        topic.type_name.clone(),
                        topic.qos.clone(),
                    )
                } else {
                    return;
                }
            }
            None => {
                // DynamicDataReader case: no Topic in list, use type_support directly
                (
                    data_reader.topic_name.clone(),
                    data_reader.type_support.get_name().to_string(),
                    TopicQos::default(),
                )
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
        // Generate TypeInformation from the data_reader's type
        #[cfg(feature = "type_lookup")]
        let type_information = Some(data_reader.type_support.to_type_information_bytes());
        #[cfg(not(feature = "type_lookup"))]
        let type_information = None;

        let discovered_reader_data = DiscoveredReaderData {
            dds_subscription_data,
            reader_proxy,
            type_information,
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
            participant_address,
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
            let mut dynamic_data =
                DynamicDataFactory::create_data(DiscoveredReaderData::get_type());
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
    async fn announce_topic(
        &mut self,
        topic_name: String,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) {
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
            participant_address,
            self.domain_participant.builtin_publisher.instance_handle,
            data_writer_handle,
            discovered_topic_data.create_dynamic_sample(),
            timestamp,
            reply_sender,
        )
        .await;
    }

    #[tracing::instrument(skip(self, participant_address))]
    async fn add_discovered_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) {
        let reader_guid_prefix = discovered_reader_data
            .reader_proxy
            .remote_reader_guid
            .prefix();
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| p.participant_proxy.guid_prefix == reader_guid_prefix)
        {
            tracing::info!(
                "add_discovered_reader: found participant {:?} with default_unicast_locator_list={:?}",
                p.participant_proxy.guid_prefix,
                p.participant_proxy.default_unicast_locator_list
            );
            p.participant_proxy.default_unicast_locator_list.clone()
        } else {
            tracing::warn!(
                "add_discovered_reader: participant not found for reader {:?}, locator fallback empty",
                reader_guid_prefix
            );
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

                    let unicast_count = unicast_locator_list.len();
                    let multicast_count = multicast_locator_list.len();
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
                    if let TransportWriterKind::Stateful(w) = &mut data_writer.transport_writer {
                        w.add_matched_reader(reader_proxy);
                        // Send cached samples to the new reader for TRANSIENT_LOCAL durability
                        // The reader proxy's highest_sent_seq_num is 0, so write_message will
                        // send all cached samples to it. Existing readers won't receive duplicates
                        // because their highest_sent_seq_num is already at the latest.
                        if durability_kind != DurabilityKind::Volatile {
                            let cached_changes = w.cached_changes_count();
                            // Small delay to allow reader to fully initialize before receiving cached data
                            // TODO: Investigate proper synchronization mechanism
                            let mut timer_handle = self.timer_handle.clone();
                            timer_handle
                                .delay(core::time::Duration::from_millis(100))
                                .await;
                            tracing::info!(
                                "TRANSIENT_LOCAL: About to send {} cached changes for topic {:?} (unicast_locators: {}, multicast_locators: {})",
                                cached_changes,
                                data_writer.topic_name,
                                unicast_count,
                                multicast_count
                            );
                            w.write_message(
                                self.transport.message_writer.as_ref(),
                                &self.clock_handle,
                            )
                            .await;
                            tracing::info!(
                                "TRANSIENT_LOCAL: write_message completed for topic {:?}",
                                data_writer.topic_name
                            );
                        }
                    }

                    if data_writer
                        .listener_mask
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let status = data_writer.get_publication_matched_status();
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
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
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
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
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
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
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
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
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
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
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
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

    #[tracing::instrument(skip(self, participant_address))]
    async fn add_discovered_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
            // Look up topic for the reader. For DynamicDataReader, there may be no Topic
            // in the list - that's OK, we fall back to using the reader's type_support directly.
            let matched_topic = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|t| t.topic_name() == data_reader.topic_name);

            // Get the type name from the topic (if available) or from the reader's type_support
            let reader_type_name: String = match matched_topic {
                Some(TopicDescriptionKind::Topic(t)) => t.type_name.clone(),
                Some(TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic)) => {
                    if let Some(TopicDescriptionKind::Topic(matched_topic)) = self
                        .domain_participant
                        .topic_description_list
                        .iter()
                        .find(|t| t.topic_name() == content_filtered_topic.related_topic_name)
                    {
                        matched_topic.type_name.clone()
                    } else {
                        return;
                    }
                }
                None => {
                    // DynamicDataReader case: no Topic, use type_support directly
                    data_reader.type_support.get_name().to_string()
                }
            };
            let reader_topic_name = &data_reader.topic_name;
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
                    if let TransportReaderKind::Stateful(r) = &mut data_reader.transport_reader {
                        r.add_matched_writer(&writer_proxy);
                    }

                    if data_reader
                        .listener_mask
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
    pub async fn add_builtin_participants_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    SpdpDiscoveredParticipantData::get_type(),
                    cache_change.data_value.as_ref(),
                ) {
                    let discovered_participant_data =
                        SpdpDiscoveredParticipantData::create_sample(dynamic_data);

                    self.add_discovered_participant(
                        discovered_participant_data,
                        participant_address,
                    )
                    .await;
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                let discovered_participant_handle = if let Some(h) = cache_change.instance_handle {
                    InstanceHandle::new(h)
                } else if let Ok(dynamic_data) = CdrDeserializer::deserialize(
                    InstanceHandle::get_dynamic_type(),
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

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn add_builtin_publications_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    DiscoveredWriterData::get_type(),
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
                            participant_address.clone(),
                        )
                        .await;
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                let discovered_writer_handle = if let Some(h) = cache_change.instance_handle {
                    InstanceHandle::new(h)
                } else if let Ok(dynamic_data) = CdrDeserializer::deserialize(
                    InstanceHandle::get_dynamic_type(),
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

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn add_builtin_subscriptions_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    DiscoveredReaderData::get_type(),
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
                            participant_address.clone(),
                        )
                        .await;
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                let discovered_reader_handle = if let Some(h) = cache_change.instance_handle {
                    InstanceHandle::new(h)
                } else if let Ok(dynamic_data) = CdrDeserializer::deserialize(
                    InstanceHandle::get_dynamic_type(),
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
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(dynamic_data) = CdrDeserializer::deserialize_builtin(
                    TopicBuiltinTopicData::get_type(),
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

    /// Handle incoming TypeLookup requests.
    ///
    /// When a TypeLookup_Request is received, this method:
    /// 1. Deserializes the request
    /// 2. Looks up the requested types in the type registry
    /// 3. Sends a TypeLookup_Reply with the found TypeObjects
    #[cfg(feature = "type_lookup")]
    #[tracing::instrument(skip(self, _participant_address))]
    pub async fn handle_type_lookup_request(
        &mut self,
        cache_change: CacheChange,
        _participant_address: MpscSender<DcpsDomainParticipantMail>,
    ) {
        use crate::dcps::data_representation_builtin_endpoints::type_lookup::{
            extract_hash_from_type_identifier, ReplyHeader, SampleIdentity, TypeLookupCall,
            TypeLookupGetTypesOut, TypeLookupReply, TypeLookupRequest, TypeLookupReturn,
            TypeIdentifierTypeObjectPairBytes, REMOTE_EX_OK,
        };
        use crate::infrastructure::type_support::TypeSupport;
        use crate::rtps_messages::{
            overall_structure::RtpsMessageWrite,
            submessages::info_timestamp::InfoTimestampSubmessage,
            types::TIME_INVALID,
        };

        // Only process Alive changes
        if cache_change.kind != ChangeKind::Alive {
            eprintln!("[TypeLookup] Ignoring non-Alive change");
            return;
        }
        let data_bytes = cache_change.data_value.as_ref();

        // TypeLookup uses CDR2 (XCDR2), not PL_CDR like SPDP/SEDP
        // Skip non-CDR2 messages (they're discovery traffic being broadcast to all readers)
        if data_bytes.len() < 4 {
            return;
        }
        let rep_id = [data_bytes[0], data_bytes[1]];
        // CDR2_LE = [0x00, 0x07], D_CDR2_LE = [0x00, 0x09], PL_CDR2_LE = [0x00, 0x0b]
        if rep_id != [0x00, 0x07] && rep_id != [0x00, 0x09] && rep_id != [0x00, 0x0b] {
            // Not CDR2 - skip silently (this is likely SPDP/SEDP traffic)
            return;
        }

        // Try to deserialize the TypeLookup_Request
        let Ok(dynamic_data) = CdrDeserializer::deserialize(
            TypeLookupRequest::get_type(),
            data_bytes,
        ) else {
            eprintln!("[TypeLookup] Failed to deserialize TypeLookup_Request");
            return;
        };

        let request = TypeLookupRequest::create_sample(dynamic_data);
        tracing::debug!(
            "Received TypeLookup request from {:?}: {:?}",
            cache_change.writer_guid,
            request
        );

        // Find the discovered participant based on writer_guid prefix
        let writer_prefix = cache_change.writer_guid.prefix();
        let reply_locators = {
            let Some(discovered_participant) = self
                .domain_participant
                .discovered_participant_list
                .iter()
                .find(|p| p.participant_proxy.guid_prefix == writer_prefix)
            else {
                tracing::warn!(
                    "Received TypeLookup request from unknown participant: {:?}",
                    writer_prefix
                );
                return;
            };

            // Clone the metatraffic unicast locators to release the borrow
            let locators = discovered_participant
                .participant_proxy
                .metatraffic_unicast_locator_list
                .clone();
            if locators.is_empty() {
                tracing::warn!("No metatraffic locators for participant {:?}", writer_prefix);
                return;
            }
            locators
        };

        // Build the TypeLookup reply
        // For GetTypes requests, look up types in the registry and return them
        let reply_data = match &request.data {
            TypeLookupCall::GetTypes(get_types_in) => {
                let mut found_types = Vec::new();

                for type_id_bytes in &get_types_in.type_ids {
                    // Extract the hash from the TypeIdentifier
                    if let Some(hash) = extract_hash_from_type_identifier(type_id_bytes) {
                        // Look up the type in our registry
                        if let Some(dynamic_type) = self.domain_participant.lookup_type(&hash) {
                            tracing::debug!(
                                "Found type '{}' for hash {:?}",
                                dynamic_type.get_name(),
                                hash
                            );
                            // Convert DynamicType to TypeObject and serialize
                            let type_object_bytes = match dynamic_type.to_type_object() {
                                Ok(type_object) => type_object.serialize_to_bytes(),
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to convert type '{}' to TypeObject: {:?}",
                                        dynamic_type.get_name(),
                                        e
                                    );
                                    Vec::new()
                                }
                            };
                            tracing::debug!(
                                "Serialized TypeObject for '{}': {} bytes",
                                dynamic_type.get_name(),
                                type_object_bytes.len()
                            );
                            found_types.push(TypeIdentifierTypeObjectPairBytes {
                                type_identifier: type_id_bytes.clone(),
                                type_object: type_object_bytes,
                            });
                        } else {
                            tracing::debug!("Type not found for hash {:?}", hash);
                        }
                    }
                }

                tracing::debug!(
                    "TypeLookup returning {} types for {} requested",
                    found_types.len(),
                    get_types_in.type_ids.len()
                );

                TypeLookupReturn::GetTypes(TypeLookupGetTypesOut {
                    types: found_types,
                    complete_to_minimal: Vec::new(),
                })
            }
            TypeLookupCall::GetTypeDependencies(_) => {
                // Not implemented yet - return empty
                TypeLookupReturn::GetTypeDependencies(
                    crate::dcps::data_representation_builtin_endpoints::type_lookup::TypeLookupGetTypeDependenciesOut::default(),
                )
            }
            TypeLookupCall::Unknown(hash) => {
                tracing::warn!("Unknown TypeLookup operation: 0x{:08x}", hash);
                return;
            }
        };

        let reply = TypeLookupReply {
            header: ReplyHeader {
                related_request_id: SampleIdentity {
                    writer_guid: cache_change.writer_guid,
                    sequence_number: cache_change.sequence_number,
                },
                remote_ex: REMOTE_EX_OK,
            },
            data: reply_data,
        };

        // Serialize the reply using CDR2 (XCDR2) since TypeLookupReply is Final extensibility
        let dynamic_reply = reply.create_dynamic_sample();
        let serialized = if cfg!(target_endian = "big") {
            Cdr2BeSerializer::serialize(&dynamic_reply)
        } else {
            Cdr2LeSerializer::serialize(&dynamic_reply)
        };
        let Ok(serialized_data) = serialized else {
            tracing::warn!("Failed to serialize TypeLookup reply");
            return;
        };

        // Create cache change for the reply
        // Use a simple incrementing sequence number - in a full implementation
        // this would be tracked per-writer
        let guid_prefix = Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix();
        let reply_writer_guid = Guid::new(guid_prefix, ENTITYID_TL_SVC_REPLY_WRITER);

        let timestamp = self.get_current_time();
        let reply_change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: reply_writer_guid,
            sequence_number: 1, // Simple fixed sequence number for stateless replies
            source_timestamp: Some(timestamp.into()),
            instance_handle: None,
            data_value: serialized_data.into(),
        };

        // Build RTPS message with InfoTimestamp and Data submessages
        let info_ts_submessage = reply_change
            .source_timestamp
            .map_or(InfoTimestampSubmessage::new(true, TIME_INVALID), |t| {
                InfoTimestampSubmessage::new(false, t.into())
            });

        let data_submessage = reply_change.as_data_submessage(
            ENTITYID_TL_SVC_REPLY_READER, // Remote reader entity ID
            ENTITYID_TL_SVC_REPLY_WRITER, // Our writer entity ID
        );

        let rtps_message = RtpsMessageWrite::from_submessages(
            &[&info_ts_submessage, &data_submessage],
            guid_prefix,
        );

        // Send the reply to the requester's metatraffic unicast locators
        self.transport
            .message_writer
            .write_message(rtps_message.buffer(), &reply_locators)
            .await;

        tracing::debug!(
            "TypeLookup sent reply to {:?} via {:?}",
            writer_prefix,
            &reply_locators
        );
    }

    /// Handle incoming TypeLookup_Reply messages (CLIENT side)
    ///
    /// When a TypeLookup_Reply is received, this method:
    /// 1. Deserializes the reply
    /// 2. Matches it with pending requests using related_request_id
    /// 3. Extracts the returned TypeObjects and notifies waiting callers
    #[cfg(feature = "type_lookup")]
    #[tracing::instrument(skip(self))]
    pub async fn handle_type_lookup_reply(&mut self, cache_change: CacheChange) {
        use crate::dcps::data_representation_builtin_endpoints::type_lookup::TypeLookupReply;
        use crate::infrastructure::type_support::TypeSupport;

        // Only process Alive changes
        if cache_change.kind != ChangeKind::Alive {
            return;
        }
        let data_bytes = cache_change.data_value.as_ref();

        // TypeLookup uses CDR2 (XCDR2), not PL_CDR like SPDP/SEDP
        // Skip non-CDR2 messages (they're discovery traffic being broadcast to all readers)
        if data_bytes.len() < 4 {
            return;
        }
        let rep_id = [data_bytes[0], data_bytes[1]];
        // CDR2_LE = [0x00, 0x07], D_CDR2_LE = [0x00, 0x09], PL_CDR2_LE = [0x00, 0x0b]
        if rep_id != [0x00, 0x07] && rep_id != [0x00, 0x09] && rep_id != [0x00, 0x0b] {
            // Not CDR2 - skip silently (this is likely SPDP/SEDP traffic)
            return;
        }

        // Try to deserialize the TypeLookup_Reply
        let Ok(dynamic_data) = CdrDeserializer::deserialize(
            TypeLookupReply::get_type(),
            data_bytes,
        ) else {
            tracing::debug!("Failed to deserialize TypeLookup_Reply");
            return;
        };

        let reply = TypeLookupReply::create_sample(dynamic_data);
        tracing::debug!(
            "Received TypeLookup reply from {:?}",
            cache_change.writer_guid,
        );

        // Check if this reply matches a pending request
        // Convert Guid to [u8; 16] for the key since Guid doesn't implement Ord
        let guid_bytes: [u8; 16] = reply.header.related_request_id.writer_guid.into();
        let request_key = (guid_bytes, reply.header.related_request_id.sequence_number);

        if let Some(reply_sender) = self.pending_type_requests.remove(&request_key) {
            // Send the reply to the waiting caller, wrapped in Ok
            reply_sender.send(Ok(reply));
            tracing::debug!("Delivered TypeLookup reply to waiting caller");
        } else {
            tracing::debug!(
                "Received TypeLookup reply for unknown request: {:?}",
                request_key
            );
        }
    }

    /// Send a TypeLookup request to a remote participant (CLIENT side)
    ///
    /// This method:
    /// 1. Finds the target participant's metatraffic locators
    /// 2. Builds and sends a TypeLookup_Request message
    /// 3. Registers the pending request for reply correlation
    ///
    /// The caller will receive the TypeLookup_Reply via the reply_sender when it arrives.
    #[cfg(feature = "type_lookup")]
    #[tracing::instrument(skip(self, reply_sender))]
    pub async fn send_type_lookup_request(
        &mut self,
        target_participant_prefix: [u8; 12],
        type_ids: Vec<Vec<u8>>,
        reply_sender: OneshotSender<
            DdsResult<crate::dcps::data_representation_builtin_endpoints::type_lookup::TypeLookupReply>,
        >,
    ) {
        use crate::dcps::data_representation_builtin_endpoints::type_lookup::{
            RequestHeader, SampleIdentity, TypeLookupCall, TypeLookupGetTypesIn, TypeLookupRequest,
        };
        use crate::infrastructure::type_support::TypeSupport;
        use crate::rtps_messages::{
            overall_structure::RtpsMessageWrite,
            submessages::info_timestamp::InfoTimestampSubmessage,
            types::TIME_INVALID,
        };

        // Find the discovered participant based on target GUID prefix
        let target_locators = {
            let Some(discovered_participant) = self
                .domain_participant
                .discovered_participant_list
                .iter()
                .find(|p| p.participant_proxy.guid_prefix == target_participant_prefix)
            else {
                tracing::warn!(
                    "Target participant not found for TypeLookup request: {:?}",
                    target_participant_prefix
                );
                reply_sender.send(Err(DdsError::PreconditionNotMet(
                    "Target participant not found".into(),
                )));
                return;
            };

            let locators = discovered_participant
                .participant_proxy
                .metatraffic_unicast_locator_list
                .clone();
            if locators.is_empty() {
                tracing::warn!(
                    "No metatraffic locators for participant {:?}",
                    target_participant_prefix
                );
                reply_sender.send(Err(DdsError::PreconditionNotMet(
                    "No metatraffic locators for target participant".into(),
                )));
                return;
            }
            locators
        };

        // Allocate a sequence number for this request
        let sequence_number = self.type_lookup_sequence_number;
        self.type_lookup_sequence_number += 1;

        // Get our request writer GUID
        let guid_prefix = Guid::from(*self.domain_participant.instance_handle.as_ref()).prefix();
        let request_writer_guid = Guid::new(guid_prefix, ENTITYID_TL_SVC_REQ_WRITER);

        // Build the TypeLookup request
        let request = TypeLookupRequest {
            header: RequestHeader {
                request_id: SampleIdentity {
                    writer_guid: request_writer_guid,
                    sequence_number,
                },
                instance_name: String::new(),
            },
            data: TypeLookupCall::GetTypes(TypeLookupGetTypesIn { type_ids }),
        };

        // Serialize the request using CDR2 (XCDR2) since TypeLookupRequest is Final extensibility
        let dynamic_request = request.create_dynamic_sample();
        let serialized = if cfg!(target_endian = "big") {
            Cdr2BeSerializer::serialize(&dynamic_request)
        } else {
            Cdr2LeSerializer::serialize(&dynamic_request)
        };
        let Ok(serialized_data) = serialized else {
            tracing::warn!("Failed to serialize TypeLookup request");
            reply_sender.send(Err(DdsError::Error("Failed to serialize request".into())));
            return;
        };

        // Create cache change for the request
        let timestamp = self.get_current_time();
        let request_change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: request_writer_guid,
            sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: None,
            data_value: serialized_data.into(),
        };

        // Build RTPS message with InfoTimestamp and Data submessages
        let info_ts_submessage = request_change
            .source_timestamp
            .map_or(InfoTimestampSubmessage::new(true, TIME_INVALID), |t| {
                InfoTimestampSubmessage::new(false, t.into())
            });

        let data_submessage = request_change.as_data_submessage(
            ENTITYID_TL_SVC_REQ_READER, // Remote reader entity ID
            ENTITYID_TL_SVC_REQ_WRITER, // Our writer entity ID
        );

        let rtps_message = RtpsMessageWrite::from_submessages(
            &[&info_ts_submessage, &data_submessage],
            guid_prefix,
        );

        // Register the pending request BEFORE sending (so we're ready for the reply)
        let guid_bytes: [u8; 16] = request_writer_guid.into();
        self.pending_type_requests
            .insert((guid_bytes, sequence_number), reply_sender);

        // Send the request to the target participant's metatraffic unicast locators
        self.transport
            .message_writer
            .write_message(rtps_message.buffer(), &target_locators)
            .await;

        tracing::debug!(
            "Sent TypeLookup request to {:?} (seq={}) via {:?}",
            target_participant_prefix,
            sequence_number,
            &target_locators
        );
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn add_cache_change(
        &mut self,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
            // Look up the topic for content filtering. For DynamicDataReader,
            // there may be no Topic in the list - that's OK, just skip filtering.
            let reader_topic = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|t| t.topic_name() == data_reader.topic_name);

            if let Some(TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic)) = reader_topic
            {
                if cache_change.kind == ChangeKind::Alive {
                    let Ok(data) = CdrDeserializer::deserialize(
                        data_reader.type_support.as_ref().clone(),
                        cache_change.data_value.as_ref(),
                    ) else {
                        return;
                    };

                    if let Some((variable_name, _position_value)) = content_filtered_topic
                        .filter_expression
                        .trim()
                        .split_once("=")
                    {
                        let Some(member_id) = data.get_member_id_by_name(variable_name.trim())
                        else {
                            return;
                        };
                        let Ok(member_descriptor) = data.get_descriptor(member_id) else {
                            return;
                        };
                        match member_descriptor.r#type.get_kind() {
                            crate::xtypes::dynamic_type::TypeKind::NONE => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::BOOLEAN => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::BYTE => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::INT16 => todo!(),
                            crate::xtypes::dynamic_type::TypeKind::INT32 => todo!(),
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
                                if member_value != &content_filtered_topic.expression_parameters[0]
                                {
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

            match data_reader.add_reader_change(cache_change, reception_timestamp) {
                Ok(AddChangeResult::Added(change_instance_handle)) => {
                    info!("New change added");
                    if let DurationKind::Finite(deadline_missed_period) =
                        data_reader.qos.deadline.period
                    {
                        let mut timer_handle = self.timer_handle.clone();
                        let participant_address = participant_address.clone();

                        self.spawner_handle.spawn(async move {
                            loop {
                                timer_handle.delay(deadline_missed_period.into()).await;
                                participant_address
                                    .send(DcpsDomainParticipantMail::Event(
                                        EventServiceMail::RequestedDeadlineMissed {
                                            subscriber_handle,
                                            data_reader_handle,
                                            change_instance_handle,
                                            participant_address: participant_address.clone(),
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
                        let Ok(the_subscriber) = self
                            .get_subscriber_async(participant_address.clone(), subscriber_handle)
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
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

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn offered_deadline_missed(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
            let Ok(the_writer) = self.get_data_writer_async(
                participant_address,
                publisher_handle,
                data_writer_handle,
            ) else {
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
            let Ok(the_writer) = self.get_data_writer_async(
                participant_address,
                publisher_handle,
                data_writer_handle,
            ) else {
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
            let Ok(the_writer) = self.get_data_writer_async(
                participant_address,
                publisher_handle,
                data_writer_handle,
            ) else {
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

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn requested_deadline_missed(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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
            let Ok(the_reader) = self.get_data_reader_async(
                participant_address,
                subscriber_handle,
                data_reader_handle,
            ) else {
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
            let Ok(the_reader) = self.get_data_reader_async(
                participant_address,
                subscriber_handle,
                data_reader_handle,
            ) else {
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
            let Ok(the_reader) = self.get_data_reader_async(
                participant_address,
                subscriber_handle,
                data_reader_handle,
            ) else {
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
        participant_address: MpscSender<DcpsDomainParticipantMail>,
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

        // Always update last_seen timestamp for lease expiry tracking (even for already-discovered participants)
        if is_domain_id_matching && is_domain_tag_matching && !is_participant_ignored {
            let handle =
                InstanceHandle::new(discovered_participant_data.dds_participant_data.key.value);
            let now = self.clock_handle.now();
            self.discovered_participant_last_seen.insert(handle, now);
        }

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

            self.announce_participant(participant_address).await;

            self.domain_participant
                .add_discovered_participant(discovered_participant_data);
        }
    }

    /// Remove discovered [domain participant](SpdpDiscoveredParticipantData) with the speficied [handle](InstanceHandle).
    #[tracing::instrument(skip(self))]
    async fn remove_discovered_participant(&mut self, handle: InstanceHandle) {
        // Remove from last_seen tracking
        self.discovered_participant_last_seen.remove(&handle);

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
                        if let TransportReaderKind::Stateful(stateful_reader) =
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
                        if let TransportWriterKind::Stateful(stateful_writer) =
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

    /// Check all discovered participants and remove those whose lease has expired.
    /// Called periodically by the liveliness check timer.
    #[tracing::instrument(skip(self))]
    pub async fn check_participant_liveness(&mut self) {
        let now = self.clock_handle.now();
        let mut expired_handles = Vec::new();

        for (handle, last_seen) in &self.discovered_participant_last_seen {
            // Find the participant's lease_duration
            if let Some(participant) = self
                .domain_participant
                .discovered_participant_list
                .iter()
                .find(|p| InstanceHandle::new(p.dds_participant_data.key().value) == *handle)
            {
                let lease_duration = participant.lease_duration;
                // Time - Time = Duration (implemented in time.rs)
                let elapsed = now - *last_seen;

                if elapsed > lease_duration {
                    tracing::debug!(
                        "Participant {:?} lease expired (elapsed: {:?}, lease: {:?}), removing",
                        handle,
                        elapsed,
                        lease_duration
                    );
                    expired_handles.push(*handle);
                }
            }
        }

        for handle in expired_handles {
            self.remove_discovered_participant(handle).await;
        }
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
                    TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
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
            if let TransportWriterKind::Stateful(w) = &mut dw.transport_writer {
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
                    TransportReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
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
            if let TransportReaderKind::Stateful(r) = &mut dr.transport_reader {
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
                    TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
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
            if let TransportWriterKind::Stateful(w) = &mut dw.transport_writer {
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
                    TransportReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
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
            if let TransportReaderKind::Stateful(r) = &mut dr.transport_reader {
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
                    TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
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
            if let TransportWriterKind::Stateful(w) = &mut dw.transport_writer {
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
                    TransportReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
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
            if let TransportReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
                r.delete_matched_writer(guid);
            }
        }
    }

    pub async fn handle_data(&mut self, data_message: Arc<[u8]>) {
        if let Ok(rtps_message) = RtpsMessageRead::try_from(data_message.as_ref()) {
            let mut message_receiver = MessageReceiver::new(&rtps_message);

            while let Some(submessage) = message_receiver.next() {
                match submessage {
                    RtpsSubmessageReadKind::Data(data_submessage) => {
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
                                    TransportReaderKind::Stateful(r) => {
                                        r.on_data_submessage_received(
                                            data_submessage,
                                            message_receiver.source_guid_prefix(),
                                            message_receiver.source_timestamp(),
                                        )
                                        .await;
                                    }
                                    TransportReaderKind::Stateless(r) => {
                                        r.on_data_submessage_received(
                                            data_submessage,
                                            message_receiver.source_guid_prefix(),
                                            message_receiver.source_timestamp(),
                                        )
                                        .await;
                                    }
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
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
                                    TransportReaderKind::Stateful(r) => {
                                        r.on_data_frag_submessage_received(
                                            data_frag_submessage,
                                            message_receiver.source_guid_prefix(),
                                            message_receiver.source_timestamp(),
                                        )
                                        .await;
                                    }
                                    TransportReaderKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::Gap(gap_submessage) => {
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
                                    TransportReaderKind::Stateful(r) => {
                                        r.on_gap_submessage_received(
                                            gap_submessage,
                                            message_receiver.source_guid_prefix(),
                                        );
                                    }
                                    TransportReaderKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
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
                                    TransportReaderKind::Stateful(r) => {
                                        r.on_heartbeat_submessage_received(
                                            heartbeat_submessage,
                                            message_receiver.source_guid_prefix(),
                                            self.transport.message_writer.as_ref(),
                                        )
                                        .await;
                                    }
                                    TransportReaderKind::Stateless(_) => (),
                                }
                            }
                        }
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
                                    TransportReaderKind::Stateful(r) => {
                                        r.on_heartbeat_frag_submessage_received(
                                            heartbeat_frag_submessage,
                                            message_receiver.source_guid_prefix(),
                                        );
                                    }
                                    TransportReaderKind::Stateless(_) => (),
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
                                    TransportWriterKind::Stateful(w) => {
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
                                        }
                                    }
                                    TransportWriterKind::Stateless(_) => (),
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
                                    TransportWriterKind::Stateful(w) => {
                                        w.on_nack_frag_submessage_received(
                                            nack_frag_submessage,
                                            message_receiver.source_guid_prefix(),
                                            self.transport.message_writer.as_ref(),
                                        )
                                        .await
                                    }
                                    TransportWriterKind::Stateless(_) => (),
                                }
                            }
                        }
                    }
                    _ => (),
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
                    TransportWriterKind::Stateful(writer) => {
                        writer
                            .write_message(
                                self.transport.message_writer.as_ref(),
                                &self.clock_handle,
                            )
                            .await
                    }
                    TransportWriterKind::Stateless(_writer) => {}
                }
            }
        }
    }
}

#[tracing::instrument(skip(type_support))]
fn get_topic_kind(type_support: &DynamicType) -> TopicKind {
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
    /// Type registry for TypeLookup service.
    /// Maps equivalence hash (14 bytes) to serialized TypeObject/DynamicType.
    #[cfg(feature = "type_lookup")]
    type_registry: BTreeMap<[u8; 14], DynamicType>,
}

impl DomainParticipantEntity {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
            #[cfg(feature = "type_lookup")]
            type_registry: BTreeMap::new(),
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

    /// Register a type in the type registry for TypeLookup service.
    /// The hash should be the first 14 bytes of MD5 of the serialized TypeObject.
    #[cfg(feature = "type_lookup")]
    fn register_type(&mut self, hash: [u8; 14], dynamic_type: DynamicType) {
        self.type_registry.insert(hash, dynamic_type);
    }

    /// Look up a type in the type registry by its equivalence hash.
    #[cfg(feature = "type_lookup")]
    fn lookup_type(&self, hash: &[u8; 14]) -> Option<&DynamicType> {
        self.type_registry.get(hash)
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
    type_support: Arc<DynamicType>,
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
        type_support: Arc<DynamicType>,
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

enum TransportWriterKind {
    Stateful(RtpsStatefulWriter),
    Stateless(RtpsStatelessWriter),
}

impl TransportWriterKind {
    fn guid(&self) -> Guid {
        match self {
            TransportWriterKind::Stateful(w) => w.guid(),
            TransportWriterKind::Stateless(w) => w.guid(),
        }
    }

    async fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        clock: &impl Clock,
    ) {
        match self {
            TransportWriterKind::Stateful(w) => {
                w.add_change(cache_change, message_writer, clock).await
            }
            TransportWriterKind::Stateless(w) => w.add_change(cache_change, message_writer).await,
        }
    }

    async fn remove_change(&mut self, sequence_number: i64) {
        match self {
            TransportWriterKind::Stateful(w) => w.remove_change(sequence_number),
            TransportWriterKind::Stateless(w) => w.remove_change(sequence_number),
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
    transport_writer: TransportWriterKind,
    topic_name: String,
    type_name: String,
    type_support: Arc<DynamicType>,
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
    acknowledgement_notification: Option<OneshotSender<()>>,
}

impl DataWriterEntity {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        instance_handle: InstanceHandle,
        transport_writer: TransportWriterKind,
        topic_name: String,
        type_name: String,
        type_support: Arc<DynamicType>,
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

        let instance_handle = get_instance_handle_from_dynamic_data(dynamic_data.clone())?;
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

        dynamic_data.clear_nonkey_values()?;
        let serialized_key = serialize(&dynamic_data, &self.qos.representation)?;

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

        let instance_handle = get_instance_handle_from_dynamic_data(dynamic_data.clone())?;
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

        dynamic_data.clear_nonkey_values()?;
        let serialized_key = serialize(&dynamic_data, &self.qos.representation)?;

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

    async fn are_all_changes_acknowledged(&self) -> bool {
        match &self.transport_writer {
            TransportWriterKind::Stateful(w) => {
                w.is_change_acknowledged(self.last_change_sequence_number)
            }
            TransportWriterKind::Stateless(_) => true,
        }
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

enum TransportReaderKind {
    Stateful(RtpsStatefulReader),
    Stateless(RtpsStatelessReader),
}

impl TransportReaderKind {
    fn guid(&self) -> Guid {
        match self {
            TransportReaderKind::Stateful(r) => r.guid(),
            TransportReaderKind::Stateless(r) => r.guid(),
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
    type_support: Arc<DynamicType>,
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
    transport_reader: TransportReaderKind,
}

impl DataReaderEntity {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        type_support: Arc<DynamicType>,
        status_condition: Actor<DcpsStatusCondition>,
        listener_sender: Option<MpscSender<ListenerMail>>,
        listener_mask: Vec<StatusKind>,
        transport_reader: TransportReaderKind,
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
        let (data_value, instance_handle) = match cache_change.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                let data_value = CdrDeserializer::deserialize(
                    self.type_support.as_ref().clone(),
                    cache_change.data_value.as_ref(),
                )?;
                let instance_handle = get_instance_handle_from_dynamic_data(data_value.clone())?;
                (data_value, instance_handle)
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => match cache_change.instance_handle {
                Some(i) => {
                    let mut key_holder = self.type_support.as_ref().clone();
                    key_holder.clear_nonkey_members();
                    let data_value = DynamicDataFactory::create_data(key_holder);
                    let instance_handle = InstanceHandle::new(i);
                    (data_value, instance_handle)
                }
                None => {
                    let mut key_holder = self.type_support.as_ref().clone();
                    key_holder.clear_nonkey_members();
                    let data_value =
                        CdrDeserializer::deserialize(key_holder, cache_change.data_value.as_ref())?;
                    let instance_handle =
                        get_instance_handle_from_dynamic_data(data_value.clone())?;
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
    dynamic_data: &DynamicData,
    representation: &DataRepresentationQosPolicy,
) -> DdsResult<Vec<u8>> {
    Ok(
        if representation.value.is_empty() || representation.value[0] == XCDR_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                Cdr1BeSerializer::serialize(dynamic_data)?
            } else {
                Cdr1LeSerializer::serialize(dynamic_data)?
            }
        } else if representation.value[0] == XCDR2_DATA_REPRESENTATION {
            if cfg!(target_endian = "big") {
                Cdr2BeSerializer::serialize(dynamic_data)?
            } else {
                Cdr2LeSerializer::serialize(dynamic_data)?
            }
        } else if representation.value[0] == BUILT_IN_DATA_REPRESENTATION {
            RtpsPlCdrSerializer::serialize(dynamic_data)?
        } else {
            panic!("Invalid data representation")
        },
    )
}
