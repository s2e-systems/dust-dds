mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;
use dust_dds::{
    domain::{
        domain_participant_factory::DomainParticipantFactory,
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DataRepresentationQosPolicy, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            TypeConsistencyEnforcementQosPolicy, TypeConsistencyKind::AllowTypeCoercion,
            XCDR2_DATA_REPRESENTATION,
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
use std::sync::mpsc::{self, channel};

// In the OMG XTypes tests the ignore_member_names of the TypeConsistencyEnforcementQosPolicy
// is set to true by default. The standards default is false though
fn reader_qos() -> DataReaderQos {
    DataReaderQos {
        representation: DataRepresentationQosPolicy {
            value: vec![XCDR2_DATA_REPRESENTATION],
        },
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
    }
}
// These are tests for XCDR2
fn writer_qos() -> DataWriterQos {
    DataWriterQos {
        representation: DataRepresentationQosPolicy {
            value: vec![XCDR2_DATA_REPRESENTATION],
        },
        ..DataWriterQos::const_default()
    }
}

struct Listener {
    sender: mpsc::Sender<String>,
}
impl DomainParticipantListener for Listener {
    fn on_publication_matched(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        _status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        self.sender
            .send("on_publication_matched()".to_string())
            .unwrap();
        core::future::ready(())
    }
    fn on_subscription_matched(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        _status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) -> impl Future<Output = ()> + Send {
        self.sender
            .send("on_subscription_matched()".to_string())
            .unwrap();
        core::future::ready(())
    }
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: dust_dds::dds_async::topic::TopicAsync,
        _status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) -> impl Future<Output = ()> + Send {
        self.sender
            .send("on_inconsistent_topic()".to_string())
            .unwrap();
        core::future::ready(())
    }
}
/// 'ext_final_struct_1' : {
///     'common_args' : ['--type-folder types --type-file extensibility'],
///     'apps' : ['pub-exe -P -t test -y Test::struct_f1 --data-folder data --data-file struct_num_x1',
///               'sub-exe -S -t test -y Test::struct_f1 --data-folder data --data-file struct_num_x1 --ignore-member-names f'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between identical struct_f1 (subscriber with ignore_member_names false)',
///     'description' : 'Verifies identical final structs can communicate:\n\n'
///                     ' * Publisher and Subscriber use `struct_f1` (final) from `extensibility`.\n'
///                     ' * Subscriber sets `--ignore-member-names` to `false`.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_extensibility_test_suite_ext_final_struct_1() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let (sender, receiver) = channel();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Listener {
                sender: sender.clone(),
            }),
            &[StatusKind::PublicationMatched],
        )
        .unwrap();

    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="struct_f1"   extensibility="final">
                    <member name="x1" type="int32" />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let type_builder =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_f1", vec![])
            .unwrap();
    let publisher_dynamic_type = type_builder.build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_f1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Listener {
                sender: sender.clone(),
            }),
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();
    let type_builder =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_f1", vec![])
            .unwrap();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_f1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            type_builder.build(),
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_member_names = false;
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    // Note: In the OMG XTYpes tests the DomainParticipantListener is used to check
    // if the publication or subscriptions are matched. To mimic that test even closer here
    // the (actually better fitting) status condition is not used
    receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();
    receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();

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

/// 'ext_final_struct_2' : {
///     'common_args' : ['--type-folder types --type-file extensibility'],
///     'apps' : ['pub-exe -P -t test -y Test::struct_f1 --data-folder data --data-file struct_num_x1',
///               'sub-exe -S -t test -y Test::struct_f2 --data-folder data --data-file struct_num_x1 --ignore-member-names f'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between struct_f1 and struct_f2 (subscriber with ignore_member_names false)',
///     'description' : 'Verifies final structs with different member counts are not assignable:\n\n'
///                     ' * Publisher uses `struct_f1` (final) from `extensibility`.\n'
///                     ' * Subscriber uses `struct_f2` (final) from `extensibility`.\n'
///                     ' * `struct_f2` has an extra member `x2` (`int32`) at the end.\n'
///                     ' * Final extensibility forbids appending members.\n'
///                     ' * Subscriber sets `--ignore-member-names` to `false`.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// }
#[test]
fn xtypes_v2_extensibility_test_suite_ext_final_struct_2() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="struct_f1"   extensibility="final">
                    <member name="x1" type="int32" />
                </struct>
                <struct name="struct_f2"   extensibility="final">
                    <member name="x1" type="int32" />
                    <member name="x2" type="int32" />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_f1", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_f1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_f2", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_f2",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'int32[10]_uint32[10]' : {
