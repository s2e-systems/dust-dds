use crate::{
    dcps_psm::InconsistentTopicStatus, dds_type::DDSType, infrastructure::listener::Listener,
};

use super::topic::Topic;

pub trait TopicListener<T: DDSType>: Listener {
    fn on_inconsistent_topic(&self, the_topic: &dyn Topic<T>, status: InconsistentTopicStatus);
}
