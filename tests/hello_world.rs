use rust_rtps::types::{DDSType, TopicKind, InstanceHandle, Data, DURATION_ZERO};
use rust_rtps::dds::domain::domain_participant_factory::DomainParticipantFactory;
use rust_rtps::dds::topic::topic::Topic;
use rust_rtps::dds_infrastructure::qos::DataWriterQos;
use rust_rtps::dds_infrastructure::qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind};

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

    fn instance_handle(&self) -> InstanceHandle {
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, self.id]
    }

    fn serialize(&self) -> Data {
        vec![self.id]
    }

    fn deserialize(_data: Data) -> Self {
        todo!()
    }
}

#[test]
fn hello_world() {
    let participant = DomainParticipantFactory::create_participant(0, None).expect("Error creating participant");
    
    let publisher = participant.create_publisher(None).expect("Error creating publisher");

    let helloworld_topic: Topic<HelloWorldType> = participant.create_topic("HelloWorld".to_string(), None).expect("Error creating topic");

//     // let subscriber = participant.create_subscriber(None).expect("Error creating subscriber");
//     // let _datareader = subscriber.create_datareader(helloworld_topic, None);

    let mut data_writer_qos = DataWriterQos::default();
    data_writer_qos.reliability = ReliabilityQosPolicy{kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos, max_blocking_time: DURATION_ZERO};
    let _datawriter = publisher.create_datawriter(&helloworld_topic, Some(data_writer_qos)).expect("Error creating data writer");

// // //     let datawriter2 = publisher.lookup_datawriter::<HelloWorldType>(&"HelloWorld".to_string());

    // let data = HelloWorldType{id: 1, _msg: "Hello World!".to_string()};
    // let handle = None;
    // let timestamp = Time{sec: 1, nanosec: 2};
    // datawriter.write_w_timestamp(data, handle, timestamp).expect("Error writing");


// //     std::thread::sleep(std::time::Duration::from_secs(5))
}