use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::DataReaderQos,
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::StatusKind,
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::{DdsSerde, DdsType},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, DdsType, DdsSerde)]
struct HelloWorldType {
    id: u8,
    msg: String,
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, &[])
        .unwrap();

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", None, None, &[])
        .unwrap();

    let subscriber = participant.create_subscriber(None, None, &[]).unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic, Some(reader_qos), None, &[])
        .unwrap();

    let reader_cond = reader.get_statuscondition().unwrap();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatchedStatus])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(reader_cond.clone()))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailableStatus])
        .unwrap();
    wait_set.wait(Duration::new(30, 0)).unwrap();

    let samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let hello_world = samples[0].data.as_ref().unwrap();
    println!("Received: {:?}", hello_world);

    std::thread::sleep(std::time::Duration::from_secs(2));
}
