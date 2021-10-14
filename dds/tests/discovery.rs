use std::net::UdpSocket;

use rust_dds::{
    infrastructure::qos::{DataReaderQos, SubscriberQos},
    subscription::data_reader::DataReader,
    udp_transport::UdpTransport,
};
use rust_dds_api::{
    builtin_topics::ParticipantBuiltinTopicData,
    dcps_psm::BuiltInTopicKey,
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        qos_policy::UserDataQosPolicy,
    },
    publication::data_writer::DataWriter,
};
use rust_dds_rtps_implementation::{
    data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    dds_impl::{
        data_reader_impl::{DataReaderImpl, RtpsReaderFlavor},
        data_writer_impl::{DataWriterImpl, RtpsWriterFlavor},
        publisher_impl::PublisherImpl,
        subscriber_impl::SubscriberImpl,
    },
    rtps_impl::{
        rtps_reader_locator_impl::RtpsReaderLocatorImpl,
        rtps_stateless_reader_impl::RtpsStatelessReaderImpl,
    },
    utils::{
        message_receiver::MessageReceiver,
        shared_object::{rtps_shared_new, rtps_shared_read_lock},
        transport::TransportRead,
    },
};
use rust_rtps_pim::{
    behavior::{types::Duration, writer::reader_locator::RtpsReaderLocatorOperations},
    discovery::{
        spdp::{
            builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
            participant_proxy::ParticipantProxy,
        },
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::{
        types::{
            EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, ProtocolVersion,
            BUILT_IN_READER_GROUP, BUILT_IN_WRITER_GROUP, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
        },
        RtpsEntity, RtpsGroup,
    },
};

#[test]
fn send_discovery_data_happy_path() {
    let spdp_discovery_locator = RtpsReaderLocatorImpl::new(
        Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        ),
        false,
    );

    let guid_prefix = GuidPrefix([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]);
    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey { value: [0, 0, 1] },
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
    let lease_duration = Duration {
        seconds: 100,
        fraction: 0,
    };

    let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
        dds_participant_data,
        participant_proxy,
        lease_duration,
    };

    let spdp_builtin_participant_rtps_writer = SpdpBuiltinParticipantWriter::create(
        GuidPrefix([3; 12]),
        vec![],
        vec![],
        vec![spdp_discovery_locator],
    );

    let mut data_writer = DataWriterImpl::new(
        DataWriterQos::default(),
        RtpsWriterFlavor::Stateless(spdp_builtin_participant_rtps_writer),
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
        RtpsGroup {
            entity: RtpsEntity {
                guid: Guid::new(
                    GuidPrefix([4; 12]),
                    EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
                ),
            },
        },
        vec![rtps_shared_new(data_writer)],
    );

    let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
    socket.set_nonblocking(true).unwrap();
    let mut transport = UdpTransport::new(socket);
    publisher.send_data(
        &PROTOCOLVERSION,
        &VENDOR_ID_UNKNOWN,
        &GuidPrefix([3; 12]),
        &mut transport,
    );

    // Reception
    let spdp_builtin_participant_rtps_reader =
        SpdpBuiltinParticipantReader::create(GuidPrefix([5; 12]), vec![], vec![]);
    let data_reader = DataReaderImpl::new(
        DataReaderQos::default(),
        RtpsReaderFlavor::Stateless(RtpsStatelessReaderImpl::new(
            spdp_builtin_participant_rtps_reader,
        )),
    );
    let shared_data_reader = rtps_shared_new(data_reader);
    let subscriber = SubscriberImpl::new(
        SubscriberQos::default(),
        RtpsGroup {
            entity: RtpsEntity {
                guid: Guid::new(
                    GuidPrefix([6; 12]),
                    EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
                ),
            },
        },
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
    assert_eq!(spdp_discovered_participant_data, result[0]);
}
