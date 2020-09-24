use rust_dds::domain::DomainParticipant;
use rust_dds::domain::qos::DomainParticipantQos;
use rust_dds::publication::qos::PublisherQos;
use rust_dds::infrastructure::listener::NoListener;

#[test]
fn hello_world() {
    let p = DomainParticipant::new(0, DomainParticipantQos::default(), NoListener, 0, true);
    p.unwrap().create_publisher(PublisherQos::default(), NoListener, 0);
    std::thread::sleep(std::time::Duration::from_secs(1));
}