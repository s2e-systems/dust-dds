use rust_dds::domain::{DomainParticipant, };
use rust_dds::domain::qos::DomainParticipantQos;
use rust_dds::infrastructure::listener::NoListener;

#[test]
fn hello_world() {
    let participant = DomainParticipant::new(0, DomainParticipantQos::default(), NoListener, 0, true);
    // participant.cre
    std::thread::sleep(std::time::Duration::from_secs(100));
}