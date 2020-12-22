use crate::builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData};
use crate::dds_infrastructure::qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos};
use crate::dds_rtps_implementation::rtps_object::RtpsObjectList;
use crate::dds_rtps_implementation::rtps_publisher::{RtpsPublisher, RtpsPublisherInner};
use crate::dds_rtps_implementation::rtps_subscriber::{RtpsSubscriber, RtpsSubscriberInner};
use crate::dds_rtps_implementation::rtps_topic::{RtpsTopic, RtpsTopicInner};
use crate::dds_rtps_implementation::discovery::sedp::SimpleEndpointDiscoveryProtocol;
use crate::rtps::structure::Participant;
use crate::rtps::transport::Transport;
use crate::rtps::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
use crate::rtps::types::{EntityId, EntityKind, GUID};
use crate::types::{DomainId, Duration, InstanceHandle, ReturnCode, ReturnCodes, Time, TopicKind};
use std::cell::RefCell;
use std::sync::{atomic, Arc, Mutex};
use std::thread::JoinHandle;

pub struct RtpsParticipantInner {
    participant: Participant,
    qos: Mutex<DomainParticipantQos>,
    default_publisher_qos: Mutex<PublisherQos>,
    default_subscriber_qos: Mutex<SubscriberQos>,
    default_topic_qos: Mutex<TopicQos>,
    userdata_transport: Box<dyn Transport>,
    metatraffic_transport: Box<dyn Transport>,
    publisher_list: RtpsObjectList<RtpsPublisherInner>,
    publisher_count: atomic::AtomicU8,
    subscriber_list: RtpsObjectList<RtpsSubscriberInner>,
    subscriber_count: atomic::AtomicU8,
    topic_list: RtpsObjectList<Arc<RtpsTopicInner>>,
    topic_count: atomic::AtomicU8,
    enabled: atomic::AtomicBool,
    sedp: SimpleEndpointDiscoveryProtocol,
}

