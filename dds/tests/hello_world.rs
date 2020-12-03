use dds::domain_participant_factory::DomainParticipantFactory;
use rust_dds_api::domain::DomainParticipant;
use rust_dds_api::infrastructure::qos::PublisherQos;
use rust_dds_api::publication::PublisherListener;
use rust_dds_api::topic::{TopicListener, Topic};
use rust_dds_api::types::DDSType;
// use rust_dds::domain::qos::DomainParticipantQos;
// use rust_dds::publication::qos::{DataWriterQos, PublisherQos};
// use rust_dds::subscription::qos::{DataReaderQos, SubscriberQos};
// use rust_dds::infrastructure::listener::NoListener;
// use rust_dds::infrastructure::qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind};
// use rust_dds::types::DDSType;
use rust_dds_api::types::{TopicKind, Time, DURATION_ZERO};

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

    fn instance_handle(&self) -> rust_dds_api::types::InstanceHandle {
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, self.id]
    }

    fn serialize(&self) -> rust_dds_api::types::Data {
        vec![self.id]
    }

    fn deserialize(_data: rust_dds_api::types::Data) -> Self {
        todo!()
    }
}

struct MockListener {}

impl PublisherListener for MockListener {}

impl<T:DDSType> TopicListener<T> for MockListener {
    fn on_inconsistent_topic(&self, _the_topic: &dyn rust_dds_api::topic::Topic<T>, _status: rust_dds_api::infrastructure::status::InconsistentTopicStatus,) {
        todo!()
    }
}
#[test]
fn hello_world() {
    let participant = DomainParticipantFactory::new(0).expect("Error creating participant");
    
    let publisher = participant.create_publisher(Some(&PublisherQos::default()), MockListener{}, 0).expect("Error creating publisher");
    let helloworld_topic : Box<dyn Topic<HelloWorldType>> = participant.create_topic("HelloWorld".to_string(), None, MockListener{}, 0).expect("Error creating topic");

//     let subscriber = participant.create_subscriber(Some(&SubscriberQos::default()), NoListener, 0).expect("Error creating subscriber");
//     let _datareader = subscriber.create_datareader(&helloworld_topic, Some(&DataReaderQos::default()), NoListener, 0);

//     let mut data_writer_qos = DataWriterQos::default();
//     data_writer_qos.reliability = ReliabilityQosPolicy{kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos, max_blocking_time: DURATION_ZERO};
//     let datawriter = publisher.create_datawriter(&helloworld_topic, Some(&data_writer_qos), NoListener, 0).expect("Error creating data writer");

//     let datawriter2 = publisher.lookup_datawriter::<HelloWorldType>(&"HelloWorld".to_string());

//     let data = HelloWorldType{id: 1, _msg: "Hello World!".to_string()};
//     let handle = None;
//     let timestamp = Time{sec: 1, nanosec: 2};
//     datawriter.write_w_timestamp(data, handle, timestamp).expect("Error writing");


//     std::thread::sleep(std::time::Duration::from_secs(5))
}