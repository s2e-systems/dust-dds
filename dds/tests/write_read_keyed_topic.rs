use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsError,
        qos::{DataReaderQos, DataWriterQos, QosKind, TopicQos},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, TimeBasedFilterQosPolicy,
        },
        status::{StatusKind, NO_STATUS},
        time::{Duration, Time},
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{
        InstanceStateKind, SampleStateKind, ViewStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE,
        ANY_VIEW_STATE,
    },
    topic_definition::type_support::{DdsSerde, DdsType},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType, DdsSerde)]
struct KeyedData {
    #[key]
    id: u8,
    value: u8,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType, DdsSerde)]
struct LargeData {
    #[key]
    id: u8,
    value: Vec<u8>,
}

#[test]
fn large_data_should_be_fragmented() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<LargeData>("LargeDataTopic", QosKind::Default, None, NO_STATUS)
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data = LargeData {
        id: 1,
        value: vec![8; 100000],
    };

    writer.write(&data, None).unwrap();

    let cond = reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(5, 0)).unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data);
}

#[test]
fn large_data_should_be_fragmented_reliable() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<LargeData>("LargeDataTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data = LargeData {
        id: 1,
        value: vec![8; 100000],
    };

    writer.write(&data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(5, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data);
}

#[test]
fn samples_are_taken() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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
    let data4 = KeyedData { id: 4, value: 30 };
    let data5 = KeyedData { id: 5, value: 40 };

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();
    writer.write(&data4, None).unwrap();
    writer.write(&data5, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
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
fn read_only_unread_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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
    let data4 = KeyedData { id: 4, value: 30 };
    let data5 = KeyedData { id: 5, value: 40 };

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();
    writer.write(&data4, None).unwrap();
    writer.write(&data5, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
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

    assert_eq!(samples3, Err(DdsError::NoData));
}

#[test]
fn read_next_sample() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    assert_eq!(reader.read_next_sample().unwrap().data.unwrap(), data1);
    assert_eq!(reader.read_next_sample().unwrap().data.unwrap(), data2);
    assert_eq!(reader.read_next_sample().unwrap().data.unwrap(), data3);
    assert_eq!(reader.read_next_sample(), Err(DdsError::NoData));
}

#[test]
fn take_next_sample() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    assert_eq!(reader.take_next_sample().unwrap().data.unwrap(), data1);
    assert_eq!(reader.take_next_sample().unwrap().data.unwrap(), data2);
    assert_eq!(reader.take_next_sample().unwrap().data.unwrap(), data3);
    assert_eq!(reader.take_next_sample(), Err(DdsError::NoData));
}

#[test]
fn each_key_sample_is_read() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(
        samples[0].sample_info.instance_handle,
        data1.get_serialized_key().into()
    );

    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
    assert_eq!(
        samples[1].sample_info.instance_handle,
        data2.get_serialized_key().into()
    );

    assert_eq!(samples[2].data.as_ref().unwrap(), &data3);
    assert_eq!(
        samples[2].sample_info.instance_handle,
        data3.get_serialized_key().into()
    );
}

#[test]
fn read_specific_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .read_instance(
            3,
            data1.get_serialized_key().into(),
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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
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
    assert_eq!(samples4, Err(DdsError::NoData));
}

#[test]
fn take_next_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
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
    assert_eq!(samples4, Err(DdsError::NoData));
}

#[test]
fn take_specific_instance() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .take_instance(
            3,
            data1.get_serialized_key().into(),
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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
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

    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();
    writer.write(&data3, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    assert_eq!(
        reader.take_instance(
            3,
            KeyedData { id: 99, value: 20 }.get_serialized_key().into(),
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ),
        Err(DdsError::BadParameter)
    );
}

#[test]
fn write_read_disposed_samples() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            depth: 1,
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
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            depth: 1,
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };

    writer.write(&data1, None).unwrap();
    writer.dispose(&data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
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
fn write_read_sample_view_state() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("OtherTopic", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1 = KeyedData { id: 1, value: 1 };

    writer.write(&data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let data1_2 = KeyedData { id: 1, value: 2 };
    let data2 = KeyedData { id: 2, value: 1 };

    writer.write(&data1_2, None).unwrap();
    writer.write(&data2, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .read(2, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].sample_info.view_state, ViewStateKind::NotNew);
    assert_eq!(samples[1].sample_info.view_state, ViewStateKind::New);
}

#[test]
fn inconsistent_topic_status_condition() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let best_effort_topic_qos = TopicQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let topic_best_effort = participant
        .create_topic::<KeyedData>(
            "Topic",
            QosKind::Specific(best_effort_topic_qos),
            None,
            NO_STATUS,
        )
        .unwrap();

    let status_cond = topic_best_effort.get_statuscondition().unwrap();
    status_cond
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(status_cond))
        .unwrap();

    let reliable_topic_qos = TopicQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let _topic_reliable = participant
        .create_topic::<KeyedData>(
            "Topic",
            QosKind::Specific(reliable_topic_qos),
            None,
            NO_STATUS,
        )
        .unwrap();

    wait_set.wait(Duration::new(2, 0)).unwrap();

    assert_eq!(
        topic_best_effort
            .get_inconsistent_topic_status()
            .unwrap()
            .total_count,
        1
    );
}

#[test]
fn reader_with_minimum_time_separation_qos() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            depth: 1,
        },
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
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            depth: 1,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        time_based_filter: TimeBasedFilterQosPolicy {
            minimum_separation: Duration::new(2, 0),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    let data1_1 = KeyedData { id: 1, value: 1 };
    let data1_2 = KeyedData { id: 1, value: 2 };
    let data1_3 = KeyedData { id: 1, value: 3 };
    let data2_1 = KeyedData { id: 2, value: 10 };
    let data2_2 = KeyedData { id: 2, value: 20 };
    let data2_3 = KeyedData { id: 2, value: 30 };

    writer
        .write_w_timestamp(&data1_1, None, Time::new(1, 0))
        .unwrap();
    writer
        .write_w_timestamp(&data1_2, None, Time::new(2, 0))
        .unwrap();
    writer
        .write_w_timestamp(&data1_3, None, Time::new(3, 0))
        .unwrap();
    writer
        .write_w_timestamp(&data2_1, None, Time::new(4, 0))
        .unwrap();
    writer
        .write_w_timestamp(&data2_2, None, Time::new(5, 0))
        .unwrap();
    writer
        .write_w_timestamp(&data2_3, None, Time::new(6, 0))
        .unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
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
fn transient_local_writer_reader_wait_for_historical_data() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            depth: 1,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();
    let data1 = KeyedData { id: 1, value: 1 };
    let data2 = KeyedData { id: 2, value: 2 };
    writer.write(&data1, None).unwrap();
    writer.write(&data2, None).unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            depth: 1,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: Duration::new(1, 0),
        },
        time_based_filter: TimeBasedFilterQosPolicy {
            minimum_separation: Duration::new(2, 0),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = reader.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

    reader
        .wait_for_historical_data(Duration::new(2, 0))
        .unwrap();
    let samples = reader
        .read(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data.as_ref().unwrap(), &data1);
    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
}
