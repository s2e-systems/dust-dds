use std::sync::Arc;

use dust_dds::{
    builtin_topics::{BuiltInTopicKey, SubscriptionBuiltinTopicData},
    data_representation_builtin_endpoints::discovered_reader_data::{
        DiscoveredReaderData, ReaderProxy, DCPS_SUBSCRIPTION,
    },
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind, SubscriberQos},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind, TopicDataQosPolicy},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    rtps::{
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        },
        messages::{
            overall_structure::{
                RtpsMessageHeader, RtpsMessageRead, RtpsMessageWrite, RtpsSubmessageReadKind,
            },
            submessage_elements::{Data, ParameterList},
            submessages::data::DataSubmessage,
        },
        types::{
            EntityId, Guid, Locator, ENTITYID_UNKNOWN, LOCATOR_KIND_UDP_V4, PROTOCOLVERSION,
            USER_DEFINED_READER_WITH_KEY, VENDOR_ID_S2E,
        },
    },
    topic_definition::type_support::DdsSerialize,
};
use dust_dds_derive::DdsType;

#[derive(Clone, Debug, PartialEq, DdsType)]
struct KeyedData {
    #[dust_dds(key)]
    id: u8,
    value: u32,
}

#[test]
fn writer_should_not_send_heartbeat_after_acknack() {
    let domain_id = 0;

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
    let mut serialized_dummy_reader_discovery_bytes = Vec::new();
    dummy_reader_discovery
        .serialize_data(&mut serialized_dummy_reader_discovery_bytes)
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
    mock_reader_socket
        .send_to(discovered_reader_rtps_message.buffer(), "127.0.0.1:7400")
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

    let received_data_heartbeat = RtpsMessageRead::new(Arc::from(buffer)).unwrap();
    assert!(received_data_heartbeat
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Data(_)))
        .is_some());
    assert!(received_data_heartbeat
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());

    let mut buffer = [0; 65535];
    mock_reader_socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    mock_reader_socket.recv(&mut buffer).unwrap();
    let received_heartbeat = RtpsMessageRead::new(Arc::from(buffer)).unwrap();
    assert!(received_heartbeat
        .submessages()
        .iter()
        .find(|s| matches!(s, RtpsSubmessageReadKind::Heartbeat(_)))
        .is_some());
}
