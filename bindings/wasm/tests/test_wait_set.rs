use dust_dds_wasm::{
    DomainParticipantFactory, DynamicTypeBuilderFactory, MemberDescriptor, StatusKind, TypeKind,
    WaitSet,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_entity_status_conditions() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(50, None, None, None)
        .await
        .expect("Create participant");

    let mut builder = DynamicTypeBuilderFactory::create_structure_type("TestStatusType");
    let u32_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::Uint32);
    builder
        .add_member(&MemberDescriptor::new(
            "id",
            &u32_type,
            Some(0),
            Some(true),
            None,
        ))
        .unwrap();
    let dynamic_type = builder.build().unwrap();

    let topic = participant
        .create_topic("TestStatusTopic", &dynamic_type, None, None, None)
        .await
        .expect("Create topic");

    let publisher = participant
        .create_publisher(None, None, None)
        .await
        .expect("Create publisher");
    let writer = publisher
        .create_datawriter(&topic, None, None, None)
        .await
        .expect("Create writer");

    let subscriber = participant
        .create_subscriber(None, None, None)
        .await
        .expect("Create subscriber");
    let reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    // Retrieve StatusConditions from all 4 entities
    let topic_cond = topic.get_status_condition();
    let sub_cond = subscriber.get_status_condition();
    let writer_cond = writer.get_status_condition();
    let reader_cond = reader.get_status_condition();

    // Verify trigger values can be queried
    assert_eq!(topic_cond.get_trigger_value().await.unwrap(), false);
    assert_eq!(sub_cond.get_trigger_value().await.unwrap(), false);
    let _ = writer_cond.get_trigger_value().await.unwrap();
    let _ = reader_cond.get_trigger_value().await.unwrap();

    // Verify equality
    assert!(topic_cond.equals(&topic.get_status_condition()));
    assert!(writer_cond.equals(&writer.get_status_condition()));
    assert!(reader_cond.equals(&reader.get_status_condition()));
    assert!(!topic_cond.equals(&writer_cond));

    // Conversion to Condition
    let cond = reader_cond.as_condition();
    assert!(cond.equals(&reader.get_status_condition().as_condition()));
    assert!(cond.as_status_condition().is_some());

    participant.delete_contained_entities().await.unwrap();
    factory.delete_participant(&participant).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_status_condition_enabled_statuses() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(51, None, None, None)
        .await
        .expect("Create participant");

    let mut builder = DynamicTypeBuilderFactory::create_structure_type("TestStatusType2");
    let u32_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::Uint32);
    builder
        .add_member(&MemberDescriptor::new(
            "id",
            &u32_type,
            Some(0),
            Some(true),
            None,
        ))
        .unwrap();
    let dynamic_type = builder.build().unwrap();

    let topic = participant
        .create_topic("TestStatusTopic2", &dynamic_type, None, None, None)
        .await
        .expect("Create topic");
    let subscriber = participant
        .create_subscriber(None, None, None)
        .await
        .expect("Create subscriber");
    let reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    let reader_cond = reader.get_status_condition();

    // Default enabled statuses contains all statuses
    let default_statuses = reader_cond.get_enabled_statuses().await.unwrap();
    assert_eq!(default_statuses.length(), 13);

    // Set enabled statuses to only DataAvailable
    let mask = js_sys::Array::new();
    mask.push(&JsValue::from(StatusKind::DataAvailable));
    reader_cond.set_enabled_statuses(&mask).await.unwrap();

    let updated_statuses = reader_cond.get_enabled_statuses().await.unwrap();
    assert_eq!(updated_statuses.length(), 1);

    participant.delete_contained_entities().await.unwrap();
    factory.delete_participant(&participant).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_wait_set_attach_detach() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(52, None, None, None)
        .await
        .expect("Create participant");

    let mut builder = DynamicTypeBuilderFactory::create_structure_type("TestStatusType3");
    let u32_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::Uint32);
    builder
        .add_member(&MemberDescriptor::new(
            "id",
            &u32_type,
            Some(0),
            Some(true),
            None,
        ))
        .unwrap();
    let dynamic_type = builder.build().unwrap();

    let topic = participant
        .create_topic("TestStatusTopic3", &dynamic_type, None, None, None)
        .await
        .expect("Create topic");
    let publisher = participant
        .create_publisher(None, None, None)
        .await
        .expect("Create publisher");
    let writer = publisher
        .create_datawriter(&topic, None, None, None)
        .await
        .expect("Create writer");
    let subscriber = participant
        .create_subscriber(None, None, None)
        .await
        .expect("Create subscriber");
    let reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    let wait_set = WaitSet::new();
    assert_eq!(wait_set.get_conditions().length(), 0);

    let reader_cond = reader.get_status_condition();
    let writer_cond = writer.get_status_condition();

    // Attach status condition directly
    wait_set
        .attach_status_condition(&reader_cond)
        .expect("Attach status condition");
    assert_eq!(wait_set.get_conditions().length(), 1);

    // Duplicate attachment has no effect
    wait_set
        .attach_status_condition(&reader_cond)
        .expect("Duplicate attach");
    assert_eq!(wait_set.get_conditions().length(), 1);

    // Attach as Condition
    wait_set
        .attach_condition(&writer_cond.as_condition())
        .expect("Attach condition");
    assert_eq!(wait_set.get_conditions().length(), 2);

    // Detach status condition
    wait_set
        .detach_status_condition(&reader_cond)
        .expect("Detach status condition");
    assert_eq!(wait_set.get_conditions().length(), 1);

    // Detaching already detached condition fails with error
    assert!(wait_set.detach_status_condition(&reader_cond).is_err());

    // Detach writer condition
    wait_set
        .detach_condition(&writer_cond.as_condition())
        .expect("Detach writer condition");
    assert_eq!(wait_set.get_conditions().length(), 0);

    participant.delete_contained_entities().await.unwrap();
    factory.delete_participant(&participant).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_wait_set_wait_trigger() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(53, None, None, None)
        .await
        .expect("Create participant");

    let mut builder = DynamicTypeBuilderFactory::create_structure_type("TestStatusType4");
    let u32_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::Uint32);
    builder
        .add_member(&MemberDescriptor::new(
            "id",
            &u32_type,
            Some(0),
            Some(true),
            None,
        ))
        .unwrap();
    let dynamic_type = builder.build().unwrap();

    let topic = participant
        .create_topic("TestStatusTopic4", &dynamic_type, None, None, None)
        .await
        .expect("Create topic");
    let publisher = participant
        .create_publisher(None, None, None)
        .await
        .expect("Create publisher");
    let writer = publisher
        .create_datawriter(&topic, None, None, None)
        .await
        .expect("Create writer");
    let subscriber = participant
        .create_subscriber(None, None, None)
        .await
        .expect("Create subscriber");
    let reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    // Configure reader status condition to only trigger on DataAvailable
    let reader_cond = reader.get_status_condition();
    let mask = js_sys::Array::new();
    mask.push(&JsValue::from(StatusKind::DataAvailable));
    reader_cond.set_enabled_statuses(&mask).await.unwrap();

    let wait_set = WaitSet::new();
    wait_set.attach_status_condition(&reader_cond).unwrap();

    // Construct JS data sample
    let sample_obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &sample_obj,
        &JsValue::from_str("id"),
        &JsValue::from_f64(42.0),
    )
    .unwrap();

    // Wait for writer and reader to match
    for _ in 0..50 {
        let status = writer.get_publication_matched_status().await.unwrap();
        if status.current_count() > 0 {
            break;
        }
        dust_dds_wasm::runtime::sleep_ms(50).await;
    }

    writer.write(sample_obj.into(), None).await.unwrap();

    // Wait on wait_set
    let triggered = wait_set.wait().await.expect("WaitSet wait succeeded");
    assert!(triggered.length() >= 1);

    // Verify sample can be taken
    let samples = reader.take(None).await.unwrap();
    assert_eq!(samples.length(), 1);

    participant.delete_contained_entities().await.unwrap();
    factory.delete_participant(&participant).await.unwrap();
}
