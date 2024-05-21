use crate::{
    dds_async::{topic::TopicAsync, topic_listener::TopicListenerAsync},
    implementation::actor::{Mail, MailHandler},
    infrastructure::status::InconsistentTopicStatus,
};

pub struct TopicListenerActor {
    listener: Option<Box<dyn TopicListenerAsync + Send>>,
}

impl TopicListenerActor {
    pub fn new(listener: Option<Box<dyn TopicListenerAsync + Send>>) -> Self {
        Self { listener }
    }
}

pub struct OnInconsistentTopic {
    pub the_topic: TopicAsync,
    pub status: InconsistentTopicStatus,
}
impl Mail for OnInconsistentTopic {
    type Result = ();
}
impl MailHandler<OnInconsistentTopic> for TopicListenerActor {
    async fn handle(
        &mut self,
        message: OnInconsistentTopic,
    ) -> <OnInconsistentTopic as Mail>::Result {
        if let Some(l) = &mut self.listener {
            l.on_inconsistent_topic(message.the_topic, message.status)
                .await
        }
    }
}
