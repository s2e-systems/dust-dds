use rust_dds::{
    domain::domain_participant::DomainParticipant,
    domain_participant_factory::DomainParticipantFactory,
    publication::{data_writer::DataWriter, publisher::Publisher},
    types::Time,
    DDSError,
};
use rust_dds_rtps_implementation::dds_type::{DdsDeserialize, DdsSerialize, DdsType};

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
    let domain_id = 8;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, None, None, 0)
        .unwrap();

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", None, None, 0)
        .unwrap();

    let publisher = participant.create_publisher(None, None, 0).unwrap();
    let mut writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };
    writer
        .write_w_timestamp(&hello_world, None, Time { sec: 0, nanosec: 0 })
        .unwrap();
}
