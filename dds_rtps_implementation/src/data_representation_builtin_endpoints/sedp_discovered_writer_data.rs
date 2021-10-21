use rust_dds_api::builtin_topics::PublicationBuiltinTopicData;
use rust_rtps_pim::{behavior::reader::writer_proxy::RtpsWriterProxy, structure::types::Locator};

use crate::dds_type::DdsType;

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
