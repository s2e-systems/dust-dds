use crate::dds::topic::topic::Topic;
use crate::dds::infrastructure::status::InconsistentTopicStatus;

pub trait TopicListener{
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus,);
}