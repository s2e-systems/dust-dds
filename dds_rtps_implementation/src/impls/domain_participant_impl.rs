use std::{
    cell::RefCell,
    sync::{atomic, Arc, Mutex, Once, Weak},
    thread::JoinHandle,
};

use rust_dds_api::{
    dcps_psm::{DomainId, StatusMask},
    dds_type::DDSType,
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    publication::publisher_listener::PublisherListener,
    return_type::DDSResult,
    subscription::subscriber_listener::SubscriberListener,
    topic::topic_listener::TopicListener,
};
// use rust_rtps::{
//     behavior::{types::Duration, RTPSStatelessWriter},
//     discovery::{
//         spdp_data::ParticipantProxy,
//         spdp_endpoints::SPDPbuiltinParticipantWriter,
//         types::{BuiltinEndpointQos, BuiltinEndpointSet},
//     },
//     structure::{RTPSEntity, RTPSParticipant},
//     types::{
//         constants::{ENTITYID_PARTICIPANT, PROTOCOL_VERSION_2_4, VENDOR_ID},
//         GuidPrefix, Locator, ProtocolVersion, VendorId, GUID,
//     },
// };

// use crate::{
//     spdp_discovered_participant_data::SPDPdiscoveredParticipantData, transport::Transport,
//     utils::message_sender::RtpsMessageSender,
// };

use super::{
    publisher_impl::PublisherImpl, subscriber_impl::SubscriberImpl, topic_impl::TopicImpl,
};

// pub struct DomainParticipantImplConfiguration {
//     pub userdata_transport: Box<dyn Transport>,
//     pub metatraffic_transport: Box<dyn Transport>,
//     pub domain_tag: &'static str,
//     pub lease_duration: Duration,
//     pub spdp_locator_list: Vec<Locator>,
// }

// struct RtpsBuiltinParticipantEntities {
//     publisher: PublisherImpl,
//     subscriber: SubscriberImpl,
//     transport: Box<dyn Transport>,
// }

// impl RtpsBuiltinParticipantEntities {
//     pub fn send_data(&self, participant_guid_prefix: GuidPrefix) {
//         let transport = &self.transport;
//         let publisher = &self.publisher;
//         for writer in publisher.into_iter() {
//             let mut writer_lock = writer.lock().unwrap();
//             let destined_messages = writer_lock.produce_messages();
//             RtpsMessageSender::send_cache_change_messages(
//                 participant_guid_prefix,
//                 transport.as_ref(),
//                 destined_messages,
//             );
//             writer_lock.unsent_changes_reset();
//         }
//     }
// }

// struct RtpsParticipantEntities {
//     publisher_list: Mutex<Vec<Arc<Mutex<PublisherImpl>>>>,
//     subscriber_list: Mutex<Vec<Arc<Mutex<SubscriberImpl>>>>,
//     topic_list: Mutex<Vec<Arc<Mutex<TopicImpl>>>>,
//     transport: Box<dyn Transport>,
// }

// impl RtpsParticipantEntities {
//     fn new(transport: Box<dyn Transport>) -> Self {
//         Self {
//             publisher_list: Default::default(),
//             subscriber_list: Default::default(),
//             topic_list: Default::default(),
//             transport,
//         }
//     }

//     pub fn send_data(&self, _participant_guid_prefix: GuidPrefix) {
//         // let _transport = &self.transport;
//         let publisher_list = self.publisher_list.lock().unwrap();
//         for _publisher in publisher_list.iter() {
//             // for _writer in publisher.lock().unwrap().writer_list() {
//             todo!()
//             // let destined_messages = writer.lock().unwrap().produce_messages();
//             // RtpsMessageSender::send_cache_change_messages(
//             //     participant_guid_prefix,
//             //     transport.as_ref(),
//             //     destined_messages,
//             // );
//             // }
//         }
//     }
// }

