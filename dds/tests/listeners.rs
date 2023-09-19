use dust_dds::{
    domain::{
        domain_participant_factory::{DomainParticipantFactory, THE_PARTICIPANT_FACTORY},
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DeadlineQosPolicy, HistoryQosPolicy, HistoryQosPolicyKind, Length,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, ResourceLimitsQosPolicy,
        },
        status::{
            OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleRejectedStatus, SampleRejectedStatusKind,
            StatusKind, SubscriptionMatchedStatus, NO_STATUS,
        },
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    publication::{
        data_writer::{AnyDataWriter, DataWriter},
        data_writer_listener::DataWriterListener,
        publisher_listener::PublisherListener,
    },
    subscription::{
        data_reader::{AnyDataReader, DataReader},
        data_reader_listener::DataReaderListener,
        subscriber::Subscriber,
        subscriber_listener::SubscriberListener,
    },
    topic_definition::type_support::DdsType,
};

use mockall::mock;

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, DdsType)]
struct MyData {
    #[key]
    id: u8,
    value: u8,
}

#[test]
fn deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<RequestedDeadlineMissedStatus>,
    }

    impl DomainParticipantListener for DeadlineMissedListener {
        fn on_requested_deadline_missed(
            &mut self,
            _the_reader: &dyn AnyDataReader,
            status: RequestedDeadlineMissedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);

    let participant_listener = DeadlineMissedListener { sender };

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Box::new(participant_listener)),
            &[StatusKind::RequestedDeadlineMissed],
        )
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = MyData { id: 1, value: 1 };
    writer.write(&data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn sample_rejected_listener() {
    struct SampleRejectedListener {
        sender: std::sync::mpsc::SyncSender<SampleRejectedStatus>,
    }

    impl DomainParticipantListener for SampleRejectedListener {
        fn on_sample_rejected(
            &mut self,
            _the_reader: &dyn AnyDataReader,
            status: SampleRejectedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let participant_listener = SampleRejectedListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Box::new(participant_listener)),
            &[StatusKind::SampleRejected],
        )
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: Length::Limited(2),
            max_instances: Length::Unlimited,
            max_samples_per_instance: Length::Limited(2),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    writer.write(&MyData { id: 1, value: 0 }, None).unwrap();
    writer.write(&MyData { id: 1, value: 1 }, None).unwrap();
    writer.write(&MyData { id: 1, value: 2 }, None).unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
    assert_eq!(
        status.last_reason,
        SampleRejectedStatusKind::RejectedBySamplesLimit
    );
}

#[test]
fn subscription_matched_listener() {
    struct SubscriptionMatchedListener {
        sender: std::sync::mpsc::SyncSender<SubscriptionMatchedStatus>,
    }

    impl DomainParticipantListener for SubscriptionMatchedListener {
        fn on_subscription_matched(
            &mut self,
            _the_reader: &dyn AnyDataReader,
            status: SubscriptionMatchedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let participant_listener = SubscriptionMatchedListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Box::new(participant_listener)),
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn requested_incompatible_qos_listener() {
    struct RequestedIncompatibleQosListener {
        sender: std::sync::mpsc::SyncSender<RequestedIncompatibleQosStatus>,
    }

    impl DomainParticipantListener for RequestedIncompatibleQosListener {
        fn on_requested_incompatible_qos(
            &mut self,
            _the_reader: &dyn AnyDataReader,
            status: RequestedIncompatibleQosStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let participant_listener = RequestedIncompatibleQosListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Box::new(participant_listener)),
            &[StatusKind::RequestedIncompatibleQos],
        )
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn publication_matched_listener() {
    struct PublicationMatchedListener {
        sender: std::sync::mpsc::SyncSender<PublicationMatchedStatus>,
    }

    impl DomainParticipantListener for PublicationMatchedListener {
        fn on_publication_matched(
            &mut self,
            _the_reader: &dyn AnyDataWriter,
            status: PublicationMatchedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let participant_listener = PublicationMatchedListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Box::new(participant_listener)),
            &[StatusKind::PublicationMatched],
        )
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };

    let _writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn offered_incompatible_qos_listener() {
    struct OfferedIncompatibleQosListener {
        sender: std::sync::mpsc::SyncSender<OfferedIncompatibleQosStatus>,
    }

    impl DomainParticipantListener for OfferedIncompatibleQosListener {
        fn on_offered_incompatible_qos(
            &mut self,
            _the_reader: &dyn AnyDataWriter,
            status: OfferedIncompatibleQosStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let participant_listener = OfferedIncompatibleQosListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Box::new(participant_listener)),
            &[StatusKind::OfferedIncompatibleQos],
        )
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };

    let _writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn on_data_available_listener() {
    struct DataAvailableListener {
        sender: std::sync::mpsc::SyncSender<()>,
    }

    impl DataReaderListener<MyData> for DataAvailableListener {
        fn on_data_available(&mut self, _the_reader: &DataReader<MyData>) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let reader_listener = DataAvailableListener { sender };

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
    wait_set.wait(Duration::new(10, 0)).unwrap();

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

    assert!(receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .is_ok());
}

#[test]
fn data_on_readers_listener() {
    struct DataOnReadersListener {
        sender: std::sync::mpsc::SyncSender<()>,
    }

    impl SubscriberListener for DataOnReadersListener {
        fn on_data_on_readers(&mut self, _the_subscriber: &Subscriber) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = DataOnReadersListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(Box::new(subscriber_listener)),
            &[StatusKind::DataOnReaders],
        )
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = MyData { id: 1, value: 1 };
    writer.write(&data1, None).unwrap();

    assert!(receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .is_ok());
}

#[test]
fn data_available_listener_not_called_when_data_on_readers_listener() {
    struct DataOnReadersListener {
        sender: std::sync::mpsc::SyncSender<()>,
    }

    impl SubscriberListener for DataOnReadersListener {
        fn on_data_on_readers(&mut self, _the_subscriber: &Subscriber) {
            self.sender.send(()).unwrap();
        }
    }

    struct DataAvailableListener {
        sender: std::sync::mpsc::SyncSender<()>,
    }

    impl DataReaderListener<MyData> for DataAvailableListener {
        fn on_data_available(&mut self, _the_reader: &DataReader<MyData>) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();

    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    let (sender, data_on_readers_receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = DataOnReadersListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(Box::new(subscriber_listener)),
            &[StatusKind::DataOnReaders],
        )
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let (sender, data_available_receiver) = std::sync::mpsc::sync_channel(1);
    let reader_listener = DataAvailableListener { sender };

    let _reader = subscriber
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
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = MyData { id: 1, value: 1 };
    writer.write(&data1, None).unwrap();

    assert!(data_on_readers_receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .is_ok());
    assert!(data_available_receiver.try_recv().is_err());
}

#[test]
fn participant_deadline_missed_listener() {
    mock! {
        DeadlineMissedListener{}

        impl DataReaderListener<MyData> for DeadlineMissedListener {
            fn on_requested_deadline_missed(
                &mut self,
                _the_reader: &DataReader<MyData>,
                _status: RequestedDeadlineMissedStatus,
            );
        }

    }
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let mut reader_listener = MockDeadlineMissedListener::new();
    reader_listener
        .expect_on_requested_deadline_missed()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());

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
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = MyData { id: 1, value: 1 };
    writer.write(&data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn participant_sample_rejected_listener() {
    mock! {
        SampleRejectedListener{}

        impl DataReaderListener<MyData> for SampleRejectedListener {
            fn on_sample_rejected(
                &mut self,
                _the_reader: &DataReader<MyData>,
                _status: SampleRejectedStatus,
            );
        }

    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
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

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: Length::Limited(2),
            max_instances: Length::Unlimited,
            max_samples_per_instance: Length::Limited(2),
        },
        ..Default::default()
    };
    let mut reader_listener = MockSampleRejectedListener::new();
    reader_listener
        .expect_on_sample_rejected()
        .times(1..)
        .withf(|_, status| {
            status.total_count >= 1 // This is not an equality because the listener might be called multiple times during testing
                && status.last_reason == SampleRejectedStatusKind::RejectedBySamplesLimit
        })
        .return_const(());

    let reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(Box::new(reader_listener)),
            &[StatusKind::SampleRejected],
        )
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    writer.write(&MyData { id: 1, value: 0 }, None).unwrap();
    writer.write(&MyData { id: 1, value: 1 }, None).unwrap();
    writer.write(&MyData { id: 1, value: 2 }, None).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn participant_subscription_matched_listener() {
    mock! {
        SubscriptionMatchedListener{}

        impl DataReaderListener<MyData> for SubscriptionMatchedListener {
            fn on_subscription_matched(
                &mut self,
                _the_reader: &DataReader<MyData>,
                _status: SubscriptionMatchedStatus,
            );
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };
    let mut reader_listener = MockSubscriptionMatchedListener::new();
    reader_listener
        .expect_on_subscription_matched()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());

    let reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(Box::new(reader_listener)),
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn participant_requested_incompatible_qos_listener() {
    mock! {
        RequestedIncompatibleQosListener{}

        impl DataReaderListener<MyData> for RequestedIncompatibleQosListener {
            fn on_requested_incompatible_qos(
                &mut self,
                _the_reader: &DataReader<MyData>,
                _status: RequestedIncompatibleQosStatus,
            );
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };
    let mut reader_listener = MockRequestedIncompatibleQosListener::new();
    reader_listener
        .expect_on_requested_incompatible_qos()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());

    let reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(Box::new(reader_listener)),
            &[StatusKind::RequestedIncompatibleQos],
        )
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn publisher_publication_matched_listener() {
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

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let mut publisher_listener = MockPublicationMatchedListener::new();
    publisher_listener
        .expect_on_publication_matched()
        .once()
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn publisher_offered_incompatible_qos_listener() {
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

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };

    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn subscriber_deadline_missed_listener() {
    mock! {
        DeadlineMissedListener{}

        impl SubscriberListener for DeadlineMissedListener {

            fn on_requested_deadline_missed(
                &mut self,
                _the_reader: &dyn AnyDataReader,
                _status: RequestedDeadlineMissedStatus,
            );
        }

    }
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic("MyTopic", "MyData", QosKind::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter(&topic, QosKind::Specific(writer_qos), None, NO_STATUS)
        .unwrap();

    let mut subscriber_listener = MockDeadlineMissedListener::new();
    subscriber_listener
        .expect_on_requested_deadline_missed()
        .times(1..)
        // .withf(|_, status| status.total_count >= 1)
        .return_const(());
    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(Box::new(subscriber_listener)),
            &[StatusKind::RequestedDeadlineMissed],
        )
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = MyData { id: 1, value: 1 };
    writer.write(&data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn subscriber_sample_rejected_listener() {
    mock! {
        SampleRejectedListener{}

        impl SubscriberListener for SampleRejectedListener {

            fn on_sample_rejected(
                &mut self,
                _the_reader: &dyn AnyDataReader,
                _status: SampleRejectedStatus,
            );
        }

    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
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

    let mut subscriber_listener = MockSampleRejectedListener::new();
    subscriber_listener
        .expect_on_sample_rejected()
        .times(1..)
        .withf(|_, status| {
            status.total_count >= 1 // This is not an equality because the listener might be called multiple times during testing
                && status.last_reason == SampleRejectedStatusKind::RejectedBySamplesLimit
        })
        .return_const(());
    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(Box::new(subscriber_listener)),
            &[StatusKind::SampleRejected],
        )
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        resource_limits: ResourceLimitsQosPolicy {
            max_samples: Length::Limited(2),
            max_instances: Length::Unlimited,
            max_samples_per_instance: Length::Limited(2),
        },
        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    writer.write(&MyData { id: 1, value: 0 }, None).unwrap();
    writer.write(&MyData { id: 1, value: 1 }, None).unwrap();
    writer.write(&MyData { id: 1, value: 2 }, None).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn subscriber_subscription_matched_listener() {
    mock! {
        SubscriptionMatchedListener{}

        impl SubscriberListener for SubscriptionMatchedListener {
            fn on_subscription_matched(
                &mut self,
                _the_reader: &dyn AnyDataReader,
                _status: SubscriptionMatchedStatus,
            );
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let mut subscriber_listener = MockSubscriptionMatchedListener::new();
    subscriber_listener
        .expect_on_subscription_matched()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());
    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(Box::new(subscriber_listener)),
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn subscriber_requested_incompatible_qos_listener() {
    mock! {
        RequestedIncompatibleQosListener{}

        impl SubscriberListener for RequestedIncompatibleQosListener {
            fn on_requested_incompatible_qos(
                &mut self,
                _the_reader: &dyn AnyDataReader,
                _status: RequestedIncompatibleQosStatus,
            );
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Specific(data_writer_qos), None, NO_STATUS)
        .unwrap();

    let mut subscriber_listener = MockRequestedIncompatibleQosListener::new();
    subscriber_listener
        .expect_on_requested_incompatible_qos()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());
    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(Box::new(subscriber_listener)),
            &[StatusKind::RequestedIncompatibleQos],
        )
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn data_writer_publication_matched_listener() {
    mock! {
        PublicationMatchedListener{}

        impl DataWriterListener<MyData> for PublicationMatchedListener {
            fn on_publication_matched(
                &mut self,
                _the_reader: &DataWriter<MyData>,
                _status: PublicationMatchedStatus,
            );
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut writer_listener = MockPublicationMatchedListener::new();
    writer_listener
        .expect_on_publication_matched()
        .once()
        .return_const(());
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            Some(Box::new(writer_listener)),
            &[StatusKind::PublicationMatched],
        )
        .unwrap();

    let cond = writer.get_statuscondition().unwrap();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}

#[test]
fn data_writer_offered_incompatible_qos_listener() {
    mock! {
        OfferedIncompatibleQosListener{}

        impl DataWriterListener<MyData> for OfferedIncompatibleQosListener {
            fn on_offered_incompatible_qos(
                &mut self,
                _the_reader: &DataWriter<MyData>,
                _status: OfferedIncompatibleQosStatus,
            );
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic(
            "SampleRejectedListenerTopic",
            "MyData",
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
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },

        ..Default::default()
    };

    let reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut writer_listener = MockOfferedIncompatibleQosListener::new();
    writer_listener
        .expect_on_offered_incompatible_qos()
        .once()
        .withf(|_, status| status.total_count == 1 && status.total_count_change == 1)
        .return_const(());
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            Some(Box::new(writer_listener)),
            &[StatusKind::OfferedIncompatibleQos],
        )
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Delete all entities to make sure listeners are dropped and missed functions
    // calls are detected by the mocking framework
    subscriber.delete_datareader(&reader).unwrap();
    publisher.delete_datawriter(&writer).unwrap();
    participant.delete_publisher(&publisher).unwrap();
    participant.delete_subscriber(&subscriber).unwrap();
    participant.delete_topic(&topic).unwrap();
    THE_PARTICIPANT_FACTORY
        .delete_participant(&participant)
        .unwrap();
}
