use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

// use crate::participant::Participant;
use crate::transport::udp::UdpTransport;
use crate::discovery::sedp::SimpleEndpointDiscoveryProtocol;
use crate::discovery::discovered_writer_data::DiscoveredWriterData;
// use crate::reader::Reader;
use crate::rtps_publisher::RtpsPublisher;
use crate::rtps_subscriber::RtpsSubscriber;


use rust_dds_api::infrastructure::qos::{DataReaderQos, DataWriterQos, PublisherQos, DomainParticipantQos};
use rust_dds_api::types::{DomainId, InstanceHandle, ReturnCode, TopicKind};
use rust_dds_api::infrastructure::entity::Entity;
use rust_dds_api::domain::DomainParticipantListener;
use rust_dds_api::domain::DomainParticipant;

pub struct RtpsProtocol {
    // participant: Participant,
    // thread_handle: RefCell<Vec<JoinHandle<()>>>,
}

impl RtpsProtocol {
    pub fn new(domain_id: DomainId) -> Self {
        let interface = "Ethernet";
        let userdata_transport =
            UdpTransport::default_userdata_transport(domain_id, interface).unwrap();
        let metatraffic_transport =
            UdpTransport::default_metatraffic_transport(domain_id, interface).unwrap();
        let domain_tag = "".to_string();
        let lease_duration = rust_dds_api::types::Duration {
            sec: 30,
            nanosec: 0,
        };

        // let participant = Participant::new(
        //     domain_id,
        //     userdata_transport,
        //     metatraffic_transport,
        //     domain_tag,
        //     lease_duration,
        // );

        Self {
        //     participant,
        //     thread_handle: RefCell::new(Vec::new()),
        }
    }
}

impl Entity for RtpsProtocol {
    type Qos = DomainParticipantQos;

    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&self, qos_list: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self, qos_list: &mut Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn set_listener(&self, a_listener: Self::Listener, mask: rust_dds_api::infrastructure::status::StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self, ) -> Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self, ) -> rust_dds_api::infrastructure::entity::StatusCondition {
        todo!()
    }

    fn get_status_changes(&self, ) -> rust_dds_api::infrastructure::status::StatusMask {
        todo!()
    }

    fn enable(&self, ) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self, ) -> ReturnCode<InstanceHandle> {
        todo!()
    }
}

impl<T:DDSType> DomainParticipant<T> for RtpsProtocol {
    type Publisher = RtpsPublisher;
    type Subscriber = RtpsSubscriber;
    type Topic = RtpsTopic<T>;

    fn create_publisher(
        &self,
        qos: Option<&PublisherQos>,
        _a_listener: impl rust_dds_api::publication::PublisherListener,
        _mask: rust_dds_api::infrastructure::status::StatusMask
    ) -> Option<Self::Publisher> {
        todo!()
    }

    fn delete_publisher<'d>(
        &self,
        a_publisher: &'d Self::Publisher
    ) -> ReturnCode<()> {
        todo!()
    }

    fn create_subscriber(
        &self,
        qos: Option<&rust_dds_api::infrastructure::qos::SubscriberQos>,
        _a_listener: impl rust_dds_api::subscription::SubscriberListener,
        _mask: rust_dds_api::infrastructure::status::StatusMask
    ) -> Option<Self::Subscriber> {
        todo!()
    }

    fn delete_subscriber<'d>(
        &self,
        a_subscriber: &'d mut Self::Subscriber,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn create_topic<T: rust_dds_api::types::DDSType>(
        &self,
        topic_name: String,
        qos: Option<&rust_dds_api::infrastructure::qos::TopicQos>,
        _a_listener: impl rust_dds_api::topic::TopicListener<T>,
        _mask: rust_dds_api::infrastructure::status::StatusMask
    ) -> Option<Box<dyn rust_dds_api::topic::Topic<T>>> {
        todo!()
    }

    fn delete_topic<T: rust_dds_api::types::DDSType>(
        &self,
        a_topic: &mut dyn rust_dds_api::topic::Topic<T>,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn find_topic<T: rust_dds_api::types::DDSType>(
        &self,
        _topic_name: String,
        _timeout: rust_dds_api::types::Duration,
    ) -> Option<Box<dyn rust_dds_api::topic::Topic<T>>> {
        todo!()
    }

    fn lookup_topicdescription<T: rust_dds_api::types::DDSType>(
        &self,
        _name: &String,
    ) -> Option<&dyn rust_dds_api::topic::TopicDescription> {
        todo!()
    }

    fn get_builtin_subscriber(&self) -> Self::Subscriber {
        todo!()
    }

    fn ignore_participant(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    fn ignore_topic(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    fn ignore_publication(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    fn ignore_subscription(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_domain_id(&self) -> DomainId {
        todo!()
    }

    fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> ReturnCode<()> {
        todo!()
    }

    fn set_default_publisher_qos(
        &self,
        qos: Option<PublisherQos>,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_default_publisher_qos(
        &self
    ) -> ReturnCode<PublisherQos> {
        todo!()
    }

    fn set_default_subscriber_qos(
        &self,
        qos: Option<rust_dds_api::infrastructure::qos::SubscriberQos>,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_default_subscriber_qos(
        &self
    ) -> ReturnCode<rust_dds_api::infrastructure::qos::SubscriberQos> {
        todo!()
    }

    fn set_default_topic_qos(
        &self,
        qos: Option<rust_dds_api::infrastructure::qos::TopicQos>,
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_default_topic_qos(
        &self
    ) -> ReturnCode<rust_dds_api::infrastructure::qos::TopicQos> {
        todo!()
    }

    fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle]
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_discovered_participant_data(
        &self,
        _participant_data: rust_dds_api::builtin_topics::ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_discovered_topics(
        &self,
        _topic_handles: &mut [InstanceHandle]
    ) -> ReturnCode<()> {
        todo!()
    }

    fn get_discovered_topic_data(
        &self,
        _topic_data: rust_dds_api::builtin_topics::TopicBuiltinTopicData,
        _topic_handle: InstanceHandle
    ) -> ReturnCode<()> {
        todo!()
    }

    fn contains_entity(
        &self,
        _a_handle: InstanceHandle
    ) -> bool {
        todo!()
    }

    fn get_current_time() -> ReturnCode<rust_dds_api::types::Time> {
        todo!()
    }
}



// fn configure_readers(sedp: &mut SimpleEndpointDiscoveryProtocol, readers: &mut[&mut Reader]) {
//     let seq_num_min = sedp.sedp_builtin_subscriptions_reader().reader.reader_cache.get_seq_num_min().unwrap();
//     let seq_num_max = sedp.sedp_builtin_subscriptions_reader().reader.reader_cache.get_seq_num_max().unwrap();
//     for seq_num in seq_num_min..=seq_num_max {   
//         let cc = sedp.sedp_builtin_subscriptions_reader().reader.reader_cache.get_change(seq_num).unwrap();

//         let discovered_writer_data = DiscoveredWriterData::from_key_data(cc.instance_handle(), cc.data_value().as_ref().unwrap());

//         for reader in readers.iter_mut() {
//             // if reader matched
//                 // get writer proxy
//                 // add writer proxy
//                 // reader.stateful_reader.matched_writer_add(a_writer_proxy);
//                 // call Reader.on_subscription_matched (listener method)
//         }



//     }
// }