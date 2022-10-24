use dust_dds::{
    dds_type::{DdsSerde, DdsType, Endianness},
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        entity::Entity,
        error::DdsResult,
        qos::{DataReaderQos, DataWriterQos},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::StatusKind,
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    subscription::sample_info::{
        InstanceStateKind, ViewStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct KeyedData {
    id: u8,
    value: u8,
}

impl DdsType for KeyedData {
    fn type_name() -> &'static str {
        "KeyedData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
        vec![self.id]
    }

    fn set_key_fields_from_serialized_key(&mut self, key: &[u8]) -> DdsResult<()> {
        self.id = key[0];
        Ok(())
    }
}

impl DdsSerde for KeyedData {}

#[test]
fn each_key_sample_is_read() {
    let domain_id = 20;

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, None, None, &[])
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", None, None, &[])
        .unwrap();

    let publisher = participant.create_publisher(None, None, &[]).unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, Some(writer_qos), None, &[])
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

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatchedStatus])
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
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into()
    );

    assert_eq!(samples[1].data.as_ref().unwrap(), &data2);
    assert_eq!(
        samples[1].sample_info.instance_handle,
        [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into()
    );

    assert_eq!(samples[2].data.as_ref().unwrap(), &data3);
    assert_eq!(
        samples[2].sample_info.instance_handle,
        [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into()
    );
}

#[test]
fn write_read_disposed_samples() {
    let domain_id = 21;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, &[])
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", None, None, &[])
        .unwrap();

    let publisher = participant.create_publisher(None, None, &[]).unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, Some(writer_qos), None, &[])
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

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatchedStatus])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(5, 0)).unwrap();

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
        InstanceStateKind::Alive
    );
    assert_eq!(
        samples[1].sample_info.instance_state,
        InstanceStateKind::NotAliveDisposed
    );
}

#[test]
fn write_read_sample_view_state() {
    let domain_id = 22;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, &[])
        .unwrap();

    let topic = participant
        .create_topic::<KeyedData>("MyTopic", None, None, &[])
        .unwrap();

    let publisher = participant.create_publisher(None, None, &[]).unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, Some(writer_qos), None, &[])
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

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatchedStatus])
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
