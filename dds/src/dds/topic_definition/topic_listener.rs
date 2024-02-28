use crate::{infrastructure::status::InconsistentTopicStatus, topic_definition::topic::Topic};

/// Listener associated with the ['Topic'] entity.
pub trait TopicListener {
    /// Method that is called when an inconsistent version of this topic is discovered.
    fn on_inconsistent_topic(&mut self, _the_topic: &Topic, _status: InconsistentTopicStatus) {}
}
