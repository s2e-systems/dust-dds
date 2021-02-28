use std::{marker::PhantomData, ops::Deref, sync::Mutex};

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
use rust_rtps::transport::Transport;

use crate::{
    impls::{
        domain_participant_impl::DomainParticipantImpl, publisher_impl::PublisherImpl,
        subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
    },
    utils::node::Node,
};

pub struct RtpsPublisher<'a>(<Self as Deref>::Target);

impl<'a> Deref for RtpsPublisher<'a> {
    type Target = Node<&'a RtpsDomainParticipant, PublisherImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsSubscriber<'a>(<Self as Deref>::Target);

impl<'a> Deref for RtpsSubscriber<'a> {
    type Target = Node<&'a RtpsDomainParticipant, SubscriberImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsTopic<'a, T: DDSType>(<Self as Deref>::Target);

impl<'a, T: DDSType> Deref for RtpsTopic<'a, T> {
    type Target = Node<(&'a RtpsDomainParticipant, PhantomData<&'a T>), TopicImpl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsDomainParticipant(Mutex<DomainParticipantImpl>);

impl RtpsDomainParticipant {
    pub fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        userdata_transport: impl Transport,
        metatraffic_transport: impl Transport,
        a_listener: Option<Box<dyn DomainParticipantListener>>,
        mask: StatusMask,
    ) -> Self {
        Self(Mutex::new(DomainParticipantImpl::new(
            domain_id,
            qos,
            userdata_transport,
            metatraffic_transport,
            a_listener,
            mask,
        )))
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
        let impl_ref = self
            .0
            .lock()
            .unwrap()
            .create_publisher(qos, a_listener, mask)
            .ok()?;

        Some(RtpsPublisher(Node {
            parent: self,
            impl_ref,
        }))
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
        if std::ptr::eq(a_publisher.0.parent, self) {
            self.0
                .lock()
                .unwrap()
                .delete_publisher(&a_publisher.impl_ref)
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
        let impl_ref = self
            .0
            .lock()
            .unwrap()
            .create_subscriber(qos, a_listener, mask)
            .ok()?;

        Some(RtpsSubscriber(Node {
            parent: self,
            impl_ref,
        }))
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
        if std::ptr::eq(a_subscriber.parent, self) {
            self.0
                .lock()
                .unwrap()
                .delete_subscriber(&a_subscriber.impl_ref)
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
        let impl_ref = self
            .0
            .lock()
            .unwrap()
            .create_topic::<T>(topic_name, qos, a_listener, mask)
            .ok()?;
        Some(RtpsTopic(Node {
            parent: (self, PhantomData),
            impl_ref,
        }))
    }

    fn delete_topic<T: DDSType>(
        &'a self,
        a_topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> DDSResult<()> {
        if std::ptr::eq(a_topic.parent.0, self) {
            self.0.lock().unwrap().delete_topic(&a_topic.impl_ref)
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
        // self.domain_id
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> DDSResult<()> {
        self.0.lock().unwrap().set_default_publisher_qos(qos)
    }

    fn get_default_publisher_qos(&self) -> PublisherQos {
        self.0.lock().unwrap().get_default_publisher_qos()
    }

    fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        self.0.lock().unwrap().set_default_subscriber_qos(qos)
    }

    fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.0.lock().unwrap().get_default_subscriber_qos()
    }

    fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> DDSResult<()> {
        self.0.lock().unwrap().set_default_topic_qos(qos)
    }

    fn get_default_topic_qos(&self) -> TopicQos {
        self.0.lock().unwrap().get_default_topic_qos()
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
        self.0.lock().unwrap().set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self.0.lock().unwrap().get_qos())
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
        // self.enabled_function.call_once(|| {
        //     use rust_rtps::structure::Entity;
        //     let guid_prefix = self.guid().prefix();
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

        //     let mut thread_list = self.thread_list.borrow_mut();
        //     let enabled = self.enabled.clone();
        //     let builtin_entities = self.builtin_entities.clone();
        //     self.enabled.store(true, atomic::Ordering::Release);
        //     thread_list.push(std::thread::spawn(move || {
        //         while enabled.load(atomic::Ordering::Acquire) {
        //             builtin_entities.send_data(guid_prefix);
        //             std::thread::sleep(std::time::Duration::from_secs(1));
        //         }
        //     }));
        // });

        // Ok(())
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}
