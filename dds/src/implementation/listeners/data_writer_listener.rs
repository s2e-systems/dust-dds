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

pub struct TriggerPublicationMatched {
    pub the_writer: DataWriterAsync<()>,
    pub status: PublicationMatchedStatus,
}
impl MailHandler<TriggerPublicationMatched> for DataWriterListenerActor {
    fn handle(&mut self, message: TriggerPublicationMatched) {
        self.listener
            .trigger_on_publication_matched(message.the_writer, message.status);
    }
}

pub struct TriggerOfferedIncompatibleQos {
    pub the_writer: DataWriterAsync<()>,
    pub status: OfferedIncompatibleQosStatus,
}
impl MailHandler<TriggerOfferedIncompatibleQos> for DataWriterListenerActor {
    fn handle(&mut self, message: TriggerOfferedIncompatibleQos) {
        self.listener
            .trigger_on_offered_incompatible_qos(message.the_writer, message.status);
    }
}

pub struct TriggerOfferedDeadlineMissed {
    pub the_writer: DataWriterAsync<()>,
    pub status: OfferedDeadlineMissedStatus,
}
impl MailHandler<TriggerOfferedDeadlineMissed> for DataWriterListenerActor {
    fn handle(&mut self, message: TriggerOfferedDeadlineMissed) {
        self.listener
            .trigger_on_offered_deadline_missed(message.the_writer, message.status);
    }
}
