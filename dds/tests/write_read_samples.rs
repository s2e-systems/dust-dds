use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsError,
        instance::InstanceHandle,
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, PublisherQos, QosKind, SubscriberQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DestinationOrderQosPolicyKind,
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            Length, LifespanQosPolicy, OwnershipQosPolicy, OwnershipQosPolicyKind,
            OwnershipStrengthQosPolicy, PresentationQosPolicy,
            PresentationQosPolicyAccessScopeKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy,
        },
        sample_info::{
            ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE, InstanceStateKind,
            SampleStateKind, ViewStateKind,
        },
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind, Time},
        type_support::DdsType,
    },
    wait_set::{Condition, WaitSet},
    xtypes::type_support::TypeSupport,
};

mod domain_id_generator;
use domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Debug, PartialEq, DdsType)]
struct UserData(u8);

#[derive(Clone, Debug, PartialEq, DdsType)]
struct KeyedData {
    #[dust_dds(key)]
    id: u8,
    value: u32,
}

#[derive(Clone, Debug, PartialEq, DdsType)]
struct LargeData {
    #[dust_dds(key)]
    id: u8,
    value: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, DdsType)]
struct StringData {
    #[dust_dds(key)]
    id: u8,
    value: String,
}

#[derive(Clone, Debug, PartialEq, DdsType)]
struct ShapeType {
    #[dust_dds(key)]
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}

#[derive(Clone, Debug, PartialEq, DdsType)]
struct CoherentData {
    #[dust_dds(key)]
    id: u8,
    value: i32,
}

#[test]
fn large_data_should_be_fragmented() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<LargeData>(
            "LargeDataTopic",
            "LargeData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<LargeData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let even_data = LargeData {
        id: 1,
        value: vec![8; 15000],
    };

    writer.write(even_data.clone(), None).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &even_data);

    let odd_data = LargeData {
        id: 1,
        value: vec![8; 15001],
    };

    writer.write(odd_data.clone(), None).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &odd_data);
}

#[test]
fn large_data_should_be_fragmented_reliable() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<LargeData>(
            "LargeDataTopic",
            "LargeData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<LargeData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(5, 0)).unwrap();

    let data = LargeData {
        id: 1,
        value: vec![8; 15000],
    };

    writer.write(data.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data);
}

#[test]
fn writer_with_keep_last_1_should_send_only_last_sample_to_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 1, value: 2 };
    let data3 = KeyedData { id: 1, value: 3 };
    let data4 = KeyedData { id: 1, value: 4 };
    let data5 = KeyedData { id: 1, value: 5 };

    writer.write(data1, None).unwrap();
    writer.write(data2, None).unwrap();
    writer.write(data3, None).unwrap();
    writer.write(data4, None).unwrap();
    writer.write(data5.clone(), None).unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    reader
        .wait_for_historical_data(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data5);
}

#[test]
fn writer_with_keep_last_3_should_send_last_3_samples_to_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(3),
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 1, value: 2 };
    let data3 = KeyedData { id: 1, value: 3 };
    let data4 = KeyedData { id: 1, value: 4 };
    let data5 = KeyedData { id: 1, value: 5 };

    writer.write(data1, None).unwrap();
    writer.write(data2, None).unwrap();
    writer.write(data3.clone(), None).unwrap();
    writer.write(data4.clone(), None).unwrap();
    writer.write(data5.clone(), None).unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    reader
        .wait_for_historical_data(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(
        samples.len(),
        3,
        "Received wrong number of samples. Received samples: {samples:?}",
    );
    assert_eq!(samples[0].data.as_ref().unwrap(), &data3);
    assert_eq!(samples[1].data.as_ref().unwrap(), &data4);
    assert_eq!(samples[2].data.as_ref().unwrap(), &data5);
}

#[test]
fn samples_are_taken() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };
    let data4 = KeyedData { id: 4, value: 30 };
    let data5 = KeyedData { id: 5, value: 40 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();
    writer.write(data4.clone(), None).unwrap();
    writer.write(data5.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples1 = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let samples2 = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples1.len(), 3);
    assert_eq!(samples1[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples1[1].data.as_ref().unwrap(), &data2);
    assert_eq!(samples1[2].data.as_ref().unwrap(), &data3);

    assert_eq!(samples2.len(), 2);
    assert_eq!(samples2[0].data.as_ref().unwrap(), &data4);
    assert_eq!(samples2[1].data.as_ref().unwrap(), &data5);
}

#[test]
fn wait_for_samples_to_be_taken_best_effort() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };
    let data4 = KeyedData { id: 4, value: 30 };
    let data5 = KeyedData { id: 5, value: 40 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();
    writer.write(data4.clone(), None).unwrap();
    writer.write(data5.clone(), None).unwrap();

    let start_time = std::time::Instant::now();
    let mut samples = Vec::new();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if let Ok(mut sample) = reader.take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        {
            samples.push(sample.remove(0))
        }

        if samples.len() >= 5 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    assert_eq!(samples.len(), 5);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
    assert_eq!(samples[2].data.as_ref().unwrap(), &data3);
    assert_eq!(samples[3].data.as_ref().unwrap(), &data4);
    assert_eq!(samples[4].data.as_ref().unwrap(), &data5);
}

#[test]
fn read_only_unread_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };
    let data4 = KeyedData { id: 4, value: 30 };
    let data5 = KeyedData { id: 5, value: 40 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();
    writer.write(data4.clone(), None).unwrap();
    writer.write(data5.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples1 = reader
        .read(
            3,
            &[SampleStateKind::NotRead],
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples2 = reader
        .read(
            3,
            &[SampleStateKind::NotRead],
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples3 = reader.read(
        3,
        &[SampleStateKind::NotRead],
        ANY_VIEW_STATE,
        ANY_INSTANCE_STATE,
    );

    assert_eq!(samples1.len(), 3);
    assert_eq!(samples1[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples1[1].data.as_ref().unwrap(), &data2);
    assert_eq!(samples1[2].data.as_ref().unwrap(), &data3);

    assert_eq!(samples2.len(), 2);
    assert_eq!(samples2[0].data.as_ref().unwrap(), &data4);
    assert_eq!(samples2[1].data.as_ref().unwrap(), &data5);

    assert!(matches!(samples3, Err(DdsError::NoData)));
}

#[test]
fn read_next_sample() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap(),
        &data1
    );
    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap(),
        &data2
    );
    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap(),
        &data3
    );
    assert!(matches!(reader.read_next_sample(), Err(DdsError::NoData)));
}

#[test]
fn take_next_sample() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.take_next_sample().unwrap().data.as_ref().unwrap(),
        &data1
    );
    assert_eq!(
        reader.take_next_sample().unwrap().data.as_ref().unwrap(),
        &data2
    );
    assert_eq!(
        reader.take_next_sample().unwrap().data.as_ref().unwrap(),
        &data3
    );
    assert!(matches!(reader.take_next_sample(), Err(DdsError::NoData)));
}

