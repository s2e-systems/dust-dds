use dust_dds::{
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync, subscriber::SubscriberAsync,
    },
    domain::{
        domain_participant_factory::DomainParticipantFactory,
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DeadlineQosPolicy, HistoryQosPolicy, HistoryQosPolicyKind, Length,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, ResourceLimitsQosPolicy,
        },
        status::{
            NO_STATUS, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleRejectedStatus, SampleRejectedStatusKind,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind},
        type_support::DdsType,
    },
    listener::NO_LISTENER,
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    runtime::DdsRuntime,
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
    topic_definition::topic_listener::TopicListener,
    wait_set::{Condition, WaitSet},
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
fn requested_deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<RequestedDeadlineMissedStatus>,
    }

    impl<R: DdsRuntime> DomainParticipantListener<R> for DeadlineMissedListener {
        async fn on_requested_deadline_missed(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: RequestedDeadlineMissedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);

    let participant_listener = DeadlineMissedListener { sender };

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(participant_listener),
            &[StatusKind::RequestedDeadlineMissed],
        )
        .unwrap();

    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

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

    impl<R: DdsRuntime> DomainParticipantListener<R> for SampleRejectedListener {
        async fn on_sample_rejected(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: SampleRejectedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let participant_listener = SampleRejectedListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(participant_listener),
            &[StatusKind::SampleRejected],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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

    let _reader = subscriber
        .create_datareader::<MyData>(
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

    writer.write(MyData { id: 1, value: 0 }, None).unwrap();
    writer.write(MyData { id: 1, value: 1 }, None).unwrap();
    writer.write(MyData { id: 1, value: 2 }, None).unwrap();

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

    impl<R: DdsRuntime> DomainParticipantListener<R> for SubscriptionMatchedListener {
        async fn on_subscription_matched(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: SubscriptionMatchedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let participant_listener = SubscriptionMatchedListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(participant_listener),
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
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

    impl<R: DdsRuntime> DomainParticipantListener<R> for RequestedIncompatibleQosListener {
        async fn on_requested_incompatible_qos(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: RequestedIncompatibleQosStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let participant_listener = RequestedIncompatibleQosListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(participant_listener),
            &[StatusKind::RequestedIncompatibleQos],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(
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

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
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

    impl<R: DdsRuntime> DomainParticipantListener<R> for PublicationMatchedListener {
        async fn on_publication_matched(
            &mut self,
            _the_writer: DataWriterAsync<R, ()>,
            status: PublicationMatchedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let participant_listener = PublicationMatchedListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(participant_listener),
            &[StatusKind::PublicationMatched],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
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
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
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

    impl<R: DdsRuntime> DomainParticipantListener<R> for OfferedIncompatibleQosListener {
        async fn on_offered_incompatible_qos(
            &mut self,
            _the_writer: DataWriterAsync<R, ()>,
            status: OfferedIncompatibleQosStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let participant_listener = OfferedIncompatibleQosListener { sender };

    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(participant_listener),
            &[StatusKind::OfferedIncompatibleQos],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
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

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
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

    impl<R: DdsRuntime> DataReaderListener<R, MyData> for DataAvailableListener {
        async fn on_data_available(&mut self, _the_reader: DataReaderAsync<R, MyData>) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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
    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let reader_listener = DataAvailableListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(reader_listener),
            &[StatusKind::DataAvailable],
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

    assert!(
        receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .is_ok()
    );
}

#[test]
fn data_on_readers_listener() {
    struct DataOnReadersListener {
        sender: std::sync::mpsc::SyncSender<()>,
    }

    impl<R: DdsRuntime> SubscriberListener<R> for DataOnReadersListener {
        async fn on_data_on_readers(&mut self, _the_subscriber: SubscriberAsync<R>) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let subscriber_listener = DataOnReadersListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(subscriber_listener),
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

    assert!(
        receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .is_ok()
    );
}

#[test]
fn data_available_listener_not_called_when_data_on_readers_listener() {
    struct DataOnReadersListener {
        sender: std::sync::mpsc::SyncSender<()>,
    }

    impl<R: DdsRuntime> SubscriberListener<R> for DataOnReadersListener {
        async fn on_data_on_readers(&mut self, _the_subscriber: SubscriberAsync<R>) {
            self.sender.send(()).unwrap();
        }
    }

    struct DataAvailableListener {
        sender: std::sync::mpsc::SyncSender<()>,
    }

    impl<R: DdsRuntime> DataReaderListener<R, MyData> for DataAvailableListener {
        async fn on_data_available(&mut self, _the_reader: DataReaderAsync<R, MyData>) {
            self.sender.send(()).unwrap();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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

    let (sender, data_on_readers_receiver) = std::sync::mpsc::sync_channel(5);
    let subscriber_listener = DataOnReadersListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(subscriber_listener),
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

    let (sender, data_available_receiver) = std::sync::mpsc::sync_channel(5);
    let reader_listener = DataAvailableListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(reader_listener),
            &[StatusKind::DataAvailable],
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

    assert!(
        data_on_readers_receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .is_ok()
    );
    assert!(
        data_available_receiver
            .recv_timeout(std::time::Duration::from_secs(1))
            .is_err()
    );
}

#[test]
fn participant_requested_deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<RequestedDeadlineMissedStatus>,
    }

    impl<R: DdsRuntime> DataReaderListener<R, MyData> for DeadlineMissedListener {
        async fn on_requested_deadline_missed(
            &mut self,
            _the_reader: DataReaderAsync<R, MyData>,
            status: RequestedDeadlineMissedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let reader_listener = DeadlineMissedListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(reader_listener),
            &[StatusKind::RequestedDeadlineMissed],
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

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
fn data_reader_sample_rejected_listener() {
    struct SampleRejectedListener {
        sender: std::sync::mpsc::SyncSender<SampleRejectedStatus>,
    }

    impl<R: DdsRuntime> DataReaderListener<R, MyData> for SampleRejectedListener {
        async fn on_sample_rejected(
            &mut self,
            _the_reader: DataReaderAsync<R, MyData>,
            status: SampleRejectedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let reader_listener = SampleRejectedListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(reader_listener),
            &[StatusKind::SampleRejected],
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

    writer.write(MyData { id: 1, value: 0 }, None).unwrap();
    writer.write(MyData { id: 1, value: 1 }, None).unwrap();
    writer.write(MyData { id: 1, value: 2 }, None).unwrap();

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
fn data_reader_subscription_matched_listener() {
    struct SubscriptionMatchedListener {
        sender: std::sync::mpsc::SyncSender<SubscriptionMatchedStatus>,
    }

    impl<R: DdsRuntime> DataReaderListener<R, MyData> for SubscriptionMatchedListener {
        async fn on_subscription_matched(
            &mut self,
            _the_reader: DataReaderAsync<R, MyData>,
            status: SubscriptionMatchedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },

        ..Default::default()
    };

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let reader_listener = SubscriptionMatchedListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(reader_listener),
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
fn data_reader_requested_incompatible_qos_listener() {
    struct RequestedIncompatibleQosListener {
        sender: std::sync::mpsc::SyncSender<RequestedIncompatibleQosStatus>,
    }

    impl<R: DdsRuntime> DataReaderListener<R, MyData> for RequestedIncompatibleQosListener {
        async fn on_requested_incompatible_qos(
            &mut self,
            _the_reader: DataReaderAsync<R, MyData>,
            status: RequestedIncompatibleQosStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(
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

        ..Default::default()
    };

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let reader_listener = RequestedIncompatibleQosListener { sender };

    let _reader = subscriber
        .create_datareader(
            &topic,
            QosKind::Specific(reader_qos),
            Some(reader_listener),
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

    impl<R: DdsRuntime> PublisherListener<R> for PublicationMatchedListener {
        async fn on_publication_matched(
            &mut self,
            _the_writer: DataWriterAsync<R, ()>,
            status: PublicationMatchedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
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
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let publisher_listener = PublicationMatchedListener { sender };
    let publisher = participant
        .create_publisher(
            QosKind::Default,
            Some(publisher_listener),
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
        },
        ..Default::default()
    };

    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
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

    impl<R: DdsRuntime> PublisherListener<R> for OfferedIncompatibleQosListener {
        async fn on_offered_incompatible_qos(
            &mut self,
            _the_writer: DataWriterAsync<R, ()>,
            status: OfferedIncompatibleQosStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
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

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let publisher_listener = OfferedIncompatibleQosListener { sender };

    let publisher = participant
        .create_publisher(
            QosKind::Default,
            Some(publisher_listener),
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
        },
        ..Default::default()
    };

    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
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
fn subscriber_requested_deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<RequestedDeadlineMissedStatus>,
    }

    impl<R: DdsRuntime> SubscriberListener<R> for DeadlineMissedListener {
        async fn on_requested_deadline_missed(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: RequestedDeadlineMissedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let subscriber_listener = DeadlineMissedListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(subscriber_listener),
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
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

    impl<R: DdsRuntime> SubscriberListener<R> for SampleRejectedListener {
        async fn on_sample_rejected(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: SampleRejectedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let subscriber_listener = SampleRejectedListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(subscriber_listener),
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

    writer.write(MyData { id: 1, value: 0 }, None).unwrap();
    writer.write(MyData { id: 1, value: 1 }, None).unwrap();
    writer.write(MyData { id: 1, value: 2 }, None).unwrap();

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

    impl<R: DdsRuntime> SubscriberListener<R> for SubscriptionMatchedListener {
        async fn on_subscription_matched(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: SubscriptionMatchedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let subscriber_listener = SubscriptionMatchedListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(subscriber_listener),
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
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
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

    impl<R: DdsRuntime> SubscriberListener<R> for RequestedIncompatibleQosListener {
        async fn on_requested_incompatible_qos(
            &mut self,
            _the_reader: DataReaderAsync<R, ()>,
            status: RequestedIncompatibleQosStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };
    let _writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Specific(data_writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let subscriber_listener = RequestedIncompatibleQosListener { sender };

    let subscriber = participant
        .create_subscriber(
            QosKind::Default,
            Some(subscriber_listener),
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
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
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

    impl<R: DdsRuntime> DataWriterListener<R, MyData> for PublicationMatchedListener {
        async fn on_publication_matched(
            &mut self,
            _the_reader: DataWriterAsync<R, MyData>,
            status: PublicationMatchedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
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
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },

        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let writer_listener = PublicationMatchedListener { sender };

    let _writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            Some(writer_listener),
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

    impl<R: DdsRuntime> DataWriterListener<R, MyData> for OfferedIncompatibleQosListener {
        async fn on_offered_incompatible_qos(
            &mut self,
            _the_reader: DataWriterAsync<R, MyData>,
            status: OfferedIncompatibleQosStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "SampleRejectedListenerTopic",
            "MyData",
            QosKind::Default,
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

    let _reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let data_writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepAll,
        },
        ..Default::default()
    };

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let writer_listener = OfferedIncompatibleQosListener { sender };

    let _writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(data_writer_qos),
            Some(writer_listener),
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

    impl<R: DdsRuntime> DomainParticipantListener<R> for NonSyncListener {}
    impl<R: DdsRuntime> PublisherListener<R> for NonSyncListener {}
    impl<R: DdsRuntime> SubscriberListener<R> for NonSyncListener {}
    impl<R: DdsRuntime> TopicListener<R> for NonSyncListener {}
    impl<R: DdsRuntime> DataWriterListener<R, MyData> for NonSyncListener {}
    impl<R: DdsRuntime> DataReaderListener<R, MyData> for NonSyncListener {}

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(NonSyncListener::new()),
            NO_STATUS,
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "NonSync",
            "MyData",
            QosKind::Default,
            Some(NonSyncListener::new()),
            NO_STATUS,
        )
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, Some(NonSyncListener::new()), NO_STATUS)
        .unwrap();
    let _data_reader = subscriber
        .create_datareader::<MyData>(
            &topic,
            QosKind::Default,
            Some(NonSyncListener::new()),
            NO_STATUS,
        )
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, Some(NonSyncListener::new()), NO_STATUS)
        .unwrap();
    let _data_writer = publisher
        .create_datawriter::<MyData>(
            &topic,
            QosKind::Default,
            Some(NonSyncListener::new()),
            NO_STATUS,
        )
        .unwrap();

    // This test doesn't assert. If trait bounds are not correct compilation will fail.
}

#[test]
fn writer_offered_deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<OfferedDeadlineMissedStatus>,
    }

    impl<R: DdsRuntime> DataWriterListener<R, MyData> for DeadlineMissedListener {
        async fn on_offered_deadline_missed(
            &mut self,
            _the_writer: DataWriterAsync<R, MyData>,
            status: OfferedDeadlineMissedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let writer_listener = DeadlineMissedListener { sender };
    let writer = publisher
        .create_datawriter(
            &topic,
            QosKind::Specific(writer_qos),
            Some(writer_listener),
            &[StatusKind::OfferedDeadlineMissed],
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

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
fn publisher_offered_deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<OfferedDeadlineMissedStatus>,
    }

    impl<R: DdsRuntime> PublisherListener<R> for DeadlineMissedListener {
        async fn on_offered_deadline_missed(
            &mut self,
            _the_writer: DataWriterAsync<R, ()>,
            status: OfferedDeadlineMissedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let publisher_listener = DeadlineMissedListener { sender };
    let publisher = participant
        .create_publisher(
            QosKind::Default,
            Some(publisher_listener),
            &[StatusKind::OfferedDeadlineMissed],
        )
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

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
fn participant_offered_deadline_missed_listener() {
    struct DeadlineMissedListener {
        sender: std::sync::mpsc::SyncSender<OfferedDeadlineMissedStatus>,
    }

    impl<R: DdsRuntime> DomainParticipantListener<R> for DeadlineMissedListener {
        async fn on_offered_deadline_missed(
            &mut self,
            _the_writer: DataWriterAsync<R, ()>,
            status: OfferedDeadlineMissedStatus,
        ) {
            self.sender.send(status).ok();
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let (sender, receiver) = std::sync::mpsc::sync_channel(5);
    let participant_listener = DeadlineMissedListener { sender };
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(participant_listener),
            &[StatusKind::OfferedDeadlineMissed],
        )
        .unwrap();
    let topic = participant
        .create_topic::<MyData>(
            "MyTopic",
            "MyData",
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
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
        deadline: DeadlineQosPolicy {
            period: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };

    let _reader = subscriber
        .create_datareader::<MyData>(
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

    let data1 = MyData { id: 1, value: 1 };
    writer.write(data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let status = receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    assert_eq!(status.total_count, 1);
    assert_eq!(status.total_count_change, 1);
}
