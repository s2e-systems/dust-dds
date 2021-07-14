use rust_dds_api::{
    dcps_psm::InstanceHandle, infrastructure::qos::SubscriberQos, return_type::DDSResult,
};
use rust_rtps_pim::structure::{RTPSEntity, ReaderGroupCollection, WriterGroupCollection, types::{EntityId, Locator, ProtocolVersion, VendorId, GUID, PROTOCOLVERSION_2_4}};

use crate::utils::shared_object::{RtpsLock, RtpsShared};

use super::{
    rtps_reader_group_impl::RTPSReaderGroupImpl, rtps_writer_group_impl::RTPSWriterGroupImpl,
};

pub struct RTPSParticipantImpl {
    guid: GUID,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    rtps_writer_groups: Vec<RtpsShared<RTPSWriterGroupImpl>>,
    rtps_reader_groups: Vec<RtpsShared<RTPSReaderGroupImpl>>,
    pub builtin_writer_group: RtpsShared<RTPSWriterGroupImpl>,
    pub builtin_reader_group: RtpsShared<RTPSReaderGroupImpl>,
}

impl RTPSParticipantImpl {
    pub fn new(guid_prefix: rust_rtps_pim::structure::types::GuidPrefix) -> Self {
        let guid = GUID::new(
            guid_prefix,
            rust_rtps_pim::structure::types::ENTITYID_PARTICIPANT,
        );

        let builtin_writer_group_guid = GUID::new(
            guid_prefix,
            EntityId {
                entity_key: [0, 0, 0],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::BuiltInWriterGroup,
            },
        );
        let builtin_writer_group = RtpsShared::new(RTPSWriterGroupImpl::new(
            builtin_writer_group_guid,
            rust_dds_api::infrastructure::qos::PublisherQos::default(),
            None,
            0,
        ));

        let builtin_reader_group_guid = GUID::new(
            guid_prefix,
            EntityId {
                entity_key: [0, 0, 0],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::BuiltInReaderGroup,
            },
        );
        let builtin_reader_group = RtpsShared::new(RTPSReaderGroupImpl::new(
            builtin_reader_group_guid,
            SubscriberQos::default(),
            None,
            0,
        ));

        Self {
            guid,
            protocol_version: PROTOCOLVERSION_2_4,
            vendor_id: [99, 99],
            rtps_writer_groups: Vec::new(),
            rtps_reader_groups: Vec::new(),
            builtin_writer_group,
            builtin_reader_group,
        }
    }

    pub fn writer_groups(&self) -> &[RtpsShared<RTPSWriterGroupImpl>] {
        &self.rtps_writer_groups
    }

    pub fn add_writer_group(&mut self, writer_group: RtpsShared<RTPSWriterGroupImpl>) {
        self.rtps_writer_groups.push(writer_group)
    }

    pub fn delete_writer_group(&mut self, _writer_group: InstanceHandle) -> DDSResult<()> {
        todo!()
        // let index = self
        //     .rtps_writer_groups
        //     .iter()
        //     .position(|x| crate::utils::instance_handle_from_guid(&x.lock().guid()) == writer_group)
        //     .ok_or(DDSError::PreconditionNotMet("RTPS writer group not found"))?;
        // self.rtps_writer_groups.swap_remove(index);
        // Ok(())
    }
}

// pub struct ReaderGroupIntoIter;

// impl IntoIterator for ReaderGroupIntoIter {
//     type Item = RTPSReaderGroupImpl;
//     type IntoIter = RtpsReaderGroupIterator;

//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }

// pub struct RtpsReaderGroupIterator;

// impl Iterator for RTPSReaderGroupImpl {
//     type Item = &'a mut RTPSReaderGroupImpl;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

impl<'a> rust_rtps_pim::structure::RTPSParticipant for RtpsLock<'a, RTPSParticipantImpl> {
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

impl<'a> ReaderGroupCollection for &'a mut RTPSParticipantImpl {
    type ReaderGroupsType = std::vec::IntoIter<RtpsLock<'a, RTPSReaderGroupImpl>>;

    fn reader_groups(self) -> Self::ReaderGroupsType {
        todo!()
    }
}

impl<'a> WriterGroupCollection for &'a mut RTPSParticipantImpl {
    type WriterGroupsType = ();

    fn writer_groups(self) -> Self::WriterGroupsType {
        todo!()
    }
}

impl RTPSEntity for RtpsLock<'_, RTPSParticipantImpl> {
    fn guid(&self) -> &GUID {
        &self.guid
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
