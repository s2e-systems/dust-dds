use dds::dds_type::{DdsSerde, DdsType};
use dds::domain::domain_participant_factory::DdsDomainParticipantFactory;
use dds::return_type::DdsError;
use dds::subscription::data_reader::DataReader;
use dds::{
    dcps_psm::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind},
    subscription::data_reader_listener::DataReaderListener,
};
use dds_derive::DdsType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, DdsType)]
struct HelloWorldType {
    id: u8,
    msg: String,
}

impl DdsSerde for HelloWorldType {}

struct ExampleListener;

impl DataReaderListener for ExampleListener {
    type Foo = HelloWorldType;

    fn on_data_available(&mut self, the_reader: &DataReader<Self::Foo>) {
        let sample = the_reader
            .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();
        println!(
            "Data id: {:?} Msg: {:?}",
            sample[0].data.as_ref().unwrap().id,
            sample[0].data.as_ref().unwrap().msg
        )
    }
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();
    println!("{:?} [S] Created participant", std::time::SystemTime::now());

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", None, None, 0)
        .unwrap();

    let mut qos = DataReaderQos::default();
    qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

    let subscriber = participant.create_subscriber(None, None, 0).unwrap();

    let reader = subscriber
        .create_datareader(&topic, Some(qos), Some(Box::new(ExampleListener)), 0)
        .unwrap();
    println!("{:?} [S] Created reader", std::time::SystemTime::now());

    while reader.get_matched_publications().unwrap().len() == 0 {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("{:?} [S] Matched with writer", std::time::SystemTime::now());

    let mut samples = reader.read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE);
    while let Err(DdsError::NoData) = samples {
        std::thread::sleep(std::time::Duration::from_millis(50));
        samples = reader.read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
    }
    println!("{:?} [S] Received data", std::time::SystemTime::now());
    let hello_world_sample = samples.unwrap();
    let hello_world = hello_world_sample[0].data.as_ref().unwrap();
    assert_eq!(8, hello_world.id);
    assert_eq!("Hello world!", hello_world.msg);
}
