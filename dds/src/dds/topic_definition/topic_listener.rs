use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{topic::TopicAsync, topic_listener::TopicListenerAsync},
    infrastructure::status::InconsistentTopicStatus,
    topic_definition::topic::Topic,
};

/// Listener associated with the [`Topic`] entity.
pub trait TopicListener {
    /// Method that is called when an inconsistent version of this topic is discovered.
    fn on_inconsistent_topic(&mut self, _the_topic: Topic, _status: InconsistentTopicStatus) {}
}

impl TopicListenerAsync for Box<dyn TopicListener + Send> {
    fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        TopicListener::on_inconsistent_topic(self.as_mut(), Topic::new(the_topic), status);
        Box::pin(std::future::ready(()))
    }
}