///     'common_args' : ['--type-folder types --type-file arrays'],
///     'apps' : ['pub-exe -P -t test -y Test::int32x10 --data-folder data --data-file array_num_10',
///               'sub-exe -S -t test -y Test::uint32x10 --data-folder data --data-file array_num_10'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between int32x10 and uint32x10',
///     'description' : 'Verifies arrays with different element types are not assignable:\n\n'
///                     ' * Publisher uses `int32x10` from `arrays`.\n'
///                     ' * Subscriber uses `uint32x10` from `arrays`.\n'
///                     ' * Publisher element type is `int32`.\n'
///                     ' * Subscriber element type is `uint32`.\n'
///                     ' * Array elements must be strongly assignable.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// }
#[test]
fn xtypes_v2_array_test_suite_int32_10_uint32_10() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="int32x10"   extensibility="final">
                    <member name="x1"   type="int32" arrayDimensions="10"  />
                </struct>
                <struct name="uint32x10"   extensibility="final">
                    <member name="x1"   type="uint32" arrayDimensions="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x10", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::int32x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::uint32x10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::uint32x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'enum1[10]_enum2[10]' : {
///     'common_args' : ['--type-folder types --type-file arrays'],
///     'apps' : ['pub-exe -P -t test -y Test::enum1x10 --data-folder data --data-file array_enum_10',
///               'sub-exe -S -t test -y Test::enum2x10 --data-folder data --data-file array_enum_10'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between enum1x10 and enum2x10',
///     'description' : 'Verifies arrays of appendable enums with subset literals are assignable:\n\n'
///                     ' * Publisher uses `enum1x10` from `arrays`.\n'
///                     ' * Subscriber uses `enum2x10` from `arrays`.\n'
///                     ' * Both are enum arrays of size 10.\n'
///                     ' * Publisher uses `E1` (3 literals: VAL0-VAL2), subscriber uses `E2` (4 literals: VAL0-VAL3).\n'
///                     ' * `E2` is a superset of `E1`, so elements are strongly assignable.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_array_test_suite_enum1_10_enum2_10() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <enum name="E1" bitBound="32" extensibility="appendable">
                    <enumerator name="VAL0" value="0"/>
                    <enumerator name="VAL1" value="1"/>
                    <enumerator name="VAL2" value="2"/>
                </enum>
                <enum name="E2" bitBound="32" extensibility="appendable">
                    <enumerator name="VAL0" value="0"/>
                    <enumerator name="VAL1" value="1"/>
                    <enumerator name="VAL2" value="2"/>
                    <enumerator name="VAL3" value="3"/>
                </enum>
                <struct name="enum1x10"   extensibility="final">
                    <member name="x1"   type="nonBasic" nonBasicTypeName="E1" arrayDimensions="10"  />
                </struct>
                <struct name="enum2x10"   extensibility="final">
                    <member name="x1"   type="nonBasic" nonBasicTypeName="E2" arrayDimensions="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::enum1x10", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::enum1x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::enum2x10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::enum2x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();
    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
                <item>VAL1</item>
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

/// 'ext_appendable_struct_2' : {
///     'common_args' : ['--type-folder types --type-file extensibility'],
///     'apps' : ['pub-exe -P -t test -y Test::struct_a1 --data-folder data --data-file struct_num_x1',
///               'sub-exe -S -t test -y Test::struct_a2 --data-folder data --data-file struct_num_x1 --ignore-member-names f'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between struct_a1 and struct_a2 (subscriber with ignore_member_names false)',
///     'description' : 'Verifies appendable structs allow an appended trailing member:\n\n'
///                     ' * Publisher uses `struct_a1` (appendable) from `extensibility`.\n'
///                     ' * Subscriber uses `struct_a2` (appendable) from `extensibility`.\n'
///                     ' * `struct_a2` has an extra member `x2` (`int32`) appended at the end.\n'
///                     ' * Appendable extensibility permits this.\n'
///                     ' * Subscriber sets `--ignore-member-names` to `false`.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_extensibility_test_suite_ext_appendable_struct_2() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="struct_a1" extensibility="appendable">
                    <member name="x1" type="int32" />
                </struct>
                <struct name="struct_a2" extensibility="appendable">
                    <member name="x1" type="int32" />
                    <member name="x2" type="int32" />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let type_builder =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_a1", vec![])
            .unwrap();
    let publisher_dynamic_type = type_builder.build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_a1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut type_builder =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_a2", vec![])
            .unwrap();
    // Connext does have UseDefault as default, the standard says in 7.2.2.7 Try Construct behavior to use Discard by default
    for (_id, member) in type_builder.get_all_members().unwrap() {
        member.descriptor.try_construct_kind = TryConstructKind::UseDefault;
    }
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_a2",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            type_builder.build(),
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_member_names = false;
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();
    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

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

/// 'ext_appendable_struct_3' : {
///     'common_args' : ['--type-folder types --type-file extensibility'],
///     'apps' : ['pub-exe -P -t test -y Test::struct_a2 --data-folder data --data-file struct_num_x1_x2',
///               'sub-exe -S -t test -y Test::struct_a1 --data-folder data --data-file struct_num_x1 --ignore-member-names f'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between struct_a2 and struct_a1 (subscriber with ignore_member_names false)',
///     'description' : 'Verifies appendable structs allow the publisher to have additional trailing members:\n\n'
///                     ' * Publisher uses `struct_a2` (appendable) from `extensibility`.\n'
///                     ' * Subscriber uses `struct_a1` (appendable) from `extensibility`.\n'
///                     ' * Publisher\'s `struct_a2` has an extra trailing member `x2` (`int32`) that the subscriber\'s `struct_a1` ignores.\n'
///                     ' * Subscriber sets `--ignore-member-names` to `false`.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_extensibility_test_suite_ext_appendable_struct_3() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let publisher_topic = publisher_participant
        .create_topic::<A2>("test", "A2", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_topic = subscriber_participant
        .create_topic::<A1>("test", "A1", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_member_names = false;
    let reader = subscriber
        .create_datareader::<A1>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();
    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

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

/// 'ext_appendable_struct_4' : {
///     'common_args' : ['--type-folder types --type-file extensibility'],
///     'apps' : ['pub-exe -P -t test -y Test::struct_a2 --data-folder data --data-file struct_num_x1_x2',
///               'sub-exe -S -t test -y Test::struct_a3 --data-folder data --data-file struct_num_x1_x2 --ignore-member-names f'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between struct_a2 and struct_a3 (subscriber with ignore_member_names false)',
///     'description' : 'Verifies appendable structs with a member inserted in the middle are not assignable:\n\n'
///                     ' * Publisher uses `struct_a2` (appendable) from `extensibility`.\n'
///                     ' * Subscriber uses `struct_a3` (appendable) from `extensibility`.\n'
///                     ' * `struct_a3` inserts member `x3` between `x1` and `x2`, changing the serialization order.\n'
///                     ' * Appendable types require positional matching.\n'
///                     ' * Subscriber sets `--ignore-member-names` to `false`.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// }
#[test]
fn xtypes_v2_extensibility_test_suite_ext_appendable_struct_4() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let publisher_topic = publisher_participant
        .create_topic::<A2>("test", "A2", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_topic = subscriber_participant
        .create_topic::<A3>("test", "A3", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_member_names = false;
    let _reader = subscriber
        .create_datareader::<A3>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'int32[10]_int32[20]' : {
///     'common_args' : ['--type-folder types --type-file arrays'],
///     'apps' : ['pub-exe -P -t test -y Test::int32x10 --data-folder data --data-file array_num_10',
///               'sub-exe -S -t test -y Test::int32x20 --data-folder data --data-file array_num_20'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between int32x10 and int32x20',
///     'description' : 'Verifies sequence with smaller bound is assignable to sequence with larger bound:\n\n'
///                     ' * Publisher uses `int32x10` from `arrays`.\n'
///                     ' * Subscriber uses `int32x20` from `arrays`.\n'
///                     ' * Publisher uses `sequence<int32, 10>`.\n'
///                     ' * Subscriber uses `sequence<int32, 20>`.\n'
///                     ' * Subscriber bound >= publisher bound, and data fits.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// }
#[test]
fn xtypes_v2_array_test_suite_int32_10_int32_20() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="int32x10"   extensibility="final">
                    <member name="x1"   type="int32" arrayDimensions="10"  />
                </struct>
                <struct name="int32x20"   extensibility="final">
                    <member name="x1"   type="int32" arrayDimensions="20"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x10", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::int32x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
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
            "test",
            "Test::int32x20",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'ext_mutable_struct_2' : {
///     'common_args' : ['--type-folder types --type-file extensibility'],
///     'apps' : ['pub-exe -P -t test -y Test::struct_m1 --data-folder data --data-file struct_num_x1',
///               'sub-exe -S -t test -y Test::struct_m2 --data-folder data --data-file struct_num_x1 --ignore-member-names f'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between struct_m1 and struct_m2 (subscriber with ignore_member_names false)',
///     'description' : 'Verifies mutable structs allow an extra member with explicit ID:\n\n'
///                     ' * Publisher uses `struct_m1` (mutable) from `extensibility`.\n'
///                     ' * Subscriber uses `struct_m2` (mutable) from `extensibility`.\n'
///                     ' * `struct_m2` has an extra member `x2` with explicit `id=2`.\n'
///                     ' * Mutable types match by member ID, so extra members are allowed.\n'
///                     ' * Subscriber sets `--ignore-member-names` to `false`.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_extensibility_test_suite_ext_mutable_struct_2() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let (sender, receiver) = channel();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Listener {
                sender: sender.clone(),
            }),
            &[StatusKind::PublicationMatched],
        )
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="struct_m1"   extensibility="mutable">
                    <member name="x1" type="int32" id="1" />
                </struct>
                <struct name="struct_m2"   extensibility="mutable">
                    <member name="x1" type="int32" id="1" />
                    <member name="x2" type="int32" id="2" />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_m1", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_m1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Listener {
                sender: sender.clone(),
            }),
            &[StatusKind::SubscriptionMatched],
        )
        .unwrap();

    let mut type_builder =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_m2", vec![])
            .unwrap();
    // Connext does have UseDefault as default, the standard says in 7.2.2.7 Try Construct behavior to use Discard by default
    for (_id, member) in type_builder.get_all_members().unwrap() {
        member.descriptor.try_construct_kind = TryConstructKind::UseDefault;
    }
    let subscriber_dynamic_type = type_builder.build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_m1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_member_names = false;
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();

    receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap();

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

    let sample = reader.read_next_sample().unwrap();
    assert_eq!(sample.data.as_ref().unwrap(), &data);
}

