use crate::{
    dds_async::{data_writer::DataWriterAsync, publisher_listener::PublisherListenerAsync},
    infrastructure::status::PublicationMatchedStatus,
    runtime::{
        actor::{Mail, MailHandler},
        executor::block_on,
    },
};

pub struct PublisherListenerActor {
    listener: Box<dyn PublisherListenerAsync + Send>,
}

impl PublisherListenerActor {
    pub fn new(listener: Box<dyn PublisherListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

pub struct TriggerOnPublicationMatched {
    pub the_writer: DataWriterAsync<()>,
    pub status: PublicationMatchedStatus,
}
impl Mail for TriggerOnPublicationMatched {
    type Result = ();
}
impl MailHandler<TriggerOnPublicationMatched> for PublisherListenerActor {
    fn handle(
        &mut self,
        message: TriggerOnPublicationMatched,
    ) -> <TriggerOnPublicationMatched as Mail>::Result {
        block_on(
            self.listener
                .on_publication_matched(message.the_writer, message.status),
        )
    }
}
