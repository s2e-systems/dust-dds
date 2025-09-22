mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;
use dust_dds::{
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{DataRepresentationQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        type_support::{DdsSerialize, DdsType},
    },
    listener::NO_LISTENER,
    publication::data_writer_listener::DataWriterListener,
    runtime::DdsRuntime,
    subscription::data_reader_listener::DataReaderListener,
    wait_set::{Condition, WaitSet},
};
use dust_dds_derive::{DdsDeserialize, TypeSupport, XTypesDeserialize};

#[test]
fn foo_with_lifetime_should_read_and_write() {
    #[derive(Clone, Debug, PartialEq, DdsType)]
    struct BorrowedData<'a> {
        #[dust_dds(key)]
        id: u8,
        value: &'a [u8],
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<BorrowedData>(
            "MyTopic",
            "BorrowedData",
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
        .create_datareader::<BorrowedData>(
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

    let data_vec = vec![1, 2, 3, 4];
    let data = BorrowedData {
        id: 1,
        value: &data_vec,
    };

    writer.write(&data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data().unwrap(), data);
}

#[test]
fn foo_with_lifetime_with_listener_should_compile() {
    #[derive(Clone, Debug, PartialEq, DdsType)]
    struct BorrowedData<'a> {
        #[dust_dds(key)]
        id: u8,
        value: &'a [u8],
    }
    struct ReaderListener;
    impl<'a, R: DdsRuntime> DataReaderListener<R, BorrowedData<'a>> for ReaderListener {}
    struct WriterListener;
    impl<'a, R: DdsRuntime> DataWriterListener<R, BorrowedData<'a>> for WriterListener {}

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<BorrowedData>(
            "MyTopic",
            "BorrowedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _writer = publisher
        .create_datawriter::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(WriterListener),
            NO_STATUS,
        )
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _reader = subscriber
        .create_datareader::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(ReaderListener),
            NO_STATUS,
        )
        .unwrap();
}

#[tokio::test]
async fn async_foo_with_lifetime_with_listener_should_compile() {
    #[derive(Clone, Debug, PartialEq, DdsType)]
    struct BorrowedData<'a> {
        #[dust_dds(key)]
        id: u8,
        value: &'a [u8],
    }
    struct ReaderListener;
    impl<'a, R: DdsRuntime> DataReaderListener<R, BorrowedData<'a>> for ReaderListener {}
    struct WriterListener;
    impl<'a, R: DdsRuntime> DataWriterListener<R, BorrowedData<'a>> for WriterListener {}

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactoryAsync::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();
    let topic = participant
        .create_topic::<BorrowedData>(
            "BorrowedDataTopic",
            "BorrowedData",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .await
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();
    let _writer = publisher
        .create_datawriter::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(WriterListener),
            NO_STATUS,
        )
        .await
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .await
        .unwrap();
    let _reader = subscriber
        .create_datareader::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(ReaderListener),
            NO_STATUS,
        )
        .await
        .unwrap();
}

