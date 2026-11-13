use dust_dds_wasm::{
    DomainParticipantFactory, DynamicTypeBuilderFactory, MemberDescriptor, TypeKind,
    runtime::sleep_ms,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn create_test_topic_type() -> dust_dds_wasm::DynamicType {
    let mut builder = DynamicTypeBuilderFactory::create_structure_type("ReaderTestData");
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
async fn test_reader_properties() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(50, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ReaderPropTopic", &dynamic_type, None, None, None)
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

    assert_eq!(reader.get_instance_handle().value().len(), 16);

    let retrieved_subscriber = reader.get_subscriber();
    assert!(
        retrieved_subscriber
            .get_instance_handle()
            .equals(&subscriber.get_instance_handle())
    );

    let topic_desc = reader.get_topicdescription();
    assert_eq!(topic_desc.get_name(), "ReaderPropTopic");
    assert_eq!(topic_desc.get_type_name(), "ReaderTestData");
    assert_eq!(topic_desc.get_participant().get_domain_id(), 50);

    subscriber
        .delete_datareader(&reader)
        .await
        .expect("Delete reader");
    participant
        .delete_subscriber(&subscriber)
        .await
        .expect("Delete subscriber");
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
async fn test_reader_qos() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(51, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ReaderQosTopic", &dynamic_type, None, None, None)
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

    let qos = reader.get_qos().await.expect("Get qos");
    reader.set_qos(Some(qos)).await.expect("Set qos");
    reader.set_qos(None).await.expect("Reset qos");

    subscriber
        .delete_datareader(&reader)
        .await
        .expect("Delete reader");
    participant
        .delete_subscriber(&subscriber)
        .await
        .expect("Delete subscriber");
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
async fn test_reader_operations() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(52, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ReaderOpsTopic", &dynamic_type, None, None, None)
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

    reader.enable().await.expect("Enable reader");

    let status = reader
        .get_subscription_matched_status()
        .await
        .expect("Get subscription matched status");
    assert_eq!(status.total_count(), 0);

    let publications = reader
        .get_matched_publications()
        .await
        .expect("Get matched publications");
    assert_eq!(publications.length(), 0);

    // With default Volatile durability, wait_for_historical_data is expected to return an error (IllegalOperation)
    let wait_res = reader.wait_for_historical_data().await;
    assert!(wait_res.is_err());

    subscriber
        .delete_datareader(&reader)
        .await
        .expect("Delete reader");
    participant
        .delete_subscriber(&subscriber)
        .await
        .expect("Delete subscriber");
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
async fn test_reader_sample_access_methods() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(53, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ReaderAccessTopic", &dynamic_type, None, None, None)
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

    for _ in 0..50 {
        let status = writer.get_publication_matched_status().await.unwrap();
        if status.current_count() > 0 {
            break;
        }
        sleep_ms(50).await;
    }

    let sample1 = js_sys::Object::new();
    js_sys::Reflect::set(&sample1, &JsValue::from_str("id"), &JsValue::from_f64(1.0)).unwrap();
    js_sys::Reflect::set(
        &sample1,
        &JsValue::from_str("msg"),
        &JsValue::from_str("Sample1"),
    )
    .unwrap();

    let sample2 = js_sys::Object::new();
    js_sys::Reflect::set(&sample2, &JsValue::from_str("id"), &JsValue::from_f64(2.0)).unwrap();
    js_sys::Reflect::set(
        &sample2,
        &JsValue::from_str("msg"),
        &JsValue::from_str("Sample2"),
    )
    .unwrap();

    let handle1 = writer
        .register_instance(sample1.clone().into())
        .await
        .unwrap()
        .unwrap();
    let handle2 = writer
        .register_instance(sample2.clone().into())
        .await
        .unwrap()
        .unwrap();

    let (first_handle, _second_handle) = if handle1.value() < handle2.value() {
        (handle1.clone(), handle2.clone())
    } else {
        (handle2.clone(), handle1.clone())
    };

    // Write samples for both instances
    writer
        .write(sample1.into(), Some(handle1.clone()))
        .await
        .unwrap();
    writer
        .write(sample2.into(), Some(handle2.clone()))
        .await
        .unwrap();

    // Wait for both samples to arrive
    for _ in 0..50 {
        let s = reader.read(Some(10)).await.unwrap();
        if s.length() == 2 {
            break;
        }
        sleep_ms(50).await;
    }

    // Test read_instance on handle1
    let instance_samples = reader
        .read_instance(&handle1, Some(10))
        .await
        .expect("Read instance");
    assert_eq!(instance_samples.length(), 1);

    // Test read_next_instance from None (returns first instance)
    let first_samples = reader
        .read_next_instance(None, Some(10))
        .await
        .expect("Read next instance from None");
    assert_eq!(first_samples.length(), 1);

    // Test read_next_instance from first_handle (returns second instance)
    let second_samples = reader
        .read_next_instance(Some(first_handle.clone()), Some(10))
        .await
        .expect("Read next instance from first");
    assert_eq!(second_samples.length(), 1);

    // Test take_instance on handle1
    let taken_instance = reader
        .take_instance(&handle1, Some(10))
        .await
        .expect("Take instance");
    assert_eq!(taken_instance.length(), 1);

    // Write sample 3 for a new sample to test read_next_sample and take_next_sample
    let sample3 = js_sys::Object::new();
    js_sys::Reflect::set(&sample3, &JsValue::from_str("id"), &JsValue::from_f64(3.0)).unwrap();
    js_sys::Reflect::set(
        &sample3,
        &JsValue::from_str("msg"),
        &JsValue::from_str("Sample3"),
    )
    .unwrap();
    let _handle3 = writer
        .register_instance(sample3.clone().into())
        .await
        .unwrap()
        .unwrap();
    writer
        .write(sample3.into(), Some(_handle3.clone()))
        .await
        .unwrap();

    // Wait for sample 3 to arrive
    for _ in 0..50 {
        let s = reader.read(Some(10)).await.unwrap();
        if s.length() >= 2 {
            // sample2 and sample3
            break;
        }
        sleep_ms(50).await;
    }

    // Test take_next_instance from first_handle (takes second_handle or handle3)
    let taken_next = reader
        .take_next_instance(Some(first_handle), Some(10))
        .await
        .expect("Take next instance");
    assert_eq!(taken_next.length(), 1);

    // Write sample 4 to test read_next_sample
    let sample4 = js_sys::Object::new();
    js_sys::Reflect::set(&sample4, &JsValue::from_str("id"), &JsValue::from_f64(4.0)).unwrap();
    js_sys::Reflect::set(
        &sample4,
        &JsValue::from_str("msg"),
        &JsValue::from_str("Sample4"),
    )
    .unwrap();
    let _handle4 = writer
        .register_instance(sample4.clone().into())
        .await
        .unwrap()
        .unwrap();
    writer.write(sample4.into(), Some(_handle4)).await.unwrap();

    let mut next_sample = Err(js_sys::Error::new("").into());
    for _ in 0..50 {
        let res = reader.read_next_sample().await;
        if res.is_ok() {
            next_sample = res;
            break;
        }
        sleep_ms(50).await;
    }
    assert!(next_sample.is_ok());

    // Write sample 5 to test take_next_sample
    let sample5 = js_sys::Object::new();
    js_sys::Reflect::set(&sample5, &JsValue::from_str("id"), &JsValue::from_f64(5.0)).unwrap();
    js_sys::Reflect::set(
        &sample5,
        &JsValue::from_str("msg"),
        &JsValue::from_str("Sample5"),
    )
    .unwrap();
    let _handle5 = writer
        .register_instance(sample5.clone().into())
        .await
        .unwrap()
        .unwrap();
    writer.write(sample5.into(), Some(_handle5)).await.unwrap();

    let mut taken_sample = Err(js_sys::Error::new("").into());
    for _ in 0..50 {
        let res = reader.take_next_sample().await;
        if res.is_ok() {
            taken_sample = res;
            break;
        }
        sleep_ms(50).await;
    }
    assert!(taken_sample.is_ok());

    participant.delete_contained_entities().await.unwrap();
    factory.delete_participant(&participant).await.unwrap();
}
