use crate::{
    dds_async::{subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync},
    runtime::{
        actor::{Mail, MailHandler},
        executor::block_on,
    },
};

pub struct SubscriberListenerActor {
    listener: Box<dyn SubscriberListenerAsync + Send>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

pub struct TriggerOnDataOnReaders {
    pub the_subscriber: SubscriberAsync,
}
impl Mail for TriggerOnDataOnReaders {
    type Result = ();
}
impl MailHandler<TriggerOnDataOnReaders> for SubscriberListenerActor {
    fn handle(
        &mut self,
        message: TriggerOnDataOnReaders,
    ) -> <TriggerOnDataOnReaders as Mail>::Result {
        block_on(self.listener.on_data_on_readers(message.the_subscriber));
    }
}
