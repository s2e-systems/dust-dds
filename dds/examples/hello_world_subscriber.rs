use cdr::CdrBe;
use rust_dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind},
    subscription::{data_reader::DataReader, subscriber::Subscriber},
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
    println!("Matched participant");

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", None, None, 0)
        .unwrap();

    let mut reader_qos = DataReaderQos::default();
    reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
    let subscriber = participant.create_subscriber(None, None, 0).unwrap();
    let reader = subscriber
        .create_datareader(&topic, Some(reader_qos), None, 0)
        .unwrap();

    // Wait for reader to be aware of the user writer
    while reader
        .as_ref()
        .upgrade()
        .unwrap()
        .write_lock()
        .rtps_reader
        .try_as_stateful_reader()
        .unwrap()
        .matched_writers
        .len()
        == 0
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("Matched with writer");

    let mut samples = reader.read(1, &[], &[], &[]);
    while let Err(DDSError::NoData) = samples {
        std::thread::sleep(std::time::Duration::from_millis(50));
        samples = reader.read(1, &[], &[], &[])
    }
    println!("Received data");

    let hello_world = &samples.unwrap()[0].0;
    assert_eq!(8, hello_world.id);
    assert_eq!("Hello world!", hello_world.msg);
}
