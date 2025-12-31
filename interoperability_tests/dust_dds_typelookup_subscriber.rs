//! TypeLookup interop test - discovers type from remote participant without compile-time knowledge.
//!
//! This test proves that dust-dds can:
//! 1. Discover a remote writer via SEDP
//! 2. Request the type via TypeLookup service
//! 3. Convert the received TypeObject to DynamicType
//! 4. Create a DynamicDataReader and read data
//!
//! Run this with either dust-dds or FastDDS publisher to test interoperability.

use dust_dds::{
    dds_async::{
        domain_participant_factory::DomainParticipantFactoryAsync,
        wait_set::{ConditionAsync, WaitSetAsync},
    },
    infrastructure::{
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        status::{StatusKind, NO_STATUS},
        time::{Duration, DurationKind},
    },
};
use std::time::Duration as StdDuration;

#[tokio::main]
async fn main() {
    println!("TypeLookup interop test starting...");
    println!("This test discovers the type from a remote participant via TypeLookup.");
    println!();

    let domain_id = 0;
    let participant_factory = DomainParticipantFactoryAsync::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None::<()>, NO_STATUS)
        .await
        .expect("Failed to create participant");

    println!("Participant created, waiting for remote publisher...");

    // Wait for discovery - need to find a writer for topic "HelloWorld"
    let mut discovered_type = None;
    for attempt in 1..=30 {
        println!("Discovery attempt {}/30...", attempt);

        // Debug: check what participants and topics are discovered
        match participant.get_discovered_participants().await {
            Ok(participants) => println!("  Discovered {} participant(s)", participants.len()),
            Err(e) => println!("  Error getting discovered participants: {:?}", e),
        }
        match participant.get_discovered_topics().await {
            Ok(topics) => println!("  Discovered {} topic(s)", topics.len()),
            Err(e) => println!("  Error getting discovered topics: {:?}", e),
        }

        match participant.discover_type("HelloWorld").await {
            Ok(dynamic_type) => {
                println!("SUCCESS: Discovered type via TypeLookup!");
                println!("  Type name: {}", dynamic_type.get_name());
                discovered_type = Some(dynamic_type);
                break;
            }
            Err(e) => {
                println!("  Not yet: {:?}", e);
                tokio::time::sleep(StdDuration::from_secs(2)).await;
            }
        }
    }

    let dynamic_type = discovered_type.expect("Failed to discover type after 60 seconds");

    // Print type information
    println!();
    println!("Discovered type details:");
    println!("  Name: {}", dynamic_type.get_name());
    let member_count = dynamic_type.get_member_count();
    println!("  Member count: {}", member_count);
    println!();

    // Create subscriber
    let subscriber = participant
        .create_subscriber(QosKind::Default, None::<()>, NO_STATUS)
        .await
        .expect("Failed to create subscriber");

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

    // Create DynamicDataReader with the DISCOVERED type (not manually constructed!)
    let reader = subscriber
        .create_dynamic_datareader("HelloWorld", dynamic_type, QosKind::Specific(reader_qos))
        .await
        .expect("Failed to create dynamic datareader");

    println!("Created DynamicDataReader for topic 'HelloWorld'");
    println!("Waiting for data (60s timeout)...");

    // Wait for subscription match
    let reader_cond = reader.get_statuscondition();
    reader_cond
        .set_enabled_statuses(&[StatusKind::SubscriptionMatched])
        .await
        .expect("Failed to set enabled statuses");

    let mut wait_set = WaitSetAsync::new();
    wait_set
        .attach_condition(ConditionAsync::StatusCondition(reader_cond.clone()))
        .await
        .expect("Failed to attach condition");

    let timeout = tokio::time::timeout(StdDuration::from_secs(60), wait_set.wait());
    if timeout.await.is_err() {
        eprintln!("ERROR: Timeout waiting for subscription match");
        std::process::exit(1);
    }
    println!("Subscription matched!");

    // Wait for data
    reader_cond
        .set_enabled_statuses(&[StatusKind::DataAvailable])
        .await
        .expect("Failed to set enabled statuses");

    let mut data_received = false;
    for attempt in 1..=20 {
        println!("Polling for data (attempt {}/20)...", attempt);

        // Try reading directly
        if let Ok(samples) = reader
            .read(1, &ANY_SAMPLE_STATE, &ANY_VIEW_STATE, &ANY_INSTANCE_STATE)
            .await
        {
            if !samples.is_empty() {
                println!("Found data via direct read!");
                data_received = true;
                break;
            }
        }

        // Also try waiting with a timeout
        let wait_result =
            tokio::time::timeout(StdDuration::from_secs(3), wait_set.wait()).await;
        if wait_result.is_ok() {
            // Condition triggered, try reading again
            if let Ok(samples) = reader
                .read(1, &ANY_SAMPLE_STATE, &ANY_VIEW_STATE, &ANY_INSTANCE_STATE)
                .await
            {
                if !samples.is_empty() {
                    println!("Found data after wait!");
                    data_received = true;
                    break;
                }
            }
        }
    }
    if !data_received {
        eprintln!("ERROR: Timeout waiting for data after 60 seconds");
        std::process::exit(1);
    }

    // Read with DynamicDataReader
    let samples = reader
        .read(1, &ANY_SAMPLE_STATE, &ANY_VIEW_STATE, &ANY_INSTANCE_STATE)
        .await
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

    // Try to access fields by name
    // For HelloWorldType: id (octet), msg (char)
    println!();
    println!("=== SUCCESS: TypeLookup + DynamicDataReader received data ===");

    if let Ok(id) = data.get_uint8_value_by_name("id") {
        println!("  id: {}", id);
    } else if let Ok(id) = data.get_int32_value_by_name("id") {
        println!("  id: {}", id);
    } else {
        println!("  id: <unknown type>");
    }

    if let Ok(msg) = data.get_char8_value_by_name("msg") {
        println!("  msg: '{}'", *msg as char);
    } else if let Ok(msg) = data.get_string_value_by_name("msg") {
        println!("  msg: \"{}\"", msg);
    } else {
        println!("  msg: <unknown type>");
    }

    println!();
    println!("This proves dust-dds can discover types via TypeLookup and read data dynamically!");
    println!("(The type was NOT known at compile time - it was discovered from the remote participant.)");

    // Sleep to allow sending acknowledgements
    tokio::time::sleep(StdDuration::from_secs(2)).await;
}
