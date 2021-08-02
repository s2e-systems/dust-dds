use rust_rtps_pim::{
    discovery::{
        sedp::sedp_participant::SedpParticipant,
        spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData,
    },
    structure::{
        types::{EntityId, Guid, Locator, ProtocolVersion, VendorId, PROTOCOLVERSION_2_4},
        RtpsEntity, RtpsParticipant,
    },
};

use crate::utils::shared_object::RtpsShared;

use super::{
    rtps_group_impl::RtpsGroupImpl, rtps_reader_impl::RtpsReaderImpl,
    rtps_writer_impl::RtpsWriterImpl,
};

pub struct RtpsParticipantImpl {
    guid: Guid,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
}

impl RtpsParticipantImpl {
    pub fn new(guid_prefix: rust_rtps_pim::structure::types::GuidPrefix) -> Self {
        let guid = Guid::new(
            guid_prefix,
            rust_rtps_pim::structure::types::ENTITYID_PARTICIPANT,
        );

        let builtin_writer_group_guid = Guid::new(
            guid_prefix,
            EntityId::new(
                [0, 0, 0],
                rust_rtps_pim::structure::types::EntityKind::BuiltInWriterGroup,
            ),
        );
        let builtin_writer_group = RtpsShared::new(RtpsGroupImpl::new(builtin_writer_group_guid));

        let builtin_reader_group_guid = Guid::new(
            guid_prefix,
            EntityId::new(
                [0, 0, 0],
                rust_rtps_pim::structure::types::EntityKind::BuiltInReaderGroup,
            ),
        );
        let builtin_reader_group = RtpsShared::new(RtpsGroupImpl::new(builtin_reader_group_guid));

        Self {
            guid,
            protocol_version: PROTOCOLVERSION_2_4,
            vendor_id: [99, 99],
        }
    }
}

impl RtpsEntity for RtpsParticipantImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}

impl RtpsParticipant for RtpsParticipantImpl {
    fn protocol_version(&self) -> &ProtocolVersion {
        &self.protocol_version
    }

    fn vendor_id(&self) -> &VendorId {
        &self.vendor_id
    }

    fn default_unicast_locator_list(&self) -> &[Locator] {
        todo!()
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        todo!()
    }
}

impl SedpParticipant for RtpsParticipantImpl {
    type BuiltinPublicationsWriter = RtpsWriterImpl;
    type BuiltinPublicationsReader = RtpsReaderImpl;
    type BuiltinSubscriptionsWriter = RtpsWriterImpl;
    type BuiltinSubscriptionsReader = RtpsReaderImpl;
    type BuiltinTopicsWriter = RtpsWriterImpl;
    type BuiltinTopicsReader = RtpsReaderImpl;

    fn sedp_builtin_publications_writer(&mut self) -> Option<&mut Self::BuiltinPublicationsWriter> {
        todo!()
    }

    fn sedp_builtin_publications_reader(&mut self) -> Option<&mut Self::BuiltinPublicationsReader> {
        todo!()
    }

    fn sedp_builtin_subscriptions_writer(
        &mut self,
    ) -> Option<&mut Self::BuiltinSubscriptionsWriter> {
        todo!()
    }

    fn sedp_builtin_subscriptions_reader(
        &mut self,
    ) -> Option<&mut Self::BuiltinSubscriptionsReader> {
        todo!()
    }

    fn sedp_builtin_topics_writer(&mut self) -> Option<&mut Self::BuiltinTopicsWriter> {
        todo!()
    }

    fn sedp_builtin_topics_reader(&mut self) -> Option<&mut Self::BuiltinTopicsReader> {
        todo!()
    }
}

impl SPDPdiscoveredParticipantData for RtpsParticipantImpl {
    type LocatorListType = Vec<Locator>;

    fn domain_id(&self) -> rust_rtps_pim::discovery::types::DomainId {
        todo!()
    }

    fn domain_tag(&self) -> &str {
        todo!()
    }

    fn protocol_version(&self) -> ProtocolVersion {
        todo!()
    }

    fn guid_prefix(&self) -> rust_rtps_pim::structure::types::GuidPrefix {
        todo!()
    }

    fn vendor_id(&self) -> VendorId {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }

    fn metatraffic_unicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn metatraffic_multicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn default_unicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn default_multicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn available_builtin_endpoints(&self) -> rust_rtps_pim::discovery::types::BuiltinEndpointSet {
        todo!()
    }

    fn manual_liveliness_count(&self) -> rust_rtps_pim::messages::types::Count {
        todo!()
    }