pub struct RtpsParticipant {
    inner: Arc<RtpsParticipantInner>,
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
    ) -> Option<Self> {
        // let domain_tag = "".to_string();
        // let lease_duration = Duration {
        //     sec: 30,
        //     nanosec: 0,
        // };
        let guid_prefix = [1; 12];
        let participant = Participant::new(guid_prefix, domain_id, PROTOCOL_VERSION_2_4, VENDOR_ID);
        let sedp = SimpleEndpointDiscoveryProtocol::new(guid_prefix);

        let inner = Arc::new(RtpsParticipantInner {
            participant,
            qos: Mutex::new(qos),
            default_publisher_qos: Mutex::new(PublisherQos::default()),
            default_subscriber_qos: Mutex::new(SubscriberQos::default()),
            default_topic_qos: Mutex::new(TopicQos::default()),
            userdata_transport: Box::new(userdata_transport),
            metatraffic_transport: Box::new(metatraffic_transport),
            publisher_list: Default::default(),
            publisher_count: atomic::AtomicU8::new(0),
            subscriber_list: Default::default(),
            subscriber_count: atomic::AtomicU8::new(0),
            topic_list: Default::default(),
            topic_count: atomic::AtomicU8::new(0),
            enabled: atomic::AtomicBool::new(false),
            sedp,
        });

        let thread_list = RefCell::new(Vec::new());
        Some(Self { inner, thread_list })
    }

    pub fn create_publisher<'a>(&'a self, qos: Option<PublisherQos>) -> Option<RtpsPublisher<'a>> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.inner
                .publisher_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedWriterGroup);
        let new_publisher_guid = GUID::new(guid_prefix, entity_id);
        let new_publisher_qos = qos.unwrap_or(self.get_default_publisher_qos());
        let new_publisher = RtpsPublisherInner::new(new_publisher_guid, new_publisher_qos);
        self.inner.publisher_list.add(new_publisher)
    }

    pub fn delete_publisher(&self, a_publisher: &RtpsPublisher) -> ReturnCode<()> {
        if a_publisher.value()?.writer_list.is_empty() {
            if self.inner.publisher_list.contains(a_publisher) {
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

    pub fn create_subscriber(&self, qos: Option<SubscriberQos>) -> Option<RtpsSubscriber> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.inner
                .subscriber_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedReaderGroup);
        let new_subscriber_guid = GUID::new(guid_prefix, entity_id);
        let new_subscriber_qos = qos.unwrap_or(self.get_default_subscriber_qos());
        let new_subscriber = RtpsSubscriberInner::new(new_subscriber_guid, new_subscriber_qos);
        self.inner.subscriber_list.add(new_subscriber)
    }

    pub fn delete_subscriber(&self, a_subscriber: &RtpsSubscriber) -> ReturnCode<()> {
        if a_subscriber.value()?.reader_list.is_empty() {
            if self.inner.subscriber_list.contains(a_subscriber) {
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

    pub fn create_topic(
        &self,
        topic_name: String,
        type_name: &'static str,
        topic_kind: TopicKind,
        qos: Option<TopicQos>,
    ) -> Option<RtpsTopic> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [
            0,
            self.inner
                .topic_count
                .fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedUnknown);
        let new_topic_guid = GUID::new(guid_prefix, entity_id);
        let new_topic_qos = qos.unwrap_or(self.get_default_topic_qos());
        let new_topic = Arc::new(RtpsTopicInner::new(
            new_topic_guid,
            topic_name,
            type_name,
            topic_kind,
            new_topic_qos,
        ));
        self.inner.topic_list.add(new_topic)
    }

    pub fn delete_topic(&self, a_topic: &RtpsTopic) -> ReturnCode<()> {
        if self.inner.topic_list.contains(a_topic) {
            if Arc::strong_count(a_topic.value()?) == 1 {
                Ok(a_topic.delete())
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

    pub fn find_topic(&self, _topic_name: String, _timeout: Duration) -> Option<RtpsTopic> {
        todo!()
    }

    pub fn lookup_topicdescription(&self, _name: &str) -> Option<RtpsTopic> {
        todo!()
    }

    pub fn get_builtin_subscriber(&self) -> RtpsSubscriber {
        todo!()
    }

    pub fn ignore_participant(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    pub fn ignore_topic(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    pub fn ignore_publication(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    pub fn ignore_subscription(&self, _handle: InstanceHandle) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_domain_id(&self) -> DomainId {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn assert_liveliness(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        *self.inner.default_publisher_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        self.inner.default_publisher_qos.lock().unwrap().clone()
    }

    pub fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        *self.inner.default_subscriber_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.inner.default_subscriber_qos.lock().unwrap().clone()
    }

    pub fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.inner.default_topic_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> TopicQos {
        self.inner.default_topic_qos.lock().unwrap().clone()
    }

    pub fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle],
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_discovered_topics(&self, _topic_handles: &mut [InstanceHandle]) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn contains_entity(&self, _a_handle: InstanceHandle) -> bool {
        todo!()
    }

    pub fn get_current_time(&self) -> ReturnCode<Time> {
        todo!()
    }

    pub fn set_qos(&self, _qos: DomainParticipantQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_qos(&self) -> ReturnCode<DomainParticipantQos> {
        Ok(self.inner.qos.lock().unwrap().clone())
    }

    pub fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        Ok(self.inner.participant.entity.guid.into())
    }

    pub fn enable(&self) -> ReturnCode<()> {
        if self.inner.enabled.load(atomic::Ordering::Acquire) == false {
            self.inner.enabled.store(true, atomic::Ordering::Release);

            let mut thread_list = self.thread_list.borrow_mut();
            let participant_inner = self.inner.clone();
            thread_list.push(std::thread::spawn(move || {
                while participant_inner.enabled.load(atomic::Ordering::Acquire) {
                    println!("{:?}", participant_inner.participant.entity.guid);
                    std::thread::sleep(std::time::Duration::from_secs(1))
                }
            }));
        }

        Ok(())
    }

    pub fn get_endpoint_discovery(&self) -> &SimpleEndpointDiscoveryProtocol {
        &self.inner.sedp
    }
}

impl Drop for RtpsParticipant {
    fn drop(&mut self) {
        self.inner.enabled.store(false, atomic::Ordering::Release);
        for thread in self.thread_list.borrow_mut().drain(..) {
            thread.join().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::messages::RtpsMessage;
    use crate::rtps::transport::TransportResult;
    use crate::rtps::types::Locator;
    use crate::types::{Duration, TopicKind};

    struct MockTransport;
    impl Transport for MockTransport {
        fn write(&self, _message: RtpsMessage, _destination_locator: &Locator) {
            todo!()
        }

        fn read(&self) -> TransportResult<Option<(RtpsMessage, Locator)>> {
            todo!()
        }

        fn unicast_locator_list(&self) -> &Vec<Locator> {
            todo!()
        }

        fn multicast_locator_list(&self) -> &Vec<Locator> {
            todo!()
        }
    }

    #[test]
    fn create_publisher() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();

        let publisher1_default_qos = participant.create_publisher(None).unwrap();
        let publisher1_instance_handle = publisher1_default_qos.get_instance_handle().unwrap();

        let mut qos = PublisherQos::default();
        qos.partition.name = "Test".to_string();
        let publisher2_custom_qos = participant.create_publisher(Some(qos.clone())).unwrap();
        let publisher2_instance_handle = publisher2_custom_qos.get_instance_handle().unwrap();

        // Test correct qos and instance handle
        assert_eq!(
            publisher1_default_qos.get_qos().unwrap(),
            PublisherQos::default()
        );
        assert_eq!(
            publisher1_instance_handle[0..12],
            participant.inner.participant.entity.guid.prefix()
        );
        assert_eq!(publisher1_instance_handle[12..15], [0, 0, 0]);
        assert_eq!(publisher1_instance_handle[15], 0x08);

        assert_eq!(publisher2_custom_qos.get_qos().unwrap(), qos);
        assert_eq!(
            publisher2_instance_handle[0..12],
            participant.inner.participant.entity.guid.prefix()
        );
        assert_eq!(publisher2_instance_handle[12..15], [0, 1, 0]);
        assert_eq!(publisher2_instance_handle[15], 0x08);
    }

    #[test]
    fn create_subscriber() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();

        let subscriber1_default_qos = participant.create_subscriber(None).unwrap();
        let subscriber1_instance_handle = subscriber1_default_qos.get_instance_handle().unwrap();

        let mut qos = SubscriberQos::default();
        qos.partition.name = "Test".to_string();
        let subscriber2_custom_qos = participant.create_subscriber(Some(qos.clone())).unwrap();
        let subscriber2_instance_handle = subscriber2_custom_qos.get_instance_handle().unwrap();

        // Test correct qos and instance handle
        assert_eq!(
            subscriber1_default_qos.get_qos().unwrap(),
            SubscriberQos::default()
        );
        assert_eq!(
            subscriber1_instance_handle[0..12],
            participant.inner.participant.entity.guid.prefix()
        );
        assert_eq!(subscriber1_instance_handle[12..15], [0, 0, 0]);
        assert_eq!(subscriber1_instance_handle[15], 0x09);

        assert_eq!(subscriber2_custom_qos.get_qos().unwrap(), qos);
        assert_eq!(
            subscriber2_instance_handle[0..12],
            participant.inner.participant.entity.guid.prefix()
        );
        assert_eq!(subscriber2_instance_handle[12..15], [0, 1, 0]);
        assert_eq!(subscriber2_instance_handle[15], 0x09);
    }

    #[test]
    fn create_topic() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();

        let topic1_default_qos = participant
            .create_topic("abc".to_string(), "TestType", TopicKind::WithKey, None)
            .unwrap();
        let topic1_instance_handle = topic1_default_qos.get_instance_handle().unwrap();

        let mut qos = TopicQos::default();
        qos.deadline.period = Duration {
            sec: 1,
            nanosec: 10,
        };
        let topic2_custom_qos = participant
            .create_topic(
                "def".to_string(),
                "TestType2",
                TopicKind::WithKey,
                Some(qos.clone()),
            )
            .unwrap();
        let topic2_instance_handle = topic2_custom_qos.get_instance_handle().unwrap();

        // Test correct qos and instance handle
        assert_eq!(topic1_default_qos.get_qos().unwrap(), TopicQos::default());
        assert_eq!(
            topic1_instance_handle[0..12],
            participant.inner.participant.entity.guid.prefix()
        );
        assert_eq!(topic1_instance_handle[12..15], [0, 0, 0]);
        assert_eq!(topic1_instance_handle[15], 0x00);

        assert_eq!(topic2_custom_qos.get_qos().unwrap(), qos);
        assert_eq!(
            topic2_instance_handle[0..12],
            participant.inner.participant.entity.guid.prefix()
        );
        assert_eq!(topic2_instance_handle[12..15], [0, 1, 0]);
        assert_eq!(topic2_instance_handle[15], 0x00);
    }

    #[test]
    fn create_delete_publisher() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();

        let publisher = participant.create_publisher(None).unwrap();

        assert_eq!(participant.delete_publisher(&publisher), Ok(()));
        assert_eq!(publisher.get_qos(), Err(ReturnCodes::AlreadyDeleted));
        assert_eq!(
            participant.delete_publisher(&publisher),
            Err(ReturnCodes::AlreadyDeleted)
        );
    }

    #[test]
    fn create_delete_subscriber() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();

        let subscriber = participant.create_subscriber(None).unwrap();
        
        assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
        assert_eq!(subscriber.get_qos(), Err(ReturnCodes::AlreadyDeleted));
        assert_eq!(
            participant.delete_subscriber(&subscriber),
            Err(ReturnCodes::AlreadyDeleted)
        );
    }

    #[test]
    fn create_delete_topic() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();

        let topic = participant
            .create_topic("abc".to_string(), "TestType", TopicKind::WithKey, None)
            .unwrap();

        assert_eq!(participant.delete_topic(&topic), Ok(()));
        assert_eq!(topic.get_qos(), Err(ReturnCodes::AlreadyDeleted));
        assert_eq!(
            participant.delete_topic(&topic),
            Err(ReturnCodes::AlreadyDeleted)
        );
    }

    #[test]
    fn not_allowed_to_delete_publisher_from_different_participant() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let other_participant = RtpsParticipant::new(
            1,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let publisher = participant.create_publisher(None).unwrap();
        assert_eq!(
            other_participant.delete_publisher(&publisher),
            Err(ReturnCodes::PreconditionNotMet(
                "Publisher not found in this participant"
            ))
        );
    }

    #[test]
    fn not_allowed_to_delete_subscriber_from_different_participant() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let other_participant = RtpsParticipant::new(
            1,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let subscriber = participant.create_subscriber(None).unwrap();
        assert_eq!(
            other_participant.delete_subscriber(&subscriber),
            Err(ReturnCodes::PreconditionNotMet(
                "Subscriber not found in this participant"
            ))
        );
    }

    #[test]
    fn not_allowed_to_delete_topic_from_different_participant() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let other_participant = RtpsParticipant::new(
            1,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let topic = participant
            .create_topic("abc".to_string(), "TestType", TopicKind::WithKey, None)
            .unwrap();
        assert_eq!(
            other_participant.delete_topic(&topic),
            Err(ReturnCodes::PreconditionNotMet(
                "Topic not found in this participant"
            ))
        );
    }

    #[test]
    fn not_allowed_to_delete_publisher_with_writer() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let writer_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let publisher = participant.create_publisher(None).unwrap();
        let _a_datawriter = publisher.create_datawriter(&writer_topic, None, participant.get_endpoint_discovery()).unwrap();

        assert_eq!(
            participant.delete_publisher(&publisher),
            Err(ReturnCodes::PreconditionNotMet(
                "Publisher still contains data writers"
            ))
        );
    }

    #[test]
    fn not_allowed_to_delete_subscriber_with_reader() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let reader_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let subscriber = participant.create_subscriber(None).unwrap();
        let _a_datareader = subscriber.create_datareader(&reader_topic, None, participant.get_endpoint_discovery()).unwrap();

        assert_eq!(
            participant.delete_subscriber(&subscriber),
            Err(ReturnCodes::PreconditionNotMet(
                "Subscriber still contains data readers"
            ))
        );
    }

    #[test]
    fn not_allowed_to_delete_topic_attached_to_reader() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let reader_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let subscriber = participant.create_subscriber(None).unwrap();
        let _a_datareader = subscriber.create_datareader(&reader_topic, None, participant.get_endpoint_discovery()).unwrap();

        assert_eq!(
            participant.delete_topic(&reader_topic),
            Err(ReturnCodes::PreconditionNotMet(
                "Topic still attached to some data reader or data writer"
            ))
        );
    }

    #[test]
    fn not_allowed_to_delete_topic_attached_to_writer() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let writer_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let publisher = participant.create_publisher(None).unwrap();
        let _a_datawriter = publisher.create_datawriter(&writer_topic, None, participant.get_endpoint_discovery()).unwrap();

        assert_eq!(
            participant.delete_topic(&writer_topic),
            Err(ReturnCodes::PreconditionNotMet(
                "Topic still attached to some data reader or data writer"
            ))
        );
    }

    #[test]
    fn allowed_to_delete_publisher_with_created_and_deleted_writer() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let writer_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let publisher = participant.create_publisher(None).unwrap();
        let a_datawriter = publisher.create_datawriter(&writer_topic, None, participant.get_endpoint_discovery()).unwrap();
        publisher
            .delete_datawriter(&a_datawriter, participant.get_endpoint_discovery())
            .expect("Failed to delete datawriter");
        assert_eq!(participant.delete_publisher(&publisher), Ok(()));
    }

    #[test]
    fn allowed_to_delete_subscriber_with_created_and_deleted_reader() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let reader_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let subscriber = participant.create_subscriber(None).unwrap();
        let a_datareader = subscriber.create_datareader(&reader_topic, None, participant.get_endpoint_discovery()).unwrap();
        subscriber
            .delete_datareader(&a_datareader, participant.get_endpoint_discovery())
            .expect("Failed to delete datareader");
        assert_eq!(participant.delete_subscriber(&subscriber), Ok(()));
    }

    #[test]
    fn allowed_to_delete_topic_with_created_and_deleted_writer() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let writer_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let publisher = participant.create_publisher(None).unwrap();
        let a_datawriter = publisher.create_datawriter(&writer_topic, None, participant.get_endpoint_discovery()).unwrap();
        publisher
            .delete_datawriter(&a_datawriter, participant.get_endpoint_discovery())
            .expect("Failed to delete datawriter");
        assert_eq!(participant.delete_topic(&writer_topic), Ok(()));
    }

    #[test]
    fn allowed_to_delete_topic_with_created_and_deleted_reader() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        let reader_topic = participant
            .create_topic("Test".to_string(), "TestType", TopicKind::WithKey, None)
            .expect("Error creating topic");
        let subscriber = participant.create_subscriber(None).unwrap();
        let a_datareader = subscriber.create_datareader(&reader_topic, None, participant.get_endpoint_discovery()).unwrap();
        subscriber
            .delete_datareader(&a_datareader, participant.get_endpoint_discovery())
            .expect("Failed to delete datareader");
        assert_eq!(participant.delete_topic(&reader_topic), Ok(()));
    }



    #[test]
    fn enable_threads() {
        let participant = RtpsParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport,
            MockTransport,
        )
        .unwrap();
        participant.enable().unwrap();
    }
}
