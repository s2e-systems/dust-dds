use crate::api::dcps_psm::InconsistentTopicStatus;

use super::Topic;

pub trait TopicListener {
    fn on_inconsistent_topic(&mut self, _the_topic: &dyn Topic, _status: InconsistentTopicStatus) {}
}
