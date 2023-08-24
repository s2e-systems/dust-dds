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
    topic_definition::type_support::{DdsKey, DdsType},
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType)]
struct KeyedData {
    #[key]
    id: u8,
    value: u8,
}

impl DdsKey for KeyedData {
    type BorrowedKeyHolder<'a> = u8;
    type OwningKeyHolder = u8;

    fn has_key() -> bool {
        true
    }

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self.id
    }

    fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {
        self.id = key_holder;
    }
}

pub fn best_effort_write_only(c: &mut Criterion) {
    let domain_id = 200;
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic("MyTopic", "KeyedData", QosKind::Default, None, NO_STATUS)
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
        .create_datareader::<KeyedData>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

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
        .create_topic("MyTopic", "KeyedData", QosKind::Default, None, NO_STATUS)
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
        .create_datareader::<KeyedData>(&topic, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

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
        sender: std::sync::mpsc::SyncSender<()>,
    }
    impl DataReaderListener for Listener {
        type Foo = KeyedData;

        fn on_data_available(&mut self, the_reader: &DataReader<Self::Foo>) {
            the_reader
                .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .ok();
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = 202;
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic("TestTopic", "KeyedData", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);

    let listener = Box::new(Listener { sender });
    let reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Default,
            Some(listener),
            &[StatusKind::DataAvailable, StatusKind::SubscriptionMatched],
        )
        .unwrap();
    let reader_cond = reader.get_statuscondition().unwrap();
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

    let mut wait_set2 = WaitSet::new();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    wait_set2
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    wait_set2.wait(Duration::new(60, 0)).unwrap();

    c.bench_function("best_effort_write_and_receive", |b| {
        b.iter(|| {
            writer.write(&KeyedData { id: 1, value: 7 }, None).unwrap();
            receiver
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
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
