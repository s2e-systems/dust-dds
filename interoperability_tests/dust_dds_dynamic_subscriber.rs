//! DynamicDataReader interop test - reads data from FastDDS without compile-time type.
//!
//! This test proves that dust-dds can read data from FastDDS using only runtime
//! type information (DynamicType), without any code generation or compile-time
//! type knowledge. This is the real TypeLookup/XTypes interop test.

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::NO_STATUS,
        time::{Duration, DurationKind},
    },
    listener::NO_LISTENER,
    xtypes::{
        binding::XTypesBinding,
        dynamic_type::{
            DynamicTypeBuilderFactory, ExtensibilityKind, MemberDescriptor, TryConstructKind,
            TypeDescriptor, TypeKind,
        },
    },
};

/// Manually construct the DynamicType for HelloWorldType.
///
/// This simulates what TypeLookup would provide - a runtime type definition
/// without any compile-time type information.
///
/// IDL:
/// ```idl
/// module interoperability {
///     module test {
///         struct HelloWorldType {
///             octet id;
///             char msg;
///         };
///     };
/// };
/// ```
fn create_hello_world_dynamic_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
    let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
        kind: TypeKind::STRUCTURE,
        name: String::from("interoperability::test::HelloWorldType"),
        base_type: None,
        discriminator_type: None,
        bound: Vec::new(),
        element_type: None,
        key_element_type: None,
        extensibility_kind: ExtensibilityKind::Final,
        is_nested: false,
    });

    // id: octet (u8)
    builder
        .add_member(MemberDescriptor {
            name: String::from("id"),
            id: 0,
            r#type: <u8 as XTypesBinding>::get_dynamic_type(),
            default_value: None,
            index: 0,
            label: Vec::new(),
            try_construct_kind: TryConstructKind::UseDefault,
            is_key: false,
            is_optional: false,
            is_must_understand: false,
            is_shared: false,
            is_default_label: false,
        })
        .expect("Failed to add 'id' member");

    // msg: char (char in DDS IDL)
    builder
        .add_member(MemberDescriptor {
            name: String::from("msg"),
            id: 1,
            r#type: <char as XTypesBinding>::get_dynamic_type(),
            default_value: None,
            index: 1,
            label: Vec::new(),
            try_construct_kind: TryConstructKind::UseDefault,
            is_key: false,
            is_optional: false,
            is_must_understand: false,
            is_shared: false,
            is_default_label: false,
        })
        .expect("Failed to add 'msg' member");

    builder.build()
}

fn main() {
    println!("DynamicDataReader interop test starting...");
    println!("This test reads FastDDS data using ONLY runtime type information.");
    println!();

    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Failed to create participant");

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .expect("Failed to create subscriber");

    // Create DynamicType manually (simulating TypeLookup discovery)
    let dynamic_type = create_hello_world_dynamic_type();
    println!("Created DynamicType: {}", dynamic_type.get_name());

    let reader_qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        ..Default::default()
    };

    // Create DynamicDataReader - NO compile-time type knowledge!
    let reader = subscriber
        .create_dynamic_datareader("HelloWorld", dynamic_type, QosKind::Specific(reader_qos))
        .expect("Failed to create dynamic datareader");

    println!("Created DynamicDataReader for topic 'HelloWorld'");
    println!("Waiting for data (60s timeout)...");

    // Wait for subscription match
    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[dust_dds::infrastructure::status::StatusKind::SubscriptionMatched])
        .expect("Failed to set enabled statuses");

    let mut wait_set = dust_dds::wait_set::WaitSet::new();
    wait_set
        .attach_condition(dust_dds::wait_set::Condition::StatusCondition(
            reader_cond.clone(),
        ))
        .expect("Failed to attach condition");

    if wait_set.wait(Duration::new(60, 0)).is_err() {
        eprintln!("ERROR: Timeout waiting for subscription match");
        std::process::exit(1);
    }
    println!("Subscription matched!");

    // Wait for data - shorter timeout, poll periodically
    reader_cond
        .set_enabled_statuses(&[dust_dds::infrastructure::status::StatusKind::DataAvailable])
        .expect("Failed to set enabled statuses");

    let mut data_received = false;
    for attempt in 1..=10 {
        println!("Polling for data (attempt {}/10)...", attempt);
        if wait_set.wait(Duration::new(3, 0)).is_ok() {
            data_received = true;
            break;
        }
        // Also try reading directly in case the condition wasn't triggered
        if let Ok(samples) = reader.read(1, &ANY_SAMPLE_STATE, &ANY_VIEW_STATE, &ANY_INSTANCE_STATE)
        {
            if !samples.is_empty() {
                println!("Found data via direct read!");
                data_received = true;
                break;
            }
        }
    }
    if !data_received {
        eprintln!("ERROR: Timeout waiting for data after 30 seconds");
        std::process::exit(1);
    }

    // Read with DynamicDataReader
    let samples = reader
        .read(1, &ANY_SAMPLE_STATE, &ANY_VIEW_STATE, &ANY_INSTANCE_STATE)
        .expect("Failed to read samples");

    if samples.is_empty() {
        eprintln!("ERROR: No samples received");
        std::process::exit(1);
    }

    let sample = &samples[0];
    if sample.data.is_none() {
        eprintln!("ERROR: Sample has no valid data");
        std::process::exit(1);
    }

    let data = sample.data.as_ref().unwrap();

    // Access fields by name - proving we can read without compile-time type
    let id = data
        .get_uint8_value_by_name("id")
        .expect("Failed to get 'id' field");
    let msg = data
        .get_char8_value_by_name("msg")
        .expect("Failed to get 'msg' field");

    println!();
    println!("=== SUCCESS: DynamicDataReader received data ===");
    println!(
        "Received: HelloWorldType {{ id: {}, msg: '{}' }}",
        id,
        *msg as char
    );
    println!();
    println!("This proves dust-dds can read data using ONLY runtime type information!");
    println!("(No compile-time type knowledge was required for this reader.)");

    // Sleep to allow sending acknowledgements
    std::thread::sleep(std::time::Duration::from_secs(2));
}
