use crate::dds::topic::topic::Topic;
use crate::dds::infrastructure::status::InconsistentTopicStatus;

pub struct TopicListener{}

impl TopicListener{
    pub fn on_inconsistent_topic(
        _the_topic: Topic,
        _status: InconsistentTopicStatus, 
    ) {

    }
}