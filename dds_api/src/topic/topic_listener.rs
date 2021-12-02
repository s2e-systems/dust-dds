use crate::dcps_psm::InconsistentTopicStatus;

use super::topic::Topic;

pub trait TopicListener {
    type DataPIM;
    fn on_inconsistent_topic(
        &self,
        the_topic: &dyn Topic<Self::DataPIM>,
        status: InconsistentTopicStatus,
    );
}
