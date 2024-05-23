use crate::{
    dds_async::{subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::actor::{ActorAddress, ActorHandler, Mail, MailHandler},
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

pub struct CallListenerFunction {
    pub listener_operation: DataReaderListenerOperation,
    pub reader_address: ActorAddress<DataReaderActor>,
    pub status_condition_address: ActorAddress<StatusConditionActor>,
    pub subscriber: SubscriberAsync,
    pub topic: TopicAsync,
}
impl Mail for CallListenerFunction {
    type Result = ();
}
impl MailHandler<CallListenerFunction> for DataReaderListenerActor {
    async fn handle(
        &mut self,
        message: CallListenerFunction,
    ) -> <CallListenerFunction as Mail>::Result {
        if let Some(l) = &mut self.listener {
            l.call_listener_function(
                message.listener_operation,
                message.reader_address,
                message.status_condition_address,
                message.subscriber,
                message.topic,
            )
            .await
        }
    }
}

impl ActorHandler for DataReaderListenerActor {
    type Message = ();

    async fn handle_message(&mut self, _: Self::Message) -> () {}
}
