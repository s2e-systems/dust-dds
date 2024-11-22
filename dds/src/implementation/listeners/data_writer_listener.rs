use crate::{
    dds_async::data_writer::DataWriterAsync,
    implementation::any_data_writer_listener::AnyDataWriterListener,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
    runtime::actor::{Mail, MailHandler},
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
impl Mail for TriggerPublicationMatched {
    type Result = ();
}
impl MailHandler<TriggerPublicationMatched> for DataWriterListenerActor {
    fn handle(
        &mut self,
        message: TriggerPublicationMatched,
    ) -> <TriggerPublicationMatched as Mail>::Result {
        self.listener
            .trigger_on_publication_matched(message.the_writer, message.status);
    }
}

pub struct TriggerOfferedIncompatibleQos {
    pub the_writer: DataWriterAsync<()>,
    pub status: OfferedIncompatibleQosStatus,
}
impl Mail for TriggerOfferedIncompatibleQos {
    type Result = ();
}
impl MailHandler<TriggerOfferedIncompatibleQos> for DataWriterListenerActor {
    fn handle(
        &mut self,
        message: TriggerOfferedIncompatibleQos,
    ) -> <TriggerOfferedIncompatibleQos as Mail>::Result {
        self.listener
            .trigger_on_offered_incompatible_qos(message.the_writer, message.status);
    }
}
