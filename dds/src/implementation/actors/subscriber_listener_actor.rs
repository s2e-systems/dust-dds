use dust_dds_derive::actor_interface;

use crate::{
    dds_async::subscriber::SubscriberAsync,
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{subscriber::Subscriber, subscriber_listener::SubscriberListener},
};

use super::{domain_participant_actor::DomainParticipantActor, subscriber_actor::SubscriberActor};

pub struct SubscriberListenerActor {
    listener: Box<dyn SubscriberListener + Send>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Box<dyn SubscriberListener + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl SubscriberListenerActor {
    async fn trigger_on_data_on_readers(
        &mut self,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_data_on_readers(&Subscriber::new(SubscriberAsync::new(
                    subscriber_address,
                    participant_address,
                    runtime_handle,
                )))
        });
    }

    async fn trigger_on_sample_rejected(&mut self, status: SampleRejectedStatus) {
        tokio::task::block_in_place(|| self.listener.on_sample_rejected(&(), status));
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_requested_incompatible_qos(&(), status));
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_requested_deadline_missed(&(), status));
    }

    async fn trigger_on_subscription_matched(&mut self, status: SubscriptionMatchedStatus) {
        tokio::task::block_in_place(|| self.listener.on_subscription_matched(&(), status));
    }

    async fn trigger_on_sample_lost(&mut self, status: SampleLostStatus) {
        tokio::task::block_in_place(|| self.listener.on_sample_lost(&(), status));
    }
}