#[test]
fn each_key_sample_is_read() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();

    let data1_handle = writer.lookup_instance(data1.clone()).unwrap();
    let data2_handle = writer.lookup_instance(data2.clone()).unwrap();
    let data3_handle = writer.lookup_instance(data3.clone()).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(
        samples[0].sample_info.instance_handle,
        data1_handle.unwrap(),
    );

    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
    assert_eq!(
        samples[1].sample_info.instance_handle,
        data2_handle.unwrap(),
    );

    assert_eq!(samples[2].data.as_ref().unwrap(), &data3);
    assert_eq!(
        samples[2].sample_info.instance_handle,
        data3_handle.unwrap(),
    );
}

#[test]
fn read_specific_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2, None).unwrap();
    writer.write(data3, None).unwrap();

    let data1_handle = writer.lookup_instance(data1.clone()).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read_instance(
            3,
            data1_handle.unwrap(),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
}

#[test]
fn read_next_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples1 = reader
        .read_next_instance(
            3,
            None,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples2 = reader
        .read_next_instance(
            3,
            Some(samples1[0].sample_info.instance_handle),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples3 = reader
        .read_next_instance(
            3,
            Some(samples2[0].sample_info.instance_handle),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples4 = reader.read_next_instance(
        3,
        Some(samples3[0].sample_info.instance_handle),
        ANY_SAMPLE_STATE,
        ANY_VIEW_STATE,
        ANY_INSTANCE_STATE,
    );

    assert_eq!(samples1[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples2[0].data.as_ref().unwrap(), &data2);
    assert_eq!(samples3[0].data.as_ref().unwrap(), &data3);
    assert!(matches!(samples4, Err(DdsError::NoData)));
}

#[test]
fn take_next_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples1 = reader
        .take_next_instance(
            3,
            None,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples2 = reader
        .take_next_instance(
            3,
            Some(samples1[0].sample_info.instance_handle),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples3 = reader
        .take_next_instance(
            3,
            Some(samples2[0].sample_info.instance_handle),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    let samples4 = reader.take_next_instance(
        3,
        Some(samples3[0].sample_info.instance_handle),
        ANY_SAMPLE_STATE,
        ANY_VIEW_STATE,
        ANY_INSTANCE_STATE,
    );

    assert_eq!(samples1[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples2[0].data.as_ref().unwrap(), &data2);
    assert_eq!(samples3[0].data.as_ref().unwrap(), &data3);
    assert!(matches!(samples4, Err(DdsError::NoData)));
}

#[test]
fn take_specific_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2, None).unwrap();
    writer.write(data3, None).unwrap();
    let data1_handle = writer.lookup_instance(data1.clone()).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take_instance(
            3,
            data1_handle.unwrap(),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        )
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
}

#[test]
fn take_specific_unknown_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 10 };
    let data3 = KeyedData { id: 3, value: 20 };

    writer.write(data1, None).unwrap();
    writer.write(data2, None).unwrap();
    writer.write(data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert!(matches!(
        reader.take_instance(
            3,
            InstanceHandle::new([99; 16]),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ),
        Err(DdsError::BadParameter)
    ));
}

#[test]
fn write_read_disposed_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };

    writer.write(data1.clone(), None).unwrap();
    writer.dispose(data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(2, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
    assert_eq!(
        samples[1].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
}

#[test]
fn write_read_disposed_samples_when_writer_is_immediately_deleted() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let topic2 = participant2
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };

    writer.write(data1.clone(), None).unwrap();
    writer.dispose(data1, None).unwrap();

    let start_loop_time = std::time::Instant::now();
    let wait_for_disposed_timeout = std::time::Duration::from_secs(10);
    let samples = loop {
        if let Ok(samples) = reader.read(2, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE) {
            if samples.len() == 2
                && samples
                    .iter()
                    .all(|s| s.sample_info.instance_state == InstanceStateKind::NotAliveDisposed)
            {
                break samples;
            }
        }

        if start_loop_time.elapsed() > wait_for_disposed_timeout {
            panic!("Disposed sample not received within expected time.")
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };

    publisher.delete_datawriter(&writer).unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
    assert_eq!(
        samples[1].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
}

#[test]
fn write_read_sample_view_state() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "OtherTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };

    writer.write(data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let data1_2 = KeyedData { id: 1, value: 2 };
    let data2 = KeyedData { id: 2, value: 1 };

    writer.write(data1_2, None).unwrap();
    writer.write(data2, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(2, 0))
        .unwrap();

    let samples = reader
        .read(2, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].sample_info.view_state, ViewStateKind::NotNew);
    assert_eq!(samples[1].sample_info.view_state, ViewStateKind::New);
}

#[test]
fn reader_with_minimum_time_separation_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        time_based_filter: TimeBasedFilterQosPolicy {
            minimum_separation: DurationKind::Finite(Duration::new(2, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1_1 = KeyedData { id: 1, value: 1 };
    let data1_2 = KeyedData { id: 1, value: 2 };
    let data1_3 = KeyedData { id: 1, value: 3 };
    let data2_1 = KeyedData { id: 2, value: 10 };
    let data2_2 = KeyedData { id: 2, value: 20 };
    let data2_3 = KeyedData { id: 2, value: 30 };

    writer
        .write_w_timestamp(data1_1.clone(), None, Time::new(1, 0))
        .unwrap();
    writer
        .write_w_timestamp(data1_2, None, Time::new(2, 0))
        .unwrap();
    writer
        .write_w_timestamp(data1_3.clone(), None, Time::new(3, 0))
        .unwrap();
    writer
        .write_w_timestamp(data2_1.clone(), None, Time::new(4, 0))
        .unwrap();
    writer
        .write_w_timestamp(data2_2, None, Time::new(5, 0))
        .unwrap();
    writer
        .write_w_timestamp(data2_3.clone(), None, Time::new(6, 0))
        .unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 4);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1_1);
    assert_eq!(samples[1].data.as_ref().unwrap(), &data1_3);
    assert_eq!(samples[2].data.as_ref().unwrap(), &data2_1);
    assert_eq!(samples[3].data.as_ref().unwrap(), &data2_3);
}

#[test]
fn reader_with_minimum_time_separation_qos_taking_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        time_based_filter: TimeBasedFilterQosPolicy {
            minimum_separation: DurationKind::Finite(Duration::new(2, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1_1 = KeyedData { id: 1, value: 1 };
    let data1_2 = KeyedData { id: 1, value: 2 };
    let data1_3 = KeyedData { id: 1, value: 3 };

    writer
        .write_w_timestamp(data1_1.clone(), None, Time::new(1, 0))
        .unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1_1);

    writer
        .write_w_timestamp(data1_2, None, Time::new(2, 0))
        .unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader.take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE);
    assert_eq!(samples, Err(DdsError::NoData));

    writer
        .write_w_timestamp(data1_3.clone(), None, Time::new(3, 0))
        .unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1_3);
}

#[test]
fn transient_local_writer_reader_wait_for_historical_data() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 2 };
    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        time_based_filter: TimeBasedFilterQosPolicy {
            minimum_separation: DurationKind::Finite(Duration::new(2, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    reader
        .wait_for_historical_data(Duration::new(10, 0))
        .unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
}

#[test]
fn volatile_writer_reader_receives_only_new_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::Volatile,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::Volatile,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data2 = KeyedData { id: 2, value: 10 };
    writer.write(data2.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn write_read_unkeyed_topic() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserData>(
            "write_read_unkeyed_topic",
            "UserData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<UserData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    writer.write(UserData(8), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(5, 0))
        .unwrap();

    let samples = reader.read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE);

    assert_eq!(samples.unwrap()[0].data.as_ref().unwrap(), &UserData(8));
}

#[test]
fn data_reader_resource_limits() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserData>(
            "data_reader_resource_limits",
            "UserData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: Length::Limited(2),
            max_instances: Length::Unlimited,
            max_samples_per_instance: Length::Limited(2),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<UserData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    writer.write(UserData(1), None).unwrap();
    writer.write(UserData(2), None).unwrap();
    writer.write(UserData(3), None).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SampleRejected])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();

    wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
}

#[test]
fn data_reader_order_by_source_timestamp() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserData>(
            "MyTopic",
            "UserData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        destination_order: DestinationOrderQosPolicy {
            kind: DestinationOrderQosPolicyKind::BySourceTimestamp,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        destination_order: DestinationOrderQosPolicy {
            kind: DestinationOrderQosPolicyKind::BySourceTimestamp,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<UserData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    writer
        .write_w_timestamp(UserData(1), None, Time::new(30, 0))
        .unwrap();
    writer
        .write_w_timestamp(UserData(2), None, Time::new(20, 0))
        .unwrap();
    writer
        .write_w_timestamp(UserData(3), None, Time::new(10, 0))
        .unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].data.as_ref().unwrap(), &UserData(3));
    assert_eq!(samples[1].data.as_ref().unwrap(), &UserData(2));
    assert_eq!(samples[2].data.as_ref().unwrap(), &UserData(1));
}

