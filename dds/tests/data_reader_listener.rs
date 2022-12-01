use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{DeadlineQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{RequestedDeadlineMissedStatus, StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::{
        data_reader::DataReader,
        data_reader_listener::DataReaderListener,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    },
    topic_definition::type_support::{DdsSerde, DdsType},
};

use mockall::mock;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType, DdsSerde)]
struct MyData {
    #[key]
    id: u8,
    value: u8,
}

#[test]
fn deadline_missed_listener() {
    mock! {
        DeadlineMissedListener{}

        impl DataReaderListener for DeadlineMissedListener {
            type Foo = MyData;

            fn on_requested_deadline_missed(
                &mut self,
                _the_reader: &DataReader<MyData>,
                _status: RequestedDeadlineMissedStatus,
            );
        }

    }
    let domain_id = 0;

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<MyData>("MyTopic", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        deadline: DeadlineQosPolicy {
            period: Duration::new(1, 0),
        },
        ..Default::default()
    };

    let mut reader_listener = MockDeadlineMissedListener::new();
    reader_listener.expect_on_requested_deadline_missed().once();

    let reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(Box::new(reader_listener)),
            &[StatusKind::RequestedDeadlineMissed],
        )
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = MyData { id: 1, value: 1 };
    writer.write(&data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);

    std::thread::sleep(std::time::Duration::from_secs(2));
}
