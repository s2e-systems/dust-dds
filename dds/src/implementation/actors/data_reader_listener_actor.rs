use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::{
    any_data_reader_listener::AnyDataReaderListener, data_reader_actor::DataReaderActor,
    status_condition_actor::StatusConditionActor,
};

pub struct DataReaderListenerActor {
    listener: Option<Box<dyn AnyDataReaderListener + Send>>,
}

impl DataReaderListenerActor {
    pub fn new(listener: Option<Box<dyn AnyDataReaderListener + Send>>) -> Self {
        Self { listener }
    }
}

pub enum DataReaderListenerOperation {
    OnDataAvailable,
    OnSampleRejected(SampleRejectedStatus),
    _OnLivelinessChanged(LivelinessChangedStatus),
    OnRequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    OnRequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    OnSubscriptionMatched(SubscriptionMatchedStatus),
    OnSampleLost(SampleLostStatus),
}

#[actor_interface]
impl DataReaderListenerActor {
    async fn call_listener_function(
        &mut self,
        listener_operation: DataReaderListenerOperation,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) {
        if let Some(l) = &mut self.listener {
            l.call_listener_function(
                listener_operation,
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            )
            .await
        }
    }
}
