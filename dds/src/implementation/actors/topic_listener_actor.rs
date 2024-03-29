

use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{topic::TopicAsync, topic_listener::TopicListenerAsync},
    infrastructure::status::InconsistentTopicStatus,
};

pub struct TopicListenerActor {
    listener: Box<dyn TopicListenerAsync + Send>,
}

impl TopicListenerActor {
    pub fn new(listener: Box<dyn TopicListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl TopicListenerActor {
    async fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> () {
        self.listener.on_inconsistent_topic(the_topic, status).await
    }
}
