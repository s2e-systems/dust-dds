use dust_dds::{
    builtin_topics::DCPS_PARTICIPANT,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsError,
        qos::{DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        type_support::{DdsDeserialize, DdsType},
    },
    listener::NoOpListener,
    rtps::types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    rtps_messages::{
        overall_structure::{
            BufRead, RtpsMessageHeader, RtpsMessageRead, RtpsMessageWrite, RtpsSubmessageReadKind,
        },
        submessage_elements::{Data, ParameterList, SequenceNumberSet},
        submessages::{ack_nack::AckNackSubmessage, data::DataSubmessage},
    },
    transport::types::{
        EntityId, BUILT_IN_READER_WITH_KEY, BUILT_IN_WRITER_WITH_KEY, USER_DEFINED_READER_WITH_KEY,
    },
    wait_set::{Condition, WaitSet},
};
use std::io::Read;

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_READER_WITH_KEY);

#[derive(Clone, Debug, PartialEq, DdsType)]
struct KeyedData {
    #[dust_dds(key)]
    id: u8,
    value: u32,
}

struct DynamicType<'a>(&'a [u8]);
impl<'de> DdsDeserialize<'de> for DynamicType<'de> {
    fn deserialize_data(
        serialized_data: &'de [u8],
    ) -> dust_dds::infrastructure::error::DdsResult<Self> {
        Ok(Self(serialized_data))
    }
}

impl<'a> DynamicType<'a> {
    fn metatraffic_unicast_locator_port(&self) -> u32 {
        const PID_METATRAFFIC_UNICAST_LOCATOR: i16 = 0x0032;
        let reader = &mut &self.0[4..];
        let mut pid = [0, 0];
        let mut length = [0, 0];
        loop {
            reader.read(&mut pid).unwrap();
            reader.read(&mut length).unwrap();
            if i16::from_le_bytes(pid) == PID_METATRAFFIC_UNICAST_LOCATOR {
                return u32::from_le_bytes([reader[4], reader[5], reader[6], reader[7]]);
            } else {
                reader.consume(u16::from_le_bytes(length) as usize);
            }
        }
    }

    fn default_unicast_locator_port(&self) -> u32 {
        const PID_DEFAULT_UNICAST_LOCATOR: i16 = 0x0031;
        let reader = &mut &self.0[4..];
        let mut pid = [0, 0];
        let mut length = [0, 0];
        loop {
            reader.read(&mut pid).unwrap();
            reader.read(&mut length).unwrap();
            if i16::from_le_bytes(pid) == PID_DEFAULT_UNICAST_LOCATOR {
                return u32::from_le_bytes([reader[4], reader[5], reader[6], reader[7]]);
            } else {
                reader.consume(u16::from_le_bytes(length) as usize);
            }
        }
    }
}

