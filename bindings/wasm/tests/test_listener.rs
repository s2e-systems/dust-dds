use dust_dds_wasm::{
    DomainParticipantFactory, DynamicTypeBuilderFactory, MemberDescriptor, StatusKind, TypeKind,
    infrastructure::status::PublicationMatchedStatus, runtime::sleep_ms,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn create_test_topic_type() -> dust_dds_wasm::DynamicType {
    let mut builder = DynamicTypeBuilderFactory::create_structure_type("ListenerTestData");
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
async fn test_data_reader_listener_on_data_available() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(60, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ListenerTestTopic1", &dynamic_type, None, None, None)
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

    // Setup listener object in JS
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    let cb = Closure::wrap(Box::new(move |_reader: JsValue| {
        called_clone.store(true, Ordering::SeqCst);
    }) as Box<dyn FnMut(JsValue)>);

    let listener_obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &listener_obj,
        &JsValue::from_str("onDataAvailable"),
        cb.as_ref().unchecked_ref(),
    )
    .unwrap();

    let mask = js_sys::Array::new();
    mask.push(&JsValue::from(StatusKind::DataAvailable));

    let reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    reader
        .set_listener(Some(listener_obj), Some(mask))
        .await
        .expect("Set listener");

    // Wait for discovery
    for _ in 0..50 {
        let p_status = writer.get_publication_matched_status().await.unwrap();
        let s_status = reader.get_subscription_matched_status().await.unwrap();
        if p_status.current_count() > 0 && s_status.current_count() > 0 {
            break;
        }
        sleep_ms(50).await;
    }

    // Write a sample
    let sample = js_sys::Object::new();
    js_sys::Reflect::set(&sample, &JsValue::from_str("id"), &JsValue::from_f64(10.0)).unwrap();
    js_sys::Reflect::set(
        &sample,
        &JsValue::from_str("msg"),
        &JsValue::from_str("HelloListener"),
    )
    .unwrap();
    writer.write(sample.into(), None).await.unwrap();

    // Poll until onDataAvailable is invoked
    for _ in 0..50 {
        if called.load(Ordering::SeqCst) {
            break;
        }
        sleep_ms(50).await;
    }

    assert!(
        called.load(Ordering::SeqCst),
        "onDataAvailable was not called!"
    );

    // Clear listener
    reader
        .set_listener(None, None)
        .await
        .expect("Clear listener");
}

#[wasm_bindgen_test]
async fn test_data_writer_listener_publication_matched() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(61, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ListenerTestTopic2", &dynamic_type, None, None, None)
        .await
        .expect("Create topic");

    let publisher = participant
        .create_publisher(None, None, None)
        .await
        .expect("Create publisher");

    let matched_count = Arc::new(AtomicU32::new(0));
    let matched_clone = matched_count.clone();
    let cb = Closure::wrap(
        Box::new(move |_writer: JsValue, status: PublicationMatchedStatus| {
            matched_clone.store(status.current_count() as u32, Ordering::SeqCst);
        }) as Box<dyn FnMut(JsValue, PublicationMatchedStatus)>,
    );

    let listener_obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &listener_obj,
        &JsValue::from_str("onPublicationMatched"),
        cb.as_ref().unchecked_ref(),
    )
    .unwrap();

    let writer = publisher
        .create_datawriter(&topic, None, None, None)
        .await
        .expect("Create writer");

    let mask = js_sys::Array::new();
    mask.push(&JsValue::from(StatusKind::PublicationMatched));

    writer
        .set_listener(Some(listener_obj), Some(mask))
        .await
        .expect("Set writer listener");

    let subscriber = participant
        .create_subscriber(None, None, None)
        .await
        .expect("Create subscriber");

    let _reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    // Wait for onPublicationMatched to be called
    for _ in 0..50 {
        if matched_count.load(Ordering::SeqCst) > 0 {
            break;
        }
        sleep_ms(50).await;
    }

    assert!(
        matched_count.load(Ordering::SeqCst) > 0,
        "onPublicationMatched was not called with count > 0!"
    );
}

