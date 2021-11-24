use std::{
    net::UdpSocket,
    sync::{Arc, RwLock},
};

use rust_dds::{
    infrastructure::{
        qos::{DataReaderQos, SubscriberQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, TopicDataQosPolicy,
        },
    },
    publication::publisher::Publisher,
    subscription::data_reader::DataReader,
    types::Duration,
    udp_transport::UdpTransport,
};
use rust_dds_api::{
    builtin_topics::{ParticipantBuiltinTopicData, PublicationBuiltinTopicData},
    dcps_psm::BuiltInTopicKey,
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        qos_policy::UserDataQosPolicy,
    },
    publication::data_writer::DataWriter,
};
use rust_dds_rtps_implementation::{
    data_representation_builtin_endpoints::{
        sedp_discovered_writer_data::SedpDiscoveredWriterData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    dds_impl::{
        data_reader_impl::DataReaderImpl, data_writer_impl::DataWriterImpl,
        publisher_impl::PublisherImpl, subscriber_impl::SubscriberImpl,
    },
    rtps_impl::{
        rtps_reader_history_cache_impl::ReaderHistoryCache,
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
    utils::{
        message_receiver::MessageReceiver,
        shared_object::{rtps_shared_new, rtps_shared_read_lock},
        transport::TransportRead,
    },
};
use rust_rtps_pim::{
    behavior::{
        reader::writer_proxy::RtpsWriterProxy,
        writer::{
            reader_locator::RtpsReaderLocator, stateless_writer::RtpsStatelessWriterOperations,
        },
    },
    discovery::{
        participant_discovery::ParticipantDiscovery,
        sedp::builtin_endpoints::{
            SedpBuiltinPublicationsReader, SedpBuiltinPublicationsWriter,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        },
        spdp::{
            builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
            participant_proxy::ParticipantProxy,
        },
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::{
        group::RtpsGroup,
        types::{
            EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, ProtocolVersion,
            BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP, GUID_UNKNOWN,
        },
    },
};

#[test]
fn send_and_receive_discovery_data_happy_path() {
    let guid_prefix = GuidPrefix([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]);
    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey {
            value: [0, 0, 0, 1],
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

    let mut spdp_builtin_participant_rtps_writer = RtpsStatelessWriterImpl::new(
        SpdpBuiltinParticipantWriter::create(GuidPrefix([3; 12]), vec![], vec![]),
    );

    let spdp_discovery_locator = RtpsReaderLocator::new(
        Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        ),
        false,
    );

    spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

    let mut data_writer = DataWriterImpl::new(
        DataWriterQos::default(),
        spdp_builtin_participant_rtps_writer,
    );

    data_writer
        .write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
        )
        .unwrap();
    let publisher = PublisherImpl::new(
        PublisherQos::default(),
        RtpsGroup::new(GUID_UNKNOWN),
        vec![Arc::new(RwLock::new(data_writer))],
    );

    let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
    socket.set_nonblocking(true).unwrap();
    let mut transport = UdpTransport::new(socket);
    publisher.send_message(&mut transport);

    // Reception

    let spdp_builtin_participant_rtps_reader_impl =
        SpdpBuiltinParticipantReader::create(GuidPrefix([5; 12]), vec![], vec![]);

    let data_reader: DataReaderImpl<SpdpDiscoveredParticipantData> = DataReaderImpl::new(
        DataReaderQos::default(),
        spdp_builtin_participant_rtps_reader_impl,
    );
    let shared_data_reader = rtps_shared_new(data_reader);
    let subscriber = SubscriberImpl::new(
        SubscriberQos::default(),
        RtpsGroup::new(Guid::new(
            GuidPrefix([6; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
        )),
        vec![shared_data_reader.clone()],
    );

    let (source_locator, message) = transport.read().unwrap();
    let participant_guid_prefix = GuidPrefix([7; 12]);
    MessageReceiver::new().process_message(
        participant_guid_prefix,
        &[rtps_shared_new(subscriber)],
        source_locator,
        &message,
    );
    let shared_data_reader = rtps_shared_read_lock(&shared_data_reader);

    let result = shared_data_reader.read(1, &[], &[], &[]).unwrap();
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
            value: [0, 0, 0, 1],
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

    let mut spdp_builtin_participant_rtps_writer = RtpsStatelessWriterImpl::new(
        SpdpBuiltinParticipantWriter::create(GuidPrefix([3; 12]), vec![], vec![]),
    );

    let spdp_discovery_locator = RtpsReaderLocator::new(
        Locator::new(
            LOCATOR_KIND_UDPv4,
            7402,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        ),
        false,
    );

    spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

    let mut spdp_builtin_participant_data_writer = DataWriterImpl::new(
        DataWriterQos::default(),
        spdp_builtin_participant_rtps_writer,
    );

    spdp_builtin_participant_data_writer
        .write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
        )
        .unwrap();

    let sedp_builtin_publications_rtps_writer = RtpsStatefulWriterImpl::new(
        SedpBuiltinPublicationsWriter::create(guid_prefix, vec![], vec![]),
    );

    let sedp_builtin_publications_data_writer = DataWriterImpl::<SedpDiscoveredWriterData, _>::new(
        DataWriterQos::default(),
        sedp_builtin_publications_rtps_writer,
    );

    let publisher = PublisherImpl::new(
        PublisherQos::default(),
        RtpsGroup::new(Guid::new(
            GuidPrefix([4; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
        )),
        vec![
            rtps_shared_new(spdp_builtin_participant_data_writer),
            rtps_shared_new(sedp_builtin_publications_data_writer),
        ],
    );

    let socket = UdpSocket::bind("127.0.0.1:7402").unwrap();
    socket.set_nonblocking(true).unwrap();
    let mut transport = UdpTransport::new(socket);
    publisher.send_message(&mut transport);

    // Reception

    let spdp_builtin_participant_rtps_reader_impl =
        SpdpBuiltinParticipantReader::create(GuidPrefix([5; 12]), vec![], vec![]);

    let spdp_builtin_participant_data_reader: DataReaderImpl<SpdpDiscoveredParticipantData> =
        DataReaderImpl::new(
            DataReaderQos::default(),
            spdp_builtin_participant_rtps_reader_impl,
        );
    let shared_data_reader = rtps_shared_new(spdp_builtin_participant_data_reader);
    let subscriber = SubscriberImpl::new(
        SubscriberQos::default(),
        RtpsGroup::new(Guid::new(
            GuidPrefix([6; 12]),
            EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
        )),
        vec![shared_data_reader.clone()],
    );

    let (source_locator, message) = transport.read().unwrap();
    let participant_guid_prefix = GuidPrefix([7; 12]);
    MessageReceiver::new().process_message(
        participant_guid_prefix,
        &[rtps_shared_new(subscriber)],
        source_locator,
        &message,
    );
    let shared_data_reader = rtps_shared_read_lock(&shared_data_reader);

    let discovered_participant = shared_data_reader.read(1, &[], &[], &[]).unwrap();

    let sedp_builtin_publications_rtps_reader = SedpBuiltinPublicationsReader::create::<
        _,
        ReaderHistoryCache<SedpDiscoveredWriterData>,
    >(guid_prefix, vec![], vec![]);
    let mut sedp_built_publications_reader = DataReaderImpl::new(
        DataReaderQos::default(),
        sedp_builtin_publications_rtps_reader,
    );

    if let Ok(participant_discovery) = ParticipantDiscovery::new(
        &discovered_participant[0].participant_proxy,
        domain_id,
        domain_tag,
    ) {
        participant_discovery
            .discovered_participant_add_publications_reader(&mut sedp_built_publications_reader);

        let sedp_builtin_publications_data_writer = publisher
            .lookup_datawriter::<SedpDiscoveredWriterData>(&())
            .unwrap();
        let mut sedp_builtin_publications_data_writer_lock =
            sedp_builtin_publications_data_writer.write().unwrap();
        todo!();
        // participant_discovery.discovered_participant_add_publications_writer(
        //     &mut *sedp_builtin_publications_data_writer_lock,
        // );
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
                key: BuiltInTopicKey { value: [1; 4] },
                participant_key: BuiltInTopicKey { value: [1; 4] },
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

        sedp_builtin_publications_data_writer_lock
            .write_w_timestamp(
                &sedp_discovered_writer_data,
                None,
                rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
            )
            .unwrap();
    }

    assert_eq!(sedp_built_publications_reader.matched_writers.len(), 1);
    assert_eq!(
        sedp_built_publications_reader.matched_writers[0].remote_writer_guid,
        Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
    );

    for _i in 1..14 {
        publisher.send_message(&mut transport);

        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