/// 'int32[10][2]_int32[20]' : {
///     'common_args' : ['--type-folder types --type-file arrays'],
///     'apps' : ['pub-exe -P -t test -y Test::int32x10x2 --data-folder data --data-file array_num_20',
///               'sub-exe -S -t test -y Test::int32x20 --data-folder data --data-file array_num_20'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between int32x10x2 and int32x20',
///     'description' : 'Verifies multi-dimensional and single-dimensional arrays of same total size are not assignable:\n\n'
///                     ' * Publisher uses `int32x10x2` from `arrays`.\n'
///                     ' * Subscriber uses `int32x20` from `arrays`.\n'
///                     ' * Publisher is `int32[10][2]` (2D, 20 elements total).\n'
///                     ' * Subscriber is `int32[20]` (1D).\n'
///                     ' * Dimensions must match structurally, not just in total count.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// }
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
            "test",
            "Test::int32x10x2",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
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
            "test",
            "Test::int32x20",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'string10[10]_string20[10]' : {
///     'common_args' : ['--type-folder types --type-file arrays'],
///     'apps' : ['pub-exe -P -t test -y Test::string10x10 --data-folder data --data-file array_string_10',
///               'sub-exe -S -t test -y Test::string20x10 --data-folder data --data-file array_string_10'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between string10x10 and string20x10',
///     'description' : 'Verifies sequences of strings with different string bounds are assignable:\n\n'
///                     ' * Publisher uses `string10x10` from `arrays`.\n'
///                     ' * Subscriber uses `string20x10` from `arrays`.\n'
///                     ' * Both are `sequence<string, 10>`.\n'
///                     ' * Publisher string bound is 10, subscriber is 20.\n'
///                     ' * String elements are strongly assignable since subscriber bound >= publisher bound.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
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
            "test",
            "Test::string10x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
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
            "test",
            "Test::string20x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let cond_publication = writer.get_statuscondition();
    cond_publication
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut wait_set_publication = WaitSet::new();
    wait_set_publication
        .attach_condition(Condition::StatusCondition(cond_publication))
        .unwrap();
    let cond_subscription = reader.get_statuscondition();
    cond_subscription
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut wait_set_subscription = WaitSet::new();
    wait_set_subscription
        .attach_condition(Condition::StatusCondition(cond_subscription))
        .unwrap();

    wait_set_subscription.wait(Duration::new(10, 0)).unwrap();
    wait_set_publication.wait(Duration::new(10, 0)).unwrap();

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

    assert_eq!(
        publisher_topic
            .get_inconsistent_topic_status()
            .unwrap()
            .total_count,
        0
    );
    assert_eq!(
        subscriber_topic
            .get_inconsistent_topic_status()
            .unwrap()
            .total_count,
        0
    );
}