#[test]
fn writer_should_send_heartbeat_periodically() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let mock_reader_socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let reader_socket_port = mock_reader_socket.local_addr().unwrap().port();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(
            topic_name,
            type_name,
            QosKind::Default,
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    // Add discovered dummy reader
    let instance_handle = participant.get_instance_handle();
    let participant_key = instance_handle.as_ref().as_slice();
    let guid_prefix = &participant_key[..12];
    let port = (reader_socket_port as u32).to_le_bytes();

    let serialized_dummy_reader_discovery_bytes = [
        &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
        ],
        guid_prefix,
        &[
            0, 0, 0, 7, // Entity ID
            0x50, 0x00, 16, 0, // PID_PARTICIPANT_GUID, length
        ],
        participant_key,
        &[
            0x05, 0x00, 12, 0x00, // PID_TOPIC_NAME, Length
            8, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'M', b'y', b'T', b'o', //
            b'p', b'i', b'c', 0, //
            0x07, 0x00, 16, 0x00, // PID_TYPE_NAME, Length
            10, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'K', b'e', b'y', b'e', //
            b'd', b'D', b'a', b't', //
            b'a', 0, 0, 0, //
            0x1A, 0x00, 12, 0x00, // PID_RELIABILITY, Length
            2, 0, 0, 0, // kind
            0xff, 0xff, 0xff, 0x7f, // max_blocking_time: sec
            0xff, 0xff, 0xff, 0xff, // max_blocking_time: nanosec
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            0, 0, 0, 0, //
            0x2F, 0x00, 24, 0, // PID_UNICAST_LOCATOR, Length
            1, 0, 0, 0, // locator kind
        ],
        &port, //locator port
        &[
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            127, 0, 0, 1, // locator address
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ],
    ]
    .concat()
    .to_vec();

    let discovered_reader_data_submessage = DataSubmessage::new(
        false,
        true,
        false,
        false,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        1,
        ParameterList::empty(),
        Data::new(serialized_dummy_reader_discovery_bytes.into()),
    );
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &RtpsMessageHeader::new(
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
            guid_prefix.try_into().unwrap(),
        ),
        &[Box::new(discovered_reader_data_submessage)],
    );

    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if participant.get_discovered_participants().unwrap().len() >= 1 {
            break;
        }
    }
    assert!(participant.get_discovered_participants().unwrap().len() == 1);

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<DynamicType>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .metatraffic_unicast_locator_port();
    mock_reader_socket
        .send_to(
            discovered_reader_rtps_message.buffer(),
            ("127.0.0.1", metatraffic_port as u16),
        )
        .unwrap();

    let mut waitset_writer = WaitSet::new();
    let writer_status_condition = writer.get_statuscondition();
    writer_status_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    waitset_writer
        .attach_condition(Condition::StatusCondition(writer_status_condition))
        .unwrap();
    waitset_writer.wait(Duration::new(10, 0)).unwrap();

    // Send data with the writer
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();

    let mut buffer = [0; 65535];
    mock_reader_socket.set_nonblocking(false).unwrap();
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();

    let received_data_heartbeat = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    let submessages = received_data_heartbeat.submessages();
    assert!(submessages
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Data(_)))
        .is_some());
    assert!(submessages
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());

    // Default heartbeat period is 200ms
    let mut buffer = [0; 65535];
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();
    let received_heartbeat = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    assert!(received_heartbeat
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());
}

#[test]
fn writer_should_not_send_heartbeat_after_acknack() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let mock_reader_socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let reader_socket_port = mock_reader_socket.local_addr().unwrap().port();
    println!("Socket open on port {}", reader_socket_port);
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(
            topic_name,
            type_name,
            QosKind::Default,
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    // Add discovered dummy reader
    let instance_handle = participant.get_instance_handle();
    let participant_key = instance_handle.as_ref().as_slice();
    let guid_prefix = &participant_key[..12];
    let port = (reader_socket_port as u32).to_le_bytes();

    let serialized_dummy_reader_discovery_bytes = [
        &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
        ],
        guid_prefix,
        &[
            0, 0, 0, 7, // Entity ID
            0x50, 0x00, 16, 0, // PID_PARTICIPANT_GUID, length
        ],
        participant_key,
        &[
            0x05, 0x00, 12, 0x00, // PID_TOPIC_NAME, Length
            8, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'M', b'y', b'T', b'o', //
            b'p', b'i', b'c', 0, //
            0x07, 0x00, 16, 0x00, // PID_TYPE_NAME, Length
            10, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'K', b'e', b'y', b'e', //
            b'd', b'D', b'a', b't', //
            b'a', 0, 0, 0, //
            0x1A, 0x00, 12, 0x00, // PID_RELIABILITY, Length
            2, 0, 0, 0, // kind
            0xff, 0xff, 0xff, 0x7f, // max_blocking_time: sec
            0xff, 0xff, 0xff, 0xff, // max_blocking_time: nanosec
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            0, 0, 0, 0, //
            0x2F, 0x00, 24, 0, // PID_UNICAST_LOCATOR, Length
            1, 0, 0, 0, // locator kind
        ],
        &port, //locator port
        &[
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            127, 0, 0, 1, // locator address
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ],
    ]
    .concat()
    .to_vec();

    let discovered_reader_data_submessage = DataSubmessage::new(
        false,
        true,
        false,
        false,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        1,
        ParameterList::empty(),
        Data::new(serialized_dummy_reader_discovery_bytes.into()),
    );
    let rtps_message_header = RtpsMessageHeader::new(
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
        guid_prefix.try_into().unwrap(),
    );
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &rtps_message_header,
        &[Box::new(discovered_reader_data_submessage)],
    );

    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if participant.get_discovered_participants().unwrap().len() >= 1 {
            break;
        }
    }
    assert!(participant.get_discovered_participants().unwrap().len() == 1);

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<DynamicType>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .metatraffic_unicast_locator_port();
    mock_reader_socket
        .send_to(
            discovered_reader_rtps_message.buffer(),
            ("127.0.0.1", metatraffic_port as u16),
        )
        .unwrap();

    let mut waitset_writer = WaitSet::new();
    let writer_status_condition = writer.get_statuscondition();
    writer_status_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    waitset_writer
        .attach_condition(Condition::StatusCondition(writer_status_condition))
        .unwrap();
    waitset_writer.wait(Duration::new(10, 0)).unwrap();

    // Send data with the writer
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();

    let mut buffer = [0; 65535];
    mock_reader_socket.set_nonblocking(false).unwrap();
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();

    let dcps_subscription_reader = builtin_subscriber
        .lookup_datareader::<DynamicType>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_subscription_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let unicast_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .default_unicast_locator_port();

    let rtps_message = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    let received_heartbeat = rtps_message
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .unwrap();

    let writer_id = match received_heartbeat {
        RtpsSubmessageReadKind::Heartbeat(h) => h.writer_id(),
        _ => panic!("Wrong message type"),
    };
    let reader_sn_state = SequenceNumberSet::new(2, []);
    let reader_id = EntityId::new([0, 0, 0], USER_DEFINED_READER_WITH_KEY);
    let reader_acknack_submessage =
        AckNackSubmessage::new(true, reader_id, writer_id, reader_sn_state, 1);

    let acknack_message =
        RtpsMessageWrite::new(&rtps_message_header, &[Box::new(reader_acknack_submessage)]);
    mock_reader_socket
        .send_to(acknack_message.buffer(), ("127.0.0.1", unicast_port as u16))
        .unwrap();

    // Default heartbeat period is 200ms
    let mut buffer = [0; 65535];
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    // No more messages expected
    assert!(mock_reader_socket.recv(&mut buffer).is_err());
}

