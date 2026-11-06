use dust_dds_wasm::DomainParticipantFactoryAsync;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_create_participant() {
    let factory = DomainParticipantFactoryAsync::new();
    let participant = factory.create_participant(0).await;
    assert!(participant.is_ok());
}

#[wasm_bindgen_test]
async fn test_create_and_delete_participant() {
    let factory = DomainParticipantFactoryAsync::get_instance();
    let participant = factory
        .create_participant(1)
        .await
        .expect("Failed to create participant");
    let delete_result = factory.delete_participant(&participant).await;
    assert!(delete_result.is_ok());
}
