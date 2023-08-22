use crate::{infrastructure::status::InconsistentTopicStatus, topic_definition::topic::Topic};

pub trait TopicListener {
    fn on_inconsistent_topic(&mut self, _the_topic: &Topic, _status: InconsistentTopicStatus) {}
}
