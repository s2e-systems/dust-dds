use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
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
fn on_data_available_listener() {
    mock! {
        DataAvailableListener{}

        impl DataReaderListener for DataAvailableListener {
            type Foo = MyData;

            fn on_data_available(
                &mut self,
                _the_reader: &DataReader<MyData>,
            );
        }

    }

    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
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
        ..Default::default()
    };
    let mut reader_listener = MockDataAvailableListener::new();
    reader_listener
        .expect_on_data_available()
        .once()
        .return_const(());

    let reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(Box::new(reader_listener)),
            &[StatusKind::DataAvailable],
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

    let reader_cond = reader.get_statuscondition().unwrap();
    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();

    let data1 = MyData { id: 1, value: 1 };
    writer.write(&data1, None).unwrap();

    wait_set.wait(Duration::new(2, 0)).unwrap();
}
