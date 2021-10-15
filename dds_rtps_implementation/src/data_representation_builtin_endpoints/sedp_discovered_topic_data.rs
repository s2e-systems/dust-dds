use rust_dds_api::builtin_topics::TopicBuiltinTopicData;

use crate::dds_type::DdsType;

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