#[test]
fn writer_should_resend_data_after_acknack_request() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let mock_reader_socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let reader_socket_port = mock_reader_socket.local_addr().unwrap().port();
    println!("Socket open on port {}", reader_socket_port);

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(
            topic_name,
            type_name,
            QosKind::Default,
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    // Add discovered dummy reader
    let instance_handle = participant.get_instance_handle();
    let participant_key = instance_handle.as_ref().as_slice();
    let guid_prefix = &participant_key[..12];
    let port = (reader_socket_port as u32).to_le_bytes();

    let serialized_dummy_reader_discovery_bytes = [
        &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
        ],
        guid_prefix,
        &[
            0, 0, 0, 7, // Entity ID
            0x50, 0x00, 16, 0, // PID_PARTICIPANT_GUID, length
        ],
        participant_key,
        &[
            0x05, 0x00, 12, 0x00, // PID_TOPIC_NAME, Length
            8, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'M', b'y', b'T', b'o', //
            b'p', b'i', b'c', 0, //
            0x07, 0x00, 16, 0x00, // PID_TYPE_NAME, Length
            10, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'K', b'e', b'y', b'e', //
            b'd', b'D', b'a', b't', //
            b'a', 0, 0, 0, //
            0x1A, 0x00, 12, 0x00, // PID_RELIABILITY, Length
            2, 0, 0, 0, // kind
            0xff, 0xff, 0xff, 0x7f, // max_blocking_time: sec
            0xff, 0xff, 0xff, 0xff, // max_blocking_time: nanosec
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            0, 0, 0, 0, //
            0x2F, 0x00, 24, 0, // PID_UNICAST_LOCATOR, Length
            1, 0, 0, 0, // locator kind
        ],
        &port, //locator port
        &[
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            127, 0, 0, 1, // locator address
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ],
    ]
    .concat()
    .to_vec();

    let discovered_reader_data_submessage = DataSubmessage::new(
        false,
        true,
        false,
        false,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        1,
        ParameterList::empty(),
        Data::new(serialized_dummy_reader_discovery_bytes.into()),
    );
    let rtps_message_header = RtpsMessageHeader::new(
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
        guid_prefix.try_into().unwrap(),
    );
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &rtps_message_header,
        &[Box::new(discovered_reader_data_submessage)],
    );

    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if participant.get_discovered_participants().unwrap().len() >= 1 {
            break;
        }
    }
    assert!(participant.get_discovered_participants().unwrap().len() == 1);

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<DynamicType>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .metatraffic_unicast_locator_port();
    let user_defined_traffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .default_unicast_locator_port();
    mock_reader_socket
        .send_to(
            discovered_reader_rtps_message.buffer(),
            ("127.0.0.1", metatraffic_port as u16),
        )
        .unwrap();

    let mut waitset_writer = WaitSet::new();
    let writer_status_condition = writer.get_statuscondition();
    writer_status_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    waitset_writer
        .attach_condition(Condition::StatusCondition(writer_status_condition))
        .unwrap();
    waitset_writer.wait(Duration::new(10, 0)).unwrap();

    // Send data with the writer
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();

    let mut buffer = [0; 65535];
    mock_reader_socket.set_nonblocking(false).unwrap();
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();

    let rtps_message = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    let received_heartbeat = rtps_message
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .unwrap();

    let writer_id = match received_heartbeat {
        RtpsSubmessageReadKind::Heartbeat(h) => h.writer_id(),
        _ => panic!("Wrong message type"),
    };

    // Everything is acknowledge up to the number prior to the base. By sending 1 this should
    // result in the Data submessage with sample 1 being resent
    let reader_sn_state = SequenceNumberSet::new(1, [1]);
    let reader_id = EntityId::new([0, 0, 0], USER_DEFINED_READER_WITH_KEY);
    let reader_acknack_submessage =
        AckNackSubmessage::new(true, reader_id, writer_id, reader_sn_state, 1);

    let acknack_message =
        RtpsMessageWrite::new(&rtps_message_header, &[Box::new(reader_acknack_submessage)]);
    mock_reader_socket
        .send_to(
            acknack_message.buffer(),
            ("127.0.0.1", user_defined_traffic_port as u16),
        )
        .unwrap();

    // Default heartbeat period is 200ms
    let mut buffer = [0; 65535];
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();
    let received_data_heartbeat = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    let submessages = received_data_heartbeat.submessages();
    assert!(submessages
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Data(_)))
        .is_some());
    assert!(submessages
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());
}

