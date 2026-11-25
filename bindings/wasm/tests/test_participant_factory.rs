use dust_dds_wasm::{
    DomainParticipantFactory, DomainParticipantFactoryQos, DomainParticipantQos,
    EntityFactoryQosPolicy, UserDataQosPolicy,
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_create_participant() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory.create_participant(0, None, None, None).await;
    assert!(participant.is_ok());
}

#[wasm_bindgen_test]
async fn test_create_and_delete_participant() {
    let factory = DomainParticipantFactory::get_instance(None);
    let participant = factory
        .create_participant(1, None, None, None)
        .await
        .expect("Failed to create participant");
    let delete_result = factory.delete_participant(&participant).await;
    assert!(delete_result.is_ok());
}

#[wasm_bindgen_test]
async fn test_lookup_participant() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(2, None, None, None)
        .await
        .expect("Failed to create participant");

    let looked_up = factory
        .lookup_participant(2)
        .await
        .expect("Lookup should succeed");
    assert!(looked_up.is_some());

    factory
        .delete_participant(&participant)
        .await
        .expect("Delete should succeed");

    let looked_up_after_delete = factory
        .lookup_participant(2)
        .await
        .expect("Lookup should succeed");
    assert!(looked_up_after_delete.is_none());
}

#[wasm_bindgen_test]
async fn test_default_participant_qos() {
    let factory = DomainParticipantFactory::new(None);
    let default_qos = factory
        .get_default_participant_qos()
        .await
        .expect("Failed to get default participant qos");

    let new_user_data = UserDataQosPolicy::new(Some(vec![1, 2, 3, 4]));
    let custom_qos =
        DomainParticipantQos::new(Some(new_user_data), Some(default_qos.entity_factory()));

    factory
        .set_default_participant_qos(Some(custom_qos))
        .await
        .expect("Failed to set default participant qos");

    let retrieved_qos = factory
        .get_default_participant_qos()
        .await
        .expect("Failed to get updated default participant qos");
    assert_eq!(retrieved_qos.user_data().value(), vec![1, 2, 3, 4]);

    // Reset to default
    factory
        .set_default_participant_qos(None)
        .await
        .expect("Failed to reset default participant qos");
}

#[wasm_bindgen_test]
async fn test_factory_qos() {
    let factory = DomainParticipantFactory::new(None);
    let current_qos = factory.get_qos().await.expect("Failed to get factory qos");
    assert!(current_qos.entity_factory().autoenable_created_entities);

    let entity_factory = EntityFactoryQosPolicy::new(Some(false));
    let new_qos = DomainParticipantFactoryQos::new(Some(entity_factory));

    factory
        .set_qos(Some(new_qos))
        .await
        .expect("Failed to set factory qos");

    let updated_qos = factory
        .get_qos()
        .await
        .expect("Failed to get updated factory qos");
    assert!(!updated_qos.entity_factory().autoenable_created_entities);

    // Reset to default
    factory
        .set_qos(None)
        .await
        .expect("Failed to reset factory qos");
    let reset_qos = factory
        .get_qos()
        .await
        .expect("Failed to get reset factory qos");
    assert!(reset_qos.entity_factory().autoenable_created_entities);
}
