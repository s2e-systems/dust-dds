use dust_dds_derive::actor_interface;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{dds::dds_data_reader::DdsDataReader, utils::actor::ActorAddress},
    infrastructure::status::{
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    publication::data_writer::DataWriter,
};

use super::{
    data_writer_actor::DataWriterActor, domain_participant_actor::DomainParticipantActor,
    publisher_actor::PublisherActor,
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
    async fn trigger_on_sample_rejected(
        &mut self,
        the_reader: DdsDataReader,
        status: SampleRejectedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_sample_rejected(&the_reader, status));
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        the_reader: DdsDataReader,
        status: RequestedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_incompatible_qos(&the_reader, status)
        });
    }

    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: OfferedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.on_offered_incompatible_qos(
                &DataWriter::<()>::new(writer_address, publisher_address, participant_address),
                status,
            )
        });
    }

    async fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.on_publication_matched(
                &DataWriter::<()>::new(writer_address, publisher_address, participant_address),
                status,
            )
        });
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        the_reader: DdsDataReader,
        status: RequestedDeadlineMissedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener
                .on_requested_deadline_missed(&the_reader, status)
        });
    }

    async fn trigger_on_subscription_matched(
        &mut self,
        the_reader: DdsDataReader,
        status: SubscriptionMatchedStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_subscription_matched(&the_reader, status));
    }

    async fn trigger_on_sample_lost(
        &mut self,
        the_reader: DdsDataReader,
        status: SampleLostStatus,
    ) {
        tokio::task::block_in_place(|| self.listener.on_sample_lost(&the_reader, status));
    }
}