#[test]
fn volatile_writer_should_send_gap_submessage_after_discovery() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let mock_reader_socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let reader_socket_port = mock_reader_socket.local_addr().unwrap().port();
    println!("Socket open on port {}", reader_socket_port);

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(
            topic_name,
            type_name,
            QosKind::Default,
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    // Add discovered dummy reader
    let instance_handle = participant.get_instance_handle();
    let participant_key = instance_handle.as_ref().as_slice();
    let guid_prefix = &participant_key[..12];
    let port = (reader_socket_port as u32).to_le_bytes();

    let serialized_dummy_reader_discovery_bytes = [
        &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
        ],
        guid_prefix,
        &[
            0, 0, 0, 7, // Entity ID
            0x50, 0x00, 16, 0, // PID_PARTICIPANT_GUID, length
        ],
        participant_key,
        &[
            0x05, 0x00, 12, 0x00, // PID_TOPIC_NAME, Length
            8, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'M', b'y', b'T', b'o', //
            b'p', b'i', b'c', 0, //
            0x07, 0x00, 16, 0x00, // PID_TYPE_NAME, Length
            10, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'K', b'e', b'y', b'e', //
            b'd', b'D', b'a', b't', //
            b'a', 0, 0, 0, //
            0x1A, 0x00, 12, 0x00, // PID_RELIABILITY, Length
            2, 0, 0, 0, // kind
            0xff, 0xff, 0xff, 0x7f, // max_blocking_time: sec
            0xff, 0xff, 0xff, 0xff, // max_blocking_time: nanosec
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            0, 0, 0, 0, //
            0x2F, 0x00, 24, 0, // PID_UNICAST_LOCATOR, Length
            1, 0, 0, 0, // locator kind
        ],
        &port, //locator port
        &[
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            127, 0, 0, 1, // locator address
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ],
    ]
    .concat()
    .to_vec();

    let discovered_reader_data_submessage = DataSubmessage::new(
        false,
        true,
        false,
        false,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        1,
        ParameterList::empty(),
        Data::new(serialized_dummy_reader_discovery_bytes.into()),
    );
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &RtpsMessageHeader::new(
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
            guid_prefix.try_into().unwrap(),
        ),
        &[Box::new(discovered_reader_data_submessage)],
    );

    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if participant.get_discovered_participants().unwrap().len() >= 1 {
            break;
        }
    }
    assert!(participant.get_discovered_participants().unwrap().len() == 1);

    // Send data with the writer before discovery
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<DynamicType>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .metatraffic_unicast_locator_port();
    mock_reader_socket
        .send_to(
            discovered_reader_rtps_message.buffer(),
            ("127.0.0.1", metatraffic_port as u16),
        )
        .unwrap();

    let mut waitset_writer = WaitSet::new();
    let writer_status_condition = writer.get_statuscondition();
    writer_status_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    waitset_writer
        .attach_condition(Condition::StatusCondition(writer_status_condition))
        .unwrap();
    waitset_writer.wait(Duration::new(10, 0)).unwrap();

    let mut buffer = [0; 65535];
    mock_reader_socket.set_nonblocking(false).unwrap();
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();

    let received_gap = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    assert!(received_gap
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Gap(_)))
        .is_some());

    // Default heartbeat period is 200ms
    let mut buffer = [0; 65535];
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();
    let received_heartbeat = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    assert!(received_heartbeat
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());
}

