use crate::dds_type::DdsType;
use crate::implementation::dds_impl::domain_participant_impl::DomainParticipantImpl;
use crate::implementation::rtps::types::GuidPrefix;
use crate::return_type::DdsError;
use crate::{dcps_psm::DomainId, infrastructure::qos::DomainParticipantQos};

struct Foo;

impl DdsType for Foo {
    fn type_name() -> &'static str {
        "Foo"
    }
}

struct Bar;

impl DdsType for Bar {
    fn type_name() -> &'static str {
        "Bar"
    }
}

#[test]
fn create_and_delete_datawriter_succeeds() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<Foo>(&topic, None, None, 0, &domain_participant)
        .unwrap();

    publisher.delete_datawriter(&data_writer).unwrap();
}

#[test]
fn delete_datawriter_from_other_publisher_returns_error() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let publisher1 = domain_participant.create_publisher(None, None, 0).unwrap();
    let publisher2 = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher1
        .create_datawriter::<Foo>(&topic, None, None, 0, &domain_participant)
        .unwrap();

    assert!(matches!(
        publisher2.delete_datawriter(&data_writer),
        Err(DdsError::PreconditionNotMet(_))
    ));
}

#[test]
fn lookup_datawriter_without_writers_created() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    assert!(publisher.lookup_datawriter::<Foo>(&topic).is_err());
}

#[test]
fn lookup_datawriter_with_one_datawriter_created() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_writer = publisher
        .create_datawriter::<Foo>(&topic, None, None, 0, &domain_participant)
        .unwrap();

    assert!(publisher.lookup_datawriter::<Foo>(&topic).unwrap() == data_writer);
}

#[test]
fn lookup_datawriter_with_one_datawriter_created_and_wrong_type() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let _topic_foo = domain_participant
        .create_topic::<Foo>("topic_foo", None, None, 0)
        .unwrap();
    let topic_bar = domain_participant
        .create_topic::<Bar>("topic_bar", None, None, 0)
        .unwrap();

    publisher
        .create_datawriter::<Bar>(&topic_bar, None, None, 0, &domain_participant)
        .unwrap();

    assert!(publisher.lookup_datawriter::<Foo>(&topic_bar).is_err());
}

#[test]
fn lookup_datawriter_with_one_datawriter_created_and_wrong_topic() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic_foo = domain_participant
        .create_topic::<Foo>("topic_foo", None, None, 0)
        .unwrap();
    let topic_bar = domain_participant
        .create_topic::<Bar>("topic_bar", None, None, 0)
        .unwrap();

    publisher
        .create_datawriter::<Bar>(&topic_bar, None, None, 0, &domain_participant)
        .unwrap();

    assert!(publisher.lookup_datawriter::<Bar>(&topic_foo).is_err());
}

#[test]
fn lookup_datawriter_with_two_datawriters_with_different_types() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic_foo = domain_participant
        .create_topic::<Foo>("topic_foo", None, None, 0)
        .unwrap();
    let topic_bar = domain_participant
        .create_topic::<Bar>("topic_bar", None, None, 0)
        .unwrap();

    let data_writer_foo = publisher
        .create_datawriter::<Foo>(&topic_foo, None, None, 0, &domain_participant)
        .unwrap();
    let data_writer_bar = publisher
        .create_datawriter::<Bar>(&topic_bar, None, None, 0, &domain_participant)
        .unwrap();

    assert!(publisher.lookup_datawriter::<Foo>(&topic_foo).unwrap() == data_writer_foo);

    assert!(publisher.lookup_datawriter::<Bar>(&topic_bar).unwrap() == data_writer_bar);
}

#[test]
fn lookup_datawriter_with_two_datawriters_with_different_topics() {
    let domain_participant = DomainParticipantImpl::new(
        GuidPrefix([1; 12]),
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
    let topic1 = domain_participant
        .create_topic::<Foo>("topic1", None, None, 0)
        .unwrap();
    let topic2 = domain_participant
        .create_topic::<Foo>("topic2", None, None, 0)
        .unwrap();

    let data_writer1 = publisher
        .create_datawriter::<Foo>(&topic1, None, None, 0, &domain_participant)
        .unwrap();
    let data_writer2 = publisher
        .create_datawriter::<Foo>(&topic2, None, None, 0, &domain_participant)
        .unwrap();

    assert!(publisher.lookup_datawriter::<Foo>(&topic1).unwrap() == data_writer1);
    assert!(publisher.lookup_datawriter::<Foo>(&topic2).unwrap() == data_writer2);
}