/// 'SFinal[10]_S[20]_SFinalAlt[10]_S[20]' : {
///     'common_args' : ['--type-folder types --type-file arrays'],
///     'apps' : ['pub-exe -P -t test -y Test::F_S__array10_F_S__array20_uint32 --data-folder data --data-file array_array_num_10_20',
///               'sub-exe -S -t test -y Test::F_S__array10_F_S__array20_uint32_alt --data-folder data --data-file array_array_num_10_20_alt'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between F_S__array10_F_S__array20_uint32 and F_S__array10_F_S__array20_uint32_alt',
///     'description' : 'Verifies arrays of final structs are assignable when inner struct elements are strongly assignable:\n\n'
///                     ' * Publisher uses `F_S__array10_F_S__array20_uint32` from `arrays`.\n'
///                     ' * Subscriber uses `F_S__array10_F_S__array20_uint32_alt` from `arrays`.\n'
///                     ' * Both are arrays of 10 final structs containing `uint32[20]`.\n'
///                     ' * Member names differ (`x1` vs `altx1`) but the types are structurally equivalent.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_array_test_suite_s_final_10_s_20_s_final_alt_10_s_20() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let (sender, receiver) = mpsc::channel();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Listener {
                sender: sender.clone(),
            }),
            &[StatusKind::InconsistentTopic],
        )
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="F_S__array20_uint32"   extensibility="final">
                    <member name="x1"   type="uint32" arrayDimensions="20"  />
                </struct>
                <struct name="F_S__array20_uint32_alt"   extensibility="final">
                    <member name="altx1"   type="uint32" arrayDimensions="20"  />
                </struct>
                <struct name="F_S__array10_F_S__array20_uint32"   extensibility="final">
                    <member name="x1"   type="nonBasic" nonBasicTypeName="F_S__array20_uint32" arrayDimensions="10"  />
                </struct>
                <struct name="F_S__array10_F_S__array20_uint32_alt"   extensibility="final">
                    <member name="altx1"   type="nonBasic" nonBasicTypeName="F_S__array20_uint32_alt" arrayDimensions="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::F_S__array10_F_S__array20_uint32",
        vec![],
    )
    .unwrap()
    .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::F_S__array10_F_S__array20_uint32",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(
            domain_id,
            QosKind::Default,
            Some(Listener {
                sender: sender.clone(),
            }),
            &[StatusKind::InconsistentTopic],
        )
        .unwrap();
    let subscriber_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::F_S__array10_F_S__array20_uint32_alt",
        vec![],
    )
    .unwrap()
    .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::F_S__array10_F_S__array20_uint32_alt",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item><item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item><item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item><item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item><item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item><item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item><item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item>
                    <item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item><item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item>
                    <item>20</item>
                    </x1>
                </item>
                <item>
                    <x1>
                    <item>1</item><item>2</item><item>3</item><item>4</item><item>5</item><item>6</item><item>7</item><item>8</item><item>9</item><item>10</item><item>11</item><item>12</item><item>13</item><item>14</item><item>15</item><item>16</item><item>17</item><item>18</item><item>19</item>
                    <item>20</item>
                    </x1>
                </item>
            </x1>
            </struct>
",
    )
    .unwrap();

    let writer_cond = writer.get_statuscondition();
    writer_cond
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_cond))
        .unwrap();
    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_cond))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    writer.write(data.clone(), None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert_eq!(
        reader.read_next_sample().unwrap().data.as_ref().unwrap(),
        &data
    );

    assert_eq!(
        publisher_topic
            .get_inconsistent_topic_status()
            .unwrap()
            .total_count,
        0
    );
    assert_eq!(
        subscriber_topic
            .get_inconsistent_topic_status()
            .unwrap()
            .total_count,
        0
    );
    assert!(
        receiver.try_recv().is_err(),
        "on_inconsistent_topic listener callback was unexpectedly triggered"
    );
}

