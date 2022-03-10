use cdr::CdrBe;
use rust_dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    publication::{data_writer::DataWriter, publisher::Publisher},
    types::Time,
    DDSError,
};
use rust_dds_rtps_implementation::dds_type::{DdsDeserialize, DdsSerialize, DdsType};
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

impl DdsSerialize for HelloWorldType {
    fn serialize<W: std::io::Write, E: rust_dds_rtps_implementation::dds_type::Endianness>(
        &self,
        mut writer: W,
    ) -> rust_dds::DDSResult<()> {
        writer
            .write(
                cdr::serialize::<_, _, CdrBe>(self, cdr::Infinite)
                    .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?
                    .as_slice(),
            )
            .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;
        Ok(())
    }
}

impl<'de> DdsDeserialize<'de> for HelloWorldType {
    fn deserialize(buf: &mut &'de [u8]) -> rust_dds::DDSResult<Self> {
        cdr::deserialize::<HelloWorldType>(buf)
            .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))
    }
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    while participant
        .get_builtin_subscriber()
        .unwrap()
        .as_ref()
        .upgrade()
        .unwrap()
        .read_lock()
        .data_reader_list
        .iter()
        .filter_map(|r| {
            r.write_lock()
                .rtps_reader
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
    println!("Matched other participant");

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    while writer
        .as_ref()
        .upgrade()
        .unwrap()
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
    println!("Matched with reader");

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };
    writer
        .write_w_timestamp(&hello_world, None, Time { sec: 0, nanosec: 0 })
        .unwrap();

    std::thread::sleep(std::time::Duration::new(2, 0));
}
