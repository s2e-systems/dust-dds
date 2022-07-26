use crate::dds_type::DdsType;
use crate::implementation::dds_impl::domain_participant_impl::DomainParticipantImpl;
use crate::return_type::DdsError;
use crate::{dcps_psm::DomainId, infrastructure::qos::DomainParticipantQos};
use rtps_pim::structure::types::GuidPrefix;

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
fn create_and_delete_datareader_succeeds() {
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

    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_reader = subscriber
        .create_datareader::<Foo>(&topic, None, None, 0)
        .unwrap();

    subscriber.delete_datareader(&data_reader).unwrap();
}

#[test]
fn delete_datareader_from_other_subscriber_returns_error() {
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

    let subscriber1 = domain_participant.create_subscriber(None, None, 0).unwrap();
    let subscriber2 = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_reader = subscriber1
        .create_datareader::<Foo>(&topic, None, None, 0)
        .unwrap();

    assert!(matches!(
        subscriber2.delete_datareader(&data_reader),
        Err(DdsError::PreconditionNotMet(_))
    ));
}

#[test]
fn lookup_datareader_without_readers_created() {
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
    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic).is_err());
}

#[test]
fn lookup_datareader_with_one_datareader_created() {
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
    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_reader = subscriber
        .create_datareader::<Foo>(&topic, None, None, 0)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic).unwrap() == data_reader);
}

#[test]
fn lookup_datareader_with_one_datareader_created_and_wrong_type() {
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
    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let _topic_foo = domain_participant
        .create_topic::<Foo>("topic_foo", None, None, 0)
        .unwrap();
    let topic_bar = domain_participant
        .create_topic::<Bar>("topic_bar", None, None, 0)
        .unwrap();

    subscriber
        .create_datareader::<Bar>(&topic_bar, None, None, 0)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic_bar).is_err());
}

#[test]
fn lookup_datareader_with_one_datareader_created_and_wrong_topic() {
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
    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic_foo = domain_participant
        .create_topic::<Foo>("topic_foo", None, None, 0)
        .unwrap();
    let topic_bar = domain_participant
        .create_topic::<Bar>("topic_bar", None, None, 0)
        .unwrap();

    subscriber
        .create_datareader::<Bar>(&topic_bar, None, None, 0)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Bar>(&topic_foo).is_err());
}

#[test]
fn lookup_datareader_with_two_datareaders_with_different_types() {
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
    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic_foo = domain_participant
        .create_topic::<Foo>("topic_foo", None, None, 0)
        .unwrap();
    let topic_bar = domain_participant
        .create_topic::<Bar>("topic_bar", None, None, 0)
        .unwrap();

    let data_reader_foo = subscriber
        .create_datareader::<Foo>(&topic_foo, None, None, 0)
        .unwrap();
    let data_reader_bar = subscriber
        .create_datareader::<Bar>(&topic_bar, None, None, 0)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic_foo).unwrap() == data_reader_foo);

    assert!(subscriber.lookup_datareader::<Bar>(&topic_bar).unwrap() == data_reader_bar);
}

#[test]
fn lookup_datareader_with_two_datareaders_with_different_topics() {
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
    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic1 = domain_participant
        .create_topic::<Foo>("topic1", None, None, 0)
        .unwrap();
    let topic2 = domain_participant
        .create_topic::<Foo>("topic2", None, None, 0)
        .unwrap();

    let data_reader1 = subscriber
        .create_datareader::<Foo>(&topic1, None, None, 0)
        .unwrap();
    let data_reader2 = subscriber
        .create_datareader::<Foo>(&topic2, None, None, 0)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic1).unwrap() == data_reader1);
    assert!(subscriber.lookup_datareader::<Foo>(&topic2).unwrap() == data_reader2);
}