/// 'seq(int32)_seq(int32,10)' : {
///     'common_args' : ['--type-folder types --type-file sequences'],
///     'apps' : ['pub-exe -P -t test -y Test::int32_unbounded --data-folder data --data-file array_num_10',
///               'sub-exe -S -t test -y Test::int32x10 --data-folder data --data-file array_num_10'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between int32_unbounded and int32x10',
///     'description' : 'Verifies unbounded sequence is assignable to bounded sequence (default ignore_seq_bounds):\n\n'
///                     ' * Publisher uses `int32_unbounded` from `sequences`.\n'
///                     ' * Subscriber uses `int32x10` from `sequences`.\n'
///                     ' * Publisher uses unbounded `sequence<int32>`.\n'
///                     ' * Subscriber uses `sequence<int32, 10>`.\n'
///                     ' * By default, sequence bounds are ignored for assignability.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
#[test]
fn xtypes_v2_sequence_test_suite_seq_int32_seq_int32_10() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="int32_unbounded"   extensibility="final">
                    <member name="x1"   type="int32" sequenceMaxLength="-1"  /> <!-- unlimited (0 or -1) -->
                </struct>
                <struct name="int32x10"   extensibility="final">
                    <member name="x1"   type="int32" sequenceMaxLength="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::int32_unbounded",
        vec![],
    )
    .unwrap()
    .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::int32_unbounded",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::int32x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_condition = writer.get_statuscondition();
    writer_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_condition))
        .unwrap();
    let reader_condition = reader.get_statuscondition();
    reader_condition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_condition))
        .unwrap();

    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>
                <item>1</item>
                <item>2</item>
                <item>3</item>
                <item>4</item>
                <item>5</item>
                <item>6</item>
                <item>7</item>
                <item>8</item>
                <item>9</item>
                <item>10</item>
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

/// 'seq(int32)_seq(int32,10)_check_bounds' : {
///     'common_args' : ['--type-folder types --type-file sequences'],
///     'apps' : ['pub-exe -P -t test -y Test::int32_unbounded --data-folder data --data-file array_num_10',
///               'sub-exe -S -t test -y Test::int32x10 --data-folder data --data-file array_num_10 --ignore-seq-bounds f'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between int32_unbounded and int32x10 (subscriber with ignore_seq_bounds false)',
///     'description' : 'Verifies unbounded sequence is assignable to bounded sequence (default ignore_seq_bounds):\n\n'
///                     ' * Publisher uses `int32_unbounded` from `sequences`.\n'
///                     ' * Subscriber uses `int32x10` from `sequences`.\n'
///                     ' * Publisher uses unbounded `sequence<int32>`.\n'
///                     ' * Subscriber uses `sequence<int32, 10>`.\n'
///                     ' * By default, sequence bounds are ignored for assignability.\n'
///                     ' * Subscriber sets `--ignore-seq-bounds` to `false`.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
#[test]
fn xtypes_v2_sequence_test_suite_seq_int32_seq_int32_10_check_bounds() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="int32_unbounded"   extensibility="final">
                    <member name="x1"   type="int32" sequenceMaxLength="-1"  /> <!-- unlimited (0 or -1) -->
                </struct>
                <struct name="int32x10"   extensibility="final">
                    <member name="x1"   type="int32" sequenceMaxLength="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::int32_unbounded",
        vec![],
    )
    .unwrap()
    .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::int32_unbounded",
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
        .create_datawriter::<DynamicData<'static>>(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::int32x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_sequence_bounds = false;

    let _reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'seq(int32,20)_seq(int32,10)' : {
///     'common_args' : ['--type-folder types --type-file sequences'],
///     'apps' : ['pub-exe -P -t test -y Test::int32x20 --data-folder data --data-file array_num_20',
///               'sub-exe -S -t test -y Test::int32x10 --data-folder data --data-file array_num_10'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.DATA_NOT_RECEIVED],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Type assignability between int32x20 and int32x10 but sample rejected',
///     'description' : 'Verifies sequence with larger bound sending data exceeding subscriber bound:\n\n'
///                     ' * Publisher uses `int32x20` from `sequences`.\n'
///                     ' * Subscriber uses `int32x10` from `sequences`.\n'
///                     ' * Publisher uses `sequence<int32, 20>` with 20 elements.\n'
///                     ' * Subscriber uses `sequence<int32, 10>`.\n'
///                     ' * The actual data size (20) exceeds subscriber bound (10).\n'
///                     '**Test passes if:** Discovery succeeds but the sample is not delivered.\n'
#[test]
fn xtypes_v2_sequence_test_suite_seq_int32_20_seq_int32_10() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="int32x10"   extensibility="final">
                    <member name="x1"   type="int32" sequenceMaxLength="10"  />
                </struct>
                <struct name="int32x20"   extensibility="final">
                    <member name="x1"   type="int32" sequenceMaxLength="20"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x20", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::int32x20",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::int32x10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::int32x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_condition = writer.get_statuscondition();
    writer_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_condition))
        .unwrap();
    let reader_condition = reader.get_statuscondition();
    reader_condition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_condition))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>
                <item>1</item>
                <item>2</item>
                <item>3</item>
                <item>4</item>
                <item>5</item>
                <item>6</item>
                <item>7</item>
                <item>8</item>
                <item>9</item>
                <item>10</item>
                <item>11</item>
                <item>12</item>
                <item>13</item>
                <item>14</item>
                <item>15</item>
                <item>16</item>
                <item>17</item>
                <item>18</item>
                <item>19</item>
                <item>20</item>
            </x1>
        </struct>",
    )
    .unwrap();

    writer.write(data, None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert!(reader.read_next_sample().unwrap().data.is_none());
}

