use rust_dds::domain_participant_factory::DomainParticipantFactory;
use rust_dds_api::{
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        qos::DataWriterQos,
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
    },
    publication::publisher::Publisher,
};
use rust_dds_types::{DDSType, Data, InstanceHandle, TopicKind, DURATION_ZERO};

struct HelloWorldType {
    id: u8,
    _msg: String,
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
    use rust_dds_api::infrastructure::entity::Entity;
    use rust_dds_types::Time;

    let participant = DomainParticipantFactory::create_participant(0, None, None, 0)
        .expect("Error creating participant");

    let publisher = participant
        .create_publisher(None, None, 0)
        .expect("Error creating publisher");

    let helloworld_topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", None, None, 0)
        .expect("Error creating topic");

    // let subscriber = participant.create_subscriber(None).expect("Error creating subscriber");
    // let _datareader = subscriber.create_datareader(helloworld_topic, None);

    let mut data_writer_qos = DataWriterQos::default();
    data_writer_qos.reliability = ReliabilityQosPolicy {
        kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
        max_blocking_time: DURATION_ZERO,
    };
    let datawriter = publisher
        .create_datawriter::<HelloWorldType>(&helloworld_topic, Some(data_writer_qos), None, 0)
        .expect("Error creating data writer");

    // // //     let datawriter2 = publisher.lookup_datawriter::<HelloWorldType>(&"HelloWorld".to_string());

    // let data = HelloWorldType {
    //     id: 1,
    //     _msg: "Hello World!".to_string(),
    // };
    // let handle = None;
    // let timestamp = Time { sec: 1, nanosec: 2 };
    // datawriter
    //     .write_w_timestamp(data, handle, timestamp)
    //     .expect("Error writing");

    // participant.enable().expect("Error enabling participant");

    // let data = HelloWorldType {
    //     id: 2,
    //     _msg: "Hello World!".to_string(),
    // };
    // datawriter
    //     .write_w_timestamp(data, handle, Time { sec: 1, nanosec: 2 })
    //     .expect("Error writing");

    // std::thread::sleep(std::time::Duration::from_secs(5));

    // let data = HelloWorldType {
    //     id: 3,
    //     _msg: "Hello World!".to_string(),
    // };
    // datawriter
    //     .write_w_timestamp(data, handle, Time { sec: 1, nanosec: 2 })
    //     .expect("Error writing");

    // std::thread::sleep(std::time::Duration::from_secs(5));
}
