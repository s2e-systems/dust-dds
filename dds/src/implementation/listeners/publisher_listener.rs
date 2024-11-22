use crate::{
    dds_async::{data_writer::DataWriterAsync, publisher_listener::PublisherListenerAsync},
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
    },
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

pub struct TriggerOfferedIncompatibleQos {
    pub the_writer: DataWriterAsync<()>,
    pub status: OfferedIncompatibleQosStatus,
}
impl Mail for TriggerOfferedIncompatibleQos {
    type Result = ();
}
impl MailHandler<TriggerOfferedIncompatibleQos> for PublisherListenerActor {
    fn handle(
        &mut self,
        message: TriggerOfferedIncompatibleQos,
    ) -> <TriggerOfferedIncompatibleQos as Mail>::Result {
        block_on(
            self.listener
                .on_offered_incompatible_qos(message.the_writer, message.status),
        )
    }
}
pub struct TriggerOfferedDeadlineMissed {
    pub the_writer: DataWriterAsync<()>,
    pub status: OfferedDeadlineMissedStatus,
}
impl Mail for TriggerOfferedDeadlineMissed {
    type Result = ();
}
impl MailHandler<TriggerOfferedDeadlineMissed> for PublisherListenerActor {
    fn handle(
        &mut self,
        message: TriggerOfferedDeadlineMissed,
    ) -> <TriggerOfferedDeadlineMissed as Mail>::Result {
        block_on(
            self.listener
                .on_offered_deadline_missed(message.the_writer, message.status),
        )
    }
}
