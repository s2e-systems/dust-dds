use std::future::Future;

use crate::infrastructure::status::InconsistentTopicStatus;

use super::topic::TopicAsync;

/// Listener associated with the [`TopicAsync`] entity.
pub trait TopicListenerAsync {
    /// Method that is called when an inconsistent version of this topic is discovered.
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: TopicAsync,
        _status: InconsistentTopicStatus,
    ) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }
}
