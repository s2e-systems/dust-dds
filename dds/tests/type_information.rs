use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        configuration::DustDdsConfigurationBuilder,
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
        type_support::DdsType,
    },
    rtps_udp_transport::udp_transport::RtpsUdpTransport,
    security::plugins::types::DdsSecurityPlugins,
    wait_set::{Condition, WaitSet},
    xtypes::type_support::TypeSupport,
};

mod domain_id_generator;
use domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[test]
fn heterogeneous_types_with_type_information_disabled_should_read_write() {
    #[derive(DdsType, Clone)]
    struct WriterData {
        a: u8,
        b: u16,
        c: [u8; 4],
    }

    #[derive(DdsType)]
    struct ReaderData {
        a: u8,
        _padding: u8,
        b: [u8; 2],
        c: u32,
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let configuration = DustDdsConfigurationBuilder::new()
        .enable_type_information(false)
        .build()
        .unwrap();
    let participant_factory = DomainParticipantFactory::get_custom_instance(
        configuration,
        RtpsUdpTransport::default(),
        DdsSecurityPlugins::disabled(),
    );

    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<WriterData>(
            "MyData",
            WriterData::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
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
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic2 = participant2
        .create_topic::<ReaderData>(
            "MyData",
            ReaderData::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<ReaderData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(3, 0)).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(3, 0)).unwrap();

    let data = WriterData {
        a: u8::MAX / 2,
        b: u16::MAX / 2,
        c: (u32::MAX / 2).to_ne_bytes(),
    };

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(3, 0))
        .unwrap();

    let mut samples = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    let my_data = samples.remove(0).data.unwrap();
    assert_eq!(my_data.a, data.a);
    assert_eq!(my_data.b, data.b.to_ne_bytes());
    assert_eq!(my_data.c.to_ne_bytes(), data.c);
}
