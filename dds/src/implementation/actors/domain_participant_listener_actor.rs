use dust_dds_derive::actor_interface;

use crate::{
    dds_async::domain_participant_listener::DomainParticipantListenerAsync,
    infrastructure::status::{
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
};

pub struct DomainParticipantListenerActor {
    listener: Box<dyn DomainParticipantListenerAsync + Send>,
}

impl DomainParticipantListenerActor {
    pub fn new(listener: Box<dyn DomainParticipantListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DomainParticipantListenerActor {
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

    async fn trigger_on_offered_incompatible_qos(&mut self, status: OfferedIncompatibleQosStatus) {
        self.listener.on_offered_incompatible_qos(&(), status).await
    }

    async fn trigger_on_publication_matched(&mut self, status: PublicationMatchedStatus) {
        self.listener.on_publication_matched(&(), status).await
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
