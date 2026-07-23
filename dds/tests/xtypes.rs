use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, TypeConsistencyEnforcementQosPolicy,
            TypeConsistencyKind::AllowTypeCoercion,
        },
        status::{NO_STATUS, StatusKind},
        time::{Duration, DurationKind},
        type_support::DdsType,
    },
    wait_set::{Condition, WaitSet},
    xtypes::dynamic_type::{
        DynamicData, DynamicDataFactory, DynamicTypeBuilderFactory, TryConstructKind,
    },
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[test]
fn ext_appendable_struct_2_with_dynamic_data() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let type_xml = r#"
    <dds>
        <types>
            <struct name="struct_a1" extensibility="appendable">
                <member name="x1" type="int32" />
            </struct>
            <struct name="struct_a2" extensibility="appendable">
                <member name="x1" type="int32" />
                <member name="x2" type="int32" />
            </struct>
        </types>
    </dds>
    "#;
    let type_builder =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "struct_a1", vec![]).unwrap();
    let a1_dynamic_type = type_builder.build();
    let topic1 = participant1
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            a1_dynamic_type,
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

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut type_builder =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "struct_a2", vec![]).unwrap();
    for (_id, member) in type_builder.get_all_members().unwrap() {
        member.descriptor.try_construct_kind = TryConstructKind::UseDefault;
    }
    let topic2 = participant2
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            type_builder.build(),
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
        type_consistency: TypeConsistencyEnforcementQosPolicy {
            kind: AllowTypeCoercion,
            ignore_sequence_bounds: true,
            ignore_string_bounds: true,
            ignore_member_names: false,
            prevent_type_widening: false,
            force_type_validation: false,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
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

    let mut data = DynamicDataFactory::create_data(a1_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>1</x1>
        </struct>",
    )
    .unwrap();

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap(),
        &data
    );
}

#[derive(DdsType, Debug, PartialEq, Clone)]
#[dust_dds(extensibility = "appendable")]
struct A1 {
    #[dust_dds(try_construct = "USE_DEFAULT")]
    x1: i32,
}

#[derive(DdsType, Debug, PartialEq, Clone)]
#[dust_dds(extensibility = "appendable")]
struct A2 {
    #[dust_dds(try_construct = "USE_DEFAULT")]
    x1: i32,
    #[dust_dds(try_construct = "USE_DEFAULT")]
    x2: i32,
}

#[derive(DdsType, Debug, PartialEq, Clone)]
#[dust_dds(extensibility = "appendable")]
struct A3 {
    #[dust_dds(try_construct = "USE_DEFAULT")]
    x1: i32,
    #[dust_dds(try_construct = "USE_DEFAULT")]
    x3: i32,
    #[dust_dds(try_construct = "USE_DEFAULT")]
    x2: i32,
}

#[test]
fn ext_appendable_struct_2() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic1 = participant1
        .create_topic::<A1>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
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

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic2 = participant2
        .create_topic::<A2>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        type_consistency: TypeConsistencyEnforcementQosPolicy {
            kind: AllowTypeCoercion,
            ignore_sequence_bounds: true,
            ignore_string_bounds: true,
            ignore_member_names: false,
            prevent_type_widening: false,
            force_type_validation: false,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<A2>(
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

    let data = A1 { x1: 1 };

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap().x1,
        data.x1
    );
}

#[test]
fn ext_appendable_struct_3() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic1 = participant1
        .create_topic::<A2>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
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

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic2 = participant2
        .create_topic::<A1>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber = participant2
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        type_consistency: TypeConsistencyEnforcementQosPolicy {
            kind: AllowTypeCoercion,
            ignore_sequence_bounds: true,
            ignore_string_bounds: true,
            ignore_member_names: false,
            prevent_type_widening: false,
            force_type_validation: false,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<A1>(
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

    let data = A2 { x1: 1, x2: 2 };

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap().x1,
        data.x1
    );
}

#[test]
fn ext_appendable_struct_4() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let participant1 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let topic1 = participant1
        .create_topic::<A2>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let status_cond = topic1.get_statuscondition();
    status_cond
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(status_cond))
        .unwrap();

    let participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _topic2 = participant2
        .create_topic::<A3>("A", "A", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    wait_set.wait(Duration::new(10, 0)).unwrap();
}

