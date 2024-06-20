use dust_dds::{
    builtin_topics::{BuiltInTopicKey, SubscriptionBuiltinTopicData},
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, ReaderProxy, DCPS_SUBSCRIPTION},
        spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
    },
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind, SubscriberQos},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, TopicDataQosPolicy,
        },
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    rtps::{
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        },
        messages::{
            overall_structure::{
                RtpsMessageHeader, RtpsMessageRead, RtpsMessageWrite, RtpsSubmessageReadKind,
            },
            submessage_elements::{Data, ParameterList, SequenceNumberSet},
            submessages::{ack_nack::AckNackSubmessage, data::DataSubmessage},
        },
        types::{
            EntityId, Guid, Locator, ENTITYID_UNKNOWN, LOCATOR_KIND_UDP_V4, PROTOCOLVERSION,
            USER_DEFINED_READER_WITH_KEY, VENDOR_ID_S2E,
        },
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::DdsSerialize,
};
use dust_dds_derive::DdsType;

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Clone, Debug, PartialEq, DdsType)]
struct KeyedData {
    #[dust_dds(key)]
    id: u8,
    value: u32,
}

#[test]
fn writer_should_send_heartbeat_periodically() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let mock_reader_socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();

    let reader_socket_port = mock_reader_socket.local_addr().unwrap().port();
    println!("Socket open on port {}", reader_socket_port);
    let reader_unicast_locator = Locator::new(
        LOCATOR_KIND_UDP_V4,
        reader_socket_port as u32,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    );

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();
    let dcps_subscription_reader = builtin_subscriber
        .lookup_datareader::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
        .unwrap()
        .unwrap();
    let dcps_subscription_reader_statuscondition = dcps_subscription_reader.get_statuscondition();
    dcps_subscription_reader_statuscondition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut waitset_builtin_reader = WaitSet::new();
    waitset_builtin_reader
        .attach_condition(Condition::StatusCondition(
            dcps_subscription_reader_statuscondition,
        ))
        .unwrap();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(topic_name, type_name, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    // Add discovered dummy reader
    let participant_handle = participant.get_instance_handle().unwrap();
    let guid_prefix = participant_handle.as_ref()[0..12].try_into().unwrap();
    let remote_reader_guid = Guid::new(
        guid_prefix,
        EntityId::new([0, 0, 0], USER_DEFINED_READER_WITH_KEY),
    );
    let reader_proxy = ReaderProxy::new(
        remote_reader_guid,
        ENTITYID_UNKNOWN,
        vec![reader_unicast_locator],
        vec![],
        false,
    );
    let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
        BuiltInTopicKey::from(<[u8; 16]>::from(remote_reader_guid)),
        BuiltInTopicKey::from(*participant_handle.as_ref()),
        topic_name.to_string(),
        type_name.to_string(),
        DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Infinite,
            },
            ..Default::default()
        },
        SubscriberQos::default(),
        TopicDataQosPolicy::default(),
        String::new(),
    );
    let dummy_reader_discovery =
        DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
    let serialized_dummy_reader_discovery_bytes = dummy_reader_discovery.serialize_data().unwrap();

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
        &RtpsMessageHeader::new(PROTOCOLVERSION, VENDOR_ID_S2E, guid_prefix),
        &[Box::new(discovered_reader_data_submessage)],
    );

    waitset_builtin_reader
        .wait(dust_dds::infrastructure::time::Duration::new(10, 0))
        .unwrap();

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .participant_proxy()
        .metatraffic_unicast_locator_list()[0]
        .port();
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
    let reader_unicast_locator = Locator::new(
        LOCATOR_KIND_UDP_V4,
        reader_socket_port as u32,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    );

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();
    let dcps_subscription_reader = builtin_subscriber
        .lookup_datareader::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
        .unwrap()
        .unwrap();
    let dcps_subscription_reader_statuscondition = dcps_subscription_reader.get_statuscondition();
    dcps_subscription_reader_statuscondition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut waitset_builtin_reader = WaitSet::new();
    waitset_builtin_reader
        .attach_condition(Condition::StatusCondition(
            dcps_subscription_reader_statuscondition,
        ))
        .unwrap();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(topic_name, type_name, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    // Add discovered dummy reader
    let participant_handle = participant.get_instance_handle().unwrap();
    let guid_prefix = participant_handle.as_ref()[0..12].try_into().unwrap();
    let reader_id = EntityId::new([0, 0, 0], USER_DEFINED_READER_WITH_KEY);
    let remote_reader_guid = Guid::new(guid_prefix, reader_id);
    let reader_proxy = ReaderProxy::new(
        remote_reader_guid,
        ENTITYID_UNKNOWN,
        vec![reader_unicast_locator],
        vec![],
        false,
    );
    let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
        BuiltInTopicKey::from(<[u8; 16]>::from(remote_reader_guid)),
        BuiltInTopicKey::from(*participant_handle.as_ref()),
        topic_name.to_string(),
        type_name.to_string(),
        DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Infinite,
            },
            ..Default::default()
        },
        SubscriberQos::default(),
        TopicDataQosPolicy::default(),
        String::new(),
    );
    let dummy_reader_discovery =
        DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
    let serialized_dummy_reader_discovery_bytes = dummy_reader_discovery.serialize_data().unwrap();

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
    let rtps_message_header = RtpsMessageHeader::new(PROTOCOLVERSION, VENDOR_ID_S2E, guid_prefix);
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &rtps_message_header,
        &[Box::new(discovered_reader_data_submessage)],
    );

    waitset_builtin_reader
        .wait(dust_dds::infrastructure::time::Duration::new(10, 0))
        .unwrap();

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .participant_proxy()
        .metatraffic_unicast_locator_list()[0]
        .port();
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
        .lookup_datareader::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_subscription_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let unicast_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .participant_proxy()
        .default_unicast_locator_list()[0]
        .port();

    let received_data_heartbeat = RtpsMessageRead::try_from(buffer.as_slice())
        .unwrap()
        .submessages();
    let received_heartbeat = received_data_heartbeat
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .unwrap();

    let writer_id = match received_heartbeat {
        RtpsSubmessageReadKind::Heartbeat(h) => h.writer_id(),
        _ => panic!("Wrong message type"),
    };
    let reader_sn_state = SequenceNumberSet::new(2, []);
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
    let reader_unicast_locator = Locator::new(
        LOCATOR_KIND_UDP_V4,
        reader_socket_port as u32,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    );

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();
    let dcps_subscription_reader = builtin_subscriber
        .lookup_datareader::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
        .unwrap()
        .unwrap();
    let dcps_subscription_reader_statuscondition = dcps_subscription_reader.get_statuscondition();
    dcps_subscription_reader_statuscondition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut waitset_builtin_reader = WaitSet::new();
    waitset_builtin_reader
        .attach_condition(Condition::StatusCondition(
            dcps_subscription_reader_statuscondition,
        ))
        .unwrap();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(topic_name, type_name, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    // Add discovered dummy reader
    let participant_handle = participant.get_instance_handle().unwrap();
    let guid_prefix = participant_handle.as_ref()[0..12].try_into().unwrap();
    let reader_id = EntityId::new([0, 0, 0], USER_DEFINED_READER_WITH_KEY);
    let remote_reader_guid = Guid::new(guid_prefix, reader_id);
    let reader_proxy = ReaderProxy::new(
        remote_reader_guid,
        ENTITYID_UNKNOWN,
        vec![reader_unicast_locator],
        vec![],
        false,
    );
    let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
        BuiltInTopicKey::from(<[u8; 16]>::from(remote_reader_guid)),
        BuiltInTopicKey::from(*participant_handle.as_ref()),
        topic_name.to_string(),
        type_name.to_string(),
        DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Infinite,
            },
            ..Default::default()
        },
        SubscriberQos::default(),
        TopicDataQosPolicy::default(),
        String::new(),
    );
    let dummy_reader_discovery =
        DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
    let serialized_dummy_reader_discovery_bytes = dummy_reader_discovery.serialize_data().unwrap();

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
    let rtps_message_header = RtpsMessageHeader::new(PROTOCOLVERSION, VENDOR_ID_S2E, guid_prefix);
    let discovered_reader_rtps_message = RtpsMessageWrite::new(
        &rtps_message_header,
        &[Box::new(discovered_reader_data_submessage)],
    );

    waitset_builtin_reader
        .wait(dust_dds::infrastructure::time::Duration::new(10, 0))
        .unwrap();

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .participant_proxy()
        .metatraffic_unicast_locator_list()[0]
        .port();
    let user_defined_traffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .participant_proxy()
        .default_unicast_locator_list()[0]
        .port();
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

    let received_data_heartbeat = RtpsMessageRead::try_from(buffer.as_slice())
        .unwrap()
        .submessages();
    let received_heartbeat = received_data_heartbeat
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
    let reader_unicast_locator = Locator::new(
        LOCATOR_KIND_UDP_V4,
        reader_socket_port as u32,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    );

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();
    let dcps_subscription_reader = builtin_subscriber
        .lookup_datareader::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
        .unwrap()
        .unwrap();
    let dcps_subscription_reader_statuscondition = dcps_subscription_reader.get_statuscondition();
    dcps_subscription_reader_statuscondition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut waitset_builtin_reader = WaitSet::new();
    waitset_builtin_reader
        .attach_condition(Condition::StatusCondition(
            dcps_subscription_reader_statuscondition,
        ))
        .unwrap();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(topic_name, type_name, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    // Add discovered dummy reader
    let participant_handle = participant.get_instance_handle().unwrap();
    let guid_prefix = participant_handle.as_ref()[0..12].try_into().unwrap();
    let remote_reader_guid = Guid::new(
        guid_prefix,
        EntityId::new([0, 0, 0], USER_DEFINED_READER_WITH_KEY),
    );
    let reader_proxy = ReaderProxy::new(
        remote_reader_guid,
        ENTITYID_UNKNOWN,
        vec![reader_unicast_locator],
        vec![],
        false,
    );
    let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
        BuiltInTopicKey::from(<[u8; 16]>::from(remote_reader_guid)),
        BuiltInTopicKey::from(*participant_handle.as_ref()),
        topic_name.to_string(),
        type_name.to_string(),
        DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Infinite,
            },
            ..Default::default()
        },
        SubscriberQos::default(),
        TopicDataQosPolicy::default(),
        String::new(),
    );
    let dummy_reader_discovery =
        DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
    let serialized_dummy_reader_discovery_bytes = dummy_reader_discovery.serialize_data().unwrap();

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
        &RtpsMessageHeader::new(PROTOCOLVERSION, VENDOR_ID_S2E, guid_prefix),
        &[Box::new(discovered_reader_data_submessage)],
    );

    waitset_builtin_reader
        .wait(dust_dds::infrastructure::time::Duration::new(10, 0))
        .unwrap();

    // Send data with the writer before discovery
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .participant_proxy()
        .metatraffic_unicast_locator_list()[0]
        .port();
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
    let reader_unicast_locator = Locator::new(
        LOCATOR_KIND_UDP_V4,
        reader_socket_port as u32,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    );

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber();
    let dcps_subscription_reader = builtin_subscriber
        .lookup_datareader::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
        .unwrap()
        .unwrap();
    let dcps_subscription_reader_statuscondition = dcps_subscription_reader.get_statuscondition();
    dcps_subscription_reader_statuscondition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut waitset_builtin_reader = WaitSet::new();
    waitset_builtin_reader
        .attach_condition(Condition::StatusCondition(
            dcps_subscription_reader_statuscondition,
        ))
        .unwrap();

    let topic_name = "MyTopic";
    let type_name = "KeyedData";
    let topic = participant
        .create_topic::<KeyedData>(topic_name, type_name, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
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
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    // Add discovered dummy reader
    let participant_handle = participant.get_instance_handle().unwrap();
    let guid_prefix = participant_handle.as_ref()[0..12].try_into().unwrap();
    let remote_reader_guid = Guid::new(
        guid_prefix,
        EntityId::new([0, 0, 0], USER_DEFINED_READER_WITH_KEY),
    );
    let reader_proxy = ReaderProxy::new(
        remote_reader_guid,
        ENTITYID_UNKNOWN,
        vec![reader_unicast_locator],
        vec![],
        false,
    );
    let subscription_builtin_topic_data = SubscriptionBuiltinTopicData::new(
        BuiltInTopicKey::from(<[u8; 16]>::from(remote_reader_guid)),
        BuiltInTopicKey::from(*participant_handle.as_ref()),
        topic_name.to_string(),
        type_name.to_string(),
        DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: DurationKind::Infinite,
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            ..Default::default()
        },
        SubscriberQos::default(),
        TopicDataQosPolicy::default(),
        String::new(),
    );
    let dummy_reader_discovery =
        DiscoveredReaderData::new(reader_proxy, subscription_builtin_topic_data);
    let serialized_dummy_reader_discovery_bytes = dummy_reader_discovery
        .serialize_data()
        .unwrap();

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
        &RtpsMessageHeader::new(PROTOCOLVERSION, VENDOR_ID_S2E, guid_prefix),
        &[Box::new(discovered_reader_data_submessage)],
    );

    waitset_builtin_reader
        .wait(dust_dds::infrastructure::time::Duration::new(10, 0))
        .unwrap();

    // Send data with the writer before discovery
    writer.write(&KeyedData { id: 1, value: 2 }, None).unwrap();

    let dcps_participant_reader = builtin_subscriber
        .lookup_datareader::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
        .unwrap()
        .unwrap();
    let dcps_sample_list = dcps_participant_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    let metatraffic_port = dcps_sample_list[0]
        .data()
        .unwrap()
        .participant_proxy()
        .metatraffic_unicast_locator_list()[0]
        .port();
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
