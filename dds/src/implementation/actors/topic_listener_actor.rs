use std::{future::Future, pin::Pin};

use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{topic::TopicAsync, topic_listener::TopicListenerAsync},
    infrastructure::status::InconsistentTopicStatus,
};

pub trait TopicListenerAsyncDyn {
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: TopicAsync,
        _status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

impl<T> TopicListenerAsyncDyn for T
where
    T: TopicListenerAsync + Send,
{
    fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async { self.on_inconsistent_topic(the_topic, status).await })
    }
}

pub struct TopicListenerActor {
    listener: Box<dyn TopicListenerAsyncDyn + Send>,
}

impl TopicListenerActor {
    pub fn new(listener: Box<dyn TopicListenerAsyncDyn + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl TopicListenerActor {
    async fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) {
        self.listener.on_inconsistent_topic(the_topic, status).await
    }
}
