use tracing::info;

use crate::{
    dds_async::topic::TopicAsync, infrastructure::status::InconsistentTopicStatus,
    runtime::DdsRuntime,
};
use core::future::Future;

/// Listener associated with the [`Topic`] entity.
pub trait TopicListener<R: DdsRuntime> {
    /// Method that is called when an inconsistent version of this topic is discovered.
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: TopicAsync<R>,
        status: InconsistentTopicStatus,
    ) -> impl Future<Output = ()> + Send {
        info!(?status, "on_inconsistent_topic");
        core::future::ready(())
    }
}
