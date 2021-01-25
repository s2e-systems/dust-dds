use rust_dds_types::DDSType;

use crate::infrastructure::{listener::Listener, status::InconsistentTopicStatus};

use super::topic::Topic;

pub trait TopicListener<T: DDSType>: Listener {
    fn on_inconsistent_topic(&self, the_topic: &dyn Topic<T>, status: InconsistentTopicStatus);
}
