use crate::{
    dds_async::data_reader::DataReaderAsync,
    implementation::any_data_reader_listener::AnyDataReaderListener,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::actor::MailHandler,
};

pub struct DataReaderListenerActor {
    listener: Box<dyn AnyDataReaderListener>,
}

impl DataReaderListenerActor {
    pub fn new(listener: Box<dyn AnyDataReaderListener>) -> Self {
        Self { listener }
    }
}
pub struct TriggerDataAvailable {
    pub the_reader: DataReaderAsync<()>,
}
impl MailHandler<TriggerDataAvailable> for DataReaderListenerActor {
    fn handle(&mut self, message: TriggerDataAvailable) {
        self.listener.trigger_on_data_available(message.the_reader);
    }
}

pub struct TriggerRequestedDeadlineMissed {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedDeadlineMissedStatus,
}
impl MailHandler<TriggerRequestedDeadlineMissed> for DataReaderListenerActor {
    fn handle(&mut self, message: TriggerRequestedDeadlineMissed) {
        self.listener
            .trigger_on_requested_deadline_missed(message.the_reader, message.status);
    }
}

pub struct TriggerSampleRejected {
    pub the_reader: DataReaderAsync<()>,
    pub status: SampleRejectedStatus,
}
impl MailHandler<TriggerSampleRejected> for DataReaderListenerActor {
    fn handle(&mut self, message: TriggerSampleRejected) {
        self.listener
            .trigger_on_sample_rejected(message.the_reader, message.status);
    }
}

pub struct TriggerSubscriptionMatched {
    pub the_reader: DataReaderAsync<()>,
    pub status: SubscriptionMatchedStatus,
}
impl MailHandler<TriggerSubscriptionMatched> for DataReaderListenerActor {
    fn handle(&mut self, message: TriggerSubscriptionMatched) {
        self.listener
            .trigger_on_subscription_matched(message.the_reader, message.status);
    }
}

pub struct TriggerRequestedIncompatibleQos {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedIncompatibleQosStatus,
}
impl MailHandler<TriggerRequestedIncompatibleQos> for DataReaderListenerActor {
    fn handle(&mut self, message: TriggerRequestedIncompatibleQos) {
        self.listener
            .trigger_on_requested_incompatible_qos(message.the_reader, message.status);
    }
}
