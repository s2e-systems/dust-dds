use dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    publication::{data_writer::DataWriter, publisher::Publisher},
};
use dds_implementation::dds_type::{
    DdsSerde, DdsType,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct HelloWorldType {
    id: u8,
    msg: String,
}

impl DdsType for HelloWorldType {
    fn type_name() -> &'static str {
        "HelloWorldType"
    }

    fn has_key() -> bool {
        false
    }
}

impl DdsSerde for HelloWorldType {}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();
    println!("{:?} [P] Created participant", std::time::SystemTime::now());

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();
    println!("{:?} [P] Created writer", std::time::SystemTime::now());

    while participant
        .get_builtin_subscriber()
        .unwrap()
        .as_ref()
        .upgrade()
        .unwrap()
        .data_reader_list
        .read_lock()
        .iter()
        .filter_map(|r| {
            r.rtps_reader
                .write_lock()
                .try_as_stateful_reader()
                .ok()
                .map(|sr| sr.matched_writers.len())
        })
        .next()
        .unwrap()
        < 2
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("{:?} [P] Matched participant", std::time::SystemTime::now());

    while writer
        .as_ref()
        .upgrade()
        .unwrap()
        .extended_rtps_writer
        .write_lock()
        .rtps_writer
        .try_as_stateful_writer()
        .unwrap()
        .matched_readers
        .len()
        == 0
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("{:?} [P] Matched with reader", std::time::SystemTime::now());

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };
    writer.write(&hello_world, None).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(15));
}
