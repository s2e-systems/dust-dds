use std::{
    cell::RefCell,
    sync::{atomic, Arc, Mutex},
    thread::JoinHandle,
};

use crate::{
    dds::infrastructure::qos::{
        DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos,
    },
    discovery::types::SpdpDiscoveredParticipantData,
    rtps::{
        behavior::endpoint_traits::CacheChangeSender,
        message_sender::RtpsMessageSender,
        structure::Participant,
        transport::Transport,
        types::{
            constants::{
                ENTITY_KIND_BUILT_IN_READER_GROUP, ENTITY_KIND_BUILT_IN_WRITER_GROUP,
                ENTITY_KIND_USER_DEFINED_READER_GROUP, ENTITY_KIND_USER_DEFINED_UNKNOWN,
                ENTITY_KIND_USER_DEFINED_WRITER_GROUP, PROTOCOL_VERSION_2_4, VENDOR_ID,
            },
            EntityId, GuidPrefix, GUID,
        },
    },
    types::{DDSType, DomainId, ReturnCode, ReturnCodes},
    utils::maybe_valid::MaybeValidList,
};

use super::{
    rtps_publisher::{RtpsPublisher, RtpsPublisherRef},
    rtps_subscriber::{RtpsSubscriber, RtpsSubscriberRef},
    rtps_topic::{AnyRtpsTopic, RtpsTopic, RtpsTopicRef},
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
        let rtps_publisher = a_publisher.value()?;
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
        let rtps_subscriber = a_subscriber.value()?;
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
    ) -> Option<RtpsTopicRef> {
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

    fn delete_topic<T: DDSType>(&self, a_topic: &RtpsTopicRef) -> ReturnCode<()> {
        let rtps_topic = a_topic.value()?;
        if self.topic_list.contains(&a_topic) {
            if Arc::strong_count(rtps_topic) == 1 {
                // discovery.remove_topic(a_topic.value()?)?;
                a_topic.delete();
                Ok(())
            } else {
                Err(ReturnCodes::PreconditionNotMet(
                    "Topic still attached to some data reader or data writer",
                ))
            }
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Topic not found in this participant",
            ))
        }
    }

    fn send_data(&self) {
        for publisher in self.publisher_list.iter() {
            if let Some(publisher) = publisher.read().unwrap().get() {
                for writer in publisher.writer_list.iter() {
                    if let Some(writer) = writer.read().unwrap().get() {
                        let mut writer_flavor = writer.writer().lock().unwrap();
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
    thread_list: RefCell<Vec<JoinHandle<()>>>,
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

        let builtin_publisher = builtin_entities
            .create_publisher(PublisherQos::default(), guid_prefix, EntityType::BuiltIn)
            .expect("Error creating built-in publisher");

        let spdp_topic_qos = TopicQos::default();
        let spdp_topic = builtin_entities
            .create_topic::<SpdpDiscoveredParticipantData>(guid_prefix, "SPDP", spdp_topic_qos)
            .expect("Error creating SPDP topic");

        let mut spdp_announcer_qos = DataWriterQos::default();
        spdp_announcer_qos.reliability.kind = crate::dds::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffortReliabilityQos;
        builtin_publisher
            .get()
            .expect("Error retrieving built-in publisher")
            .create_stateless_builtin_datawriter::<SpdpDiscoveredParticipantData>(
                &spdp_topic,
                Some(spdp_announcer_qos),
            )
            .expect("Error creating SPDP built-in writer");

        RtpsParticipant {
            participant,
            qos: Mutex::new(qos),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            builtin_entities: builtin_entities.clone(), // Clone because entities have been created already so move is not possible
            user_defined_entities,
            enabled: Arc::new(atomic::AtomicBool::new(false)),
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
    ) -> Option<RtpsTopicRef> {
        let guid_prefix = self.participant.entity.guid.prefix();
        let qos = qos.unwrap_or_default();
        qos.is_consistent().ok()?;
        self.user_defined_entities
            .create_topic::<T>(guid_prefix, topic_name, qos)
    }

    pub fn delete_topic<T: DDSType>(&self, a_topic: &RtpsTopicRef) -> ReturnCode<()> {
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
        // self.builtin_entities.publisher_list[0].

        // let key = BuiltInTopicKey([1, 2, 3]);
        // let user_data = UserDataQosPolicy { value: vec![] };
        // let dds_participant_data = ParticipantBuiltinTopicData { key, user_data };
        // let participant_proxy = proxy_from_rtp_participant(&self.builtin_participant);
        // let lease_duration = DURATION_INFINITE;

        // let data = SpdpDiscoveredParticipantData {
        //     dds_participant_data,
        //     participant_proxy,
        //     lease_duration,
        // };

        // rtps_writer.write_w_timestamp(data, None, TIME_INVALID).ok();

        if self.enabled.load(atomic::Ordering::Acquire) == false {
            self.enabled.store(true, atomic::Ordering::Release);

            let mut thread_list = self.thread_list.borrow_mut();
            let enabled = self.enabled.clone();
            let builtin_entities = self.builtin_entities.clone();
            thread_list.push(std::thread::spawn(move || {
                while enabled.load(atomic::Ordering::Acquire) {
                    builtin_entities.send_data();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }));
        }

        Ok(())
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
