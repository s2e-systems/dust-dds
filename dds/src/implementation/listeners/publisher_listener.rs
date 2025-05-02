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
    OnPublicationMatched {
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    },
    OfferedIncompatibleQos {
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    },
    OfferedDeadlineMissed {
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    },
}

impl MailHandler for PublisherListenerActor {
    type Mail = PublisherListenerMail;
    fn handle(&mut self, message: PublisherListenerMail) {
        match message {
            PublisherListenerMail::OnPublicationMatched { the_writer, status } => {
                block_on(self.listener.on_publication_matched(the_writer, status))
            }
            PublisherListenerMail::OfferedIncompatibleQos { the_writer, status } => block_on(
                self.listener
                    .on_offered_incompatible_qos(the_writer, status),
            ),
            PublisherListenerMail::OfferedDeadlineMissed { the_writer, status } => {
                block_on(self.listener.on_offered_deadline_missed(the_writer, status))
            }
        }
    }
}
