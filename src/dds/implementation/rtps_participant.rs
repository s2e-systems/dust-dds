use std::{
    cell::RefCell,
    sync::{atomic, Arc, Mutex, Once},
    thread::JoinHandle,
};

use crate::{
    builtin_topics::ParticipantBuiltinTopicData,
    dds::infrastructure::{
        qos::{DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
        qos_policy::UserDataQosPolicy,
    },
    discovery::types::{ParticipantProxy, SpdpDiscoveredParticipantData},
    rtps::{
        behavior::endpoint_traits::CacheChangeSender,
        endpoint_types::BuiltInEndpointSet,
        message_sender::RtpsMessageSender,
        structure::Participant,
        transport::Transport,
        types::{
            constants::{
                ENTITY_KIND_BUILT_IN_READER_GROUP,
                ENTITY_KIND_BUILT_IN_WRITER_GROUP, ENTITY_KIND_USER_DEFINED_READER_GROUP,
                ENTITY_KIND_USER_DEFINED_UNKNOWN, ENTITY_KIND_USER_DEFINED_WRITER_GROUP,
                PROTOCOL_VERSION_2_4, VENDOR_ID,
            },
            EntityId, GuidPrefix, Locator, GUID,
        },
    },
    types::{
        BuiltInTopicKey, DDSType, DomainId, ReturnCode, ReturnCodes, DURATION_INFINITE,
        TIME_INVALID,
    },
    utils::maybe_valid::MaybeValidList,
};

use super::{
    rtps_publisher::{RtpsPublisher, RtpsPublisherRef},
    rtps_subscriber::{RtpsSubscriber, RtpsSubscriberRef},
    rtps_topic::{AnyRtpsTopic, RtpsTopic, RtpsAnyTopicRef},
};

enum EntityType {
    BuiltIn,
    UserDefined,
}

struct RtpsParticipantEntities {
    transport: Box<dyn Transport>,
    publisher_list: MaybeValidList<Box<RtpsPublisher>>,
    publisher_count: atomic::AtomicU8,
    subscriber_list: MaybeValidList<Box<RtpsSubscriber>>,
    subscriber_count: atomic::AtomicU8,
    topic_list: MaybeValidList<Arc<dyn AnyRtpsTopic>>,
    topic_count: atomic::AtomicU8,
}

impl RtpsParticipantEntities {
    fn new(transport: impl Transport) -> Self {
        Self {
            transport: Box::new(transport),
            publisher_list: Default::default(),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_list: Default::default(),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_list: Default::default(),
            topic_count: atomic::AtomicU8::new(0),
        }
    }

    fn create_publisher(
        &self,
        qos: PublisherQos,
        guid_prefix: GuidPrefix,
        entity_type: EntityType,
    ) -> Option<RtpsPublisherRef> {
        let entity_key = [
            0,
            self.publisher_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match entity_type {
            EntityType::BuiltIn => ENTITY_KIND_BUILT_IN_WRITER_GROUP,
            EntityType::UserDefined => ENTITY_KIND_USER_DEFINED_WRITER_GROUP,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_publisher_guid = GUID::new(guid_prefix, entity_id);
        let new_publisher = Box::new(RtpsPublisher::new(new_publisher_guid, qos, None, 0));
        self.publisher_list.add(new_publisher)
    }

    fn delete_publisher(&self, a_publisher: &RtpsPublisherRef) -> ReturnCode<()> {
        let rtps_publisher = a_publisher.get()?;
        if rtps_publisher.writer_list.is_empty() {
            if self.publisher_list.contains(&a_publisher) {
                a_publisher.delete();
                Ok(())
            } else {
                Err(ReturnCodes::PreconditionNotMet(
                    "Publisher not found in this participant",
                ))
            }
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Publisher still contains data writers",
            ))
        }
    }

    fn create_subscriber(
        &self,
        qos: SubscriberQos,
        guid_prefix: GuidPrefix,
        entity_type: EntityType, // _a_listener: impl SubscriberListener,
                                 // _mask: StatusMask
    ) -> Option<RtpsSubscriberRef> {
        let entity_key = [
            0,
            self.subscriber_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match entity_type {
            EntityType::BuiltIn => ENTITY_KIND_BUILT_IN_READER_GROUP,
            EntityType::UserDefined => ENTITY_KIND_USER_DEFINED_READER_GROUP,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_subscriber_guid = GUID::new(guid_prefix, entity_id);
        let new_subscriber = Box::new(RtpsSubscriber::new(new_subscriber_guid, qos, None, 0));

        self.subscriber_list.add(new_subscriber)
    }

    fn delete_subscriber(&self, a_subscriber: &RtpsSubscriberRef) -> ReturnCode<()> {
        let rtps_subscriber = a_subscriber.get()?;
        if rtps_subscriber.reader_list.is_empty() {
            if self.subscriber_list.contains(&a_subscriber) {
                a_subscriber.delete();
                Ok(())
            } else {
                Err(ReturnCodes::PreconditionNotMet(
                    "Subscriber not found in this participant",
                ))
            }
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Subscriber still contains data readers",
            ))
        }
    }

    fn create_topic<T: DDSType>(
        &self,
        guid_prefix: GuidPrefix,
        topic_name: &str,
        qos: TopicQos,
        // _a_listener: impl TopicListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyTopicRef> {
        qos.is_consistent().ok()?;
        let entity_key = [
            0,
            self.topic_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, ENTITY_KIND_USER_DEFINED_UNKNOWN);
        let new_topic_guid = GUID::new(guid_prefix, entity_id);
        let new_topic: Arc<RtpsTopic<T>> = Arc::new(RtpsTopic::new(
            new_topic_guid,
            topic_name.clone().into(),
            qos,
            None,
            0,
        ));
        self.topic_list.add(new_topic)
    }

    fn delete_topic<T: DDSType>(&self, a_topic: &RtpsAnyTopicRef) -> ReturnCode<()> {
        if self.topic_list.contains(&a_topic) {
            a_topic.delete()
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Topic not found in this participant",
            ))
        }
    }

    fn send_data(&self) {
        for publisher in self.publisher_list.into_iter() {
            if let Some(publisher) = publisher.get().ok() {
                for writer in publisher.writer_list.into_iter() {
                    if let Some(writer) = writer.get().ok() {
                        let mut writer_flavor = writer.writer();
                        println!(
                            "last_change_sequence_number = {:?}",
                            writer_flavor.last_change_sequence_number
                        );
                        let destined_messages = writer_flavor.produce_messages();
                        let participant_guid_prefix = writer_flavor.endpoint.entity.guid.prefix();
                        RtpsMessageSender::send_cache_change_messages(
                            participant_guid_prefix,
                            self.transport.as_ref(),
                            destined_messages,
                        );
                    }
                }
            }
        }
    }
}

