use std::io::Write;

use rust_dds_api::{builtin_topics::PublicationBuiltinTopicData, return_type::DDSResult};
use rust_rtps_pim::{behavior::reader::writer_proxy::RtpsWriterProxy, structure::types::Locator};

use crate::dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness};

pub struct SedpDiscoveredWriterData {
    pub writer_proxy: RtpsWriterProxy<Vec<Locator>>,
    pub publication_builtin_topic_data: PublicationBuiltinTopicData,
}

impl DdsType for SedpDiscoveredWriterData {
    fn type_name() -> &'static str {
        "SedpDiscoveredWriterData"
    }

    fn has_key() -> bool {
        true
    }
}

impl DdsSerialize for SedpDiscoveredWriterData {
    fn serialize<W: Write, E: Endianness>(&self, mut writer: W) -> DDSResult<()> {
        writer.write(&[0, 0, 0, 0, 1, 2, 3, 4]).unwrap();
        Ok(())
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredWriterData {
    fn deserialize(_buf: &mut &'_ [u8]) -> DDSResult<Self> {
        todo!()
    }
}