/// 'string_string10' : {
///     'common_args' : ['--type-folder types --type-file strings'],
///     'apps' : ['pub-exe -P -t test -y Test::string_unbounded --data-folder data --data-file strings',
///               'sub-exe -S -t test -y Test::string10 --data-folder data --data-file strings'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.DATA_NOT_RECEIVED],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Type assignability between string_unbounded and string10 but sample rejected',
///     'description' : 'Verifies unbounded string sending data exceeding subscriber bound:\n\n'
///                     ' * Publisher uses `string_unbounded` from `strings`.\n'
///                     ' * Subscriber uses `string10` from `strings`.\n'
///                     ' * Publisher uses unbounded `string`.\n'
///                     ' * Subscriber uses `string<10>`.\n'
///                     ' * The published string ("hello world!") exceeds the subscriber bound.\n'
///                     '**Test passes if:** Discovery succeeds but the sample is not delivered.\n'
/// }
#[test]
fn xtypes_v2_sequence_test_suite_string_string10() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="string_unbounded"   extensibility="final">
                    <member name="x1"   type="string"   />
                </struct>
                <struct name="string10"   extensibility="final">
                    <member name="x1"   type="string" stringMaxLength="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::string_unbounded",
        vec![],
    )
    .unwrap()
    .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::string_unbounded",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type.clone(),
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::string10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::string10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_condition = writer.get_statuscondition();
    writer_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_condition))
        .unwrap();
    let reader_condition = reader.get_statuscondition();
    reader_condition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_condition))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<strings>
            <x1>Hello there.</x1>
        </strings>",
    )
    .unwrap();

    writer.write(data, None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert!(reader.read_next_sample().unwrap().data.is_none());
}

/// 'string_string10_check' : {
///     'common_args' : ['--type-folder types --type-file strings'],
///     'apps' : ['pub-exe -P -t test -y Test::string_unbounded --data-folder data --data-file strings',
///               'sub-exe -S -t test -y Test::string10 --data-folder data --data-file strings --ignore-str-bounds f'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between string_unbounded and string10 (subscriber with ignore_str_bounds false)',
///     'description' : 'Verifies unbounded string sending data exceeding subscriber bound:\n\n'
///                     ' * Publisher uses `string_unbounded` from `strings`.\n'
///                     ' * Subscriber uses `string10` from `strings`.\n'
///                     ' * Publisher uses unbounded `string`.\n'
///                     ' * Subscriber uses `string<10>`.\n'
///                     ' * The published string ("hello world!") exceeds the subscriber bound.\n'
///                     ' * Subscriber sets `--ignore-str-bounds` to `false`.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// },
#[test]
fn xtypes_v2_sequence_test_suite_string_string10_check() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="string_unbounded"   extensibility="final">
                    <member name="x1"   type="string"   />
                </struct>
                <struct name="string10"   extensibility="final">
                    <member name="x1"   type="string" stringMaxLength="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::string_unbounded",
        vec![],
    )
    .unwrap()
    .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::string_unbounded",
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
        .create_datawriter::<DynamicData<'static>>(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::string10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::string10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_string_bounds = false;
    let _reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'seq(str20,10)_seq(str10,10)_check' : {
///     'common_args' : ['--type-folder types --type-file sequences'],
///     'apps' : ['pub-exe -P -t test -y Test::string20x10 --data-folder data --data-file array_string_10',
///               'sub-exe -S -t test -y Test::string10x10 --data-folder data --data-file array_string_10 --ignore-str-bounds f'],
///     'expected_codes' : [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'No type assignability between string20x10 and string10x10 (subscriber with ignore_str_bounds false)',
///     'description' : 'Verifies sequences of strings where publisher string bound exceeds subscriber string bound:\n\n'
///                     ' * Publisher uses `string20x10` from `sequences`.\n'
///                     ' * Subscriber uses `string10x10` from `sequences`.\n'
///                     ' * Both are `sequence<string, 10>`.\n'
///                     ' * Publisher string bound is 20, subscriber is 10.\n'
///                     ' * Subscriber sets `--ignore-str-bounds` to `false`.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// }
#[test]
fn xtypes_v2_sequence_test_suite_seq_str20_10_seq_str10_10_check() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="string20x10"   extensibility="final">
                    <member name="x1"   type="string" stringMaxLength="20" sequenceMaxLength="10"  />
                </struct>
                <struct name="string10x10"   extensibility="final">
                    <member name="x1"   type="string" stringMaxLength="10" sequenceMaxLength="10"  />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::string20x10", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::string20x10",
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
        .create_datawriter::<DynamicData<'static>>(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::string10x10", vec![])
            .unwrap()
            .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::string10x10",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let mut reader_qos = reader_qos();
    reader_qos.type_consistency.ignore_string_bounds = false;
    let _reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'wstring_wstring' : {
///     'common_args' : ['--type-folder types --type-file strings'],
///     'apps' : ['pub-exe -P -t test -y Test::wstring_unbounded --data-folder data --data-file wstrings',
///               'sub-exe -S -t test -y Test::wstring_unbounded --data-folder data --data-file wstrings'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.OK],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Communication between identical wstring_unbounded',
///     'description' : 'Verifies identical unbounded wide strings communicate:\n\n'
///                     ' * Publisher and Subscriber use `wstring_unbounded` from `strings`.\n'
///                     ' * Both use unbounded `wstring` type.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_string_test_suite_wstring_wstring() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="wstring_unbounded"   extensibility="final">
                    <member name="x1"   type="wstring"   />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::wstring_unbounded",
        vec![],
    )
    .unwrap()
    .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::wstring_unbounded",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type.clone(),
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::wstring_unbounded",
        vec![],
    )
    .unwrap()
    .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::wstring_unbounded",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_condition = writer.get_statuscondition();
    writer_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_condition))
        .unwrap();
    let reader_condition = reader.get_statuscondition();
    reader_condition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_condition))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<wstring_unbounded>
            <x1>Hello world</x1>
        </wstring_unbounded>",
    )
    .unwrap();

    writer.write(data, None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let sample = reader.read_next_sample().unwrap().data.unwrap();
    assert_eq!(sample.get_string_value(0).unwrap(), "Hello world");
}

