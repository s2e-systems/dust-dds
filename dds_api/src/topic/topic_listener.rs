use crate::{
    dcps_psm::InconsistentTopicStatus, infrastructure::listener::Listener,
};

use super::topic::Topic;

pub trait TopicListener: Listener {
    fn on_inconsistent_topic(&self, the_topic: &dyn Topic, status: InconsistentTopicStatus);
}
