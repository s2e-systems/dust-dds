use cdr::CdrBe;
use dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind},
    subscription::{
        data_reader::DataReader, data_reader_listener::DataReaderListener, subscriber::Subscriber,
    },
    DDSError,
};
use dds_implementation::dds_type::{DdsDeserialize, DdsSerialize, DdsType};
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

struct ExampleListener;

impl DataReaderListener for ExampleListener {
    type Foo = HelloWorldType;

    fn on_data_available(&self, the_reader: &dyn DataReader<Self::Foo>) {
        let sample = the_reader.read(1, &[], &[], &[]).unwrap();
        println!("Data id: {:?} Msg: {:?}", sample[0].0.id, sample[0].0.msg)
    }
}

impl DdsSerialize for HelloWorldType {
    fn serialize<W: std::io::Write, E: dds_implementation::dds_type::Endianness>(
        &self,
        mut writer: W,
    ) -> dds::DDSResult<()> {
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
    fn deserialize(buf: &mut &'de [u8]) -> dds::DDSResult<Self> {
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
    println!("{:?} [S] Matched participant", std::time::SystemTime::now());

    // Wait for reader to be aware of the user writer
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
    println!("{:?} [S] Matched with writer", std::time::SystemTime::now());

    let mut samples = reader.read(1, &[], &[], &[]);
    while let Err(DDSError::NoData) = samples {
        std::thread::sleep(std::time::Duration::from_millis(50));
        samples = reader.read(1, &[], &[], &[])
    }
    println!("{:?} [S] Received data", std::time::SystemTime::now());

    let hello_world = &samples.unwrap()[0].0;
    assert_eq!(8, hello_world.id);
    assert_eq!("Hello world!", hello_world.msg);
}