#[test]
fn data_reader_publication_handle_sample_info() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<UserData>(
            "MyTopic",
            "UserData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<UserData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    writer.write(UserData(1), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert!(
        reader
            .get_matched_publication_data(samples[0].sample_info.publication_handle)
            .is_ok()
    );
}

#[test]
fn volatile_writer_with_reader_new_reader_receives_only_new_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::Volatile,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::Volatile,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let _reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos.clone()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();
    writer.get_publication_matched_status().unwrap(); // To reset wait_set for subsequent calls

    let data1 = KeyedData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(3, 0))
        .unwrap();

    let reader_new = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Wait for writer to match reader
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data2 = KeyedData { id: 2, value: 10 };
    writer.write(data2.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader_new
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn write_read_unregistered_samples_not_alive_no_writers() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };

    writer.write(data1.clone(), None).unwrap();
    writer.unregister_instance(data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(2, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::NotAliveNoWriters
    );
    assert_eq!(
        samples[1].sample_info.instance_state,
        InstanceStateKind::NotAliveNoWriters
    );
}

#[test]
fn multiple_writers_unregister_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer1 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos.clone()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer2 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let start_time = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start_time) < std::time::Duration::from_secs(10)
    {
        if reader.get_matched_publications().unwrap().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        reader.get_matched_publications().unwrap().len(),
        2,
        "Reader must have 2 matched writers"
    );

    let cond1 = writer1.get_statuscondition();
    cond1
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set1 = WaitSet::new();
    wait_set1
        .attach_condition(Condition::StatusCondition(cond1))
        .unwrap();
    wait_set1.wait(Duration::new(10, 0)).unwrap();

    let cond2 = writer2.get_statuscondition();
    cond2
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set2 = WaitSet::new();
    wait_set2
        .attach_condition(Condition::StatusCondition(cond2))
        .unwrap();
    wait_set2.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };

    writer1.write(data1.clone(), None).unwrap();
    writer2.write(data1.clone(), None).unwrap();
    writer1.unregister_instance(data1.clone(), None).unwrap();

    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    // Since writer1 unregisters with autodispose=true and writer2 is still registered, instance is NotAliveDisposed
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
    assert_eq!(
        samples[1].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
    assert_eq!(
        samples[2].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );

    // Now writer2 unregisters as well
    writer2.unregister_instance(data1, None).unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples2 = reader
        .read(4, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples2.len(), 4);
    // Since instance was disposed by writer1 (autodispose=true), it remains NotAliveDisposed
    assert_eq!(
        samples2[3].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
}

#[test]
fn data_reader_instance_state_not_alive_no_writers_when_writer_is_deleted() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant_factory = DomainParticipantFactory::get_instance();
    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let topic2 = participant2
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    writer.write(data1.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::Alive
    );

    reader.get_subscription_matched_status().unwrap();

    // Delete writer / publisher / participant1
    participant1.delete_contained_entities().unwrap();
    participant_factory
        .delete_participant(&participant1)
        .unwrap();

    // Wait for subscription matched status to update or reader to detect writer deletion
    let cond_reader = reader.get_statuscondition();
    cond_reader
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set_reader = WaitSet::new();
    wait_set_reader
        .attach_condition(Condition::StatusCondition(cond_reader))
        .unwrap();
    wait_set_reader.wait(Duration::new(10, 0)).unwrap();

    let samples_after_delete = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(
        samples_after_delete[0].sample_info.instance_state,
        InstanceStateKind::NotAliveNoWriters
    );
}

#[test]
fn data_reader_instance_state_not_alive_disposed_when_writer_disposes_and_is_deleted() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant_factory = DomainParticipantFactory::get_instance();
    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let topic2 = participant2
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    writer.write(data1.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::Alive
    );

    writer.dispose(data1, None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    reader.get_subscription_matched_status().unwrap();

    // Delete writer / publisher / participant1
    participant1.delete_contained_entities().unwrap();
    participant_factory
        .delete_participant(&participant1)
        .unwrap();

    // Wait for subscription matched status to update or reader to detect writer deletion
    let cond_reader = reader.get_statuscondition();
    cond_reader
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set_reader = WaitSet::new();
    wait_set_reader
        .attach_condition(Condition::StatusCondition(cond_reader))
        .unwrap();
    wait_set_reader.wait(Duration::new(10, 0)).unwrap();

    let samples_after_delete = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(
        samples_after_delete[0].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
}

#[test]
fn transient_local_writer_does_not_deliver_lifespan_expired_data_at_write() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        lifespan: LifespanQosPolicy {
            duration: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 2 };
    writer
        .write_w_timestamp(data1, None, Time::new(0, 0))
        .unwrap();
    writer
        .write_w_timestamp(data2.clone(), None, Time::new(i32::MAX, 0))
        .unwrap(); // Never stale sample

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    reader
        .wait_for_historical_data(Duration::new(10, 0))
        .unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn transient_local_writer_does_not_deliver_lifespan_expired_data_after_write() {
    const LIFESPAN_MS: u32 = 100;
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        lifespan: LifespanQosPolicy {
            duration: DurationKind::Finite(Duration::new(0, LIFESPAN_MS * 1_000_000)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 2 };
    writer.write(data1, None).unwrap();
    writer
        .write_w_timestamp(data2.clone(), None, Time::new(i32::MAX, 0))
        .unwrap(); // Never stale sample

    std::thread::sleep(std::time::Duration::from_millis(LIFESPAN_MS as u64 * 10));

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    reader
        .wait_for_historical_data(Duration::new(10, 0))
        .unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn reader_joining_after_writer_writes_many_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(5, 0)),
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    for value in 0..500 {
        let data = KeyedData { id: 1, value };
        writer.write(data, None).unwrap();
    }

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(5, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let new_data = KeyedData { id: 1, value: 1000 };
    writer.write(new_data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(20, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &new_data);
}

#[test]
fn reader_with_exclusive_ownership_should_not_read_samples_from_second_weaker_writer() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 10 },
        ..Default::default()
    };
    let writer1 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer1_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer2_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 1 },
        ..Default::default()
    };
    let writer2 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer2_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let start_time = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start_time) < std::time::Duration::from_secs(10)
    {
        if reader.get_matched_publications().unwrap().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        reader.get_matched_publications().unwrap().len(),
        2,
        "Reader must have 2 matched writers"
    );

    let cond = writer1.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = writer2.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 10 };
    writer1.write(data1.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let data2 = KeyedData { id: 1, value: 20 };
    writer2.write(data2, None).unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
}