#[test]
fn int32_10_int32_20_should_be_inconsistent_topic() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let type_xml = r#"
    <dds>
        <types>
            <struct name="int32x10"   extensibility="final">
                <member name="x1"   type="int32" arrayDimensions="10"  />
            </struct>
            <struct name="int32x20"   extensibility="final">
                <member name="x1"   type="int32" arrayDimensions="20"  />
            </struct>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "int32x10", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let status_cond = publisher_topic.get_statuscondition();
    status_cond
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();

    let mut wait_set = WaitSet::new();
    wait_set
        .attach_condition(Condition::StatusCondition(status_cond))
        .unwrap();

    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "int32x20", vec![])
            .unwrap()
            .build();
    let _subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();

    wait_set.wait(Duration::new(10, 0)).unwrap();
}

#[test]
fn xtypes_v2_extensibility_test_suite_ext_mutable_struct_2() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let type_xml = r#"
    <dds>
        <types>
            <struct name="struct_m1"   extensibility="mutable">
                <member name="x1" type="int32" id="1" />
            </struct>
            <struct name="struct_m2"   extensibility="mutable">
                <member name="x1" type="int32" id="1" />
                <member name="x2" type="int32" id="2" />
            </struct>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "struct_m1", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "struct_m2", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        type_consistency: TypeConsistencyEnforcementQosPolicy {
            kind: AllowTypeCoercion,
            ignore_sequence_bounds: true,
            ignore_string_bounds: true,
            ignore_member_names: false,
            prevent_type_widening: false,
            force_type_validation: false,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
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

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>1</x1>
        </struct>",
    )
    .unwrap();

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap(),
        &data
    );
}

#[test]
fn xtypes_v2_array_test_suite_int32_10_2_int32_20() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="int32x10x2"   extensibility="final">
                    <member name="x1"   type="int32" arrayDimensions="10,2"  />
                </struct>
                <struct name="int32x20"   extensibility="final">
                    <member name="x1"   type="int32" arrayDimensions="20"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x10x2", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let _writer = publisher
        .create_datawriter::<DynamicData<'static>>(&publisher_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let status_cond = publisher_topic.get_statuscondition();
    status_cond
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();

    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond))
        .unwrap();

    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x20", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();

    let status_cond_subscriber = subscriber_topic.get_statuscondition();
    status_cond_subscriber
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();

    let mut wait_set_subscriber = WaitSet::new();
    wait_set_subscriber
        .attach_condition(Condition::StatusCondition(status_cond_subscriber))
        .unwrap();

    wait_set_publisher.wait(Duration::new(10, 0)).unwrap();
    wait_set_subscriber.wait(Duration::new(10, 0)).unwrap();
}


#[test]
#[ignore = "not yet working"]
fn xtypes_v2_array_test_suite_string10_10_string20_10() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let type_xml = r#"
    <dds>
        <types>
            <struct name="string10x10"   extensibility="final">
                <member name="x1"   type="string" stringMaxLength="10" arrayDimensions="10"  />
            </struct>
            <struct name="string20x10"   extensibility="final">
                <member name="x1"   type="string" stringMaxLength="20" arrayDimensions="10"  />
            </struct>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "string10x10", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        ..Default::default()
    };
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "string20x10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "A",
            "A",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        type_consistency: TypeConsistencyEnforcementQosPolicy {
            kind: AllowTypeCoercion,
            ignore_sequence_bounds: true,
            ignore_string_bounds: true,
            ignore_member_names: true,
            prevent_type_widening: false,
            force_type_validation: false,
        },
        ..Default::default()
    };
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
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

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>
                <item>ab</item>
                <item>cd</item>
                <item>ef</item>
                <item>gh</item>
                <item>ij</item>
                <item>kl</item>
                <item>mn</item>
                <item>op</item>
                <item>qr</item>
                <item>st</item>
            </x1>
        </struct>",
    )
    .unwrap();

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap(),
        &data
    );
}