    fn builtin_endpoint_qos(&self) -> rust_rtps_pim::discovery::types::BuiltinEndpointQos {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use rust_dds_api::infrastructure::qos::PublisherQos;
    // use rust_rtps_pim::structure::types::GUID;
    // use rust_rtps_udp_psm::RtpsUdpPsm;

    // use super::*;

    // #[test]
    // fn add_writer_group() {
    //     let mut participant: RTPSParticipantImpl<RtpsUdpPsm> = RTPSParticipantImpl::new([1;12]);
    //     let guid = GUID::new([1; 12], [0, 0, 0, 1].into());
    //     let shared_writer_group = RtpsShared::new(RTPSWriterGroupImpl::new(
    //         guid,
    //         PublisherQos::default(),
    //         None,
    //         0,
    //     ));
    //     participant.add_writer_group(shared_writer_group);

    //     assert_eq!(participant.rtps_writer_groups.len(), 1)
    // }

    // #[test]
    // fn delete_writer_group() {
    //     let mut participant: RTPSParticipantImpl<RtpsUdpPsm> = RTPSParticipantImpl::new([1;12]);
    //     let guid = GUID::new([1; 12], [0, 0, 0, 1].into());
    //     let shared_writer_group = RtpsShared::new(RTPSWriterGroupImpl::new(
    //         guid,
    //         PublisherQos::default(),
    //         None,
    //         0,
    //     ));
    //     participant.add_writer_group(shared_writer_group.clone());
    //     let instance_handle = crate::utils::instance_handle_from_guid(
    //         &shared_writer_group.lock().guid(),
    //     );
    //     participant
    //         .delete_writer_group(instance_handle)
    //         .unwrap();

    //     assert_eq!(participant.rtps_writer_groups.len(), 0)
    // }

    // #[test]
    // fn delete_not_present_writer_group() {
    //     let mut participant: RTPSParticipantImpl<RtpsUdpPsm> = RTPSParticipantImpl::new([1;12]);
    //     let expected = Err(DDSError::PreconditionNotMet("RTPS writer group not found"));
    //     let result = participant.delete_writer_group(1);

    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn participant_guid() {
    // todo!()
    // let prefix = [1; 12];
    // let rtps_participant: RTPSParticipantImpl<MockPsm> = RTPSParticipantImpl::new([1; 12]);
    // let guid = rtps_participant.guid();

    // assert_eq!(guid.prefix(), &prefix);
    // assert_eq!(
    //     guid.entity_id(),
    //     &<MockPsm as rust_rtps_pim::structure::Types>::ENTITYID_PARTICIPANT
    // );
    // }

    // #[test]
    //     fn demo_participant_test() {
    //         let builtin_subscriber = SubscriberImpl::new(SubscriberQos::default(), None, 0);
    //         let mut builtin_publisher = PublisherImpl::new(PublisherQos::default(), None, 0);

    //         let mut qos = DataWriterQos::default();
    //         qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;
    //         let mut stateless_data_writer = StatelessDataWriterImpl::new(qos);
    //         stateless_data_writer.reader_locator_add(Locator::new(
    //             <RtpsUdpPsm as rust_rtps_pim::structure::Types>::LOCATOR_KIND_UDPv4,
    //             7400,
    //             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
    //         ));
    //         stateless_data_writer.write_w_timestamp().unwrap();

    //         builtin_publisher.stateless_writer_add(stateless_data_writer);

    //         let mut participant = DomainParticipantImpl::new(
    //             0,
    //             DomainParticipantQos::default(),
    //             None,
    //             0,
    //             builtin_subscriber,
    //             builtin_publisher,
    //         );

    //         participant.enable().unwrap();
    //         std::thread::sleep(std::time::Duration::from_secs(1));
    //     }

    //     // #[test]
    //     // fn create_publisher() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };
    //     //configuration
    //     // let builtin_subscriber = SubscriberImpl::new(SubscriberQos::default(), None, 0);
    //     // let mut builtin_publisher = PublisherImpl::new(PublisherQos::default(), None, 0);

    //     // let stateless_data_writer = StatelessDataWriterImpl::new(DataWriterQos::default());
    //     // builtin_publisher.stateless_writer_add(stateless_data_writer);

    //     // let participant = DomainParticipantImpl::new(
    //     //     0,
    //     //     DomainParticipantQos::default(),
    //     //     None,
    //     //     0,
    //     //     builtin_subscriber,
    //     //     builtin_publisher,
    //     // );

    //     //         let qos = Some(PublisherQos::default());
    //     //         let a_listener = None;
    //     //         let mask = 0;
    //     //         participant
    //     //             .create_publisher(qos, a_listener, mask)
    //     //             .expect("Error creating publisher");

    //     //         assert_eq!(
    //     //             participant
    //     //                 .user_defined_entities
    //     //                 .publisher_list
    //     //                 .lock()
    //     //                 .unwrap()
    //     //                 .len(),
    //     //             1
    //     //         );
    //     // }

    //     //     #[test]
    //     //     fn create_delete_publisher() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let qos = Some(PublisherQos::default());
    //     //         let a_listener = None;
    //     //         let mask = 0;
    //     //         let a_publisher = participant.create_publisher(qos, a_listener, mask).unwrap();

    //     //         participant
    //     //             .delete_publisher(&a_publisher)
    //     //             .expect("Error deleting publisher");
    //     //         assert_eq!(
    //     //             participant
    //     //                 .user_defined_entities
    //     //                 .publisher_list
    //     //                 .lock()
    //     //                 .unwrap()
    //     //                 .len(),
    //     //             0
    //     //         );
    //     //     }

    //     //     #[test]
    //     //     fn create_subscriber() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let qos = Some(SubscriberQos::default());
    //     //         let a_listener = None;
    //     //         let mask = 0;
    //     //         participant
    //     //             .create_subscriber(qos, a_listener, mask)
    //     //             .expect("Error creating subscriber");
    //     //         assert_eq!(
    //     //             participant
    //     //                 .user_defined_entities
    //     //                 .subscriber_list
    //     //                 .lock()
    //     //                 .unwrap()
    //     //                 .len(),
    //     //             1
    //     //         );
    //     //     }

    //     //     #[test]
    //     //     fn create_delete_subscriber() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let qos = Some(SubscriberQos::default());
    //     //         let a_listener = None;
    //     //         let mask = 0;
    //     //         let a_subscriber = participant
    //     //             .create_subscriber(qos, a_listener, mask)
    //     //             .unwrap();

    //     //         participant
    //     //             .delete_subscriber(&a_subscriber)
    //     //             .expect("Error deleting subscriber");
    //     //         assert_eq!(
    //     //             participant
    //     //                 .user_defined_entities
    //     //                 .subscriber_list
    //     //                 .lock()
    //     //                 .unwrap()
    //     //                 .len(),
    //     //             0
    //     //         );
    //     //     }

    //     //     #[test]
    //     //     fn create_topic() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let topic_name = "Test";
    //     //         let qos = Some(TopicQos::default());
    //     //         let a_listener = None;
    //     //         let mask = 0;
    //     //         participant
    //     //             .create_topic::<TestType>(topic_name, qos, a_listener, mask)
    //     //             .expect("Error creating topic");
    //     //     }

    //     //     #[test]
    //     //     fn create_delete_topic() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let topic_name = "Test";
    //     //         let qos = Some(TopicQos::default());
    //     //         let a_listener = None;
    //     //         let mask = 0;
    //     //         let a_topic = participant
    //     //             .create_topic::<TestType>(topic_name, qos, a_listener, mask)
    //     //             .unwrap();

    //     //         participant
    //     //             .delete_topic(&a_topic)
    //     //             .expect("Error deleting topic")
    //     //     }

    //     //     #[test]
    //     //     fn set_get_default_publisher_qos() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let mut participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let mut publisher_qos = PublisherQos::default();
    //     //         publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //         participant
    //     //             .set_default_publisher_qos(Some(publisher_qos.clone()))
    //     //             .expect("Error setting default publisher qos");

    //     //         assert_eq!(publisher_qos, participant.get_default_publisher_qos())
    //     //     }

    //     //     #[test]
    //     //     fn set_get_default_subscriber_qos() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let mut participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let mut subscriber_qos = SubscriberQos::default();
    //     //         subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //         participant
    //     //             .set_default_subscriber_qos(Some(subscriber_qos.clone()))
    //     //             .expect("Error setting default subscriber qos");

    //     //         assert_eq!(subscriber_qos, participant.get_default_subscriber_qos())
    //     //     }

    //     //     #[test]
    //     //     fn set_get_default_topic_qos() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let mut participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let mut topic_qos = TopicQos::default();
    //     //         topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
    //     //         participant
    //     //             .set_default_topic_qos(Some(topic_qos.clone()))
    //     //             .expect("Error setting default subscriber qos");

    //     //         assert_eq!(topic_qos, participant.get_default_topic_qos())
    //     //     }

    //     //     #[test]
    //     //     fn set_default_publisher_qos_to_default_value() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let mut participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let mut publisher_qos = PublisherQos::default();
    //     //         publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //         participant
    //     //             .set_default_publisher_qos(Some(publisher_qos.clone()))
    //     //             .unwrap();

    //     //         participant
    //     //             .set_default_publisher_qos(None)
    //     //             .expect("Error setting default publisher qos");

    //     //         assert_eq!(
    //     //             PublisherQos::default(),
    //     //             participant.get_default_publisher_qos()
    //     //         )
    //     //     }

    //     //     #[test]
    //     //     fn set_default_subscriber_qos_to_default_value() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let mut participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let mut subscriber_qos = SubscriberQos::default();
    //     //         subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //         participant
    //     //             .set_default_subscriber_qos(Some(subscriber_qos.clone()))
    //     //             .unwrap();

    //     //         participant
    //     //             .set_default_subscriber_qos(None)
    //     //             .expect("Error setting default subscriber qos");

    //     //         assert_eq!(
    //     //             SubscriberQos::default(),
    //     //             participant.get_default_subscriber_qos()
    //     //         )
    //     //     }

    //     //     #[test]
    //     //     fn set_default_topic_qos_to_default_value() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let mut participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         let mut topic_qos = TopicQos::default();
    //     //         topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
    //     //         participant
    //     //             .set_default_topic_qos(Some(topic_qos.clone()))
    //     //             .unwrap();

    //     //         participant
    //     //             .set_default_topic_qos(None)
    //     //             .expect("Error setting default subscriber qos");

    //     //         assert_eq!(TopicQos::default(), participant.get_default_topic_qos())
    //     //     }

    //     //     #[test]
    //     //     fn enable() {
    //     //         let configuration = DomainParticipantImplConfiguration {
    //     //             userdata_transport: Box::new(MockTransport::default()),
    //     //             metatraffic_transport: Box::new(MockTransport::default()),
    //     //             domain_tag: "",
    //     //             lease_duration: Duration {
    //     //                 seconds: 30,
    //     //                 fraction: 0,
    //     //             },
    //     //             spdp_locator_list: vec![],
    //     //         };

    //     //         let mut participant =
    //     //             DomainParticipantImpl::new(0, DomainParticipantQos::default(), None, 0, configuration);

    //     //         participant.enable().expect("Failed to enable");
    //     //         assert_eq!(participant.thread_list.borrow().len(), 1);
    //     //     }

    //     //     // #[test]
    //     //     // fn create_publisher_factory_default_qos() {
    //     //     //     let participant = DomainParticipantImpl::new(
    //     //     //         0,
    //     //     //         DomainParticipantQos::default(),
    //     //     //         MockTransport::default(),
    //     //     //         MockTransport::default(),
    //     //     //         None,
    //     //     //         0,
    //     //     //     );

    //     //     //     let mut publisher_qos = PublisherQos::default();
    //     //     //     publisher_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //     //     participant
    //     //     //         .set_default_publisher_qos(Some(publisher_qos.clone()))
    //     //     //         .unwrap();

    //     //     //     let qos = None;
    //     //     //     let a_listener = None;
    //     //     //     let mask = 0;
    //     //     //     let publisher = participant
    //     //     //         .create_publisher(qos, a_listener, mask)
    //     //     //         .expect("Error creating publisher");

    //     //     //     assert_eq!(publisher.get_qos().unwrap(), publisher_qos);
    //     //     // }

    //     //     // #[test]
    //     //     // fn create_subscriber_factory_default_qos() {
    //     //     //     let participant = DomainParticipantImpl::new(
    //     //     //         0,
    //     //     //         DomainParticipantQos::default(),
    //     //     //         MockTransport::default(),
    //     //     //         MockTransport::default(),
    //     //     //         None,
    //     //     //         0,
    //     //     //     );

    //     //     //     let mut subscriber_qos = SubscriberQos::default();
    //     //     //     subscriber_qos.group_data.value = vec![b'a', b'b', b'c'];
    //     //     //     participant
    //     //     //         .set_default_subscriber_qos(Some(subscriber_qos.clone()))
    //     //     //         .unwrap();

    //     //     //     let qos = None;
    //     //     //     let a_listener = None;
    //     //     //     let mask = 0;
    //     //     //     let subscriber = participant
    //     //     //         .create_subscriber(qos, a_listener, mask)
    //     //     //         .expect("Error creating publisher");

    //     //     //     assert_eq!(subscriber.get_qos().unwrap(), subscriber_qos);
    //     //     // }

    //     //     // #[test]
    //     //     // fn create_topic_factory_default_qos() {
    //     //     //     let participant = DomainParticipantImpl::new(
    //     //     //         0,
    //     //     //         DomainParticipantQos::default(),
    //     //     //         MockTransport::default(),
    //     //     //         MockTransport::default(),
    //     //     //         None,
    //     //     //         0,
    //     //     //     );

    //     //     //     let mut topic_qos = TopicQos::default();
    //     //     //     topic_qos.topic_data.value = vec![b'a', b'b', b'c'];
    //     //     //     participant
    //     //     //         .set_default_topic_qos(Some(topic_qos.clone()))
    //     //     //         .unwrap();

    //     //     //     let qos = None;
    //     //     //     let a_listener = None;
    //     //     //     let mask = 0;
    //     //     //     let topic = participant
    //     //     //         .create_topic::<TestType>("name", qos, a_listener, mask)
    //     //     //         .expect("Error creating publisher");

    //     //     //     assert_eq!(topic.get_qos().unwrap(), topic_qos);
    //     //     // }
}
