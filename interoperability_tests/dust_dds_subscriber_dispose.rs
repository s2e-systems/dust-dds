use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        sample_info::{InstanceStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
    },
    listener::NoOpListener,
    wait_set::{Condition, WaitSet},
};

mod dispose_data {
    include!("target/idl/dispose_data.rs");
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let topic = participant
        .find_topic::<dispose_data::DisposeDataType>("DisposeData", Duration::new(120, 0))
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<dispose_data::DisposeDataType>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener,
            NO_STATUS,
        )
        .unwrap();

    let reader_cond = reader.get_statuscondition();
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

    let mut samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    if samples[0].sample_info().instance_state != InstanceStateKind::NotAliveDisposed {
        wait_set.wait(Duration::new(30, 0)).unwrap();
        samples = reader
            .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();
    }

    assert_eq!(
        samples[0].sample_info().instance_state,
        InstanceStateKind::NotAliveDisposed,
    );

    println!("Received disposed instance state");

    // Sleep to allow sending acknowledgements
    std::thread::sleep(std::time::Duration::from_secs(5));
}
