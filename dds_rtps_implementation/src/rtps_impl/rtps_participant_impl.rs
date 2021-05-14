use rust_dds_api::{
    dcps_psm::{InstanceHandle, HANDLE_NIL_NATIVE},
    infrastructure::{
        entity::Entity,
        qos::{SubscriberQos, TopicQos},
    },
};
use rust_rtps_pim::structure::RTPSEntity;

use crate::utils::shared_object::RtpsShared;

use super::rtps_writer_group_impl::RTPSWriterGroupImpl;

const ENTITYKIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;
const ENTITYKIND_USER_DEFINED_READER_GROUP: u8 = 0x09;

pub struct RTPSParticipantImpl<'a, PSM: rust_rtps_pim::PIM> {
    unicast_locator_list: Vec<rust_rtps_pim::structure::types::Locator<PSM>>,
    multicast_locator_list: Vec<rust_rtps_pim::structure::types::Locator<PSM>>,
    rtps_writer_groups: Vec<RtpsShared<RTPSWriterGroupImpl<'a, PSM>>>,
    // builtin_writer_group: Arc<Mutex<RTPSWriterGroupImpl<'a, PSM>>>,
    default_subscriber_qos: SubscriberQos<'a>,
    default_topic_qos: TopicQos<'a>,
}

impl<'a, PSM: rust_rtps_pim::PIM> RTPSParticipantImpl<'a, PSM> {
    pub fn new() -> Self {
        // let guid = GUID::new(prefix, PSM::ENTITYID_PARTICIPANT);
        // let builtin_writer_group = Arc::new(Mutex::new(RTPSWriterGroupImpl::new(
        //     GUID::new(prefix, [0, 0, 0, 0xc8].into()),
        //     PublisherQos::default(),
        //     None,
        //     0,
        // )));

        Self {
            unicast_locator_list: Vec::new(),
            multicast_locator_list: Vec::new(),
            rtps_writer_groups: Vec::new(),
            // builtin_writer_group,
            default_subscriber_qos: SubscriberQos::default(),
            default_topic_qos: TopicQos::default(),
        }
    }

    pub fn add_writer_group(&mut self, writer_group: RtpsShared<RTPSWriterGroupImpl<'a, PSM>>) {
        self.rtps_writer_groups.push(writer_group)
    }

    pub fn delete_writer_group(&mut self, writer_group: InstanceHandle) {
        self.rtps_writer_groups
            .retain(|x| x.get_instance_handle().unwrap_or(HANDLE_NIL_NATIVE) != writer_group);
    }
}

impl<'a, PSM: rust_rtps_pim::PIM> rust_rtps_pim::structure::RTPSParticipant<PSM>
    for RTPSParticipantImpl<'a, PSM>
{
    fn protocol_version(&self) -> PSM::ProtocolVersion {
        todo!()
    }

    fn vendor_id(&self) -> PSM::VendorId {
        todo!()
    }

    fn default_unicast_locator_list(&self) -> &[PSM::Locator] {
        todo!()
    }

    fn default_multicast_locator_list(&self) -> &[PSM::Locator] {
        todo!()
    }
}

impl<'a, PSM: rust_rtps_pim::PIM> RTPSEntity<PSM> for RTPSParticipantImpl<'a, PSM> {
    fn guid(&self) -> rust_rtps_pim::structure::types::GUID<PSM> {
        // self.guid
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_dds_api::infrastructure::qos::PublisherQos;
    use rust_rtps_pim::{messages::submessage_elements::Parameter, structure::types::GUID};
    use rust_rtps_udp_psm::RtpsUdpPsm;

    use super::*;

    struct MockPsm;

    impl rust_rtps_pim::structure::Types for MockPsm {
        type GuidPrefix = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];

        type EntityId = [u8; 4];
        const ENTITYID_UNKNOWN: Self::EntityId = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityId = [1; 4];

        type SequenceNumber = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = -1;

        type LocatorKind = i8;
        type LocatorPort = u16;
        type LocatorAddress = [u8; 16];

        const LOCATOR_KIND_INVALID: Self::LocatorKind = -1;
        const LOCATOR_KIND_RESERVED: Self::LocatorKind = 0;
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv4: Self::LocatorKind = 1;
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv6: Self::LocatorKind = 2;
        const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress = [0; 16];
        const LOCATOR_PORT_INVALID: Self::LocatorPort = 0;

        type InstanceHandle = i32;

        type ProtocolVersion = [u8; 2];

        const PROTOCOLVERSION: Self::ProtocolVersion = Self::PROTOCOLVERSION_2_4;
        const PROTOCOLVERSION_1_0: Self::ProtocolVersion = [1, 0];
        const PROTOCOLVERSION_1_1: Self::ProtocolVersion = [1, 1];
        const PROTOCOLVERSION_2_0: Self::ProtocolVersion = [2, 0];
        const PROTOCOLVERSION_2_1: Self::ProtocolVersion = [2, 1];
        const PROTOCOLVERSION_2_2: Self::ProtocolVersion = [2, 2];
        const PROTOCOLVERSION_2_3: Self::ProtocolVersion = [2, 3];
        const PROTOCOLVERSION_2_4: Self::ProtocolVersion = [2, 4];

        type VendorId = u8;

        const VENDOR_ID_UNKNOWN: Self::VendorId = 0;

        type Data = Vec<u8>;
        type SequenceNumberVector = Vec<i64>;

        type Locator = u8;
        type LocatorVector = Vec<u8>;

        type Parameter = MockParameter;
        type ParameterVector = Vec<MockParameter>;
    }

