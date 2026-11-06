use dust_dds_wasm::{
    DomainParticipantFactory, DomainParticipantQos, DynamicTypeBuilderFactory,
    EntityFactoryQosPolicy, InstanceHandle, PublisherQos, SubscriberQos, TopicQos,
    UserDataQosPolicy,
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_participant_properties() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(10, None)
        .await
        .expect("Failed to create participant");

    assert_eq!(participant.get_domain_id(), 10);
    let handle = participant.get_instance_handle();
    assert_eq!(handle.value().len(), 16);
    assert_eq!(handle.to_bytes().len(), 16);

    let recreated = InstanceHandle::new(Some(handle.value())).expect("Valid handle bytes");
    assert!(handle.equals(&recreated));

    let nil_handle = InstanceHandle::new(None).expect("Valid nil handle");
    assert!(!handle.equals(&nil_handle));

    // Test ignore operations
    participant
        .ignore_participant(&handle)
        .await
        .expect("Ignore participant");
    participant
        .ignore_publication(&handle)
        .await
        .expect("Ignore publication");
    participant
        .ignore_subscription(&handle)
        .await
        .expect("Ignore subscription");

    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}

#[wasm_bindgen_test]
async fn test_participant_qos() {
    let factory = DomainParticipantFactory::new(None);
    let custom_qos = DomainParticipantQos::new(
        Some(UserDataQosPolicy::new(Some(vec![42]))),
        Some(EntityFactoryQosPolicy::new(Some(true))),
    );
    let participant = factory
        .create_participant(11, Some(custom_qos))
        .await
        .expect("Failed to create participant");

    let current_qos = participant.get_qos().await.expect("Failed to get qos");
    participant
        .set_qos(Some(current_qos))
        .await
        .expect("Failed to set qos");

    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}

#[wasm_bindgen_test]
async fn test_publisher_lifecycle() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(12, None)
        .await
        .expect("Failed to create participant");

    let default_qos = participant
        .get_default_publisher_qos()
        .await
        .expect("Failed to get default publisher qos");
    participant
        .set_default_publisher_qos(Some(default_qos))
        .await
        .expect("Failed to set default publisher qos");

    let custom_pub_qos = PublisherQos::new();
    let publisher = participant
        .create_publisher(Some(custom_pub_qos))
        .await
        .expect("Failed to create publisher");
    assert_eq!(publisher.get_instance_handle().value().len(), 16);

    participant
        .delete_publisher(&publisher)
        .await
        .expect("Failed to delete publisher");

    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}

#[wasm_bindgen_test]
async fn test_subscriber_lifecycle() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(13, None)
        .await
        .expect("Failed to create participant");

    let default_qos = participant
        .get_default_subscriber_qos()
        .await
        .expect("Failed to get default subscriber qos");
    participant
        .set_default_subscriber_qos(Some(default_qos))
        .await
        .expect("Failed to set default subscriber qos");

    let custom_sub_qos = SubscriberQos::new();
    let subscriber = participant
        .create_subscriber(Some(custom_sub_qos))
        .await
        .expect("Failed to create subscriber");
    assert_eq!(subscriber.get_instance_handle().value().len(), 16);

    let builtin_subscriber = participant.get_builtin_subscriber();
    assert_eq!(builtin_subscriber.get_instance_handle().value().len(), 16);

    participant
        .delete_subscriber(&subscriber)
        .await
        .expect("Failed to delete subscriber");

    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}

#[wasm_bindgen_test]
async fn test_topic_lifecycle() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(14, None)
        .await
        .expect("Failed to create participant");

    let default_qos = participant
        .get_default_topic_qos()
        .await
        .expect("Failed to get default topic qos");
    participant
        .set_default_topic_qos(Some(default_qos))
        .await
        .expect("Failed to set default topic qos");

    let custom_topic_qos = TopicQos::new();
    let dynamic_type = DynamicTypeBuilderFactory::create_structure_type("HelloWorldType")
        .build()
        .expect("Build dynamic type");
    let topic = participant
        .create_topic("HelloWorldTopic", &dynamic_type, Some(custom_topic_qos))
        .await
        .expect("Failed to create topic");

    assert_eq!(topic.get_name(), "HelloWorldTopic");
    assert_eq!(topic.get_type_name(), "HelloWorldType");
    assert_eq!(topic.get_instance_handle().value().len(), 16);

    participant
        .delete_topic(&topic)
        .await
        .expect("Failed to delete topic");

    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}

#[wasm_bindgen_test]
async fn test_delete_contained_entities() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(15, None)
        .await
        .expect("Failed to create participant");

    let _publisher = participant
        .create_publisher(None)
        .await
        .expect("Failed to create publisher");
    let _subscriber = participant
        .create_subscriber(None)
        .await
        .expect("Failed to create subscriber");
    let temp_dynamic_type = DynamicTypeBuilderFactory::create_structure_type("TempType")
        .build()
        .expect("Build dynamic type");
    let _topic = participant
        .create_topic("TempTopic", &temp_dynamic_type, None)
        .await
        .expect("Failed to create topic");

    participant
        .delete_contained_entities()
        .await
        .expect("Failed to delete contained entities");

    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}

#[wasm_bindgen_test]
async fn test_discovered_entities() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(16, None)
        .await
        .expect("Failed to create participant");

    let participants = participant
        .get_discovered_participants()
        .await
        .expect("Failed to get discovered participants");
    assert_eq!(participants.length(), 1);

    let topics = participant
        .get_discovered_topics()
        .await
        .expect("Failed to get discovered topics");
    assert_eq!(topics.length(), 0);

    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}
