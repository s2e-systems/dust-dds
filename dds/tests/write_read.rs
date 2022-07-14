use dds::{
    domain::{
        domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
    },
    domain_participant_factory::DomainParticipantFactoryImpl,
    publication::{data_writer::FooDataWriter, publisher::Publisher},
    subscription::{
        data_reader::{DataReader, FooDataReader},
        subscriber::Subscriber,
    },
    types::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
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
    let participant_factory = DomainParticipantFactoryImpl::get_instance();

    let participant1 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let topic = participant1
        .create_topic::<UserData>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let reader = subscriber.create_datareader(&topic, None, None, 0).unwrap();

    //Wait for reader to be aware of the user writer
    while reader
        .get_subscription_matched_status()
        .unwrap()
        .total_count
        < 1
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    writer.write(&UserData(8), None).unwrap();

    let mut samples = reader.read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE);
    while let Err(DdsError::NoData) = samples {
        std::thread::sleep(std::time::Duration::from_millis(50));
        samples = reader.read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
    }

    assert_eq!(samples.unwrap()[0].data.as_ref().unwrap(), &UserData(8));
}
