use rust_dds::domain::DomainParticipant;
use rust_dds::domain::qos::DomainParticipantQos;
use rust_dds::topic::qos::TopicQos;
use rust_dds::publication::qos::{/*DataWriterQos, */PublisherQos};
use rust_dds::infrastructure::listener::NoListener;
use rust_dds::types::DDSType;

struct HelloWorldType {
    _msg: String
}

impl DDSType for HelloWorldType {
    fn instance_handle(&self) -> rust_dds_interface::types::InstanceHandle {
        todo!()
    }

    fn serialize(&self) -> rust_dds_interface::types::Data {
        todo!()
    }

    fn deserialize(_data: rust_dds_interface::types::Data) -> Self {
        todo!()
    }
}

#[test]
fn hello_world() {
    let participant = DomainParticipant::new(0, DomainParticipantQos::default(), NoListener, 0, true).expect("Error creating participant");
    
    let _publisher = participant.create_publisher(PublisherQos::default(), NoListener, 0).expect("Error creating publisher");
    let _helloworld_topic = participant.create_topic("HelloWorld".to_string(), "HelloWorldType".to_string(), TopicQos::default(), NoListener, 0).expect("Error creating topic");

    //let _datawriter = publisher.create_datawriter::<HelloWorldType>(helloworld_topic, DataWriterQos::default(), Box::new(NoListener), 0).expect("Error creating data writer");
}