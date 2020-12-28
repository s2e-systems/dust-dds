use crate::types::DDSType;
use crate::dds::topic::topic::Topic;
use crate::dds::infrastructure::listener::Listener;
use crate::dds::infrastructure::status::InconsistentTopicStatus;

pub trait TopicListener<T: DDSType> : Listener{
    fn on_inconsistent_topic(&self, the_topic: Topic<T>, status: InconsistentTopicStatus);
}