use crate::{
    dds_async::{subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync},
    implementation::actor::{Mail, MailHandler},
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
    OnDataOnReaders(SubscriberAsync),
    OnSampleRejected(SampleRejectedStatus),
    OnRequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    OnRequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    OnSubscriptionMatched(SubscriptionMatchedStatus),
    OnSampleLost(SampleLostStatus),
}
pub struct CallListenerFunction {
    pub listener_operation: SubscriberListenerOperation,
}
impl Mail for CallListenerFunction {
    type Result = ();
}
impl MailHandler<CallListenerFunction> for SubscriberListenerActor {
    fn handle(
        &mut self,
        message: CallListenerFunction,
    ) -> impl std::future::Future<Output = <CallListenerFunction as Mail>::Result> + Send {
        async move {
            if let Some(l) = &mut self.listener {
                match message.listener_operation {
                    SubscriberListenerOperation::OnDataOnReaders(subscriber) => {
                        l.on_data_on_readers(subscriber).await
                    }
                    SubscriberListenerOperation::OnSampleRejected(status) => {
                        l.on_sample_rejected(&(), status).await
                    }
                    SubscriberListenerOperation::OnRequestedIncompatibleQos(status) => {
                        l.on_requested_incompatible_qos(&(), status).await
                    }
                    SubscriberListenerOperation::OnRequestedDeadlineMissed(status) => {
                        l.on_requested_deadline_missed(&(), status).await
                    }
                    SubscriberListenerOperation::OnSubscriptionMatched(status) => {
                        l.on_subscription_matched(&(), status).await
                    }
                    SubscriberListenerOperation::OnSampleLost(status) => {
                        l.on_sample_lost(&(), status).await
                    }
                }
            }
        }
    }
}
