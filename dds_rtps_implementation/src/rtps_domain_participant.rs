use std::{
    cell::RefCell,
    marker::PhantomData,
    ops::Deref,
    sync::{atomic, Arc, Mutex, Once, Weak},
    thread::JoinHandle,
};

use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time},
    dds_type::DDSType,
    domain::{
        domain_participant::{DomainParticipant, TopicGAT},
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publication::publisher_listener::PublisherListener,
    return_type::{DDSError, DDSResult},
    subscription::subscriber_listener::SubscriberListener,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps::{
    structure::Participant,
    transport::Transport,
    types::{
        constants::{
            ENTITY_KIND_USER_DEFINED_READER_GROUP, ENTITY_KIND_USER_DEFINED_UNKNOWN,
            PROTOCOL_VERSION_2_4, VENDOR_ID,
        },
        EntityId, GuidPrefix, GUID,
    },
};

use crate::{
    impls::{
        rtps_publisher_impl::RtpsPublisherImpl, rtps_subscriber_impl::RtpsSubscriberImpl,
        rtps_topic_impl::RtpsTopicImpl,
    },
    utils::node::Node,
};

pub struct RtpsPublisher<'a>(<Self as Deref>::Target);

impl<'a> Deref for RtpsPublisher<'a> {
    type Target = Node<&'a RtpsDomainParticipant, RtpsPublisherImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsSubscriber<'a>(<Self as Deref>::Target);

impl<'a> Deref for RtpsSubscriber<'a> {
    type Target = Node<&'a RtpsDomainParticipant, RtpsSubscriberImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsTopic<'a, T: DDSType>(<Self as Deref>::Target);

impl<'a, T: DDSType> Deref for RtpsTopic<'a, T> {
    type Target = Node<(&'a RtpsDomainParticipant, PhantomData<&'a T>), RtpsTopicImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct RtpsParticipantEntities {
    publisher_list: Mutex<Vec<Arc<Mutex<RtpsPublisherImpl>>>>,
    subscriber_list: Mutex<Vec<Arc<Mutex<RtpsSubscriberImpl>>>>,
    topic_list: Mutex<Vec<Arc<Mutex<RtpsTopicImpl>>>>,
    transport: Box<dyn Transport>,
}

impl RtpsParticipantEntities {
    fn new(transport: impl Transport) -> Self {
        Self {
            publisher_list: Default::default(),
            subscriber_list: Default::default(),
            topic_list: Default::default(),
            transport: Box::new(transport),
        }
    }

    pub fn send_data(&self, _guid_prefix: GuidPrefix) {
        let publisher_list = self.publisher_list.lock().unwrap();
        for _publisher in publisher_list.iter() {
            // publisher.send_data(self.transport.as_ref());
            todo!()
        }
    }
}

pub struct RtpsDomainParticipant {
    domain_id: DomainId,
    participant: Participant,
    qos: Mutex<DomainParticipantQos>,
    publisher_count: atomic::AtomicU8,
    subscriber_count: atomic::AtomicU8,
    topic_count: atomic::AtomicU8,
    default_publisher_qos: Mutex<PublisherQos>,
    default_subscriber_qos: Mutex<SubscriberQos>,
    default_topic_qos: Mutex<TopicQos>,
    builtin_entities: Arc<RtpsParticipantEntities>,
    user_defined_entities: Arc<RtpsParticipantEntities>,
    enabled: Arc<atomic::AtomicBool>,
    enabled_function: Once,
    thread_list: RefCell<Vec<JoinHandle<()>>>,
    a_listener: Option<Box<dyn DomainParticipantListener>>,
    mask: StatusMask,
}

impl RtpsDomainParticipant {
    pub fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        userdata_transport: impl Transport,
        metatraffic_transport: impl Transport,
        a_listener: Option<Box<dyn DomainParticipantListener>>,
        mask: StatusMask,
    ) -> Self {
        let guid_prefix = [1; 12];
        let participant = Participant::new(
            guid_prefix,
            userdata_transport.unicast_locator_list().clone(),
            userdata_transport.multicast_locator_list().clone(),
            PROTOCOL_VERSION_2_4,
            VENDOR_ID,
        );

        let builtin_entities = Arc::new(RtpsParticipantEntities::new(metatraffic_transport));
        let user_defined_entities = Arc::new(RtpsParticipantEntities::new(userdata_transport));

        RtpsDomainParticipant {
            domain_id,
            participant,
            qos: Mutex::new(qos),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_count: atomic::AtomicU8::new(0),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            builtin_entities,
            user_defined_entities,
            enabled: Arc::new(atomic::AtomicBool::new(false)),
            enabled_function: Once::new(),
            thread_list: RefCell::new(Vec::new()),
            a_listener,
            mask,
        }
    }
}

impl<'a, T: DDSType> TopicGAT<'a, T> for RtpsDomainParticipant {
    type TopicType = RtpsTopic<'a, T>;
}

impl<'a> DomainParticipant<'a> for RtpsDomainParticipant {
    type PublisherType = RtpsPublisher<'a>;
    type SubscriberType = RtpsSubscriber<'a>;

    fn create_publisher(
        &'a self,
        qos: Option<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener>>,
        mask: StatusMask,
    ) -> Option<Self::PublisherType> {
        let guid_prefix = self.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.publisher_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = ENTITY_KIND_USER_DEFINED_READER_GROUP;
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        let group = rust_rtps::structure::Group::new(guid);
        let qos = qos.unwrap_or(self.get_default_publisher_qos());
        let publisher = Arc::new(Mutex::new(RtpsPublisherImpl::new(
            group, qos, a_listener, mask,
        )));

        self.user_defined_entities
            .publisher_list
            .lock()
            .unwrap()
            .push(publisher.clone());

        Some(RtpsPublisher(Node {
            parent: self,
            impl_ref: Arc::downgrade(&publisher),
        }))
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
        if std::ptr::eq(a_publisher.0.parent, self) {
            let publisher_impl = a_publisher
                .0
                .impl_ref
                .upgrade()
                .ok_or(DDSError::AlreadyDeleted)?;
            if publisher_impl.lock().unwrap().writer_list().is_empty() {
                self.user_defined_entities
                    .publisher_list
                    .lock()
                    .unwrap()
                    .retain(|x| !Arc::ptr_eq(x, &publisher_impl));
                Ok(())
            } else {
                Err(DDSError::PreconditionNotMet(
                    "Publisher still contains data writers",
                ))
            }
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }

    fn create_subscriber(
        &'a self,
        qos: Option<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener>>,
        mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        let guid_prefix = self.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.subscriber_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = ENTITY_KIND_USER_DEFINED_READER_GROUP;
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        let group = rust_rtps::structure::Group::new(guid);
        let qos = qos.unwrap_or(self.get_default_subscriber_qos());
        let subscriber = Arc::new(Mutex::new(RtpsSubscriberImpl::new(
            group, qos, a_listener, mask,
        )));

        self.user_defined_entities
            .subscriber_list
            .lock()
            .unwrap()
            .push(subscriber.clone());

        Some(RtpsSubscriber(Node {
            parent: self,
            impl_ref: Arc::downgrade(&subscriber),
        }))
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
        if std::ptr::eq(a_subscriber.parent, self) {
            let subscriber_impl = a_subscriber
                .impl_ref
                .upgrade()
                .ok_or(DDSError::AlreadyDeleted)?;
            if subscriber_impl.lock().unwrap().reader_list().is_empty() {
                self.user_defined_entities
                    .subscriber_list
                    .lock()
                    .unwrap()
                    .retain(|x| !Arc::ptr_eq(x, &subscriber_impl));
                Ok(())
            } else {
                Err(DDSError::PreconditionNotMet(
                    "Subscriber still contains data readers",
                ))
            }
        } else {
            Err(DDSError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant",
            ))
        }
    }

    fn create_topic<T: DDSType>(
        &'a self,
        topic_name: &str,
        qos: Option<TopicQos>,
        a_listener: Option<Box<dyn TopicListener>>,
        mask: StatusMask,
    ) -> Option<<Self as TopicGAT<'a, T>>::TopicType> {
        let guid_prefix = self.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.topic_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = ENTITY_KIND_USER_DEFINED_UNKNOWN;
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        let entity = rust_rtps::structure::Entity::new(guid);
        let qos = qos.unwrap_or(self.get_default_topic_qos());
        let topic = Arc::new(Mutex::new(RtpsTopicImpl::new(
            entity,
            topic_name,
            T::type_name(),
            qos,
            a_listener,
            mask,
        )));

        self.user_defined_entities
            .topic_list
            .lock()
            .unwrap()
            .push(topic.clone());

        Some(RtpsTopic(Node {
            parent: (self, PhantomData),
            impl_ref: Arc::downgrade(&topic),
        }))
    }

    fn delete_topic<T: DDSType>(
        &'a self,
        a_topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> DDSResult<()> {
        if std::ptr::eq(a_topic.parent.0, self) {
            a_topic.impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?; // Just to check if already deleted
            if Weak::strong_count(&a_topic.impl_ref) == 1 {
                self.user_defined_entities
                    .topic_list
                    .lock()
                    .unwrap()
                    .retain(|x| !Weak::ptr_eq(&Arc::downgrade(x), &a_topic.impl_ref));
                Ok(())
            } else {
                Err(DDSError::PreconditionNotMet(
                    "Topic still attached to some data reader or data writer",
                ))
            }
        } else {
            Err(DDSError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant",
            ))
        }
    }

    fn find_topic<T: DDSType>(
        &self,
        _topic_name: &str,
        _timeout: Duration,
    ) -> Option<<Self as TopicGAT<'a, T>>::TopicType> {
        todo!()
    }

    fn lookup_topicdescription<T: DDSType>(
        &self,
        _name: &str,
    ) -> Option<Box<dyn TopicDescription>> {
        todo!()
    }

    fn get_builtin_subscriber(&self) -> Self::SubscriberType {
        todo!()
        //     self.builtin_entities
        //         .subscriber_list()
        //         .into_iter()
        //         .find(|x| {
        //             if let Some(subscriber) = x.get().ok() {
        //                 subscriber.group.entity.guid.entity_id().entity_kind()
        //                     == ENTITY_KIND_BUILT_IN_READER_GROUP
        //             } else {
        //                 false
        //             }
        //         })
        // }
    }

    fn ignore_participant(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_topic(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_publication(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn ignore_subscription(&self, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn get_domain_id(&self) -> DomainId {
        self.domain_id
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        *self.default_publisher_qos.lock().unwrap() = qos;
        Ok(())
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.lock().unwrap().clone()
    }

    fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        *self.default_subscriber_qos.lock().unwrap() = qos;
        Ok(())
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.lock().unwrap().clone()
    }

    fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.default_topic_qos.lock().unwrap() = qos;
        Ok(())
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.lock().unwrap().clone()
    }

    fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> DDSResult<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn contains_entity(&self, _a_handle: InstanceHandle) -> bool {
        todo!()
    }

    fn get_current_time(&self) -> DDSResult<Time> {
        todo!()
    }
}

impl Entity for RtpsDomainParticipant {
    type Qos = DomainParticipantQos;
    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        *self.qos.lock().unwrap() = qos;
        Ok(())
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self.qos.lock().unwrap().clone())
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        self.enabled_function.call_once(|| {
            let guid_prefix = self.participant.entity.guid.prefix();
            //     let builtin_publisher = RtpsPublisherInner::new_builtin(
            //         guid_prefix,
            //         [0, 0, 0],
            //         PublisherQos::default(),
            //         None,
            //         0,
            //     );

            //     let spdp_topic_qos = TopicQos::default();
            //     let spdp_topic = Arc::new(RtpsTopicInner::new(
            //         guid_prefix,
            //         ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER.entity_key(),
            //         "SPDP".to_string(),
            //         SpdpDiscoveredParticipantData::type_name(),
            //         rust_rtps::types::TopicKind::WithKey,
            //         spdp_topic_qos,
            //         None,
            //         0,
            //     ));
            //     // // let _spdp_topic_ref = self
            //     // //     .builtin_entities
            //     // //     .topic_list
            //     // //     .add(spdp_topic.clone())
            //     // //     .expect("Error creating SPDP topic");

            //     let spdp_unicast_locator_list = self.builtin_entities.transport.unicast_locator_list().clone();
            //     let spdp_multicast_locator_list = self.builtin_entities.transport.multicast_locator_list().clone();
            //     let spdp_resend_period = rust_rtps::behavior::types::Duration::from_secs(30);
            //     let spdp_reader_locators = vec![ReaderLocator::new(Locator::new_udpv4(7400, [239, 255, 0, 0]))];

            //     let mut spdp_announcer_qos = DataWriterQos::default();
            //     spdp_announcer_qos.reliability.kind = rust_dds_api::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffortReliabilityQos;
            //     let spdp_announcer = RtpsDataWriterImpl::new::<SpdpDiscoveredParticipantData>(RtpsWriterFlavor::Stateless(SPDPbuiltinParticipantWriter::new(guid_prefix, spdp_unicast_locator_list, spdp_multicast_locator_list, spdp_resend_period, spdp_reader_locators)), &spdp_topic, spdp_announcer_qos, None, 0);

            //     {
            //         let spdp_announcer_ref = builtin_publisher.writer_list().add(spdp_announcer).expect("Error adding SPDP writer to built_in publisher");
            //         spdp_announcer_ref.write_w_timestamp::<SpdpDiscoveredParticipantData>(SpdpDiscoveredParticipantData{value:5}, None, Time{sec:10, nanosec:0}).expect("Error announcing participant");
            //     }

            //     self
            //         .builtin_entities
            //         .publisher_list
            //         .add(Box::new(builtin_publisher))
            //         .expect("Error creating built-in publisher");

            let mut thread_list = self.thread_list.borrow_mut();
            let enabled = self.enabled.clone();
            let builtin_entities = self.builtin_entities.clone();
            self.enabled.store(true, atomic::Ordering::Release);
            thread_list.push(std::thread::spawn(move || {
                while enabled.load(atomic::Ordering::Acquire) {
                    builtin_entities.send_data(guid_prefix);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }));
        });

        Ok(())
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

impl<'a> Drop for RtpsDomainParticipant {
    fn drop(&mut self) {
        self.enabled.store(false, atomic::Ordering::Release);
        for thread in self.thread_list.borrow_mut().drain(..) {
            thread.join().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::types::Locator;

    use super::*;

    struct TestType;

    impl DDSType for TestType {
        fn type_name() -> &'static str {
            "TestType"
        }

        fn has_key() -> bool {
            todo!()
        }

        fn key(&self) -> Vec<u8> {
            todo!()
        }

        fn serialize(&self) -> Vec<u8> {
            todo!()
        }

        fn deserialize(_data: Vec<u8>) -> Self {
            todo!()
        }
    }

    #[derive(Default)]
    struct MockTransport {
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    }

    impl Transport for MockTransport {
        fn write(
            &self,
            _message: rust_rtps::messages::RtpsMessage,
            _destination_locator: &rust_rtps::types::Locator,
        ) {
            todo!()
        }

        fn read(
            &self,
        ) -> rust_rtps::transport::TransportResult<
            Option<(rust_rtps::messages::RtpsMessage, rust_rtps::types::Locator)>,
        > {
            todo!()
        }

        fn unicast_locator_list(&self) -> &Vec<rust_rtps::types::Locator> {
            &self.unicast_locator_list
        }

        fn multicast_locator_list(&self) -> &Vec<rust_rtps::types::Locator> {
            &self.multicast_locator_list
        }
    }

    #[test]
    fn create_publisher() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let qos = Some(PublisherQos::default());
        let a_listener = None;
        let mask = 0;
        participant
            .create_publisher(qos, a_listener, mask)
            .expect("Error creating publisher");

        assert_eq!(
            participant
                .user_defined_entities
                .publisher_list
                .lock()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn create_delete_publisher() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let qos = Some(PublisherQos::default());
        let a_listener = None;
        let mask = 0;
        let a_publisher = participant.create_publisher(qos, a_listener, mask).unwrap();

        participant
            .delete_publisher(&a_publisher)
            .expect("Error deleting publisher");
        assert_eq!(
            participant
                .user_defined_entities
                .publisher_list
                .lock()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn create_subscriber() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let qos = Some(SubscriberQos::default());
        let a_listener = None;
        let mask = 0;
        participant
            .create_subscriber(qos, a_listener, mask)
            .expect("Error creating subscriber");
        assert_eq!(
            participant
                .user_defined_entities
                .subscriber_list
                .lock()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn create_delete_subscriber() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let qos = Some(SubscriberQos::default());
        let a_listener = None;
        let mask = 0;
        let a_subscriber = participant
            .create_subscriber(qos, a_listener, mask)
            .unwrap();

        participant
            .delete_subscriber(&a_subscriber)
            .expect("Error deleting subscriber");
        assert_eq!(
            participant
                .user_defined_entities
                .subscriber_list
                .lock()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn create_topic() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let topic_name = "Test";
        let qos = Some(TopicQos::default());
        let a_listener = None;
        let mask = 0;
        participant
            .create_topic::<TestType>(topic_name, qos, a_listener, mask)
            .expect("Error creating topic");
    }

    #[test]
    fn create_delete_topic() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let topic_name = "Test";
        let qos = Some(TopicQos::default());
        let a_listener = None;
        let mask = 0;
        let a_topic = participant
            .create_topic::<TestType>(topic_name, qos, a_listener, mask)
            .unwrap();

        participant
            .delete_topic(&a_topic)
            .expect("Error deleting topic")
    }

    #[test]
    fn set_get_default_publisher_qos() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut publisher_qos = PublisherQos::default();
        publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_publisher_qos(Some(publisher_qos.clone()))
            .expect("Error setting default publisher qos");

        assert_eq!(publisher_qos, participant.get_default_publisher_qos())
    }

    #[test]
    fn set_get_default_subscriber_qos() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut subscriber_qos = SubscriberQos::default();
        subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_subscriber_qos(Some(subscriber_qos.clone()))
            .expect("Error setting default subscriber qos");

        assert_eq!(subscriber_qos, participant.get_default_subscriber_qos())
    }

    #[test]
    fn set_get_default_topic_qos() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut topic_qos = TopicQos::default();
        topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_topic_qos(Some(topic_qos.clone()))
            .expect("Error setting default subscriber qos");

        assert_eq!(topic_qos, participant.get_default_topic_qos())
    }

    #[test]
    fn set_default_publisher_qos_to_default_value() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut publisher_qos = PublisherQos::default();
        publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_publisher_qos(Some(publisher_qos.clone()))
            .unwrap();

        participant
            .set_default_publisher_qos(None)
            .expect("Error setting default publisher qos");

        assert_eq!(
            PublisherQos::default(),
            participant.get_default_publisher_qos()
        )
    }

    #[test]
    fn set_default_subscriber_qos_to_default_value() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut subscriber_qos = SubscriberQos::default();
        subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_subscriber_qos(Some(subscriber_qos.clone()))
            .unwrap();

        participant
            .set_default_subscriber_qos(None)
            .expect("Error setting default subscriber qos");

        assert_eq!(
            SubscriberQos::default(),
            participant.get_default_subscriber_qos()
        )
    }

    #[test]
    fn set_default_topic_qos_to_default_value() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut topic_qos = TopicQos::default();
        topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_topic_qos(Some(topic_qos.clone()))
            .unwrap();

        participant
            .set_default_topic_qos(None)
            .expect("Error setting default subscriber qos");

        assert_eq!(TopicQos::default(), participant.get_default_topic_qos())
    }

    #[test]
    fn create_publisher_factory_default_qos() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut publisher_qos = PublisherQos::default();
        publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_publisher_qos(Some(publisher_qos.clone()))
            .unwrap();

        let qos = None;
        let a_listener = None;
        let mask = 0;
        let publisher = participant
            .create_publisher(qos, a_listener, mask)
            .expect("Error creating publisher");

        assert_eq!(publisher.get_qos().unwrap(), publisher_qos);
    }

    #[test]
    fn create_subscriber_factory_default_qos() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut subscriber_qos = SubscriberQos::default();
        subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_subscriber_qos(Some(subscriber_qos.clone()))
            .unwrap();

        let qos = None;
        let a_listener = None;
        let mask = 0;
        let subscriber = participant
            .create_subscriber(qos, a_listener, mask)
            .expect("Error creating publisher");

        assert_eq!(subscriber.get_qos().unwrap(), subscriber_qos);
    }

    #[test]
    fn create_topic_factory_default_qos() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        let mut topic_qos = TopicQos::default();
        topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
        participant
            .set_default_topic_qos(Some(topic_qos.clone()))
            .unwrap();

        let qos = None;
        let a_listener = None;
        let mask = 0;
        let topic = participant
            .create_topic::<TestType>("name", qos, a_listener, mask)
            .expect("Error creating publisher");

        assert_eq!(topic.get_qos().unwrap(), topic_qos);
    }

    #[test]
    fn get_domain_id() {
        let participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );

        assert_eq!(participant.get_domain_id(), 0);
    }
}
