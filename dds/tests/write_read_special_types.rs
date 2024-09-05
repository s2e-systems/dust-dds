mod utils;

use std::ops::Range;

use dust_dds::{
    dds_async::{
        data_reader_listener::DataReaderListenerAsync,
        data_writer_listener::DataWriterListenerAsync,
        domain_participant_factory::DomainParticipantFactoryAsync,
    },
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
        wait_set::{Condition, WaitSet},
    },
    publication::data_writer_listener::DataWriterListener,
    subscription::{
        data_reader_listener::DataReaderListener,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, DdsType, DynamicTypeInterface},
};

use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<BorrowedData>("MyTopic", "BorrowedData", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader::<BorrowedData>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
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
    impl<'a> DataReaderListener<'a> for ReaderListener {
        type Foo = BorrowedData<'a>;
    }
    struct WriterListener;
    impl<'a> DataWriterListener<'a> for WriterListener {
        type Foo = BorrowedData<'a>;
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();
    let topic = participant
        .create_topic::<BorrowedData>("MyTopic", "BorrowedData", QosKind::Default, None, NO_STATUS)
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _writer = publisher
        .create_datawriter::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(Box::new(WriterListener)),
            NO_STATUS,
        )
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .unwrap();
    let _reader = subscriber
        .create_datareader::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(Box::new(ReaderListener)),
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
    impl<'a> DataReaderListenerAsync<'a> for ReaderListener {
        type Foo = BorrowedData<'a>;
    }
    struct WriterListener;
    impl<'a> DataWriterListenerAsync<'a> for WriterListener {
        type Foo = BorrowedData<'a>;
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant_factory = DomainParticipantFactoryAsync::new();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .await
        .unwrap();
    let topic = participant
        .create_topic::<BorrowedData>(
            "BorrowedDataTopic",
            "BorrowedData",
            QosKind::Default,
            None,
            NO_STATUS,
        )
        .await
        .unwrap();
    let publisher = participant
        .create_publisher(QosKind::Default, None, NO_STATUS)
        .await
        .unwrap();
    let _writer = publisher
        .create_datawriter::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(Box::new(WriterListener)),
            NO_STATUS,
        )
        .await
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, None, NO_STATUS)
        .await
        .unwrap();
    let _reader = subscriber
        .create_datareader::<BorrowedData>(
            &topic,
            QosKind::Default,
            Some(Box::new(ReaderListener)),
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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<NonConsecutiveKey>(
            "MyTopic",
            "NonConsecutiveKey",
            QosKind::Default,
            None,
            NO_STATUS,
        )
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
    let reader = subscriber
        .create_datareader::<NonConsecutiveKey>(
            &topic,
            QosKind::Specific(reader_qos),
            None,
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
fn foo_with_specialized_type_support_should_read_and_write() {
    #[derive(Clone, Debug, PartialEq)]
    struct DynamicType {
        value: Vec<u8>,
    }

    impl DdsSerialize for DynamicType {
        fn serialize_data(&self) -> DdsResult<Vec<u8>> {
            Ok(self.value.clone())
        }
    }

    impl<'de> DdsDeserialize<'de> for DynamicType {
        fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
            Ok(Self {
                value: serialized_data.to_vec(),
            })
        }
    }

    struct DynamicTypeSupport {
        range: Range<usize>,
    }

    impl DynamicTypeInterface for DynamicTypeSupport {
        fn has_key(&self) -> bool {
            true
        }

        fn get_serialized_key_from_serialized_foo(
            &self,
            serialized_foo: &[u8],
        ) -> DdsResult<Vec<u8>> {
            Ok(serialized_foo[self.range.clone()].to_vec())
        }

        fn instance_handle_from_serialized_foo(
            &self,
            serialized_foo: &[u8],
        ) -> DdsResult<InstanceHandle> {
            let mut handle_array = [0; 16];
            for (dst_index, src_index) in self.range.clone().enumerate() {
                handle_array[dst_index] = serialized_foo[src_index];
            }
            Ok(InstanceHandle::new(handle_array))
        }

        fn instance_handle_from_serialized_key(
            &self,
            _serialized_key: &[u8],
        ) -> DdsResult<InstanceHandle> {
            unimplemented!("Not used for this test")
        }

        fn xml_type(&self) -> String {
            String::default()
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_dynamic_topic(
            "MyTopic",
            "DynamicType",
            QosKind::Default,
            None,
            NO_STATUS,
            Box::new(DynamicTypeSupport { range: 0..4 }),
        )
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
    let reader = subscriber
        .create_datareader::<DynamicType>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
        .unwrap();

    let cond = writer.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    wait_set.wait(Duration::new(10, 0)).unwrap();

    let data1 = DynamicType {
        value: vec![1, 0, 0, 0, 1, 2, 3, 4],
    };

    let data2 = DynamicType {
        value: vec![2, 0, 0, 0, 1, 2, 3, 4],
    };

    writer.write(&data1, None).unwrap();

    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    writer.write(&data2, None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let samples = reader
        .take(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].data().unwrap(), data1);
    assert_eq!(samples[1].data().unwrap(), data2);
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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<MyEnum>("MyEnumTopic", "MyEnum", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader::<MyEnum>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
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
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<OuterType>("MyEnumTopic", "MyEnum", QosKind::Default, None, NO_STATUS)
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
    let reader = subscriber
        .create_datareader::<OuterType>(&topic, QosKind::Specific(reader_qos), None, NO_STATUS)
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
