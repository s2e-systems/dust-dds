use rust_dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    publication::{data_writer::DataWriter, publisher::Publisher},
    subscription::{data_reader::DataReader, subscriber::Subscriber},
    types::Time,
    DDSError,
};
use rust_dds_rtps_implementation::dds_type::{DdsDeserialize, DdsSerialize, DdsType};

#[derive(Debug, PartialEq)]
struct UserData(u8);

impl DdsType for UserData {
    fn type_name() -> &'static str {
        "UserData"
    }

    fn has_key() -> bool {
        false
    }
}

impl DdsSerialize for UserData {
    fn serialize<W: std::io::Write, E: rust_dds_rtps_implementation::dds_type::Endianness>(
        &self,
        mut writer: W,
    ) -> rust_dds::DDSResult<()> {
        writer
            .write(&[self.0])
            .map(|_| ())
            .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))
    }
}

impl<'de> DdsDeserialize<'de> for UserData {
    fn deserialize(buf: &mut &'de [u8]) -> rust_dds::DDSResult<Self> {
        Ok(UserData(buf[0]))
    }
}

#[test]
fn user_defined_write_read_auto_enable() {
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
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let mut writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let mut reader = subscriber
        .create_datareader(&topic, None, None, 0)
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

    writer
        .write_w_timestamp(&UserData(8), None, Time { sec: 0, nanosec: 0 })
        .unwrap();

    let mut samples = reader.read(1, &[], &[], &[]);
    while let Err(DDSError::NoData) = samples {
        std::thread::sleep(std::time::Duration::from_millis(500));
        samples = reader.read(1, &[], &[], &[])
    }

    assert_eq!(samples.unwrap().samples, vec![UserData(8)]);
}
