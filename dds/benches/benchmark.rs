use std::sync::{Arc, Barrier};

use criterion::{criterion_group, criterion_main, Criterion};
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::QosKind,
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::{
        data_reader::DataReader,
        data_reader_listener::DataReaderListener,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    },
    topic_definition::type_support::DdsType,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType)]
struct KeyedData {
    #[key]
    id: u8,
    value: u8,
}

pub fn best_effort_write_only(c: &mut Criterion) {
    let domain_id = 200;
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _reader = subscriber
        .create_datareader(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    // let cond = writer.get_statuscondition().unwrap();
    // cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
    //     .unwrap();

    // let mut wait_set = WaitSet::new();
    // wait_set
    //     .attach_condition(Condition::StatusCondition(cond))
    //     .unwrap();
    // wait_set.wait(Duration::new(10, 0)).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    c.bench_function("best_effort_write_only", |b| {
        b.iter(|| {
            writer.write(&KeyedData { id: 1, value: 1 }, None).unwrap();
        })
    });
}

pub fn best_effort_read_only(c: &mut Criterion) {
    let domain_id = 201;
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    // let cond = writer.get_statuscondition().unwrap();
    // cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
    //     .unwrap();

    // let mut wait_set = WaitSet::new();
    // wait_set
    //     .attach_condition(Condition::StatusCondition(cond))
    //     .unwrap();
    // wait_set.wait(Duration::new(10, 0)).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));

    writer.write(&KeyedData { id: 1, value: 1 }, None).unwrap();

    c.bench_function("best_effort_read_only", |b| {
        b.iter(|| {
            reader
                .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .ok();
        })
    });
}

fn best_effort_write_and_receive(c: &mut Criterion) {
    struct Listener {
        barrier: Arc<Barrier>,
    }
    impl DataReaderListener for Listener {
        type Foo = KeyedData;

        fn on_data_available(&mut self, the_reader: &DataReader<Self::Foo>) {
            the_reader
                .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .ok();
            self.barrier.wait();
        }
    }

    let domain_id = 202;
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<KeyedData>("TestTopic", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = barrier.clone();

    let listener = Box::new(Listener { barrier });
    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Default,
            Some(listener),
            &[StatusKind::DataAvailable, StatusKind::SubscriptionMatched],
        )
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, None, NO_STATUS)
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

    c.bench_function("best_effort_write_and_receive", |b| {
        b.iter(|| {
            writer.write(&KeyedData { id: 1, value: 7 }, None).unwrap();
            barrier_clone.wait();
        })
    });
}

criterion_group!(
    benches,
    best_effort_write_only,
    best_effort_read_only,
    best_effort_write_and_receive
);
criterion_main!(benches);
