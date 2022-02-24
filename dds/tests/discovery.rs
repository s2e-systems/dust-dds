use std::net::UdpSocket;

use rust_dds::{
    communication::Communication,
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::{DomainParticipantFactory, RtpsStructureImpl},
    infrastructure::{
        entity::Entity,
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, TopicDataQosPolicy,
        },
    },
    publication::{data_writer::DataWriter, publisher::Publisher},
    subscription::{data_reader::DataReader, subscriber::Subscriber},
    types::{Duration, Time},
    udp_transport::UdpTransport,
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
    data_representation_builtin_endpoints::{
        sedp_discovered_writer_data::{RtpsWriterProxy, SedpDiscoveredWriterData},
        spdp_discovered_participant_data::{ParticipantProxy, SpdpDiscoveredParticipantData},
    },
    dds_impl::{
        data_reader_proxy::{DataReaderAttributes, DataReaderProxy, RtpsReader, Samples},
        data_writer_proxy::{DataWriterAttributes, DataWriterProxy, RtpsWriter},
        domain_participant_proxy::DomainParticipantProxy,
        publisher_proxy::PublisherAttributes,
        subscriber_proxy::SubscriberAttributes,
        topic_proxy::TopicAttributes,
    },
    dds_type::{DdsDeserialize, DdsSerialize, DdsType},
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl,
        rtps_reader_locator_impl::RtpsReaderLocatorAttributesImpl,
        rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_reader_impl::RtpsStatelessReaderImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{RtpsShared, RtpsWeak},
    },
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
    structure::{
        group::RtpsGroupConstructor,
        types::{
            EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, BUILT_IN_READER_GROUP,
            BUILT_IN_WRITER_GROUP, PROTOCOLVERSION, VENDOR_ID_S2E,
        },
    },
};

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

    let shared_data_reader = sedp_built_publications_reader_proxy
        .as_ref()
        .upgrade()
        .unwrap();
    let mut data_reader = shared_data_reader.write_lock();
    let matched_writers = &data_reader
        .rtps_reader
        .try_as_stateful_reader()
        .unwrap()
        .matched_writers;

    assert_eq!(matched_writers.len(), 1);
    // assert_eq!(
    //     matched_writers[0].remote_writer_guid,
    //     Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
    // );

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

fn num_matched_writers(participant: &DomainParticipantProxy<RtpsStructureImpl>) -> usize {
    let subscriber = participant
        .get_builtin_subscriber()
        .unwrap()
        .as_ref()
        .upgrade()
        .unwrap();
    let data_readers = &subscriber.read_lock().data_reader_list;
    data_readers
        .iter()
        .filter_map(|w| {
            w.write_lock()
                .rtps_reader
                .try_as_stateful_reader()
                .ok()
                .map(|w| w.matched_writers.len())
        })
        .sum::<usize>()
}

struct MyType {}

impl DdsType for MyType {
    fn type_name() -> &'static str {
        "MyType"
    }

    fn has_key() -> bool {
        false
    }
}

impl DdsSerialize for MyType {
    fn serialize<W: std::io::Write, E: rust_dds_rtps_implementation::dds_type::Endianness>(
        &self,
        _writer: W,
    ) -> rust_dds::DDSResult<()> {
        Ok(())
    }
}

impl<'de> DdsDeserialize<'de> for MyType {
    fn deserialize(_buf: &mut &'de [u8]) -> rust_dds::DDSResult<Self> {
        Ok(MyType {})
    }
}

#[test]
fn participant_discovery() {
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();
    participant1.enable().unwrap();

    let participant2 = participant_factory
        .create_participant(0, None, None, 0)
        .unwrap();
    participant2.enable().unwrap();

    let topic = participant1
        .create_topic::<MyType>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let mut writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let reader = subscriber.create_datareader(&topic, None, None, 0).unwrap();

    writer
        .write_w_timestamp(&MyType {}, None, Time { sec: 0, nanosec: 0 })
        .unwrap();

    // std::thread::sleep(std::time::Duration::new(5, 0));

    println!("P1 matched writers: {}", num_matched_writers(&participant1));
    println!("P2 matched writers: {}", num_matched_writers(&participant2));

    println!(
        "User reader matched writers: {}",
        reader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writers
            .len()
    );
    println!(
        "User writer matched readers: {}",
        writer
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_readers
            .len()
    );

    // let samples = reader.read(1, &[], &[], &[]).unwrap();
    // assert!(samples.len() == 1);

    assert!(participant1.get_builtin_subscriber().is_ok());
    assert!(participant2.get_builtin_subscriber().is_ok());
}
