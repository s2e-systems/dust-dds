use rust_dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind},
    publication::{data_writer::DataWriter, publisher::Publisher},
    subscription::{data_reader::DataReader, subscriber::Subscriber},
    types::Time,
    DDSError,
};
use rust_dds_rtps_implementation::dds_type::{DdsDeserialize, DdsSerialize, DdsType};

#[derive(Debug, PartialEq)]
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
            .write(&[self.id])
            .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;
        writer
            .write(self.msg.as_bytes())
            .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;
        Ok(())
    }
}

impl<'de> DdsDeserialize<'de> for HelloWorldType {
    fn deserialize(buf: &mut &'de [u8]) -> rust_dds::DDSResult<Self> {
        let id = buf[0];
        let msg = String::from_utf8(buf[1..].iter().filter(|&&c| c > 0).cloned().collect())
            .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;
        Ok(HelloWorldType { id, msg })
    }
}

fn main() {
    let domain_id = 7;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    // Wait for the participants to discover each other
    std::thread::sleep(std::time::Duration::from_millis(600));

    let topic = participant1
        .create_topic::<HelloWorldType>("HelloWorld", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let mut writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let mut reader_qos = DataReaderQos::default();
    reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let mut reader = subscriber
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

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };
    writer
        .write_w_timestamp(&hello_world, None, Time { sec: 0, nanosec: 0 })
        .unwrap();

    let mut samples = reader.read(1, &[], &[], &[]);
    while let Err(DDSError::NoData) = samples {
        std::thread::sleep(std::time::Duration::from_millis(50));
        samples = reader.read(1, &[], &[], &[])
    }

    assert_eq!(hello_world, samples.unwrap().samples[0]);
}
