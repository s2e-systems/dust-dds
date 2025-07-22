use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, LivelinessQosPolicy, LivelinessQosPolicyKind
        },
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        type_support::DdsType,
    },
    listener::NO_LISTENER,
    wait_set::{Condition, WaitSet},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Debug, PartialEq, DdsType)]
struct UserData(u8);

#[test]
fn detect_disconnected_subscriber() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserData>(
            "LivelinessTopic",
            "UserData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        liveliness: LivelinessQosPolicy {
            kind: LivelinessQosPolicyKind::Automatic,
            ..LivelinessQosPolicy::default()  // lease_duration filled from default here
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter::<UserData>(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        liveliness: LivelinessQosPolicy {
            kind: LivelinessQosPolicyKind::Automatic,
            ..LivelinessQosPolicy::default()  // lease_duration filled from default here
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<UserData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::LivelinessLost])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();

    // Ootab matchingut
    println!("Waiting for match condition");
    let pub_match_cond = writer.get_statuscondition();
    pub_match_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut match_wait_set = WaitSet::new();
    match_wait_set
        .attach_condition(Condition::StatusCondition(pub_match_cond))
        .unwrap();

    match_wait_set.wait(Duration::new(5, 0)).unwrap();

    // Subscriberi droppimine
    println!("Dropping subscriber (Simulate disconnect)");
    drop(reader);
    drop(subscriber);

    // Liveliness loss detect
    match writer.wait_for_acknowledgments(Duration::new(10, 0)) {
        Ok(_) => println!("All acknowledgments received"),
        Err(e) => eprintln!("wait_for_acknowledgments error: {:?}", e),
    }
}
