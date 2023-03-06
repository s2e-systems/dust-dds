use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind, DurabilityQosPolicy, DurabilityQosPolicyKind},
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::{DdsSerde, DdsType},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, DdsType, DdsSerde)]
struct HelloWorldType {
    #[key]
    id: u8,
    msg: String,
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let reader_cond = reader.get_statuscondition().unwrap();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(reader_cond.clone()))
        .unwrap();

    wait_set.wait(Duration::new(60, 0)).unwrap();

    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    wait_set.wait(Duration::new(30, 0)).unwrap();

    let samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let hello_world = samples[0].data.as_ref().unwrap();
    println!("Received: {:?}", hello_world);

    // Sleep to allow sending acknowledgements
    std::thread::sleep(std::time::Duration::from_secs(2));
}
