//! Test that crashed participants are detected and removed after lease expiry.
//!
//! This test uses process-level isolation to simulate a crash. The test binary
//! can run in two modes:
//! - Normal test mode: spawns a child process and observes its participant
//! - Child mode (CRASH_TEST_PARTICIPANT=1): creates a participant and waits to be killed

use dust_dds::{
    configuration::DustDdsConfigurationBuilder,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DomainParticipantQos, QosKind},
        qos_policy::DiscoveryConfigQosPolicy,
        status::NO_STATUS,
        time::Duration as DdsDuration,
    },
    listener::NO_LISTENER,
};
use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

mod utils;
use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

const CRASH_TEST_ENV_VAR: &str = "CRASH_TEST_PARTICIPANT";
const DOMAIN_ID_ENV_VAR: &str = "CRASH_TEST_DOMAIN_ID";
const LEASE_DURATION_ENV_VAR: &str = "CRASH_TEST_LEASE_SECS";
const ANNOUNCEMENT_INTERVAL_ENV_VAR: &str = "CRASH_TEST_ANNOUNCE_MS";

/// Entry point for child process mode.
/// Creates a participant with the given configuration and waits to be killed.
fn run_as_crash_participant() {
    let domain_id: i32 = env::var(DOMAIN_ID_ENV_VAR)
        .expect("CRASH_TEST_DOMAIN_ID not set")
        .parse()
        .expect("Invalid domain_id");
    let lease_secs: u64 = env::var(LEASE_DURATION_ENV_VAR)
        .expect("CRASH_TEST_LEASE_SECS not set")
        .parse()
        .expect("Invalid lease_secs");
    let announce_ms: u64 = env::var(ANNOUNCEMENT_INTERVAL_ENV_VAR)
        .expect("CRASH_TEST_ANNOUNCE_MS not set")
        .parse()
        .expect("Invalid announce_ms");

    let participant_factory = DomainParticipantFactory::get_instance();

    let configuration = DustDdsConfigurationBuilder::new()
        .participant_announcement_interval(Duration::from_millis(announce_ms))
        .build()
        .expect("Failed to build configuration");

    participant_factory
        .set_configuration(configuration)
        .expect("Failed to set configuration");

    let qos = DomainParticipantQos {
        discovery_config: DiscoveryConfigQosPolicy {
            participant_lease_duration: DdsDuration::new(lease_secs as i32, 0),
        },
        ..Default::default()
    };

    let _participant = participant_factory
        .create_participant(domain_id, QosKind::Specific(qos), NO_LISTENER, NO_STATUS)
        .expect("Failed to create participant");

    // Signal ready
    println!("READY");

    // Sleep forever until killed
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}

/// Spawns a child process that creates a participant and returns the child handle.
fn spawn_crash_participant(
    domain_id: i32,
    lease_secs: u64,
    announce_ms: u64,
) -> std::process::Child {
    // Get the path to the current test binary
    let current_exe = env::current_exe().expect("Failed to get current exe path");

    let mut child = Command::new(current_exe)
        // Run the specific test that triggers child mode
        .args(["--exact", "crash_participant_child_entry", "--nocapture"])
        .env(CRASH_TEST_ENV_VAR, "1")
        .env(DOMAIN_ID_ENV_VAR, domain_id.to_string())
        .env(LEASE_DURATION_ENV_VAR, lease_secs.to_string())
        .env(ANNOUNCEMENT_INTERVAL_ENV_VAR, announce_ms.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn child process");

    // Wait for READY signal
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Read lines until we get READY (cargo test outputs some noise first)
    loop {
        line.clear();
        reader.read_line(&mut line).expect("Failed to read line");
        if line.trim() == "READY" {
            break;
        }
        // Timeout protection
        if line.is_empty() {
            panic!("Child process closed stdout without sending READY");
        }
    }

    child
}

/// This test is the entry point for the child process.
/// When CRASH_TEST_PARTICIPANT is set, it runs as the crash participant.
#[test]
fn crash_participant_child_entry() {
    if env::var(CRASH_TEST_ENV_VAR).is_ok() {
        run_as_crash_participant();
    }
    // If env var not set, this test does nothing (it's just an entry point)
}

/// Test that a crashed participant is removed after lease expiry.
///
/// This test:
/// 1. Creates an observer participant
/// 2. Spawns a child process with a short-lease participant
/// 3. Waits for the child to be discovered
/// 4. Kills the child process (simulating crash)
/// 5. Waits for lease expiry
/// 6. Verifies the crashed participant is removed
#[test]
#[ignore = "Slow test (~15s) - requires process spawning and lease expiry wait"]
fn crashed_participant_is_removed_after_lease_expiry() {
    // Skip if we're running as the child
    if env::var(CRASH_TEST_ENV_VAR).is_ok() {
        return;
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory = DomainParticipantFactory::get_instance();

    // Configuration for fast testing
    let lease_secs: u64 = 3;
    let announce_ms: u64 = 500;
    let liveliness_check_secs: u64 = 10; // Hardcoded in domain_participant_factory.rs

    // Configure the observer with the same settings
    let configuration = DustDdsConfigurationBuilder::new()
        .participant_announcement_interval(Duration::from_millis(announce_ms))
        .build()
        .unwrap();
    domain_participant_factory
        .set_configuration(configuration)
        .unwrap();

    let observer_qos = DomainParticipantQos {
        discovery_config: DiscoveryConfigQosPolicy {
            participant_lease_duration: DdsDuration::new(lease_secs as i32, 0),
        },
        ..Default::default()
    };

    // Create observer participant
    let observer = domain_participant_factory
        .create_participant(domain_id, QosKind::Specific(observer_qos), NO_LISTENER, NO_STATUS)
        .expect("Failed to create observer participant");

    // Spawn crash participant in child process
    let mut child = spawn_crash_participant(domain_id, lease_secs, announce_ms);

    // Wait for discovery
    let start = Instant::now();
    loop {
        let discovered = observer.get_discovered_participants().unwrap();
        if discovered.len() == 2 {
            break;
        }
        if start.elapsed() > Duration::from_secs(10) {
            child.kill().ok();
            panic!(
                "Child participant not discovered within timeout. Found: {}",
                discovered.len()
            );
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Verify we discovered 2 participants (self + child)
    assert_eq!(
        observer.get_discovered_participants().unwrap().len(),
        2,
        "Should discover self + child participant"
    );

    // Kill the child to simulate crash
    child.kill().expect("Failed to kill child");
    child.wait().expect("Failed to wait for child");

    // Wait for lease expiry + liveliness check + buffer
    let wait_time = Duration::from_secs(lease_secs + liveliness_check_secs + 2);
    thread::sleep(wait_time);

    // Verify crashed participant was removed
    let discovered = observer.get_discovered_participants().unwrap();
    assert_eq!(
        discovered.len(),
        1,
        "Only observer should remain after lease expiry. Found: {:?}",
        discovered
    );

    // Cleanup
    domain_participant_factory
        .delete_participant(&observer)
        .unwrap();
}
