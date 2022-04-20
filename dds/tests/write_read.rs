use dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    publication::{data_writer::DataWriter, publisher::Publisher},
    subscription::{data_reader::{DataReader, ANY_SAMPLE}, subscriber::Subscriber},
    DdsError,
};
use dds_implementation::dds_type::{DdsDeserialize, DdsSerialize, DdsType};

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
    fn serialize<W: std::io::Write, E: dds_implementation::dds_type::Endianness>(
        &self,
        mut writer: W,
    ) -> dds::DdsResult<()> {
        writer
            .write(&[self.0])
            .map(|_| ())
            .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
    }
}

impl<'de> DdsDeserialize<'de> for UserData {
    fn deserialize(buf: &mut &'de [u8]) -> dds::DdsResult<Self> {
        Ok(UserData(buf[0]))
    }
}

#[test]
fn user_defined_write_read_auto_enable() {
    let domain_id = 8;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    // Wait for the participants to discover each other
    while participant1
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
        + participant2
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
        < 4
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let topic = participant1
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let reader = subscriber.create_datareader(&topic, None, None, 0).unwrap();

    //Wait for reader to be aware of the user writer
    while reader
        .as_ref()
        .upgrade()
        .unwrap()
        .rtps_reader
        .write_lock()
        .try_as_stateful_reader()
        .unwrap()
        .matched_writers
        .len()
        == 0
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    writer.write(&UserData(8), None).unwrap();

    let mut samples = reader.read(1, ANY_SAMPLE, &[], &[]);
    while let Err(DdsError::NoData) = samples {
        std::thread::sleep(std::time::Duration::from_millis(50));
        samples = reader.read(1, ANY_SAMPLE, &[], &[])
    }

    assert_eq!(samples.unwrap()[0].0, UserData(8));
}
