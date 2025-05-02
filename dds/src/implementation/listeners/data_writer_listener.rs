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
    TriggerPublicationMatched {
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

impl MailHandler<DataWriterListenerMail> for DataWriterListenerActor {
    fn handle(&mut self, message: DataWriterListenerMail) {
        match message {
            DataWriterListenerMail::TriggerPublicationMatched { the_writer, status } => self
                .listener
                .trigger_on_publication_matched(the_writer, status),
            DataWriterListenerMail::TriggerOfferedIncompatibleQos { the_writer, status } => self
                .listener
                .trigger_on_offered_incompatible_qos(the_writer, status),
            DataWriterListenerMail::TriggerOfferedDeadlineMissed { the_writer, status } => self
                .listener
                .trigger_on_offered_deadline_missed(the_writer, status),
        }
    }
}
