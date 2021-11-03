use std::io::Write;

use rust_dds_api::builtin_topics::TopicBuiltinTopicData;

use crate::dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness};

pub struct SedpDiscoveredTopicData {
    pub topic_builtin_topic_data: TopicBuiltinTopicData,
}

impl DdsType for SedpDiscoveredTopicData {
    fn type_name() -> &'static str {
        "SedpDiscoveredTopicData"
    }

    fn has_key() -> bool {
        true
    }
}

impl DdsSerialize for SedpDiscoveredTopicData {
    fn serialize<W: Write, E: Endianness>(
        &self,
        _writer: W,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredTopicData {
    fn deserialize(_buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
        todo!()
    }
}
