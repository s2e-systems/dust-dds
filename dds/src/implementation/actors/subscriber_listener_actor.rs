use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync},
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

pub struct SubscriberListenerActor {
    listener: Box<dyn SubscriberListenerAsync + Send>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl SubscriberListenerActor {
    async fn trigger_on_data_on_readers(&mut self, subscriber: SubscriberAsync) {
        self.listener.on_data_on_readers(subscriber).await
    }

    async fn trigger_on_sample_rejected(&mut self, status: SampleRejectedStatus) {
        self.listener.on_sample_rejected(&(), status).await
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.listener
            .on_requested_incompatible_qos(&(), status)
            .await
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.listener
            .on_requested_deadline_missed(&(), status)
            .await
    }

    async fn trigger_on_subscription_matched(&mut self, status: SubscriptionMatchedStatus) {
        self.listener.on_subscription_matched(&(), status).await
    }

    async fn trigger_on_sample_lost(&mut self, status: SampleLostStatus) {
        self.listener.on_sample_lost(&(), status).await
    }
}
