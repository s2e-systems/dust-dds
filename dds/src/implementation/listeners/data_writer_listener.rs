use crate::{
    dds_async::data_writer::DataWriterAsync,
    implementation::any_data_writer_listener::AnyDataWriterListener,
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
    },
    runtime::actor::MailHandler,
};

pub struct DataWriterListenerActor {
    listener: Box<dyn AnyDataWriterListener>,
}

impl DataWriterListenerActor {
    pub fn new(listener: Box<dyn AnyDataWriterListener>) -> Self {
        Self { listener }
    }
}

pub enum DataWriterListenerMail {
    PublicationMatched {
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

impl MailHandler for DataWriterListenerActor {
    type Mail = DataWriterListenerMail;
    fn handle(&mut self, message: DataWriterListenerMail) {
        match message {
            DataWriterListenerMail::PublicationMatched { the_writer, status } => self
                .listener
                .trigger_on_publication_matched(the_writer, status),
            DataWriterListenerMail::OfferedIncompatibleQos { the_writer, status } => self
                .listener
                .trigger_on_offered_incompatible_qos(the_writer, status),
            DataWriterListenerMail::OfferedDeadlineMissed { the_writer, status } => self
                .listener
                .trigger_on_offered_deadline_missed(the_writer, status),
        }
    }
}
