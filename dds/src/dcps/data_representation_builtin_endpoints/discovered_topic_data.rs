use crate::{builtin_topics::TopicBuiltinTopicData, infrastructure::type_support::TypeSupport};

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct DiscoveredTopicData {
    pub(crate) topic_builtin_topic_data: TopicBuiltinTopicData,
}
