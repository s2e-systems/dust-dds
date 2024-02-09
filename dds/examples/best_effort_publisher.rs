use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listeners::NoOpListener,
        qos::QosKind,
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    topic_definition::type_support::DdsType,
};

#[derive(DdsType, Debug)]
struct BestEffortExampleType {
    id: i32,
}

fn main() {
    let domain_id = 1;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<BestEffortExampleType>(
            "BestEffortExampleTopic",
            "BestEffortExampleType",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let writer_cond = writer.get_statuscondition().unwrap();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    for id in 1..=10 {
        let sample = BestEffortExampleType { id };
        writer.write(&sample, None).unwrap();
        println!("Wrote sample: {:?}", sample);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    publisher.delete_datawriter(&writer).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
}
