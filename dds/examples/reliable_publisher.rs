use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataWriterQos, QosKind},
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

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(5, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();
    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    for data in 1..=100 {
        let sample = ReliableExampleType { id: 0, data };
        writer.write(&sample, None).unwrap();
        println!("Wrote sample: {:?}", sample);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    publisher.delete_datawriter(&writer).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
}
