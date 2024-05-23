use crate::{
    dds_async::{subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync},
    implementation::actor::{ActorHandler, Mail, MailHandler},
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

pub struct SubscriberListenerActor {
    listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Option<Box<dyn SubscriberListenerAsync + Send>>) -> Self {
        Self { listener }
    }
}

pub enum SubscriberListenerOperation {
    DataOnReaders(SubscriberAsync),
    SampleRejected(SampleRejectedStatus),
    RequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    RequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    SubscriptionMatched(SubscriptionMatchedStatus),
    SampleLost(SampleLostStatus),
}
pub struct CallListenerFunction {
    pub listener_operation: SubscriberListenerOperation,
}
impl Mail for CallListenerFunction {
    type Result = ();
}
impl MailHandler<CallListenerFunction> for SubscriberListenerActor {
    async fn handle(
        &mut self,
        message: CallListenerFunction,
    ) -> <CallListenerFunction as Mail>::Result {
        if let Some(l) = &mut self.listener {
            match message.listener_operation {
                SubscriberListenerOperation::DataOnReaders(subscriber) => {
                    l.on_data_on_readers(subscriber).await
                }
                SubscriberListenerOperation::SampleRejected(status) => {
                    l.on_sample_rejected(&(), status).await
                }
                SubscriberListenerOperation::RequestedIncompatibleQos(status) => {
                    l.on_requested_incompatible_qos(&(), status).await
                }
                SubscriberListenerOperation::RequestedDeadlineMissed(status) => {
                    l.on_requested_deadline_missed(&(), status).await
                }
                SubscriberListenerOperation::SubscriptionMatched(status) => {
                    l.on_subscription_matched(&(), status).await
                }
                SubscriberListenerOperation::SampleLost(status) => {
                    l.on_sample_lost(&(), status).await
                }
            }
        }
    }
}

impl ActorHandler for SubscriberListenerActor {
    type Message = ();

    async fn handle_message(&mut self, _: Self::Message) {}
}
