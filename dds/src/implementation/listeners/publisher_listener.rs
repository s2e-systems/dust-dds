use crate::{
    dds_async::{data_writer::DataWriterAsync, publisher_listener::PublisherListenerAsync},
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
    },
    runtime::{actor::MailHandler, executor::block_on},
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
impl MailHandler<TriggerOnPublicationMatched> for PublisherListenerActor {
    fn handle(&mut self, message: TriggerOnPublicationMatched) {
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
impl MailHandler<TriggerOfferedIncompatibleQos> for PublisherListenerActor {
    fn handle(&mut self, message: TriggerOfferedIncompatibleQos) {
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
impl MailHandler<TriggerOfferedDeadlineMissed> for PublisherListenerActor {
    fn handle(&mut self, message: TriggerOfferedDeadlineMissed) {
        block_on(
            self.listener
                .on_offered_deadline_missed(message.the_writer, message.status),
        )
    }
}
