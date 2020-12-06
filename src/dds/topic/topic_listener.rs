use crate::types::DDSType;
use crate::topic::topic::Topic;
use crate::infrastructure::status::InconsistentTopicStatus;

pub trait TopicListener<T: DDSType> {
    fn on_inconsistent_topic(&self, _the_topic: &dyn Topic<T>, _status: InconsistentTopicStatus,);
}