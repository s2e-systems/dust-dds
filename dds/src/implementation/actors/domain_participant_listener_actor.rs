use dust_dds_derive::actor_interface;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::dds::nodes::{DataReaderNode, DataWriterNode},
    infrastructure::status::{
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
};

pub struct DdsDomainParticipantListener {
    listener: Box<dyn DomainParticipantListener + Send>,
}

impl DdsDomainParticipantListener {
    pub fn new(listener: Box<dyn DomainParticipantListener + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DdsDomainParticipantListener {
    async fn trigger_on_sample_rejected(
        &mut self,
        the_reader: DataReaderNode,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_sample_rejected(&the_reader, status));
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_incompatible_qos(&the_reader, status)
        });
    }

    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_offered_incompatible_qos(&the_writer, status)
        });
    }

    async fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_publication_matched(&the_writer, status));
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_deadline_missed(&the_reader, status)
        });
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        the_reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_subscription_matched(&the_reader, status));
    }

    async fn trigger_on_sample_lost(
        &mut self,
        the_reader: DataReaderNode,
        status: SampleLostStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_sample_lost(&the_reader, status));
    }
}