pub struct RtpsParticipant {
    participant: Participant,
    qos: Mutex<DomainParticipantQos>,
    default_publisher_qos: Mutex<PublisherQos>,
    default_subscriber_qos: Mutex<SubscriberQos>,
    default_topic_qos: Mutex<TopicQos>,
    builtin_entities: Arc<RtpsParticipantEntities>,
    user_defined_entities: Arc<RtpsParticipantEntities>,
    enabled: Arc<atomic::AtomicBool>,
    enabled_function: Once,
    thread_list: RefCell<Vec<JoinHandle<()>>>,
}

impl Into<ParticipantProxy> for &RtpsParticipant {
    fn into(self) -> ParticipantProxy {
        ParticipantProxy {
            domain_id: self.participant.domain_id,
            domain_tag: "".to_string(),
            protocol_version: self.participant.protocol_version,
            guid_prefix: self.participant.entity.guid.prefix(),
            vendor_id: self.participant.vendor_id,
            expects_inline_qos: true,
            available_built_in_endpoints: BuiltInEndpointSet { value: 9 },
            // built_in_endpoint_qos:
            metatraffic_unicast_locator_list: self
                .builtin_entities
                .transport
                .unicast_locator_list()
                .clone(),
            metatraffic_multicast_locator_list: self
                .builtin_entities
                .transport
                .multicast_locator_list()
                .clone(),
            default_unicast_locator_list: vec![],
            default_multicast_locator_list: vec![],
            manual_liveliness_count: 8,
        }
    }
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        userdata_transport: impl Transport,
        metatraffic_transport: impl Transport,
        //     a_listener: impl DomainParticipantListener,
        //     mask: StatusMask,
    ) -> Self {
        let guid_prefix = [1; 12];
        let participant = Participant::new(guid_prefix, domain_id, PROTOCOL_VERSION_2_4, VENDOR_ID);

        let builtin_entities = Arc::new(RtpsParticipantEntities::new(metatraffic_transport));
        let user_defined_entities = Arc::new(RtpsParticipantEntities::new(userdata_transport));

        RtpsParticipant {
            participant,
            qos: Mutex::new(qos),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            builtin_entities,
            user_defined_entities,
            enabled: Arc::new(atomic::AtomicBool::new(false)),
            enabled_function: Once::new(),
            thread_list: RefCell::new(Vec::new()),
        }
    }

    pub fn create_publisher(&self, qos: Option<PublisherQos>) -> Option<RtpsPublisherRef> {
        let guid_prefix = self.participant.entity.guid.prefix();
        let qos = qos.unwrap_or_default();
        self.user_defined_entities
            .create_publisher(qos, guid_prefix, EntityType::UserDefined)
    }

    pub fn delete_publisher(&self, a_publisher: &RtpsPublisherRef) -> ReturnCode<()> {
        self.user_defined_entities.delete_publisher(a_publisher)
    }

    pub fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        // _a_listener: impl SubscriberListener,
        // _mask: StatusMask
    ) -> Option<RtpsSubscriberRef> {
        let guid_prefix = self.participant.entity.guid.prefix();
        let qos = qos.unwrap_or_default();
        self.user_defined_entities
            .create_subscriber(qos, guid_prefix, EntityType::UserDefined)
    }

    pub fn delete_subscriber(&self, a_subscriber: &RtpsSubscriberRef) -> ReturnCode<()> {
        self.user_defined_entities.delete_subscriber(a_subscriber)
    }

    pub fn create_topic<T: DDSType>(
        &self,
        topic_name: &str,
        qos: Option<TopicQos>,
        // _a_listener: impl TopicListener<T>,
        // _mask: StatusMask
    ) -> Option<RtpsAnyTopicRef> {
        let guid_prefix = self.participant.entity.guid.prefix();
        let qos = qos.unwrap_or_default();
        qos.is_consistent().ok()?;
        self.user_defined_entities
            .create_topic::<T>(guid_prefix, topic_name, qos)
    }

    pub fn delete_topic<T: DDSType>(&self, a_topic: &RtpsAnyTopicRef) -> ReturnCode<()> {
        self.user_defined_entities.delete_topic::<T>(a_topic)
    }

    pub fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        *self.default_publisher_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.lock().unwrap().clone()
    }

    pub fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        *self.default_subscriber_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.lock().unwrap().clone()
    }

    pub fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.default_topic_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.lock().unwrap().clone()
    }

    pub fn set_qos(&self, qos: Option<DomainParticipantQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        *self.qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> ReturnCode<DomainParticipantQos> {
        Ok(self.qos.lock().unwrap().clone())
    }

    pub fn get_domain_id(&self) -> DomainId {
        self.participant.domain_id
    }

    pub fn enable(&self) -> ReturnCode<()> {
        self.enabled_function.call_once(||{
            let guid_prefix = self.participant.entity.guid.prefix();
            let builtin_publisher_ref = self
                .builtin_entities
                .create_publisher(PublisherQos::default(), guid_prefix, EntityType::BuiltIn)
                .expect("Error creating built-in publisher");
            let builtin_publisher = builtin_publisher_ref.get().expect("Error retrieving built-in publisher");

            let spdp_topic_qos = TopicQos::default();
            let spdp_topic = self.builtin_entities
                .create_topic::<SpdpDiscoveredParticipantData>(guid_prefix, "SPDP", spdp_topic_qos)
                .expect("Error creating SPDP topic");

            let mut spdp_announcer_qos = DataWriterQos::default();
            spdp_announcer_qos.reliability.kind = crate::dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffortReliabilityQos;
            let spdp_announcer_anywriter_ref = builtin_publisher
                .create_stateless_builtin_datawriter::<SpdpDiscoveredParticipantData>(
                    &spdp_topic,
                    Some(spdp_announcer_qos),
                )
                .expect("Error creating SPDP built-in writer");

            let spdp_locator = Locator::new_udpv4(7400, [239, 255, 0, 0]);
            let spdp_announcer = spdp_announcer_anywriter_ref.get().expect("Error retrieving SPDP announcer");
            spdp_announcer
                .writer()
                .try_get_stateless()
                .unwrap()
                .reader_locator_add(spdp_locator);

            let key = BuiltInTopicKey([1, 2, 3]);
            let user_data = UserDataQosPolicy { value: vec![] };
            let dds_participant_data = ParticipantBuiltinTopicData { key, user_data };
            let participant_proxy = self.into();
            let lease_duration = DURATION_INFINITE;

            let data = SpdpDiscoveredParticipantData {
                dds_participant_data,
                participant_proxy,
                lease_duration,
            };

            spdp_announcer_anywriter_ref
                .get_as::<SpdpDiscoveredParticipantData>()
                .unwrap()
                .write_w_timestamp(data, None, TIME_INVALID)
                .ok();

            let mut thread_list = self.thread_list.borrow_mut();
            let enabled = self.enabled.clone();
            let builtin_entities = self.builtin_entities.clone();
            self.enabled.store(true, atomic::Ordering::Release);
            thread_list.push(std::thread::spawn(move || {
                while enabled.load(atomic::Ordering::Acquire) {
                    builtin_entities.send_data();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }));
            
        });

        Ok(())
    }

    pub fn get_builtin_publisher(&self) -> Option<RtpsPublisherRef> {
        self.builtin_entities.publisher_list.into_iter().find(|x| {
            if let Some(publisher) = x.get().ok() {
                publisher.group.entity.guid.entity_id().entity_kind()
                    == ENTITY_KIND_BUILT_IN_WRITER_GROUP
            } else {
                false
            }
        })
    }

    pub fn get_builtin_subscriber(&self) -> Option<RtpsSubscriberRef> {
        self.builtin_entities.subscriber_list.into_iter().find(|x| {
            if let Some(subscriber) = x.get().ok() {
                subscriber.group.entity.guid.entity_id().entity_kind()
                    == ENTITY_KIND_BUILT_IN_READER_GROUP
            } else {
                false
            }
        })
    }
}

impl Drop for RtpsParticipant {
    fn drop(&mut self) {
        self.enabled.store(false, atomic::Ordering::Release);
        for thread in self.thread_list.borrow_mut().drain(..) {
            thread.join().ok();
        }
    }
}
