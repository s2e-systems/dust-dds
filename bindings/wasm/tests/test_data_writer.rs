use dust_dds_wasm::{
    DomainParticipantFactory, DynamicTypeBuilderFactory, MemberDescriptor, TypeKind,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn create_test_topic_type() -> dust_dds_wasm::DynamicType {
    let mut builder = DynamicTypeBuilderFactory::create_structure_type("WriterTestData");
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

    let str_type = DynamicTypeBuilderFactory::create_string_type(None);
    builder
        .add_member(&MemberDescriptor::new(
            "msg",
            &str_type,
            Some(1),
            None,
            None,
        ))
        .unwrap();

    builder.build().unwrap()
}

#[wasm_bindgen_test]
async fn test_writer_properties() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(40, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("WriterPropTopic", &dynamic_type, None, None, None)
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

    assert_eq!(writer.get_instance_handle().value().len(), 16);
    let retrieved_topic = writer.get_topic();
    assert_eq!(retrieved_topic.get_name(), "WriterPropTopic");

    let retrieved_publisher = writer.get_publisher();
    assert!(
        retrieved_publisher
            .get_instance_handle()
            .equals(&publisher.get_instance_handle())
    );

    publisher
        .delete_datawriter(&writer)
        .await
        .expect("Delete writer");
    participant
        .delete_publisher(&publisher)
        .await
        .expect("Delete publisher");
    participant
        .delete_topic(&topic)
        .await
        .expect("Delete topic");
    factory
        .delete_participant(&participant)
        .await
        .expect("Delete participant");
}

#[wasm_bindgen_test]
async fn test_writer_qos() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(41, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("WriterQosTopic", &dynamic_type, None, None, None)
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

    let qos = writer.get_qos().await.expect("Get qos");
    writer.set_qos(Some(qos)).await.expect("Set qos");
    writer.set_qos(None).await.expect("Reset qos");

    publisher
        .delete_datawriter(&writer)
        .await
        .expect("Delete writer");
    participant
        .delete_publisher(&publisher)
        .await
        .expect("Delete publisher");
    participant
        .delete_topic(&topic)
        .await
        .expect("Delete topic");
    factory
        .delete_participant(&participant)
        .await
        .expect("Delete participant");
}

#[wasm_bindgen_test]
async fn test_writer_instance_management() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(42, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("WriterInstanceTopic", &dynamic_type, None, None, None)
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

    let sample_obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &sample_obj,
        &JsValue::from_str("id"),
        &JsValue::from_f64(10.0),
    )
    .unwrap();
    js_sys::Reflect::set(
        &sample_obj,
        &JsValue::from_str("msg"),
        &JsValue::from_str("TestInstance"),
    )
    .unwrap();

    let registered_handle = writer
        .register_instance(sample_obj.clone().into())
        .await
        .expect("Register instance");
    assert!(registered_handle.is_some());
    let handle = registered_handle.unwrap();
    assert_eq!(handle.value().len(), 16);

    let looked_up_handle = writer
        .lookup_instance(sample_obj.clone().into())
        .await
        .expect("Lookup instance");
    assert!(looked_up_handle.is_some());
    assert!(looked_up_handle.unwrap().equals(&handle));

    writer
        .unregister_instance(sample_obj.clone().into(), Some(handle.clone()))
        .await
        .expect("Unregister instance");

    writer
        .dispose(sample_obj.into(), Some(handle))
        .await
        .expect("Dispose instance");

    publisher
        .delete_datawriter(&writer)
        .await
        .expect("Delete writer");
    participant
        .delete_publisher(&publisher)
        .await
        .expect("Delete publisher");
    participant
        .delete_topic(&topic)
        .await
        .expect("Delete topic");
    factory
        .delete_participant(&participant)
        .await
        .expect("Delete participant");
}

#[wasm_bindgen_test]
async fn test_writer_operations() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(43, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("WriterOpsTopic", &dynamic_type, None, None, None)
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

    writer.enable().await.expect("Enable writer");

    let status = writer
        .get_publication_matched_status()
        .await
        .expect("Get publication matched status");
    assert_eq!(status.total_count(), 0);

    let subscriptions = writer
        .get_matched_subscriptions()
        .await
        .expect("Get matched subscriptions");
    assert_eq!(subscriptions.length(), 0);

    writer
        .wait_for_acknowledgments()
        .await
        .expect("Wait for acknowledgments");

    publisher
        .delete_datawriter(&writer)
        .await
        .expect("Delete writer");
    participant
        .delete_publisher(&publisher)
        .await
        .expect("Delete publisher");
    participant
        .delete_topic(&topic)
        .await
        .expect("Delete topic");
    factory
        .delete_participant(&participant)
        .await
        .expect("Delete participant");
}