#[test]
fn reader_with_exclusive_ownership_should_read_samples_from_second_writer_with_higher_ownership_strength()
 {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 1 },
        ..Default::default()
    };
    let writer1 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer1_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer2_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 10 },
        ..Default::default()
    };
    let writer2 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer2_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let start_time = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start_time) < std::time::Duration::from_secs(10)
    {
        if reader.get_matched_publications().unwrap().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        reader.get_matched_publications().unwrap().len(),
        2,
        "Reader must have 2 matched writers"
    );

    let cond = writer1.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = writer2.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 10 };
    writer1.write(data1.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let data2 = KeyedData { id: 1, value: 20 };
    writer2.write(data2.clone(), None).unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
}

#[test]
fn reader_with_exclusive_ownership_should_read_samples_from_second_writer_after_deadline_missed() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 10 },
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer1 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer1_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer2_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 1 },
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer2 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer2_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let start_time = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start_time) < std::time::Duration::from_secs(10)
    {
        if reader.get_matched_publications().unwrap().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        reader.get_matched_publications().unwrap().len(),
        2,
        "Reader must have 2 matched writers"
    );

    let cond = writer1.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = writer2.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 10 };
    writer1.write(data1.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::RequestedDeadlineMissed])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data2 = KeyedData { id: 1, value: 20 };
    writer2.write(data2.clone(), None).unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
}

#[test]
fn reader_with_exclusive_ownership_should_read_samples_from_second_weaker_writer_after_unregister()
{
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 10 },
        ..Default::default()
    };
    let writer1 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer1_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer2_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 1 },
        ..Default::default()
    };
    let writer2 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer2_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let start_time = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start_time) < std::time::Duration::from_secs(10)
    {
        if reader.get_matched_publications().unwrap().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        reader.get_matched_publications().unwrap().len(),
        2,
        "Reader must have 2 matched writers"
    );

    let cond = writer1.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = writer2.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 10 };
    writer1.write(data1.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    writer1.unregister_instance(data1.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let data2 = KeyedData { id: 1, value: 20 };
    writer2.write(data2.clone(), None).unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples[2].data.as_ref().unwrap(), &data2);
}

#[test]
fn reader_with_exclusive_ownership_should_read_samples_from_second_weaker_writer_after_dispose() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 10 },
        ..Default::default()
    };
    let writer1 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer1_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer2_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ownership_strength: OwnershipStrengthQosPolicy { value: 1 },
        ..Default::default()
    };
    let writer2 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer2_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Exclusive,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let start_time = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start_time) < std::time::Duration::from_secs(10)
    {
        if reader.get_matched_publications().unwrap().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        reader.get_matched_publications().unwrap().len(),
        2,
        "Reader must have 2 matched writers"
    );

    let cond = writer1.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = writer2.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 10 };
    writer1.write(data1.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    writer1.dispose(data1.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let data2 = KeyedData { id: 1, value: 20 };
    writer2.write(data2.clone(), None).unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples[2].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_are_transfered_between_two_participants() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant1
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let topic = participant2
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(100, 0)).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data = KeyedData { id: 1, value: 1 };

    writer.write(data.clone(), None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data);
}

#[test]
fn shared_ownership_writer1_should_write_and_writer2_should_dispose_same_data() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>(
            "MyTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Shared,
        },
        ..Default::default()
    };
    let writer1 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer1_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer2_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Shared,
        },
        ..Default::default()
    };
    let writer2 = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer2_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ownership: OwnershipQosPolicy {
            kind: OwnershipQosPolicyKind::Shared,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let start_time = std::time::Instant::now();
    while std::time::Instant::now().duration_since(start_time) < std::time::Duration::from_secs(10)
    {
        if reader.get_matched_publications().unwrap().len() >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        reader.get_matched_publications().unwrap().len(),
        2,
        "Reader must have 2 matched writers"
    );

    let cond = writer1.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = writer2.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data = KeyedData { id: 1, value: 10 };
    writer1.write(data.clone(), None).unwrap();
    writer1
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data);
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::Alive
    );

    let _handle = writer2.register_instance(data.clone()).unwrap();
    writer2.dispose(data, None).unwrap();
    writer2
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert!(samples[0].data.is_none());
    assert_eq!(
        samples[0].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
}