#[test]
fn foo_with_non_consecutive_key_should_read_and_write() {
    #[derive(Clone, Debug, PartialEq, DdsType)]
    struct NonConsecutiveKey {
        #[dust_dds(key)]
        id: u32,
        value: Vec<u8>,
        #[dust_dds(key)]
        another_id: u64,
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<NonConsecutiveKey>(
            "MyTopic",
            "NonConsecutiveKey",
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
        .create_datareader::<NonConsecutiveKey>(
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

    let data = NonConsecutiveKey {
        id: 1,
        value: vec![1, 2, 3, 4],
        another_id: 20,
    };

    writer.write(&data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data().unwrap(), data);
}

#[test]
fn foo_enumerator_should_read_and_write() {
    #[derive(Clone, Debug, PartialEq, DdsType)]
    enum MyEnum {
        VariantA = 5,
        VariantB = 6,
        VariantC,
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<MyEnum>(
            "MyEnumTopic",
            "MyEnum",
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
        .create_datareader::<MyEnum>(
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

    let data = MyEnum::VariantB;

    writer.write(&data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data().unwrap(), data);
}

#[test]
fn nested_types_should_read_and_write() {
    #[derive(PartialEq, Eq, Debug, DdsType)]
    struct InnerType {
        a: i32,
        b: u8,
    }

    #[derive(PartialEq, Eq, Debug, DdsType)]
    struct OuterType {
        inner: InnerType,
        flag: bool,
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<OuterType>(
            "MyEnumTopic",
            "MyEnum",
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
        .create_datareader::<OuterType>(
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

    let data = OuterType {
        inner: InnerType { a: 20, b: 5 },
        flag: true,
    };

    writer.write(&data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data().unwrap(), data);
}

#[test]
fn foo_xtypes_union_should_read_and_write() {
    #[derive(Clone, Debug, PartialEq, DdsType)]
    struct MyInnerType(u32);

    #[derive(Clone, Debug, PartialEq, DdsType)]
    #[repr(u8)]
    enum MyEnum {
        VariantA(MyInnerType) = 5,
        VariantB { a: u32, b: i16 } = 6,
        VariantC = 7,
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<MyEnum>(
            "MyEnumTopic",
            "MyEnum",
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
        .create_datareader::<MyEnum>(
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

    let data = MyEnum::VariantB { a: 10, b: -20 };

    writer.write(&data, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data().unwrap(), data);
}

#[test]
fn xcdr2_writer_and_reader() {
    #[derive(Debug, PartialEq, TypeSupport, XTypesDeserialize, DdsDeserialize)]
    #[dust_dds(extensibility = "appendable")]
    pub struct ShapeType {
        #[dust_dds(key)]
        pub color: String,
        pub x: i32,
        pub y: i32,
        pub shapesize: i32,
        pub additional_payload_size: Vec<u8>,
    }

    impl DdsSerialize for ShapeType {
        fn serialize_data(&self) -> dust_dds::infrastructure::error::DdsResult<Vec<u8>> {
            Ok(vec![
                0,
                9,
                0,
                0,
                0x1c,
                0,
                0,
                0,
                // color
                5,
                0,
                0,
                0,
                b'B',
                b'L',
                b'U',
                b'E',
                0,
                //padding
                0,
                0,
                0,
                self.x.to_le_bytes()[0],
                self.x.to_le_bytes()[1],
                self.x.to_le_bytes()[2],
                self.x.to_le_bytes()[3],
                self.y.to_le_bytes()[0],
                self.y.to_le_bytes()[1],
                self.y.to_le_bytes()[2],
                self.y.to_le_bytes()[3],
                self.shapesize.to_le_bytes()[0],
                self.shapesize.to_le_bytes()[1],
                self.shapesize.to_le_bytes()[2],
                self.shapesize.to_le_bytes()[3],
                0,
                0,
                0,
                0,
            ])
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant1
        .create_topic::<ShapeType>(
            "MyTopic",
            "ShapeType",
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        representation: DataRepresentationQosPolicy { value: vec![2] },
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
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        representation: DataRepresentationQosPolicy { value: vec![2] },
        ..Default::default()
    };
    let topic = participant2
        .create_topic::<ShapeType>(
            "MyTopic",
            "ShapeType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
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
    wait_set.wait(Duration::new(100, 0)).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data = ShapeType {
        color: String::from("BLUE"),
        x: 0x2f,
        y: 0x45,
        shapesize: 0x14,
        additional_payload_size: vec![],
    };

    writer.write(&data, None).unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let samples = reader
        .take(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].data().unwrap(), data);
    assert_eq!(
        samples[0].sample_info().instance_handle.as_ref(),
        &[0, 0, 0, 5, b'B', b'L', b'U', b'E', 0, 0, 0, 0, 0, 0, 0, 0]
    )
}