    impl rust_rtps_pim::messages::Types for MockPsm {
        type ProtocolId = u8;
        const PROTOCOL_RTPS: Self::ProtocolId = 0;

        type SubmessageFlag = bool;

        type SubmessageKind = u8;

        const DATA: Self::SubmessageKind = 1;
        const GAP: Self::SubmessageKind = 2;
        const HEARTBEAT: Self::SubmessageKind = 3;
        const ACKNACK: Self::SubmessageKind = 4;
        const PAD: Self::SubmessageKind = 5;
        const INFO_TS: Self::SubmessageKind = 6;
        const INFO_REPLY: Self::SubmessageKind = 7;
        const INFO_DST: Self::SubmessageKind = 8;
        const INFO_SRC: Self::SubmessageKind = 9;
        const DATA_FRAG: Self::SubmessageKind = 10;
        const NACK_FRAG: Self::SubmessageKind = 11;
        const HEARTBEAT_FRAG: Self::SubmessageKind = 12;

        type Time = i64;

        const TIME_ZERO: Self::Time = 0;
        const TIME_INVALID: Self::Time = -1;
        const TIME_INFINITE: Self::Time = i64::MAX;

        type Count = u32;

        type ParameterId = u8;

        type FragmentNumber = u8;

        type GroupDigest = u8;

        type FragmentNumberVector = Vec<u8>;
    }

    struct MockParameter;

    impl Parameter for MockParameter {
        type PSM = MockPsm;

        fn parameter_id(&self) -> u8 {
            todo!()
        }

        fn length(&self) -> i16 {
            todo!()
        }

        fn value(&self) -> &[u8] {
            todo!()
        }
    }

    #[test]
    fn add_writer_group() {
        let mut participant: RTPSParticipantImpl<RtpsUdpPsm> = RTPSParticipantImpl::new();
        let guid = GUID::new([1; 12], [0, 0, 0, 1].into());
        let shared_writer_group = RtpsShared::new(RTPSWriterGroupImpl::new(
            guid,
            PublisherQos::default(),
            None,
            0,
        ));
        participant.add_writer_group(shared_writer_group);

        assert_eq!(participant.rtps_writer_groups.len(), 1)
    }

    #[test]
    fn delete_writer_group() {
        let mut participant: RTPSParticipantImpl<RtpsUdpPsm> = RTPSParticipantImpl::new();
        let guid = GUID::new([1; 12], [0, 0, 0, 1].into());
        let shared_writer_group = RtpsShared::new(RTPSWriterGroupImpl::new(
            guid,
            PublisherQos::default(),
            None,
            0,
        ));
        participant.add_writer_group(shared_writer_group.clone());
        participant.delete_writer_group(shared_writer_group.get_instance_handle().unwrap());

        assert_eq!(participant.rtps_writer_groups.len(), 0)
    }

    #[test]
    fn participant_guid() {
        todo!()
        // let prefix = [1; 12];
        // let rtps_participant: RTPSParticipantImpl<MockPsm> = RTPSParticipantImpl::new([1; 12]);
        // let guid = rtps_participant.guid();

        // assert_eq!(guid.prefix(), &prefix);
        // assert_eq!(
        //     guid.entity_id(),
        //     &<MockPsm as rust_rtps_pim::structure::Types>::ENTITYID_PARTICIPANT
        // );
    }

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