#[test]
fn data_reader_does_not_read_lifespan_expired_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<KeyedData>(
            "LifespanTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        lifespan: LifespanQosPolicy {
            duration: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic2 = participant2
        .create_topic::<KeyedData>(
            "LifespanTopic",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 1, value: 2 };

    writer
        .write_w_timestamp(data1, None, Time::new(0, 0))
        .unwrap();
    writer
        .write_w_timestamp(data2.clone(), None, Time::new(i32::MAX, 0))
        .unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

// Port of Test_Lifespan_2 from the OMG DDS-RTPS interoperability test suite
#[test]
fn data_reader_removes_stale_samples_when_read_period_exceeds_lifespan() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<KeyedData>(
            "Square",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        lifespan: LifespanQosPolicy {
            duration: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic2 = participant2
        .create_topic::<KeyedData>(
            "Square",
            "KeyedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<KeyedData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let writer_thread = std::thread::spawn(move || {
        for i in 1..=10 {
            writer.write(KeyedData { id: 1, value: i }, None).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    });

    std::thread::sleep(std::time::Duration::from_millis(2000));
    let samples1 = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(2000));
    let samples2 = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    writer_thread.join().unwrap();

    assert!(
        samples1.len() == 2 || samples1.len() == 3,
        "Expected 2 or 3 samples in first read, got {}",
        samples1.len()
    );
    assert!(
        samples2.len() == 2 || samples2.len() == 3,
        "Expected 2 or 3 samples in second read, got {}",
        samples2.len()
    );

    for window in samples1.windows(2) {
        let v1 = window[0].data.as_ref().unwrap().value;
        let v2 = window[1].data.as_ref().unwrap().value;
        assert_eq!(v1 + 1, v2, "Samples in first read must be consecutive");
    }
    for window in samples2.windows(2) {
        let v1 = window[0].data.as_ref().unwrap().value;
        let v2 = window[1].data.as_ref().unwrap().value;
        assert_eq!(v1 + 1, v2, "Samples in second read must be consecutive");
    }

    let last_s1 = samples1.last().unwrap().data.as_ref().unwrap().value;
    let first_s2 = samples2.first().unwrap().data.as_ref().unwrap().value;
    assert!(
        first_s2 > last_s1 + 1,
        "Expected gap between batches due to lifespan expiration, but got last_s1={last_s1}, first_s2={first_s2}"
    );
}

#[test]
fn raw_struct_and_field_names_are_unrawed_in_metadata_and_hash_ids() {
    #[derive(DdsType)]
    #[dust_dds(autoid = "hash")]
    struct r#Type {
        r#match: u32,
        #[dust_dds(hashid)]
        r#async: u32,
    }

    #[derive(DdsType)]
    #[dust_dds(autoid = "hash")]
    struct Reference {
        #[dust_dds(name = "match")]
        field_one: u32,
        #[dust_dds(name = "async", hashid)]
        field_two: u32,
    }

    #[derive(DdsType)]
    #[dust_dds(name = "r#Type", autoid = "hash")]
    struct Explicit {
        #[dust_dds(name = "r#match")]
        r#match: u32,
    }

    let raw = r#Type::get_type();
    let reference = Reference::get_type();
    let explicit = Explicit::get_type();

    assert_eq!(raw.get_name(), "Type");
    assert_eq!(explicit.get_name(), "r#Type");
    for index in 0..2 {
        let raw_member = raw.get_member_by_index(index).unwrap();
        let reference_member = reference.get_member_by_index(index).unwrap();
        assert_eq!(raw_member.get_name(), reference_member.get_name());
        assert_eq!(raw_member.get_id(), reference_member.get_id());
    }
    assert_eq!(raw.get_member_by_index(0).unwrap().get_name(), "match");
    assert_eq!(raw.get_member_by_index(0).unwrap().get_id(), 26_397_923);
    assert_eq!(raw.get_member_by_index(1).unwrap().get_name(), "async");
    assert_eq!(raw.get_member_by_index(1).unwrap().get_id(), 71_235_853);
    assert_eq!(
        explicit.get_member_by_index(0).unwrap().get_name(),
        "r#match"
    );
    assert_eq!(
        explicit.get_member_by_index(0).unwrap().get_id(),
        72_379_351
    );
}

