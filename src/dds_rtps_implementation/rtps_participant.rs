use crate::builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData};
use crate::dds_infrastructure::qos::{PublisherQos, SubscriberQos, TopicQos};
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::dds_rtps_implementation::rtps_publisher::{RtpsPublisher, RtpsPublisherInner};
use crate::dds_rtps_implementation::rtps_subscriber::{RtpsSubscriber, RtpsSubscriberInner};
use crate::dds_rtps_implementation::rtps_topic::RtpsTopic;
use crate::rtps::transport::udp::UdpTransport;
use crate::rtps::transport::Transport;
use crate::types::{DomainId, Duration, InstanceHandle, ReturnCode, ReturnCodes, Time};

pub struct RtpsParticipant {
    userdata_transport: Box<dyn Transport>,
    metatraffic_transport: Box<dyn Transport>,
    publisher_list: [RtpsObject<RtpsPublisherInner>; 32],
    subscriber_list: [RtpsObject<RtpsSubscriberInner>; 32],
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        //     qos: DomainParticipantQos,
        //     a_listener: impl DomainParticipantListener,
        //     mask: StatusMask,
        //     enabled: bool,
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

        // let participant = RtpsParticipant::new(domain_id);

        // // if enabled {
        // //     new_participant.enable().ok()?;
        // // }

        Some(Self {
            userdata_transport,
            metatraffic_transport,
            publisher_list: Default::default(),
            subscriber_list: Default::default(),
        })
    }

    pub fn create_publisher<'a>(
        &'a self,
        _qos: Option<&PublisherQos>,
    ) -> Option<RtpsPublisher<'a>> {
        let publisher_object = self.publisher_list.iter().find(|&x| x.is_empty())?;
        let new_publisher_inner = RtpsPublisherInner::default();
        publisher_object.initialize(new_publisher_inner).ok()?;
        publisher_object.get_reference().ok()
    }

    pub fn delete_publisher(&self, a_publisher: &RtpsPublisher) -> ReturnCode<()> {
        let publisher_object = self
            .publisher_list
            .iter()
            .find(|&x| x == a_publisher)
            .ok_or(ReturnCodes::PreconditionNotMet(
                "Publisher not found in participant",
            ))?;
        publisher_object.delete();
        Ok(())
    }

    pub fn create_subscriber(&self, _qos: Option<&SubscriberQos>) -> Option<RtpsSubscriber> {
        let subscriber_object = self.subscriber_list.iter().find(|&x| x.is_empty())?;
        let new_subscriber_inner = RtpsSubscriberInner::default();
        subscriber_object.initialize(new_subscriber_inner).ok()?;
        subscriber_object.get_reference().ok()
    }

    pub fn delete_subscriber(&self, a_subscriber: &RtpsSubscriber) -> ReturnCode<()> {
        self
            .subscriber_list
            .iter()
            .find(|&x| x == a_subscriber)
            .ok_or(ReturnCodes::PreconditionNotMet(
                "Publisher not found in participant",
            ))?
            .delete();
        Ok(())
    }

    pub fn create_topic(
        &self,
        _topic_name: String,
        _qos: Option<&TopicQos>,
    ) -> Option<RtpsTopic> {
        todo!()
    }

    pub fn delete_topic(&self, _a_topic: &RtpsTopic) -> ReturnCode<()> {
        todo!()
    }

    pub fn find_topic(
        &self,
        _topic_name: String,
        _timeout: Duration,
    ) -> Option<RtpsTopic> {
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
}
