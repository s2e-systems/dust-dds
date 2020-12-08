use std::sync::{Arc,Weak,Mutex};

use crate::types::{DomainId, ReturnCode, ReturnCodes, DDSType};

use crate::rtps::transport::Transport;
use crate::rtps::transport::udp::UdpTransport;

use crate::dds::implementation::rtps_subscriber::RtpsSubscriber;
use crate::dds::implementation::rtps_publisher::{RtpsPublisher, RtpsPublisherInner};
use crate::dds::implementation::rtps_topic::RtpsTopic;
use crate::dds::implementation::rtps_object::RtpsObject;

pub struct RtpsParticipant {
    userdata_transport: Box<dyn Transport>,
    metatraffic_transport: Box<dyn Transport>,
    publisher_list: Arc<[RtpsObject<RtpsPublisherInner>;32]>,
}

impl RtpsParticipant {
    pub fn new (
        domain_id: DomainId,
    //     qos: DomainParticipantQos,
    //     a_listener: impl DomainParticipantListener,
    //     mask: StatusMask,
    //     enabled: bool,
    ) ->  Option<Self> {
        let interface = "Ethernet";
        let userdata_transport = Box::new(
            UdpTransport::default_userdata_transport(domain_id, interface).unwrap());
        let metatraffic_transport = Box::new(
            UdpTransport::default_metatraffic_transport(domain_id, interface).unwrap());
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
            publisher_list: Arc::new(Default::default())
        })
    }

    pub fn create_publisher<'a>(&'a self) -> Option<RtpsPublisher<'a>> {
        todo!()
    }

    pub fn delete_publisher(&self, _a_publisher: &RtpsPublisher) -> ReturnCode<()> {
        todo!()
    }

    pub fn create_topic<T:DDSType>(&self) -> Option<&RtpsTopic<T>> {
        todo!()
    }

    pub fn delete_topic<T:DDSType>(&self, _a_topic: &RtpsTopic<T>) -> ReturnCode<()> {
        todo!()
    }

    pub fn create_subscriber(&self) -> Option<&RtpsSubscriber> {
        todo!()
    }

    pub fn delete_subscriber(&self, _a_subscriber: &RtpsSubscriber) -> ReturnCode<()> {
        todo!()
    }

    
}