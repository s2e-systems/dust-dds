use std::net::UdpSocket;

use rust_dds::{
    infrastructure::qos::{DataReaderQos, SubscriberQos},
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
        data_reader_impl::DataReaderImpl, data_writer_impl::DataWriterImpl,
        publisher_impl::PublisherImpl, subscriber_impl::SubscriberImpl,
    },
    dds_type::DdsDeserialize,
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl, rtps_reader_impl::RtpsReaderImpl,
        rtps_writer_impl::RtpsWriterImpl,
    },
    utils::{
        message_receiver::MessageReceiver, shared_object::RtpsShared, transport::TransportRead,
    },
};
use rust_rtps_pim::{
    behavior::{reader::reader::RtpsReader, types::Duration},
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
            EntityId, Guid, LOCATOR_KIND_UDPv4, Locator, ProtocolVersion, BUILT_IN_READER_GROUP,
            BUILT_IN_WRITER_GROUP, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
        },
        RtpsHistoryCache,
    },
};

#[test]
fn send_discovery_data_happy_path() {
    let spdp_discovery_locator = Locator::new(
        LOCATOR_KIND_UDPv4,
        7400,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    );

    let dds_participant_data = ParticipantBuiltinTopicData {
        key: BuiltInTopicKey { value: [8, 8, 8] },
        user_data: UserDataQosPolicy { value: &[] },
    };
    let participant_proxy = ParticipantProxy {
        domain_id: 1,
        domain_tag: "ab".to_string(),
        protocol_version: ProtocolVersion { major: 1, minor: 4 },
        guid_prefix: [8; 12],
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

    let spdp_builtin_participant_rtps_writer: RtpsWriterImpl =
        SpdpBuiltinParticipantWriter::create([3; 12], &[], &[], &[spdp_discovery_locator]);

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
        RtpsGroupImpl::new(Guid::new(
            [4; 12],
            EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
        )),
        vec![RtpsShared::new(data_writer)],
    );

    let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
    socket.set_nonblocking(true).unwrap();
    let mut transport = UdpTransport::new(socket);
    publisher.send_data(
        &PROTOCOLVERSION,
        &VENDOR_ID_UNKNOWN,
        &[3; 12],
        &mut transport,
    );

    // Reception
    let spdp_builtin_participant_rtps_reader: RtpsReaderImpl =
        SpdpBuiltinParticipantReader::create([5; 12]);
    let data_reader = DataReaderImpl::new(
        spdp_builtin_participant_rtps_reader,
        DataReaderQos::default(),
    );
    let shared_data_reader = RtpsShared::new(data_reader);
    let subscriber = SubscriberImpl::new(
        SubscriberQos::default(),
        RtpsGroupImpl::new(Guid::new(
            [6; 12],
            EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
        )),
        vec![shared_data_reader.clone()],
    );

    let (source_locator, message) = transport.read().unwrap();
    let participant_guid_prefix = [7; 12];
    MessageReceiver::new().process_message(
        participant_guid_prefix,
        &[RtpsShared::new(subscriber)],
        source_locator,
        &message,
    );
    let shared_data_reader = shared_data_reader.read();
    let reader_cache = shared_data_reader.rtps_reader().reader_cache();
    let cc = reader_cache.get_change(&1).unwrap();
    let mut data = cc.data_value;

    let result: SpdpDiscoveredParticipantData<String, Vec<Locator>> =
        DdsDeserialize::deserialize(&mut data).unwrap();
    assert_eq!(spdp_discovered_participant_data, result);
}
