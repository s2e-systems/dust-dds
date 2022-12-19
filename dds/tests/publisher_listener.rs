use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusKind, NO_STATUS},
        time::Duration,
        wait_set::{Condition, WaitSet},
    },
    publication::{data_writer::AnyDataWriter, publisher_listener::PublisherListener},
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
fn publication_matched_listener() {
    mock! {
        PublicationMatchedListener{}

        impl PublisherListener for PublicationMatchedListener {
            fn on_publication_matched(
                &mut self,
                _the_reader: &dyn AnyDataWriter,
                _status: PublicationMatchedStatus,
            );
        }
    }

    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: Duration::new(1, 0),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let mut publisher_listener = MockPublicationMatchedListener::new();
    publisher_listener
        .expect_on_publication_matched()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());
    let publisher = participant
        .create_publisher(
            QosKind::Default,
            Some(Box::new(publisher_listener)),
            &[StatusKind::PublicationMatched],
        )
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: Duration::new(1, 0),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();
}

#[test]
fn offered_incompatible_qos_listener() {
    mock! {
        OfferedIncompatibleQosListener{}

        impl PublisherListener for OfferedIncompatibleQosListener {
            fn on_offered_incompatible_qos(
                &mut self,
                _the_reader: &dyn AnyDataWriter,
                _status: OfferedIncompatibleQosStatus,
            );
        }
    }

    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            QosKind::Default,
            None,
            NO_STATUS,
        )
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
            ..Default::default()
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let mut publisher_listener = MockOfferedIncompatibleQosListener::new();
    publisher_listener
        .expect_on_offered_incompatible_qos()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());
    let publisher = participant
        .create_publisher(
            QosKind::Default,
            Some(Box::new(publisher_listener)),
            &[StatusKind::OfferedIncompatibleQos],
        )
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: Duration::new(1, 0),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::OfferedIncompatibleQos])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();
}