/// 'struct_final_appendable': {
///     'common_args': ['--type-folder types --type-file primitives'],
///     'apps': ['pub-exe -P -t test -y Test::struct_primitives_final --data-folder data --data-file struct_primitives',
///              'sub-exe -S -t test -y Test::struct_primitives_appendable --data-folder data --data-file struct_primitives'],
///     'expected_codes': [ReturnCode.INCONSISTENT_TOPIC, ReturnCode.INCONSISTENT_TOPIC],
///     'check_function': tsf.data_is_correct,
///     'title' : 'No type assignability between struct_primitives_final and struct_primitives_appendable',
///     'description' : 'Verifies structs with mismatched extensibility are not assignable:\n\n'
///                     ' * Publisher uses `struct_primitives_final` (final) from `primitives`.\n'
///                     ' * Subscriber uses `struct_primitives_appendable` (appendable) from `primitives`.\n'
///                     ' * Publisher is `final`.\n'
///                     ' * Subscriber is `appendable`.\n'
///                     ' * Extensibility must match for assignability.\n'
///                     '**Test passes if:** Discovery fails due to type incompatibility.\n'
/// }
#[test]
fn xtypes_v2_struct_test_suite_struct_final_appendable() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <struct name="struct_primitives_final"   extensibility="final">
                    <member name="x1"   type="uint8"   />
                    <member name="x2"   type="uint16"  />
                    <member name="x3"   type="uint32"  />
                    <member name="x4"   type="uint64"  />
                    <member name="x5"   type="int8"    />
                    <member name="x6"   type="int16"   />
                    <member name="x7"   type="int32"   />
                    <member name="x8"   type="int64"   />
                    <member name="x9"   type="boolean" />
                    <member name="x10"  type="float32" />
                    <member name="x11"  type="float64" />
                    <member name="x12"  type="float128"/>
                    <member name="x13"  type="byte"    />
                    <member name="x14"  type="char8"   />
                </struct>
                <struct name="struct_primitives_appendable"   extensibility="appendable">
                    <member name="x1"   type="uint8"   />
                    <member name="x2"   type="uint16"  />
                    <member name="x3"   type="uint32"  />
                    <member name="x4"   type="uint64"  />
                    <member name="x5"   type="int8"   />
                    <member name="x6"   type="int16"   />
                    <member name="x7"   type="int32"   />
                    <member name="x8"   type="int64"   />
                    <member name="x9"   type="boolean" />
                    <member name="x10"  type="float32" />
                    <member name="x11"  type="float64" />
                    <member name="x12"  type="float128"/>
                    <member name="x13"  type="byte"    />
                    <member name="x14"  type="char8"   />
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::struct_primitives_final",
        vec![],
    )
    .unwrap()
    .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_primitives_final",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::struct_primitives_appendable",
        vec![],
    )
    .unwrap()
    .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_primitives_appendable",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();

    let status_cond_publisher = publisher_topic.get_statuscondition();
    status_cond_publisher
        .set_enabled_statuses(&[StatusKind::InconsistentTopic])
        .unwrap();
    let mut wait_set_publisher = WaitSet::new();
    wait_set_publisher
        .attach_condition(Condition::StatusCondition(status_cond_publisher))
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

