use rust_dds::domain::DomainParticipant;
use rust_dds::domain::qos::DomainParticipantQos;
use rust_dds::infrastructure::listener::NoListener;

#[test]
fn hello_world() {
    let _p = DomainParticipant::new(0, DomainParticipantQos::default(), NoListener, 0, true);
}