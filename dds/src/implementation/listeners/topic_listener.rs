use crate::{dds_async::topic_listener::TopicListenerAsync, runtime::actor::MailHandler};

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

pub enum TopicListenerActorMail {}

impl MailHandler for TopicListenerActor {
    type Mail = TopicListenerActorMail;

    async fn handle(&mut self, message: Self::Mail) {
        match message {}
    }
}
