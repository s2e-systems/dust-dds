use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_reader_listener::DataReaderListenerAsync, subscriber::SubscriberAsync,
        topic::TopicAsync,
    },
    implementation::{actor::ActorAddress, actors::status_condition_actor::StatusConditionActor},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

pub enum DataReaderListenerOperation {
    DataAvailable,
    SampleRejected(SampleRejectedStatus),
    _LivelinessChanged(LivelinessChangedStatus),
    RequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    RequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    SubscriptionMatched(SubscriptionMatchedStatus),
    SampleLost(SampleLostStatus),
}

pub trait AnyDataReaderListener {
    fn call_listener_function(
        &mut self,
        listener_operation: DataReaderListenerOperation,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

impl<'a, Foo> AnyDataReaderListener for Box<dyn DataReaderListenerAsync<'a, Foo = Foo> + Send + 'a>
where
    Foo: 'a,
{
    fn call_listener_function(
        &mut self,
        listener_operation: DataReaderListenerOperation,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        todo!()
        // Box::pin(async {
        //     let reader =
        //         DataReaderAsync::new(reader_address, status_condition_address, subscriber, topic);
        //     match listener_operation {
        //         DataReaderListenerOperation::DataAvailable => self.on_data_available(reader).await,
        //         DataReaderListenerOperation::SampleRejected(status) => {
        //             self.on_sample_rejected(reader, status).await
        //         }
        //         DataReaderListenerOperation::_LivelinessChanged(status) => {
        //             self.on_liveliness_changed(reader, status).await
        //         }
        //         DataReaderListenerOperation::RequestedDeadlineMissed(status) => {
        //             self.on_requested_deadline_missed(reader, status).await
        //         }
        //         DataReaderListenerOperation::RequestedIncompatibleQos(status) => {
        //             self.on_requested_incompatible_qos(reader, status).await
        //         }
        //         DataReaderListenerOperation::SubscriptionMatched(status) => {
        //             self.on_subscription_matched(reader, status).await
        //         }
        //         DataReaderListenerOperation::SampleLost(status) => {
        //             self.on_sample_lost(reader, status).await
        //         }
        //     }
        // })
    }
}
