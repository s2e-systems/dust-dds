use std::{cell::RefCell, marker::PhantomData, sync::{atomic, Arc, Mutex, Once}, thread::JoinHandle};

use rust_dds_api::{builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData}, dcps_psm::{DomainId, Duration, InstanceHandle, StatusMask, Time}, dds_type::DDSType, domain::{
        domain_participant::{DomainParticipant, TopicGAT},
        domain_participant_listener::DomainParticipantListener,
    }, infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    }, publication::publisher_listener::PublisherListener, return_type::DDSResult, subscription::subscriber_listener::SubscriberListener, topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener}};
use rust_rtps::{structure::Participant, transport::Transport, types::{Locator, constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, PROTOCOL_VERSION_2_4, VENDOR_ID}}};


use crate::{
    inner::rtps_participant_entities::RtpsParticipantEntities, rtps_publisher::RtpsPublisher,
    rtps_subscriber::RtpsSubscriber, rtps_topic::RtpsTopic,
};

struct SpdpDiscoveredParticipantData {
    value: u8,
}

impl DDSType for SpdpDiscoveredParticipantData {
    fn type_name() -> &'static str {
        "SpdpDiscoveredParticipantData"
    }
    
    fn has_key() -> bool {
        false
    }

    fn key(&self) -> Vec<u8> {
        vec![]
    }

    fn serialize(&self) -> Vec<u8> {
        vec![0,0,0,0,1,2,3,4]
    }

    fn deserialize(_data: Vec<u8>) -> Self {
        todo!()
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
        let participant = Participant::new(guid_prefix, userdata_transport.unicast_locator_list().clone(), userdata_transport.multicast_locator_list().clone(), PROTOCOL_VERSION_2_4, VENDOR_ID);

        let builtin_entities =
            Arc::new(RtpsParticipantEntities::new_builtin(metatraffic_transport));
        let user_defined_entities = Arc::new(RtpsParticipantEntities::new_user_defined(
            userdata_transport,
        ));

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
        let entity_key = [
            0,
            self.publisher_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];

        let qos = qos.unwrap_or(self.get_default_publisher_qos());
        let publisher_ref = self.user_defined_entities.create_publisher(
            self.participant.entity.guid.prefix(),
            entity_key,
            qos,
            a_listener,
            mask,
        )?;
        Some(RtpsPublisher {
            parent_participant: self,
            publisher_ref,
        })
    }

    fn delete_publisher(&self, a_publisher: &Self::PublisherType) -> DDSResult<()> {
        self.user_defined_entities
            .delete_publisher(&a_publisher.publisher_ref)
    }

    fn create_subscriber(
        &'a self,
        qos: Option<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener>>,
        mask: StatusMask,
    ) -> Option<Self::SubscriberType> {
        let entity_key = [
            0,
            self.subscriber_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let qos = qos.unwrap_or(self.get_default_subscriber_qos());
        let subscriber_ref = self.user_defined_entities.create_subscriber(
            self.participant.entity.guid.prefix(),
            entity_key,
            qos,
            a_listener,
            mask,
        )?;
        Some(RtpsSubscriber{parent_participant: self, subscriber_ref})
    }

    fn delete_subscriber(&self, a_subscriber: &Self::SubscriberType) -> DDSResult<()> {
        self.user_defined_entities
            .delete_subscriber(&a_subscriber.subscriber_ref)
    }

    fn create_topic<T: DDSType>(
        &'a self,
        topic_name: &str,
        qos: Option<TopicQos>,
        a_listener: Option<Box<dyn TopicListener<T>>>,
        mask: StatusMask,
    ) -> Option<<Self as TopicGAT<'a, T>>::TopicType> {
        let entity_key = [
            0,
            self.topic_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let qos = qos.unwrap_or(self.get_default_topic_qos());
        qos.is_consistent().ok()?;
        let topic_ref = self.user_defined_entities.create_topic(
            self.participant.entity.guid.prefix(),
            entity_key,
            topic_name,
            qos,
            a_listener,
            mask,
        )?;
        Some(RtpsTopic{parent_participant:self, topic_ref, phantom_data:PhantomData})
    }

    fn delete_topic<T: DDSType>(
        &'a self,
        a_topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> DDSResult<()> {
        self.user_defined_entities.delete_topic(&a_topic.topic_ref)
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
    ) -> Option<Box<dyn TopicDescription<T>>> {
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
        todo!()
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

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
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
            let builtin_publisher = self
                .builtin_entities
                .create_publisher(guid_prefix, [0, 0, 0], PublisherQos::default(), None, 0)
                .expect("Error creating built-in publisher");

            let spdp_topic_qos = TopicQos::default();
            let spdp_topic = self
                .builtin_entities
                .create_topic::<SpdpDiscoveredParticipantData>(
                    guid_prefix,
                    ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER.entity_key(),
                    "SPDP",
                    spdp_topic_qos,
                    None,
                    0,
                )
                .expect("Error creating SPDP topic");

                let mut spdp_announcer_qos = DataWriterQos::default();
                spdp_announcer_qos.reliability.kind = rust_dds_api::infrastructure::qos_policy::ReliabilityQosPolicyKind::BestEffortReliabilityQos;
                let spdp_announcer = builtin_publisher
                    .create_stateless_datawriter::<SpdpDiscoveredParticipantData>(
                        ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER.entity_key(),
                        &spdp_topic,
                        Some(spdp_announcer_qos),
                        None,
                        0
                    )
                    .expect("Error creating SPDP built-in writer");

                let spdp_locator = Locator::new_udpv4(7400, [239, 255, 0, 0]);
                
                spdp_announcer.get().expect("Error retrieven SPDP announcer")
                    .writer()
                    .try_get_stateless()
                    .unwrap()
                    .reader_locator_add(spdp_locator);
                
                spdp_announcer.write_w_timestamp::<SpdpDiscoveredParticipantData>(SpdpDiscoveredParticipantData{value:5}, None, Time{sec:10, nanosec:0}).expect("Error announcing participant");

            //         // let key = BuiltInTopicKey([1, 2, 3]);
            //         // let user_data = UserDataQosPolicy { value: vec![] };
            //         // let dds_participant_data = ParticipantBuiltinTopicData { key, user_data };
            //         // let participant_proxy = self.into();
            //         // let lease_duration = DURATION_INFINITE;

            //         // let data = SpdpDiscoveredParticipantData {
            //         //     dds_participant_data,
            //         //     participant_proxy,
            //         //     lease_duration,
            //         // };

            //         // spdp_announcer_anywriter_ref
            //         //     .get_as::<SpdpDiscoveredParticipantData>()
            //         //     .unwrap()
            //         //     .write_w_timestamp(data, None, TIME_INVALID)
            //         //     .ok();

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
