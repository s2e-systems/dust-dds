use dust_dds::{
    domain::{
        domain_participant_factory::DomainParticipantFactory,
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        listeners::NoOpListener,
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
    topic_definition::{topic_listener::TopicListener, type_support::DdsType},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[derive(Debug, PartialEq, DdsType)]
struct MyData {
    #[dust_dds(key)]
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
            participant_listener,
            &[StatusKind::RequestedDeadlineMissed],
        )
        .unwrap();

    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
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
            participant_listener,
            &[StatusKind::SampleRejected],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
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
            participant_listener,
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
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
            participant_listener,
            &[StatusKind::RequestedIncompatibleQos],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
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
            participant_listener,
            &[StatusKind::PublicationMatched],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
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
            participant_listener,
            &[StatusKind::OfferedIncompatibleQos],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
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

    impl DataReaderListener for DataAvailableListener {
        type Foo = MyData;
        fn on_data_available(&mut self, _the_reader: &DataReader<MyData>) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            reader_listener,
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
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = DataOnReadersListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            subscriber_listener,
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
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

    impl DataReaderListener for DataAvailableListener {
        type Foo = MyData;
        fn on_data_available(&mut self, _the_reader: &DataReader<MyData>) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, data_on_readers_receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = DataOnReadersListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            subscriber_listener,
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
            reader_listener,
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
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<RequestedDeadlineMissedStatus>,
    }

    impl DataReaderListener for DeadlineMissedListener {
        type Foo = MyData;
        fn on_requested_deadline_missed(
            &mut self,
            _the_reader: &DataReader<MyData>,
            status: RequestedDeadlineMissedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let reader_listener = DeadlineMissedListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            reader_listener,
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

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn participant_sample_rejected_listener() {
    struct SampleRejectedListener {
        sender: std::sync::mpsc::SyncSender<SampleRejectedStatus>,
    }

    impl DataReaderListener for SampleRejectedListener {
        type Foo = MyData;
        fn on_sample_rejected(
            &mut self,
            _the_reader: &DataReader<MyData>,
            status: SampleRejectedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let reader_listener = SampleRejectedListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            reader_listener,
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

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();

    assert!(status.total_count >= 1);
    assert_eq!(
        status.last_reason,
        SampleRejectedStatusKind::RejectedBySamplesLimit
    );
}

#[test]
fn participant_subscription_matched_listener() {
    struct SubscriptionMatchedListener {
        sender: std::sync::mpsc::SyncSender<SubscriptionMatchedStatus>,
    }

    impl DataReaderListener for SubscriptionMatchedListener {
        type Foo = MyData;
        fn on_subscription_matched(
            &mut self,
            _the_reader: &DataReader<MyData>,
            status: SubscriptionMatchedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let reader_listener = SubscriptionMatchedListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            reader_listener,
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn participant_requested_incompatible_qos_listener() {
    struct RequestedIncompatibleQosListener {
        sender: std::sync::mpsc::SyncSender<RequestedIncompatibleQosStatus>,
    }

    impl DataReaderListener for RequestedIncompatibleQosListener {
        type Foo = MyData;
        fn on_requested_incompatible_qos(
            &mut self,
            _the_reader: &DataReader<MyData>,
            status: RequestedIncompatibleQosStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let reader_listener = RequestedIncompatibleQosListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            reader_listener,
            &[StatusKind::RequestedIncompatibleQos],
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn publisher_publication_matched_listener() {
    struct PublicationMatchedListener {
        sender: std::sync::mpsc::SyncSender<PublicationMatchedStatus>,
    }

    impl PublisherListener for PublicationMatchedListener {
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

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let publisher_listener = PublicationMatchedListener { sender };
    let publisher = participant
        .create_publisher(
            QosKind::Default,
            publisher_listener,
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

    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert!(status.current_count >= 1);
    assert_eq!(status.current_count_change, 1);
}

#[test]
fn publisher_offered_incompatible_qos_listener() {
    struct OfferedIncompatibleQosListener {
        sender: std::sync::mpsc::SyncSender<OfferedIncompatibleQosStatus>,
    }

    impl PublisherListener for OfferedIncompatibleQosListener {
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

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let publisher_listener = OfferedIncompatibleQosListener { sender };

    let publisher = participant
        .create_publisher(
            QosKind::Default,
            publisher_listener,
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

    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn subscriber_deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<RequestedDeadlineMissedStatus>,
    }

    impl SubscriberListener for DeadlineMissedListener {
        fn on_requested_deadline_missed(
            &mut self,
            _the_reader: &dyn AnyDataReader,
            status: RequestedDeadlineMissedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = DeadlineMissedListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            subscriber_listener,
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

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
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
        .wait_for_acknowledgments(Duration::new(1, 0))
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
}

#[test]
fn subscriber_sample_rejected_listener() {
    struct SampleRejectedListener {
        sender: std::sync::mpsc::SyncSender<SampleRejectedStatus>,
    }

    impl SubscriberListener for SampleRejectedListener {
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

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = SampleRejectedListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            subscriber_listener,
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

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
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

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(
        status.last_reason,
        SampleRejectedStatusKind::RejectedBySamplesLimit
    );
}

#[test]
fn subscriber_subscription_matched_listener() {
    struct SubscriptionMatchedListener {
        sender: std::sync::mpsc::SyncSender<SubscriptionMatchedStatus>,
    }

    impl SubscriberListener for SubscriptionMatchedListener {
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

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = SubscriptionMatchedListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            subscriber_listener,
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

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn subscriber_requested_incompatible_qos_listener() {
    struct RequestedIncompatibleQosListener {
        sender: std::sync::mpsc::SyncSender<RequestedIncompatibleQosStatus>,
    }

    impl SubscriberListener for RequestedIncompatibleQosListener {
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

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let subscriber_listener = RequestedIncompatibleQosListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            subscriber_listener,
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

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn data_writer_publication_matched_listener() {
    struct PublicationMatchedListener {
        sender: std::sync::mpsc::SyncSender<PublicationMatchedStatus>,
    }

    impl DataWriterListener for PublicationMatchedListener {
        type Foo = MyData;
        fn on_publication_matched(
            &mut self,
            _the_reader: &DataWriter<MyData>,
            status: PublicationMatchedStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let writer_listener = PublicationMatchedListener { sender };

    let _writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            writer_listener,
            &[StatusKind::PublicationMatched],
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.current_count, 1);
    assert_eq!(status.current_count_change, 1);
}

#[test]
fn data_writer_offered_incompatible_qos_listener() {
    struct OfferedIncompatibleQosListener {
        sender: std::sync::mpsc::SyncSender<OfferedIncompatibleQosStatus>,
    }

    impl DataWriterListener for OfferedIncompatibleQosListener {
        type Foo = MyData;

        fn on_offered_incompatible_qos(
            &mut self,
            _the_reader: &DataWriter<MyData>,
            status: OfferedIncompatibleQosStatus,
        ) {
            self.sender.send(status).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let writer_listener = OfferedIncompatibleQosListener { sender };

    let _writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            writer_listener,
            &[StatusKind::OfferedIncompatibleQos],
        )
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}

#[test]
fn non_sync_listener_should_be_accepted() {
    // Use Cell to create a type which is Send but not Sync
    struct NonSyncListener(std::cell::Cell<()>);

    impl NonSyncListener {
        fn new() -> Self {
            Self(std::cell::Cell::new(()))
        }
    }

    impl DomainParticipantListener for NonSyncListener {}
    impl PublisherListener for NonSyncListener {}
    impl SubscriberListener for NonSyncListener {}
    impl TopicListener for NonSyncListener {}
    impl DataWriterListener for NonSyncListener {
        type Foo = MyData;
    }
    impl DataReaderListener for NonSyncListener {
        type Foo = MyData;
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            NonSyncListener::new(),
            NO_STATUS,
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "NonSync",
            "MyData",
            QosKind::Default,
            NonSyncListener::new(),
            NO_STATUS,
        )
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, NonSyncListener::new(), NO_STATUS)
        .unwrap();
    let _data_reader = subscriber
        .create_datareader::<MyData>(&topic, QosKind::Default, NonSyncListener::new(), NO_STATUS)
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, NonSyncListener::new(), NO_STATUS)
        .unwrap();
    let _data_writer = publisher
        .create_datawriter::<MyData>(&topic, QosKind::Default, NonSyncListener::new(), NO_STATUS)
        .unwrap();

    // This test doesn't assert. If trait bounds are not correct compilation will fail.
}
