use dust_dds::dcps_psm::{Time, LENGTH_UNLIMITED};
use dust_dds::dds_type::{DdsDeserialize, DdsSerialize, DdsType};
use dust_dds::infrastructure::entity::Entity;
use dust_dds::infrastructure::qos::{DataReaderQos, DataWriterQos};
use dust_dds::infrastructure::qos_policy::{
    DestinationOrderQosPolicy, DestinationOrderQosPolicyKind, HistoryQosPolicy,
    HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind, ResourceLimitsQosPolicy,
};

use dust_dds::infrastructure::wait_set::{Condition, WaitSet};
use dust_dds::return_type::{DdsError, DdsResult};

use dust_dds::{
    dcps_psm::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    domain::domain_participant_factory::DomainParticipantFactory,
};

#[derive(Debug, PartialEq)]
struct UserData(u8);

impl DdsType for UserData {
    fn type_name() -> &'static str {
        "UserData"
    }

    fn has_key() -> bool {
        false
    }
}

impl DdsSerialize for UserData {
    fn serialize<W: std::io::Write, E: dust_dds::dds_type::Endianness>(
        &self,
        mut writer: W,
    ) -> DdsResult<()> {
        writer
            .write(&[self.0])
            .map(|_| ())
            .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
    }
}

impl<'de> DdsDeserialize<'de> for UserData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        Ok(UserData(buf[0]))
    }
}

#[test]
fn write_read_unkeyed_topic() {
    let domain_id = 8;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let topic = participant
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, Some(writer_qos), None, 0)
        .unwrap();

    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic, Some(reader_qos), None, 0)
        .unwrap();

    let mut cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(dust_dds::dcps_psm::SUBSCRIPTION_MATCHED_STATUS)
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set
        .wait(dust_dds::dcps_psm::Duration::new(5, 0))
        .unwrap();

    writer.write(&UserData(8), None).unwrap();

    writer
        .wait_for_acknowledgments(dust_dds::dcps_psm::Duration::new(1, 0))
        .unwrap();

    let samples = reader.read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE);

    assert_eq!(samples.unwrap()[0].data.as_ref().unwrap(), &UserData(8));
}

#[test]
fn data_reader_resource_limits() {
    let domain_id = 10;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();
    let topic1 = participant1
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();
    let topic2 = participant2
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAllHistoryQos,
            depth: LENGTH_UNLIMITED,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic1, Some(data_writer_qos), None, 0)
        .unwrap();

    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAllHistoryQos,
            depth: LENGTH_UNLIMITED,
        },
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: 2,
            max_instances: LENGTH_UNLIMITED,
            max_samples_per_instance: LENGTH_UNLIMITED,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic2, Some(reader_qos), None, 0)
        .unwrap();

    let mut cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(dust_dds::dcps_psm::SUBSCRIPTION_MATCHED_STATUS)
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set
        .wait(dust_dds::dcps_psm::Duration::new(5, 0))
        .unwrap();

    writer.write(&UserData(1), None).unwrap();
    writer.write(&UserData(2), None).unwrap();
    writer.write(&UserData(3), None).unwrap();

    writer
        .wait_for_acknowledgments(dust_dds::dcps_psm::Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
}

#[test]
fn data_reader_order_by_source_timestamp() {
    let domain_id = 11;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();
    let topic = participant
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        destination_order: DestinationOrderQosPolicy {
            kind: DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAllHistoryQos,
            depth: LENGTH_UNLIMITED,
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, Some(data_writer_qos), None, 0)
        .unwrap();

    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAllHistoryQos,
            depth: 1,
        },
        destination_order: DestinationOrderQosPolicy {
            kind: DestinationOrderQosPolicyKind::BySourceTimestampDestinationOrderQoS,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic, Some(reader_qos), None, 0)
        .unwrap();

    let mut cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(dust_dds::dcps_psm::SUBSCRIPTION_MATCHED_STATUS)
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set
        .wait(dust_dds::dcps_psm::Duration::new(5, 0))
        .unwrap();

    writer
        .write_w_timestamp(
            &UserData(1),
            None,
            Time {
                sec: 30,
                nanosec: 0,
            },
        )
        .unwrap();
    writer
        .write_w_timestamp(
            &UserData(2),
            None,
            Time {
                sec: 20,
                nanosec: 0,
            },
        )
        .unwrap();
    writer
        .write_w_timestamp(
            &UserData(3),
            None,
            Time {
                sec: 10,
                nanosec: 0,
            },
        )
        .unwrap();

    writer
        .wait_for_acknowledgments(dust_dds::dcps_psm::Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .read(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 3);
    assert_eq!(&samples[0].data, &Some(UserData(3)));
    assert_eq!(&samples[1].data, &Some(UserData(2)));
    assert_eq!(&samples[2].data, &Some(UserData(1)));
}

#[test]
fn data_reader_publication_handle_sample_info() {
    let domain_id = 12;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let topic = participant
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, Some(writer_qos), None, 0)
        .unwrap();

    let subscriber = participant.create_subscriber(None, None, 0).unwrap();

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
            max_blocking_time: dust_dds::dcps_psm::Duration::new(1, 0),
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader(&topic, Some(reader_qos), None, 0)
        .unwrap();

    let mut cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(dust_dds::dcps_psm::SUBSCRIPTION_MATCHED_STATUS)
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set
        .wait(dust_dds::dcps_psm::Duration::new(5, 0))
        .unwrap();

    writer.write(&UserData(1), None).unwrap();

    writer
        .wait_for_acknowledgments(dust_dds::dcps_psm::Duration::new(1, 0))
        .unwrap();

    let samples = reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert!(reader
        .get_matched_publication_data(samples[0].sample_info.publication_handle)
        .is_ok());
}
