use crate::dds_async::topic_listener::TopicListenerAsync;

pub struct TopicListenerActor {
    _listener: Box<dyn TopicListenerAsync + Send>,
}

impl TopicListenerActor {
    pub fn new(listener: Box<dyn TopicListenerAsync + Send>) -> Self {
        Self {
            _listener: listener,
        }
    }
}
