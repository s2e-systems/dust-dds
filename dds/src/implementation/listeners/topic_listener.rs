use crate::dds_async::topic_listener::TopicListenerAsync;

pub struct TopicListenerActor {
    listener: Box<dyn TopicListenerAsync + Send>,
}

impl TopicListenerActor {
    pub fn new(listener: Box<dyn TopicListenerAsync + Send>) -> Self {
        Self { listener }
    }
}
