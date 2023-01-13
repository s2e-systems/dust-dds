use dust_dds::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::QosKind, status::NO_STATUS},
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[test]
fn builtin_reader_access() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber().unwrap();

    assert!(builtin_subscriber
        .lookup_datareader::<ParticipantBuiltinTopicData>("DCPSParticipant")
        .is_ok());

    assert!(builtin_subscriber
        .lookup_datareader::<TopicBuiltinTopicData>("DCPSTopic")
        .is_ok());

    assert!(builtin_subscriber
        .lookup_datareader::<PublicationBuiltinTopicData>("DCPSPublication")
        .is_ok());

    assert!(builtin_subscriber
        .lookup_datareader::<SubscriptionBuiltinTopicData>("DCPSSubscription")
        .is_ok());
}

#[test]
fn get_discovery_data_from_builtin_reader() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let participant = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let _participant2 = DomainParticipantFactory::get_instance()
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .unwrap();

    let builtin_subscriber = participant.get_builtin_subscriber().unwrap();

    let participants_reader = builtin_subscriber
        .lookup_datareader::<ParticipantBuiltinTopicData>("DCPSParticipant")
        .unwrap()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(2000));

    let participant_samples = participants_reader
        .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
        .unwrap();

    let topics_reader = builtin_subscriber
        .lookup_datareader::<TopicBuiltinTopicData>("DCPSTopic")
        .unwrap();

    let publications_reader = builtin_subscriber
        .lookup_datareader::<PublicationBuiltinTopicData>("DCPSPublication")
        .unwrap();

    let subscriptions_reader = builtin_subscriber
        .lookup_datareader::<SubscriptionBuiltinTopicData>("DCPSSubscription")
        .unwrap();
}
