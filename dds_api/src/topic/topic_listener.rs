use crate::dcps_psm::InconsistentTopicStatus;

pub trait TopicListener {
    fn on_inconsistent_topic(&self, _status: InconsistentTopicStatus) {}
}
