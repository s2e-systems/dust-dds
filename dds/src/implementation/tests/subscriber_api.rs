use crate::dds_type::{DdsDeserialize, DdsType};
use crate::implementation::dds_impl::domain_participant_impl::DomainParticipantImpl;
use crate::implementation::rtps::participant::RtpsParticipant;
use crate::implementation::rtps::types::{GuidPrefix, PROTOCOLVERSION, VENDOR_ID_S2E};
use crate::return_type::{DdsError, DdsResult};
use crate::{dcps_psm::DomainId, infrastructure::qos::DomainParticipantQos};

struct Foo;

impl DdsType for Foo {
    fn type_name() -> &'static str {
        "Foo"
    }
}

impl<'de> DdsDeserialize<'de> for Foo {
    fn deserialize(_buf: &mut &'de [u8]) -> DdsResult<Self> {
        todo!()
    }
}

struct Bar;

impl DdsType for Bar {
    fn type_name() -> &'static str {
        "Bar"
    }
}

impl<'de> DdsDeserialize<'de> for Bar {
    fn deserialize(_buf: &mut &'de [u8]) -> DdsResult<Self> {
        todo!()
    }
}

#[test]
fn create_and_delete_datareader_succeeds() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
    );

    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_reader = subscriber
        .create_datareader::<Foo>(&topic, None, None, 0, &domain_participant)
        .unwrap();

    subscriber.delete_datareader(&data_reader).unwrap();
}

#[test]
fn delete_datareader_from_other_subscriber_returns_error() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
    );

    let subscriber1 = domain_participant.create_subscriber(None, None, 0).unwrap();
    let subscriber2 = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_reader = subscriber1
        .create_datareader::<Foo>(&topic, None, None, 0, &domain_participant)
        .unwrap();

    assert!(matches!(
        subscriber2.delete_datareader(&data_reader),
        Err(DdsError::PreconditionNotMet(_))
    ));
}

#[test]
fn lookup_datareader_without_readers_created() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
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
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
        vec![],
        vec![],
    );
    let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();
    let topic = domain_participant
        .create_topic::<Foo>("topic", None, None, 0)
        .unwrap();

    let data_reader = subscriber
        .create_datareader::<Foo>(&topic, None, None, 0, &domain_participant)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic).unwrap() == data_reader);
}

#[test]
fn lookup_datareader_with_one_datareader_created_and_wrong_type() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
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
        .create_datareader::<Bar>(&topic_bar, None, None, 0, &domain_participant)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic_bar).is_err());
}

#[test]
fn lookup_datareader_with_one_datareader_created_and_wrong_topic() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
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
        .create_datareader::<Bar>(&topic_bar, None, None, 0, &domain_participant)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Bar>(&topic_foo).is_err());
}

#[test]
fn lookup_datareader_with_two_datareaders_with_different_types() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
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
        .create_datareader::<Foo>(&topic_foo, None, None, 0, &domain_participant)
        .unwrap();
    let data_reader_bar = subscriber
        .create_datareader::<Bar>(&topic_bar, None, None, 0, &domain_participant)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic_foo).unwrap() == data_reader_foo);

    assert!(subscriber.lookup_datareader::<Bar>(&topic_bar).unwrap() == data_reader_bar);
}

#[test]
fn lookup_datareader_with_two_datareaders_with_different_topics() {
    let rtps_participant = RtpsParticipant::new(
        GuidPrefix([1; 12]),
        &[],
        &[],
        PROTOCOLVERSION,
        VENDOR_ID_S2E,
    );
    let domain_participant = DomainParticipantImpl::new(
        rtps_participant,
        DomainId::default(),
        "".to_string(),
        DomainParticipantQos::default(),
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
        .create_datareader::<Foo>(&topic1, None, None, 0, &domain_participant)
        .unwrap();
    let data_reader2 = subscriber
        .create_datareader::<Foo>(&topic2, None, None, 0, &domain_participant)
        .unwrap();

    assert!(subscriber.lookup_datareader::<Foo>(&topic1).unwrap() == data_reader1);
    assert!(subscriber.lookup_datareader::<Foo>(&topic2).unwrap() == data_reader2);
}
