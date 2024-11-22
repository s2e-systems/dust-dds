use crate::{
    dds_async::data_reader::DataReaderAsync,
    implementation::any_data_reader_listener::AnyDataReaderListener,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
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

pub struct TriggerOnRequestedDeadlineMissed {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedDeadlineMissedStatus,
}
impl Mail for TriggerOnRequestedDeadlineMissed {
    type Result = ();
}
impl MailHandler<TriggerOnRequestedDeadlineMissed> for DataReaderListenerActor {
    fn handle(
        &mut self,
        message: TriggerOnRequestedDeadlineMissed,
    ) -> <TriggerOnRequestedDeadlineMissed as Mail>::Result {
        self.listener
            .trigger_on_requested_deadline_missed(message.the_reader, message.status);
    }
}

pub struct TriggerOnSampleRejected {
    pub the_reader: DataReaderAsync<()>,
    pub status: SampleRejectedStatus,
}
impl Mail for TriggerOnSampleRejected {
    type Result = ();
}
impl MailHandler<TriggerOnSampleRejected> for DataReaderListenerActor {
    fn handle(
        &mut self,
        message: TriggerOnSampleRejected,
    ) -> <TriggerOnSampleRejected as Mail>::Result {
        self.listener
            .trigger_on_sample_rejected(message.the_reader, message.status);
    }
}

pub struct TriggerOnSubscriptionMatched {
    pub the_reader: DataReaderAsync<()>,
    pub status: SubscriptionMatchedStatus,
}
impl Mail for TriggerOnSubscriptionMatched {
    type Result = ();
}
impl MailHandler<TriggerOnSubscriptionMatched> for DataReaderListenerActor {
    fn handle(
        &mut self,
        message: TriggerOnSubscriptionMatched,
    ) -> <TriggerOnSubscriptionMatched as Mail>::Result {
        self.listener
            .trigger_on_subscription_matched(message.the_reader, message.status);
    }
}

pub struct TriggerOnRequestedIncompatibleQos {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedIncompatibleQosStatus,
}
impl Mail for TriggerOnRequestedIncompatibleQos {
    type Result = ();
}
impl MailHandler<TriggerOnRequestedIncompatibleQos> for DataReaderListenerActor {
    fn handle(
        &mut self,
        message: TriggerOnRequestedIncompatibleQos,
    ) -> <TriggerOnRequestedIncompatibleQos as Mail>::Result {
        self.listener
            .trigger_on_requested_incompatible_qos(message.the_reader, message.status);
    }
}
