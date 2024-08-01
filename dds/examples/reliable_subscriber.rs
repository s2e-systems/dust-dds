use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    topic_definition::type_support::DdsType,
};

#[derive(DdsType, Debug)]
struct ReliableExampleType {
    #[dust_dds(key)]
    id: i32,
    data: i32,
}

fn main() {
    let domain_id = 1;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ReliableExampleType>(
            "ReliableExampleTopic",
            "ReliableExampleType",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ReliableExampleType>(
            &topic,
            QosKind::Specific(reader_qos),
            None,
            NO_STATUS,
        )
        .unwrap();
    let reader_status = reader.get_statuscondition();
    reader_status
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut waitset = WaitSet::new();
    waitset
        .attach_condition(Condition::StatusCondition(reader_status))
        .unwrap();
    waitset.wait(Duration::new(30, 0)).unwrap();

    for _ in 1..=100 {
        if let Ok(samples) = reader.take_next_sample() {
            let sample = samples.data().unwrap();
            println!("Read sample: {:?}", sample);
        } else {
            println!("Error reading sample");
        }

        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}
