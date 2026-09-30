use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER,
        qos::QosKind,
        status::{NO_STATUS, StatusKind},
        time::Duration,
    },
    wait_set::{Condition, WaitSet},
};
use micro_dust_dds_interface::ButtonState;
use std::process::Command;

#[test]
fn sample_published_on_target_should_be_received_on_host() {
    let status = Command::new("cargo")
        .current_dir("micro_app")
        .args(["build", "--profile", "micro-dev"])
        .status()
        .expect("Failed to execute cargo build for micro_app");
    if !status.success() {
        panic!("Failed to build micro_app firmware");
    }

    let elf_path = "../target/thumbv7em-none-eabihf/debug/micro_dust_dds_app";
    let status = Command::new("probe-rs")
        .args([
            "download",
            "--chip",
            "STM32H563ZITx",
            "--preverify",
            elf_path,
        ])
        .status()
        .expect("Failed to execute probe-rs download");
    if !status.success() {
        panic!("Failed to download firmware to Nucleo board with probe-rs");
    }

    let status = Command::new("probe-rs")
        .args(["reset", "--chip", "STM32H563ZITx"])
        .status()
        .expect("Failed to execute probe-rs reset");
    if !status.success() {
        panic!("Failed to reset Nucleo board with probe-rs");
    }

    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();
    let button_topic = participant
        .create_topic::<ButtonState>(
            "Button",
            "ButtonState",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();
    let reader = subscriber
        .create_datareader::<ButtonState>(&button_topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let cond = reader.get_statuscondition();
    cond.set_enabled_statuses(&[StatusKind::DataAvailable])
        .unwrap();
    let mut reader_wait_set = WaitSet::new();
    reader_wait_set
        .attach_condition(Condition::StatusCondition(cond))
        .unwrap();
    reader_wait_set.wait(Duration::new(10, 0)).unwrap();

    let sample = reader.take_next_sample();
    assert!(sample.is_ok(), "Did not receive a ButtonState");
}
