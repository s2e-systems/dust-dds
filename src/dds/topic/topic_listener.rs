use std::any::Any;
use crate::dds::topic::topic::Topic;
use crate::dds::infrastructure::status::InconsistentTopicStatus;
use crate::dds::infrastructure::listener::NoListener;

pub trait TopicListener : Any + Send + Sync {
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus,);
}

impl TopicListener for NoListener {
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus,) {
        todo!()
    }
}