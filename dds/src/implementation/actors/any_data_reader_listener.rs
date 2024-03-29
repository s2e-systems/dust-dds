use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync,
        subscriber::SubscriberAsync, topic::TopicAsync,
    },
    implementation::utils::actor::ActorAddress,
};

use super::{
    data_reader_actor::DataReaderActor, data_reader_listener_actor::DataReaderListenerOperation,
    status_condition_actor::StatusConditionActor,
};

pub trait AnyDataReaderListener {
    fn call_listener_function(
        &mut self,
        listener_operation: DataReaderListenerOperation,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

impl<T> AnyDataReaderListener for T
where
    T: DataReaderListenerAsync + Send,
{
    fn call_listener_function(
        &mut self,
        listener_operation: DataReaderListenerOperation,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            let reader =
                DataReaderAsync::new(reader_address, status_condition_address, subscriber, topic);
            match listener_operation {
                DataReaderListenerOperation::OnDataAvailable => {
                    self.on_data_available(reader).await
                }
                DataReaderListenerOperation::OnSampleRejected(status) => {
                    self.on_sample_rejected(reader, status).await
                }
                DataReaderListenerOperation::_OnLivelinessChanged(status) => {
                    self.on_liveliness_changed(reader, status).await
                }
                DataReaderListenerOperation::OnRequestedDeadlineMissed(status) => {
                    self.on_requested_deadline_missed(reader, status).await
                }
                DataReaderListenerOperation::OnRequestedIncompatibleQos(status) => {
                    self.on_requested_incompatible_qos(reader, status).await
                }
                DataReaderListenerOperation::OnSubscriptionMatched(status) => {
                    self.on_subscription_matched(reader, status).await
                }
                DataReaderListenerOperation::OnSampleLost(status) => {
                    self.on_sample_lost(reader, status).await
                }
            }
        })
    }
}
