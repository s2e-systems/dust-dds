use crate::types::DDSType;
use crate::topic::topic::Topic;
use crate::infrastructure::status::InconsistentTopicStatus;
use crate::infrastructure::qos::TopicQos;

pub trait TopicListener<T: DDSType> {
    fn on_inconsistent_topic(&self, _the_topic: dyn Topic<T, Listener=dyn TopicListener<T>, Qos=TopicQos>, _status: InconsistentTopicStatus,);
}