use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER,
        qos::{DataReaderQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{NO_STATUS, StatusKind},
        time::Duration,
        type_support::DdsType,
    },
    rtps_udp_transport::IpVersion,
    wait_set::{Condition, WaitSet},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Debug, PartialEq, DdsType)]
struct UdpV6Data {
    #[dust_dds(key)]
    id: u8,
    value: u32,
}

#[test]
fn udp_v6_locators_and_communication() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    // Set transport to IPv6
    participant_factory
        .get_mut_transport()
        .set_ip_version(IpVersion::V6);

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UdpV6Data>(
            "UdpV6Topic",
            "UdpV6Data",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<UdpV6Data>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(Duration::new(
                1, 0,
            )),
        },
        ..Default::default()
    };

    let data_reader = subscriber
        .create_datareader::<UdpV6Data>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = data_writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let reader_cond = data_reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();

    let data = UdpV6Data {
        id: 42,
        value: 123456,
    };
    data_writer.write(data, None).unwrap();

    reader_wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = data_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].data.as_ref().unwrap(),
        &UdpV6Data {
            id: 42,
            value: 123456
        }
    );

    // Clean up entities
    participant.delete_contained_entities().unwrap();
    participant_factory
        .delete_participant(&participant)
        .unwrap();

    // Reset transport back to default V4
    participant_factory
        .get_mut_transport()
        .set_ip_version(IpVersion::V4);
}

#[test]
fn udp_transport_factory_ip_version_configuration() {
    let mut transport = dust_dds::rtps_udp_transport::RtpsUdpTransportParticipantFactory::default();
    assert_eq!(transport.ip_version(), IpVersion::V4);

    transport.set_ip_version(IpVersion::V6);
    assert_eq!(transport.ip_version(), IpVersion::V6);

    transport.set_ip_version(IpVersion::Both);
    assert_eq!(transport.ip_version(), IpVersion::Both);
}

#[test]
fn udp_both_locators_and_communication() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    // Set transport to Both (IPv4 and IPv6)
    participant_factory
        .get_mut_transport()
        .set_ip_version(IpVersion::Both);

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UdpV6Data>(
            "UdpBothTopic",
            "UdpV6Data",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<UdpV6Data>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(Duration::new(
                1, 0,
            )),
        },
        ..Default::default()
    };

    let data_reader = subscriber
        .create_datareader::<UdpV6Data>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = data_writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let reader_cond = data_reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();

    let data = UdpV6Data {
        id: 99,
        value: 987654,
    };
    data_writer.write(data, None).unwrap();

    reader_wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = data_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].data.as_ref().unwrap(),
        &UdpV6Data {
            id: 99,
            value: 987654
        }
    );

    // Clean up entities
    participant.delete_contained_entities().unwrap();
    participant_factory
        .delete_participant(&participant)
        .unwrap();

    // Reset transport back to default V4
    participant_factory
        .get_mut_transport()
        .set_ip_version(IpVersion::V4);
}
