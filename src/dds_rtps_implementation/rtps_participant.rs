use crate::builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData};
use crate::dds_infrastructure::qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos};
use crate::dds_rtps_implementation::rtps_object::RtpsObjectList;
use crate::dds_rtps_implementation::rtps_publisher::{RtpsPublisher, RtpsPublisherInner};
use crate::dds_rtps_implementation::rtps_subscriber::{RtpsSubscriber, RtpsSubscriberInner};
use crate::dds_rtps_implementation::rtps_topic::{RtpsTopic, RtpsTopicInner};
use crate::rtps::structure::Participant;
use crate::rtps::transport::Transport;
use crate::rtps::types::{GUID, EntityId, EntityKind};
use crate::rtps::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID,};
use crate::types::{DomainId, Duration, InstanceHandle, ReturnCode, Time};
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
    topic_list: RtpsObjectList<RtpsTopicInner>,
    topic_count: atomic::AtomicU8,
    enabled: atomic::AtomicBool,
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
        });

        let thread_list = RefCell::new(Vec::new());
        Some(Self { inner, thread_list })
    }

    pub fn create_publisher<'a>(
        &'a self,
        qos: Option<PublisherQos>,
    ) -> Option<RtpsPublisher<'a>> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [0, self.inner.publisher_count.fetch_add(1, atomic::Ordering::Relaxed), 0];
        let entity_id = EntityId::new(entity_key, EntityKind::BuiltInWriterGroup);
        let new_publisher_guid = GUID::new(guid_prefix, entity_id);
        let new_publisher_qos = qos.unwrap_or(self.get_default_publisher_qos());
        let new_publisher = RtpsPublisherInner::new(new_publisher_guid, new_publisher_qos);
        self.inner.publisher_list.add(new_publisher)
    }

    pub fn delete_publisher(&self, a_publisher: &RtpsPublisher) -> ReturnCode<()> {
        a_publisher.delete();
        Ok(())
    }

    pub fn create_subscriber(&self, qos: Option<SubscriberQos>) -> Option<RtpsSubscriber> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [0, self.inner.subscriber_count.fetch_add(1, atomic::Ordering::Relaxed), 0];
        let entity_id = EntityId::new(entity_key, EntityKind::BuiltInReaderGroup);
        let new_subscriber_guid = GUID::new(guid_prefix, entity_id);
        let new_subscriber_qos = qos.unwrap_or(self.get_default_subscriber_qos());
        let new_subscriber = RtpsSubscriberInner::new(new_subscriber_guid, new_subscriber_qos);
        self.inner.subscriber_list.add(new_subscriber)
    }

    pub fn delete_subscriber(&self, a_subscriber: &RtpsSubscriber) -> ReturnCode<()> {
        a_subscriber.delete();
        Ok(())
    }

    pub fn create_topic(&self, topic_name: String, type_name: &'static str, qos: Option<TopicQos>) -> Option<RtpsTopic> {
        let guid_prefix = self.inner.participant.entity.guid.prefix();
        let entity_key = [0, self.inner.topic_count.fetch_add(1, atomic::Ordering::Relaxed), 0];
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedUnknown);
        let new_topic_guid = GUID::new(guid_prefix, entity_id);
        let new_topic_qos = qos.unwrap_or(self.get_default_topic_qos());
        let new_topic = RtpsTopicInner::new(new_topic_guid, topic_name, type_name, new_topic_qos);
        self.inner.topic_list.add(new_topic)
    }

    pub fn delete_topic(&self, _a_topic: &RtpsTopic) -> ReturnCode<()> {
        todo!()
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

    pub fn enable(&self) {
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

fn process_metatraffic() {}

fn process_user_traffic() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::messages::RtpsMessage;
    use crate::rtps::types::Locator;
    use crate::rtps::transport::TransportResult;

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
    fn enable_threads() {
        let participant = RtpsParticipant::new(0, DomainParticipantQos::default(), MockTransport, MockTransport).unwrap();
        participant.enable();
    }
}