#[test]
fn transient_local_writer_should_send_data_submessage_after_discovery() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let mock_reader_socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let reader_socket_port = mock_reader_socket.local_addr().unwrap().port();
    println!("Socket open on port {}", reader_socket_port);

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(
            topic_name,
            type_name,
            QosKind::Default,
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    // Add discovered dummy reader
    let instance_handle = participant.get_instance_handle();
    let participant_key = instance_handle.as_ref().as_slice();
    let guid_prefix = &participant_key[..12];
    let port = (reader_socket_port as u32).to_le_bytes();

    let serialized_dummy_reader_discovery_bytes = [
        &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
        ],
        guid_prefix,
        &[
            0, 0, 0, 7, // Entity ID
            0x50, 0x00, 16, 0, // PID_PARTICIPANT_GUID, length
        ],
        participant_key,
        &[
            0x05, 0x00, 12, 0x00, // PID_TOPIC_NAME, Length
            8, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'M', b'y', b'T', b'o', //
            b'p', b'i', b'c', 0, //
            0x07, 0x00, 16, 0x00, // PID_TYPE_NAME, Length
            10, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'K', b'e', b'y', b'e', //
            b'd', b'D', b'a', b't', //
            b'a', 0, 0, 0, //
            0x1A, 0x00, 12, 0x00, // PID_RELIABILITY, Length
            2, 0, 0, 0, // kind
            0xff, 0xff, 0xff, 0x7f, // max_blocking_time: sec
            0xff, 0xff, 0xff, 0xff, // max_blocking_time: nanosec
            0x1D, 0x00, 4, 0x00, // PID_DURABILITY, Length
            1, 0, 0, 0, // kind (transient local)
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            0, 0, 0, 0, //
            0x2F, 0x00, 24, 0, // PID_UNICAST_LOCATOR, Length
            1, 0, 0, 0, // locator kind
        ],
        &port, //locator port
        &[
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            127, 0, 0, 1, // locator address
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ],
    ]
    .concat()
    .to_vec();

    let discovered_reader_data_submessage = DataSubmessage::new(
        false,
        true,
        false,
        false,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        1,
        ParameterList::empty(),
        Data::new(serialized_dummy_reader_discovery_bytes.into()),
    );
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &RtpsMessageHeader::new(
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
            guid_prefix.try_into().unwrap(),
        ),
        &[Box::new(discovered_reader_data_submessage)],
    );

    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if participant.get_discovered_participants().unwrap().len() >= 1 {
            break;
        }
    }
    assert!(participant.get_discovered_participants().unwrap().len() == 1);

    // Send data with the writer before discovery
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<DynamicType>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .metatraffic_unicast_locator_port();
    mock_reader_socket
        .send_to(
            discovered_reader_rtps_message.buffer(),
            ("127.0.0.1", metatraffic_port as u16),
        )
        .unwrap();

    let mut waitset_writer = WaitSet::new();
    let writer_status_condition = writer.get_statuscondition();
    writer_status_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    waitset_writer
        .attach_condition(Condition::StatusCondition(writer_status_condition))
        .unwrap();
    waitset_writer.wait(Duration::new(10, 0)).unwrap();

    let mut buffer = [0; 65535];
    mock_reader_socket.set_nonblocking(false).unwrap();
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();

    let received_data_heartbeat = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    let submessages = received_data_heartbeat.submessages();
    assert!(submessages
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Data(_)))
        .is_some());
    assert!(submessages
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());

    // Default heartbeat period is 200ms
    let mut buffer = [0; 65535];
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();
    let received_heartbeat = RtpsMessageRead::try_from(buffer.as_slice()).unwrap();
    let submessages = received_heartbeat.submessages();
    assert!(submessages
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());
}