/// 'tryc_enum_1' : {
///     'common_args' : ['--type-folder types --type-file try_construct'],
///     'apps' : ['pub-exe -P -t test -y Test::struct_enum_1 --data-folder data --data-file tryconstruct/enum_val3',
///               'sub-exe -S -t test -y Test::struct_enum_2_discard --data-folder data --data-file tryconstruct/enum_val1'],
///     'expected_codes' : [ReturnCode.OK, ReturnCode.DATA_NOT_RECEIVED],
///     'check_function' : tsf.data_is_correct,
///     'title' : 'Type assignability between struct_enum_1 and struct_enum_2_discard but sample rejected',
///     'description' : 'Verifies enum with `@try_construct(discard)` rejects unrepresentable literals:\n\n'
///                     ' * Publisher uses `struct_enum_1` from `try_construct`.\n'
///                     ' * Subscriber uses `struct_enum_2_discard` from `try_construct`.\n'
///                     ' * Publisher uses `E1` (4 literals: VAL0-VAL3).\n'
///                     ' * Subscriber uses `E2` (3 literals: VAL0-VAL2) with `@try_construct(discard)`.\n'
///                     ' * Literal `VAL3` is not in `E2`, so the sample is discarded.\n'
///                     '**Test passes if:** Discovery succeeds but the sample is not delivered.\n'
/// }
#[test]
fn xtypes_v2_tryconstruct_test_suite_tryc_enum_1() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <enum name="E1" bitBound="32" extensibility="appendable">
                    <enumerator name="VAL0" value="0"/>
                    <enumerator name="VAL1" value="1"/>
                    <enumerator name="VAL2" value="2"/>
                    <enumerator name="VAL3" value="3"/>
                </enum>
                <enum name="E2" bitBound="32" extensibility="appendable">
                    <enumerator name="VAL0" value="0"/>
                    <enumerator name="VAL1" value="1" defaultLiteral="true"/>
                    <enumerator name="VAL2" value="2"/>
                </enum>
                <struct name="struct_enum_1" extensibility="mutable">
                    <member name="x1" type="nonBasic" nonBasicTypeName="E1" />
                </struct>
                <struct name="struct_enum_2_discard" extensibility="mutable">
                    <member name="x1" type="nonBasic" nonBasicTypeName="E2" tryConstruct="discard"/>
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_enum_1", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_enum_1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type.clone(),
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_dynamic_type = DynamicTypeBuilderFactory::create_type_w_document(
        type_xml,
        "Test::struct_enum_2_discard",
        vec![],
    )
    .unwrap()
    .build();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_enum_2_discard",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_condition = writer.get_statuscondition();
    writer_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_condition))
        .unwrap();
    let reader_condition = reader.get_statuscondition();
    reader_condition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_condition))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let mut data = DynamicDataFactory::create_data(publisher_dynamic_type);
    data.from_xml(
        "<struct>
            <x1>VAL3</x1>
        </struct>",
    )
    .unwrap();

    writer.write(data, None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    assert!(reader.read_next_sample().unwrap().data.is_none());
}

/// 'struct_different_ids_ok': {
///     'common_args': ['--type-folder types --type-file struct_names'],
///     'apps': ['pub-exe -P -t test -y Test::struct_1 --data-folder data --data-file struct_num_x1_x5',
///              'sub-exe -S -t test -y Test::struct_2 --data-folder data --data-file struct_num_x5'],
///     'expected_codes': [ReturnCode.OK, ReturnCode.OK],
///     'check_function': tsf.data_is_correct,
///     'title' : 'Communication between struct_1 and struct_2',
///     'description' : 'Verifies mutable structs where member names match but IDs differ are assignable by default:\n\n'
///                     ' * Publisher uses `struct_1` from `struct_names`.\n'
///                     ' * Subscriber uses `struct_2` from `struct_names`.\n'
///                     ' * Both have member `x1` but with different IDs (id=1 in publisher, id=2 in subscriber). Both share member `x5` (id=5). By default, `ignore_member_names` is true so ID matching is used.\n'
///                     '**Test passes if:** Discovery succeeds and the subscriber receives the sample.\n'
/// }
#[test]
fn xtypes_v2_struct_test_suite_struct_different_ids_ok() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let publisher_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let type_xml = r#"
    <dds>
        <types>
            <module name="Test">
                <!-- names match, ids don't -->
                <struct name="struct_1"   extensibility="mutable">
                    <member name="x1" type="int32" id="1"  />
                    <member name="x5" type="int32" id="5"  />  <!-- so we have a member in common -->
                </struct>
                <struct name="struct_2"   extensibility="mutable">
                    <member name="x1" type="int32" id="2"  />
                    <member name="x5" type="int32" id="5"  /> 
                </struct>
            </module>
        </types>
    </dds>
    "#;
    let publisher_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_1", vec![])
            .unwrap()
            .build();
    let subscriber_dynamic_type =
        DynamicTypeBuilderFactory::create_type_w_document(type_xml, "Test::struct_2", vec![])
            .unwrap()
            .build();
    let publisher_topic = publisher_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_1",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            publisher_dynamic_type.clone(),
        )
        .unwrap();
    let publisher = publisher_participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let writer = publisher
        .create_datawriter(
            &publisher_topic,
            QosKind::Specific(writer_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let subscriber_participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber_topic = subscriber_participant
        .create_dynamic_topic(
            "test",
            "Test::struct_2",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
            subscriber_dynamic_type,
        )
        .unwrap();
    let subscriber = subscriber_participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let reader = subscriber
        .create_datareader::<DynamicData<'static>>(
            &subscriber_topic,
            QosKind::Specific(reader_qos()),
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let writer_condition = writer.get_statuscondition();
    writer_condition
        .set_enabled_statuses(&[StatusKind::PublicationMatched])
        .unwrap();
    let mut writer_wait_set = WaitSet::new();
    writer_wait_set
        .attach_condition(Condition::StatusCondition(writer_condition))
        .unwrap();
    let reader_condition = reader.get_statuscondition();
    reader_condition
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(reader_condition))
        .unwrap();
    writer_wait_set.wait(Duration::new(10, 0)).unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let mut publisher_data = DynamicDataFactory::create_data(publisher_dynamic_type);
    publisher_data
        .from_xml(
            "<struct>
            <x1>1</x1>
            <x5>5</x5>
        </struct>",
        )
        .unwrap();

    let mut subscriber_data = DynamicDataFactory::create_data(subscriber_dynamic_type);
    subscriber_data
        .from_xml(
            "<struct>
            <x5>5</x5>
        </struct>",
        )
        .unwrap();

    writer.write(publisher_data, None).unwrap();
    writer
        .wait_for_acknowledgments(Duration::new(10, 0))
        .unwrap();

    let sample = reader.read_next_sample().unwrap().data.unwrap();
    assert_eq!(sample, subscriber_data);
}
