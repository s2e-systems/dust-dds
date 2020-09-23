use std::any::Any;
use crate::topic::topic::Topic;
use crate::infrastructure::status::InconsistentTopicStatus;
use crate::infrastructure::listener::NoListener;

pub trait TopicListener : Any + Send + Sync {
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus,);
}

impl TopicListener for NoListener {
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus,) {
        todo!()
    }
}