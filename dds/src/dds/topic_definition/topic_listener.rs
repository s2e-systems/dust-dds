use crate::{
    runtime::DdsRuntime, dds_async::topic::TopicAsync,
    infrastructure::status::InconsistentTopicStatus,
};
use core::future::Future;

/// Listener associated with the [`Topic`] entity.
pub trait TopicListener<R: DdsRuntime> {
    /// Method that is called when an inconsistent version of this topic is discovered.
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: TopicAsync<R>,
        _status: InconsistentTopicStatus,
    ) -> impl Future<Output = ()> + Send {
        core::future::ready(())
    }
}
