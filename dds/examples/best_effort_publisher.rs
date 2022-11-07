use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::Qos, status::{NO_STATUS, StatusKind}, wait_set::{WaitSet, Condition}, time::Duration},
    topic_definition::type_support::{DdsSerde, DdsType},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, DdsType, DdsSerde, Debug)]
struct BestEffortExampleType {
    id: i32,
}

fn main() {
    let domain_id = 1;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, Qos::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<BestEffortExampleType>("BestEffortExampleTopic", Qos::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(Qos::Default, None, NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter(&topic, Qos::Default, None, NO_STATUS)
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
}
