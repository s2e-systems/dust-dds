use dust_dds_wasm::{
    DomainParticipantFactory, DynamicTypeBuilderFactory, MemberDescriptor, TypeKind,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use dust_dds_wasm::runtime::sleep_ms;

#[wasm_bindgen_test]
async fn test_dynamic_type_builder() {
    let mut builder = DynamicTypeBuilderFactory::create_structure_type("SensorData");

    let id_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::Uint32);
    let id_member = MemberDescriptor::new("sensor_id", &id_type, Some(0), Some(true), None);
    builder
        .add_member(&id_member)
        .expect("Add sensor_id member");

    let location_type = DynamicTypeBuilderFactory::create_string_type(None);
    let location_member = MemberDescriptor::new("location", &location_type, Some(1), None, None);
    builder
        .add_member(&location_member)
        .expect("Add location member");

    let u8_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::Uint8);
    let payload_type = DynamicTypeBuilderFactory::create_sequence_type(&u8_type, None);
    let payload_member = MemberDescriptor::new("payload", &payload_type, Some(2), None, None);
    builder
        .add_member(&payload_member)
        .expect("Add payload member");

    let dynamic_type = builder.build().expect("Build dynamic type");
    assert_eq!(dynamic_type.get_name(), "SensorData");
    assert_eq!(dynamic_type.get_kind(), TypeKind::Structure);
    assert_eq!(dynamic_type.get_member_count(), 3);
}

#[wasm_bindgen_test]
async fn test_write_read_dynamic_data() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(100, None)
        .await
        .expect("Create participant");

    // Define DynamicType
    let mut builder = DynamicTypeBuilderFactory::create_structure_type("TestData");
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

    let u8_type = DynamicTypeBuilderFactory::get_primitive_type(TypeKind::Uint8);
    let bytes_type = DynamicTypeBuilderFactory::create_sequence_type(&u8_type, None);
    builder
        .add_member(&MemberDescriptor::new(
            "data",
            &bytes_type,
            Some(2),
            None,
            None,
        ))
        .unwrap();

    let dynamic_type = builder.build().unwrap();

    let topic = participant
        .create_topic("TestTopic", &dynamic_type, None)
        .await
        .expect("Create topic");
    let retrieved_type = topic
        .get_dynamic_type()
        .await
        .expect("Get dynamic type from topic");
    assert_eq!(retrieved_type.get_name(), "TestData");

    let publisher = participant
        .create_publisher(None)
        .await
        .expect("Create publisher");
    let writer = publisher
        .create_datawriter(&topic, None)
        .await
        .expect("Create writer");

    let subscriber = participant
        .create_subscriber(None)
        .await
        .expect("Create subscriber");
    let reader = subscriber
        .create_datareader(&topic, None)
        .await
        .expect("Create reader");

    // Construct JS data sample
    let sample_obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &sample_obj,
        &JsValue::from_str("id"),
        &JsValue::from_f64(42.0),
    )
    .unwrap();
    js_sys::Reflect::set(
        &sample_obj,
        &JsValue::from_str("msg"),
        &JsValue::from_str("Hello WASM DDS"),
    )
    .unwrap();
    let uint8_arr = js_sys::Uint8Array::from(&[1u8, 2, 3, 4][..]);
    js_sys::Reflect::set(&sample_obj, &JsValue::from_str("data"), &uint8_arr.into()).unwrap();

    // Wait for writer and reader to match
    for _ in 0..50 {
        let status = writer.get_publication_matched_status().await.unwrap();
        if status.current_count() > 0 {
            break;
        }
        sleep_ms(50).await;
    }

    writer
        .write(sample_obj.into(), None)
        .await
        .expect("Write sample");

    // Poll for samples
    let mut samples = js_sys::Array::new();
    for _ in 0..50 {
        samples = reader.read(Some(1)).await.expect("Read samples");
        if samples.length() > 0 {
            break;
        }
        sleep_ms(50).await;
    }
    assert_eq!(samples.length(), 1);

    let sample_val = samples.get(0);
    let get_data_fn = js_sys::Reflect::get(&sample_val, &JsValue::from_str("getData"))
        .unwrap()
        .dyn_into::<js_sys::Function>()
        .unwrap();
    let data = get_data_fn.call0(&sample_val).unwrap();

    assert!(data.is_object());

    let read_id = js_sys::Reflect::get(&data, &JsValue::from_str("id")).unwrap();
    assert_eq!(read_id.as_f64().unwrap(), 42.0);

    let read_msg = js_sys::Reflect::get(&data, &JsValue::from_str("msg")).unwrap();
    assert_eq!(read_msg.as_string().unwrap(), "Hello WASM DDS");

    let read_data = js_sys::Reflect::get(&data, &JsValue::from_str("data")).unwrap();
    assert!(read_data.is_instance_of::<js_sys::Uint8Array>());
    let read_bytes = js_sys::Uint8Array::new(&read_data).to_vec();
    assert_eq!(read_bytes, vec![1, 2, 3, 4]);

    participant
        .delete_contained_entities()
        .await
        .expect("Delete contained entities");
    factory
        .delete_participant(&participant)
        .await
        .expect("Delete participant");
}