#[test]
fn reliable_writer_should_not_remove_unacked_sample_from_history() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let mock_reader_socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let reader_socket_port = mock_reader_socket.local_addr().unwrap().port();
    println!("Socket open on port {}", reader_socket_port);

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(
            topic_name,
            type_name,
            QosKind::Default,
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(0, 100_000_000)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    // Add discovered dummy reader
    let instance_handle = participant.get_instance_handle();
    let participant_key = instance_handle.as_ref().as_slice();
    let guid_prefix = &participant_key[..12];
    let port = (reader_socket_port as u32).to_le_bytes();

    let serialized_dummy_reader_discovery_bytes = [
        &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            // SubscriptionBuiltinTopicData:
            0x5a, 0x00, 16, 0, //PID_ENDPOINT_GUID, length
        ],
        guid_prefix,
        &[
            0, 0, 0, 7, // Entity ID
            0x50, 0x00, 16, 0, // PID_PARTICIPANT_GUID, length
        ],
        participant_key,
        &[
            0x05, 0x00, 12, 0x00, // PID_TOPIC_NAME, Length
            8, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'M', b'y', b'T', b'o', //
            b'p', b'i', b'c', 0, //
            0x07, 0x00, 16, 0x00, // PID_TYPE_NAME, Length
            10, 0x00, 0x00, 0x00, // string length (incl. terminator)
            b'K', b'e', b'y', b'e', //
            b'd', b'D', b'a', b't', //
            b'a', 0, 0, 0, //
            0x1A, 0x00, 12, 0x00, // PID_RELIABILITY, Length
            2, 0, 0, 0, // kind
            0xff, 0xff, 0xff, 0x7f, // max_blocking_time: sec
            0xff, 0xff, 0xff, 0xff, // max_blocking_time: nanosec
            // ReaderProxy:
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            0, 0, 0, 0, //
            0x2F, 0x00, 24, 0, // PID_UNICAST_LOCATOR, Length
            1, 0, 0, 0, // locator kind
        ],
        &port, //locator port
        &[
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            0, 0, 0, 0, // locator address
            127, 0, 0, 1, // locator address
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ],
    ]
    .concat()
    .to_vec();

    let discovered_reader_data_submessage = DataSubmessage::new(
        false,
        true,
        false,
        false,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        1,
        ParameterList::empty(),
        Data::new(serialized_dummy_reader_discovery_bytes.into()),
    );
    let rtps_message_header = RtpsMessageHeader::new(
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
        guid_prefix.try_into().unwrap(),
    );
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &rtps_message_header,
        &[Box::new(discovered_reader_data_submessage)],
    );

    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if participant.get_discovered_participants().unwrap().len() >= 1 {
            break;
        }
    }
    assert!(participant.get_discovered_participants().unwrap().len() == 1);

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<DynamicType>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .metatraffic_unicast_locator_port();
    mock_reader_socket
        .send_to(
            discovered_reader_rtps_message.buffer(),
            ("127.0.0.1", metatraffic_port as u16),
        )
        .unwrap();

    let mut waitset_writer = WaitSet::new();
    let writer_status_condition = writer.get_statuscondition();
    writer_status_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    waitset_writer
        .attach_condition(Condition::StatusCondition(writer_status_condition))
        .unwrap();
    waitset_writer.wait(Duration::new(10, 0)).unwrap();

    // Send data with the writer
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();
    // Second send fails because sample is not acknowledged
    assert_eq!(
        writer.write(&KeyedData { id: 1, value: 2 }, None),
        Err(DdsError::Timeout)
    );
}
