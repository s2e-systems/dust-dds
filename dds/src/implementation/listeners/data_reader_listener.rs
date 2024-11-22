use crate::{
    dds_async::data_reader::DataReaderAsync,
    implementation::any_data_reader_listener::AnyDataReaderListener,
    runtime::actor::{Mail, MailHandler},
};

pub struct DataReaderListenerActor {
    listener: Box<dyn AnyDataReaderListener>,
}

impl DataReaderListenerActor {
    pub fn new(listener: Box<dyn AnyDataReaderListener>) -> Self {
        Self { listener }
    }
}
pub struct TriggerOnDataAvailable {
    pub the_reader: DataReaderAsync<()>,
}
impl Mail for TriggerOnDataAvailable {
    type Result = ();
}
impl MailHandler<TriggerOnDataAvailable> for DataReaderListenerActor {
    fn handle(
        &mut self,
        message: TriggerOnDataAvailable,
    ) -> <TriggerOnDataAvailable as Mail>::Result {
        self.listener.trigger_on_data_available(message.the_reader);
    }
}
