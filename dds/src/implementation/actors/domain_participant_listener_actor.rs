use dust_dds_derive::actor_interface;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::status::{
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
};

pub struct DomainParticipantListenerActor {
    listener: Box<dyn DomainParticipantListener + Send>,
}

impl DomainParticipantListenerActor {
    pub fn new(listener: Box<dyn DomainParticipantListener + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DomainParticipantListenerActor {
    async fn trigger_on_sample_rejected(&mut self, status: SampleRejectedStatus) {
        tokio::task::block_in_place(|| self.listener.on_sample_rejected(&(), status));
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_requested_incompatible_qos(&(), status));
    }

    async fn trigger_on_offered_incompatible_qos(&mut self, status: OfferedIncompatibleQosStatus) {
        tokio::task::block_in_place(|| self.listener.on_offered_incompatible_qos(&(), status));
    }

    async fn trigger_on_publication_matched(&mut self, status: PublicationMatchedStatus) {
        tokio::task::block_in_place(|| self.listener.on_publication_matched(&(), status));
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
