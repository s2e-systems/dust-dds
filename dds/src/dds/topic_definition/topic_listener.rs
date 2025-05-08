use crate::{
    dcps::runtime::DdsRuntime, dds_async::topic::TopicAsync,
    infrastructure::status::InconsistentTopicStatus,
};
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

/// Listener associated with the [`Topic`] entity.
pub trait TopicListener<R: DdsRuntime> {
    /// Method that is called when an inconsistent version of this topic is discovered.
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: TopicAsync<R>,
        _status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(core::future::ready(()))
    }
}
