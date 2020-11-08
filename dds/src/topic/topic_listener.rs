use crate::types::DDSType;
use crate::topic::topic::Topic;
use crate::infrastructure::status::InconsistentTopicStatus;
use crate::infrastructure::listener::NoListener;

pub trait TopicListener<T: DDSType> {
    fn on_inconsistent_topic(&self, _the_topic: Topic<T>, _status: InconsistentTopicStatus,);
}

impl<T: DDSType> TopicListener<T> for NoListener {
    fn on_inconsistent_topic(&self, _the_topic: Topic<T>, _status: InconsistentTopicStatus,) {
        todo!()
    }
}