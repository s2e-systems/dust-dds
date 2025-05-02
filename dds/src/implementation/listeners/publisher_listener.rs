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

pub enum PublisherListenerMail {
    TriggerOnPublicationMatched {
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    },
    TriggerOfferedIncompatibleQos {
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    },
    TriggerOfferedDeadlineMissed {
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    },
}

impl MailHandler<PublisherListenerMail> for PublisherListenerActor {
    fn handle(&mut self, message: PublisherListenerMail) {
        match message {
            PublisherListenerMail::TriggerOnPublicationMatched { the_writer, status } => {
                block_on(self.listener.on_publication_matched(the_writer, status))
            }
            PublisherListenerMail::TriggerOfferedIncompatibleQos { the_writer, status } => {
                block_on(
                    self.listener
                        .on_offered_incompatible_qos(the_writer, status),
                )
            }
            PublisherListenerMail::TriggerOfferedDeadlineMissed { the_writer, status } => {
                block_on(self.listener.on_offered_deadline_missed(the_writer, status))
            }
        }
    }
}
