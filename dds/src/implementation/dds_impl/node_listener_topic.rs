use crate::topic_definition::topic::AnyTopic;

#[derive(PartialEq, Eq, Debug)]
pub struct ListenerTopicNode;

impl AnyTopic for ListenerTopicNode {}
