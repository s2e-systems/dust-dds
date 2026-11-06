use dust_dds_wasm::{DomainParticipantFactory, DynamicTypeBuilderFactory};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_publisher_properties() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(20, None)
        .await
        .expect("Failed to create participant");

    let publisher = participant
        .create_publisher(None)
        .await
        .expect("Failed to create publisher");

    assert_eq!(publisher.get_instance_handle().value().len(), 16);
    let parent_participant = publisher.get_participant();
    assert_eq!(parent_participant.get_domain_id(), 20);
    assert!(
        parent_participant
            .get_instance_handle()
            .equals(&participant.get_instance_handle())
    );

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
async fn test_publisher_qos() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(21, None)
        .await
        .expect("Failed to create participant");

    let publisher = participant
        .create_publisher(None)
        .await
        .expect("Failed to create publisher");

    let qos = publisher.get_qos().await.expect("Failed to get qos");
    publisher
        .set_qos(Some(qos))
        .await
        .expect("Failed to set qos");
    publisher.set_qos(None).await.expect("Failed to reset qos");

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
async fn test_publisher_default_datawriter_qos() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(22, None)
        .await
        .expect("Failed to create participant");

    let publisher = participant
        .create_publisher(None)
        .await
        .expect("Failed to create publisher");

    let default_writer_qos = publisher
        .get_default_datawriter_qos()
        .await
        .expect("Failed to get default datawriter qos");
    publisher
        .set_default_datawriter_qos(Some(default_writer_qos))
        .await
        .expect("Failed to set default datawriter qos");
    publisher
        .set_default_datawriter_qos(None)
        .await
        .expect("Failed to reset default datawriter qos");

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
async fn test_publisher_datawriter_lifecycle() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(23, None)
        .await
        .expect("Failed to create participant");

    let publisher = participant
        .create_publisher(None)
        .await
        .expect("Failed to create publisher");

    let dynamic_type = DynamicTypeBuilderFactory::create_structure_type("PubTestType")
        .build()
        .expect("Build dynamic type");
    let topic = participant
        .create_topic("PubTestTopic", &dynamic_type, None)
        .await
        .expect("Failed to create topic");

    let writer = publisher
        .create_datawriter(&topic, None)
        .await
        .expect("Failed to create datawriter");

    publisher
        .delete_datawriter(&writer)
        .await
        .expect("Failed to delete datawriter");

    participant
        .delete_publisher(&publisher)
        .await
        .expect("Failed to delete publisher");
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
async fn test_publisher_coherent_changes() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(24, None)
        .await
        .expect("Failed to create participant");

    let publisher = participant
        .create_publisher(None)
        .await
        .expect("Failed to create publisher");

    publisher
        .begin_coherent_changes()
        .await
        .expect("Failed to begin coherent changes");
    publisher
        .end_coherent_changes()
        .await
        .expect("Failed to end coherent changes");

    participant
        .delete_publisher(&publisher)
        .await
        .expect("Failed to delete publisher");
    factory
        .delete_participant(&participant)
        .await
        .expect("Failed to delete participant");
}
