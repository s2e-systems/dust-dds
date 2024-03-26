use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync},
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

#[actor_interface]
impl SubscriberListenerActor {
    async fn trigger_on_data_on_readers(&mut self, subscriber: SubscriberAsync) {
        if let Some(l) = &mut self.listener {
            l.on_data_on_readers(subscriber).await
        }
    }

    async fn trigger_on_sample_rejected(&mut self, status: SampleRejectedStatus) {
        if let Some(l) = &mut self.listener {
            l.on_sample_rejected(&(), status).await
        }
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        status: RequestedIncompatibleQosStatus,
    ) {
        if let Some(l) = &mut self.listener {
            l.on_requested_incompatible_qos(&(), status).await
        }
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        status: RequestedDeadlineMissedStatus,
    ) {
        if let Some(l) = &mut self.listener {
            l.on_requested_deadline_missed(&(), status).await
        }
    }

    async fn trigger_on_subscription_matched(&mut self, status: SubscriptionMatchedStatus) {
        if let Some(l) = &mut self.listener {
            l.on_subscription_matched(&(), status).await
        }
    }

    async fn trigger_on_sample_lost(&mut self, status: SampleLostStatus) {
        if let Some(l) = &mut self.listener {
            l.on_sample_lost(&(), status).await
        }
    }
}