#[wasm_bindgen_test]
async fn test_listener_missing_methods_and_error_handling() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(62, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ListenerTestTopic3", &dynamic_type, None, None, None)
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

    // 1. Empty listener object (no methods at all) - should not error or crash
    let empty_listener = js_sys::Object::new();
    let reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    reader
        .set_listener(Some(empty_listener), None)
        .await
        .expect("Set empty listener");

    // 2. Listener with throwing callback - should be caught and logged without crashing DDS
    let throwing_fn =
        js_sys::Function::new_no_args("throw new Error('intentional error in listener');");

    let throwing_listener = js_sys::Object::new();
    js_sys::Reflect::set(
        &throwing_listener,
        &JsValue::from_str("onDataAvailable"),
        &throwing_fn,
    )
    .unwrap();

    let mask = js_sys::Array::new();
    mask.push(&JsValue::from(StatusKind::DataAvailable));

    reader
        .set_listener(Some(throwing_listener), Some(mask))
        .await
        .expect("Set throwing listener");

    // Wait for discovery
    for _ in 0..50 {
        let status = writer.get_publication_matched_status().await.unwrap();
        if status.current_count() > 0 {
            break;
        }
        sleep_ms(50).await;
    }

    // Write a sample
    let sample = js_sys::Object::new();
    js_sys::Reflect::set(&sample, &JsValue::from_str("id"), &JsValue::from_f64(1.0)).unwrap();
    js_sys::Reflect::set(
        &sample,
        &JsValue::from_str("msg"),
        &JsValue::from_str("ThrowTest"),
    )
    .unwrap();
    writer.write(sample.into(), None).await.unwrap();

    // Sleep a bit so the callback fires and the error is caught
    sleep_ms(200).await;

    // Reader should still function normally and read the sample
    let samples = reader.read(Some(10)).await.expect("Read samples");
    assert_eq!(samples.length(), 1);
}

#[wasm_bindgen_test]
async fn test_subscriber_listener_on_data_on_readers() {
    let factory = DomainParticipantFactory::new(None);
    let participant = factory
        .create_participant(63, None, None, None)
        .await
        .expect("Create participant");

    let dynamic_type = create_test_topic_type();
    let topic = participant
        .create_topic("ListenerTestTopic4", &dynamic_type, None, None, None)
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

    let data_on_readers_called = Arc::new(AtomicBool::new(false));
    let data_on_readers_clone = data_on_readers_called.clone();
    let cb = Closure::wrap(Box::new(move |_sub: JsValue| {
        data_on_readers_clone.store(true, Ordering::SeqCst);
    }) as Box<dyn FnMut(JsValue)>);

    let sub_listener = js_sys::Object::new();
    js_sys::Reflect::set(
        &sub_listener,
        &JsValue::from_str("onDataOnReaders"),
        cb.as_ref().unchecked_ref(),
    )
    .unwrap();

    let mask = js_sys::Array::new();
    mask.push(&JsValue::from(StatusKind::DataOnReaders));

    let subscriber = participant
        .create_subscriber(None, Some(sub_listener), Some(mask))
        .await
        .expect("Create subscriber");

    let reader = subscriber
        .create_datareader(&topic, None, None, None)
        .await
        .expect("Create reader");

    // Wait for discovery
    for _ in 0..50 {
        let p_status = writer.get_publication_matched_status().await.unwrap();
        let s_status = reader.get_subscription_matched_status().await.unwrap();
        if p_status.current_count() > 0 && s_status.current_count() > 0 {
            break;
        }
        sleep_ms(50).await;
    }

    // Write a sample
    let sample = js_sys::Object::new();
    js_sys::Reflect::set(&sample, &JsValue::from_str("id"), &JsValue::from_f64(42.0)).unwrap();
    js_sys::Reflect::set(
        &sample,
        &JsValue::from_str("msg"),
        &JsValue::from_str("DataOnReadersTest"),
    )
    .unwrap();
    writer.write(sample.into(), None).await.unwrap();

    // Poll until onDataOnReaders is invoked
    for _ in 0..50 {
        if data_on_readers_called.load(Ordering::SeqCst) {
            break;
        }
        sleep_ms(50).await;
    }

    assert!(
        data_on_readers_called.load(Ordering::SeqCst),
        "onDataOnReaders was not called!"
    );
}
