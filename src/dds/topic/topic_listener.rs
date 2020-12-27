use crate::types::DDSType;
use crate::dds::topic::topic::Topic;
use crate::dds::infrastructure::status::InconsistentTopicStatus;

pub trait TopicListener<T: DDSType> {
    fn on_inconsistent_topic(&self, _the_topic: Topic<T>, _status: InconsistentTopicStatus);
}