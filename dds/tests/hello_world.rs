use rust_dds::domain::DomainParticipant;
use rust_dds::domain::qos::DomainParticipantQos;
use rust_dds::publication::qos::{DataWriterQos, PublisherQos};
use rust_dds::subscription::qos::{DataReaderQos, SubscriberQos};
use rust_dds::infrastructure::listener::NoListener;
use rust_dds::infrastructure::qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind};
use rust_dds::types::DDSType;
use rust_dds_interface::types::{TopicKind, Time, DURATION_ZERO};

struct HelloWorldType {
    id: u8,
    _msg: String
}

impl DDSType for HelloWorldType {
    fn type_name() -> &'static str {
        "HelloWorldType"
    }

    fn topic_kind() -> TopicKind {
        TopicKind::WithKey
    }

    fn instance_handle(&self) -> rust_dds_interface::types::InstanceHandle {
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, self.id]
    }

    fn serialize(&self) -> rust_dds_interface::types::Data {
        vec![self.id]
    }

    fn deserialize(_data: rust_dds_interface::types::Data) -> Self {
        todo!()
    }
}

#[test]
fn hello_world() {
    let participant = DomainParticipant::new(0, DomainParticipantQos::default(), NoListener, 0, true).expect("Error creating participant");
    
    let publisher = participant.create_publisher(Some(&PublisherQos::default()), NoListener, 0).expect("Error creating publisher");
    let helloworld_topic = participant.create_topic("HelloWorld".to_string(), None, NoListener, 0).expect("Error creating topic");

    let subscriber = participant.create_subscriber(Some(&SubscriberQos::default()), NoListener, 0).expect("Error creating subscriber");
    let _datareader = subscriber.create_datareader(&helloworld_topic, Some(&DataReaderQos::default()), NoListener, 0);

    let mut data_writer_qos = DataWriterQos::default();
    data_writer_qos.reliability = ReliabilityQosPolicy{kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos, max_blocking_time: DURATION_ZERO};
    let datawriter = publisher.create_datawriter(&helloworld_topic, Some(&data_writer_qos), NoListener, 0).expect("Error creating data writer");
    let data = HelloWorldType{id: 1, _msg: "Hello World!".to_string()};
    let handle = None;
    let timestamp = Time{sec: 1, nanosec: 2};
    datawriter.write_w_timestamp(data, handle, timestamp).expect("Error writing");


    // std::thread::sleep(std::time::Duration::from_secs(5))
}