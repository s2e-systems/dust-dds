use dust_dds_wasm::{DomainParticipantFactory, DynamicTypeBuilderFactory};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_subscriber_properties() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(30, None)
        .await
        .expect("Failed to create participant");

    let subscriber = participant
        .create_subscriber(None)
        .await
        .expect("Failed to create subscriber");

    assert_eq!(subscriber.get_instance_handle().value().len(), 16);
    let parent_participant = subscriber.get_participant();
    assert_eq!(parent_participant.get_domain_id(), 30);
    assert!(
        parent_participant
            .get_instance_handle()
            .equals(&participant.get_instance_handle())
    );

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
async fn test_subscriber_qos() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(31, None)
        .await
        .expect("Failed to create participant");

    let subscriber = participant
        .create_subscriber(None)
        .await
        .expect("Failed to create subscriber");

    let qos = subscriber.get_qos().await.expect("Failed to get qos");
    subscriber
        .set_qos(Some(qos))
        .await
        .expect("Failed to set qos");
    subscriber.set_qos(None).await.expect("Failed to reset qos");

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
async fn test_subscriber_default_datareader_qos() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(32, None)
        .await
        .expect("Failed to create participant");

    let subscriber = participant
        .create_subscriber(None)
        .await
        .expect("Failed to create subscriber");

    let default_reader_qos = subscriber
        .get_default_datareader_qos()
        .await
        .expect("Failed to get default datareader qos");
    subscriber
        .set_default_datareader_qos(Some(default_reader_qos))
        .await
        .expect("Failed to set default datareader qos");
    subscriber
        .set_default_datareader_qos(None)
        .await
        .expect("Failed to reset default datareader qos");

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
async fn test_subscriber_datareader_lifecycle() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(33, None)
        .await
        .expect("Failed to create participant");

    let subscriber = participant
        .create_subscriber(None)
        .await
        .expect("Failed to create subscriber");

    let dynamic_type = DynamicTypeBuilderFactory::create_structure_type("SubTestType")
        .build()
        .expect("Build dynamic type");
    let topic = participant
        .create_topic("SubTestTopic", &dynamic_type, None)
        .await
        .expect("Failed to create topic");

    let reader = subscriber
        .create_datareader(&topic, None)
        .await
        .expect("Failed to create datareader");

    subscriber
        .delete_datareader(&reader)
        .await
        .expect("Failed to delete datareader");

    participant
        .delete_subscriber(&subscriber)
        .await
        .expect("Failed to delete subscriber");
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
async fn test_subscriber_access() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(34, None)
        .await
        .expect("Failed to create participant");

    let subscriber = participant
        .create_subscriber(None)
        .await
        .expect("Failed to create subscriber");

    subscriber
        .begin_access()
        .await
        .expect("Failed to begin access");
    subscriber.end_access().await.expect("Failed to end access");

    participant
        .delete_subscriber(&subscriber)
        .await
        .expect("Failed to delete subscriber");
    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}