#[test]
fn raw_enum_and_union_names_are_unrawed_in_metadata() {
    #[derive(DdsType)]
    enum r#Enum {
        Value,
    }

    #[derive(DdsType)]
    #[dust_dds(switch(u8))]
    enum r#Union {
        #[dust_dds(case = 0)]
        r#Type(u32),
    }

    #[derive(DdsType)]
    #[dust_dds(name = "r#Enum")]
    enum ExplicitEnum {
        Value,
    }

    #[derive(DdsType)]
    #[dust_dds(name = "r#Union", switch(u8))]
    enum ExplicitUnion {
        #[dust_dds(case = 0, name = "r#Type")]
        r#Type(u32),
    }

    assert_eq!(r#Enum::get_type().get_name(), "Enum");
    assert_eq!(r#Union::get_type().get_name(), "Union");
    assert_eq!(
        r#Union::get_type()
            .get_member_by_index(1)
            .unwrap()
            .get_name(),
        "Type"
    );
    assert_eq!(ExplicitEnum::get_type().get_name(), "r#Enum");
    assert_eq!(ExplicitUnion::get_type().get_name(), "r#Union");
    assert_eq!(
        ExplicitUnion::get_type()
            .get_member_by_index(1)
            .unwrap()
            .get_name(),
        "r#Type"
    );
}

#[test]
fn custom_struct_member_name_should_be_used_for_hash_autoid() {
    #[derive(DdsType)]
    #[dust_dds(name = "HashNamedData", autoid = "hash")]
    struct FirstRustType {
        value: u32,
    }

    #[derive(DdsType)]
    #[dust_dds(name = "HashNamedData", autoid = "hash")]
    struct SecondRustType {
        #[dust_dds(name = "value")]
        second_rust_field: u32,
    }

    #[derive(DdsType)]
    #[dust_dds(name = "HashNamedData", autoid = "hash")]
    struct MismatchedRustType {
        mismatched_rust_field: u32,
    }

    let first_type = FirstRustType::get_type();
    let second_type = SecondRustType::get_type();
    let mismatched_type = MismatchedRustType::get_type();

    let first_member = first_type.get_member_by_index(0).unwrap();
    let second_member = second_type.get_member_by_index(0).unwrap();
    let mismatched_member = mismatched_type.get_member_by_index(0).unwrap();

    assert_eq!(first_member.get_name(), "value");
    assert_eq!(second_member.get_name(), "value");
    assert_eq!(mismatched_member.get_name(), "mismatched_rust_field");
    assert_eq!(first_member.get_id(), second_member.get_id());
    assert_ne!(first_member.get_id(), mismatched_member.get_id());

    assert_eq!(first_type, second_type);
    assert_ne!(first_type, mismatched_type);
}

#[test]
fn custom_struct_and_union_variant_name_should_read_and_write() {
    #[derive(Clone, Debug, DdsType)]
    struct Data {
        #[dust_dds(key)]
        id: u32,
        value: DataValue,
    }

    #[derive(Clone, Debug, DdsType)]
    #[dust_dds(switch(u8))]
    enum DataValue {
        #[dust_dds(case = 0)]
        A,
        #[dust_dds(case = 1)]
        B(u16),
        #[dust_dds(case = 2)]
        C { c: u32 },
    }

    #[derive(Clone, Debug, DdsType)]
    #[dust_dds(name = "Data")]
    struct MyData {
        #[dust_dds(key, name = "id")]
        my_id: u32,
        #[dust_dds(name = "value")]
        my_value: MyDataValue,
    }

    #[derive(Clone, Debug, DdsType)]
    #[dust_dds(name = "DataValue", switch(u8))]
    enum MyDataValue {
        #[dust_dds(case = 0, name = "A")]
        MyA,
        #[dust_dds(case = 1, name = "B")]
        MyB(u16),
        #[dust_dds(case = 2, name = "C")]
        MyC { my_c: u32 },
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<Data>(
            "Data",
            Data::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic2 = participant2
        .create_topic::<MyData>(
            "Data",
            MyData::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(3, 0)).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(3, 0)).unwrap();

    let data = Data {
        id: 1,
        value: DataValue::C { c: 1 },
    };

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(3, 0))
        .unwrap();

    let mut samples = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    let my_data = samples.remove(0).data.unwrap();
    assert_eq!(my_data.my_id, data.id);
    match (my_data.my_value, data.value) {
        (MyDataValue::MyA, DataValue::A) => (),
        (MyDataValue::MyB(my_b), DataValue::B(b)) if my_b == b => (),
        (MyDataValue::MyC { my_c }, DataValue::C { c }) if my_c == c => (),
        _ => panic!(),
    }
}

#[test]
fn mismatched_struct_field_name_should_not_read_and_write() {
    #[derive(Clone, Debug, DdsType)]
    struct Data {
        #[dust_dds(key)]
        id: u32,
        value: u8,
    }

    #[derive(Clone, Debug, DdsType)]
    #[dust_dds(name = "Data")]
    struct MyData {
        #[dust_dds(key, name = "id")]
        my_id: u32,
        #[dust_dds(name = "invalid")]
        my_value: u8,
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<Data>(
            "Data",
            Data::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter::<Data>(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic2 = participant2
        .create_topic::<MyData>(
            "Data",
            MyData::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();

    let data_type = Data::get_type();
    let my_data_type = MyData::get_type();
    assert_eq!(
        data_type.get_member_by_index(1).unwrap().get_name(),
        "value"
    );
    assert_eq!(
        my_data_type.get_member_by_index(1).unwrap().get_name(),
        "invalid"
    );
    assert_ne!(data_type, my_data_type);

    assert!(matches!(
        wait_set.wait(Duration::new(3, 0)),
        Err(DdsError::Timeout)
    ));
}

#[test]
fn custom_union_variant_name_should_not_read_and_write() {
    #[derive(Clone, Debug, DdsType)]
    #[dust_dds(switch(u8))]
    enum DataValue {
        #[dust_dds(case = 0)]
        A,
        #[dust_dds(case = 1)]
        B(u16),
        #[dust_dds(case = 2)]
        C { c: u32 },
    }

    #[derive(Clone, Debug, DdsType)]
    #[dust_dds(name = "DataValue", switch(u8))]
    enum MyDataValue {
        #[dust_dds(case = 0, name = "A")]
        MyA,
        #[dust_dds(case = 1, name = "B")]
        MyB(u16),
        #[dust_dds(case = 2, name = "invalid")]
        MyC { my_c: u32 },
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic1 = participant1
        .create_topic::<DataValue>(
            "DataValue",
            DataValue::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant1
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter::<DataValue>(
            &topic1,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic2 = participant2
        .create_topic::<MyDataValue>(
            "DataValue",
            MyDataValue::get_type().get_name(),
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyDataValue>(
            &topic2,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();

    let data_value_type = DataValue::get_type();
    let my_data_value_type = MyDataValue::get_type();
    assert_eq!(
        data_value_type.get_member_by_index(3).unwrap().get_name(),
        "C"
    );
    assert_eq!(
        my_data_value_type
            .get_member_by_index(3)
            .unwrap()
            .get_name(),
        "invalid"
    );
    assert_ne!(data_value_type, my_data_value_type);

    assert!(matches!(
        wait_set.wait(Duration::new(3, 0)),
        Err(DdsError::Timeout)
    ));
}

#[test]
fn samples_should_be_content_filtered() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<StringData>(
            "MyTopic",
            "StringData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "MyTopicFiltered",
            &topic,
            String::from("value = %0"),
            vec![String::from("RED")],
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<StringData>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = StringData {
        id: 1,
        value: String::from("BLUE"),
    };
    let data2 = StringData {
        id: 2,
        value: String::from("RED"),
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_shapesize() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "ShapeTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "ShapeTopicFiltered",
            &topic,
            String::from("shapesize <= 20"),
            vec![],
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = ShapeType {
        color: String::from("BLUE"),
        x: 100,
        y: 100,
        shapesize: 30,
    };
    let data2 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 20,
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_parameterized_shapesize() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "ShapeTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "ShapeTopicFiltered",
            &topic,
            String::from("shapesize <= %0"),
            vec![String::from("20")],
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = ShapeType {
        color: String::from("BLUE"),
        x: 100,
        y: 100,
        shapesize: 30,
    };
    let data2 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 20,
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_literal_string() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<StringData>(
            "MyTopic",
            "StringData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "MyTopicFiltered",
            &topic,
            String::from("value = 'RED'"),
            vec![],
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<StringData>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = StringData {
        id: 1,
        value: String::from("BLUE"),
    };
    let data2 = StringData {
        id: 2,
        value: String::from("RED"),
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data2);
}

#[test]
fn samples_should_be_content_filtered_by_compound_expression() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "ShapeTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let content_filtered_topic = participant
        .create_contentfilteredtopic(
            "ShapeTopicFiltered",
            &topic,
            String::from("(color = 'RED') AND (shapesize <= 20)"),
            vec![],
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &content_filtered_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 15,
    };
    let data2 = ShapeType {
        color: String::from("RED"),
        x: 100,
        y: 100,
        shapesize: 25,
    };
    let data3 = ShapeType {
        color: String::from("BLUE"),
        x: 100,
        y: 100,
        shapesize: 15,
    };

    writer.write(data1.clone(), None).unwrap();
    writer.write(data2.clone(), None).unwrap();
    writer.write(data3.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(5, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
}

// Test_CoherentSets_10: Test the behavior of coherent access with INSTANCE_PRESENTATION with several instances
#[test]
fn coherent_sets_with_multiple_instances_and_topics() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic_names = ["Square", "Square1", "Square2"];
    let mut topics = Vec::new();
    for topic_name in topic_names {
        let topic = participant
            .create_topic::<ShapeType>(
                topic_name,
                "ShapeType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        topics.push(topic);
    }

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let mut writers = Vec::new();
    for topic in &topics {
        let writer = publisher
            .create_datawriter(
                topic,
                QosKind::Specific(writer_qos.clone()),
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        writers.push(writer);
    }

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let mut readers = Vec::new();
    for topic in &topics {
        let reader = subscriber
            .create_datareader::<ShapeType>(
                topic,
                QosKind::Specific(reader_qos.clone()),
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        readers.push(reader);
    }

    for writer in &writers {
        let cond = writer.get_statuscondition();
        cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
            .unwrap();

        let mut wait_set = WaitSet::new();
        wait_set
            .attach_condition(Condition::StatusCondition(cond))
            .unwrap();
        wait_set.wait(Duration::new(10, 0)).unwrap();
    }

    let instance_colors = ["BLUE", "BLUE1", "BLUE2", "BLUE3"];
    let coherent_sample_count = 3;

    publisher.begin_coherent_changes().unwrap();

    for n in 1..=coherent_sample_count {
        for writer in &writers {
            for &color in &instance_colors {
                let data = ShapeType {
                    color: color.to_string(),
                    x: 100,
                    y: 100,
                    shapesize: n,
                };
                writer.write(data, None).unwrap();
            }
        }
    }

    for writer in &writers {
        writer
            .wait_for_acknowledgments(Duration::new(10, 0))
            .unwrap();
    }

    for reader in &readers {
        assert!(
            reader
                .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .is_err()
        );
    }

    publisher.end_coherent_changes().unwrap();

    for writer in &writers {
        writer
            .wait_for_acknowledgments(Duration::new(10, 0))
            .unwrap();
    }

    for reader in &readers {
        let samples = reader
            .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(
            samples.len(),
            instance_colors.len() * coherent_sample_count as usize
        );

        for &color in &instance_colors {
            let instance_samples: Vec<_> = samples
                .iter()
                .filter(|s| s.data.as_ref().unwrap().color == color)
                .collect();
            assert_eq!(instance_samples.len(), coherent_sample_count as usize);
            let sizes: Vec<i32> = instance_samples
                .iter()
                .map(|s| s.data.as_ref().unwrap().shapesize)
                .collect();
            assert_eq!(sizes, vec![1, 2, 3]);
        }
    }
}

// Test_CoherentSets_12: Test the behavior of coherent access with GROUP_PRESENTATION with several instances
#[test]
fn coherent_sets_group_presentation_with_multiple_instances_and_topics() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic_names = ["Square", "Square1", "Square2"];
    let mut topics = Vec::new();
    for topic_name in topic_names {
        let topic = participant
            .create_topic::<ShapeType>(
                topic_name,
                "ShapeType",
                QosKind::Default,
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        topics.push(topic);
    }

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Group,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let mut writers = Vec::new();
    for topic in &topics {
        let writer = publisher
            .create_datawriter(
                topic,
                QosKind::Specific(writer_qos.clone()),
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        writers.push(writer);
    }

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Group,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let mut readers = Vec::new();
    for topic in &topics {
        let reader = subscriber
            .create_datareader::<ShapeType>(
                topic,
                QosKind::Specific(reader_qos.clone()),
                NO_LISTENER,
                NO_STATUS,
            )
            .unwrap();
        readers.push(reader);
    }

    for writer in &writers {
        let cond = writer.get_statuscondition();
        cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
            .unwrap();

        let mut wait_set = WaitSet::new();
        wait_set
            .attach_condition(Condition::StatusCondition(cond))
            .unwrap();
        wait_set.wait(Duration::new(10, 0)).unwrap();
    }

    let instance_colors = ["BLUE", "BLUE1", "BLUE2", "BLUE3"];
    let coherent_sample_count = 3;

    // Verify calling sample-accessing operations without begin_access returns PreconditionNotMet
    for reader in &readers {
        assert!(matches!(
            reader.take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE),
            Err(DdsError::PreconditionNotMet(_))
        ));
    }

    publisher.begin_coherent_changes().unwrap();

    for n in 1..=coherent_sample_count {
        for writer in &writers {
            for &color in &instance_colors {
                let data = ShapeType {
                    color: color.to_string(),
                    x: 100,
                    y: 100,
                    shapesize: n,
                };
                writer.write(data, None).unwrap();
            }
        }
    }

    for writer in &writers {
        writer
            .wait_for_acknowledgments(Duration::new(10, 0))
            .unwrap();
    }

    // Inside begin_access but before coherent set ends, no samples should be available
    subscriber.begin_access().unwrap();
    for reader in &readers {
        assert!(
            reader
                .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .is_err()
        );
    }
    subscriber.end_access().unwrap();

    publisher.end_coherent_changes().unwrap();

    for writer in &writers {
        writer
            .wait_for_acknowledgments(Duration::new(10, 0))
            .unwrap();
    }

    // After coherent set ends, taking without begin_access still fails with PreconditionNotMet
    for reader in &readers {
        assert!(matches!(
            reader.take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE),
            Err(DdsError::PreconditionNotMet(_))
        ));
    }

    subscriber.begin_access().unwrap();

    let mut total_samples = 0;
    for reader in &readers {
        let samples = reader
            .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        total_samples += samples.len();
        assert_eq!(
            samples.len(),
            instance_colors.len() * coherent_sample_count as usize
        );

        for &color in &instance_colors {
            let instance_samples: Vec<_> = samples
                .iter()
                .filter(|s| s.data.as_ref().unwrap().color == color)
                .collect();
            assert_eq!(instance_samples.len(), coherent_sample_count as usize);
            let sizes: Vec<i32> = instance_samples
                .iter()
                .map(|s| s.data.as_ref().unwrap().shapesize)
                .collect();
            assert_eq!(sizes, vec![1, 2, 3]);
        }
    }
    assert_eq!(total_samples, 36);

    subscriber.end_access().unwrap();

    // Calling end_access again without matching begin_access returns PreconditionNotMet
    assert!(matches!(
        subscriber.end_access(),
        Err(DdsError::PreconditionNotMet(_))
    ));
}

#[test]
fn coherent_sets_topic_presentation() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "CoherentTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<ShapeType>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    publisher.begin_coherent_changes().unwrap();

    for i in 1..=3 {
        let data = ShapeType {
            color: "RED".to_string(),
            x: 10,
            y: 20,
            shapesize: i,
        };
        writer.write(data, None).unwrap();
    }

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert!(
        reader
            .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .is_err()
    );

    publisher.end_coherent_changes().unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(50, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 3);
}

#[test]
fn coherent_changes_topic_access_scope() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<CoherentData>(
            "CoherentTopic",
            "CoherentData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<CoherentData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    // 1. Begin coherent set and write 2 samples
    publisher.begin_coherent_changes().unwrap();
    writer
        .write(CoherentData { id: 1, value: 10 }, None)
        .unwrap();
    writer
        .write(CoherentData { id: 1, value: 20 }, None)
        .unwrap();

    // Reader should not see data before coherent set ends
    let data_cond = reader.get_statuscondition();
    data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut data_wait_set = WaitSet::new();
    data_wait_set
        .attach_condition(Condition::StatusCondition(data_cond.clone()))
        .unwrap();
    assert!(data_wait_set.wait(Duration::new(0, 200_000_000)).is_err());

    // 2. End coherent changes -> triggers DataAvailable
    publisher.end_coherent_changes().unwrap();
    data_wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 10);
    assert_eq!(samples[1].data.as_ref().unwrap().value, 20);

    // 3. Subsequent sample outside coherent set is received immediately
    writer
        .write(CoherentData { id: 1, value: 30 }, None)
        .unwrap();
    data_wait_set.wait(Duration::new(5, 0)).unwrap();
    let samples = reader
        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 30);
}

#[test]
fn coherent_changes_instance_access_scope() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<CoherentData>(
            "CoherentInstanceTopic",
            "CoherentData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(publisher_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(subscriber_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<CoherentData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    publisher.begin_coherent_changes().unwrap();
    writer
        .write(CoherentData { id: 1, value: 100 }, None)
        .unwrap();
    writer
        .write(CoherentData { id: 1, value: 200 }, None)
        .unwrap();

    let data_cond = reader.get_statuscondition();
    data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut data_wait_set = WaitSet::new();
    data_wait_set
        .attach_condition(Condition::StatusCondition(data_cond))
        .unwrap();
    assert!(data_wait_set.wait(Duration::new(0, 200_000_000)).is_err());

    publisher.end_coherent_changes().unwrap();
    data_wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 100);
    assert_eq!(samples[1].data.as_ref().unwrap().value, 200);
}

#[test]
fn multiple_writers_coherent_and_non_coherent() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<CoherentData>(
            "CoherentMultiWriterTopic",
            "CoherentData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let pub_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };

    // Publisher 1: coherent access enabled
    let pub1 = participant
        .create_publisher(QosKind::Specific(pub_qos.clone()), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer1 = pub1
        .create_datawriter(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    // Publisher 2: also coherent access enabled
    let pub2 = participant
        .create_publisher(QosKind::Specific(pub_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer2 = pub2
        .create_datawriter(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    // Subscriber: coherent access
    let sub_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: true,
            ordered_access: false,
        },
        ..Default::default()
    };
    let subscriber = participant
        .create_subscriber(QosKind::Specific(sub_qos), NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<CoherentData>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    // Wait for matching
    let cond1 = writer1.get_statuscondition();
    cond1
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut ws1 = WaitSet::new();
    ws1.attach_condition(Condition::StatusCondition(cond1))
        .unwrap();
    ws1.wait(Duration::new(10, 0)).unwrap();

    let cond2 = writer2.get_statuscondition();
    cond2
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut ws2 = WaitSet::new();
    ws2.attach_condition(Condition::StatusCondition(cond2))
        .unwrap();
    ws2.wait(Duration::new(10, 0)).unwrap();

    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut rws = WaitSet::new();
    rws.attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    while reader
        .get_subscription_matched_status()
        .unwrap()
        .current_count
        < 2
    {
        rws.wait(Duration::new(10, 0)).unwrap();
    }

    let data_cond = reader.get_statuscondition();
    data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut dws = WaitSet::new();
    dws.attach_condition(Condition::StatusCondition(data_cond))
        .unwrap();

    // Writer 1 begins coherent set and writes
    pub1.begin_coherent_changes().unwrap();
    writer1
        .write(CoherentData { id: 1, value: 1 }, None)
        .unwrap();

    // Writer 2 writes a non-coherent sample
    writer2
        .write(CoherentData { id: 1, value: 99 }, None)
        .unwrap();

    // Reader should receive Writer 2's sample even though Writer 1 is in an open coherent set
    dws.wait(Duration::new(5, 0)).unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 99);

    // Writer 1 ends coherent changes
    pub1.end_coherent_changes().unwrap();
    dws.wait(Duration::new(5, 0)).unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap().value, 1);
}

#[test]
fn ordered_access_instance_and_topic_presentation() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<ShapeType>(
            "Square",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let pub_qos = PublisherQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: false,
            ordered_access: true,
        },
        ..Default::default()
    };
    let publisher = participant
        .create_publisher(QosKind::Specific(pub_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Subscriber 1 with INSTANCE presentation
    let sub1_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Instance,
            coherent_access: false,
            ordered_access: true,
        },
        ..Default::default()
    };
    let subscriber1 = participant
        .create_subscriber(QosKind::Specific(sub1_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader_qos = DataReaderQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let reader1 = subscriber1
        .create_datareader::<ShapeType>(
            &topic,
            QosKind::Specific(reader_qos.clone()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Subscriber 2 with TOPIC presentation
    let sub2_qos = SubscriberQos {
        presentation: PresentationQosPolicy {
            access_scope: PresentationQosPolicyAccessScopeKind::Topic,
            coherent_access: false,
            ordered_access: true,
        },
        ..Default::default()
    };
    let subscriber2 = participant
        .create_subscriber(QosKind::Specific(sub2_qos), NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader2 = subscriber2
        .create_datareader::<ShapeType>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Wait for match
    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut ws = WaitSet::new();
    ws.attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();
    while writer
        .get_publication_matched_status()
        .unwrap()
        .current_count
        < 2
    {
        ws.wait(Duration::new(10, 0)).unwrap();
    }

    let r1_cond = reader1.get_statuscondition();
    r1_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut r1_ws = WaitSet::new();
    r1_ws
        .attach_condition(Condition::StatusCondition(r1_cond))
        .unwrap();
    while reader1
        .get_subscription_matched_status()
        .unwrap()
        .current_count
        < 1
    {
        r1_ws.wait(Duration::new(10, 0)).unwrap();
    }

    let r2_cond = reader2.get_statuscondition();
    r2_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut r2_ws = WaitSet::new();
    r2_ws
        .attach_condition(Condition::StatusCondition(r2_cond))
        .unwrap();
    while reader2
        .get_subscription_matched_status()
        .unwrap()
        .current_count
        < 1
    {
        r2_ws.wait(Duration::new(10, 0)).unwrap();
    }

    // Publish 4 instances interleaved
    let instances = ["BLUE", "BLUE1", "BLUE2", "BLUE3"];
    let num_samples_per_instance = 5;
    for size in 0..num_samples_per_instance {
        for color in &instances {
            writer
                .write(
                    ShapeType {
                        color: color.to_string(),
                        x: 10,
                        y: 10,
                        shapesize: size,
                    },
                    None,
                )
                .unwrap();
        }
    }

    let total_samples = instances.len() * num_samples_per_instance as usize;

    // Wait until both readers have all samples
    let r1_data_cond = reader1.get_statuscondition();
    r1_data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut r1_dws = WaitSet::new();
    r1_dws
        .attach_condition(Condition::StatusCondition(r1_data_cond))
        .unwrap();
    while match reader1.read(100, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE) {
        Ok(s) => s.len() < total_samples,
        Err(DdsError::NoData) => true,
        Err(e) => panic!("{e:?}"),
    } {
        r1_dws.wait(Duration::new(10, 0)).unwrap();
    }

    let r2_data_cond = reader2.get_statuscondition();
    r2_data_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut r2_dws = WaitSet::new();
    r2_dws
        .attach_condition(Condition::StatusCondition(r2_data_cond))
        .unwrap();
    while match reader2.read(100, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE) {
        Ok(s) => s.len() < total_samples,
        Err(DdsError::NoData) => true,
        Err(e) => panic!("{e:?}"),
    } {
        r2_dws.wait(Duration::new(10, 0)).unwrap();
    }

    // Subscriber 1 (Instance Presentation): samples must be grouped contiguously by instance
    let sub1_samples = reader1
        .take(100, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(sub1_samples.len(), total_samples);

    // Grouping check: for each instance, all its samples must appear contiguously
    let mut seen_instances = std::collections::HashSet::new();
    let mut current_instance = String::new();
    for sample in &sub1_samples {
        let sample_data = sample.data.as_ref().unwrap();
        if sample_data.color != current_instance {
            assert!(
                !seen_instances.contains(&sample_data.color),
                "Instance {} appeared again after another instance; samples are not grouped by instance",
                sample_data.color
            );
            seen_instances.insert(sample_data.color.clone());
            current_instance = sample_data.color.clone();
        }
    }

    // Subscriber 2 (Topic Presentation): samples must be ordered in publish order (interleaved)
    let sub2_samples = reader2
        .take(100, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();
    assert_eq!(sub2_samples.len(), total_samples);

    let mut expected_index = 0;
    for size in 0..num_samples_per_instance {
        for color in &instances {
            let data = sub2_samples[expected_index].data.as_ref().unwrap();
            assert_eq!(data.color, *color);
            assert_eq!(data.shapesize, size);
            expected_index += 1;
        }
    }
}
