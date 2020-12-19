use crate::builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData};
use crate::dds_infrastructure::qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos};
use crate::dds_rtps_implementation::rtps_object::RtpsObjectList;
use crate::dds_rtps_implementation::rtps_publisher::{RtpsPublisher, RtpsPublisherInner};
use crate::dds_rtps_implementation::rtps_subscriber::{RtpsSubscriber, RtpsSubscriberInner};
use crate::dds_rtps_implementation::rtps_topic::{RtpsTopic, RtpsTopicInner};
use crate::rtps::structure::Participant;
use crate::rtps::transport::udp::UdpTransport;
use crate::rtps::transport::Transport;
use crate::rtps::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
use crate::types::{DomainId, Duration, InstanceHandle, ReturnCode, Time};
use std::cell::RefCell;
use std::sync::{atomic, Arc, Mutex};
use std::thread::JoinHandle;

pub struct RtpsParticipantInner {
    participant: Participant,
    qos: Mutex<DomainParticipantQos>,
    userdata_transport: Box<dyn Transport>,
    metatraffic_transport: Box<dyn Transport>,
    publisher_list: RtpsObjectList<RtpsPublisherInner>,
    subscriber_list: RtpsObjectList<RtpsSubscriberInner>,
    topic_list: RtpsObjectList<RtpsTopicInner>,
    enabled: atomic::AtomicBool,
}

pub struct RtpsParticipant {
    inner: Arc<RtpsParticipantInner>,
    thread_list: RefCell<Vec<JoinHandle<()>>>,
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        //     a_listener: impl DomainParticipantListener,
        //     mask: StatusMask,
    ) -> Option<Self> {
        let interface = "Ethernet";
        let userdata_transport =
            Box::new(UdpTransport::default_userdata_transport(domain_id, interface).unwrap());
        let metatraffic_transport =
            Box::new(UdpTransport::default_metatraffic_transport(domain_id, interface).unwrap());
        // let domain_tag = "".to_string();
        // let lease_duration = Duration {
        //     sec: 30,
        //     nanosec: 0,
        // };
        let guid_prefix = [1; 12];
        let participant = Participant::new(guid_prefix, domain_id, PROTOCOL_VERSION_2_4, VENDOR_ID);
        let qos = qos.unwrap_or_default();
        let enabled = atomic::AtomicBool::new(false);

        let inner = Arc::new(RtpsParticipantInner {
            participant,
            qos: Mutex::new(qos),
            userdata_transport,
            metatraffic_transport,
            publisher_list: Default::default(),
            subscriber_list: Default::default(),
            topic_list: Default::default(),
            enabled,
        });

        let thread_list = RefCell::new(Vec::new());
        Some(Self { inner, thread_list })
    }

    pub fn create_publisher<'a>(
        &'a self,
        _qos: Option<&PublisherQos>,
    ) -> Option<RtpsPublisher<'a>> {
        let new_publisher = RtpsPublisherInner::default();
        self.inner.publisher_list.create(new_publisher)
    }

    pub fn delete_publisher(&self, a_publisher: &RtpsPublisher) -> ReturnCode<()> {
        a_publisher.delete();
        Ok(())
    }

    pub fn create_subscriber(&self, _qos: Option<&SubscriberQos>) -> Option<RtpsSubscriber> {
        todo!()
        // let subscriber_object = self.subscriber_list.iter().find(|&x| x.is_empty())?;
        // let new_subscriber_inner = RtpsSubscriberInner::default();
        // subscriber_object.initialize(new_subscriber_inner).ok()?;
        // subscriber_object.get_reference().ok()
    }

    pub fn delete_subscriber(&self, a_subscriber: &RtpsSubscriber) -> ReturnCode<()> {
        a_subscriber.delete();
        Ok(())
    }

    pub fn create_topic(&self, _topic_name: String, _qos: Option<&TopicQos>) -> Option<RtpsTopic> {
        todo!()
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

    pub fn set_default_publisher_qos(&self, _qos: Option<PublisherQos>) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        todo!()
    }

    pub fn set_default_subscriber_qos(&self, _qos: Option<SubscriberQos>) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        todo!()
    }

    pub fn set_default_topic_qos(&self, _qos: Option<TopicQos>) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_topic_qos(&self) -> TopicQos {
        todo!()
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
        println!("Dropping");
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

    #[test]
    fn enable_threads() {
        let participant = RtpsParticipant::new(0, None).unwrap();
        participant.enable();
    }
}