pub struct DomainParticipantImpl {
    domain_id: DomainId,
    // guid_prefix: GuidPrefix,
    qos: DomainParticipantQos,
    publisher_count: usize,
    subscriber_count: usize,
    topic_count: usize,
    default_publisher_qos: PublisherQos,
    default_subscriber_qos: SubscriberQos,
    default_topic_qos: TopicQos,
    builin_publisher: Arc<Mutex<PublisherImpl>>,
    builtin_subscriber: Arc<Mutex<SubscriberImpl>>,
    // builtin_entities: Arc<RtpsBuiltinParticipantEntities>,
    // user_defined_entities: Arc<RtpsParticipantEntities>,
    enabled: Arc<atomic::AtomicBool>,
    enabled_function: Once,
    thread_list: RefCell<Vec<JoinHandle<()>>>,
    a_listener: Option<Box<dyn DomainParticipantListener>>,
    mask: StatusMask,
}

impl DomainParticipantImpl {
    pub fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        a_listener: Option<Box<dyn DomainParticipantListener>>,
        mask: StatusMask,
        builtin_subscriber: SubscriberImpl,
        builtin_publisher: PublisherImpl,
        // configuration: DomainParticipantImplConfiguration,
    ) -> Self {
        let _guid_prefix = [1; 12];

        // let spdp_builtin_participant_writer = SPDPbuiltinParticipantWriter::create(
        //     guid_prefix,
        //     configuration.metatraffic_transport.unicast_locator_list(),
        //     configuration.metatraffic_transport.multicast_locator_list(),
        //     &configuration.spdp_locator_list,
        // );

        // let mut builtin_publisher = PublisherImpl::new(PublisherQos::default(), None, 0);
        // let spdp_announcer = Arc::new(Mutex::new(DataWriterImpl::new(
        //     spdp_builtin_participant_writer,
        // )));

        // let dds_participant_data = ParticipantBuiltinTopicData {
        //     key: BuiltInTopicKey { value: [0i32; 3] },
        //     user_data: qos.user_data.clone(),
        // };
        // let participant_proxy = ParticipantProxy {
        //     domain_id: domain_id as rust_rtps::discovery::types::DomainId,
        //     domain_tag: configuration.domain_tag,
        //     protocol_version: PROTOCOL_VERSION_2_4,
        //     guid_prefix,
        //     vendor_id: VENDOR_ID,
        //     expects_inline_qos: false,
        //     metatraffic_unicast_locator_list: configuration
        //         .metatraffic_transport
        //         .unicast_locator_list()
        //         .into(),
        //     metatraffic_multicast_locator_list: configuration
        //         .metatraffic_transport
        //         .multicast_locator_list()
        //         .into(),
        //     default_unicast_locator_list: configuration
        //         .userdata_transport
        //         .unicast_locator_list()
        //         .into(),
        //     default_multicast_locator_list: configuration
        //         .userdata_transport
        //         .multicast_locator_list()
        //         .into(),
        //     available_builtin_endpoints: BuiltinEndpointSet::default(),
        //     manual_liveliness_count: 0,
        //     builtin_endpoint_qos: BuiltinEndpointQos::default(),
        // };
        // let spdp_discovered_participant_data = SPDPdiscoveredParticipantData {
        //     dds_participant_data,
        //     participant_proxy,
        //     lease_duration: configuration.lease_duration,
        // };
        // spdp_announcer
        //     .lock()
        //     .unwrap()
        //     .write_w_timestamp(
        //         spdp_discovered_participant_data,
        //         None,
        //         Time { sec: 0, nanosec: 0 },
        //     )
        //     .ok();
        // builtin_publisher.add_datawriter(spdp_announcer);

        // let builtin_subscriber = SubscriberImpl::new(SubscriberQos::default(), None, 0);
        // let builtin_entities = Arc::new(RtpsBuiltinParticipantEntities {
        //     publisher: builtin_publisher,
        //     subscriber: builtin_subscriber,
        //     transport: configuration.metatraffic_transport,
        // });

        // let user_defined_entities = Arc::new(RtpsParticipantEntities::new(
        //     configuration.userdata_transport,
        // ));

        Self {
            domain_id,
            // guid_prefix,
            qos,
            publisher_count: 0,
            subscriber_count: 0,
            topic_count: 0,
            default_publisher_qos: PublisherQos::default(),
            default_subscriber_qos: SubscriberQos::default(),
            default_topic_qos: TopicQos::default(),
            builin_publisher: Arc::new(Mutex::new(builtin_publisher)),
            builtin_subscriber: Arc::new(Mutex::new(builtin_subscriber)),
            enabled: Arc::new(atomic::AtomicBool::new(false)),
            enabled_function: Once::new(),
            thread_list: RefCell::new(Vec::new()),
            a_listener,
            mask,
        }
    }

    pub fn create_publisher(
        &self,
        qos: Option<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener>>,
        mask: StatusMask,
    ) -> DDSResult<Weak<Mutex<PublisherImpl>>> {
        // let guid_prefix = self.participant.entity.guid.prefix();
        let qos = qos.unwrap_or(self.get_default_publisher_qos());
        let _publisher = Arc::new(Mutex::new(PublisherImpl::new(qos, a_listener, mask)));

        // self.user_defined_entities
        //     .publisher_list
        //     .lock()
        //     .unwrap()
        //     .push(publisher.clone());

        // Ok(Arc::downgrade(&publisher))
        todo!()
    }

    pub fn delete_publisher(&self, _impl_ref: &Weak<Mutex<PublisherImpl>>) -> DDSResult<()> {
        todo!()
        // let publisher_impl = impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        // if publisher_impl.lock().unwrap().writer_list().is_empty() {
        //     self.user_defined_entities
        //         .publisher_list
        //         .lock()
        //         .unwrap()
        //         .retain(|x| !Arc::ptr_eq(x, &publisher_impl));
        //     Ok(())
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Publisher still contains data writers",
        //     ))
        // }
    }

    pub fn create_subscriber(
        &self,
        qos: Option<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener>>,
        mask: StatusMask,
    ) -> DDSResult<Weak<Mutex<SubscriberImpl>>> {
        // let guid_prefix = self.participant.entity.guid.prefix();
        // let qos = qos.unwrap_or(self.get_default_publisher_qos());
        // let publisher = Arc::new(Mutex::new(RtpsPublisherImpl::new(qos, a_listener, mask)));

        // self.user_defined_entities
        //     .publisher_list
        //     .lock()
        //     .unwrap()
        //     .push(publisher.clone());

        // let guid_prefix = self.participant.entity.guid.prefix();
        // let entity_key = [
        //     0,
        //     self.subscriber_count
        //         .fetch_add(1, atomic::Ordering::Relaxed),
        //     0,
        // ];
        // let entity_kind = ENTITY_KIND_USER_DEFINED_READER_GROUP;
        // let entity_id = EntityId::new(entity_key, entity_kind);
        // let guid = GUID::new(guid_prefix, entity_id);
        // let group = rust_rtps::structure::Group::new(guid);
        let qos = qos.unwrap_or(self.get_default_subscriber_qos().clone());
        let _subscriber = Arc::new(Mutex::new(SubscriberImpl::new(qos, a_listener, mask)));

        // self.user_defined_entities
        //     .subscriber_list
        //     .lock()
        //     .unwrap()
        //     .push(subscriber.clone());

        // Ok(Arc::downgrade(&subscriber))
        todo!()
    }

    pub fn delete_subscriber(&self, _impl_ref: &Weak<Mutex<SubscriberImpl>>) -> DDSResult<()> {
        // let subscriber_impl = impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        // if subscriber_impl.lock().unwrap().reader_list().is_empty() {
        //     self.user_defined_entities
        //         .subscriber_list
        //         .lock()
        //         .unwrap()
        //         .retain(|x| !Arc::ptr_eq(x, &subscriber_impl));
        //     Ok(())
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Subscriber still contains data readers",
        //     ))
        // }
        todo!()
    }

    pub fn create_topic<T: DDSType>(
        &self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<Box<dyn TopicListener>>,
        _mask: StatusMask,
    ) -> DDSResult<Weak<Mutex<TopicImpl>>> {
        // let guid_prefix = self.participant.entity.guid.prefix();
        // let entity_key = [
        //     0,
        //     self.topic_count.fetch_add(1, atomic::Ordering::Relaxed),
        //     0,
        // ];
        // let entity_kind = ENTITY_KIND_USER_DEFINED_UNKNOWN;
        // let entity_id = EntityId::new(entity_key, entity_kind);
        // let guid = GUID::new(guid_prefix, entity_id);
        // let entity = rust_rtps::structure::Entity::new(guid);
        // let qos = qos.unwrap_or(self.get_default_topic_qos());
        // qos.is_consistent()?;
        // let topic = Arc::new(Mutex::new(TopicImpl::new(
        //     topic_name,
        //     T::type_name(),
        //     qos,
        //     a_listener,
        //     mask,
        // )));

        // self.user_defined_entities
        //     .topic_list
        //     .lock()
        //     .unwrap()
        //     .push(topic.clone());

        // Ok(Arc::downgrade(&topic))
        todo!()
    }

    pub fn delete_topic(&self, _impl_ref: &Weak<Mutex<TopicImpl>>) -> DDSResult<()> {
        // impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?; // Just to check if already deleted
        // if Weak::strong_count(impl_ref) == 1 {
        //     self.user_defined_entities
        //         .topic_list
        //         .lock()
        //         .unwrap()
        //         .retain(|x| !Weak::ptr_eq(&Arc::downgrade(x), &impl_ref));
        //     Ok(())
        // } else {
        //     Err(DDSError::PreconditionNotMet(
        //         "Topic still attached to some data reader or data writer",
        //     ))
        // }
        todo!()
    }

    pub fn set_qos(&mut self, qos: Option<DomainParticipantQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> DomainParticipantQos {
        self.qos.clone()
    }

    pub fn set_default_publisher_qos(&mut self, qos: Option<PublisherQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.default_publisher_qos = qos;
        Ok(())
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQos {
        self.default_publisher_qos.clone()
    }

    pub fn set_default_subscriber_qos(&mut self, qos: Option<SubscriberQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.default_subscriber_qos = qos;
        Ok(())
    }

    pub fn get_default_subscriber_qos(&self) -> SubscriberQos {
        self.default_subscriber_qos.clone()
    }

    pub fn set_default_topic_qos(&mut self, qos: Option<TopicQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        self.default_topic_qos = qos;
        Ok(())
    }

    pub fn get_default_topic_qos(&self) -> TopicQos {
        self.default_topic_qos.clone()
    }

    pub fn enable(&mut self) -> DDSResult<()> {
        // let enabled = self.enabled.clone();
        // let mut thread_list = self.thread_list.borrow_mut();
        // self.enabled.store(true, atomic::Ordering::Release);
        // let builtin_entities = self.builtin_entities.clone();
        // let participant_guid_prefix = self.guid_prefix;
        // self.enabled_function.call_once(|| {
        //     thread_list.push(std::thread::spawn(move || {
        //         while enabled.load(atomic::Ordering::Acquire) {
        //             Self::send();

        //             builtin_entities.send_data(participant_guid_prefix);

        //             std::thread::sleep(std::time::Duration::from_secs(1));
        //         }
        //     }));
        // });
        Ok(())
    }

    fn send() {

        // let writer = Writer::new();
        // let spdp_announcer = StatelessWriter::new(writer);
        // builtin_entities.send_data(guid_prefix);

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

        //
        //     let enabled = self.enabled.clone();
        //     let builtin_entities = self.builtin_entities.clone();
    }
}

// impl RTPSEntity for DomainParticipantImpl {
//     fn guid(&self) -> GUID {
//         GUID::new(self.guid_prefix, ENTITYID_PARTICIPANT)
//     }
// }

// impl RTPSParticipant for DomainParticipantImpl {
//     fn default_unicast_locator_list(&self) -> &[Locator] {
//         todo!()
//         // self.user_defined_entities.transport.unicast_locator_list()
//     }

//     fn default_multicast_locator_list(&self) -> &[Locator] {
//         todo!()
//         // self.user_defined_entities
//         //     .transport
//         //     .multicast_locator_list()
//     }

//     fn protocol_version(&self) -> ProtocolVersion {
//         PROTOCOL_VERSION_2_4
//     }

//     fn vendor_id(&self) -> VendorId {
//         VENDOR_ID
//     }
// }

impl Drop for DomainParticipantImpl {
    fn drop(&mut self) {
        self.enabled.store(false, atomic::Ordering::Release);
        for thread in self.thread_list.borrow_mut().drain(..) {
            thread.join().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    //     use rust_rtps::types::Locator;

    //     use crate::transport::Transport;

    use rust_dds_api::infrastructure::qos::DataWriterQos;
    use rust_rtps_pim::{behavior::stateless_writer::RTPSStatelessWriter, structure};
    use rust_rtps_udp_psm::{types::Locator, RtpsUdpPsm};

    use crate::impls::stateless_data_writer_impl::StatelessDataWriterImpl;

    use super::*;

    //     struct TestType;

    //     impl DDSType for TestType {
    //         fn type_name() -> &'static str {
    //             "TestType"
    //         }

    //         fn has_key() -> bool {
    //             todo!()
    //         }

    //         fn key(&self) -> Vec<u8> {
    //             todo!()
    //         }

    //         fn serialize(&self) -> Vec<u8> {
    //             todo!()
    //         }

    //         fn deserialize(_data: Vec<u8>) -> Self {
    //             todo!()
    //         }
    //     }

    //     #[derive(Default)]
    //     struct MockTransport {
    //         unicast_locator_list: Vec<Locator>,
    //         multicast_locator_list: Vec<Locator>,
    //     }

    //     impl Transport for MockTransport {
    //         fn write<'a>(
    //             &'a self,
    //             _message: rust_rtps::messages::RtpsMessage<'a>,
    //             _destination_locator: &Locator,
    //         ) {
    //         }

    //         fn unicast_locator_list(&self) -> &[Locator] {
    //             &self.unicast_locator_list
    //         }

    //         fn multicast_locator_list(&self) -> &[Locator] {
    //             &self.multicast_locator_list
    //         }
    //     }

    #[test]
    fn demo_participant_test() {
        let builtin_subscriber = SubscriberImpl::new(SubscriberQos::default(), None, 0);
        let mut builtin_publisher = PublisherImpl::new(PublisherQos::default(), None, 0);

        let mut stateless_data_writer = StatelessDataWriterImpl::new(DataWriterQos::default());
        stateless_data_writer.reader_locator_add(Locator {
            kind: <<RtpsUdpPsm as structure::Types>::Locator as structure::types::Locator>::LOCATOR_KIND_UDPv4,
            port: 7400,
            address: [0,0,0,0,0,0,0,0,0,0,0,0,239,255,0,1],
        });
        stateless_data_writer.write_w_timestamp().unwrap();

        builtin_publisher.stateless_writer_add(stateless_data_writer);

        let mut participant = DomainParticipantImpl::new(
            0,
            DomainParticipantQos::default(),
            None,
            0,
            builtin_subscriber,
            builtin_publisher,
        );

        participant.enable().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // #[test]
    // fn create_publisher() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };
    //configuration
    // let builtin_subscriber = SubscriberImpl::new(SubscriberQos::default(), None, 0);
    // let mut builtin_publisher = PublisherImpl::new(PublisherQos::default(), None, 0);

    // let stateless_data_writer = StatelessDataWriterImpl::new(DataWriterQos::default());
    // builtin_publisher.stateless_writer_add(stateless_data_writer);

    // let participant = DomainParticipantImpl::new(
    //     0,
    //     DomainParticipantQos::default(),
    //     None,
    //     0,
    //     builtin_subscriber,
    //     builtin_publisher,
    // );

    //         let qos = Some(PublisherQos::default());
    //         let a_listener = None;
    //         let mask = 0;
    //         participant
    //             .create_publisher(qos, a_listener, mask)
    //             .expect("Error creating publisher");

    //         assert_eq!(
    //             participant
    //                 .user_defined_entities
    //                 .publisher_list
    //                 .lock()
    //                 .unwrap()
    //                 .len(),
    //             1
    //         );
    // }

    //     #[test]
    //     fn create_delete_publisher() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let qos = Some(PublisherQos::default());
    //         let a_listener = None;
    //         let mask = 0;
    //         let a_publisher = participant.create_publisher(qos, a_listener, mask).unwrap();

    //         participant
    //             .delete_publisher(&a_publisher)
    //             .expect("Error deleting publisher");
    //         assert_eq!(
    //             participant
    //                 .user_defined_entities
    //                 .publisher_list
    //                 .lock()
    //                 .unwrap()
    //                 .len(),
    //             0
    //         );
    //     }

    //     #[test]
    //     fn create_subscriber() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let qos = Some(SubscriberQos::default());
    //         let a_listener = None;
    //         let mask = 0;
    //         participant
    //             .create_subscriber(qos, a_listener, mask)
    //             .expect("Error creating subscriber");
    //         assert_eq!(
    //             participant
    //                 .user_defined_entities
    //                 .subscriber_list
    //                 .lock()
    //                 .unwrap()
    //                 .len(),
    //             1
    //         );
    //     }

    //     #[test]
    //     fn create_delete_subscriber() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let qos = Some(SubscriberQos::default());
    //         let a_listener = None;
    //         let mask = 0;
    //         let a_subscriber = participant
    //             .create_subscriber(qos, a_listener, mask)
    //             .unwrap();

    //         participant
    //             .delete_subscriber(&a_subscriber)
    //             .expect("Error deleting subscriber");
    //         assert_eq!(
    //             participant
    //                 .user_defined_entities
    //                 .subscriber_list
    //                 .lock()
    //                 .unwrap()
    //                 .len(),
    //             0
    //         );
    //     }

    //     #[test]
    //     fn create_topic() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let topic_name = "Test";
    //         let qos = Some(TopicQos::default());
    //         let a_listener = None;
    //         let mask = 0;
    //         participant
    //             .create_topic::<TestType>(topic_name, qos, a_listener, mask)
    //             .expect("Error creating topic");
    //     }

    //     #[test]
    //     fn create_delete_topic() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let topic_name = "Test";
    //         let qos = Some(TopicQos::default());
    //         let a_listener = None;
    //         let mask = 0;
    //         let a_topic = participant
    //             .create_topic::<TestType>(topic_name, qos, a_listener, mask)
    //             .unwrap();

    //         participant
    //             .delete_topic(&a_topic)
    //             .expect("Error deleting topic")
    //     }

    //     #[test]
    //     fn set_get_default_publisher_qos() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let mut participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let mut publisher_qos = PublisherQos::default();
    //         publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
    //         participant
    //             .set_default_publisher_qos(Some(publisher_qos.clone()))
    //             .expect("Error setting default publisher qos");

    //         assert_eq!(publisher_qos, participant.get_default_publisher_qos())
    //     }

    //     #[test]
    //     fn set_get_default_subscriber_qos() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let mut participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let mut subscriber_qos = SubscriberQos::default();
    //         subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
    //         participant
    //             .set_default_subscriber_qos(Some(subscriber_qos.clone()))
    //             .expect("Error setting default subscriber qos");

    //         assert_eq!(subscriber_qos, participant.get_default_subscriber_qos())
    //     }

    //     #[test]
    //     fn set_get_default_topic_qos() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let mut participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let mut topic_qos = TopicQos::default();
    //         topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
    //         participant
    //             .set_default_topic_qos(Some(topic_qos.clone()))
    //             .expect("Error setting default subscriber qos");

    //         assert_eq!(topic_qos, participant.get_default_topic_qos())
    //     }

    //     #[test]
    //     fn set_default_publisher_qos_to_default_value() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let mut participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let mut publisher_qos = PublisherQos::default();
    //         publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
    //         participant
    //             .set_default_publisher_qos(Some(publisher_qos.clone()))
    //             .unwrap();

    //         participant
    //             .set_default_publisher_qos(None)
    //             .expect("Error setting default publisher qos");

    //         assert_eq!(
    //             PublisherQos::default(),
    //             participant.get_default_publisher_qos()
    //         )
    //     }

    //     #[test]
    //     fn set_default_subscriber_qos_to_default_value() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let mut participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let mut subscriber_qos = SubscriberQos::default();
    //         subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
    //         participant
    //             .set_default_subscriber_qos(Some(subscriber_qos.clone()))
    //             .unwrap();

    //         participant
    //             .set_default_subscriber_qos(None)
    //             .expect("Error setting default subscriber qos");

    //         assert_eq!(
    //             SubscriberQos::default(),
    //             participant.get_default_subscriber_qos()
    //         )
    //     }

    //     #[test]
    //     fn set_default_topic_qos_to_default_value() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let mut participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         let mut topic_qos = TopicQos::default();
    //         topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
    //         participant
    //             .set_default_topic_qos(Some(topic_qos.clone()))
    //             .unwrap();

    //         participant
    //             .set_default_topic_qos(None)
    //             .expect("Error setting default subscriber qos");

    //         assert_eq!(TopicQos::default(), participant.get_default_topic_qos())
    //     }

    //     #[test]
    //     fn enable() {
    //         let configuration = DomainParticipantImplConfiguration {
    //             userdata_transport: Box::new(MockTransport::default()),
    //             metatraffic_transport: Box::new(MockTransport::default()),
    //             domain_tag: "",
    //             lease_duration: Duration {
    //                 seconds: 30,
    //                 fraction: 0,
    //             },
    //             spdp_locator_list: vec![],
    //         };

    //         let mut participant =
    //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //         participant.enable().expect("Failed to enable");
    //         assert_eq!(participant.thread_list.borrow().len(), 1);
    //     }

    //     // #[test]
    //     // fn create_publisher_factory_default_qos() {
    //     //     let participant = DomainParticipantImpl::new(
    //     //         0,
    //     //         DomainParticipantQos::default(),
    //     //         MockTransport::default(),
    //     //         MockTransport::default(),
    //     //         None,
    //     //         0,
    //     //     );

    //     //     let mut publisher_qos = PublisherQos::default();
    //     //     publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //     participant
    //     //         .set_default_publisher_qos(Some(publisher_qos.clone()))
    //     //         .unwrap();

    //     //     let qos = None;
    //     //     let a_listener = None;
    //     //     let mask = 0;
    //     //     let publisher = participant
    //     //         .create_publisher(qos, a_listener, mask)
    //     //         .expect("Error creating publisher");

    //     //     assert_eq!(publisher.get_qos().unwrap(), publisher_qos);
    //     // }

    //     // #[test]
    //     // fn create_subscriber_factory_default_qos() {
    //     //     let participant = DomainParticipantImpl::new(
    //     //         0,
    //     //         DomainParticipantQos::default(),
    //     //         MockTransport::default(),
    //     //         MockTransport::default(),
    //     //         None,
    //     //         0,
    //     //     );

    //     //     let mut subscriber_qos = SubscriberQos::default();
    //     //     subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //     participant
    //     //         .set_default_subscriber_qos(Some(subscriber_qos.clone()))
    //     //         .unwrap();

    //     //     let qos = None;
    //     //     let a_listener = None;
    //     //     let mask = 0;
    //     //     let subscriber = participant
    //     //         .create_subscriber(qos, a_listener, mask)
    //     //         .expect("Error creating publisher");

    //     //     assert_eq!(subscriber.get_qos().unwrap(), subscriber_qos);
    //     // }

    //     // #[test]
    //     // fn create_topic_factory_default_qos() {
    //     //     let participant = DomainParticipantImpl::new(
    //     //         0,
    //     //         DomainParticipantQos::default(),
    //     //         MockTransport::default(),
    //     //         MockTransport::default(),
    //     //         None,
    //     //         0,
    //     //     );

    //     //     let mut topic_qos = TopicQos::default();
    //     //     topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
    //     //     participant
    //     //         .set_default_topic_qos(Some(topic_qos.clone()))
    //     //         .unwrap();

    //     //     let qos = None;
    //     //     let a_listener = None;
    //     //     let mask = 0;
    //     //     let topic = participant
    //     //         .create_topic::<TestType>("name", qos, a_listener, mask)
    //     //         .expect("Error creating publisher");

    //     //     assert_eq!(topic.get_qos().unwrap(), topic_qos);
    //     // }
}
