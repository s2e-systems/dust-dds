use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataWriterQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
};

mod shapes_type;

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<shapes_type::ShapeType>("Circle", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
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

    let data = shapes_type::ShapeType {
        color: "BLUE".to_string(), //['B', 'L', 'U', 'E'].to_vec(),
        x: 140,
        y: 46,
        shapesize: 30,
    };
    loop {
        writer.write(&data, None).unwrap();

        // writer
        //     .wait_for_acknowledgments(Duration::new(30, 0))
        //     .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
