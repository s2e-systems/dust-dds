use std::net::UdpSocket;

use rust_dds::{
    communication::Communication,
    data_representation_builtin_endpoints::{
        sedp_discovered_writer_data::{RtpsWriterProxy, SedpDiscoveredWriterData},
        spdp_discovered_participant_data::{ParticipantProxy, SpdpDiscoveredParticipantData},
    },
    domain_participant_factory::{RtpsStructureImpl, DomainParticipantFactory},
    infrastructure::{
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, TopicDataQosPolicy,
        },
    },
    publication::data_writer::DataWriter,
    subscription::data_reader::DataReader,
    types::{Duration},
    udp_transport::UdpTransport, domain::domain_participant::DomainParticipant,
};
use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, PublicationBuiltinTopicData},
    dcps_psm::BuiltInTopicKey,
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        qos_policy::UserDataQosPolicy,
    },
};
use rust_dds_rtps_implementation::{
    dds_impl::{
        data_reader_proxy::{DataReaderAttributes, DataReaderProxy, RtpsReader, Samples},
        data_writer_proxy::{DataWriterAttributes, DataWriterProxy, RtpsWriter},
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
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl, rtps_participant_impl::RtpsParticipantImpl,
    },
    utils::{shared_object::{RtpsShared, RtpsWeak}, rtps_structure::RtpsStructure},
};
use rust_rtps_pim::{
    behavior::{
        writer::reader_locator::RtpsReaderLocatorConstructor,
        writer::stateless_writer::RtpsStatelessWriterOperations,
    },
    discovery::{
        participant_discovery::ParticipantDiscovery,
        sedp::builtin_endpoints::{
            SedpBuiltinPublicationsReader, SedpBuiltinPublicationsWriter,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        },
        spdp::builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::{types::{
        EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator,
        BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP, GUID_UNKNOWN, PROTOCOLVERSION, VENDOR_ID_S2E,
    }, group::RtpsGroupConstructor},
};

#[test]
fn send_and_receive_discovery_data_happy_path() {
    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey {
            value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        },
        user_data: UserDataQosPolicy { value: vec![] },
    };

    let participant_proxy = ParticipantProxy {
        domain_id: 1,
        domain_tag: "ab".to_string(),
        protocol_version: PROTOCOLVERSION,
        guid_prefix: GuidPrefix([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]),
        vendor_id: VENDOR_ID_S2E,
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
            8000,
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

    let spdp_discovered_participant_topic = RtpsShared::new(TopicAttributes::new(
        TopicQos::default(),
        SpdpDiscoveredParticipantData::type_name(),
        "DCPSParticipant",
        RtpsWeak::new(),
    ));

    let mut data_writer_proxy = {
        let data_writer = DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateless(spdp_builtin_participant_rtps_writer),
            spdp_discovered_participant_topic,
            publisher.downgrade(),
        );

        let shared_data_writer = RtpsShared::new(data_writer);
        let weak_data_writer = shared_data_writer.downgrade();
        publisher
            .write_lock()
            .data_writer_list
            .push(shared_data_writer);

        DataWriterProxy::<_, RtpsStructureImpl>::new(weak_data_writer)
    };

    data_writer_proxy
        .write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
        )
        .unwrap();

    let socket = UdpSocket::bind("127.0.0.1:8000").unwrap();
    socket.set_nonblocking(true).unwrap();
    let transport = UdpTransport::new(socket);
    let mut communication = Communication {
        version: PROTOCOLVERSION,
        vendor_id: VENDOR_ID_S2E,
        guid_prefix: GuidPrefix([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]),
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

    let mut data_reader_proxy = {
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

        let shared_data_reader = RtpsShared::new(data_reader);
        let weak_data_reader = shared_data_reader.downgrade();
        subscriber
            .write_lock()
            .data_reader_list
            .push(shared_data_reader);

        DataReaderProxy::<_, RtpsStructureImpl>::new(weak_data_reader)
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
        protocol_version: PROTOCOLVERSION,
        guid_prefix,
        vendor_id: VENDOR_ID_S2E,
        expects_inline_qos: false,
        metatraffic_unicast_locator_list: vec![Locator::new(
            LOCATOR_KIND_UDPv4,
            8010,
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

    let publisher = RtpsShared::new(PublisherAttributes::new(
        PublisherQos::default(),
        RtpsGroupImpl::new(Guid::new(
            GuidPrefix([4; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
        )),
        None,
        RtpsWeak::new(),
    ));

    let mut spdp_builtin_participant_data_writer_proxy = {
        let mut spdp_builtin_participant_rtps_writer = SpdpBuiltinParticipantWriter::create::<
            RtpsStatelessWriterImpl,
        >(GuidPrefix([3; 12]), &[], &[]);

        let spdp_discovery_locator = RtpsReaderLocatorAttributesImpl::new(
            Locator::new(
                LOCATOR_KIND_UDPv4,
                8008,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
            ),
            false,
        );

        spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

        let spdp_discovered_participant_topic = RtpsShared::new(TopicAttributes::new(
            TopicQos::default(),
            SpdpDiscoveredParticipantData::type_name(),
            "DCPSParticipant",
            RtpsWeak::new(),
        ));

        let data_writer = DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateless(spdp_builtin_participant_rtps_writer),
            spdp_discovered_participant_topic.clone(),
            publisher.downgrade(),
        );

        let shared_data_writer = RtpsShared::new(data_writer);
        let weak_data_writer = shared_data_writer.downgrade();
        publisher
            .write_lock()
            .data_writer_list
            .push(shared_data_writer);

        DataWriterProxy::<_, RtpsStructureImpl>::new(weak_data_writer)
    };

    spdp_builtin_participant_data_writer_proxy
        .write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
        )
        .unwrap();

    let mut sedp_builtin_publications_data_writer_proxy = {
        let sedp_builtin_publications_rtps_writer =
            SedpBuiltinPublicationsWriter::create::<RtpsStatefulWriterImpl>(guid_prefix, &[], &[]);

        let sedp_builtin_publications_topic = RtpsShared::new(TopicAttributes::new(
            TopicQos::default(),
            SedpDiscoveredWriterData::type_name(),
            "DCPSPublication",
            RtpsWeak::new(),
        ));

        let data_writer = DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateful(sedp_builtin_publications_rtps_writer),
            sedp_builtin_publications_topic,
            publisher.downgrade(),
        );

        let shared_data_writer = RtpsShared::new(data_writer);
        let weak_data_writer = shared_data_writer.downgrade();
        publisher
            .write_lock()
            .data_writer_list
            .push(shared_data_writer);

        DataWriterProxy::new(weak_data_writer)
    };

    let socket = UdpSocket::bind("127.0.0.1:8008").unwrap();
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

    let subscriber = RtpsShared::new(SubscriberAttributes::new(
        SubscriberQos::default(),
        RtpsGroupImpl::new(Guid::new(
            GuidPrefix([6; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
        )),
        RtpsWeak::new(),
    ));

    let spdp_builtin_participant_rtps_reader_impl = SpdpBuiltinParticipantReader::create::<
        RtpsStatelessReaderImpl,
    >(GuidPrefix([5; 12]), &[], &[]);

    let mut spdp_builtin_participant_data_reader_proxy = {
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

        let shared_data_reader = RtpsShared::new(data_reader);
        let weak_data_reader = shared_data_reader.downgrade();
        subscriber
            .write_lock()
            .data_reader_list
            .push(shared_data_reader);

        DataReaderProxy::new(weak_data_reader)
    };

    communication.receive(core::slice::from_ref(&subscriber));

    let discovered_participant: Samples<SpdpDiscoveredParticipantData> =
        spdp_builtin_participant_data_reader_proxy
            .read(1, &[], &[], &[])
            .unwrap();

    let sedp_builtin_publications_rtps_reader =
        SedpBuiltinPublicationsReader::create::<RtpsStatefulReaderImpl>(guid_prefix, &[], &[]);

    let sedp_built_publications_reader_proxy = {
        let data_reader = DataReaderAttributes::new(
            DataReaderQos::default(),
            RtpsReader::Stateful(sedp_builtin_publications_rtps_reader),
            RtpsShared::new(TopicAttributes::new(
                TopicQos::default(),
                SedpDiscoveredWriterData::type_name(),
                "DCPSPublication",
                RtpsWeak::new(),
            )),
            subscriber.downgrade(),
        );

        let shared_data_reader = RtpsShared::new(data_reader);
        let weak_data_reader = shared_data_reader.downgrade();
        subscriber
            .write_lock()
            .data_reader_list
            .push(shared_data_reader);

        DataReaderProxy::<SedpBuiltinPublicationsReader, RtpsStructureImpl>::new(weak_data_reader)
    };

    if let Ok(participant_discovery) = ParticipantDiscovery::new(
        &discovered_participant[0].participant_proxy,
        &domain_id,
        domain_tag,
    ) {
        if let RtpsReader::Stateful(sedp_built_publications_reader) =
            &mut sedp_built_publications_reader_proxy
                .as_ref()
                .upgrade()
                .unwrap()
                .write_lock()
                .rtps_reader
        {
            participant_discovery
                .discovered_participant_add_publications_reader(sedp_built_publications_reader);
        }

        participant_discovery.discovered_participant_add_publications_writer(
            sedp_builtin_publications_data_writer_proxy
                .as_ref()
                .upgrade()
                .unwrap()
                .write_lock()
                .rtps_writer
                .try_as_stateful_writer()
                .unwrap(),
        );
        let sedp_discovered_writer_data = SedpDiscoveredWriterData {
            writer_proxy: RtpsWriterProxy {
                remote_writer_guid: Guid::new(
                    GuidPrefix([1; 12]),
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId::new([0; 3], 0),
            },
            publication_builtin_topic_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey { value: [1; 16] },
                participant_key: BuiltInTopicKey { value: [1; 16] },
                topic_name: "MyTopic".to_string(),
                type_name: "MyType".to_string(),
                durability: DurabilityQosPolicy::default(),
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                    max_blocking_time: Duration::new(3, 0),
                },
                lifespan: LifespanQosPolicy::default(),
                user_data: UserDataQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                ownership_strength: OwnershipStrengthQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                presentation: PresentationQosPolicy::default(),
                partition: PartitionQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
                group_data: GroupDataQosPolicy::default(),
            },
        };

        sedp_builtin_publications_data_writer_proxy
            .write_w_timestamp(
                &sedp_discovered_writer_data,
                None,
                rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
            )
            .unwrap();
    }

    // {
    //     let shared_data_reader = sedp_built_publications_reader_proxy
    //         .as_ref()
    //         .upgrade()
    //         .unwrap();
    //     let mut data_reader = shared_data_reader.write().unwrap();
    //     let matched_writers = &data_reader
    //         .rtps_reader
    //         .try_as_stateful_reader()
    //         .unwrap()
    //         .matched_writers;

    //     assert_eq!(matched_writers.len(), 1);
    //     assert_eq!(
    //         matched_writers[0].remote_writer_guid,
    //         Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
    //     );
    // }

    for _i in 1..14 {
        communication.send(core::slice::from_ref(&publisher));

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

struct Rtps;
impl RtpsStructure for Rtps {
    type Group = RtpsGroupImpl;
    type Participant = RtpsParticipantImpl;

    type StatelessWriter = RtpsStatelessWriterImpl;
    type StatelessReader = RtpsStatelessReaderImpl;

    type StatefulWriter = RtpsStatefulWriterImpl;
    type StatefulReader = RtpsStatefulReaderImpl;
}

#[test]
fn create_two_participants_with_different_domains() {
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory.create_participant(1, None, None, 0)
        .unwrap();
    let participant2 = participant_factory.create_participant(2, None, None, 0)
        .unwrap();

    assert!(participant1.get_builtin_subscriber().is_ok());
    assert!(participant2.get_builtin_subscriber().is_ok());
}