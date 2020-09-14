use std::any::Any;
use crate::dds::topic::topic::Topic;
use crate::dds::infrastructure::status::InconsistentTopicStatus;

pub trait TopicListener : Any + Send + Sync {
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus,);
}