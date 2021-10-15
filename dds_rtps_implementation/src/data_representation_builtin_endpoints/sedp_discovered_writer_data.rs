use rust_dds_api::builtin_topics::PublicationBuiltinTopicData;

use crate::{dds_type::DdsType, rtps_impl::rtps_writer_proxy_impl::RtpsWriterProxyImpl};

pub struct SedpDiscoveredWriterData {
    pub writer_proxy: RtpsWriterProxyImpl,
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
