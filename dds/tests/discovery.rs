use std::net::UdpSocket;

use rust_dds::{
    communication::Communication,
    infrastructure::qos::{DataReaderQos, SubscriberQos, TopicQos},
    udp_transport::UdpTransport, publication::data_writer::{DataWriter}, subscription::data_reader::{DataReader},
};
use rust_dds_api::{
    builtin_topics::ParticipantBuiltinTopicData,
    dcps_psm::BuiltInTopicKey,
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        qos_policy::UserDataQosPolicy,
    },
};
use rust_dds_rtps_implementation::{
    data_representation_builtin_endpoints::{
        sedp_discovered_writer_data::SedpDiscoveredWriterData,
        spdp_discovered_participant_data::{ParticipantProxy, SpdpDiscoveredParticipantData},
    },
    dds_impl::{
        data_reader_proxy::{DataReaderAttributes, RtpsReader, Samples, DataReaderProxy},
        data_writer_proxy::{DataWriterAttributes, RtpsWriter, DataWriterProxy},
        publisher_proxy::PublisherAttributes,
        subscriber_proxy::SubscriberAttributes,
        topic_proxy::TopicAttributes,
    },
    dds_type::DdsType,
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl, rtps_reader_locator_impl::RtpsReaderLocatorAttributesImpl,
        rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_reader_impl::RtpsStatelessReaderImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
    utils::shared_object::{RtpsWeak, RtpsShared},
};
use rust_rtps_pim::{
    behavior::{
        writer::reader_locator::RtpsReaderLocatorConstructor,
        writer::stateless_writer::RtpsStatelessWriterOperations,
    },
    discovery::{
        sedp::builtin_endpoints::{SedpBuiltinPublicationsReader, SedpBuiltinPublicationsWriter},
        spdp::builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::types::{
        EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, ProtocolVersion,
        BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP, GUID_UNKNOWN, PROTOCOLVERSION, VENDOR_ID_S2E,
    },
};

#[test]
fn send_and_receive_discovery_data_happy_path() {
    let guid_prefix = GuidPrefix([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]);

    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey {
            value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        },
        user_data: UserDataQosPolicy { value: vec![] },
    };

    let participant_proxy = ParticipantProxy {
        domain_id: 1,
        domain_tag: "ab".to_string(),
        protocol_version: ProtocolVersion { major: 1, minor: 4 },
        guid_prefix,
        vendor_id: [73, 74],
        expects_inline_qos: false,
        metatraffic_unicast_locator_list: vec![Locator::new(11, 12, [1; 16])],
        metatraffic_multicast_locator_list: vec![],
        default_unicast_locator_list: vec![],
        default_multicast_locator_list: vec![],
        available_builtin_endpoints: BuiltinEndpointSet::new(
            BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
            | BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR,
        ),
        manual_liveliness_count: Count(0),
        builtin_endpoint_qos: BuiltinEndpointQos::default(),
    };

    let lease_duration = rust_rtps_pim::behavior::types::Duration {
        seconds: 100,
        fraction: 0,
    };

    let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
        dds_participant_data,
        participant_proxy,
        lease_duration,
    };

    let mut spdp_builtin_participant_rtps_writer = SpdpBuiltinParticipantWriter::create::<
        RtpsStatelessWriterImpl,
    >(GuidPrefix([3; 12]), &[], &[]);

    let spdp_discovery_locator = RtpsReaderLocatorAttributesImpl::new(
        Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        ),
        false,
    );

    spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

    let publisher = RtpsShared::new(PublisherAttributes::new(
        PublisherQos::default(),
        RtpsGroupImpl::new(GUID_UNKNOWN),
        None,
        RtpsWeak::new(),
    ));

    let data_writer = DataWriterAttributes::new(
        DataWriterQos::default(),
        RtpsWriter::Stateless(spdp_builtin_participant_rtps_writer),
        RtpsWeak::new(),
        publisher.downgrade(),
    );

    let mut data_writer_proxy = {
        let data_writer_ptr = RtpsShared::new(data_writer);
        let data_writer_weak = data_writer_ptr.downgrade();
        publisher.write().unwrap().data_writer_list.push(data_writer_ptr);

        DataWriterProxy::new(data_writer_weak)
    };

    data_writer_proxy
        .write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
        )
        .unwrap();

    let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
    socket.set_nonblocking(true).unwrap();
    let transport = UdpTransport::new(socket);
    let mut communication = Communication {
        version: PROTOCOLVERSION,
        vendor_id: VENDOR_ID_S2E,
        guid_prefix,
        transport,
    };

    communication.send(core::slice::from_ref(&publisher));

    // Reception

    let spdp_builtin_participant_rtps_reader_impl = SpdpBuiltinParticipantReader::create::<
        RtpsStatelessReaderImpl,
    >(GuidPrefix([5; 12]), &[], &[]);

    let subscriber = RtpsShared::new(SubscriberAttributes::new(
        SubscriberQos::default(),
        RtpsGroupImpl::new(Guid::new(
            GuidPrefix([6; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
        )),
        RtpsWeak::new(),
    ));

    let data_reader = DataReaderAttributes::new(
        DataReaderQos::default(),
        RtpsReader::Stateless(spdp_builtin_participant_rtps_reader_impl),
        RtpsShared::new(TopicAttributes::new(
            TopicQos::default(),
            SpdpDiscoveredParticipantData::type_name(),
            "DCPSParticipant",
            RtpsWeak::new(),
        )),
        subscriber.downgrade(),
    );

    let mut data_reader_proxy = {
        let data_reader_ptr = RtpsShared::new(data_reader);
        let data_reader_weak = data_reader_ptr.downgrade();
        subscriber.write().unwrap().data_reader_list.push(data_reader_ptr);

        DataReaderProxy::new(data_reader_weak)
    };

    communication.receive(core::slice::from_ref(&subscriber));

    let result: Samples<SpdpDiscoveredParticipantData> =
        data_reader_proxy.read(1, &[], &[], &[]).unwrap();
    assert_eq!(result[0].participant_proxy.domain_id, 1);
    assert_eq!(result[0].participant_proxy.domain_tag, "ab");
}

#[test]
fn process_discovery_data_happy_path() {
    let guid_prefix = GuidPrefix([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]);
    let domain_id = 1;
    let domain_tag = "ab";

    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey {
            value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        },
        user_data: UserDataQosPolicy { value: vec![] },
    };

    let participant_proxy = ParticipantProxy {
        domain_id,
        domain_tag: domain_tag.to_string(),
        protocol_version: ProtocolVersion { major: 1, minor: 4 },
        guid_prefix,
        vendor_id: [73, 74],
        expects_inline_qos: false,
        metatraffic_unicast_locator_list: vec![Locator::new(
            LOCATOR_KIND_UDPv4,
            7405,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        )],
        metatraffic_multicast_locator_list: vec![],
        default_unicast_locator_list: vec![],
        default_multicast_locator_list: vec![],
        available_builtin_endpoints: BuiltinEndpointSet::new(
            BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
                | BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
                | BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER
                | BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR,
        ),
        manual_liveliness_count: Count(0),
        builtin_endpoint_qos: BuiltinEndpointQos::default(),
    };

    let lease_duration = rust_rtps_pim::behavior::types::Duration {
        seconds: 100,
        fraction: 0,
    };

    let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
        dds_participant_data,
        participant_proxy,
        lease_duration,
    };

    let mut spdp_builtin_participant_rtps_writer = SpdpBuiltinParticipantWriter::create::<
        RtpsStatelessWriterImpl,
    >(GuidPrefix([3; 12]), &[], &[]);

    let spdp_discovery_locator = RtpsReaderLocatorAttributesImpl::new(
        Locator::new(
            LOCATOR_KIND_UDPv4,
            7402,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        ),
        false,
    );

    spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

    let spdp_builtin_participant_data_writer = DataWriterAttributes::new(
        DataWriterQos::default(),
        RtpsWriter::Stateless(spdp_builtin_participant_rtps_writer),
        RtpsWeak::new(),
        RtpsWeak::new(),
    );

    let spdp_builtin_participant_data_writer_ptr = RtpsShared::new(spdp_builtin_participant_data_writer);

    let mut spdp_builtin_participant_data_writer_proxy = DataWriterProxy::new(
        spdp_builtin_participant_data_writer_ptr.downgrade()
    );

    spdp_builtin_participant_data_writer_proxy
        .write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
        )
        .unwrap();

    let sedp_builtin_publications_rtps_writer =
        SedpBuiltinPublicationsWriter::create::<RtpsStatefulWriterImpl>(guid_prefix, &[], &[]);

    let _sedp_builtin_publications_data_writer = RtpsShared::new(DataWriterAttributes::new(
        DataWriterQos::default(),
        RtpsWriter::Stateful(sedp_builtin_publications_rtps_writer),
        RtpsWeak::new(),
        RtpsWeak::new(),
    ));

    let publisher = RtpsShared::new(PublisherAttributes::new(
        PublisherQos::default(),
        RtpsGroupImpl::new(Guid::new(
            GuidPrefix([4; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
        )),
        None,
        RtpsWeak::new(),
    ));

    // vec![
    //         RtpsShared::new(spdp_builtin_participant_data_writer),
    //         sedp_builtin_publications_data_writer.clone(),
    //     ],

    let socket = UdpSocket::bind("127.0.0.1:7402").unwrap();
    socket.set_nonblocking(true).unwrap();
    let transport = UdpTransport::new(socket);
    let mut communication = Communication {
        version: PROTOCOLVERSION,
        vendor_id: VENDOR_ID_S2E,
        guid_prefix,
        transport,
    };
    communication.send(core::slice::from_ref(&publisher));

    // Reception

    let spdp_builtin_participant_rtps_reader_impl = SpdpBuiltinParticipantReader::create::<
        RtpsStatelessReaderImpl,
    >(GuidPrefix([5; 12]), &[], &[]);

    let spdp_builtin_participant_data_reader = DataReaderAttributes::new(
        DataReaderQos::default(),
        RtpsReader::Stateless(spdp_builtin_participant_rtps_reader_impl),
        RtpsShared::new(TopicAttributes::new(
            TopicQos::default(),
            SpdpDiscoveredParticipantData::type_name(),
            "DCPSParticipant",
            RtpsWeak::new(),
        )),
        RtpsWeak::new(),
    );
    let shared_data_reader = RtpsShared::new(spdp_builtin_participant_data_reader);
    let subscriber = RtpsShared::new(SubscriberAttributes::new(
        SubscriberQos::default(),
        RtpsGroupImpl::new(Guid::new(
            GuidPrefix([6; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
        )),
        RtpsWeak::new(),
    ));
    // vec![shared_data_reader.clone()],

    communication.receive(core::slice::from_ref(&subscriber));

    communication.receive(core::slice::from_ref(&subscriber));
    let mut _shared_data_reader = shared_data_reader.write_lock();

    // let discovered_participant: Samples<SpdpDiscoveredParticipantData> =
    //     shared_data_reader.read(1, &[], &[], &[]).unwrap();

    let sedp_builtin_publications_rtps_reader =
        SedpBuiltinPublicationsReader::create::<RtpsStatefulReaderImpl>(guid_prefix, &[], &[]);
    let mut _sedp_built_publications_reader = DataReaderAttributes::new(
        DataReaderQos::default(),
        RtpsReader::Stateful(sedp_builtin_publications_rtps_reader),
        RtpsShared::new(TopicAttributes::new(
            TopicQos::default(),
            SedpDiscoveredWriterData::type_name(),
            "DCPSPublication",
            RtpsWeak::new(),
        )),
        RtpsWeak::new(),
    );

    // if let Ok(participant_discovery) = ParticipantDiscovery::new(
    //     &discovered_participant[0].participant_proxy,
    //     &domain_id,
    //     domain_tag,
    // ) {
    //     if let RtpsReader::Stateful(sedp_built_publications_reader) =
    //         sedp_built_publications_reader.as_mut()
    //     {
    //         participant_discovery
    //             .discovered_participant_add_publications_reader(sedp_built_publications_reader);
    //     }

    //     let mut sedp_builtin_publications_data_writer_lock =
    //         sedp_builtin_publications_data_writer.write().unwrap();

    //     participant_discovery.discovered_participant_add_publications_writer(
    //         sedp_builtin_publications_data_writer_lock
    //             .as_mut()
    //             .try_as_stateful_writer()
    //             .unwrap(),
    //     );
    //     let sedp_discovered_writer_data = SedpDiscoveredWriterData {
    //         writer_proxy: RtpsWriterProxy {
    //             remote_writer_guid: Guid::new(
    //                 GuidPrefix([1; 12]),
    //                 ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    //             ),
    //             unicast_locator_list: vec![],
    //             multicast_locator_list: vec![],
    //             data_max_size_serialized: None,
    //             remote_group_entity_id: EntityId::new([0; 3], 0),
    //         },
    //         publication_builtin_topic_data: PublicationBuiltinTopicData {
    //             key: BuiltInTopicKey { value: [1; 16] },
    //             participant_key: BuiltInTopicKey { value: [1; 16] },
    //             topic_name: "MyTopic".to_string(),
    //             type_name: "MyType".to_string(),
    //             durability: DurabilityQosPolicy::default(),
    //             durability_service: DurabilityServiceQosPolicy::default(),
    //             deadline: DeadlineQosPolicy::default(),
    //             latency_budget: LatencyBudgetQosPolicy::default(),
    //             liveliness: LivelinessQosPolicy::default(),
    //             reliability: ReliabilityQosPolicy {
    //                 kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
    //                 max_blocking_time: Duration::new(3, 0),
    //             },
    //             lifespan: LifespanQosPolicy::default(),
    //             user_data: UserDataQosPolicy::default(),
    //             ownership: OwnershipQosPolicy::default(),
    //             ownership_strength: OwnershipStrengthQosPolicy::default(),
    //             destination_order: DestinationOrderQosPolicy::default(),
    //             presentation: PresentationQosPolicy::default(),
    //             partition: PartitionQosPolicy::default(),
    //             topic_data: TopicDataQosPolicy::default(),
    //             group_data: GroupDataQosPolicy::default(),
    //         },
    //     };

    // sedp_builtin_publications_data_writer_lock
    //     .write_w_timestamp(
    //         &sedp_discovered_writer_data,
    //         None,
    //         rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
    //     )
    //     .unwrap();
    // }

    // assert_eq!(sedp_built_publications_reader.matched_writers.len(), 1);
    // assert_eq!(
    //     sedp_built_publications_reader.matched_writers[0].remote_writer_guid,
    //     Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
    // );

    for _i in 1..14 {
        communication.send(core::slice::from_ref(&publisher));

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
